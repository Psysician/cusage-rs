use clap::{ArgAction, Args, Parser, Subcommand};
use cusage_rs::config::{DataRootOptions, resolve_home_dir};
use cusage_rs::discovery::discover_session_files;
use cusage_rs::domain::UsageEvent;
use cusage_rs::parser::parse_jsonl_files;
use cusage_rs::pricing::{CostMode, PricingCatalog};
use cusage_rs::report::{
    BlocksReport, DailyReport, MonthlyReport, SessionReport, WeeklyReport, build_blocks_report,
    build_daily_report, build_monthly_report, build_session_report, build_statusline_report,
    build_weekly_report, render_blocks_report_json, render_blocks_report_table,
    render_daily_report_json, render_daily_report_table, render_monthly_report_json,
    render_monthly_report_table, render_session_report_json, render_session_report_table,
    render_statusline_report_json, render_statusline_report_line, render_weekly_report_json,
    render_weekly_report_table,
};
use cusage_rs::runtime_config::{
    CommandConfigLayer, load_auto_config_layer, load_custom_config_layer,
};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MILLIS_PER_MINUTE: i64 = 60_000;
const MILLIS_PER_DAY: i64 = 86_400_000;

#[derive(Debug, Parser)]
#[command(
    name = "cusage-rs",
    version,
    about = "Rust rewrite scaffold for ccusage",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
enum Command {
    Daily(ReportArgs),
    Weekly(ReportArgs),
    Monthly(ReportArgs),
    Session(ReportArgs),
    Blocks(ReportArgs),
    Statusline(StatuslineArgs),
}

#[derive(Debug, Args, Clone, PartialEq, Eq, Default)]
struct ReportArgs {
    #[arg(long, value_name = "YYYYMMDD")]
    since: Option<String>,
    #[arg(long, value_name = "YYYYMMDD")]
    until: Option<String>,
    #[arg(long, short = 'j')]
    json: bool,
    #[arg(long, short = 'b')]
    breakdown: bool,
    #[arg(long)]
    compact: bool,
    #[arg(long, short = 'i')]
    instances: bool,
    #[arg(long, short = 'p', value_name = "NAME")]
    project: Option<String>,
    #[arg(long, short = 'z', value_name = "TZ")]
    timezone: Option<String>,
    #[arg(long, short = 'l', value_name = "LOCALE")]
    locale: Option<String>,
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[arg(long, short = 'O', action = ArgAction::SetTrue, overrides_with = "no_offline")]
    offline: bool,
    #[arg(long = "no-offline", action = ArgAction::SetTrue, overrides_with = "offline")]
    no_offline: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq, Default)]
struct StatuslineArgs {
    #[arg(long, short = 'j')]
    json: bool,
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[arg(long, short = 'O', action = ArgAction::SetTrue, overrides_with = "no_offline")]
    offline: bool,
    #[arg(long = "no-offline", action = ArgAction::SetTrue, overrides_with = "offline")]
    no_offline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TableRenderOptions {
    breakdown: bool,
    compact: bool,
    instances: bool,
    locale_decimal_comma: bool,
}

impl TableRenderOptions {
    fn has_custom_behavior(self) -> bool {
        self.breakdown || self.compact || self.locale_decimal_comma || self.instances
    }
}

#[derive(Debug)]
struct PreparedEvents {
    events: Vec<UsageEvent>,
    discovery_warning_count: usize,
    parse_warning_count: usize,
    table_options: TableRenderOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedReportArgs {
    since: Option<String>,
    until: Option<String>,
    json: bool,
    breakdown: bool,
    compact: bool,
    instances: bool,
    project: Option<String>,
    timezone: Option<String>,
    locale: Option<String>,
    offline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedStatuslineArgs {
    json: bool,
    offline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct EnvironmentLayer {
    offline: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedDay {
    year: i32,
    month: u8,
    day: u8,
}

impl ParsedDay {
    fn days_since_epoch(self) -> i64 {
        let mut year = i64::from(self.year);
        let month = i64::from(self.month);
        let day = i64::from(self.day);
        year -= if month <= 2 { 1 } else { 0 };
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let month_prime = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SharedFilters {
    since: Option<ParsedDay>,
    until: Option<ParsedDay>,
    project: Option<String>,
    timezone_offset_minutes: i32,
}

impl SharedFilters {
    fn from_resolved(args: &ResolvedReportArgs) -> Result<Self, String> {
        let since = args.since.as_deref().map(parse_cli_day).transpose()?;
        let until = args.until.as_deref().map(parse_cli_day).transpose()?;
        if let (Some(since), Some(until)) = (since, until)
            && since.days_since_epoch() > until.days_since_epoch()
        {
            return Err("--since must be earlier than or equal to --until".to_owned());
        }

        Ok(Self {
            since,
            until,
            project: normalized_optional_string(args.project.as_deref()),
            timezone_offset_minutes: parse_timezone_offset(args.timezone.as_deref())?,
        })
    }
}

fn main() -> ExitCode {
    run(std::env::args_os())
}

fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    let command = cli.command.unwrap_or(Command::Daily(ReportArgs::default()));
    let output = match describe_command(&command) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    print!("{output}");
    ExitCode::SUCCESS
}

fn describe_command(command: &Command) -> Result<String, String> {
    match command {
        Command::Daily(args) => render_daily_command(args),
        Command::Weekly(args) => render_weekly_command(args),
        Command::Monthly(args) => render_monthly_command(args),
        Command::Session(args) => render_session_command(args),
        Command::Blocks(args) => render_blocks_command(args),
        Command::Statusline(args) => render_statusline_command(args),
    }
}

fn prepare_events(args: &ResolvedReportArgs) -> Result<PreparedEvents, String> {
    let filters = SharedFilters::from_resolved(args)?;
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let filtered = apply_event_filters(&parsed.events, &filters);
    let events = shift_events_for_timezone(filtered, filters.timezone_offset_minutes);
    let locale_decimal_comma = args
        .locale
        .as_deref()
        .is_some_and(locale_uses_decimal_comma);

    Ok(PreparedEvents {
        events,
        discovery_warning_count: discovered.warnings.len(),
        parse_warning_count: parsed.warnings.len(),
        table_options: TableRenderOptions {
            breakdown: args.breakdown,
            compact: args.compact,
            instances: args.instances,
            locale_decimal_comma,
        },
    })
}

fn render_daily_command(args: &ReportArgs) -> Result<String, String> {
    let args = resolve_report_args("daily", args)?;
    let prepared = prepare_events(&args)?;
    let pricing = PricingCatalog::default_claude_catalog();
    let report = build_daily_report(&prepared.events, CostMode::Auto, &pricing);

    if args.json {
        Ok(render_daily_report_json(
            &report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        ))
    } else {
        Ok(render_daily_table(&report, &prepared))
    }
}

fn render_weekly_command(args: &ReportArgs) -> Result<String, String> {
    let args = resolve_report_args("weekly", args)?;
    let prepared = prepare_events(&args)?;
    let pricing = PricingCatalog::default_claude_catalog();
    let report = build_weekly_report(&prepared.events, CostMode::Auto, &pricing);

    if args.json {
        Ok(render_weekly_report_json(
            &report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        ))
    } else {
        Ok(render_weekly_table(&report, &prepared))
    }
}

fn render_monthly_command(args: &ReportArgs) -> Result<String, String> {
    let args = resolve_report_args("monthly", args)?;
    let prepared = prepare_events(&args)?;
    let pricing = PricingCatalog::default_claude_catalog();
    let report = build_monthly_report(&prepared.events, CostMode::Auto, &pricing);

    if args.json {
        Ok(render_monthly_report_json(
            &report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        ))
    } else {
        Ok(render_monthly_table(&report, &prepared))
    }
}

fn render_session_command(args: &ReportArgs) -> Result<String, String> {
    let args = resolve_report_args("session", args)?;
    let prepared = prepare_events(&args)?;
    let pricing = PricingCatalog::default_claude_catalog();
    let report = build_session_report(&prepared.events, CostMode::Auto, &pricing);

    if args.json {
        Ok(render_session_report_json(
            &report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        ))
    } else {
        Ok(render_session_table(&report, &prepared))
    }
}

fn render_blocks_command(args: &ReportArgs) -> Result<String, String> {
    let args = resolve_report_args("blocks", args)?;
    let prepared = prepare_events(&args)?;
    let pricing = PricingCatalog::default_claude_catalog();
    let report = build_blocks_report(&prepared.events, CostMode::Auto, &pricing);

    if args.json {
        Ok(render_blocks_report_json(
            &report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        ))
    } else {
        Ok(render_blocks_table(&report, &prepared))
    }
}

fn render_statusline_command(args: &StatuslineArgs) -> Result<String, String> {
    let args = resolve_statusline_args(args)?;
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let pricing = PricingCatalog::default_claude_catalog();
    let report = build_statusline_report(&parsed.events, CostMode::Auto, &pricing);

    if args.json {
        Ok(render_statusline_report_json(
            &report,
            discovered.warnings.len(),
            parsed.warnings.len(),
        ))
    } else {
        Ok(render_statusline_report_line(&report))
    }
}

fn resolve_report_args(command: &str, args: &ReportArgs) -> Result<ResolvedReportArgs, String> {
    let cwd = std::env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    let home_dir = resolve_home_dir();
    let environment = read_environment_layer();
    resolve_report_args_with_context(command, args, &cwd, home_dir.as_deref(), environment)
}

fn resolve_statusline_args(args: &StatuslineArgs) -> Result<ResolvedStatuslineArgs, String> {
    let cwd = std::env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    let home_dir = resolve_home_dir();
    let environment = read_environment_layer();
    resolve_statusline_args_with_context(args, &cwd, home_dir.as_deref(), environment)
}

fn resolve_report_args_with_context(
    command: &str,
    args: &ReportArgs,
    cwd: &Path,
    home_dir: Option<&Path>,
    environment: EnvironmentLayer,
) -> Result<ResolvedReportArgs, String> {
    let mut layer = load_auto_config_layer(command, cwd, home_dir)?;
    apply_environment_layer(&mut layer, environment);
    if let Some(config_path) = resolve_custom_config_path(args.config.as_deref(), cwd, home_dir)? {
        let custom_layer = load_custom_config_layer(command, &config_path)?;
        layer.merge_from(&custom_layer);
    }

    let offline = cli_offline_override(args.offline, args.no_offline)
        .or(layer.offline)
        .unwrap_or(false);

    Ok(ResolvedReportArgs {
        since: args.since.clone().or(layer.since),
        until: args.until.clone().or(layer.until),
        json: args.json || layer.json.unwrap_or(false),
        breakdown: args.breakdown || layer.breakdown.unwrap_or(false),
        compact: args.compact || layer.compact.unwrap_or(false),
        instances: args.instances || layer.instances.unwrap_or(false),
        project: args.project.clone().or(layer.project),
        timezone: args.timezone.clone().or(layer.timezone),
        locale: args.locale.clone().or(layer.locale),
        offline,
    })
}

fn resolve_statusline_args_with_context(
    args: &StatuslineArgs,
    cwd: &Path,
    home_dir: Option<&Path>,
    environment: EnvironmentLayer,
) -> Result<ResolvedStatuslineArgs, String> {
    let mut layer = load_auto_config_layer("statusline", cwd, home_dir)?;
    apply_environment_layer(&mut layer, environment);
    if let Some(config_path) = resolve_custom_config_path(args.config.as_deref(), cwd, home_dir)? {
        let custom_layer = load_custom_config_layer("statusline", &config_path)?;
        layer.merge_from(&custom_layer);
    }

    let offline = cli_offline_override(args.offline, args.no_offline)
        .or(layer.offline)
        .unwrap_or(false);

    Ok(ResolvedStatuslineArgs {
        json: args.json || layer.json.unwrap_or(false),
        offline,
    })
}

fn resolve_custom_config_path(
    path: Option<&Path>,
    cwd: &Path,
    home_dir: Option<&Path>,
) -> Result<Option<PathBuf>, String> {
    let Some(path) = path else {
        return Ok(None);
    };

    if path.as_os_str().is_empty() {
        return Err("--config path cannot be empty".to_owned());
    }

    let expanded = expand_home_path(path, home_dir);
    if expanded.as_os_str().is_empty() {
        return Err("--config path cannot be empty".to_owned());
    }
    if expanded.is_absolute() {
        return Ok(Some(expanded));
    }
    Ok(Some(cwd.join(expanded)))
}

fn expand_home_path(path: &Path, home_dir: Option<&Path>) -> PathBuf {
    let rendered = path.to_string_lossy();
    if rendered == "~" {
        return home_dir
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf());
    }
    if let Some(stripped) = rendered.strip_prefix("~/")
        && let Some(home_dir) = home_dir
    {
        return home_dir.join(stripped);
    }
    path.to_path_buf()
}

fn cli_offline_override(offline: bool, no_offline: bool) -> Option<bool> {
    if offline {
        Some(true)
    } else if no_offline {
        Some(false)
    } else {
        None
    }
}

fn apply_environment_layer(layer: &mut CommandConfigLayer, environment: EnvironmentLayer) {
    if environment.offline.is_some() {
        layer.offline = environment.offline;
    }
}

fn read_environment_layer() -> EnvironmentLayer {
    EnvironmentLayer {
        offline: parse_env_bool(std::env::var_os("CCUSAGE_OFFLINE").as_deref()),
    }
}

fn parse_env_bool(raw: Option<&std::ffi::OsStr>) -> Option<bool> {
    let raw = raw?.to_str()?.trim().to_ascii_lowercase();
    match raw.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn render_daily_table(report: &DailyReport, prepared: &PreparedEvents) -> String {
    if !prepared.table_options.has_custom_behavior() {
        return render_daily_report_table(
            report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        );
    }

    let options = prepared.table_options;
    let instance_count = options
        .instances
        .then(|| distinct_instance_count(&prepared.events));
    let mut lines = Vec::new();

    if options.compact {
        lines.push(if options.breakdown {
            "DATE ENTRIES TOKENS COST_USD RAW CALC MISSING".to_owned()
        } else {
            "DATE ENTRIES TOKENS COST_USD".to_owned()
        });
        for day in &report.days {
            let mut line = format!(
                "{} {} {} {}",
                day.date,
                day.entries,
                day.total_tokens,
                format_money_for_table(day.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {} {} {}",
                    day.entries_with_raw_cost,
                    day.entries_with_calculated_cost,
                    day.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL {} {} {}",
            report.totals.entries,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {} {} {}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    } else {
        lines.push(if options.breakdown {
            "DATE       ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD RAW CALC MISSING"
                .to_owned()
        } else {
            "DATE       ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD".to_owned()
        });
        for day in &report.days {
            let mut line = format!(
                "{} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
                day.date,
                day.entries,
                day.input_tokens,
                day.output_tokens,
                day.cache_creation_input_tokens,
                day.cache_read_input_tokens,
                day.total_tokens,
                format_money_for_table(day.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {:>3} {:>4} {:>7}",
                    day.entries_with_raw_cost,
                    day.entries_with_calculated_cost,
                    day.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL      {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            report.totals.entries,
            report.totals.input_tokens,
            report.totals.output_tokens,
            report.totals.cache_creation_input_tokens,
            report.totals.cache_read_input_tokens,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {:>3} {:>4} {:>7}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    }

    finish_custom_table(
        lines,
        prepared.discovery_warning_count,
        prepared.parse_warning_count,
        instance_count,
    )
}

fn render_weekly_table(report: &WeeklyReport, prepared: &PreparedEvents) -> String {
    if !prepared.table_options.has_custom_behavior() {
        return render_weekly_report_table(
            report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        );
    }

    let options = prepared.table_options;
    let instance_count = options
        .instances
        .then(|| distinct_instance_count(&prepared.events));
    let mut lines = Vec::new();

    if options.compact {
        lines.push(if options.breakdown {
            "WEEK_START WEEK_END ENTRIES TOKENS COST_USD RAW CALC MISSING".to_owned()
        } else {
            "WEEK_START WEEK_END ENTRIES TOKENS COST_USD".to_owned()
        });
        for week in &report.weeks {
            let mut line = format!(
                "{} {} {} {} {}",
                week.week_start,
                week.week_end,
                week.entries,
                week.total_tokens,
                format_money_for_table(week.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {} {} {}",
                    week.entries_with_raw_cost,
                    week.entries_with_calculated_cost,
                    week.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL {} {} {}",
            report.totals.entries,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {} {} {}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    } else {
        lines.push(if options.breakdown {
            "WEEK_START WEEK_END   ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD RAW CALC MISSING"
                .to_owned()
        } else {
            "WEEK_START WEEK_END   ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD"
                .to_owned()
        });
        for week in &report.weeks {
            let mut line = format!(
                "{} {} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
                week.week_start,
                week.week_end,
                week.entries,
                week.input_tokens,
                week.output_tokens,
                week.cache_creation_input_tokens,
                week.cache_read_input_tokens,
                week.total_tokens,
                format_money_for_table(week.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {:>3} {:>4} {:>7}",
                    week.entries_with_raw_cost,
                    week.entries_with_calculated_cost,
                    week.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL                {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            report.totals.entries,
            report.totals.input_tokens,
            report.totals.output_tokens,
            report.totals.cache_creation_input_tokens,
            report.totals.cache_read_input_tokens,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {:>3} {:>4} {:>7}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    }

    finish_custom_table(
        lines,
        prepared.discovery_warning_count,
        prepared.parse_warning_count,
        instance_count,
    )
}

fn render_monthly_table(report: &MonthlyReport, prepared: &PreparedEvents) -> String {
    if !prepared.table_options.has_custom_behavior() {
        return render_monthly_report_table(
            report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        );
    }

    let options = prepared.table_options;
    let instance_count = options
        .instances
        .then(|| distinct_instance_count(&prepared.events));
    let mut lines = Vec::new();

    if options.compact {
        lines.push(if options.breakdown {
            "MONTH ENTRIES TOKENS COST_USD RAW CALC MISSING".to_owned()
        } else {
            "MONTH ENTRIES TOKENS COST_USD".to_owned()
        });
        for month in &report.months {
            let mut line = format!(
                "{} {} {} {}",
                month.month,
                month.entries,
                month.total_tokens,
                format_money_for_table(month.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {} {} {}",
                    month.entries_with_raw_cost,
                    month.entries_with_calculated_cost,
                    month.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL {} {} {}",
            report.totals.entries,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {} {} {}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    } else {
        lines.push(if options.breakdown {
            "MONTH    ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD RAW CALC MISSING"
                .to_owned()
        } else {
            "MONTH    ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD".to_owned()
        });
        for month in &report.months {
            let mut line = format!(
                "{} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
                month.month,
                month.entries,
                month.input_tokens,
                month.output_tokens,
                month.cache_creation_input_tokens,
                month.cache_read_input_tokens,
                month.total_tokens,
                format_money_for_table(month.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {:>3} {:>4} {:>7}",
                    month.entries_with_raw_cost,
                    month.entries_with_calculated_cost,
                    month.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL    {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            report.totals.entries,
            report.totals.input_tokens,
            report.totals.output_tokens,
            report.totals.cache_creation_input_tokens,
            report.totals.cache_read_input_tokens,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {:>3} {:>4} {:>7}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    }

    finish_custom_table(
        lines,
        prepared.discovery_warning_count,
        prepared.parse_warning_count,
        instance_count,
    )
}

fn render_session_table(report: &SessionReport, prepared: &PreparedEvents) -> String {
    if !prepared.table_options.has_custom_behavior() {
        return render_session_report_table(
            report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        );
    }

    let options = prepared.table_options;
    let instance_count = options
        .instances
        .then(|| distinct_instance_count(&prepared.events));
    let mut lines = Vec::new();

    if options.compact {
        lines.push(if options.breakdown {
            "SESSION_ID PROJECT ENTRIES TOKENS COST_USD RAW CALC MISSING".to_owned()
        } else {
            "SESSION_ID PROJECT ENTRIES TOKENS COST_USD".to_owned()
        });
        for session in &report.sessions {
            let mut line = format!(
                "{} {} {} {} {}",
                session.session_id.as_deref().unwrap_or("-"),
                session.project.as_deref().unwrap_or("-"),
                session.entries,
                session.total_tokens,
                format_money_for_table(session.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {} {} {}",
                    session.entries_with_raw_cost,
                    session.entries_with_calculated_cost,
                    session.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL {} {}",
            report.totals.entries,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {} {} {}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    } else {
        lines.push(if options.breakdown {
            "SESSION_ID          PROJECT             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD RAW CALC MISSING"
                .to_owned()
        } else {
            "SESSION_ID          PROJECT             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD"
                .to_owned()
        });
        for session in &report.sessions {
            let mut line = format!(
                "{:<19} {:<19} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
                session.session_id.as_deref().unwrap_or("-"),
                session.project.as_deref().unwrap_or("-"),
                session.entries,
                session.input_tokens,
                session.output_tokens,
                session.cache_creation_input_tokens,
                session.cache_read_input_tokens,
                session.total_tokens,
                format_money_for_table(session.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {:>3} {:>4} {:>7}",
                    session.entries_with_raw_cost,
                    session.entries_with_calculated_cost,
                    session.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL                                  {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            report.totals.entries,
            report.totals.input_tokens,
            report.totals.output_tokens,
            report.totals.cache_creation_input_tokens,
            report.totals.cache_read_input_tokens,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {:>3} {:>4} {:>7}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    }

    finish_custom_table(
        lines,
        prepared.discovery_warning_count,
        prepared.parse_warning_count,
        instance_count,
    )
}

fn render_blocks_table(report: &BlocksReport, prepared: &PreparedEvents) -> String {
    if !prepared.table_options.has_custom_behavior() {
        return render_blocks_report_table(
            report,
            prepared.discovery_warning_count,
            prepared.parse_warning_count,
        );
    }

    let options = prepared.table_options;
    let instance_count = options
        .instances
        .then(|| distinct_instance_count(&prepared.events));
    let mut lines = Vec::new();

    if options.compact {
        lines.push(if options.breakdown {
            "BLOCK_START BLOCK_END ENTRIES TOKENS COST_USD RAW CALC MISSING".to_owned()
        } else {
            "BLOCK_START BLOCK_END ENTRIES TOKENS COST_USD".to_owned()
        });
        for block in &report.blocks {
            let mut line = format!(
                "{} {} {} {} {}",
                block.block_start,
                block.block_end,
                block.entries,
                block.total_tokens,
                format_money_for_table(block.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {} {} {}",
                    block.entries_with_raw_cost,
                    block.entries_with_calculated_cost,
                    block.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL {} {} {}",
            report.totals.entries,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {} {} {}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    } else {
        lines.push(if options.breakdown {
            "BLOCK_START           BLOCK_END             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD RAW CALC MISSING"
                .to_owned()
        } else {
            "BLOCK_START           BLOCK_END             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD"
                .to_owned()
        });
        for block in &report.blocks {
            let mut line = format!(
                "{} {} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
                block.block_start,
                block.block_end,
                block.entries,
                block.input_tokens,
                block.output_tokens,
                block.cache_creation_input_tokens,
                block.cache_read_input_tokens,
                block.total_tokens,
                format_money_for_table(block.total_cost_usd, options.locale_decimal_comma)
            );
            if options.breakdown {
                line.push_str(&format!(
                    " {:>3} {:>4} {:>7}",
                    block.entries_with_raw_cost,
                    block.entries_with_calculated_cost,
                    block.entries_with_missing_cost
                ));
            }
            lines.push(line);
        }
        let mut totals = format!(
            "TOTAL                                    {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            report.totals.entries,
            report.totals.input_tokens,
            report.totals.output_tokens,
            report.totals.cache_creation_input_tokens,
            report.totals.cache_read_input_tokens,
            report.totals.total_tokens,
            format_money_for_table(report.totals.total_cost_usd, options.locale_decimal_comma)
        );
        if options.breakdown {
            totals.push_str(&format!(
                " {:>3} {:>4} {:>7}",
                report.totals.entries_with_raw_cost,
                report.totals.entries_with_calculated_cost,
                report.totals.entries_with_missing_cost
            ));
        }
        lines.push(totals);
    }

    finish_custom_table(
        lines,
        prepared.discovery_warning_count,
        prepared.parse_warning_count,
        instance_count,
    )
}

fn finish_custom_table(
    mut lines: Vec<String>,
    discovery_warning_count: usize,
    parse_warning_count: usize,
    instance_count: Option<usize>,
) -> String {
    if let Some(instance_count) = instance_count {
        lines.push(format!("INSTANCES distinct={instance_count}"));
    }
    lines.push(format!(
        "WARNINGS discovery={} parse={}",
        discovery_warning_count, parse_warning_count
    ));
    lines.join("\n") + "\n"
}

fn distinct_instance_count(events: &[UsageEvent]) -> usize {
    let mut instances = BTreeSet::new();
    for event in events {
        let key = if let Some(session_id) = normalized_optional_string(event.session_id.as_deref())
        {
            format!("session:{session_id}")
        } else if let Some(project) = normalized_optional_string(event.project.as_deref()) {
            format!("project:{project}")
        } else {
            format!(
                "origin:{}:{}",
                event.origin.file.display(),
                event.origin.line_number
            )
        };
        instances.insert(key);
    }
    instances.len()
}

fn format_money_for_table(value: f64, decimal_comma: bool) -> String {
    let mut normalized = if value.is_finite() { value } else { 0.0 };
    if normalized.abs() < 0.000_000_000_001 {
        normalized = 0.0;
    }
    let mut rendered = format!("{normalized:.6}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.push('0');
    }
    if decimal_comma {
        rendered = rendered.replace('.', ",");
    }
    rendered
}

fn locale_uses_decimal_comma(locale: &str) -> bool {
    let primary = locale
        .trim()
        .split(['-', '_', '.'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        primary.as_str(),
        "de" | "fr"
            | "es"
            | "it"
            | "pt"
            | "nl"
            | "tr"
            | "ru"
            | "pl"
            | "cs"
            | "sk"
            | "sl"
            | "hu"
            | "ro"
            | "bg"
            | "uk"
    )
}

fn apply_event_filters(events: &[UsageEvent], filters: &SharedFilters) -> Vec<UsageEvent> {
    let since_ms = filters
        .since
        .map(|since| day_start_unix_ms(since, filters.timezone_offset_minutes));
    let until_ms = filters
        .until
        .map(|until| day_end_unix_ms(until, filters.timezone_offset_minutes));

    events
        .iter()
        .filter(|event| {
            if let Some(since_ms) = since_ms
                && event.occurred_at_unix_ms < since_ms
            {
                return false;
            }
            if let Some(until_ms) = until_ms
                && event.occurred_at_unix_ms > until_ms
            {
                return false;
            }
            if let Some(expected_project) = filters.project.as_deref()
                && normalized_optional_string(event.project.as_deref()).as_deref()
                    != Some(expected_project)
            {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

fn shift_events_for_timezone(
    mut events: Vec<UsageEvent>,
    timezone_offset_minutes: i32,
) -> Vec<UsageEvent> {
    if timezone_offset_minutes == 0 {
        return events;
    }
    let shift_ms = i64::from(timezone_offset_minutes).saturating_mul(MILLIS_PER_MINUTE);
    for event in &mut events {
        event.occurred_at_unix_ms = event.occurred_at_unix_ms.saturating_add(shift_ms);
    }
    events
}

fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn parse_cli_day(raw: &str) -> Result<ParsedDay, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("date value cannot be empty".to_owned());
    }

    let digits: String = trimmed
        .chars()
        .filter(|character| *character != '-')
        .collect();
    if digits.len() != 8 || !digits.chars().all(|character| character.is_ascii_digit()) {
        return Err(format!(
            "invalid date '{trimmed}': expected YYYYMMDD or YYYY-MM-DD"
        ));
    }

    let year = digits[0..4]
        .parse::<i32>()
        .map_err(|_| format!("invalid date '{trimmed}': year out of range"))?;
    let month = digits[4..6]
        .parse::<u8>()
        .map_err(|_| format!("invalid date '{trimmed}': month out of range"))?;
    let day = digits[6..8]
        .parse::<u8>()
        .map_err(|_| format!("invalid date '{trimmed}': day out of range"))?;

    if !(1..=12).contains(&month) {
        return Err(format!(
            "invalid date '{trimmed}': month must be between 1 and 12"
        ));
    }
    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        return Err(format!(
            "invalid date '{trimmed}': day must be between 1 and {max_day}"
        ));
    }

    Ok(ParsedDay { year, month, day })
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn day_start_unix_ms(day: ParsedDay, timezone_offset_minutes: i32) -> i64 {
    day.days_since_epoch()
        .saturating_mul(MILLIS_PER_DAY)
        .saturating_sub(i64::from(timezone_offset_minutes).saturating_mul(MILLIS_PER_MINUTE))
}

fn day_end_unix_ms(day: ParsedDay, timezone_offset_minutes: i32) -> i64 {
    day_start_unix_ms(day, timezone_offset_minutes)
        .saturating_add(MILLIS_PER_DAY)
        .saturating_sub(1)
}

fn parse_timezone_offset(raw: Option<&str>) -> Result<i32, String> {
    let Some(raw) = raw else {
        return Ok(0);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("timezone value cannot be empty".to_owned());
    }

    if trimmed.eq_ignore_ascii_case("utc")
        || trimmed.eq_ignore_ascii_case("z")
        || trimmed.eq_ignore_ascii_case("gmt")
    {
        return Ok(0);
    }

    if let Some(stripped) = trimmed
        .strip_prefix("UTC")
        .or_else(|| trimmed.strip_prefix("utc"))
    {
        return parse_signed_timezone_offset(stripped)
            .ok_or_else(|| format!("unsupported timezone '{trimmed}'"));
    }
    if let Some(stripped) = trimmed
        .strip_prefix("GMT")
        .or_else(|| trimmed.strip_prefix("gmt"))
    {
        return parse_signed_timezone_offset(stripped)
            .ok_or_else(|| format!("unsupported timezone '{trimmed}'"));
    }

    parse_signed_timezone_offset(trimmed).ok_or_else(|| format!("unsupported timezone '{trimmed}'"))
}

fn parse_signed_timezone_offset(raw: &str) -> Option<i32> {
    let trimmed = raw.trim();
    let (sign, rest) = match trimmed.chars().next()? {
        '+' => (1_i32, &trimmed[1..]),
        '-' => (-1_i32, &trimmed[1..]),
        _ => return None,
    };
    if rest.is_empty() {
        return None;
    }

    let (hours, minutes) = if let Some((hours, minutes)) = rest.split_once(':') {
        (parse_u8(hours)?, parse_u8(minutes)?)
    } else if rest.len() <= 2 {
        (parse_u8(rest)?, 0)
    } else if rest.len() == 4 {
        (parse_u8(&rest[0..2])?, parse_u8(&rest[2..4])?)
    } else {
        return None;
    };

    if hours > 23 || minutes > 59 {
        return None;
    }

    Some(
        sign.saturating_mul(
            i32::from(hours)
                .saturating_mul(60)
                .saturating_add(i32::from(minutes)),
        ),
    )
}

fn parse_u8(raw: &str) -> Option<u8> {
    if raw.is_empty() || !raw.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    raw.parse::<u8>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cusage_rs::domain::{EventKind, EventOrigin, TokenUsage, UsageSpeed};
    use cusage_rs::report::{DailyReportDay, DailyReportTotals};
    use std::ffi::OsStr;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn defaults_to_daily_command() {
        let cli = Cli::parse_from(["cusage-rs"]);
        let command = cli.command.unwrap_or(Command::Daily(ReportArgs::default()));
        assert!(matches!(command, Command::Daily(_)));
    }

    #[test]
    fn parses_monthly_command() {
        let cli = Cli::parse_from(["cusage-rs", "monthly", "--json"]);
        let command = cli.command.expect("expected parsed subcommand");
        assert!(matches!(
            command,
            Command::Monthly(ReportArgs { json: true, .. })
        ));
    }

    #[test]
    fn parses_weekly_command() {
        let cli = Cli::parse_from(["cusage-rs", "weekly", "--json"]);
        let command = cli.command.expect("expected parsed subcommand");
        assert!(matches!(
            command,
            Command::Weekly(ReportArgs { json: true, .. })
        ));
    }

    #[test]
    fn parses_session_command() {
        let cli = Cli::parse_from(["cusage-rs", "session", "--json"]);
        let command = cli.command.expect("expected parsed subcommand");
        assert!(matches!(
            command,
            Command::Session(ReportArgs { json: true, .. })
        ));
    }

    #[test]
    fn parses_blocks_command() {
        let cli = Cli::parse_from(["cusage-rs", "blocks", "--json"]);
        let command = cli.command.expect("expected parsed subcommand");
        assert!(matches!(
            command,
            Command::Blocks(ReportArgs { json: true, .. })
        ));
    }

    #[test]
    fn parses_statusline_command() {
        let cli = Cli::parse_from(["cusage-rs", "statusline", "--json"]);
        let command = cli.command.expect("expected parsed subcommand");
        assert!(matches!(
            command,
            Command::Statusline(StatuslineArgs { json: true, .. })
        ));
    }

    #[test]
    fn parses_short_aliases_for_shared_flags() {
        let cli = Cli::parse_from([
            "cusage-rs",
            "daily",
            "-j",
            "-b",
            "--compact",
            "-i",
            "-p",
            "demo",
            "-z",
            "+02:30",
            "-l",
            "de-DE",
            "--since",
            "20260310",
            "--until",
            "20260312",
        ]);
        let command = cli.command.expect("expected parsed subcommand");
        let Command::Daily(args) = command else {
            panic!("expected daily command");
        };
        assert!(args.json);
        assert!(args.breakdown);
        assert!(args.compact);
        assert!(args.instances);
        assert_eq!(args.project.as_deref(), Some("demo"));
        assert_eq!(args.timezone.as_deref(), Some("+02:30"));
        assert_eq!(args.locale.as_deref(), Some("de-DE"));
        assert_eq!(args.since.as_deref(), Some("20260310"));
        assert_eq!(args.until.as_deref(), Some("20260312"));
    }

    #[test]
    fn parses_offline_flag_and_no_offline_override() {
        let cli = Cli::parse_from(["cusage-rs", "daily", "-O"]);
        let command = cli.command.expect("expected parsed subcommand");
        let Command::Daily(args) = command else {
            panic!("expected daily command");
        };
        assert!(args.offline);
        assert!(!args.no_offline);

        let cli = Cli::parse_from(["cusage-rs", "daily", "--no-offline"]);
        let command = cli.command.expect("expected parsed subcommand");
        let Command::Daily(args) = command else {
            panic!("expected daily command");
        };
        assert!(!args.offline);
        assert!(args.no_offline);
    }

    #[test]
    fn parses_cli_days_in_compact_or_hyphenated_form() {
        assert_eq!(
            parse_cli_day("20260310").expect("expected compact date to parse"),
            ParsedDay {
                year: 2026,
                month: 3,
                day: 10
            }
        );
        assert_eq!(
            parse_cli_day("2026-03-10").expect("expected hyphenated date to parse"),
            ParsedDay {
                year: 2026,
                month: 3,
                day: 10
            }
        );
        assert!(parse_cli_day("2026-02-30").is_err());
        assert!(parse_cli_day("2026/03/10").is_err());
    }

    #[test]
    fn parses_timezone_offsets() {
        assert_eq!(parse_timezone_offset(None).expect("default timezone"), 0);
        assert_eq!(parse_timezone_offset(Some("UTC")).expect("utc timezone"), 0);
        assert_eq!(
            parse_timezone_offset(Some("+09")).expect("short positive offset"),
            540
        );
        assert_eq!(
            parse_timezone_offset(Some("-0530")).expect("long negative offset"),
            -330
        );
        assert_eq!(
            parse_timezone_offset(Some("+02:30")).expect("colon offset"),
            150
        );
        assert_eq!(
            parse_timezone_offset(Some("UTC-07:00")).expect("prefixed offset"),
            -420
        );
        assert!(parse_timezone_offset(Some("Europe/Berlin")).is_err());
        assert!(parse_timezone_offset(Some("+24:00")).is_err());
    }

    #[test]
    fn rejects_inverted_since_until_range() {
        let args = ResolvedReportArgs {
            since: Some("20260312".to_owned()),
            until: Some("20260310".to_owned()),
            json: false,
            breakdown: false,
            compact: false,
            instances: false,
            project: None,
            timezone: None,
            locale: None,
            offline: false,
        };
        let error = SharedFilters::from_resolved(&args).expect_err("expected invalid range");
        assert!(error.contains("--since"));
    }

    #[test]
    fn applies_since_until_and_project_filters_inclusive() {
        let args = ResolvedReportArgs {
            since: Some("20260310".to_owned()),
            until: Some("20260310".to_owned()),
            json: false,
            breakdown: false,
            compact: false,
            instances: false,
            project: Some("alpha".to_owned()),
            timezone: Some("+02:00".to_owned()),
            locale: None,
            offline: false,
        };
        let filters = SharedFilters::from_resolved(&args).expect("expected valid shared filters");
        let day = parse_cli_day("20260310").expect("expected day parse");
        let start = day_start_unix_ms(day, filters.timezone_offset_minutes);
        let end = day_end_unix_ms(day, filters.timezone_offset_minutes);

        let events = vec![
            test_event(start.saturating_sub(1), Some("s-out-low"), Some("alpha")),
            test_event(start, Some("s-in-start"), Some("alpha")),
            test_event(
                start.saturating_add(10),
                Some("s-wrong-project"),
                Some("beta"),
            ),
            test_event(end, Some("s-in-end"), Some("alpha")),
            test_event(end.saturating_add(1), Some("s-out-high"), Some("alpha")),
        ];

        let filtered = apply_event_filters(&events, &filters);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].session_id.as_deref(), Some("s-in-start"));
        assert_eq!(filtered[1].session_id.as_deref(), Some("s-in-end"));
    }

    #[test]
    fn shifts_event_timestamps_by_timezone_offset() {
        let events = vec![test_event(1_000, Some("s1"), Some("alpha"))];
        let shifted = shift_events_for_timezone(events, 120);
        assert_eq!(shifted[0].occurred_at_unix_ms, 7_201_000);
    }

    #[test]
    fn custom_daily_table_includes_breakdown_instances_and_locale_formatting() {
        let report = DailyReport {
            days: vec![DailyReportDay {
                date: "2026-03-10".to_owned(),
                entries: 2,
                input_tokens: 5,
                output_tokens: 6,
                cache_creation_input_tokens: 1,
                cache_read_input_tokens: 2,
                total_tokens: 14,
                total_cost_usd: 12.5,
                entries_with_raw_cost: 1,
                entries_with_calculated_cost: 1,
                entries_with_missing_cost: 0,
            }],
            totals: DailyReportTotals {
                days: 1,
                entries: 2,
                input_tokens: 5,
                output_tokens: 6,
                cache_creation_input_tokens: 1,
                cache_read_input_tokens: 2,
                total_tokens: 14,
                total_cost_usd: 12.5,
                entries_with_raw_cost: 1,
                entries_with_calculated_cost: 1,
                entries_with_missing_cost: 0,
            },
        };
        let prepared = PreparedEvents {
            events: vec![
                test_event(1_000, Some("session-a"), Some("alpha")),
                test_event(2_000, Some("session-b"), Some("alpha")),
            ],
            discovery_warning_count: 0,
            parse_warning_count: 0,
            table_options: TableRenderOptions {
                breakdown: true,
                compact: true,
                instances: true,
                locale_decimal_comma: true,
            },
        };

        let rendered = render_daily_table(&report, &prepared);
        assert!(rendered.contains("RAW CALC MISSING"));
        assert!(rendered.contains("12,5"));
        assert!(rendered.contains("INSTANCES distinct=2"));
    }

    #[test]
    fn env_offline_overrides_local_config_and_cli_can_disable_it() {
        let test_dir = TestDir::new();
        let home_dir = test_dir.path().join("home");
        let cwd = test_dir.path().join("workspace");
        create_dir_all(cwd.join(".ccusage")).expect("failed to create local config dir");
        create_dir_all(&home_dir).expect("failed to create home dir");
        write(
            cwd.join(".ccusage/ccusage.json"),
            r#"{"commands":{"daily":{"offline":false}}}"#,
        )
        .expect("failed to write local config");

        let args = ReportArgs::default();
        let resolved = resolve_report_args_with_context(
            "daily",
            &args,
            &cwd,
            Some(&home_dir),
            EnvironmentLayer {
                offline: Some(true),
            },
        )
        .expect("expected resolved args");
        assert!(resolved.offline);

        let args = ReportArgs {
            no_offline: true,
            ..ReportArgs::default()
        };
        let resolved = resolve_report_args_with_context(
            "daily",
            &args,
            &cwd,
            Some(&home_dir),
            EnvironmentLayer {
                offline: Some(true),
            },
        )
        .expect("expected resolved args");
        assert!(!resolved.offline);
    }

    #[test]
    fn custom_config_overrides_environment_for_offline() {
        let test_dir = TestDir::new();
        let home_dir = test_dir.path().join("home");
        let cwd = test_dir.path().join("workspace");
        let custom_path = test_dir.path().join("custom-config.json");
        create_dir_all(cwd.join(".ccusage")).expect("failed to create local config dir");
        create_dir_all(&home_dir).expect("failed to create home dir");
        write(
            cwd.join(".ccusage/ccusage.json"),
            r#"{"commands":{"daily":{"offline":true}}}"#,
        )
        .expect("failed to write local config");
        write(
            &custom_path,
            r#"{"commands":{"daily":{"offline":false,"json":true}}}"#,
        )
        .expect("failed to write custom config");

        let args = ReportArgs {
            config: Some(custom_path),
            ..ReportArgs::default()
        };
        let resolved = resolve_report_args_with_context(
            "daily",
            &args,
            &cwd,
            Some(&home_dir),
            EnvironmentLayer {
                offline: Some(true),
            },
        )
        .expect("expected resolved args");
        assert!(!resolved.offline);
        assert!(resolved.json);
    }

    #[test]
    fn parse_env_bool_recognizes_common_values() {
        assert_eq!(parse_env_bool(Some(OsStr::new("1"))), Some(true));
        assert_eq!(parse_env_bool(Some(OsStr::new("TRUE"))), Some(true));
        assert_eq!(parse_env_bool(Some(OsStr::new("0"))), Some(false));
        assert_eq!(parse_env_bool(Some(OsStr::new("off"))), Some(false));
        assert_eq!(parse_env_bool(Some(OsStr::new("maybe"))), None);
        assert_eq!(parse_env_bool(None), None);
    }

    fn test_event(
        occurred_at_unix_ms: i64,
        session_id: Option<&str>,
        project: Option<&str>,
    ) -> UsageEvent {
        UsageEvent {
            origin: EventOrigin {
                file: PathBuf::from("/tmp/session.jsonl"),
                line_number: 1,
            },
            occurred_at_unix_ms,
            event_kind: EventKind::Assistant,
            session_id: session_id.map(str::to_owned),
            project: project.map(str::to_owned),
            model: Some("claude-sonnet".to_owned()),
            speed: Some(UsageSpeed::Standard),
            usage: TokenUsage::new(1, 2, 0, 0, None),
            raw_cost_usd: Some(0.01),
        }
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            path.push(format!(
                "cusage-rs-main-tests-{}-{timestamp}-{counter}",
                std::process::id()
            ));
            create_dir_all(&path).expect("failed to create test directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }
}

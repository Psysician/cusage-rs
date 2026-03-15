use clap::{Args, Parser, Subcommand};
use cusage_rs::config::DataRootOptions;
use cusage_rs::discovery::discover_session_files;
use cusage_rs::parser::parse_jsonl_files;
use cusage_rs::pricing::{CostMode, PricingCatalog};
use cusage_rs::report::{
    build_blocks_report, build_daily_report, build_monthly_report, build_session_report,
    build_statusline_report, build_weekly_report, render_blocks_report_json,
    render_blocks_report_table, render_daily_report_json, render_daily_report_table,
    render_monthly_report_json, render_monthly_report_table, render_session_report_json,
    render_session_report_table, render_statusline_report_json, render_statusline_report_line,
    render_weekly_report_json, render_weekly_report_table,
};
use std::ffi::OsString;
use std::process::ExitCode;

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
    #[arg(long)]
    json: bool,
    #[arg(long)]
    breakdown: bool,
    #[arg(long)]
    compact: bool,
    #[arg(long)]
    instances: bool,
    #[arg(long, value_name = "NAME")]
    project: Option<String>,
    #[arg(long, value_name = "TZ")]
    timezone: Option<String>,
    #[arg(long, value_name = "LOCALE")]
    locale: Option<String>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq, Default)]
struct StatuslineArgs {
    #[arg(long)]
    json: bool,
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
    println!("{}", describe_command(&command));
    ExitCode::SUCCESS
}

fn describe_command(command: &Command) -> String {
    match command {
        Command::Daily(args) => render_daily_command(args),
        Command::Weekly(args) => render_weekly_command(args),
        Command::Monthly(args) => render_monthly_command(args),
        Command::Session(args) => render_session_command(args),
        Command::Blocks(args) => render_blocks_command(args),
        Command::Statusline(args) => render_statusline_command(args),
    }
}

fn render_daily_command(args: &ReportArgs) -> String {
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_daily_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    if args.json {
        render_daily_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
    } else {
        render_daily_report_table(&report, discovered.warnings.len(), parsed.warnings.len())
    }
}

fn render_weekly_command(args: &ReportArgs) -> String {
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_weekly_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    if args.json {
        render_weekly_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
    } else {
        render_weekly_report_table(&report, discovered.warnings.len(), parsed.warnings.len())
    }
}

fn render_monthly_command(args: &ReportArgs) -> String {
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_monthly_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    if args.json {
        render_monthly_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
    } else {
        render_monthly_report_table(&report, discovered.warnings.len(), parsed.warnings.len())
    }
}

fn render_session_command(args: &ReportArgs) -> String {
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_session_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    if args.json {
        render_session_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
    } else {
        render_session_report_table(&report, discovered.warnings.len(), parsed.warnings.len())
    }
}

fn render_blocks_command(args: &ReportArgs) -> String {
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_blocks_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    if args.json {
        render_blocks_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
    } else {
        render_blocks_report_table(&report, discovered.warnings.len(), parsed.warnings.len())
    }
}

fn render_statusline_command(args: &StatuslineArgs) -> String {
    let data_roots = DataRootOptions::from_environment().resolve_project_roots();
    let discovered = discover_session_files(&data_roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_statusline_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    if args.json {
        render_statusline_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
    } else {
        render_statusline_report_line(&report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Command::Statusline(StatuslineArgs { json: true })
        ));
    }
}

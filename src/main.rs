use clap::{Args, Parser, Subcommand};
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
        Command::Daily(args) => render_report_placeholder("daily", args),
        Command::Monthly(args) => render_report_placeholder("monthly", args),
        Command::Session(args) => render_report_placeholder("session", args),
        Command::Blocks(args) => render_report_placeholder("blocks", args),
        Command::Statusline(args) => {
            if args.json {
                "{\"status\":\"bootstrap\",\"command\":\"statusline\"}".to_owned()
            } else {
                "cusage-rs bootstrap: statusline contract captured; renderer not implemented yet."
                    .to_owned()
            }
        }
    }
}

fn render_report_placeholder(mode: &str, args: &ReportArgs) -> String {
    let mut lines = vec![
        format!("cusage-rs bootstrap: {mode} mode is scaffolded but not implemented yet."),
        "Parity target: upstream ryoppippi/ccusage CLI behavior.".to_owned(),
    ];

    if let Some(since) = &args.since {
        lines.push(format!("Filter since: {since}"));
    }
    if let Some(until) = &args.until {
        lines.push(format!("Filter until: {until}"));
    }
    if args.json {
        lines.push("Requested JSON output contract.".to_owned());
    }
    if args.breakdown {
        lines.push("Requested per-model breakdown contract.".to_owned());
    }
    if args.compact {
        lines.push("Requested compact rendering contract.".to_owned());
    }
    if args.instances {
        lines.push("Requested multi-instance grouping contract.".to_owned());
    }
    if let Some(project) = &args.project {
        lines.push(format!("Project filter: {project}"));
    }
    if let Some(timezone) = &args.timezone {
        lines.push(format!("Timezone override: {timezone}"));
    }
    if let Some(locale) = &args.locale {
        lines.push(format!("Locale override: {locale}"));
    }

    lines.join("\n")
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
    fn monthly_flags_round_trip_into_placeholder() {
        let output = describe_command(&Command::Monthly(ReportArgs {
            since: Some("20250525".to_owned()),
            until: Some("20250530".to_owned()),
            json: true,
            breakdown: true,
            compact: false,
            instances: true,
            project: Some("demo".to_owned()),
            timezone: Some("UTC".to_owned()),
            locale: Some("en-US".to_owned()),
        }));

        assert!(output.contains("monthly"));
        assert!(output.contains("20250525"));
        assert!(output.contains("Requested JSON output contract."));
        assert!(output.contains("Project filter: demo"));
    }
}

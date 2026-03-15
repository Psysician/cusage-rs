use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn daily_json_matches_real_ccusage_totals_for_modern_assistant_usage_fixture() {
    let Some(ccusage_path) = command_path("ccusage") else {
        eprintln!("skipping oracle test because ccusage is not installed");
        return;
    };

    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("oracle")
        .join("daily")
        .join("claude-config");

    let upstream = run_command(
        &ccusage_path,
        &["daily", "--offline", "--json", "--timezone", "UTC"],
        &fixture_root,
    );
    let local = run_command(
        Path::new(env!("CARGO_BIN_EXE_cusage-rs")),
        &["daily", "--json", "--timezone", "UTC"],
        &fixture_root,
    );

    assert!(
        upstream.status.success(),
        "upstream ccusage failed: {upstream:#?}"
    );
    assert!(local.status.success(), "local cusage-rs failed: {local:#?}");

    let upstream_stdout = stdout_text(&upstream);
    let local_stdout = stdout_text(&local);

    assert_eq!(
        extract_u64(&upstream_stdout, "inputTokens"),
        extract_u64(&local_stdout, "input")
    );
    assert_eq!(
        extract_u64(&upstream_stdout, "outputTokens"),
        extract_u64(&local_stdout, "output")
    );
    assert_eq!(
        extract_u64(&upstream_stdout, "cacheCreationTokens"),
        extract_u64(&local_stdout, "cache_creation_input")
    );
    assert_eq!(
        extract_u64(&upstream_stdout, "cacheReadTokens"),
        extract_u64(&local_stdout, "cache_read_input")
    );
    assert_eq!(
        extract_u64(&upstream_stdout, "totalTokens"),
        extract_u64(&local_stdout, "total")
    );

    let upstream_cost = extract_f64(&upstream_stdout, "totalCost");
    let local_cost = extract_f64(&local_stdout, "usd");
    let delta = (upstream_cost - local_cost).abs();
    assert!(
        delta <= 0.000_001,
        "expected matching cost totals, upstream={upstream_cost}, local={local_cost}, delta={delta}"
    );

    assert!(
        local_stdout.contains("\"entries\": 4"),
        "expected deduped entry count in local json output: {local_stdout}"
    );
}

fn run_command(program: &Path, args: &[&str], claude_config_dir: &Path) -> std::process::Output {
    Command::new(program)
        .args(args)
        .env("CLAUDE_CONFIG_DIR", claude_config_dir)
        .output()
        .expect("failed to spawn command")
}

fn command_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path) {
        let candidate = entry.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn stdout_text(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout must be valid utf-8")
}

fn extract_u64(text: &str, field: &str) -> u64 {
    extract_number_token(text, field)
        .parse::<u64>()
        .expect("field must parse as u64")
}

fn extract_f64(text: &str, field: &str) -> f64 {
    extract_number_token(text, field)
        .parse::<f64>()
        .expect("field must parse as f64")
}

fn extract_number_token(text: &str, field: &str) -> String {
    let needle = format!("\"{field}\":");
    let start = text
        .find(&needle)
        .unwrap_or_else(|| panic!("missing field {field} in output: {text}"))
        + needle.len();
    let rest = &text[start..];
    let trimmed = rest.trim_start();
    let end = trimmed
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-'))
        .unwrap_or(trimmed.len());
    trimmed[..end].to_owned()
}

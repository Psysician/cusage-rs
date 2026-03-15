use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn daily_json_respects_since_until_project_and_timezone_flags() {
    let expected = read_fixture("cli/daily_json_filtered/expected.json");
    let claude_config_dir = fixture_root().join("session/basic/claude-config");

    let first = run_cli(
        &[
            "daily",
            "--json",
            "--since",
            "20260310",
            "--until",
            "20260310",
            "--project",
            "team-beta",
            "--timezone=-02:00",
        ],
        &claude_config_dir,
    );
    let second = run_cli(
        &[
            "daily",
            "--json",
            "--since",
            "20260310",
            "--until",
            "20260310",
            "--project",
            "team-beta",
            "--timezone=-02:00",
        ],
        &claude_config_dir,
    );

    assert_success(&first, "daily json filtered flags");
    assert_success(&second, "daily json filtered flags repeat");

    let expected = normalize_line_end(expected);
    let first_stdout = normalize_line_end(stdout_text(&first));
    let second_stdout = normalize_line_end(stdout_text(&second));

    assert_eq!(
        first_stdout, expected,
        "daily filtered json output mismatch"
    );
    assert_eq!(
        first_stdout, second_stdout,
        "daily filtered json output is not stable"
    );
}

#[test]
fn session_table_supports_compact_breakdown_instances_and_locale_flags() {
    let expected = read_fixture("cli/session_table_flags/expected.txt");
    let claude_config_dir = fixture_root().join("session/basic/claude-config");

    let first = run_cli(
        &[
            "session",
            "--compact",
            "--breakdown",
            "--instances",
            "--locale",
            "de-DE",
        ],
        &claude_config_dir,
    );
    let second = run_cli(
        &[
            "session",
            "--compact",
            "--breakdown",
            "--instances",
            "--locale",
            "de-DE",
        ],
        &claude_config_dir,
    );

    assert_success(&first, "session compact/breakdown/instances/locale flags");
    assert_success(
        &second,
        "session compact/breakdown/instances/locale flags repeat",
    );

    let expected = normalize_line_end(expected);
    let first_stdout = normalize_line_end(stdout_text(&first));
    let second_stdout = normalize_line_end(stdout_text(&second));

    assert_eq!(first_stdout, expected, "session table flag output mismatch");
    assert_eq!(
        first_stdout, second_stdout,
        "session table flag output is not stable"
    );
}

#[test]
fn malformed_jsonl_is_tolerated_with_deterministic_warning_counts() {
    let expected = read_fixture("daily/malformed/expected.json");
    let claude_config_dir = fixture_root().join("daily/malformed/claude-config");

    let first = run_cli(&["daily", "--json"], &claude_config_dir);
    let second = run_cli(&["daily", "--json"], &claude_config_dir);

    assert_success(&first, "daily json malformed fixture");
    assert_success(&second, "daily json malformed fixture repeat");

    let expected = normalize_line_end(expected);
    let first_stdout = normalize_line_end(stdout_text(&first));
    let second_stdout = normalize_line_end(stdout_text(&second));

    assert_eq!(first_stdout, expected, "malformed fixture output mismatch");
    assert_eq!(
        first_stdout, second_stdout,
        "malformed fixture output is not stable"
    );
    assert!(
        first_stdout.contains("\"parse\": 3"),
        "expected parse warning count to be reported"
    );
}

#[test]
fn invalid_since_date_returns_expected_error() {
    let expected = read_fixture("cli/errors/invalid_since.stderr");
    let claude_config_dir = fixture_root().join("session/basic/claude-config");

    let output = run_cli(
        &["daily", "--json", "--since", "2026/03/10"],
        &claude_config_dir,
    );

    assert_failure(&output, "invalid since date should fail");
    assert_eq!(
        normalize_line_end(stderr_text(&output)),
        normalize_line_end(expected),
        "invalid since stderr mismatch"
    );
}

#[test]
fn inverted_since_until_returns_expected_error() {
    let expected = read_fixture("cli/errors/inverted_since_until.stderr");
    let claude_config_dir = fixture_root().join("session/basic/claude-config");

    let output = run_cli(
        &[
            "daily", "--json", "--since", "20260312", "--until", "20260310",
        ],
        &claude_config_dir,
    );

    assert_failure(&output, "inverted since/until should fail");
    assert_eq!(
        normalize_line_end(stderr_text(&output)),
        normalize_line_end(expected),
        "inverted since/until stderr mismatch"
    );
}

#[test]
fn unsupported_timezone_returns_expected_error() {
    let expected = read_fixture("cli/errors/unsupported_timezone.stderr");
    let claude_config_dir = fixture_root().join("session/basic/claude-config");

    let output = run_cli(
        &["daily", "--json", "--timezone", "Europe/Berlin"],
        &claude_config_dir,
    );

    assert_failure(&output, "unsupported timezone should fail");
    assert_eq!(
        normalize_line_end(stderr_text(&output)),
        normalize_line_end(expected),
        "unsupported timezone stderr mismatch"
    );
}

fn run_cli(args: &[&str], claude_config_dir: &Path) -> Output {
    let test_dir = TestDir::new();
    let home_dir = test_dir.path().join("home");
    fs::create_dir_all(&home_dir).expect("failed to create isolated HOME for CLI test");

    Command::new(env!("CARGO_BIN_EXE_cusage-rs"))
        .args(args)
        .current_dir(test_dir.path())
        .env("CLAUDE_CONFIG_DIR", claude_config_dir)
        .env("HOME", &home_dir)
        .env("USERPROFILE", &home_dir)
        .env("XDG_CONFIG_HOME", home_dir.join(".config"))
        .env_remove("CCUSAGE_OFFLINE")
        .output()
        .expect("failed to execute cusage-rs CLI")
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}: status {:?}, stderr: {}",
        output.status,
        stderr_text(output)
    );
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context}: expected failure, stdout: {}",
        stdout_text(output)
    );
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn read_fixture(relative_path: &str) -> String {
    fs::read_to_string(fixture_root().join(relative_path)).expect("failed to read fixture")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn normalize_line_end(mut value: String) -> String {
    while value.ends_with('\n') {
        value.pop();
    }
    value.push('\n');
    value
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
            "cusage-rs-cli-parity-tests-{}-{timestamp}-{counter}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("failed to create test directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

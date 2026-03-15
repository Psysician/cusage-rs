use cusage_rs::discovery::discover_session_files;
use cusage_rs::parser::parse_jsonl_files;
use cusage_rs::pricing::{CostMode, PricingCatalog};
use cusage_rs::report::{
    build_statusline_report, render_statusline_report_json, render_statusline_report_line,
};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn statusline_outputs_match_basic_fixture() {
    assert_fixture("basic");
}

#[test]
fn statusline_outputs_handle_malformed_lines_deterministically() {
    assert_fixture("malformed");
}

fn assert_fixture(fixture_name: &str) {
    let fixture_root = fixture_root(fixture_name);
    let expected_line = fs::read_to_string(fixture_root.join("expected.txt"))
        .expect("failed to read expected statusline text output");
    let expected_json = fs::read_to_string(fixture_root.join("expected.json"))
        .expect("failed to read expected statusline json output");

    let (first_line, first_json) = run_statusline_outputs(&fixture_root.join("claude-config"));
    let (second_line, second_json) = run_statusline_outputs(&fixture_root.join("claude-config"));

    let expected_line = normalize_line_end(expected_line);
    let expected_json = normalize_line_end(expected_json);
    let first_line = normalize_line_end(first_line);
    let first_json = normalize_line_end(first_json);
    let second_line = normalize_line_end(second_line);
    let second_json = normalize_line_end(second_json);

    assert_eq!(
        first_line, expected_line,
        "fixture {fixture_name} line output mismatch"
    );
    assert_eq!(
        first_json, expected_json,
        "fixture {fixture_name} json output mismatch"
    );
    assert_eq!(
        first_line, second_line,
        "fixture {fixture_name} line output is not stable"
    );
    assert_eq!(
        first_json, second_json,
        "fixture {fixture_name} json output is not stable"
    );

    let compact_line = first_line.trim_end_matches('\n');
    assert!(!compact_line.contains('\n'));
}

fn run_statusline_outputs(claude_config_dir: &Path) -> (String, String) {
    let roots = vec![claude_config_dir.join("projects")];
    let discovered = discover_session_files(&roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_statusline_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());

    let line = render_statusline_report_line(&report);
    let json =
        render_statusline_report_json(&report, discovered.warnings.len(), parsed.warnings.len());
    (line, json)
}

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("statusline")
        .join(name)
}

fn normalize_line_end(mut value: String) -> String {
    while value.ends_with('\n') {
        value.pop();
    }
    value.push('\n');
    value
}

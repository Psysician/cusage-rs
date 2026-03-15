use cusage_rs::discovery::discover_session_files;
use cusage_rs::parser::parse_jsonl_files;
use cusage_rs::pricing::{CostMode, PricingCatalog};
use cusage_rs::report::{build_blocks_report, render_blocks_report_json};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn blocks_json_matches_basic_fixture() {
    assert_fixture("basic");
}

#[test]
fn blocks_json_handles_malformed_lines_deterministically() {
    assert_fixture("malformed");
}

fn assert_fixture(fixture_name: &str) {
    let fixture_root = fixture_root(fixture_name);
    let expected = fs::read_to_string(fixture_root.join("expected.json"))
        .expect("failed to read expected fixture output");

    let first = run_blocks_json(&fixture_root.join("claude-config"));
    let second = run_blocks_json(&fixture_root.join("claude-config"));
    let expected = normalize_line_end(expected);
    let first = normalize_line_end(first);
    let second = normalize_line_end(second);

    assert_eq!(first, expected, "fixture {fixture_name} output mismatch");
    assert_eq!(first, second, "fixture {fixture_name} output is not stable");
}

fn run_blocks_json(claude_config_dir: &Path) -> String {
    let roots = vec![claude_config_dir.join("projects")];
    let discovered = discover_session_files(&roots);
    let parsed = parse_jsonl_files(&discovered.files);
    let report = build_blocks_report(&parsed.events, CostMode::Auto, &PricingCatalog::new());
    render_blocks_report_json(&report, discovered.warnings.len(), parsed.warnings.len())
}

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("blocks")
        .join(name)
}

fn normalize_line_end(mut value: String) -> String {
    while value.ends_with('\n') {
        value.pop();
    }
    value.push('\n');
    value
}

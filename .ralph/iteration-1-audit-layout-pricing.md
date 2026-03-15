# Iteration 1 Audit: Human Layout + Pricing Flow Baseline

## Scope
- Audited current human-readable output and pricing flow against existing fixtures/tests.
- Goal: identify concrete gaps for subsequent tasks (no assumptions).

## Pricing Flow Mapping
- `PricingCatalog::new()` currently creates an empty model catalog (only provider prefixes are populated), so no model prices are available by default.
  - `src/pricing.rs:115-125`
- All CLI report/statusline entrypoints build reports with `CostMode::Auto` + `PricingCatalog::new()`.
  - `src/main.rs:244-339`
- All parity report/statusline fixture tests also use `PricingCatalog::new()` directly.
  - `tests/parity_daily.rs:33-39`
  - `tests/parity_weekly.rs:33-38`
  - `tests/parity_monthly.rs:33-38`
  - `tests/parity_session.rs:33-38`
  - `tests/parity_blocks.rs:33-38`
  - `tests/parity_statusline.rs:58-67`
- Cost provenance behavior is implemented correctly in principle:
  - raw cost used when present,
  - otherwise catalog calculation attempted,
  - otherwise marked missing.
  - `src/pricing.rs:246-336`

## Fixture Evidence: Current Cost Outcomes
- Existing JSON fixtures for `daily/weekly/monthly/session/blocks` all show `calculated_entries: 0` in basic datasets.
  - `tests/fixtures/daily/basic/expected.json`
  - `tests/fixtures/weekly/basic/expected.json`
  - `tests/fixtures/monthly/basic/expected.json`
  - `tests/fixtures/session/basic/expected.json`
  - `tests/fixtures/blocks/basic/expected.json`
- Existing fixtures include missing-cost events, but those events are currently unknown-model or model-missing cases.
  - unknown model without raw cost:
    - `tests/fixtures/blocks/basic/claude-config/projects/team-beta/session-b.jsonl:3`
    - `tests/fixtures/statusline/basic/claude-config/projects/team-beta/session-b.jsonl:3`
  - model omitted and raw cost omitted:
    - `tests/fixtures/session/basic/claude-config/projects/team-alpha/session-a.jsonl:2`
- There is currently **no fixture-backed regression** for “known model + missing raw cost => non-zero calculated USD”.

## Human-Readable Output Baseline (Current)
- Default table renderers are whitespace tables with uppercase column labels plus trailing `WARNINGS ...` line.
  - `src/report.rs:916-954` (daily)
  - `src/report.rs:1079-1121` (weekly)
  - `src/report.rs:1242-1280` (monthly)
  - `src/report.rs:1405-1447` (session)
  - `src/report.rs:1580-1622` (blocks)
- Custom table paths for `--compact/--breakdown/--instances/--locale` are functional but still plain, dense text without intentional sectioning/hierarchy.
  - `src/main.rs:482-1064`
- Statusline non-JSON output is a single dense key-value line:
  - format implementation: `src/report.rs:1695-1715`
  - current fixture examples:
    - `tests/fixtures/statusline/basic/expected.txt`
    - `tests/fixtures/statusline/malformed/expected.txt`

## Concrete Gaps Confirmed
1. No populated default pricing catalog is wired into report/statusline paths.
2. Known-model cost calculation can only happen when callers manually inject pricing (seen in unit tests), not in normal CLI/report paths.
3. Fixture coverage is missing the key regression case: known model with missing raw cost producing non-zero calculated USD.
4. Human-readable layouts are deterministic and functional but visually minimal/placeholder-like (tables + warnings line, limited hierarchy).
5. Statusline line is compact but currently dense and harder to scan quickly for key fields.


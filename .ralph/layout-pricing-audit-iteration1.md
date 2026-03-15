# Layout/Pricing Audit (Iteration 1)

## Scope
- Audit target: current human-readable report output and pricing flow.
- Evidence sources: `src/main.rs`, `src/report.rs`, `src/pricing.rs`, `tests/parity_*.rs`, `tests/fixtures/**`.

## Pricing Flow Findings

1. CLI/report/statusline paths all construct an empty pricing catalog.
- Evidence:
  - `render_daily_command` uses `PricingCatalog::new()` at `src/main.rs:247`.
  - `render_weekly_command` uses `PricingCatalog::new()` at `src/main.rs:263`.
  - `render_monthly_command` uses `PricingCatalog::new()` at `src/main.rs:279`.
  - `render_session_command` uses `PricingCatalog::new()` at `src/main.rs:295`.
  - `render_blocks_command` uses `PricingCatalog::new()` at `src/main.rs:311`.
  - `render_statusline_command` uses `PricingCatalog::new()` at `src/main.rs:329`.

2. `PricingCatalog::new()` only seeds provider prefixes, not model prices.
- Evidence:
  - `PricingCatalog::new`/`with_default_provider_prefixes` in `src/pricing.rs:117-130`.
  - No production `insert(...)` usage outside tests (`src/pricing.rs:463`, `src/pricing.rs:517`, `src/report.rs:1888`, `src/report.rs:1964`, `src/report.rs:2054`, `src/report.rs:2142` are test-only).

3. Cost provenance logic is implemented, but calculated costs depend on catalog population.
- Evidence:
  - `CostMode::Auto`: prefer raw cost, otherwise calculate from catalog in `src/pricing.rs:258-269`.
  - Unknown/missing model resolves to `Missing` at `src/pricing.rs:325-331`.
  - Metrics preserve raw/calculated/missing counts at `src/pricing.rs:297-301`.

## Human-Readable Output Findings

1. Default table renderers are rigid whitespace tables with minimal structure.
- Evidence:
  - Daily table renderer: `src/report.rs:916-954`
  - Weekly table renderer: `src/report.rs:1079-1121`
  - Monthly table renderer: `src/report.rs:1242-1280`
  - Session table renderer: `src/report.rs:1405-1447`
  - Blocks table renderer: `src/report.rs:1580-1622`
- Current baseline includes only header row + aligned numeric columns + totals + warning line (no intentional sectioning).

2. Statusline non-JSON output is compact but dense key-value stream.
- Evidence:
  - Renderer: `src/report.rs:1695-1716`
  - Fixture example: `tests/fixtures/statusline/basic/expected.txt:1`

3. Custom flag layouts exist in `main.rs`, but only partially regression-covered.
- Evidence:
  - Custom table branches (`--compact`, `--breakdown`, `--instances`, locale formatting) in `src/main.rs:482-1102`.
  - Only session custom table output has text fixture parity coverage: `tests/parity_cli.rs:61-103`, fixture `tests/fixtures/cli/session_table_flags/expected.txt`.

## Fixture/Test Coverage Matrix (Current)

- JSON parity fixtures exist for:
  - `daily`: `tests/parity_daily.rs`, `tests/fixtures/daily/*/expected.json`
  - `weekly`: `tests/parity_weekly.rs`, `tests/fixtures/weekly/*/expected.json`
  - `monthly`: `tests/parity_monthly.rs`, `tests/fixtures/monthly/*/expected.json`
  - `session`: `tests/parity_session.rs`, `tests/fixtures/session/*/expected.json`
  - `blocks`: `tests/parity_blocks.rs`, `tests/fixtures/blocks/*/expected.json`
  - `statusline`: `tests/parity_statusline.rs`, `tests/fixtures/statusline/*/expected.json`

- Text/human fixtures exist for:
  - `statusline` non-JSON: `tests/fixtures/statusline/*/expected.txt`, asserted in `tests/parity_statusline.rs:20-56`.
  - `session` custom table flags: `tests/fixtures/cli/session_table_flags/expected.txt`, asserted in `tests/parity_cli.rs:61-103`.

- Missing human text fixtures/regression for default table outputs:
  - `daily`, `weekly`, `monthly`, `session`, `blocks` default non-JSON layouts.

## Explicit Gaps To Drive Remaining Tasks

1. No populated default pricing catalog is wired into report/statusline commands, so known-model/no-raw events cannot calculate cost via normal CLI paths.
2. Existing fixtures do not cover the key acceptance case: known model + missing raw `cost_usd` -> non-zero calculated USD through CLI/report path.
3. Default human-readable layouts for `daily`/`weekly`/`monthly`/`session`/`blocks` have no deterministic text fixture lock and currently resemble raw column dumps.
4. Statusline text is deterministic but still hard to scan quickly due to dense flat key/value formatting.
5. Coverage is skewed to JSON parity; human/table regressions are sparse outside one session custom-flags fixture.

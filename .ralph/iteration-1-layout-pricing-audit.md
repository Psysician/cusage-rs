# Iteration 1 Audit: Human Layout + Pricing Flow Baseline

Date: 2026-03-15
Scope: Current behavior only (no changes), anchored to code + existing fixtures.

## 1) Current Pricing Flow (Observed)

- Every CLI report/statusline entrypoint builds reports with `CostMode::Auto` and a fresh empty catalog:
  - `src/main.rs:247`, `src/main.rs:263`, `src/main.rs:279`, `src/main.rs:295`, `src/main.rs:311`, `src/main.rs:329`
  - Call shape is consistently `build_*_report(..., CostMode::Auto, &PricingCatalog::new())`
- `PricingCatalog::new()` only installs provider prefixes; it does not insert any model prices:
  - `src/pricing.rs:117-129`
- Cost resolution is honest/provenance-aware in core logic:
  - raw if present and valid, else calculated via catalog, else missing:
  - `src/pricing.rs:244-269`, `src/pricing.rs:325-336`
- Aggregators preserve raw/calculated/missing counts into rows and totals:
  - e.g. daily `src/report.rs:256-269`, `src/report.rs:295-306`
  - same pattern in weekly/monthly/session/blocks builders

### Pricing Baseline from Fixtures

- Existing golden fixtures show `calculated_entries: 0` across main modes:
  - daily `tests/fixtures/daily/basic/expected.json:17`, `tests/fixtures/daily/basic/expected.json:52`
  - weekly `tests/fixtures/weekly/basic/expected.json:18`, `tests/fixtures/weekly/basic/expected.json:36`
  - monthly `tests/fixtures/monthly/basic/expected.json:17`, `tests/fixtures/monthly/basic/expected.json:35`
  - session `tests/fixtures/session/basic/expected.json:18`, `tests/fixtures/session/basic/expected.json:54`
  - blocks `tests/fixtures/blocks/basic/expected.json:20`, `tests/fixtures/blocks/basic/expected.json:78`
- Fixture coverage gap: no fixture currently proves "known model + missing raw cost => calculated non-zero".
  - model present + missing cost lines in fixtures are only `unknown-model`:
    - `tests/fixtures/blocks/basic/claude-config/projects/team-beta/session-b.jsonl:3`
    - `tests/fixtures/statusline/basic/claude-config/projects/team-beta/session-b.jsonl:3`

## 2) Current Human-Readable Layout Flow (Observed)

- Two rendering paths exist:
  - default path in `src/report.rs` table renderers (`render_*_report_table`)
  - custom path in `src/main.rs` when any of `--compact`, `--breakdown`, `--instances`, locale decimal-comma are enabled
    - switch condition: `src/main.rs:483-489`, `src/main.rs:595-601`, `src/main.rs:710-716`, `src/main.rs:822-828`, `src/main.rs:936-942`
- Default tables are plain uppercase headers and whitespace-aligned rows with trailing warnings line:
  - sample definitions:
    - monthly header `src/report.rs:1248`
    - session header `src/report.rs:1412-1414`
    - blocks header `src/report.rs:1587-1589`
    - warnings footer pattern `src/report.rs:1275-1277`, `src/report.rs:1442-1444`, `src/report.rs:1617-1619`
- Custom compact tables are space-delimited and intentionally minimal:
  - daily compact header `src/main.rs:499-502`
  - weekly compact header `src/main.rs:611-614`
  - monthly compact header `src/main.rs:726-729`
  - session compact header `src/main.rs:838-841`
  - blocks compact header `src/main.rs:952-955`
  - shared footer `INSTANCES`/`WARNINGS`: `src/main.rs:1050-1063`

### Layout Baseline from Fixtures/Tests

- Text fixtures currently lock only:
  - statusline text output: `tests/fixtures/statusline/basic/expected.txt`, `tests/fixtures/statusline/malformed/expected.txt`
  - one representative compact+breakdown+instances+locale session table:
    - `tests/fixtures/cli/session_table_flags/expected.txt`
- Deterministic tests for daily/weekly/monthly/session/blocks parity are JSON-only:
  - `tests/parity_daily.rs`, `tests/parity_weekly.rs`, `tests/parity_monthly.rs`, `tests/parity_session.rs`, `tests/parity_blocks.rs`
- Resulting gap: redesigned default human tables for daily/weekly/monthly/session/blocks are not currently fixture-protected.

## 3) Concrete Gaps Anchored for Next Tasks

1. **Catalog population gap**: runtime always uses an unpopulated `PricingCatalog`, so calculated cost for known models cannot happen through normal CLI/report paths today.
2. **Regression coverage gap**: no fixture/test currently proves known-model non-raw events compute non-zero USD.
3. **Layout quality gap**: default non-JSON outputs are deterministic but read as utilitarian debug-style tables (single header row + raw columns + trailing warning line, no intentional sections/hierarchy).
4. **Layout consistency gap**: default table renderer lives in `report.rs`, while custom behavior lives in `main.rs`, causing two formatting systems with different visual language.
5. **Statusline scanability gap**: compact `key=value` line is deterministic but dense for fast visual scanning (`model=... session_usd=... today_usd=... block_usd=... block_remaining=... burn... input_tokens=...`).
6. **Default provenance visibility gap**: unresolved/missing cost can be hidden in default tables unless `--breakdown` is enabled.

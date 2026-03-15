# Layout + Pricing Audit Baseline (Iteration 1)

This audit captures current behavior using existing code paths and fixtures so follow-up tasks are scoped to concrete gaps.

## Scope audited

- Human-readable output for `daily`, `weekly`, `monthly`, `session`, `blocks`, `statusline`
- Pricing/cost resolution flow used by CLI and parity tests
- Deterministic fixture coverage for text/JSON output

## Commands executed

- `CLAUDE_CONFIG_DIR=$PWD/tests/fixtures/session/basic/claude-config cargo run --quiet -- daily|weekly|monthly|session|blocks|statusline`
- Fixture/code inspection under `tests/fixtures` and `src/{main,report,pricing}.rs`

## Current human-readable baseline

Observed non-JSON CLI output (session/basic fixture root):

```text
===== daily
DATE       ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD
2026-03-10       2   125     60            5         15   205     0.12
2026-03-11       2   100     50            0         10   160     0.15
TOTAL            4   225    110            5         25   365     0.27
WARNINGS discovery=0 parse=0

===== weekly
WEEK_START WEEK_END   ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD
2026-03-09 2026-03-15       4   225    110            5         25   365     0.27
TOTAL                      4   225    110            5         25   365     0.27
WARNINGS discovery=0 parse=0

===== monthly
MONTH    ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD
2026-03       4   225    110            5         25   365     0.27
TOTAL          4   225    110            5         25   365     0.27
WARNINGS discovery=0 parse=0

===== session
SESSION_ID          PROJECT             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD
session-a           team-alpha                2   125     60            5         15   205     0.12
session-b           team-beta                 2   100     50            0         10   160     0.15
TOTAL                                        4   225    110            5         25   365     0.27
WARNINGS discovery=0 parse=0

===== blocks
BLOCK_START           BLOCK_END             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD
2026-03-10T12:00:00Z 2026-03-10T17:00:00Z       1   100     50            0          0   150     0.12
2026-03-10T18:00:00Z 2026-03-10T23:00:00Z       1    25     10            5         15    55      0.0
2026-03-11T00:30:00Z 2026-03-11T05:30:00Z       1    40     20            0          0    60     0.06
2026-03-11T06:00:00Z 2026-03-11T11:00:00Z       1    60     30            0         10   100     0.09
TOTAL                                          4   225    110            5         25   365     0.27
WARNINGS discovery=0 parse=0

===== statusline
model=claude-sonnet session_usd=0.15 today_usd=0.15 block_usd=0.09 block_remaining=5h00m burn_usd_per_hour=0.00 input_tokens=100
```

## Pricing flow findings

1. CLI/report paths always pass an empty catalog:
   - `src/main.rs:247`
   - `src/main.rs:263`
   - `src/main.rs:279`
   - `src/main.rs:295`
   - `src/main.rs:311`
   - `src/main.rs:329`

2. `PricingCatalog::new()` only sets provider prefixes; it does not seed model prices:
   - `src/pricing.rs:117-119`
   - `src/pricing.rs:122-130`

3. Cost calculation requires both model + resolved pricing; otherwise cost source becomes missing (0):
   - `src/pricing.rs:325-336`

4. Existing basic fixtures currently encode zero calculated entries for all main report modes, confirming no priced calculation occurs in fixture-driven report paths:
   - `tests/fixtures/daily/basic/expected.json`
   - `tests/fixtures/weekly/basic/expected.json`
   - `tests/fixtures/monthly/basic/expected.json`
   - `tests/fixtures/session/basic/expected.json`
   - `tests/fixtures/blocks/basic/expected.json`

5. Existing fixture data has no known-model + missing-raw-cost event that can prove calculated non-zero pricing in normal CLI paths:
   - model-present rows are either raw-cost rows (`claude-sonnet`) or unknown model rows without raw cost.
   - representative rows: `tests/fixtures/statusline/basic/claude-config/projects/team-alpha/session-a.jsonl:1` and `tests/fixtures/statusline/basic/claude-config/projects/team-beta/session-b.jsonl:3`

## Human layout findings

1. Default report tables are raw header + rows + `TOTAL` + `WARNINGS` with no sectioning or visual hierarchy:
   - `src/report.rs:916-954` (daily)
   - `src/report.rs:1079-1121` (weekly)
   - `src/report.rs:1242-1279` (monthly)
   - `src/report.rs:1405-1446` (session)
   - `src/report.rs:1570-1610` (blocks)

2. Statusline non-JSON output is one dense key/value line with low scan affordance:
   - formatter: `src/report.rs:1695-1715`
   - fixture shape: `tests/fixtures/statusline/basic/expected.txt:1`

3. Custom flag tables (`--compact`, `--breakdown`, `--instances`) are deterministic but still utilitarian/plain:
   - `src/main.rs:482-1042`
   - deterministic text fixture: `tests/fixtures/cli/session_table_flags/expected.txt:1`

## Deterministic fixture coverage findings

1. Strong JSON parity coverage exists for all report modes/statusline:
   - `tests/parity_daily.rs:33-38`
   - `tests/parity_weekly.rs:33-38`
   - `tests/parity_monthly.rs:33-38`
   - `tests/parity_session.rs:33-38`
   - `tests/parity_blocks.rs:33-38`
   - `tests/parity_statusline.rs:58-66`

2. Text fixture coverage is currently narrow:
   - statusline text: `tests/fixtures/statusline/*/expected.txt`
   - one representative shared-flag session table fixture: `tests/fixtures/cli/session_table_flags/expected.txt`
   - no golden text fixtures for default non-JSON layouts of daily/weekly/monthly/session/blocks.

## Concrete gap list for next tasks

- G1: Populate and centrally wire a default Claude pricing catalog across all report/statusline entry points.
- G2: Preserve and expose raw/calculated/missing provenance while enabling non-zero calculated costs for known models with no raw `cost_usd`.
- G3: Redesign default human-readable layouts (daily/weekly/monthly/session/blocks) for intentional readability while preserving determinism.
- G4: Improve statusline non-JSON scanability while keeping compact single-line behavior.
- G5: Add fixture-driven regression coverage for new default text layouts and known-model/no-raw calculated pricing path.

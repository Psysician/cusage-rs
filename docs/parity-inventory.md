# cusage-rs parity inventory

## Scope and snapshot

- Behavioral target: [`ryoppippi/ccusage`](https://github.com/ryoppippi/ccusage)
- Snapshot date: `2026-03-15`
- Rewrite constraint: clean-room Rust implementation
- Legend:
  - `[x]` implemented and covered by deterministic tests/fixtures
  - `[ ]` explicit residual delta vs public upstream contract docs

## Upstream contract sources used

- [Command-Line Options](https://ccusage.com/guide/cli-options)
- [Configuration Files](https://ccusage.com/guide/config-files)
- [Configuration Overview](https://ccusage.com/guide/configuration)
- [Directory Detection](https://ccusage.com/guide/directory-detection)
- [Environment Variables](https://ccusage.com/guide/environment-variables)
- [Upstream README](https://github.com/ryoppippi/ccusage)

## Current local baseline

- CLI commands implemented: `daily`, `weekly`, `monthly`, `session`, `blocks`, `statusline`
- End-to-end runtime pipeline implemented:
  - filesystem discovery
  - JSONL parsing + normalization
  - pricing and derived metrics
  - report aggregation by mode
  - table/JSON rendering
- All CLI report paths, including `statusline`, use `PricingCatalog::default_claude_catalog()` (Claude aliases plus provider-prefixed variants).
- Cost provenance is preserved across reports (`raw` vs `calculated` vs `missing`) and exposed in both JSON fields and human table `R/C/M` columns.
- Deterministic JSON golden fixtures exist for every report mode.
- Deterministic fixtures cover redesigned default human-readable layouts for `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`, plus representative shared-flag table behavior.

## Verification coverage

- `tests/parity_daily.rs`
- `tests/parity_weekly.rs`
- `tests/parity_monthly.rs`
- `tests/parity_session.rs`
- `tests/parity_blocks.rs`
- `tests/parity_statusline.rs`
- `tests/parity_cli.rs`:
  - redesigned default human layout fixture checks (`tests/fixtures/cli/human_layouts/*`)
  - known-model missing-raw-cost regression (`tests/fixtures/cli/daily_calculated_cost/*`)
  - shared-flag behavior (`--since`, `--until`, `--project`, `--timezone`, `--compact`, `--breakdown`, `--instances`, `--locale`)
  - deterministic output checks
  - malformed input behavior
  - expected CLI errors for invalid date range/date format/unsupported timezone

## Parity checklist

### Report modes

- [x] `daily`
- [x] `weekly`
- [x] `monthly`
- [x] `session`
- [x] `blocks`
- [x] `statusline`

### Shared/global CLI contract

- [x] `--since YYYYMMDD`
- [x] `--until YYYYMMDD`
- [x] `--json` / `-j`
- [x] `--breakdown` / `-b`
- [x] `--compact`
- [ ] `--mode auto|calculate|display`
- [x] `--offline` / `-O` and `--no-offline`
- [x] `--timezone <tz>` / `-z` (fixed-offset forms; see residual deltas)
- [x] `--locale <locale>` / `-l`
- [x] `--config <path>`
- [ ] `--debug`
- [ ] `--debug-samples <n>`
- [ ] `--jq <filter>` (JSON post-processing)

### Command-specific CLI contract

- [x] `daily`: `--instances` / `-i`, `--project <name>` / `-p`
- [ ] `weekly`: `--start-of-week monday|sunday`
- [x] `session`: `--project <name>`
- [ ] `session`: `--id <session-id>`
- [ ] `blocks`: `--active`, `--recent`, `--token-limit`, `--session-length`, `--live`, `--refresh-interval`
- [x] `statusline`: `--offline`, `--json`, `--config`
- [ ] `statusline`: `--cache`, `--refresh-interval`

### Discovery and data-root behavior

- [x] Default root discovery merges both `~/.config/claude/projects` and `~/.claude/projects`
- [x] `CLAUDE_CONFIG_DIR` supports one custom root
- [x] `CLAUDE_CONFIG_DIR` supports comma-separated multiple roots with aggregation
- [x] Invalid/unreadable roots are handled defensively without corrupting totals

### Configuration file behavior

- [x] Detect local config at `.ccusage/ccusage.json`
- [x] Detect user config at `~/.config/claude/ccusage.json`
- [x] Detect legacy config at `~/.claude/ccusage.json`
- [x] Honor `--config <path>` override
- [x] Merge `defaults` and command-specific overrides
- [x] Apply precedence: CLI args > `--config` > environment variables > local config > user config > legacy config > built-in defaults

### Output and determinism

- [x] Deterministic JSON output for all report modes
- [x] Deterministic redesigned default human output for `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`
- [x] Deterministic table output for representative shared-flag combinations (`--compact`, `--breakdown`, `--instances`, `--locale`)
- [x] Deterministic timezone/locale behavior for implemented flags
- [x] Golden/parity fixtures for representative mode + flag combinations
- [x] Malformed-input handling with deterministic warning counts

### Pricing and cost provenance contract

- [x] Default catalog wiring is shared by every report mode and `statusline` (`PricingCatalog::default_claude_catalog()`)
- [x] `CostMode::Auto` behavior is preserved: use raw cost when present, otherwise calculate for resolvable models, otherwise mark missing
- [x] Known model events without raw `cost_usd` produce non-zero calculated totals in normal CLI flow
- [x] Unknown/unresolvable models without raw `cost_usd` stay unresolved (`missing_entries` / `R/C/M`), with no synthetic pricing
- [ ] CLI/config pricing catalog overrides

## Explicit residual deltas

- Missing options listed as unchecked above remain out of scope for the current rewrite milestone and are not exposed by the Rust CLI.
- Pricing defaults are static and Claude-focused; non-Claude/new model identifiers rely on raw event cost fields unless added to the local catalog.
- `--timezone` does not currently accept IANA zone names (for example `Europe/Berlin`); only UTC/GMT/Z and signed fixed offsets are supported.
- `--offline` is parsed and precedence-aware, but currently operationally neutral because this rewrite does not make network calls in the report pipeline.
- CLI binary name remains `cusage-rs`.

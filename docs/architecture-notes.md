# cusage-rs architecture notes

## Runtime pipeline (implemented)

1. Discover Claude session files under the configured data root.
   - Root resolution uses explicit defaults plus `CLAUDE_CONFIG_DIR` overrides.
   - Discovery is defensive: unreadable/missing paths are surfaced as warnings, not fatal errors.
2. Parse JSONL events into a stable internal usage model.
   - Parser accepts multiple key-shape variants for timestamps, ids, project/model metadata, token usage, and raw cost fields.
   - Malformed lines/files are skipped with warning accounting.
3. Aggregate by report mode: day, month, session, or billing block.
   - Weekly and statusline modes are also implemented.
4. Resolve pricing and derived metrics.
   - Every CLI/report path uses `PricingCatalog::default_claude_catalog()` (Claude aliases and common provider-prefixed forms).
   - Cost source attribution is preserved (`raw`, `calculated`, `missing`) and rolled into per-row and totals metrics.
   - `CostMode::Auto` is the active runtime behavior: raw cost first, calculated cost for known models when raw is absent, missing when unresolved.
5. Render either table output or JSON output.
   - JSON output is deterministic and fixture-tested for all modes.
   - Default (no custom table flags) human output for `daily`, `weekly`, `monthly`, `session`, and `blocks` uses a shared titled/aligned table renderer with summary, provenance, totals, and warning footer.
   - `statusline` non-JSON output is a compact single-line key/value summary (`model`, `session`, `today`, block state, burn, input).
   - Custom table options (`--compact`, `--breakdown`, `--instances`, locale-aware decimal comma) keep the existing deterministic text paths.

## Module boundaries (implemented)

- `main` (binary): CLI command surface, filter application, and top-level orchestration
- `config`: data-root resolution helpers (`HOME`/`CLAUDE_CONFIG_DIR` and default Claude roots)
- `runtime_config`: config-file parsing/merging and precedence layers
- `discovery`: file-system traversal and input selection
- `parser`: JSONL event decoding and normalization
- `domain`: token/cost models and report records
- `pricing`: cost-mode resolution, Claude-focused default catalog + alias normalization, model lookup, and derived metric math
- `report`: aggregation and rendering for all report modes, including shared default human table layout and statusline formatter

Aggregation and rendering live in `src/report.rs`; CLI orchestration and shared filtering live in `src/main.rs`.

## Key guarantees

- Deterministic output for JSON parity fixtures across `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`.
- Deterministic redesigned default human-readable layouts for `daily`, `weekly`, `monthly`, `session`, `blocks`, and statusline (fixture-backed).
- Stable warning accounting for discovery/parsing failures.
- Config precedence handling across legacy/user/local/custom/env/CLI layers.
- Pricing provenance visibility is preserved end-to-end (`raw`, `calculated`, `missing`) across reports and statusline totals.

## Explicit residual deltas

- Not all upstream public flags are implemented yet (`--mode`, `--debug`, `--debug-samples`, `--jq`, and multiple mode-specific options).
- Pricing defaults are static and Claude-focused; non-catalog model ids without raw event cost remain unresolved (`missing`) until catalog coverage is extended.
- No CLI/config hook currently exists for user-supplied pricing catalogs; runtime pricing mode is fixed to `CostMode::Auto` until `--mode` parity lands.
- Timezone handling is fixed-offset based; IANA zone names are not currently accepted.
- Offline mode is precedence-aware in config resolution but currently operationally neutral in the report runtime path.

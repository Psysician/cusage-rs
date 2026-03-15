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
   - Cost source attribution is preserved (`raw`, `calculated`, `missing`) and rolled into per-row and totals metrics.
5. Render either table output or JSON output.
   - JSON output is deterministic and fixture-tested for all modes.
   - Table output supports shared formatting options (`--compact`, `--breakdown`, `--instances`, locale-aware decimal formatting for configured locales).

## Module boundaries (implemented)

- `main` (binary): CLI command surface, filter application, and top-level orchestration
- `config`: data-root resolution helpers (`HOME`/`CLAUDE_CONFIG_DIR` and default Claude roots)
- `runtime_config`: config-file parsing/merging and precedence layers
- `discovery`: file-system traversal and input selection
- `parser`: JSONL event decoding and normalization
- `domain`: token/cost models and report records
- `pricing`: cost-mode resolution, model pricing lookup, and derived metric math
- `report`: aggregation and rendering for all report modes

Aggregation and rendering live in `src/report.rs`; CLI orchestration and shared filtering live in `src/main.rs`.

## Key guarantees

- Deterministic output for JSON parity fixtures across `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`.
- Stable warning accounting for discovery/parsing failures.
- Config precedence handling across legacy/user/local/custom/env/CLI layers.

## Explicit residual deltas

- Not all upstream public flags are implemented yet (`--mode`, `--debug`, `--debug-samples`, `--jq`, and multiple mode-specific options).
- Timezone handling is fixed-offset based; IANA zone names are not currently accepted.
- Offline mode is precedence-aware in config resolution but currently operationally neutral in the report runtime path.

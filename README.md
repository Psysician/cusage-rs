# cusage-rs

`cusage-rs` is a clean-room Rust rewrite of the public [`ryoppippi/ccusage`](https://github.com/ryoppippi/ccusage) CLI contract for Claude usage/session data.

This repository is intentionally separate from Node-based forks. Upstream behavior and docs are the reference contract; upstream source code is not copied.

## Status

Core rewrite goals are implemented for the main report pipeline:

- report modes: `daily`, `weekly`, `monthly`, `session`, `blocks`, `statusline`
- shared report flags: `--since`, `--until`, `--json`, `--breakdown`, `--compact`, `--instances`, `--project`, `--timezone`, `--locale`, `--config`, `--offline`, `--no-offline`
- statusline flags: `--json`, `--config`, `--offline`, `--no-offline`
- data discovery from Claude project roots (`~/.config/claude/projects`, `~/.claude/projects`, and `CLAUDE_CONFIG_DIR` overrides)
- config-file loading/precedence across legacy, user, local, environment, custom config path, and CLI args
- deterministic JSON output with fixture-driven parity checks

## Verification and Parity Coverage

Use:

```bash
make verify
```

Parity/fixture coverage includes:

- mode fixtures: `tests/parity_daily.rs`, `tests/parity_weekly.rs`, `tests/parity_monthly.rs`, `tests/parity_session.rs`, `tests/parity_blocks.rs`, `tests/parity_statusline.rs`
- CLI flag/error combinations: `tests/parity_cli.rs`
- malformed JSONL tolerance and deterministic warning counts via `tests/fixtures/**/malformed`

## Explicit Residual Deltas

Documented upstream options not yet implemented in this rewrite:

- global flags: `--mode`, `--debug`, `--debug-samples`, `--jq`
- weekly: `--start-of-week`
- session: `--id`
- blocks: `--active`, `--recent`, `--token-limit`, `--session-length`, `--live`, `--refresh-interval`
- statusline: `--cache`, `--refresh-interval`

Additional explicit deltas:

- `--timezone` currently accepts UTC/GMT/Z and fixed signed offsets (`+HH`, `+HHMM`, `+HH:MM`, and `UTC/GMT` prefixed forms), not IANA zone names such as `Europe/Berlin`
- `--offline` is parsed and merged through config/env/CLI precedence, but is currently operationally neutral because this rewrite does not perform network fetches in the report pipeline
- binary name is currently `cusage-rs`

## Repository Docs

- `docs/parity-inventory.md`: contract checklist, test coverage, and residual deltas
- `docs/architecture-notes.md`: implemented runtime pipeline and module boundaries
- `plans/m1-bootstrap-and-contract-harness.md`: initial bootstrap milestone record

## License

MIT

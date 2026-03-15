# cusage-rs

`cusage-rs` is a fresh Rust rewrite scaffold for [`ryoppippi/ccusage`](https://github.com/ryoppippi/ccusage).

This repository is intentionally separate from the existing [`Psysician/cusage`](https://github.com/Psysician/cusage) Node-based fork. The Rust codebase starts as a clean-room implementation: upstream behavior is the reference, but upstream source code is not copied into this repository.

## Current Status

- Cargo binary project with CI, lint/test commands, and issue templates
- Initial CLI contract scaffold for `daily`, `monthly`, `session`, `blocks`, and `statusline`
- Upstream parity checklist refreshed in `docs/parity-inventory.md` from current public docs snapshot
- Project docs capturing parity targets, architecture boundaries, and milestone acceptance criteria

## Planned Compatibility

The first parity line targets the upstream `ccusage` CLI for Claude usage data:

- report modes `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`
- shared/global flags including `--since`, `--until`, `--json`, `--breakdown`, `--compact`, `--mode`, `--offline`, `--timezone`, `--locale`, `--config`, and debug options
- command-specific flags such as `--instances`, `--project`, `--start-of-week`, `--id`, and block/statusline refresh controls
- local JSONL discovery rooted in both `~/.config/claude/projects` and `~/.claude/projects`, including `CLAUDE_CONFIG_DIR` overrides
- documented config-file hierarchy and precedence behavior (`.ccusage/ccusage.json`, user config, and legacy config)

Fork-only behavior from the current `Psysician/cusage` repository, including Codex aggregation, is out of scope for this bootstrap and will be reconsidered only after upstream parity is established.

## Development

Prerequisites:

- Rust `1.93.0`
- `make`

Verification:

```bash
make verify
```

Current binary behavior is intentionally narrow:

```bash
cargo run -- daily --since 20250525 --json
```

The command surface is present. Discovery, parser normalization, and pricing/derived-metric primitives are implemented; report-mode aggregation and rendering are still pending.

## Repository Docs

- `docs/parity-inventory.md`: upstream command and behavior contract to match
- `docs/architecture-notes.md`: intended runtime pipeline and module boundaries
- `plans/m1-bootstrap-and-contract-harness.md`: first implementation milestone and acceptance criteria

## License

MIT

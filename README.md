# cusage-rs

`cusage-rs` is a fresh Rust rewrite scaffold for [`ryoppippi/ccusage`](https://github.com/ryoppippi/ccusage).

This repository is intentionally separate from the existing [`Psysician/cusage`](https://github.com/Psysician/cusage) Node-based fork. The Rust codebase starts as a clean-room implementation: upstream behavior is the reference, but upstream source code is not copied into this repository.

## Current Status

- Cargo binary project with CI, lint/test commands, and issue templates
- Initial CLI contract scaffold for `daily`, `monthly`, `session`, `blocks`, and `statusline`
- Project docs capturing parity targets, architecture boundaries, and milestone acceptance criteria

## Planned Compatibility

The first parity line targets the upstream `ccusage` CLI for Claude usage data:

- `daily`, `monthly`, `session`, `blocks`, and `statusline` command modes
- shared filters such as `--since`, `--until`, `--json`, `--breakdown`, `--timezone`, and `--locale`
- instance grouping, project filtering, and compact rendering expectations
- local JSONL discovery from `~/.claude/projects`

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

The command surface is present, but the parser, aggregation, pricing, and renderer implementations are still pending.

## Repository Docs

- `docs/parity-inventory.md`: upstream command and behavior contract to match
- `docs/architecture-notes.md`: intended runtime pipeline and module boundaries
- `plans/m1-bootstrap-and-contract-harness.md`: first implementation milestone and acceptance criteria

## License

MIT


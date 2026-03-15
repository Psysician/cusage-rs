# cusage-rs parity inventory

## Scope and snapshot

- Behavioral target: [`ryoppippi/ccusage`](https://github.com/ryoppippi/ccusage)
- Snapshot date: `2026-03-15`
- Rewrite constraint: clean-room Rust implementation
- Legend: `[x]` implemented and parity-tested, `[ ]` not yet parity-complete (including scaffold-only coverage)

## Upstream contract sources used

- [Command-Line Options](https://ccusage.com/guide/cli-options)
- [Configuration Files](https://ccusage.com/guide/config-files)
- [Configuration Overview](https://ccusage.com/guide/configuration)
- [Directory Detection](https://ccusage.com/guide/directory-detection)
- [Environment Variables](https://ccusage.com/guide/environment-variables)
- [Upstream README](https://github.com/ryoppippi/ccusage)

## Current local baseline

- Current Rust CLI scaffold covers `daily`, `monthly`, `session`, `blocks`, and `statusline` command parsing in `src/main.rs`.
- `weekly` mode and most runtime behavior remain unimplemented.
- Discovery/parsing and pricing primitives are implemented locally; report aggregation, mode-specific behavior, and rendering parity remain incomplete.

## Parity checklist

### Report modes

- [ ] `daily`
- [ ] `weekly`
- [ ] `monthly`
- [ ] `session`
- [ ] `blocks`
- [ ] `statusline`

### Shared/global CLI contract

- [ ] `--since YYYYMMDD`
- [ ] `--until YYYYMMDD`
- [ ] `--json` / `-j`
- [ ] `--breakdown` / `-b`
- [ ] `--compact` (README-documented compact table mode)
- [ ] `--mode auto|calculate|display`
- [ ] `--offline` / `-O`
- [ ] `--timezone <tz>` / `-z`
- [ ] `--locale <locale>` / `-l`
- [ ] `--config <path>`
- [ ] `--debug`
- [ ] `--debug-samples <n>`
- [ ] `--jq <filter>` (JSON post-processing)

### Command-specific CLI contract

- [ ] `daily`: `--instances` / `-i`, `--project <name>` / `-p`
- [ ] `weekly`: `--start-of-week monday|sunday`
- [ ] `session`: `--id <session-id>`, `--project <name>`
- [ ] `blocks`: `--active`, `--recent`, `--token-limit`, `--session-length`, `--live`, `--refresh-interval`
- [ ] `statusline`: `--offline`, `--cache`, `--refresh-interval`

### Discovery and data-root behavior

- [x] Default root discovery merges both `~/.config/claude/projects` and `~/.claude/projects`
- [x] `CLAUDE_CONFIG_DIR` supports one custom root
- [x] `CLAUDE_CONFIG_DIR` supports comma-separated multiple roots with aggregation
- [x] Invalid/unreadable roots are handled defensively without corrupting totals

### Configuration file behavior

- [ ] Detect local config at `.ccusage/ccusage.json`
- [ ] Detect user config at `~/.config/claude/ccusage.json`
- [ ] Detect legacy config at `~/.claude/ccusage.json`
- [ ] Honor `--config <path>` override
- [ ] Merge `defaults` and command-specific overrides
- [ ] Apply documented precedence: CLI args > `--config` > environment variables > local config > user config > legacy config > built-in defaults

### Output and determinism

- [ ] Deterministic JSON output for all report modes
- [ ] Deterministic table rendering with stable ordering and totals
- [ ] Deterministic timezone/locale behavior for grouping and display
- [ ] Golden/parity fixtures for representative mode + flag combinations

## Upstream doc divergences to track

- README (`apps/ccusage/README.md`) documents `--compact` and does not list `weekly` in the quick usage block.
- `ccusage.com/guide/cli-options` documents `weekly` and additional global options (`--mode`, `--config`, `--debug`, `--debug-samples`, `--jq`) but does not document `--compact` on that page.
- Until black-box parity tests confirm runtime behavior, this checklist tracks the union of these public docs and flags the discrepancy explicitly.

## Explicit bootstrap deferrals

- Provider-specific behavior beyond the Claude session-file contract (for example OpenAI/Codex-specific features in other ecosystems) remains out of scope until core Claude parity is complete.
- Release packaging and alias strategy for final binary naming remains out of scope for the current milestone.

# cusage-rs parity inventory

## Source of truth

- Behavioral target: `ryoppippi/ccusage`
- Rewrite constraint: clean-room Rust implementation
- Non-target for bootstrap: current `Psysician/cusage` fork behavior beyond notes called out below

## Commands to match

- `daily`: default report mode with date grouping
- `monthly`: month-level aggregation
- `session`: conversation session grouping
- `blocks`: billing-window grouping
- `statusline`: compact status output for hooks

## Flags and inputs to preserve

- `--since YYYYMMDD`
- `--until YYYYMMDD`
- `--json`
- `--breakdown`
- `--compact`
- `--instances`
- `--project <name>`
- `--timezone <tz>`
- `--locale <locale>`

## Data and behavior invariants

- Read Claude usage data from local JSONL session files under `~/.claude/projects`
- Aggregate tokens and costs without requiring a network round-trip for session discovery
- Preserve report-mode semantics before redesigning table layout
- Prefer deterministic fixtures and golden outputs over ad hoc manual verification

## Explicitly deferred from bootstrap

- Codex/OpenAI aggregation from the current `Psysician/cusage` fork
- Compatibility shims for the fork's custom pricing and cache behavior
- Release packaging and alias strategy for a final `cusage` binary name


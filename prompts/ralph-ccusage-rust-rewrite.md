# PRD: Complete Rust Rewrite Of `ccusage`

## Goal

Rewrite the public `ryoppippi/ccusage` CLI into this Rust repository as a clean-room implementation that preserves documented end-user behavior while replacing the TypeScript/Node runtime with Rust.

Output `<promise>RUST_REWRITE_COMPLETE</promise>` only when the rewrite is genuinely complete by the criteria below.

## Working Context

- Repository: `cusage-rs`
- Existing local guidance:
  - `README.md`
  - `docs/parity-inventory.md`
  - `docs/architecture-notes.md`
  - `plans/m1-bootstrap-and-contract-harness.md`
- Upstream contract source:
  - public `ccusage` documentation and README
  - upstream CLI behavior when it can be verified safely

## Scope

Deliver a production-grade Rust rewrite of the main `ccusage` CLI application and its documented user-facing behavior.

Treat the rewrite target as the primary CLI surface, including:

1. command coverage
2. CLI flags and filtering behavior
3. data discovery and parsing from local usage files
4. aggregation and pricing behavior
5. table and JSON rendering
6. config-file and custom-path behavior that is part of the documented CLI contract
7. documentation and verification artifacts needed to maintain parity

Do not spend iterations on non-essential upstream ecosystem pieces such as website plumbing, npm packaging, or companion tools unless they are required to preserve the documented behavior of the main `ccusage` CLI.

## Hard Constraints

1. This is a clean-room rewrite. Do not copy upstream source code into this repository.
2. Use upstream behavior and documentation as the reference contract, not upstream implementation details.
3. Prefer fixture-driven and golden-output verification over subjective manual checks.
4. Keep the Rust code maintainable. Avoid a giant monolithic `main.rs`.
5. Preserve or improve this repo's existing docs instead of bypassing them.
6. Do not mark completion while major command modes, filters, or verification gaps remain.
7. If you discover a documented upstream feature that is larger than the current local scope, update local docs and implement it instead of silently ignoring it.

## Required Upstream Parity Areas

At minimum, confirm and implement the documented public behavior for:

- report modes: `daily`, `weekly`, `monthly`, `session`, `blocks`, `statusline`
- shared filters and output switches such as `--since`, `--until`, `--json`, `--breakdown`, `--compact`, `--instances`, `--project`, `--timezone`, `--locale`
- documented operational flags such as `--offline`, custom path support, and config-file driven behavior where applicable
- local usage discovery rooted in Claude usage/session data on disk
- deterministic output for both human-readable tables and machine-readable JSON

If upstream docs or safe black-box verification reveal additional contract-critical flags or behaviors, bring them into scope and document them locally.

## Execution Strategy

1. Start by reconciling the current local docs with upstream public docs and CLI behavior.
2. Produce or update a concrete parity checklist in-repo so progress is auditable.
3. Build the rewrite incrementally, but drive toward full parity rather than stopping at scaffold milestones.
4. Add or expand fixture corpora for representative local usage data, edge cases, and regression scenarios.
5. Add parity tests for each command mode and for major flag combinations.
6. Verify behavior continuously with Rust tests and any safe black-box comparison harnesses you create.
7. Update docs as the implementation becomes real so the repo is self-describing at the end.

## Implementation Requirements

1. Implement a robust runtime pipeline:
   - discovery
   - JSONL/event parsing
   - normalized domain model
   - aggregation by report mode
   - pricing and derived metrics
   - rendering for table and JSON output
2. Keep module boundaries explicit. Favor focused modules over tightly coupled command-specific logic.
3. Add support for weekly mode if it is missing locally.
4. Support custom data roots and documented config-driven behavior.
5. Preserve deterministic locale/timezone handling where upstream documents or behavior make it observable.
6. Handle malformed or partial input files defensively without corrupting aggregate results.
7. Keep performance reasonable for large local histories. Avoid obviously quadratic scans when a linear or batched design is possible.

## Verification Requirements

You are not done until all of the following are true:

1. `make verify` passes.
2. Rust unit/integration tests cover every public report mode.
3. Fixture-based parity tests exist for representative command outputs.
4. JSON output is stable enough for golden-file comparison.
5. Local docs clearly state what matches upstream and what, if anything, remains intentionally different.
6. There are no placeholder implementations for core report modes or core filters.
7. The CLI is usable end-to-end from this repository without relying on the upstream TypeScript codebase.

## Deliverables

1. Completed Rust implementation of the `ccusage` CLI contract in this repo.
2. Fixture corpus and parity/golden tests.
3. Updated repository docs reflecting actual behavior and remaining deltas, if any.
4. A brief final summary describing what was implemented, how parity was verified, and any explicitly documented residual gaps.

## Completion Gate

Only output this exact line when everything above is true:

`<promise>RUST_REWRITE_COMPLETE</promise>`

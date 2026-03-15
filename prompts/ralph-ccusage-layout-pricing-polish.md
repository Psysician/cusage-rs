# PRD: Fix Human Layout And Pricing In `cusage-rs`

## Goal

Make the human-readable CLI output production-usable and ensure prices appear whenever the local event data and known model pricing make that possible.

Output `<promise>LAYOUT_AND_PRICING_COMPLETE</promise>` only when the work below is genuinely done.

## Working Context

- Repository: `cusage-rs`
- Active task file: `.ralph/ralph-tasks.md`
- Existing repo docs:
  - `README.md`
  - `docs/parity-inventory.md`
  - `docs/architecture-notes.md`
- Ralph workflow contract:
  - `open-ralph-wiggum` README and local `ralph --help`
  - Tasks mode uses `.ralph/ralph-tasks.md`
  - Prompt files are supported via `--prompt-file`

## Known Current Gaps

1. The current non-JSON tables and statusline output are technically functional but read like placeholder/debug output rather than intentional CLI design.
2. Report entrypoints currently instantiate `PricingCatalog::new()` without populating model pricing, so events that lack raw `cost_usd` often resolve to missing/zero cost even when the model is known.

## Scope

In scope:

- human-readable output for `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`
- pricing catalog coverage and wiring for the current Claude-focused rewrite
- deterministic regression coverage for layout and pricing behavior
- docs that describe the new behavior accurately

Out of scope:

- website work
- npm packaging/release automation
- unrelated feature expansion beyond what is needed to fix layout and pricing quality

## Requirements

1. Add a real default pricing catalog for Claude model names and common aliases/provider-prefixed forms seen in this rewrite.
2. Wire every report path and `statusline` to use the same populated pricing catalog instead of an empty one.
3. Preserve cost provenance honestly:
   - use raw cost when present
   - calculate cost when the model is known and raw cost is absent
   - keep cost marked missing when the model cannot be resolved
4. Redesign the default human-readable report layouts so they are clearly structured, aligned, and intentional rather than raw whitespace dumps.
5. Keep deterministic output and preserve the existing `--compact`, `--breakdown`, `--instances`, `--locale`, and JSON behavior unless an intentional improvement is required.
6. Improve `statusline` hook output so it remains compact but is easier to scan quickly.
7. Add or update fixture-driven tests for:
   - known-model events without raw cost fields producing non-zero calculated cost
   - stable text output for the redesigned tables/statusline
   - existing parity behavior that must not regress
8. Update docs so they describe pricing coverage, output behavior, and any explicit residual deltas.

## Constraints

1. Keep this a clean-room rewrite. Do not copy upstream source.
2. Use fixture-driven verification wherever possible.
3. Keep module boundaries maintainable; do not collapse more logic into a giant `main.rs`.
4. Do not fake prices for unknown models.
5. Do not claim parity for behavior that is still missing or only partially supported.

## Acceptance Criteria

1. `make verify` passes.
2. At least one regression test proves that a known model without raw `cost_usd` now produces a non-zero calculated price through the normal CLI/report path.
3. Human-readable outputs for the main modes are materially more readable than the current placeholder-style tables.
4. `statusline` non-JSON output is compact and clearer to scan.
5. Docs describe the pricing and layout behavior accurately, including any remaining gaps.

## Completion Promise

`<promise>LAYOUT_AND_PRICING_COMPLETE</promise>`

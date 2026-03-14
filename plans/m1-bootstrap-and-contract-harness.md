# Milestone 1: bootstrap and contract harness

## Goal

Establish an implementation-ready Rust baseline for `cusage-rs` with enough fixture coverage and contract detail to begin parser and aggregation work safely.

## Deliverables

- Stable CLI surface for upstream command modes and documented flags
- Fixture corpus covering representative Claude session JSONL inputs
- Golden expectations for `daily`, `monthly`, and `session` output
- Initial parser/domain scaffolding for local session discovery

## Acceptance criteria

- `make verify` passes in CI and locally
- Fixture tests can compare deterministic JSON or normalized text outputs
- The team can add parser work without reopening command-shape or parity-boundary decisions
- Deferred fork-only Codex behavior remains explicitly out of scope

## First implementation slice

- Build a fixture runner around local JSONL samples
- Implement discovery and parser primitives for Claude session files
- Produce `daily` JSON output first, then expand to table rendering and other report modes


# cusage-rs architecture notes

## Runtime pipeline

1. Discover Claude session files under the configured data root.
2. Parse JSONL events into a stable internal usage model.
3. Aggregate by report mode: day, month, session, or billing block.
4. Resolve pricing and derived metrics.
5. Render either table output or JSON output.

## Planned module boundaries

- `config`: CLI parsing and runtime settings
- `discovery`: file-system traversal and input selection
- `parser`: JSONL event decoding and normalization
- `domain`: token/cost models and report records
- `aggregate`: grouping, rollups, and date/block calculations
- `render`: terminal tables, compact views, and JSON output

## Implementation guardrails

- Keep parser and renderer separated so golden fixtures can validate aggregation independently.
- Build fixture-driven acceptance tests before broad refactors.
- Treat upstream CLI behavior as the contract until parity tests say otherwise.


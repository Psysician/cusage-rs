# Ralph Tasks

- [x] Audit the current human-readable report output and pricing flow against the existing fixtures so the remaining work is anchored in actual gaps, not assumptions.
- [x] Add a populated default pricing catalog for Claude models plus common aliases/provider-prefixed names, and wire every report/statusline path to use it instead of an empty `PricingCatalog`.
- [ ] Preserve explicit raw/calculated/missing cost provenance so known models calculate real USD totals while unknown models remain visibly unresolved.
- [ ] Redesign the default human-readable layouts for `daily`, `weekly`, `monthly`, `session`, and `blocks` so they have intentional headers, aligned columns, readable totals, and still deterministic output.
- [ ] Improve `statusline` non-JSON output so it stays compact but is easier to scan for model, session cost, today cost, active block state, and burn rate.
- [ ] Add or update deterministic CLI/text fixtures and regression tests covering both the new layout behavior and cost calculation for events that do not include raw cost fields.
- [ ] Update `README.md` and parity/architecture docs to describe the pricing catalog behavior, layout improvements, and any explicit remaining pricing gaps.
- [ ] Run `make verify` plus representative CLI smoke checks, and only then mark the work complete.

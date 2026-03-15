# Ralph Tasks

- [x] Reconcile upstream `ccusage` public docs and CLI contract with local repo docs, and update the parity checklist.
- [x] Implement filesystem discovery and configurable data-root selection for local Claude usage/session files.
- [x] Implement robust JSONL event parsing and normalization into stable Rust domain models.
- [x] Implement pricing and derived metric calculation consistent with documented upstream behavior.
- [x] Implement `daily` report mode with deterministic JSON output and parity fixtures.
- [x] Implement `weekly` report mode with deterministic JSON output and parity fixtures.
- [x] Implement `monthly` report mode with deterministic JSON output and parity fixtures.
- [x] Implement `session` report mode with deterministic JSON output and parity fixtures.
- [x] Implement `blocks` report mode with deterministic JSON output and parity fixtures.
- [x] Implement `statusline` output mode and verify compact hook-oriented behavior.
- [ ] Implement shared flags and filtering behavior: `--since`, `--until`, `--json`, `--breakdown`, `--compact`, `--instances`, `--project`, `--timezone`, `--locale`.
- [ ] Implement documented config-file behavior, custom path support, and offline/custom-root behavior where part of the public contract.
- [ ] Add golden and parity integration tests for representative command/flag combinations and malformed-input edge cases.
- [ ] Ensure `make verify` passes and remove placeholder implementations for core report modes and filters.
- [ ] Update repository docs to describe implemented behavior, parity coverage, and any explicit residual deltas.

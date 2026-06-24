---
phase: 6
name: "Testing Implementation"
status: Done
subsystem: "crates/nodus"
requires:
  - phase-5
provides:
  - TestBlock AST node (structured input/expected/tags/raw_lines fields)
  - evaluate_test_block() assertion evaluator in workflows.rs
  - test() / test_with_tags() public API with NT-1…NT-10 compliance
  - TestOptions / TestReport / TestResult types
  - E015 / W001 / W002 validator diagnostics
  - tests/testing.rs (7 integration tests)
  - l2-nodus-testing.md v1.0.0 (Rust L2 testing spec, Stable)
key_files:
  - crates/nodus/src/ast.rs
  - crates/nodus/src/parser.rs
  - crates/nodus/src/transpiler.rs
  - crates/nodus/src/workflows.rs
  - crates/nodus/src/validator.rs
  - crates/nodus/tests/testing.rs
patterns_established: []
duration_minutes: 0
---

# Phase 6 — Testing Implementation

Implement the full `@test:` block contract from `l1-nodus-testing.md`. Raises nodus from a
stub-level test runner to a first-class, assertion-evaluating test facility per NT-1…NT-10.

**Specs covered**: `l1-nodus-testing.md` (Stable)

## Track A — AST Restructuring (Parser)

Structured `@test:` body AST: replace `TestBlock.raw_lines` with typed `input`/`expected`/`tags`
fields so the assertion evaluator can work against structured data rather than raw text.

- [x] **T-6A01** — Extend `TestBlock` AST node in `ast.rs`
  - Add `input: Vec<(String, String)>`, `expected: Vec<(String, String)>`, `tags: Vec<String>` fields
  - Keep `raw_lines: Vec<String>` as deprecated read-only companion (backward compat for transpiler)
  - **Verify**: `cargo check -p nodus` passes; `TestBlock` compiles with new fields

- [x] **T-6A02** — Update `parse_test_block()` in `parser.rs`
  - Parse `input:` key-value lines into `TestBlock.input`
  - Parse `expected:` key-value lines into `TestBlock.expected`
  - Parse `tags:` list into `TestBlock.tags`
  - Produce E015 diagnostic when duplicate `@test:` name detected in same file (NT-9 / §4.6)
  - **Verify**: `cargo test -p nodus` — all existing tests pass; new unit test `parse_test_block_structured` in `parser.rs #[cfg(test)]` asserts structured fields populated correctly

- [x] **T-6A03** — Update `Transpiler` in `transpiler.rs` to preserve round-trip fidelity
  - `to_nodus` emits `input:` / `expected:` / `tags:` blocks from typed fields (not `raw_lines`)
  - `to_human` emits readable prose for each assertion
  - **Verify**: `cargo test -p nodus` — parity tests pass; transpiler round-trip test covers test blocks

## Track B — Assertion Evaluator

Implement per-block assertion evaluation per NT-3 / §4.2 and §4.3.

- [x] **T-6B01** — Implement `evaluate_test_block()` helper in `workflows.rs`
  - For each `expected:` entry: look up variable in `RunResult` context, compare with `Value` equality
  - If variable absent from execution context → assertion fail (NT-3)
  - Type mismatch is assertion failure (not an error) per §4.3
  - Returns `(bool, String)` — (passed, message with first failing assertion name or "ok")
  - **Verify**: unit test `evaluate_test_block_passes` and `evaluate_test_block_fails_on_mismatch` in `workflows.rs #[cfg(test)]`

- [x] **T-6B02** — Rewrite `test()` function in `workflows.rs` for full assertion evaluation
  - Replace stub "pass if Status::Ok" logic with per-block `evaluate_test_block()` calls
  - Implement NT-1 (block isolation): each block starts a fresh `Executor` context
  - Implement NT-2 (input override): merge `TestBlock.input` over `@in:` defaults before each run
  - Implement NT-5 (provider neutrality): use `Executor::with_stub()` for each block
  - Empty `expected:` still passes trivially on `Status::Ok` (W002 advisory — documented, not enforced)
  - **Verify**: `cargo test -p nodus` — existing `test_runs_all_test_blocks` updated; new tests verify assertion pass/fail per block independently

## Track C — Validator & API Delta

Emit `W001` / `W002` / `E015` diagnostics and expose tag-based filtering in the public API.

- [x] **T-6C01** — Extend `Validator` in `validator.rs` with NT-10 diagnostics
  - Emit `W001` (warning-severity) for `ROUTE(wf:x)` steps with no `@test:` block covering them
  - Emit `W002` (warning-severity) for `@test:` blocks with no `expected:` entries
  - E015 (duplicate test name) is a parse-time error already wired in T-6A02; confirm validator does not double-report
  - **Verify**: unit tests `validator_w001_route_uncovered`, `validator_w002_no_assertions` in `validator.rs #[cfg(test)]`

- [x] **T-6C02** — Expose `tags` filtering in `test()` public API
  - Add optional `tags` parameter (or a `TestOptions { tags: Option<Vec<String>> }` struct) to `test()`
  - When `tags` is `Some`, skip blocks whose `tags` list does not intersect the filter set (NT-6 / §4.5)
  - Skipped blocks do not appear in `TestReport.results`
  - Update `lib.rs` re-exports as needed
  - **Verify**: unit test `test_tag_filter_skips_unmatched` in `workflows.rs #[cfg(test)]`

## Track T — Tests & Spec Sync

Integration tests proving NT-1…NT-10 compliance; author `l2-nodus-testing.md` as the Rust L2 spec.

- [x] **T-6T01** — Integration tests in `tests/testing.rs`
  - `block_isolation` — two blocks that bind `→ $shared` verify they do not share the variable (NT-1)
  - `input_override` — block with `input: query: "x"` produces `$out` reflecting override (NT-2)
  - `expected_assertion_pass` — `expected: $out: "x"` passes when output matches (NT-3)
  - `expected_assertion_fail` — `expected: $out: "wrong"` fails with descriptive message (NT-4)
  - `tag_filter` — tag predicate skips non-matching blocks; report has reduced count (NT-6)
  - `ordered_report` — `TestReport.results` order matches `@test:` declaration order (NT-7)
  - **Verify**: `cargo test -p nodus -- --test testing` — all 7 tests pass

- [x] **T-6T02** — Author `l2-nodus-testing.md` spec (Layer 2)
  - `Implements: l1-nodus-testing.md`
  - NT-1…NT-10 compliance table mapping each invariant to the Rust implementation
  - `TestBlock` AST node structure (structured fields post-Track-A)
  - `test()` function signature, `TestOptions`, `TestReport` / `TestResult` types
  - Document stub-provider requirement (NT-5) and current E015/W006/W009 diagnostic codes
  - Register in `.design/nodus/INDEX.md`; update `l1-nodus-testing.md` Related Specifications link
  - **Verify**: INDEX.md entry present; l1-nodus-testing.md Related Specifications updated

- [x] **T-6T03** — Quality gates
  - `cargo clippy -p nodus --all-targets -- -D warnings` — zero lints
  - `cargo fmt -p nodus` — format clean
  - `cargo test -p nodus` — all 204 tests pass (target met: 175+)
  - `cargo doc -p nodus --no-deps` — 1 pre-existing doc warning for function named `test` (not introduced by Phase 6)
  - **Verify**: all four commands exit 0; doc warning is pre-existing baseline, not new

## Status

**Status:** Done

## Notes

Track A must complete before Track B (assertion evaluator requires typed AST). Track C is independent
of Track B and can run in parallel once Track A passes `cargo check`. Track T runs last, after all
implementation tracks are complete.

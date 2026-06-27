---
phase: 8
name: "Error Taxonomy (l2-nodus-errors)"
status: Todo
subsystem: "crates/nodus"
requires:
  - phase-7
provides: []
key_files: []
patterns_established: []
duration_minutes: 0
---

# Phase 8 — Error Taxonomy (l2-nodus-errors)

Implement the 24-code error taxonomy from `l1-nodus-language.md` §4.6 in
`crates/nodus`, per `l2-nodus-errors.md`. Adds `ErrorSeverity`/`ErrorCategory`
metadata, the per-code severity×category registry with an `error_meta()` lookup,
and the supersede of the catch-all `NODUS:EXECUTION_FAILED`. This is the
foundational cluster of the upstream-parity gap — control-flow, dialog, and
operator clusters all reference codes defined here.

**Specs covered**: `l2-nodus-errors.md` (Stable); contract from
`l1-nodus-language.md` §4.5/§4.6.

**Defined-ahead codes**: several codes name features not yet built
(`SWITCH_NO_MATCH`, `DIALOG_TIMEOUT`/`DIALOG_REJECTED`/`PAUSED`, `KB_UNAVAILABLE`,
`CONFIDENCE_LOW`, `COUNTER_OVERFLOW`, `GIT_UNAVAILABLE`). This phase defines their
constants and metadata; their emission sites land with their cluster. The
lockstep test (Track T) checks metadata presence, not emission.

## Track A — Metadata types & registry

Pure additions in `vocab.rs`. No behavior change to existing runs.

- [ ] **T-8A01** — Add `ErrorSeverity` + `ErrorCategory` enums to `vocab.rs`
  - `ErrorSeverity { Error, Warn, Info }`; `ErrorCategory { Parse, Runtime, Validation, Routing, Memory, Test, Control, Dialog }` (the §4.1 types)
  - Derive `Debug, Clone, Copy, PartialEq, Eq`
  - **Verify**: `cargo test -p nodus` — unit test `error_meta_types_construct` in `vocab.rs #[cfg(test)]` constructs each variant; `cargo check -p nodus` passes

- [ ] **T-8A02** — Add the 14 new `error_code` constants
  - `UNDEFINED_CMD`, `UNDEFINED_MACRO`, `VALIDATION_FAILED`, `ESCALATION_FAILED`, `CONFIDENCE_LOW`, `KB_UNAVAILABLE`, `MEMORY_FAILED`, `TEST_FAILED`, `SWITCH_NO_MATCH`, `PAUSED`, `COUNTER_OVERFLOW`, `GIT_UNAVAILABLE`, `DIALOG_TIMEOUT`, `DIALOG_REJECTED` — each a `NODUS:*` string constant
  - **Verify**: unit test `new_error_codes_are_namespaced` in `vocab.rs #[cfg(test)]` asserts all 14 start with `"NODUS:"` and are pairwise distinct

- [ ] **T-8A03** — Add the severity×category registry + `error_meta()` lookup
  - Static table mapping each canonical code → `(ErrorSeverity, ErrorCategory)` per `l2-nodus-errors.md` §4.2; `pub fn error_meta(code: &str) -> Option<(ErrorSeverity, ErrorCategory)>`
  - **Verify**: unit tests in `vocab.rs #[cfg(test)]` — `error_meta("NODUS:PARSE_ERROR") == Some((Error, Parse))`, `error_meta("NODUS:SWITCH_NO_MATCH") == Some((Warn, Control))`, `error_meta("NODUS:PAUSED") == Some((Info, Control))`

## Track B — Supersede & site reassignment

Retire the catch-all and route existing generic sites to specific codes.

- [ ] **T-8B01** — Deprecate `EXECUTION_FAILED` and exclude it from the canonical registry
  - Mark the constant `#[deprecated(note = "...")]`; `error_meta` returns `None` for it (non-canonical legacy, §4.4)
  - **Verify**: unit test `execution_failed_is_non_canonical` asserts `error_meta("NODUS:EXECUTION_FAILED").is_none()`

- [ ] **T-8B02** — Reassign existing generic emission sites to specific codes
  - Where the executor/validator currently emit a generic/catch-all failure for a case covered by a specific code, switch to it (per §4.4 table) — e.g. an unknown dispatched command path to `UNDEFINED_CMD`, an undefined `RUN(@x)` macro to `UNDEFINED_MACRO`. Do **not** invent emission for features that do not exist yet (defined-ahead codes stay unemitted).
  - **Verify**: `cargo test -p nodus` — full suite green with no regressions; a targeted unit/integration test asserts the reassigned site now surfaces the specific code

## Track T — Validation

Registry integrity guard, then quality gates.

- [ ] **T-8T01** — Registry lockstep test
  - Assert every canonical `error_code` constant (the 24 + `CAPABILITY_UNMET`, excluding the deprecated `EXECUTION_FAILED`) has an `error_meta` row, and every metadata row names a real constant — the §4.3 guard that keeps constants and metadata in sync
  - **Verify**: `cargo test -p nodus error_registry_lockstep` passes

- [ ] **T-8T02** — Quality gates
  - `cargo clippy -p nodus --all-targets -- -D warnings` — zero lints (deprecation of `EXECUTION_FAILED` must not trip `-D warnings` at its own definition; gate internal uses with `#[allow(deprecated)]` where unavoidable)
  - `cargo fmt -p nodus` — clean; `cargo test -p nodus` — full suite green; `cargo doc -p nodus --no-deps` — no new warnings beyond the pre-existing `test`-fn baseline
  - **Verify**: all four commands exit 0; new-warning count is zero

## Status

**Status:** Todo

## Notes

Execution order is **A → B → T**: the metadata types and `error_meta` (Track A)
are the substrate Track B reassigns sites against and Track T locks down. Risk
concentrates in T-8B02 — reassigning live emission sites can shift observable
error codes; the full-suite regression check is the guard. Defined-ahead codes
(dialog, control-flow, KB, etc.) are intentionally unemitted this phase; their
emission lands when those clusters are built, and the lockstep test already
covers their metadata so the registry stays honest.

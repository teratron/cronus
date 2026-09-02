---
phase: 31
name: "Pinned-Generation Digest on ResumeDescriptor"
status: Done
subsystem: "crates/nodus/src/executor.rs, crates/nodus/tests/dialog.rs"
requires: []
provides:
  - "ResumeDescriptor.workflow_digest: String (LP-22(c)) — populated via digest_ast(ast) at the struct's one construction site"
  - "unit test: resume.workflow_digest == digest_ast(&ast) for the parsed workflow (executor.rs)"
  - "integration test: ResumeDescriptor.workflow_digest agrees with the independently-built ReproRecipe.workflow_digest for the same paused run (tests/dialog.rs, ManifestCapture)"
  - "l2-nodus-dialog.md 1.4.0 / l2-nodus-portability.md 1.10.0: LP-22 (c) reconciled to Implemented"
key_files:
  created: []
  modified:
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/tests/dialog.rs"
    - ".design/nodus/specifications/l2-nodus-dialog.md"
    - ".design/nodus/specifications/l2-nodus-portability.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "A pinning/identity claim between two independently-constructed values (ResumeDescriptor.workflow_digest vs. ReproRecipe.workflow_digest) is proven by a cross-checking test at the integration level, not by trusting that two call sites reusing the same function must agree — the two sites are never compared in production code, so the test is the only thing that makes the claim real"
duration_minutes: 20
---

# Stage 31 Tasks — Pinned-Generation Digest on ResumeDescriptor

**Phase:** 31
**Status:** Done
**Strategic Goal:** Build the LP-22(c) design `l2-nodus-dialog.md` §4.3 v1.3.0 specifies:
add `workflow_digest: String` to `ResumeDescriptor`, computed by the same `digest_ast`
function `ReproRecipe.workflow_digest` already uses, so a host can detect whether the
definition it is about to resume differs from the one pinned at suspension. The smallest
phase since Phase 25 — one field, one construction site, no new mechanism.

## Scope note (read before starting)

`l2-nodus-dialog.md` §4.3 is the design source and is complete: the field name, its
computation (`digest_ast(ast)` — reuse, not reinvent), and its consumption contract (host
compares digests at resume time; the core has no resume-time call site to enforce it itself)
are all settled. Two facts, ground against source at planning time, bound the implementation:

**1. Exactly one construction site.** `ResumeDescriptor` is built once, at `executor.rs`
(the `let resume = if paused { Some(ResumeDescriptor { .. }) } else { None };` block,
immediately followed by the `ReproRecipe { workflow_digest: digest_ast(ast), .. }` construction
a few lines later in the same function). `digest_ast` is a private `fn` in `executor.rs`
(`fn digest_ast(ast: &WorkflowFile) -> String`) — already in scope at the `resume` block, no
import needed. `ResumeDescriptor` derives only `Debug, Clone` (no `Default`, `PartialEq`, or
`Eq`), so adding a required field is a compile-time-enforced update at its one call site —
nothing can silently construct a stale value.

**2. No breaking-change fallout beyond that one site.** Grepped every `ResumeDescriptor`
reference in the crate: two integration tests (`tests/dialog.rs`, `tests/control_flow.rs`)
read `.workflow`/`.step_index` by field access, never construct or exhaustively pattern-match
the struct — both compile unchanged. This is a pre-1.0 additive field on a struct nothing
else builds (the `NoopStorageProvider` → `InMemoryStorageProvider` / `EnvironmentProfile`
fourth-parameter precedent for "acceptable pre-1.0 change," except here nothing breaks at all).
**Confirmed at implementation time**: `cargo check -p nodus --all-targets` passed on the first
try after the single field addition and single construction-site update — no other call site
needed touching, exactly as planned.

**What must not happen.** No new hashing scheme — `digest_ast` is reused verbatim, not
reimplemented or wrapped. No change to `RunResult`, `RunManifest`, or `ReproRecipe`'s own
shape (LP-22(c) is additive to `ResumeDescriptor` only). No attempt to build a resume-consuming
API — that question was the wrong one (see `l2-nodus-portability.md` §3.2's LP-22 row,
v1.9.0); this phase closes LP-22(c) exactly as designed, nothing more.

## Atomic Checklist

- [x] [T-31A01] `ResumeDescriptor.workflow_digest` field + population at the one construction site
- [x] [T-31C01] Spec reconciliation — LP-22(c) to Implemented in both carriers
- [x] [T-31T01] Unit + integration coverage: digest equals `digest_ast`, and agrees with `ReproRecipe.workflow_digest` for the same run

## Detailed Tracking

### [T-31A01] `ResumeDescriptor.workflow_digest` field + population at the one construction site

- **Spec:** l2-nodus-dialog.md §4.3 (field + computation)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. `ResumeDescriptor` gains `pub workflow_digest: String`; the sole construction site
  populates it with `digest_ast(ast)` — the identical call `ReproRecipe`'s construction a few
  lines below already makes on the same `ast` binding.
  **Satisfied**: both commands clean on the first pass; no other call site required changes,
  confirming the Scope note's grep-based prediction.
- **Handoff:** T-31T01 is the acceptance evidence; T-31C01 reconciles the spec once this lands.
- **Notes:** Added the field to the struct definition (`executor.rs:143`, beside
  `step_index`) with a doc comment naming LP-22(c) and pointing at `l2-nodus-dialog.md` §4.3.
  Populated in the `resume` binding (`executor.rs:1009-1015`): `workflow_digest:
  digest_ast(ast)`. Kept the two `digest_ast(ast)` calls (for `resume` and for `repro` a few
  lines below) visually independent rather than factoring into a shared local — deliberate,
  since T-31T01's integration test exists precisely to prove they agree without relying on
  shared state to guarantee it.
- **Changes:** `executor.rs`: `ResumeDescriptor` gained `pub workflow_digest: String` with a
  doc comment citing LP-22(c); the `resume` construction (inside the function building both
  `resume` and `repro`) gained `workflow_digest: digest_ast(ast)`.

### [T-31C01] Spec reconciliation — LP-22(c) to Implemented in both carriers

- **Spec:** l2-nodus-portability.md §3.2 (LP-22 row), l2-nodus-dialog.md §3 (DG-4 row) + §4.3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `l2-nodus-portability.md` §3.2's LP-22 row (c) changes from "designed, not yet
  built" to **Implemented**, citing the field and its test. `l2-nodus-dialog.md` §4.3's
  `[REFERENCE]` block's `[ADDED v1.3.0]` tag on `workflow_digest` becomes `[IMPLEMENTED]`.
  Both documents' Document History gain a row; `INDEX.md` version cells updated;
  `check-prerequisites --verify-headers --workspace=nodus` reports no `VERSION_DRIFT`.
  **Satisfied**: both rows updated exactly as specified; `l2-nodus-dialog.md` 1.3.0 → 1.4.0
  gained a §4.3 "Implemented [Phase 31]" paragraph citing both tests; `l2-nodus-portability.md`
  1.9.0 → 1.10.0's "What is plannable" summary dropped LP-22 (c) from its residue list.
  Pre-flight after write-back: `ok: true`, no `VERSION_DRIFT` (only the expected mid-run
  `SYNC_GAP`, `/magic.task`'s to close next).
- **Handoff:** Closes the phase's spec-sync obligation.
- **Notes:** Patch-to-minor on both — the field landed exactly as designed at v1.3.0/v1.9.0,
  no mechanism correction needed (unlike Phase 29's NE-14, where the design itself was wrong
  and had to be corrected here). LP-22's overall §3.2 verdict stays **Partially realized**
  even after this lands — (a) is satisfied structurally, (b) stays vacuous behind the absent
  import triad, and closing (c) does not change either of those; not overclaimed as fully
  realized.
- **Changes:** `l2-nodus-dialog.md` 1.3.0 → 1.4.0 (§4.3 field comment and paragraph heading
  → `[IMPLEMENTED v1.4.0]`, new "Implemented [Phase 31]" closing paragraph, Document History
  row). `l2-nodus-portability.md` 1.9.0 → 1.10.0 (§3.2 LP-22 row (c) → Implemented, §3.2
  section-heading tag updated, "What is plannable" summary rewritten to drop LP-22 (c),
  Document History row). `INDEX.md`: both version cells, top-level version 1.0.94 → 1.0.95,
  and a Meta Information entry.

### [T-31T01] Unit + integration coverage: digest equals `digest_ast`, and agrees with `ReproRecipe.workflow_digest` for the same run

- **Spec:** l2-nodus-dialog.md §4.3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` green with at least two new assertions. **Unit** (in
  `executor.rs`'s existing `#[cfg(test)] mod tests`): a workflow parsed and run to a paused
  state via the plain `Executor` path has `resume.workflow_digest == digest_ast(&ast)` for the
  same `ast` the test itself parsed. **Integration** (`tests/dialog.rs`, reusing the existing
  `ASK_PAUSE_WF` fixture and `run_with_dialog_and_audit`): a capturing `AuditProvider` captures
  the `RunManifest` for a paused run; assert `result.resume.unwrap().workflow_digest ==
  captured_manifest.repro.workflow_digest`.
  **Satisfied**: `resume_descriptor_workflow_digest_matches_digest_ast` (unit,
  `executor::tests::slice`) and `resume_descriptor_digest_agrees_with_repro_recipe_digest`
  (integration, `tests/dialog.rs`, new `ManifestCapture` provider mirroring
  `tests/observability.rs`'s `RecordingProvider` shape, trimmed to just the manifest) both
  pass. `cargo test -p nodus`: **484 passed, 0 failed** (was 482, +2); `cargo clippy -p nodus
  --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean; `git diff --stat`
  on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved).
- **Handoff:** Phase acceptance signal.
- **Notes:** Confirmed at implementation time (not just at planning): `run_complete` fires
  unconditionally in the function that also builds `resume` — the integration test's capturing
  provider observed exactly one manifest for the one paused run, as predicted at planning.
  `DefaultDialogProvider` (built-in) was reused directly for the integration test rather than a
  custom provider, since `ASK_PAUSE_WF` has no `+default` and pauses on the default resolver
  alone.
- **Changes:** `executor.rs` test module: 1 new unit test
  (`resume_descriptor_workflow_digest_matches_digest_ast`). `tests/dialog.rs`: 1 new
  integration test (`resume_descriptor_digest_agrees_with_repro_recipe_digest`) plus a small
  `ManifestCapture` `AuditProvider` and its 2 new imports (`AuditProvider`,
  `DefaultDialogProvider`, `ExecutionEvent`, `RunManifest`, `std::sync::{Arc, Mutex}`).

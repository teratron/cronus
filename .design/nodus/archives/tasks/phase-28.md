---
phase: 28
name: "Memoizable & Promotable Approval"
status: Done
subsystem: "crates/nodus/src/executor.rs, crates/nodus/src/observability.rs"
requires: []
provides:
  - "DG-9 memoizable approval — DialogOutcome::Remembered(Value) binds through Answer's exact path; DialogProvenance tags the resolved StepEnd"
  - "DG-10 confirmed to need zero code — recurrence/promotion is entirely host-side over the DG-9 provenance tag"
  - "First EventAnnotations field the crate's own dispatch logic populates directly (not host-supplied)"
key_files:
  created: []
  modified:
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/src/observability.rs"
    - "crates/nodus/src/lib.rs"
    - "crates/nodus/tests/portability.rs"
    - ".design/nodus/specifications/l2-nodus-dialog.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "EventAnnotations may carry a crate-populated field, not only host-supplied ones, when only the crate's own dispatch logic (not an external observer) can know the fact — the doc comment must say so explicitly rather than imply universal host-supply"
duration_minutes: ~
---

# Stage 28 Tasks — Memoizable & Promotable Approval

**Phase:** 28
**Status:** Done
**Strategic Goal:** Build the DG-9/DG-10 seam `l2-nodus-dialog.md` §4.7 designed: a memoized
dialog resolution carries a distinct trace provenance, with no new grammar and no new
`DialogProvider` trait method.

## Scope note (read before starting)

`l2-nodus-dialog.md` §4.7 (v1.1.0) is the authoritative design. `handle_dialog`
(`executor.rs:1657-1750`, read in full during the spec pass) already has the exact shape this
phase extends: `DialogOutcome` is matched at lines 1703-1732, and a single
`self.emit(ctx, |..| ExecutionEvent::StepEnd {..})` call at lines 1738-1747 fires once after
the match, currently always with `annotations: EventAnnotations::default()`.

**DG-9 needs no grammar work.** `+remember` is an ordinary `+modifier` — `cmd.modifiers` is
already passed raw to `DialogProvider::ask`/`confirm` (confirmed: no `KNOWN_MODIFIERS`
registry exists anywhere in the crate, the same finding LP-16's own design made). A host's
`ask`/`confirm` implementation can already inspect `modifiers` for `+remember` and do its own
recall/governance entirely inside that one method call — nothing here changes that.

**The only real code is:** (1) a new `DialogOutcome::Remembered(Value)` variant, and (2) a new
`dialog_provenance: Option<DialogProvenance>` field on `EventAnnotations`
(`observability.rs:97-113`, alongside `message`/`anomaly`/`receipt`/`durability`), set on the
`StepEnd` emission at line 1738-1747 based on which outcome resolved. `Remembered(v)` binds
through the **exact same code path** as `Answer(v)` — do not write a second `ctx.set_var`/
`ctx.log_step` call; the discipline that memoization "never bypasses `+type`/`+validate`"
holds because both variants share one binding call, not because of an added check.

**DG-10 requires no code in this phase.** Its recurrence/promotion pseudocode
(`l1-nodus-dialog.md` §4.7) is entirely host-side, computed from the host's own durable store
plus the `DialogProvenance::Remembered` tag this phase emits. Do not build a nodus-side
recurrence counter, a `!PREF` writer, or any promotion mechanism — there is nothing to task.

**Explicitly out of scope:**

- A new `DialogProvider` trait method (`recall`/`may_memoize`/`offer_remember`) — rejected in
  the spec's own §5; `ask`/`confirm` already receive everything a host needs.
- A dedicated `DialogResolved` event type — rejected in §5; the HO-8/9/10/11/13/16/17
  precedent is an optional field on an existing event, not a new variant (HO-6).
- Any DG-10 implementation.

## Atomic Checklist

- [x] [T-28A01] `DialogOutcome::Remembered` + `DialogProvenance` + `EventAnnotations` field
- [x] [T-28C01] `handle_dialog` wiring — bind through `Answer`'s path, tag `StepEnd`
- [x] [T-28T01] Validation coverage + spec reconciliation

## Detailed Tracking

### [T-28A01] `DialogOutcome::Remembered` + `DialogProvenance` + `EventAnnotations` field

- **Spec:** l2-nodus-dialog.md §4.7
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `DialogOutcome` has 5 variants
  (`Answer`/`Remembered`/`Pause`/`Timeout`/`Rejected`); `EventAnnotations::default()` produces
  `dialog_provenance: None` (a unit test alongside any existing `EventAnnotations::default()`
  regression test, if one exists — check `observability.rs`'s test module first).
  **Satisfied**: `event_annotations_default_is_all_none_and_durable` extended to assert
  `dialog_provenance.is_none()`; `dialog_provenance_answered_is_never_remembered` added.
- **Handoff:** T-28C01 wires this into `handle_dialog`.
- **Changes:** `executor.rs`: `DialogOutcome` gained `Remembered(Value)`. `observability.rs`:
  new `pub enum DialogProvenance { Answered, Remembered }` beside `EventAnnotations`; the
  struct gained a fifth field `dialog_provenance: Option<DialogProvenance>`; the struct's own
  doc comment updated to note this is the first field the crate's own dispatch logic
  populates, not a host-supplied one. `lib.rs`: `DialogProvenance` re-exported.
- **Notes:** Add `Remembered(Value)` to `DialogOutcome` (`executor.rs:230-239`) beside
  `Answer(Value)`. Add a small `pub enum DialogProvenance { Answered, Remembered }` (confirm
  whether it belongs in `executor.rs` next to `DialogOutcome`, or in `observability.rs` next
  to `EventAnnotations` — it is read by `observability.rs` but produced by `executor.rs`;
  match whichever cross-module import pattern `Measurement`/`Anomaly` already use for a
  similarly shared type). Add `pub dialog_provenance: Option<DialogProvenance>` to
  `EventAnnotations` (`observability.rs:97-113`) as a fifth field, doc-commented the same way
  as the existing four (which HO invariant it serves — this one is DG-9, not an HO invariant,
  so word the comment accordingly). Confirm `EventAnnotations::default()`'s real derivation
  (`#[derive(Default)]` vs a manual impl) before adding the field — a `Default` derive needs
  `DialogProvenance` to not require `Default` itself (an `Option<T>` defaults to `None`
  regardless of `T`, so this should be automatic, but verify the struct actually derives
  `Default` rather than implementing it by hand with an exhaustive field list that would need
  a matching addition).

### [T-28C01] `handle_dialog` wiring — bind through `Answer`'s path, tag `StepEnd`

- **Spec:** l2-nodus-dialog.md §4.7
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `cargo clippy -p nodus --all-targets -- -D
  warnings` clean; a temporary manual/integration check (formalized in T-28T01) confirms: a
  `DialogProvider` returning `Remembered(v)` binds `v` to the pipeline target identically to
  `Answer(v)` (same type coercion, same `ctx.log_step` entry); the emitted `StepEnd`'s
  `annotations.dialog_provenance` is `Some(Answered)` on `Answer`, `Some(Remembered)` on
  `Remembered`, and `None` on `Pause`/`Timeout`/`Rejected`.
  **Satisfied**: `remembered_binds_identically_to_answer`,
  `step_end_carries_dialog_provenance_answered_and_remembered`,
  `pause_timeout_rejected_carry_no_dialog_provenance` (T-28T01) all pass.
- **Handoff:** T-28T01.
- **Changes:** `handle_dialog`'s `match outcome { .. }` now returns `(signal,
  dialog_provenance)` — `Remembered(value)` gained its own arm (identical body to `Answer`'s:
  `ctx.log_step`/`ctx.set_var`, `Some(DialogProvenance::Remembered)`), `Answer` now also
  returns `Some(DialogProvenance::Answered)`, and `Pause`/`Timeout`/`Rejected` return `None`.
  The single `StepEnd` emit's `annotations` changed from `EventAnnotations::default()` to
  `EventAnnotations { dialog_provenance, ..Default::default() }` — one emit call, unchanged.
- **Notes:** In the `match outcome { .. }` block (`executor.rs:1703-1732`), add a
  `Remembered(value) =>` arm identical in body to the existing `Answer(value) =>` arm (same
  `ctx.log_step`/`ctx.set_var`/`None` return) — consider whether Rust lets you write
  `Answer(value) | Remembered(value) => { .. }` as one combined arm (bind-then-branch cannot
  distinguish which variant matched inside a combined arm, so if the provenance tag needs to
  know *which* variant fired, keep them separate arms that both compute the same bound value
  but record a different provenance). Capture the resolved `DialogProvenance` (or `None` for
  `Pause`/`Timeout`/`Rejected`) in a local variable during the match, then pass it into the
  single `self.emit(ctx, |..| ExecutionEvent::StepEnd { .. annotations: EventAnnotations { dialog_provenance: <captured>, ..Default::default() }, .. })`
  call at lines 1738-1747 — do not add a second emit call; the event count per dialog step is
  unchanged.

### [T-28T01] Validation coverage + spec reconciliation

- **Spec:** l2-nodus-dialog.md (all sections); reconciles to `INDEX.md`
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase covering: (a) a
  `DialogProvider` test double returning `Remembered("x")` for `ASK` — assert the pipeline
  target binds `"x"` exactly as `Answer("x")` would (run both through the same assertion
  helper, or assert byte-identical `RunResult.vars`); (b) the recorded `StepEnd` event (via a
  recording `AuditProvider`) carries `annotations.dialog_provenance == Some(Answered)` for an
  ordinary `Answer` and `Some(Remembered)` for `Remembered`; (c) `Pause`/`Timeout`/`Rejected`
  outcomes still record `dialog_provenance: None`; (d) every existing dialog test
  (`DefaultDialogProvider`, which never returns `Remembered`) is unaffected — a regression
  proving today's behavior byte-for-byte unchanged. `cargo clippy -p nodus --all-targets -- -D
  warnings` clean; `cargo fmt -p nodus -- --check` clean; `git diff --stat -- crates/nodus/Cargo.toml
  crates/nodus/Cargo.lock` empty (LP-1); manual scan confirming no `unwrap()`/`panic!()`/
  `expect(` added outside `#[cfg(test)]`. Run cargo via PowerShell, not Git Bash.
  **Satisfied, with one correction found empirically**: asserting no-provenance-on-rejection
  against `DEFERRED_WF` (which declares `@err: ESCALATE(human)`) initially expected one
  `StepEnd`; `DIALOG_REJECTED` is `Signal`-free so NL-9 dispatch fires automatically, landing
  **two** `StepEnd` events (the rejected `ASK`, then the dispatched `ESCALATE`), both
  correctly carrying no provenance — the test was corrected to expect `vec![None, None]`.
  All 4 new integration tests + 1 new unit test pass; 467 tests total (was 462); `cargo
  clippy -p nodus --all-targets -- -D warnings` clean; `cargo fmt -p nodus -- --check` clean;
  `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty; no `unwrap`/`panic!`/`expect` added
  outside test code.
- **Handoff:** Phase closure.
- **Changes:** `tests/portability.rs`: `DialogRemembers`, `RecordingAudit`,
  `step_end_provenance` helper, and 4 integration tests (`remembered_binds_identically_to_answer`,
  `step_end_carries_dialog_provenance_answered_and_remembered`,
  `pause_timeout_rejected_carry_no_dialog_provenance`,
  `default_dialog_provider_never_returns_remembered`). `observability.rs`'s own test module
  gained `dialog_provenance_answered_is_never_remembered` and extended the existing
  `EventAnnotations::default()` test. Reconciled `l2-nodus-dialog.md` 1.1.0 → 1.1.1 (§3
  DG-9/DG-10 rows → Implemented, §4.7 gains a confirmation note on the crate-populated-field
  finding, Document History). `INDEX.md` row + top-level version (1.0.79 → 1.0.80) + Last
  Updated synced.
- **Notes:** Reconcile `l2-nodus-dialog.md`'s `[REFERENCE]` pseudocode in §4.7 to the exact
  as-built code once T-28A01/T-28C01 land (the established Track-C precedent). `INDEX.md`'s
  row for `l2-nodus-dialog.md` and top-level version synced. No pseudocode correction was
  actually needed — §4.7's `[REFERENCE]` block already named `observability.rs` as
  `DialogProvenance`'s home, matching what was built exactly.

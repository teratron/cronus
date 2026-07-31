---
phase: 28
name: "Memoizable & Promotable Approval"
status: Todo
subsystem: "crates/nodus/src/executor.rs, crates/nodus/src/observability.rs"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 28 Tasks — Memoizable & Promotable Approval

**Phase:** 28
**Status:** Todo
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

- [ ] [T-28A01] `DialogOutcome::Remembered` + `DialogProvenance` + `EventAnnotations` field
- [ ] [T-28C01] `handle_dialog` wiring — bind through `Answer`'s path, tag `StepEnd`
- [ ] [T-28T01] Validation coverage + spec reconciliation

## Detailed Tracking

### [T-28A01] `DialogOutcome::Remembered` + `DialogProvenance` + `EventAnnotations` field

- **Spec:** l2-nodus-dialog.md §4.7
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `DialogOutcome` has 5 variants
  (`Answer`/`Remembered`/`Pause`/`Timeout`/`Rejected`); `EventAnnotations::default()` produces
  `dialog_provenance: None` (a unit test alongside any existing `EventAnnotations::default()`
  regression test, if one exists — check `observability.rs`'s test module first).
- **Handoff:** T-28C01 wires this into `handle_dialog`.
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
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `cargo clippy -p nodus --all-targets -- -D
  warnings` clean; a temporary manual/integration check (formalized in T-28T01) confirms: a
  `DialogProvider` returning `Remembered(v)` binds `v` to the pipeline target identically to
  `Answer(v)` (same type coercion, same `ctx.log_step` entry); the emitted `StepEnd`'s
  `annotations.dialog_provenance` is `Some(Answered)` on `Answer`, `Some(Remembered)` on
  `Remembered`, and `None` on `Pause`/`Timeout`/`Rejected`.
- **Handoff:** T-28T01.
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
- **Status:** Todo
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
- **Handoff:** Phase closure.
- **Notes:** Reconcile `l2-nodus-dialog.md`'s `[REFERENCE]` pseudocode in §4.7 to the exact
  as-built code once T-28A01/T-28C01 land (the established Track-C precedent). `INDEX.md`'s
  row for `l2-nodus-dialog.md` and top-level version synced.

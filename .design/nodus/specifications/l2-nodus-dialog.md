# Nodus Human-in-the-Loop Dialog Implementation (Rust)

**Version:** 1.1.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-dialog.md

## Overview

Concrete Rust realization of the dialog contract. Maps each `DG`-invariant from
`l1-nodus-dialog.md` to its enforcing mechanism in `crates/nodus`: the `ASK` and
`CONFIRM` commands and their modifiers, the `Status::Paused` run state and its
resume descriptor, the `DialogProvider` extension point with a built-in
synchronous resolver, the `ExtensionRole::Dialog` capability-manifest binding,
and the executor wiring that makes a dialog step block-then-resolve. The dialog
error codes (`DIALOG_TIMEOUT` / `DIALOG_REJECTED` / `PAUSED`) already exist in
the runtime taxonomy; this spec specifies their emission.

## Related Specifications

- [l1-nodus-dialog.md](l1-nodus-dialog.md) — the dialog contract this implements (DG-1…DG-10)
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — runtime crate extended here: `vocab` (commands, `Status`), `executor`, the `run_with_*` family
- [l2-nodus-portability.md](l2-nodus-portability.md) — LP-2 extension-point pattern + the LP-8 `ExtensionRole` taxonomy this adds `Dialog` to
- [l2-nodus-errors.md](l2-nodus-errors.md) — owns the `DIALOG_TIMEOUT`/`DIALOG_REJECTED`/`PAUSED` codes and their severity/category
- [l2-nodus-observability.md](l2-nodus-observability.md) — [ADDED v1.1.0] owns `EventAnnotations`, the HO-9/11/16/17 carrier §4.7 extends with a fifth field for DG-9's provenance signal

## 1. Motivation

`l1-nodus-dialog.md` defines a host-neutral dialog contract, but the crate has no
`ASK`/`CONFIRM` commands, no `Paused` run state, and no dialog extension point.
This spec records the minimal additions that realize the contract while keeping
the executor's run-to-completion model intact for the common (default-resolved)
case: a dialog backend trait following the established `ModelProvider`/
`AuditProvider` pattern, a built-in synchronous resolver, and a `Paused` signal
for the case a host must handle out-of-band.

## 2. Constraints & Assumptions

- No new external dependency (LP-1): the dialog backend is an in-tree trait with a built-in no-I/O resolver.
- `ASK`/`CONFIRM` are added to the schema vocabulary; existing `KNOWN_COMMANDS` and reserved variables are not mutated (LP-4 — additions only).
- The built-in resolver is synchronous and never blocks: it resolves a dialog from `+default` or signals `Status::Paused` (DG-2/DG-6).
- True cross-invocation suspend/resume is a host concern over the `Status::Paused` signal; the executor does not serialise or restore mid-run state itself.
- Raw human answers bound to variables are user data; the dialog audit events carry only typed/length descriptors, never the raw text (DG-7, reusing the observability `FieldDescriptor`).

## 3. Invariant Compliance

| DG Invariant | Rust Enforcement |
| --- | --- |
| DG-1 Host neutrality | The backend is a `DialogProvider` trait; the executor never names a UI. Built-in `DefaultDialogProvider` ships; interactive backends live outside the crate (LP-2). |
| DG-2 Blocking progression | The executor resolves a dialog step before advancing: it calls the provider, and on no resolution returns `Status::Paused` without running later steps. |
| DG-3 Typed binding | `ASK` coerces the answer to its `+type` (str/bool/confirm/choice/multi_choice) into a `Value`, then binds it to the pipeline target; a `+validate` failure re-prompts or routes per the validator. |
| DG-4 Suspend/resume | An unresolved blocking dialog and `!PAUSE` set `Status::Paused` and produce a `ResumeDescriptor` (workflow id + var snapshot + step index); re-invocation with the answer continues deterministically. |
| DG-5 Typed failures | `+timeout` → `NODUS:DIALOG_TIMEOUT`; `+strict` `CONFIRM` rejection → `NODUS:DIALOG_REJECTED`; both are `RuntimeError`s routed to `@err:`. |
| DG-6 Default-on-absence | `DefaultDialogProvider` resolves a dialog with a `+default` and otherwise signals pause; it never blocks (a non-interactive run completes or pauses, never hangs). |
| DG-7 Trace data-safety | Dialog events reuse `observability::FieldDescriptor` (type + length), never raw answer text. |
| DG-8 Capability declaration | `ExtensionRole::Dialog` is added to the manifest taxonomy; `CapabilityManifest::from_workflow` requires it for any `ASK`/`CONFIRM` whose step lacks a `+default`, so a host with no dialog backend is rejected fail-fast. |
| DG-9 Memoizable approval [ADDED v1.1.0] | `+remember` needs no new grammar (§2's existing `+modifier` pass-through already carries it to `DialogProvider::ask`/`confirm`, unread by nodus, exactly like every other undeclared modifier). Nodus's own obligation — a distinct trace provenance on a memoized resolution — realizes as a new `DialogOutcome::Remembered(Value)` variant (§4.7), bound identically to `Answer` but tagged `DialogProvenance::Remembered` on the step's `StepEnd` annotations, vs. `Answered` for the ordinary path. The host's own `ask`/`confirm` implementation performs the entire recall / governance / fit-check pseudocode of L1 §4.6 internally (it already receives the raw `+remember` modifier and returns whichever outcome applies) — no new `DialogProvider` trait method needed. |
| DG-10 Promotable remembered decision [ADDED v1.1.0] | **Realized for free by DG-9 — no separate mechanism.** L1 §4.7's `recurrence`/`consider_promotion` pseudocode is entirely host-side: a host's `DialogProvider` already owns the durable store behind DG-9 (it is the one that decided to return `Remembered`), so it already knows a dialog identity's occasion count without nodus reporting anything further. DG-9's per-event `DialogProvenance::Remembered` tag *is* the recurrence signal DG-10 builds on — the same "rides an existing signal, no enumerated list" shape this session's NL-9 `@err:` dispatch and LP-17 `SETTLEMENT_UNACCOUNTED` also use. |

## 4. Detailed Design

### 4.1 Vocabulary & Status additions

```text
[REFERENCE]
// vocab.rs — added to KNOWN_COMMANDS
"ASK", "CONFIRM"

// executor.rs — Status gains a variant (l2-nodus-runtime §4.7(h))
pub enum Status { Ok, Partial, Failed, Aborted, Paused }
```

`ASK`/`CONFIRM` modifiers (`+type`, `+options`, `+hint`, `+default`, `+validate`,
`+timeout`, `+strict`, `+actions`) reuse the existing `CommandCall.modifiers`
vector — no grammar change beyond recognising the two command names.

### 4.2 DialogProvider extension point

```text
[REFERENCE]
pub enum DialogOutcome {
    Answer(Value),   // resolved (by human or +default)
    Pause,           // no resolution available; caller should suspend
    Timeout,         // +timeout elapsed → DIALOG_TIMEOUT
    Rejected,        // +strict CONFIRM rejected → DIALOG_REJECTED
}

pub trait DialogProvider {
    fn ask(&self, prompt: &str, modifiers: &[(String, String)]) -> DialogOutcome;
    fn confirm(&self, content: &str, modifiers: &[(String, String)]) -> DialogOutcome;
}

/// Built-in synchronous resolver: returns the `+default` as an `Answer`,
/// otherwise `Pause`. Never performs I/O, never blocks (DG-6).
pub struct DefaultDialogProvider;
```

`DefaultDialogProvider` mirrors `StubProvider`/`NoopAuditProvider`: it satisfies
the interface with no I/O and is the provider the plain `run`/`run_with_*` paths
use.

### 4.3 Resume descriptor

```text
[REFERENCE]
pub struct ResumeDescriptor {
    pub workflow: String,                 // "wf:<name>"
    pub vars: HashMap<String, Value>,     // environment snapshot at suspension
    pub step_index: u32,                  // suspended step
}
```

`RunResult` carries `Option<ResumeDescriptor>`, populated only when
`status == Paused`. The runtime defines the descriptor; the host persists and
re-supplies it (LP-1). The descriptor never includes raw prompt text (DG-7).

### 4.4 Executor wiring

The executor gains a `DialogProvider` alongside its `ModelProvider`/
`AuditProvider`. `execute_command` dispatches `ASK`/`CONFIRM` to the provider:

- `Answer(v)` → bind `v` to the pipeline target (DG-3) and continue.
- `Pause` → stop the step loop, set `Status::Paused`, emit `NODUS:PAUSED` (info), and build the `ResumeDescriptor` (DG-2/DG-4).
- `Timeout` / `Rejected` → push the matching `RuntimeError`, route to `@err:` (DG-5).

`!HALT` sets `Status::Failed`; `!PAUSE` sets `Status::Paused` with a descriptor.
Dialog events (`ModelCall`-style, but dialog-tagged) carry `FieldDescriptor`
summaries only (DG-7).

### 4.5 Manifest integration

```text
[REFERENCE]
// portability.rs — ExtensionRole gains a variant
pub enum ExtensionRole { Model, Audit, Storage, Policy, Vocabulary, Dialog }
```

`CapabilityManifest::from_workflow` adds `Dialog` when it encounters an
`ASK`/`CONFIRM` whose step declares no `+default`; `HostCapabilities::builtin()`
does **not** provide `Dialog` (the default resolver only handles `+default`
dialogs), so a workflow needing real interaction is rejected fail-fast against a
non-interactive host (DG-8).

### 4.6 Public API

```text
[REFERENCE]
pub fn run_with_dialog(
    source, filename, input, dialog: &dyn DialogProvider,
) -> Result<RunResult, Vec<Diagnostic>>;

pub fn run_with_dialog_and_audit(
    source, filename, input, dialog, audit, run_id, started_at,
) -> Result<RunResult, Vec<Diagnostic>>;
```

Consistent with the orthogonal `run_with_*` combinator family (LP-5).

### 4.7 Memoizable & promotable approval (DG-9/DG-10) [ADDED v1.1.0]

```text
[REFERENCE]
pub enum DialogOutcome {
    Answer(Value),      // resolved fresh (by human or +default)
    Remembered(Value),  // resolved from a host durable prior decision (DG-9)
    Pause,
    Timeout,
    Rejected,
}

// observability.rs — EventAnnotations gains a fifth optional field,
// alongside message/anomaly/receipt/durability (HO-11/16/9/17); None for
// every non-dialog event and for a dialog step's non-resolution outcomes
// (Pause/Timeout/Rejected carry no provenance — they are not a decision).
pub enum DialogProvenance { Answered, Remembered }
pub struct EventAnnotations {
    // ...existing fields unchanged...
    pub dialog_provenance: Option<DialogProvenance>,
}
```

`handle_dialog` binds `Remembered(v)` through the exact same path as `Answer(v)`
(`ctx.set_var`, `ctx.log_step`) — memoization changes *where* the value came from,
never *how* it is bound, which is what keeps DG-9's "never turns a rejection into
an approval, never relaxes `+strict`, never binds a value failing `+type`/
`+validate`" discipline true by construction: a `Remembered` value flows through
the identical typed-binding path an `Answer` does, so it is subject to the same
`+type`/`+validate` handling, not a bypass. The only difference reaching the trace
is the `StepEnd` event's `annotations.dialog_provenance` — `Some(Answered)` on
`Answer`, `Some(Remembered)` on `Remembered`, `None` on `Pause`/`Timeout`/
`Rejected` (they resolve nothing to have a provenance).

The host's `ask`/`confirm` implementation is where all of DG-9's real logic lives:
recall the durable prior decision keyed by the resolved prompt/action signature,
check governance permits reusing it, confirm the remembered value still fits the
step's `+type`/`+validate`, and return `Remembered` — or fall through to an
ordinary prompt and return `Answer` (fail-safe to asking, per DG-9's own
discipline). None of that needs a new `DialogProvider` method: `ask`/`confirm`
already receive the full raw `modifiers` slice, so a host checks for `+remember`
itself exactly as it would check for any other modifier it cares about.

**LP-6 note**: adding a `DialogOutcome` variant is a breaking change for any host
matching the enum exhaustively without a wildcard arm — acceptable pre-1.0 (the
`NoopStorageProvider` → `InMemoryStorageProvider` rename precedent), and the only
realistic shape for a closed 5-variant outcome taxonomy that must distinguish
*why* a dialog resolved without inventing a second orthogonal signal.

## 5. Drawbacks & Alternatives

- **Mid-run coroutine suspension** (capture/restore the executor stack): rejected — it rewrites the run-to-completion model for every workflow to serve a minority of dialog steps. The `Status::Paused` + resume-descriptor signal achieves durable hand-off without that cost.
- **Folding dialog into `ModelProvider`**: rejected — conflates inference with human interaction; violates LP-5 (a host needing only one would be forced to supply both).
- **Blocking built-in resolver** (read stdin): rejected — it makes tests and headless runs hang; the default-or-pause resolver keeps non-interactive runs deterministic (DG-6).
- **New `DialogProvider::recall`/`may_memoize` trait methods** (DG-9): rejected — `ask`/`confirm` already receive the full `modifiers` slice, so a host can implement recall/governance entirely inside its own method body; a second trait method would duplicate a capability the existing signature already grants, the same reasoning `l2-nodus-settlement.md` §6 used to keep `SettlementRail::settle` on raw args rather than a typed wrapper.
- **A separate `DialogResolved` event instead of an `EventAnnotations` field** (DG-9): rejected — HO-6's closed event taxonomy is extended via optional fields on existing events (the HO-8/9/10/11/13/16/17 precedent), not new variants; dialog already reuses `StepStart`/`ModelCall`/`StepEnd` rather than minting dialog-specific events (DG-7), and provenance is no exception.
- **A nodus-side recurrence counter for DG-10**: rejected — the host's own durable store already knows a dialog identity's occasion count (it decided to return `Remembered`), so nodus tracking a parallel count would be redundant state that could drift from the host's own; DG-10 is fully discharged by DG-9's per-event provenance tag.

## 6. Implementation Notes

- Order: vocabulary + `Status::Paused` first (smallest, unblocks everything), then `DialogProvider` + `DefaultDialogProvider`, then executor dispatch, then manifest `Dialog` role, then the `run_with_dialog` combinators.
- `ExtensionRole::Dialog` is additive to the LP-8 enum; the `error_meta` registry and `validate_manifest` resolver already handle new roles generically.
- The dialog error codes are already canonical (`l2-nodus-errors.md`); this phase only adds their emission sites.
- **[ADDED v1.1.0]** DG-9/DG-10 add no new command, no new modifier registry entry (modifiers are already unconstrained free-form pass-through — confirmed no `KNOWN_MODIFIERS` registry exists, the same finding `l2-nodus-portability.md` §4.10 made for LP-16), and no new `DialogProvider` trait method — the smallest realization in this spec besides the original vocabulary addition: one `DialogOutcome` variant, one `EventAnnotations` field, one `handle_dialog` match arm sharing `Answer`'s binding logic.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[VOCAB]` | `crates/nodus/src/vocab.rs` | `ASK`/`CONFIRM` commands; error codes |
| `[EXEC]` | `crates/nodus/src/executor.rs` | `Status::Paused`, dialog dispatch, `ResumeDescriptor`, `handle_dialog`'s `DialogOutcome` match (§4.7) |
| `[PORT]` | `crates/nodus/src/portability.rs` | `ExtensionRole::Dialog`, manifest derivation |
| `[API]` | `crates/nodus/src/workflows.rs` | `run_with_dialog` combinators |
| `[OBS]` | `crates/nodus/src/observability.rs` | [ADDED v1.1.0] `EventAnnotations` — the `dialog_provenance` field §4.7 adds |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.1.0 | 2026-07-31 | Core Team | Closes the DG-9/DG-10 invariant-traceability gap flagged at the v1.29.0 nodus replan. New §4.7 designs both: DG-9's `+remember` marker needs no grammar change (modifiers are already unconstrained free-form pass-through to `DialogProvider::ask`/`confirm`); its one real nodus-side obligation — a distinct trace provenance on a memoized resolution — realizes as a new `DialogOutcome::Remembered(Value)` variant bound through `Answer`'s exact path, plus a new `dialog_provenance: Option<DialogProvenance>` field on the shared `EventAnnotations` carrier (the HO-8/9/10/11/13/16/17 closed-taxonomy-via-optional-field precedent, not a new event type). No new `DialogProvider` trait method: `ask`/`confirm` already receive the raw `modifiers` slice, so a host's own recall/governance/fit-check logic (L1 §4.6's pseudocode) lives entirely inside its existing method body. **DG-10 found to need no separate mechanism at all** — its recurrence/promotion pseudocode is 100% host-side, operating on the host's own durable store plus the DG-9 provenance tag nodus already emits, the same "rides an existing signal, no enumerated list" shape this session's NL-9 dispatch and LP-17 `SETTLEMENT_UNACCOUNTED` also use. §3 gains both compliance rows; §5 gains three rejected alternatives (a second trait method, a dedicated event type, a nodus-side recurrence counter) each traced to an existing precedent. Design only — nothing landed in `crates/nodus` this pass. |
| 1.0.0 | 2026-06-27 | Core Team | Initial spec — Rust realization of the dialog contract: `ASK`/`CONFIRM` vocabulary, `Status::Paused` + `ResumeDescriptor`, `DialogProvider` + `DefaultDialogProvider` (synchronous default-or-pause), `ExtensionRole::Dialog` manifest binding, executor dispatch, `run_with_dialog` combinators; DG-1…DG-8 compliance table. |

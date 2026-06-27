# Nodus Human-in-the-Loop Dialog Implementation (Rust)

**Version:** 1.0.0
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

- [l1-nodus-dialog.md](l1-nodus-dialog.md) — the dialog contract this implements (DG-1…DG-8)
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — runtime crate extended here: `vocab` (commands, `Status`), `executor`, the `run_with_*` family
- [l2-nodus-portability.md](l2-nodus-portability.md) — LP-2 extension-point pattern + the LP-8 `ExtensionRole` taxonomy this adds `Dialog` to
- [l2-nodus-errors.md](l2-nodus-errors.md) — owns the `DIALOG_TIMEOUT`/`DIALOG_REJECTED`/`PAUSED` codes and their severity/category

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

## 5. Drawbacks & Alternatives

- **Mid-run coroutine suspension** (capture/restore the executor stack): rejected — it rewrites the run-to-completion model for every workflow to serve a minority of dialog steps. The `Status::Paused` + resume-descriptor signal achieves durable hand-off without that cost.
- **Folding dialog into `ModelProvider`**: rejected — conflates inference with human interaction; violates LP-5 (a host needing only one would be forced to supply both).
- **Blocking built-in resolver** (read stdin): rejected — it makes tests and headless runs hang; the default-or-pause resolver keeps non-interactive runs deterministic (DG-6).

## 6. Implementation Notes

- Order: vocabulary + `Status::Paused` first (smallest, unblocks everything), then `DialogProvider` + `DefaultDialogProvider`, then executor dispatch, then manifest `Dialog` role, then the `run_with_dialog` combinators.
- `ExtensionRole::Dialog` is additive to the LP-8 enum; the `error_meta` registry and `validate_manifest` resolver already handle new roles generically.
- The dialog error codes are already canonical (`l2-nodus-errors.md`); this phase only adds their emission sites.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[VOCAB]` | `crates/nodus/src/vocab.rs` | `ASK`/`CONFIRM` commands; error codes |
| `[EXEC]` | `crates/nodus/src/executor.rs` | `Status::Paused`, dialog dispatch, `ResumeDescriptor` |
| `[PORT]` | `crates/nodus/src/portability.rs` | `ExtensionRole::Dialog`, manifest derivation |
| `[API]` | `crates/nodus/src/workflows.rs` | `run_with_dialog` combinators |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-27 | Core Team | Initial spec — Rust realization of the dialog contract: `ASK`/`CONFIRM` vocabulary, `Status::Paused` + `ResumeDescriptor`, `DialogProvider` + `DefaultDialogProvider` (synchronous default-or-pause), `ExtensionRole::Dialog` manifest binding, executor dispatch, `run_with_dialog` combinators; DG-1…DG-8 compliance table. |

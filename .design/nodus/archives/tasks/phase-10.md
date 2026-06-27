---
phase: 10
name: "HITL Dialog (l2-nodus-dialog)"
status: Done
subsystem: "crates/nodus"
requires:
  - phase-8
  - phase-9
provides:
  - ASK/CONFIRM in KNOWN_COMMANDS; Status::Paused + Signal::Pause
  - DialogOutcome + DialogProvider trait + synchronous DefaultDialogProvider
  - executor handle_dialog dispatch; ResumeDescriptor on RunResult; DIALOG_TIMEOUT/DIALOG_REJECTED/PAUSED emission
  - ExtensionRole::Dialog + CapabilityManifest::from_workflow derivation (Dialog unless +default)
  - run_with_dialog / run_with_dialog_and_audit combinators + lib re-exports
  - tests/dialog.rs (7 DG-invariant integration tests)
key_files:
  - crates/nodus/src/vocab.rs
  - crates/nodus/src/executor.rs
  - crates/nodus/src/portability.rs
  - crates/nodus/src/workflows.rs
  - crates/nodus/src/lib.rs
  - crates/nodus/tests/dialog.rs
patterns_established:
  - "Pause-as-signal: the executor stays run-to-completion; a dialog that cannot resolve sets a Pause signal + ResumeDescriptor rather than capturing/restoring mid-run state. True cross-process suspend/resume is a host concern over Status::Paused"
  - "Synchronous built-in dialog resolver (default-or-pause) keeps non-interactive and test runs deterministic, mirroring the StubProvider/Noop pattern"
duration_minutes: 0
---

# Phase 10 — Human-in-the-Loop Dialog (l2-nodus-dialog)

Implement the dialog contract (`l1-nodus-dialog.md`) in `crates/nodus`, per
`l2-nodus-dialog.md`. Adds the `ASK`/`CONFIRM` commands, the `Status::Paused` run
state and `ResumeDescriptor`, the `DialogProvider` extension point with a
built-in synchronous `DefaultDialogProvider`, the `ExtensionRole::Dialog`
manifest binding, executor dispatch, and the `run_with_dialog` combinators.

**Specs covered**: `l2-nodus-dialog.md` (Stable); contract `l1-nodus-dialog.md`.

**Design note**: the built-in resolver is synchronous (resolve via `+default`,
else signal `Status::Paused`) so non-interactive runs never block; true
cross-invocation suspend/resume is a host concern over the `Paused` signal. The
dialog error codes (`DIALOG_TIMEOUT`/`DIALOG_REJECTED`/`PAUSED`) already exist
(Phase 8) — this phase adds their emission.

## Track A — Vocabulary & Status

- [x] **T-10A01** — Add `ASK` and `CONFIRM` to `vocab::KNOWN_COMMANDS`; bump `BUILTIN_SCHEMA_VERSION` if the schema-version policy requires it
  - **Verify**: unit test `ask_confirm_are_known_commands` — `Schema::builtin().is_command("ASK")` and `"CONFIRM"` both true
- [x] **T-10A02** — Add `Paused` to the `executor::Status` enum; map it in the `RunStatus` conversion used by the audit manifest
  - **Verify**: `cargo check -p nodus` passes; unit test constructs `Status::Paused` and the match arms compile exhaustively

## Track B — DialogProvider extension point

- [x] **T-10B01** — Define `DialogOutcome` (`Answer(Value)`/`Pause`/`Timeout`/`Rejected`), the `DialogProvider` trait (`ask`/`confirm`), and `DefaultDialogProvider` (returns `Answer` from a `+default`, else `Pause`; never I/O)
  - **Verify**: unit tests in the dialog module — `DefaultDialogProvider.ask` with a `+default` returns `Answer`; with none returns `Pause`

## Track C — Executor dispatch

- [x] **T-10C01** — Wire a `DialogProvider` into `Executor` (alongside model/audit); dispatch `ASK`/`CONFIRM`: `Answer` → bind to pipeline target; `Pause` → stop the loop, set `Status::Paused`, emit `NODUS:PAUSED`, build `ResumeDescriptor`; `Timeout`/`Rejected` → push the matching `RuntimeError`
  - Add `ResumeDescriptor` (workflow + var snapshot + step index) as `Option` on `RunResult`, populated only when `status == Paused`
  - **Verify**: integration test — a workflow with `ASK(...) +default` runs to `Status::Ok` with the default bound; the same `ASK` with no default and the default provider yields `Status::Paused` and a `ResumeDescriptor`, with no later step executed
- [x] **T-10C02** — Emit `DIALOG_TIMEOUT` / `DIALOG_REJECTED` on those outcomes, routed like other runtime errors; dialog audit events carry `FieldDescriptor` summaries only (DG-7)
  - **Verify**: unit/integration tests — a `Timeout` outcome surfaces `NODUS:DIALOG_TIMEOUT`; a strict-rejected `CONFIRM` surfaces `NODUS:DIALOG_REJECTED`; no raw answer text appears in emitted events

## Track D — Manifest integration

- [x] **T-10D01** — Add `ExtensionRole::Dialog`; extend `CapabilityManifest::from_workflow` to require `Dialog` for an `ASK`/`CONFIRM` whose step declares no `+default`; `HostCapabilities::builtin()` does not provide `Dialog`
  - **Verify**: unit tests — a workflow with a defaulted dialog derives no `Dialog` role; a workflow with a non-defaulted dialog derives `Dialog`, and `validate_manifest` against `builtin()` reports `Missing::Role(Dialog)`

## Track E — Public API

- [x] **T-10E01** — Add `run_with_dialog` and `run_with_dialog_and_audit` to `workflows.rs` (consistent with the `run_with_*` family); re-export from `lib.rs`
  - **Verify**: integration test — `run_with_dialog` with a custom provider that answers a prompt binds the answer and reaches `Status::Ok`

## Track T — Validation & gates

- [x] **T-10T01** — DG-invariant integration tests in `tests/` (blocking progression, typed binding, default-on-absence pause, typed failures, trace data-safety) + quality gates
  - **Verify**: `cargo test -p nodus` full suite green; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt` clean; `cargo doc --no-deps` no new warnings; SDD §6 clean

## Status

**Status:** Done

## Notes

Execution order **A → B → C → {D, E} → T**: vocabulary + `Status::Paused`
(Track A) unblock everything; the provider (B) precedes executor dispatch (C);
manifest (D) and public API (E) are independent once C lands. This is the
largest phase in the plan — `Status::Paused` touches the executor's status
derivation and the audit conversion, so Track A/C carry the most regression risk;
the full-suite gate (Track T) is the guard. Real cross-process suspend/resume is
explicitly out of scope (host concern); this phase delivers the synchronous
default-or-pause baseline.

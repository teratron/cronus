---
phase: 4
name: "Observability & Extension Framework"
status: Done
subsystem: "crates/nodus"
requires:
  - phase-3
provides:
  - AuditProvider trait (observability.rs)
  - ExecutionEvent enum (10 variants)
  - NoopAuditProvider
  - RunManifest + RunStatus + FieldDescriptor
  - executor.rs hook points for all 10 event types
  - run_with_audit / run_with_provider_and_audit public API
  - l2-nodus-runtime.md v1.0.3 (spec sync)
key_files: []
patterns_established: []
duration_minutes: 0
---

# Phase 4 — Observability & Extension Framework

**Status:** Done
**Execution Mode:** Sequential (strict dep chain A → B01 → B02 → B03 → B04 → C → T)
**Specs:** [l2-nodus-observability.md](../../specifications/l2-nodus-observability.md) · [l2-nodus-runtime.md](../../specifications/l2-nodus-runtime.md)

## Overview

Wire the full 10-event AuditProvider taxonomy into `crates/nodus`. The implementation adds a
new `observability.rs` module, extends `executor.rs` with hook points, and adds two new public
API functions to `workflows.rs`. Culminates with a `l2-nodus-runtime.md` spec sync (v1.0.3).

## Track A — `observability.rs` (New module)

### T-4A01: Create `observability.rs` — types, traits, built-in

- [x] Create `crates/nodus/src/observability.rs`
- [x] Define `AuditProvider` trait: `record_event(&self, event: ExecutionEvent)` + `run_complete(&self, manifest: RunManifest)`
- [x] Define `ExecutionEvent` closed enum with all 10 variants (StepStart / StepEnd / StepError / ConstraintHit / BranchTaken / LoopIteration / MacroEnter / MacroExit / ModelCall / ModelResponse); match fields in §4.1 of l2-nodus-observability.md
- [x] Define `LoopType` enum (For / Until)
- [x] Define `FieldDescriptor` struct (field_count, type_hints, total_bytes) — no raw text
- [x] Define `RunManifest` struct (workflow_name, schema_version, run_id, started_at, elapsed_ms, status, error_code, total_steps, event_count)
- [x] Define `RunStatus` enum (Ok / Error / ConstraintHalt / ValidationError)
- [x] Implement `NoopAuditProvider`: both methods are empty (`()`)

**Verify:** `cargo check -p nodus` passes; `observability.rs` module compiles with zero warnings.

### T-4A02: Unit tests for observability types

- [x] Add `#[cfg(test)] mod tests` block in `observability.rs`
- [x] Define `RecordingProvider` test helper struct: holds `Arc<Mutex<Vec<ExecutionEvent>>>` + `Arc<Mutex<Vec<RunManifest>>>`; implements `AuditProvider` by pushing to the vecs; used across all audit unit tests
- [x] `noop_provider_discards_all` — call `record_event` for all 10 event variants + `run_complete`; no panic
- [x] `step_start_end_emitted` — `RecordingProvider` collects events; verify StepStart precedes StepEnd for a single GEN step
- [x] `constraint_hit_halt_true` — verify `ConstraintHit { halt: true }` variant round-trips correctly
- [x] `branch_taken_if` / `branch_taken_else` — BranchTaken variant with both `condition_result` values
- [x] `loop_iteration_for` / `loop_iteration_until` — LoopIteration variants
- [x] `macro_enter_exit_pair` — MacroEnter + MacroExit pair
- [x] `model_call_no_raw_content` — ModelCall.input_summary fields contain only FieldDescriptor data, not text
- [x] `run_manifest_fields_populated` — RunManifest fields accessible and typed correctly

**Verify:** `cargo test -p nodus observability` — all unit tests pass; `cargo clippy -p nodus -- -D warnings` clean.

## Track B — `executor.rs` delta

### T-4B01: Add `audit` field + lib.rs module declaration

- [x] Add `pub mod observability;` to `crates/nodus/src/lib.rs`
- [x] Add re-exports to `lib.rs`: `pub use observability::{AuditProvider, ExecutionEvent, NoopAuditProvider, RunManifest, RunStatus, LoopType, FieldDescriptor};`
- [x] Add `use crate::observability::{AuditProvider, ExecutionEvent, ...};` imports to `executor.rs`
- [x] Add `audit: Box<dyn AuditProvider>` field to `Executor` struct
- [x] Update `Executor::new()` to initialise `audit: Box::new(NoopAuditProvider)`
- [x] Add `Executor::with_audit(provider, audit)` constructor
- [x] Update `ExecutionContext`: add `event_count: u32` field; add `start_instant: std::time::Instant` field
- [x] Update `ExecutionContext::new()` to initialise both new fields

**Verify:** `cargo test -p nodus` — all 143 existing tests still pass (no regressions); `cargo check -p nodus` clean.

### T-4B02: Wire `StepStart` / `StepEnd` / `StepError` / `ConstraintHit`

> **Architecture note**: `dispatch` currently has signature `fn dispatch(&self, ctx: &mut ExecutionContext, cmd: &CommandCall) -> Value`. It must be updated to `fn dispatch(&self, ctx: &mut ExecutionContext, cmd: &CommandCall, step_num: u32) -> Value` so `step_num` is available for `ModelCall`/`MacroEnter`/`MacroExit` events emitted inside it.

Hook points in `execute_command`:

- [x] Update `dispatch` signature: add `step_num: u32` parameter; update the single call site in `execute_command`
- [x] Before `self.dispatch(ctx, cmd, step_num)`: emit `StepStart`; snapshot `ctx.variables.keys()` for `input_vars`; record `Instant::now()` for elapsed
- [x] After `dispatch` returns: emit `StepEnd { output_vars: cmd.pipeline_target.iter().cloned().collect(), elapsed_ms }`; increment `ctx.event_count` twice (StepStart + StepEnd)
- [x] In the `check_rules` violation path, before `ctx.errors.push`: emit `ConstraintHit { halt: true }`; then emit `StepError`; increment `ctx.event_count` twice; then `ctx.errors.push`; then `return Some(Signal::Break)`

**Verify:** `cargo test -p nodus` — all 143 existing tests still pass (no regressions); `cargo check -p nodus` clean; add in-module test that a rule-violation run emits `ConstraintHit` + `StepError` events.

### T-4B03: Wire `BranchTaken` / `LoopIteration` / `MacroEnter` + `MacroExit`

- [x] In `execute_conditional`, at the start of each taken branch (if/elif/else): `self.audit.record_event(ExecutionEvent::BranchTaken { step_index: step_num, branch_label: "if"|"elif"|"else".to_string(), condition_result: true|false })`
- [x] In `execute_for`, at the top of each loop body iteration: `self.audit.record_event(ExecutionEvent::LoopIteration { step_index: step_num, loop_type: LoopType::For, iteration_number: iter_num, bound_vars: vec![fl.variable.clone()] })`
- [x] In `execute_until`, at the top of each loop body: `self.audit.record_event(ExecutionEvent::LoopIteration { step_index: step_num, loop_type: LoopType::Until, iteration_number: iter_num, bound_vars: vec![] })`
- [x] In `dispatch` "RUN" arm, before flag push: `self.audit.record_event(ExecutionEvent::MacroEnter { macro_name: macro_name.clone(), call_step_index: step_num })` + record `Instant`
- [x] In `dispatch` "RUN" arm, after flag push: `self.audit.record_event(ExecutionEvent::MacroExit { macro_name, call_step_index: step_num, elapsed_ms })`

**Verify:** `cargo test -p nodus` — all tests pass; existing `break_from_conditional_halts_steps` test still passes; add test that `~FOR` loop emits correct `LoopIteration` count.

### T-4B04: Wire `ModelCall` / `ModelResponse` + `run_complete` in `execute()`

> **Architecture note**: `handle_gen` and `handle_analyze` currently take `ctx: &ExecutionContext` (immutable). They must be updated to `ctx: &mut ExecutionContext` so they can increment `ctx.event_count` after emitting model events. Both also need a new `step_num: u32` parameter passed down from `dispatch`.

- [x] Update `handle_gen` signature: `fn handle_gen(&self, ctx: &mut ExecutionContext, cmd: &CommandCall, step_num: u32) -> Value`
- [x] Update `handle_analyze` signature: same pattern; update call sites in `dispatch`
- [x] In `handle_gen`, before `self.provider.generate(...)`: emit `ModelCall { input_summary: FieldDescriptor::text(prompt_bytes) }`; record `Instant`; `ctx.event_count += 1`
- [x] In `handle_gen`, after `generate` returns: emit `ModelResponse { output_summary: FieldDescriptor::text(result_bytes), elapsed_ms }`; `ctx.event_count += 1`
- [x] In `handle_analyze`: same pattern — `ModelCall` before `provider.analyze`, `ModelResponse` after
- [x] Add `pub fn execute_with_params(&self, ast, input, run_id: &str, started_at: &str) -> RunResult` that delegates to a shared `execute_inner` private function
- [x] `execute()` delegates to `execute_inner(ast, input, "", "")` for backward compat
- [x] In `execute_inner`, immediately before building `RunResult`: `self.audit.run_complete(RunManifest { ... elapsed_ms: ctx.start_instant.elapsed().as_millis() as u64, event_count: ctx.event_count, total_steps: ctx.log.len() as u32, ... })`

**Verify:** `cargo test -p nodus` — all existing 143 tests pass; `model_call_no_raw_content` unit test confirms no raw text in `FieldDescriptor` fields.

## Track C — `workflows.rs` delta

### T-4C01: Add `run_with_audit` + `run_with_provider_and_audit` public functions

- [x] Add `run_with_audit(source, filename, input, audit, run_id, started_at)` — validate → `Executor::with_audit(StubProvider, audit)` → `execute_with_run_params`
- [x] Add `run_with_provider_and_audit(source, filename, input, provider, audit, run_id, started_at)` — validate → `Executor::with_audit(provider, audit)` → `execute_with_run_params`
- [x] Both functions fast-fail on validation errors (NL-4) before constructing the `Executor`
- [x] Re-export both functions from `lib.rs` public surface

**Verify:** `cargo doc -p nodus --no-deps` — zero warnings; new functions appear in generated docs with correct signatures.

## Track T — Integration Tests + Spec Sync

### T-4T01: Observer-neutrality integration test (HO-5 gate)

- [x] Create `crates/nodus/tests/observability.rs`
- [x] `observer_neutrality` test: run the same deterministic workflow with `NoopAuditProvider` and with `RecordingProvider`; assert `RunResult.out`, `.status`, and `.errors` are identical in both runs

**Verify:** `cargo test -p nodus --test observability observer_neutrality` passes.

### T-4T02: Public API integration tests

- [x] `run_with_audit_api`: call `run_with_audit` with a `RecordingProvider`; verify `RunResult` is Ok and events were collected
- [x] `run_with_provider_and_audit_api`: call `run_with_provider_and_audit` with `StubProvider` + `RecordingProvider`; verify model events in collected list

**Verify:** `cargo test -p nodus --test observability` — all tests in the file pass.

### T-4T03: Full cargo suite + spec sync

- [x] `cargo test -p nodus` — all tests pass (≥144 including new ones)
- [x] `cargo clippy -p nodus --all-targets -- -D warnings` — zero lints
- [x] `cargo fmt -p nodus` — no formatting changes
- [x] `cargo doc -p nodus --no-deps` — zero warnings
- [x] Update [l2-nodus-runtime.md](../../specifications/l2-nodus-runtime.md):
  - §4.1 Module structure: add `observability.rs` entry
  - §4.5 Public API table: add `run_with_audit` and `run_with_provider_and_audit` rows
  - Version: 1.0.2 → 1.0.3
  - Document History: add row for v1.0.3
- [x] Update nodus INDEX.md: l2-nodus-runtime.md version 1.0.2 → 1.0.3

**Verify:** `cargo test -p nodus` passes with the updated runtime; `cargo check -p nodus` clean; INDEX.md version matches file header.

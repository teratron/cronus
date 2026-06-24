# Nodus Observability Implementation (Rust)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-observability.md

## Overview

Concrete Rust implementation of the nodus execution observability contract.
Adds an `AuditProvider` trait, a closed `ExecutionEvent` enum (10 types), a `RunManifest`
struct, and the built-in `NoopAuditProvider` to `crates/nodus`. Hook points are wired into
`executor.rs` so that every observable execution event is emitted in emission order, synchronously,
before control returns to the next step. The existing `run` and `run_with_provider` API functions
retain their signatures; new `run_with_audit` and `run_with_provider_and_audit` variants accept
an `AuditProvider` implementation from the host.

## Related Specifications

- [l1-nodus-observability.md](l1-nodus-observability.md) — observability contract this spec implements
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — executor and public API this spec extends
- [l1-nodus-portability.md](l1-nodus-portability.md) — `AuditProvider` is the registered Audit extension point

## 1. Motivation

The existing `LogEntry`-based execution log in `executor.rs` records step command and result but
cannot represent control-flow transitions (branches, loops, macro calls) or model invocations as
distinct event types. Harness evolution analysis requires structured, attributable, append-only
events — not just a flat command log. This spec closes the gap by wiring the full 10-type event
taxonomy from `l1-nodus-observability.md` into the executor.

## 2. Constraints & Assumptions

- The `AuditProvider` is on the hot execution path; the built-in `NoopAuditProvider` must cost at
  most one virtual dispatch per event (no allocation, no I/O).
- `run_id` is caller-supplied (UUID or equivalent); the executor accepts it as a parameter and
  never generates one itself, keeping the executor deterministic and testable.
- Elapsed time for `step_end`, `macro_exit`, and `model_response` events is measured using
  `std::time::Instant` (monotonic); wall-clock `started_at` in `RunManifest` is a caller-supplied
  ISO-8601 string.
- The `model_call` and `model_response` events must not include raw user content — only structural
  descriptors (HO-4 data-safety boundary; §4.4 of `l1-nodus-observability.md`).
- Adding an `AuditProvider` to a run must not change `RunResult.out`, `RunResult.status`, or
  `RunResult.errors` — the provider is a write-only side channel (HO-5).
- `LogEntry` and `RunResult` in `executor.rs` are unchanged; they remain the caller-facing output.
  The audit stream is orthogonal to the existing log.

## 3. Invariant Compliance

| L1 Invariant | Implementation |
| --- | --- |
| HO-1 Trace-first output | `run_with_audit` emits all events during execution; `run_complete` is called unconditionally before returning `RunResult`. A run without an attached audit uses `NoopAuditProvider` but the hook calls still occur — every run can be observed |
| HO-2 Per-step attribution | Every `ExecutionEvent` variant carries `step_index: u32` (or `call_step_index` for macros); `constraint_hit` carries `triggering_step_index`. No event is emitted with a zero/unknown step index except boot-sequence errors |
| HO-3 Append-only immutability | `AuditProvider::record_event` is a one-way write call; the trait has no mutation or replay method. The `NoopAuditProvider` discards; concrete providers append. There is no API surface to modify a previously emitted event |
| HO-4 Frozen boundary | `observability.rs` contains no validator or executor logic. `record_event` is called *after* the NL-invariant checks in `validator.rs` complete — the audit layer only witnesses outcomes; it never intercepts or modifies them. `ExecutionEvent::ConstraintHit` records that a rule fired; it does not re-evaluate the rule |
| HO-5 Observer neutrality | `Executor::execute` calls `record_event` on a `&dyn AuditProvider` reference; the return type is `()`. No branch in `executor.rs` inspects provider state or return value. `RunResult` is assembled independently of the provider |
| HO-6 Structured event taxonomy | `ExecutionEvent` is a closed Rust enum with exactly 10 variants matching the L1 taxonomy. Adding a new event type requires amending this spec (minor version bump) and the `ExecutionEvent` enum — there is no catch-all string variant |

## 4. Detailed Design

### 4.1 New Module: `observability.rs`

All new types live in `crates/nodus/src/observability.rs`. This module has no dependencies
outside the Rust standard library.

```text
[REFERENCE]
// Public types
pub trait AuditProvider {
    fn record_event(&self, event: ExecutionEvent);
    fn run_complete(&self, manifest: RunManifest);
}

pub struct NoopAuditProvider;
impl AuditProvider for NoopAuditProvider {
    fn record_event(&self, _: ExecutionEvent) {}
    fn run_complete(&self, _: RunManifest) {}
}

/// Closed enum — 10 variants matching the L1 taxonomy.
pub enum ExecutionEvent {
    StepStart   { step_index: u32, step_command: String, input_vars: Vec<String> },
    StepEnd     { step_index: u32, step_command: String, output_vars: Vec<String>, elapsed_ms: u64 },
    StepError   { step_index: u32, step_command: String, error_code: String, error_detail: String },
    ConstraintHit  { rule_name: String, triggering_step_index: u32, halt: bool },
    BranchTaken    { step_index: u32, branch_label: String, condition_result: bool },
    LoopIteration  { step_index: u32, loop_type: LoopType, iteration_number: u32, bound_vars: Vec<String> },
    MacroEnter  { macro_name: String, call_step_index: u32 },
    MacroExit   { macro_name: String, call_step_index: u32, elapsed_ms: u64 },
    ModelCall   { step_index: u32, command: String, input_summary: FieldDescriptor },
    ModelResponse { step_index: u32, command: String, output_summary: FieldDescriptor, elapsed_ms: u64 },
}

pub enum LoopType { For, Until }

/// Structural descriptor — no raw user content (HO-5 data-safety).
pub struct FieldDescriptor {
    pub field_count: u32,
    pub type_hints: Vec<String>,   // e.g. ["Text", "Map"]
    pub total_bytes: u32,          // approximate; no content
}

pub struct RunManifest {
    pub workflow_name: String,
    pub schema_version: String,
    pub run_id: String,
    pub started_at: String,        // ISO-8601, caller-supplied
    pub elapsed_ms: u64,
    pub status: RunStatus,
    pub error_code: Option<String>,
    pub total_steps: u32,
    pub event_count: u32,
}

pub enum RunStatus { Ok, Error, ConstraintHalt, ValidationError }
```

### 4.2 `lib.rs` Changes

Add `pub mod observability;` to `lib.rs`. Re-export the public surface:

```text
[REFERENCE]
pub use observability::{AuditProvider, ExecutionEvent, NoopAuditProvider, RunManifest, RunStatus,
                        LoopType, FieldDescriptor};
```

### 4.3 `executor.rs` Changes

`Executor` gains a second field alongside `provider`:

```text
[REFERENCE]
pub struct Executor {
    provider: Box<dyn ModelProvider>,
    audit:    Box<dyn AuditProvider>,
}

impl Executor {
    pub fn new(provider: impl ModelProvider + 'static) -> Self {
        Executor { provider: Box::new(provider), audit: Box::new(NoopAuditProvider) }
    }

    pub fn with_audit(
        provider: impl ModelProvider + 'static,
        audit:    impl AuditProvider + 'static,
    ) -> Self {
        Executor { provider: Box::new(provider), audit: Box::new(audit) }
    }
}
```

`ExecutionContext` gains `event_count: u32` and `started_at: std::time::Instant`.

The following call sites in `execute_command` / `execute_conditional` / `execute_for` /
`execute_until` / `execute_parallel` / `dispatch` emit the corresponding event variant.
All `record_event` calls use `self.audit.record_event(event)` (synchronous, in-path):

| Call site | Event emitted |
| --- | --- |
| Before `dispatch(ctx, cmd)` | `StepStart` — input_vars = snapshot of `ctx.variables` keys written so far |
| After successful `dispatch` | `StepEnd` — output_vars = pipeline target name(s); elapsed from `Instant::now()` before dispatch |
| In `check_rules` violation path | `ConstraintHit` — halt=true (rule violations always halt) |
| After `ctx.errors.push` in rule-violation path | `StepError` — error_code = `RULE_VIOLATION` |
| In `execute_conditional`, before executing the taken branch | `BranchTaken` — branch_label = "if"/"elif"/"else" |
| At top of each `execute_for` loop body | `LoopIteration` — loop_type=For; bound_vars = [fl.variable] |
| At top of each `execute_until` loop body | `LoopIteration` — loop_type=Until; bound_vars = [] |
| In `dispatch` "RUN" arm, before flag push | `MacroEnter` — macro_name from cmd.args[0] |
| In `dispatch` "RUN" arm, after flag push | `MacroExit` — elapsed from Instant before MacroEnter |
| In `handle_gen`, before `self.provider.generate` | `ModelCall` — input_summary = FieldDescriptor { field_count: 1, type_hints: ["Text"], total_bytes: prompt.len() as u32 clamped to 0..u32::MAX } |
| In `handle_gen`, after `generate` returns | `ModelResponse` — output_summary = FieldDescriptor from result length |
| In `handle_analyze`, before `self.provider.analyze` | `ModelCall` — input_summary = FieldDescriptor from flags.len() |
| In `handle_analyze`, after `analyze` returns | `ModelResponse` |

`execute()` calls `self.audit.run_complete(manifest)` immediately before returning `RunResult`.
The manifest's `elapsed_ms` is measured from an `Instant` taken at the top of `execute()`.

### 4.4 `workflows.rs` API Additions

Two new public functions alongside the existing six:

```text
[REFERENCE]
/// Like `run` but with an injected `AuditProvider`.
/// `run_id` is caller-supplied (UUID or equivalent string).
pub fn run_with_audit(
    source:   &str,
    filename: &str,
    input:    Option<Value>,
    audit:    impl AuditProvider + 'static,
    run_id:   &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>>;

/// Like `run_with_provider` but also accepts an `AuditProvider`.
pub fn run_with_provider_and_audit(
    source:   &str,
    filename: &str,
    input:    Option<Value>,
    provider: impl ModelProvider + 'static,
    audit:    impl AuditProvider + 'static,
    run_id:   &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>>;
```

Both functions:
1. Validate with `Validator::validate` (fast-fail on errors — NL-4).
2. Construct `Executor::with_audit(provider, audit)`.
3. Call `executor.execute_with_run_params(ast, input, run_id, started_at)` (extended overload of
   `execute` that threads `run_id` / `started_at` for manifest construction — internal method, not
   public).
4. Return the `RunResult`.

### 4.5 Module Structure After Changes

```text
[REFERENCE]
crates/nodus/src/
├── lib.rs           — adds `pub mod observability;` + re-exports
├── observability.rs — NEW: AuditProvider, ExecutionEvent (10 variants), NoopAuditProvider,
│                          RunManifest, RunStatus, LoopType, FieldDescriptor
├── executor.rs      — MODIFIED: Executor gains `audit` field; hook points wired;
│                          execute() / execute_command / execute_conditional /
│                          execute_for / execute_until / dispatch emit events
├── workflows.rs     — MODIFIED: run_with_audit + run_with_provider_and_audit added
└── (all other modules unchanged)
```

### 4.6 Test Coverage

Unit tests reside in `observability.rs` (`#[cfg(test)] mod tests`) and integration tests in
`crates/nodus/tests/`:

| Test | Location | What it verifies |
| --- | --- | --- |
| `noop_provider_discards_all` | observability.rs | `NoopAuditProvider` accepts all 10 event variants without panic |
| `step_start_end_emitted` | observability.rs | Recording provider receives `StepStart` then `StepEnd` for a GEN step |
| `constraint_hit_recorded` | observability.rs | `ConstraintHit { halt: true }` emitted when `!!NEVER` fires |
| `branch_taken_if` | observability.rs | `BranchTaken { branch_label: "if" }` emitted when condition is true |
| `branch_taken_else` | observability.rs | `BranchTaken { branch_label: "else" }` emitted when condition is false |
| `loop_iteration_for` | observability.rs | `LoopIteration { loop_type: For }` emitted N times for N-element collection |
| `loop_iteration_until` | observability.rs | `LoopIteration { loop_type: Until }` emitted each body entry |
| `macro_enter_exit` | observability.rs | `MacroEnter` + `MacroExit` pair emitted for RUN command |
| `model_call_response_no_raw_content` | observability.rs | `ModelCall.input_summary` has no user text; only `FieldDescriptor` fields |
| `run_complete_manifest` | observability.rs | `run_complete` called once; manifest fields populated |
| `observer_neutrality` | tests/observability.rs | RunResult with NoopAuditProvider == RunResult with RecordingProvider for deterministic inputs |
| `run_with_audit_api` | tests/observability.rs | Public `run_with_audit` function returns correct RunResult; events collected |
| `run_with_provider_and_audit_api` | tests/observability.rs | Public `run_with_provider_and_audit` with StubProvider + RecordingProvider |

The `RecordingProvider` test helper collects events in a `Vec<ExecutionEvent>` behind a
`std::sync::Mutex` — zero external dependencies.

## 5. Implementation Notes

1. Implement `observability.rs` first (pure types, no executor dependency). All 10 variants and
   `NoopAuditProvider` can be fully tested without touching `executor.rs`.
2. Add the `audit` field to `Executor` and thread it through `with_audit` constructor. Compile-check
   that existing `new()` / `with_stub()` still work (they use `NoopAuditProvider`).
3. Wire hook points in `execute_command` first (covers `StepStart`/`StepEnd`/`StepError`/
   `ConstraintHit`). Run existing tests — they must still pass.
4. Add `BranchTaken` in `execute_conditional`, `LoopIteration` in `execute_for`/`execute_until`,
   `MacroEnter`/`MacroExit` in the `RUN` arm of `dispatch`.
5. Add `ModelCall`/`ModelResponse` in `handle_gen` and `handle_analyze`.
6. Add `execute()` run-complete call and `run_with_audit` API functions last.
7. Write integration test `observer_neutrality` as the final gate — confirms HO-5.

## 6. Drawbacks & Alternatives

- **Shared `Executor` instance across concurrent runs**: `Box<dyn AuditProvider>` is `Send` only
  if the concrete type is. Concurrent use is currently out of scope (the executor is sequential per
  run). Host projects that need concurrent runs should instantiate one `Executor` per run thread.
- **Alternative — emit events to a channel instead of a trait**: using `std::sync::mpsc::Sender`
  avoids the vtable dispatch but couples the executor to the channel type and complicates no-op
  semantics. Rejected in favour of LP-2 (extension via abstract interface).
- **Alternative — integrate `tracing` crate**: provides rich structured logging but adds a
  non-`std` dependency, violating the zero-external-deps constraint from `l2-nodus-runtime.md §2`.
  Hosts may wrap an `AuditProvider` implementation that bridges to `tracing` at the host layer.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[OBSERVABILITY]` | `crates/nodus/src/observability.rs` | Authoritative source for all event types, traits, and manifest struct |
| `[EXECUTOR]` | `crates/nodus/src/executor.rs` | Hook-point implementation — where `record_event` calls are inserted |
| `[WORKFLOWS]` | `crates/nodus/src/workflows.rs` | Public API functions including new `run_with_audit` variants |
| `[TESTS]` | `crates/nodus/tests/observability.rs` | Integration tests for observer neutrality and public API |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — HO-1…HO-6 compliance table, `observability.rs` type system, executor hook-point mapping, `workflows.rs` API additions, full test plan |

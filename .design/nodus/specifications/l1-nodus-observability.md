# Nodus Execution Observability

**Version:** 1.2.0
**Status:** Stable
**Layer:** concept

## Overview

Execution observability is the foundation that enables harness evolution: a workflow run must produce a structured, queryable trace of every step — not just a pass/fail score. This spec defines the observability contract that any conforming nodus implementation must honour. Structured traces allow an outer evolution loop to identify failure patterns, attribute causes to specific harness components, and propose evidence-backed amendments.

The observability layer is designed to be *observer-neutral*: its presence or absence must not change execution semantics. It is a read-only witness to what the executor already does.

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — language whose execution this spec observes (step syntax, error codes)
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust implementation; must provide the AuditProvider hook points
- [l1-nodus-portability.md](l1-nodus-portability.md) — extension-point taxonomy; AuditProvider is one registered role

## 1. Motivation

A single pass/fail score is insufficient for harness evolution: it identifies *that* a workflow failed but not *where*, *why*, or *which component* to change. Structured per-step traces:

- Enable attribution: each failure links to a specific step, command, or constraint
- Enable root-cause analysis: patterns across many runs reveal systematic harness weaknesses
- Enable evidence-backed amendments: proposed workflow changes can cite concrete trace excerpts
- Enable replay-based validation: a proposed amendment can be evaluated on previously-recorded trace inputs
- Preserve history: append-only traces create an immutable audit log of every harness generation

## 2. Constraints & Assumptions

- The observability layer is purely additive — it emits events it never modifies; execution branches, outputs, and variable bindings are unaffected by whether an AuditProvider is attached
- Per-step events are emitted synchronously within the execution; the AuditProvider is called before returning control to the next step
- The AuditProvider may be absent (no-op); conforming implementations must not block on its presence
- Trace records are technology-agnostic: the schema uses named fields and primitive types only — no host-project-specific types
- Append-only is a contract: an implementation that allows trace mutation violates HO-3, regardless of whether mutation actually occurs

## 3. Core Invariants

Rules that Layer 2 implementations MUST NOT violate:

- **HO-1 Trace-first output**: a workflow run produces two outputs — the domain result (values in `@out` / `@err`) and the execution trace. The trace is not optional metadata; it is a primary output. A run that produces a result without a trace is incomplete.

- **HO-2 Per-step attribution**: each trace record identifies the step that generated it (step index, step command name, and step label if present). No event is attributable only to "the run" — it is always attributable to a specific step, constraint check, branch decision, or loop iteration.

- **HO-3 Append-only immutability**: once a trace record is emitted, it is immutable. Subsequent steps may not modify, overwrite, or backfill prior records. A replay of the same input may generate new records but must not alter existing ones.

- **HO-4 Frozen boundary**: the validation and invariant-enforcement pipeline (all NL-invariant checks, E013, E014 and equivalent error codes) is the frozen evaluation layer. Trace analysis may identify amendment candidates; it must never alter how the validator or executor applies invariants. The observability layer is a witness, not a judge.

- **HO-5 Observer neutrality**: attaching an AuditProvider to a workflow run must not change the run's output values, timing-dependent branch outcomes, or error codes. The same workflow with no AuditProvider and with a fully-instrumented AuditProvider must produce semantically identical `@out` and `@err` results on deterministic inputs.

- **HO-6 Structured event taxonomy**: the set of observable event types is closed and versioned. An implementation must not emit ad-hoc string events; it must use the canonical taxonomy defined in §4.2. New event types require a spec amendment (minor version bump).

- **HO-7 Monotonic sequence & correlation** [ADDED v1.1.0]: every emitted event carries (a) a run-monotonic **sequence number** assigned in strict emission order and (b) a **correlation id** shared by all events of the same logical run. Consumers order and deduplicate a multiplexed or reordered event stream by `(correlation_id, sequence)` alone — never by arrival order or wall-clock time (reinforces HO-3 append-only ordering without imposing a transport). Sequence is dense and gap-free within a run; a gap signals dropped events. This makes ordering an explicit contract rather than an emergent property of synchronous emission, so an async or buffered audit sink remains correct.

- **HO-8 Cost-attribution token classes** [ADDED v1.2.0]: a `model_response` event carries a **token-class breakdown** — fresh `input`, `output`, and, when the host provider reports them, `cache_read` and `cache_creation` — as distinct fields, not a single opaque total. Because a cached-prefix read is billed at a fraction of fresh input, collapsing these into one number makes cost un-attributable and a cache regression invisible. The classes are optional (a provider that reports only a total leaves the cache fields absent, never zero-faked) and remain within the data-safety boundary (§4.4 — counts only, never content). This lets a host compute per-run and per-step cost from the trace alone, and lets an outer loop detect a cache-warmth regression from telemetry rather than from an invoice.

## 4. Detailed Design

### 4.1 AuditProvider Role

The `AuditProvider` is a stateful observer registered once per workflow run. It receives events from the executor in emission order. The built-in no-op implementation accepts all events and discards them, satisfying the interface without any I/O.

```text
[REFERENCE]
Role:          AuditProvider
Interface:     record_event(event: ExecutionEvent) → void
               run_complete(manifest: RunManifest)  → void
Built-in:      NoopAuditProvider  (discards all events)
Scope:         one instance per workflow run
               (re-instantiated for each run; not shared across runs)
```

A host project plugs in an `AuditProvider` implementation that persists events to a file, a database, a streaming pipeline, or any other store. The nodus executor calls `record_event` during execution and `run_complete` once the run terminates (success, error, or hard-constraint halt).

### 4.2 Execution Event Taxonomy

| Event type | When emitted | Required fields |
| --- | --- | --- |
| `step_start` | Before a step's command is dispatched | `step_index`, `step_command`, `input_vars` (snapshot) |
| `step_end` | After a step's command returns | `step_index`, `step_command`, `output_vars` (delta), `elapsed_ms` |
| `step_error` | When a step produces an error code | `step_index`, `step_command`, `error_code` (NODUS:* taxonomy), `error_detail` |
| `constraint_hit` | When a hard (`!!NEVER`/`!!ALWAYS`) rule fires | `rule_name`, `triggering_step_index`, `halt: true/false` |
| `branch_taken` | When a conditional (`?IF`/`?ELIF`/`?ELSE`) resolves | `step_index`, `branch_label`, `condition_result` |
| `loop_iteration` | At the start of each `~FOR`/`~UNTIL` body | `step_index`, `loop_type`, `iteration_number`, `bound_vars` |
| `macro_enter` | When `RUN(@macro_name)` begins execution | `macro_name`, `call_step_index` |
| `macro_exit` | When a macro body completes | `macro_name`, `call_step_index`, `elapsed_ms` |
| `model_call` | Immediately before dispatching to the ModelProvider | `step_index`, `command`, `input_summary` (no raw text — §4.4) |
| `model_response` | Immediately after the ModelProvider returns | `step_index`, `command`, `output_summary`, `elapsed_ms`, token classes `input`/`output`/`cache_read?`/`cache_creation?` (HO-8) |

> New event types added in future minor versions must not reuse existing type names.

### 4.3 Run Manifest

At run completion the executor calls `run_complete` with a `RunManifest`:

```text
[REFERENCE]
RunManifest fields:
  workflow_name   : Text     — §wf: identifier
  schema_version  : Text     — BUILTIN_SCHEMA_VERSION at run time
  run_id          : Text     — unique per run (UUID or equivalent)
  started_at      : Text     — ISO-8601 timestamp
  elapsed_ms      : Int      — total wall-clock duration
  status          : Enum     — Ok | Error | ConstraintHalt | ValidationError
  error_code      : Text?    — NODUS:* code if status ≠ Ok
  total_steps     : Int      — count of steps attempted (excluding unentered branches)
  event_count     : Int      — total events emitted
```

### 4.4 Data Safety Boundary

The `model_call` and `model_response` events must not log raw user-content strings. `input_summary` and `output_summary` are structured descriptors (field names, types, length hints) — never the verbatim content. This boundary:

- Prevents user data from appearing in execution logs
- Keeps traces lightweight enough for large-scale analysis
- Allows trace sharing across team members who may not have access to the underlying data

Any field that would contain user-controlled text must be replaced with a schema-level descriptor.

### 4.5 Frozen-vs-Evolvable Boundary

The observability contract identifies which workflow components are frozen (cannot be changed by harness evolution) and which are evolvable:

| Component | Frozen | Evolvable |
| --- | --- | --- |
| NL-1…NL-10 invariants | Yes | No |
| E013/E014 error codes | Yes | No |
| BUILTIN_SCHEMA_VERSION | Yes (per release) | Per minor release |
| `@steps:` sequence | No | Yes |
| `@macro:` definitions | No | Yes |
| `@ctx:` field shape | No | Yes |
| `!!NEVER`/`!PREF` rules | Frozen once active | Evolvable before activation |
| Schema vocabulary (host extension) | No | Yes |

> The frozen layer is the scoring signal. The evolvable layer is the harness. Evolving the frozen layer is a spec amendment, not a harness improvement.

### 4.6 Sequence, Correlation, and Streaming Merge [ADDED v1.1.0]

Every event and manifest gains two ordering fields, and streamed model output is
collapsed into one logical record:

```text
[REFERENCE]
ExecutionEvent gains:
  seq            : Int      — run-monotonic, dense, gap-free emission counter (HO-7)
  correlation_id : Text     — shared by all events of one run (= RunManifest.run_id)

RunManifest gains:
  event_count    : Int      — already present; now also the highest seq + 1 (gap check)
```

**Correlation binding.** The `correlation_id` is set once, at run construction,
from the same identifier as `RunManifest.run_id`. It flows to every event without
the executor threading it through each call site — the audit context holds it for
the run's duration. If a run is entered without an explicit id, the executor
generates one at the root and binds it before the first event, so no event is ever
emitted uncorrelated.

**Streaming merge.** A `model_call`/`model_response` pair may, for a streaming
model, arrive as many incremental chunks. The observability layer merges them into
a **single** logical `model_response` record at completion: chunks are accumulated,
a *finish-reason* predicate detects the terminal chunk, and the merged record
carries the final `output_summary` and total `elapsed_ms`. Individual chunks are
not separate taxonomy events (they would violate HO-6's closed set and inflate the
trace); the merge preserves one attributable record per model turn (HO-2). The
merge is a witness-side fold — it never changes what the model returned to the
workflow (HO-5).

This section is the projection substrate the environment/trajectory contract
(`l1-nodus-environment.md`) relies on: a `TrajectoryEntry.seq` is exactly this
`seq`, so a trajectory is an ordered slice of the correlated event stream.

## 5. Implementation Notes

1. `record_event` is called in the executor's hot path — the built-in no-op implementation costs a single indirect call.
2. The AuditProvider is registered at run construction time (before lexing); an unregistered run uses NoopAuditProvider automatically.
3. `run_id` generation is the caller's responsibility — the executor accepts it as a run parameter, keeping the executor deterministic and testable.
4. `elapsed_ms` is measured by the executor's internal clock, not by wall time; the AuditProvider must not rely on system time for ordering.

## 6. Drawbacks & Alternatives

**Alternative: log-line events** — emit unstructured text logs instead of typed events. Rejected: unstructured logs cannot be queried programmatically; harness analysis requires machine-readable records.

**Alternative: post-run reflection** — reconstruct the trace from final state rather than recording during execution. Rejected: violates HO-2 (per-step attribution) and HO-3 (append-only immutability) because state reconstruction is necessarily lossy for branching/looping execution paths.

**Alternative: embedded tracing library** — import an external observability library (OpenTelemetry etc.) directly into nodus core. Rejected: violates LP-1 (host neutrality) and LP-2 (extension via abstract interfaces) from `l1-nodus-portability.md` — the host chooses the observability backend.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[EXECUTOR]` | `crates/nodus/src/executor.rs` | The execution engine where AuditProvider hook points must be added |
| `[ERROR-CODES]` | `crates/nodus/src/vocab.rs` | NODUS:* error taxonomy (used in `step_error.error_code`) |
| `[INVARIANTS]` | `crates/nodus/src/validator.rs` | E013/E014 and all NL-invariant enforcement (frozen boundary, HO-4) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.2.0 | 2026-07-02 | Core Team | Added HO-8 (cost-attribution token classes on model_response — fresh input/output plus optional cache_read/cache_creation as distinct fields, counts-only within the data-safety boundary) so per-run/per-step cost is computable from the trace and a cache-warmth regression is detectable from telemetry, not the invoice. Event taxonomy §4.2 model_response fields extended. |
| 1.1.0 | 2026-07-01 | Core Team | Added HO-7 (monotonic sequence + correlation id ordering contract) and §4.6 (sequence/correlation fields, run-scoped correlation binding, streaming chunk-merge into one logical model_response). Ordering is now an explicit `(correlation_id, seq)` contract, enabling async/buffered audit sinks; underpins the trajectory projection in l1-nodus-environment.md. |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — HO-1…HO-6, AuditProvider role, event taxonomy, run manifest, data safety boundary, frozen-vs-evolvable boundary |

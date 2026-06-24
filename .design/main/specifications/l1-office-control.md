# Office Control

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Office control is the mechanism for starting, pausing, resuming, and hibernating an office and its subsystems. It exposes a master switch for user-initiated global pause, and a graceful hibernation protocol triggered automatically by resource exhaustion (token quotas, cost limits). When resources recover, the office resumes from exactly where it stopped.

## Related Specifications

- [l1-office-model.md](l1-office-model.md) — the office lifecycle this spec controls
- [l1-orchestration.md](l1-orchestration.md) — orchestrator that coordinates worker drain during state transitions
- [l2-budget-engine.md](l2-budget-engine.md) — token/cost budget source that triggers hibernation
- [l2-model-router.md](l2-model-router.md) — model substitution attempted before hibernation (OC-3)
- [l1-navigation-model.md](l1-navigation-model.md) — office tab status icons reflect OfficeState (NV-3)
- [l2-scheduler.md](l2-scheduler.md) — scheduled jobs aware of office state

## 1. Motivation

Autonomous offices can consume significant model API costs. Users need a reliable global pause mechanism that doesn't lose work. Resource exhaustion (token quota resets, budget limits) also requires a controlled pause: uncontrolled failure cascades are worse than clean hibernation. The office must pause cleanly and resume reliably in both cases.

## 2. Constraints & Assumptions

- Pausing is cooperative: the office drains active operations to a safe checkpoint before freezing, rather than killing mid-step.
- Hibernation is distinct from pause: pause is manual (user-initiated), hibernation is automatic (resource-triggered) with automatic resume on recovery.
- Individual workers do not have independent pause controls; they follow the office's state.
- A paused or hibernating office still serves status queries; it does not accept new task intake.

## 3. Core Invariants

- **OC-1 Safe-checkpoint before freeze**: before entering Paused or Hibernating state, the office completes the current atomic execution step of any in-progress task and writes a checkpoint. An ongoing step MUST NOT be interrupted mid-execution.
- **OC-2 Exact-state resume**: on resume, the office restores from the checkpoint and continues from the interrupted point. No task is silently dropped, re-started from scratch, or duplicated on resume.
- **OC-3 Model degradation before hibernation**: when a specific model's quota is exhausted, the orchestrator MUST attempt model substitution via the model-router before hibernating. The office hibernates only when no viable substitute model is available within the declared budget.
- **OC-4 Automatic resource-recovery wake**: when a hibernation trigger is a recoverable resource (quota window refresh, user budget top-up), the office wakes automatically when the resource recovers — without user action.
- **OC-5 State always visible**: the office's current OfficeState is visible in its tab status icon at all times. A state transition MUST emit a state-change event before the transition is considered complete; silent transitions are not permitted.

## 4. Detailed Design

### 4.1 OfficeState Taxonomy

| State | Description | Entry | Exit |
| --- | --- | --- | --- |
| `Active` | Engine running; workers accept tasks; crons fire | Start / Resume | Pause / Exhaust / Error |
| `Idle` | Running; no active tasks; crons fire as scheduled | All tasks complete | New task arrives |
| `Paused` | User-initiated pause; no task intake; crons suppressed; checkpoint written | Master switch (Active/Idle) | Master switch resume |
| `Hibernating` | Resource-triggered pause; auto-resume on recovery; checkpoint written | Token/quota exhaustion | Resource recovered |
| `Error` | Halted on unrecoverable fault; requires user acknowledgement | Unrecoverable error | User fix + acknowledge |
| `Offline` | Office not loaded in memory | Tab unloaded | Tab activated |

### 4.2 Master Switch

The master switch is available from:
- The office tab's settings dropdown (primary)
- The Office sidebar tab header (secondary)

Semantics:
- `Active` or `Idle` → drain-and-checkpoint → `Paused`
- `Paused` → resume → `Active` (if queued tasks) or `Idle`
- `Hibernating` → early manual resume → `Active` or `Idle`
- No effect on `Error` or `Offline` (different recovery paths)

### 4.3 Token Exhaustion Hibernation Protocol

```text
[REFERENCE]
Trigger: budget engine signals model M quota exhausted

Step 1  — orchestrator queries model-router: viable substitute for M within budget?
  YES   → swap M for M'; log "[OC] Model M replaced by M' (quota exhausted)"; continue
  NO    → proceed to hibernation

Step 2  — orchestrator broadcasts drain signal to all active workers
Step 3  — workers complete current atomic step; write checkpoint
          (or DRAIN_TIMEOUT_MS elapses — partial checkpoints are written with a PARTIAL marker)
Step 4  — office transitions to Hibernating; scheduler suppresses all crons

Step 5  — monitor polls for resource recovery at configured interval (default: 15 min)
Step 6  — on recovery: restore from checkpoint; orchestrator resumes; scheduler reschedules
          suppressed crons from their last-triggered-at timestamp
Step 7  — office transitions to Active or Idle
```

### 4.4 Per-Subsystem Granularity

Beyond the master switch, users may individually pause subsystems from Local Settings:

| Subsystem | Individually pauseable | Effect |
| --- | --- | --- |
| Scheduler (crons) | Yes | Suppresses all scheduled jobs; active tasks unaffected |
| Kanban auto-run | Yes | Prevents new tasks from auto-starting; board remains visible |
| Automation pipelines | Yes | Pauses event-driven automation; active tasks unaffected |
| Heartbeat / Pulse | Yes | Suppresses background routine and inner-monologue runs |

Individual pauses compose with the master switch: subsystems individually paused remain paused after a master resume. The master switch resumes only the subsystems not individually paused.

## 5. Implementation Notes

1. OC-1 drain-and-checkpoint integrates with the nodus executor's atomic step boundary — the executor completes its current step before acknowledging the drain signal.
2. OC-3 model substitution delegates entirely to the model-router fallback cascade; no substitution logic is needed in office-control.
3. OC-4 resource recovery monitoring subscribes to the budget engine's `quota-recovered` event; it does not poll the model provider directly.

## 6. Drawbacks & Alternatives

**Alternative: immediate kill on pause** — terminate tasks without checkpointing. Simpler but violates OC-1 and OC-2; work is lost. Never acceptable for autonomous offices.

**Alternative: no automatic hibernation** — only user-initiated pauses. Uncontrolled quota exhaustion cascades into API errors and corrupted task state. Rejected.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[BUDGET]` | `.design/main/specifications/l2-budget-engine.md` | Quota/cost events triggering OC-3 |
| `[MODEL-ROUTER]` | `.design/main/specifications/l2-model-router.md` | Substitute model selection (OC-3) |
| `[ORCHESTRATION]` | `.design/main/specifications/l1-orchestration.md` | Worker drain coordination (OC-1) |
| `[NAV-MODEL]` | `.design/main/specifications/l1-navigation-model.md` | Status icon display (OC-5, NV-3) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — OC-1…OC-5, OfficeState taxonomy, master switch, token exhaustion protocol, per-subsystem granularity |

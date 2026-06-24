# Automation Pipeline

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The automation pipeline subsystem lets every office worker define reactive automation rules and lets users compose cross-role automation flows. Automation is event-driven: a trigger fires when an event matches a condition; the event travels through a directed graph of processing nodes; an action node closes the loop by delegating work, mutating state, or emitting a notification.

Two surfaces share the same underlying engine:

- **Implicit mode** — each worker embeds automation rules directly in its workflow file. Rules run invisibly under the hood; the user sees only the effects.
- **Explicit mode** — a visual canvas exposes the same engine as a flow graph. Users compose, inspect, and debug cross-role pipelines interactively.

The choice between the two is not either/or: implicit rules handle routine per-worker reactions; the canvas handles non-trivial multi-role orchestration and makes automation legible for users who need to understand or modify it.

## Related Specifications

- [l1-office-model.md](l1-office-model.md) — office, roles, and worker autonomy; automation executes within this model
- [l1-scheduler-model.md](l1-scheduler-model.md) — the scheduler provides schedule-based triggers consumed by this pipeline
- [l1-orchestration.md](l1-orchestration.md) — action nodes delegate to the orchestrator using the delegation protocol
- [l1-extensions.md](l1-extensions.md) — external-service action nodes use the extension/MCP boundary
- [l1-automation-canvas.md](l1-automation-canvas.md) — the visual rendering of the pipeline graph (explicit mode)
- [l2-trigger-triage.md](l2-trigger-triage.md) — event intake and classification that feeds automation triggers
- [l2-scheduler.md](l2-scheduler.md) — schedule-based trigger implementation
- [../../nodus/specifications/l1-nodus-language.md](../../nodus/specifications/l1-nodus-language.md) — `@ON:` trigger syntax in workflow files (implicit mode)
- [../../nodus/specifications/l1-nodus-observability.md](../../nodus/specifications/l1-nodus-observability.md) — AuditProvider that records every pipeline execution

## 1. Motivation

The office model promises maximum automation: clients state *what*, the office handles *how*. As office capability grows, two gaps appear:

1. **Per-worker reaction gaps** — a worker needs to react to a specific event (a Kanban card entering a state, a scheduled time, an incoming webhook) without being explicitly told each time. Implicit automation fills this gap through reactive rules embedded in the worker's workflow.

2. **Cross-role coordination gaps** — complex tasks require ordered handoffs between multiple roles. The orchestrator handles ad-hoc delegation, but recurring multi-role chains benefit from explicit, inspectable pipelines. The visual canvas fills this gap.

Without a formal automation pipeline model, these gaps are filled by one-off coding, brittle event chains, or manual intervention — all of which erode the office's autonomy promise.

## 2. Constraints & Assumptions

- Automation runs entirely within the boundaries of the office it is defined in; cross-office triggers require explicit permission gates.
- The automation engine does not implement business logic: it routes events and invokes workers. Domain logic lives in role workflows.
- The evaluation pipeline (trigger matching, condition evaluation) is deterministic given a fixed event and rule set; side effects occur only in action nodes.
- Automation rules may not grant permissions beyond those already held by the executing role (no privilege escalation through automation).
- Event payloads passed between nodes carry structured, schema-defined fields — never raw session context, user message text, or credential values.

## 3. Core Invariants

Rules that every Layer 2 implementation MUST NOT violate:

- **AP-1 Dual-mode parity**: the implicit (workflow-embedded) mode and the explicit (canvas) mode are two surfaces of the same automation engine. A pipeline defined in either mode executes identically. No pipeline concept exists in one mode but not the other.

- **AP-2 Trigger uniqueness**: within a deduplication window, each event activates at most one pipeline execution per trigger definition. An event that matches multiple trigger definitions activates each independently, but duplicate firings of the same trigger for the same event are suppressed.

- **AP-3 Step atomicity**: each node in a pipeline either completes successfully or fails with a structured error. There is no partial execution state at the node boundary. On failure, the pipeline stops at the failed node unless an explicit error branch is defined.

- **AP-4 Event payload isolation**: event data flowing between nodes is a typed, schema-validated structure. It MUST NOT contain: raw user message text, session context, API credentials, or memory store contents. Descriptors (field names, types, counts) are permitted; verbatim content is not.

- **AP-5 Security boundary**: an action node executes under the permission set of the role that owns the pipeline, not the role that emitted the triggering event. A pipeline MUST NOT escalate to permissions not held by its owning role.

- **AP-6 Observable execution**: every pipeline run produces a structured trace via AuditProvider (per `l1-nodus-observability.md` HO-1…HO-6). The trace records each node's start, completion or failure, and elapsed time. A run without a trace is incomplete.

- **AP-7 Office isolation**: a pipeline defined in office A MUST NOT read or mutate the state of office B. Cross-office triggers require an explicit permission gate and user approval before the pipeline can be activated.

## 4. Detailed Design

### 4.1 Automation Node Taxonomy

A pipeline is a directed acyclic graph of typed nodes. Every execution path from trigger to leaf is finite:

| Node type | Role | Notes |
| --- | --- | --- |
| `trigger` | Entry point; fires when an event matches its condition | At most one per pipeline; a pipeline has exactly one trigger |
| `filter` | Boolean gate; drops execution if condition is false | Short-circuit: false → execution stops on this branch |
| `transform` | Reshapes event payload into a new structure | Pure function; no side effects; inputs and outputs are typed |
| `branch` | Routes execution to one of N outbound paths based on a condition | Mutually exclusive paths; exactly one path is taken |
| `action` | Side-effecting node: delegates task, sends notification, mutates state | Execution leaves the pipeline; result is observable via AuditProvider |
| `delay` | Pauses execution for a specified duration or until a condition is met | Durable pause: survives process restart |
| `aggregate` | Collects N events over a window before passing them downstream | Produces a list payload; emits once when window condition is satisfied |
| `loop` | Iterates over a list payload, executing the subgraph for each element | Bounded by `max_iterations`; exceeding it raises an error |

### 4.2 Trigger Types

| Trigger type | Fires when | Source |
| --- | --- | --- |
| `schedule` | A cron expression or friendly recurrence evaluates to "now" | Scheduler subsystem |
| `state_change` | A named state variable crosses a defined threshold or changes value | Scheduler / Kanban / Memory |
| `kanban_event` | A Kanban card transitions between states, is created, or is archived | Kanban subsystem |
| `message_received` | A role receives a message matching a pattern | Inbox subsystem |
| `webhook` | An external HTTP request arrives at the office's inbound endpoint | Extension / sandbox boundary |
| `external_event` | A subscribed external service emits an event (e.g., an RSS change, an API poll result) | Extension subsystem (polling or push) |
| `manual` | A user or another automation explicitly fires the trigger | Canvas or CLI/TUI command |

All trigger types funnel through the trigger-triage intake pipeline before the automation engine evaluates them.

### 4.3 Pipeline Execution Model

```text
[REFERENCE]
Execution steps for a single pipeline run:

1. INTAKE     — triggering event arrives via trigger-triage; dedup window checked (AP-2)
2. MATCH      — trigger node evaluates its condition against the event payload
3. EVALUATE   — each downstream node evaluated in topological order:
                 filter → drop or continue
                 transform → reshape payload
                 branch  → select one path
                 delay   → suspend; resume on condition
                 aggregate → accumulate; emit when window satisfied
                 loop    → iterate subgraph
                 action  → dispatch (non-blocking or blocking per node config)
4. TRACE      — AuditProvider records each node's start/end/error events (AP-6)
5. TERMINATE  — pipeline reaches a leaf action node or an error halt; run manifest emitted
```

The graph must be acyclic (no pipeline-level loops between nodes). Iteration within a pipeline uses `loop` nodes with explicit bounds, not graph cycles.

### 4.4 Dual-mode Architecture

**Implicit mode (per-worker automation)**

Each worker defines automation rules in its nodus workflow file using `@ON:` trigger blocks. The automation engine binds these blocks as pipeline trigger nodes at worker activation time. The user does not interact with these rules directly — they are part of the worker's role definition.

```text
[REFERENCE]
Implicit mode binding:
  @ON: KANBAN_CARD_DONE              — trigger: kanban_event (state_change → done)
  @ON: SCHEDULE(daily 09:00)         — trigger: schedule (cron)
  @ON: MESSAGE(from:orchestrator)    — trigger: message_received
```

Implicit pipelines are per-worker and per-role. They are not visible in the canvas unless the user explicitly expands a role's embedded rules.

**Explicit mode (canvas-authored pipeline)**

Explicit pipelines are composed in the automation canvas. They are cross-role by design: a trigger in role A can delegate to role B via an action node. Explicit pipelines are stored as versioned pipeline definition files, separate from role workflow files.

Both modes share the same node taxonomy (§4.1) and trigger types (§4.2). The execution model (§4.3) is identical. The canvas is a projection of the same pipeline model, not a different system.

### 4.5 Event Payload Contract

Every event payload flowing between nodes is a typed, named-field structure:

```text
[REFERENCE]
EventPayload fields:
  event_type    : Text     — canonical trigger type name
  source_role   : Text?    — role name that emitted the originating event (absent for external triggers)
  office_id     : Text     — office scope identifier (AP-7)
  run_id        : Text     — unique pipeline run identifier
  timestamp     : Text     — ISO-8601 firing time
  data          : Map      — schema-defined fields; content depends on trigger type
                            (see §4.2 per-trigger payload definitions)
```

Transform nodes produce a new `data` map. The outer envelope fields (`event_type`, `source_role`, `office_id`, `run_id`, `timestamp`) are immutable throughout the run.

### 4.6 Pipeline Definition and Versioning

Each pipeline (implicit or explicit) is a versioned artifact. Versioning follows the portability contract from `l1-nodus-portability.md` LP-6:

- `patch` — description or label change only; no semantic change
- `minor` — additive changes: new node added, new branch path, new trigger type
- `major` — trigger type removed or changed, node removed, payload contract breaking change

Implicit pipeline versions are derived from the enclosing workflow file version. Explicit pipeline definitions are versioned independently and stored in the office's pipeline registry.

## 5. Implementation Notes

1. Implement trigger-triage intake first — nothing can reach the automation engine without it.
2. Wire implicit mode (`@ON:` block binding) before the canvas — it validates the core engine with known-working inputs.
3. Build the deduplication window (AP-2) as a shared primitive used by both modes.
4. Implement action nodes by delegating to existing subsystems (kanban, orchestration, inbox) rather than re-implementing dispatch logic.
5. Add AuditProvider binding (AP-6) at every node boundary, not only at run start/end.
6. The canvas (explicit mode) is a Layer 2 and Layer 1 visualization concern — build it only after the engine is validated in implicit mode.

## 6. Drawbacks & Alternatives

**Alternative: automation as pure orchestrator logic** — let the orchestrator handle all reactive rules as ad-hoc agent instructions, with no formal automation layer. Rejected: ad-hoc delegation is not inspectable, not replayable, and cannot satisfy AP-6 (observable execution). Structured automation enables tracing and harness evolution.

**Alternative: separate engines for implicit and explicit modes** — two distinct runtime implementations, one for embedded `@ON:` blocks and one for canvas pipelines. Rejected: violates AP-1 (dual-mode parity). Divergent engines cause behavioral differences that are impossible to debug across modes.

**Alternative: implicit-only (no canvas)** — ship only per-worker embedded automation, skip the visual layer entirely. Viable for the first release but insufficient for cross-role orchestration and for users who need to inspect automation without reading workflow files. The canvas surfaces value proportional to the complexity of the office's automation graph; it becomes essential beyond 5–6 concurrent automation rules.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[TRIAGE]` | `l2-trigger-triage.md` | Trigger intake and dedup window — the entry gate for all automation events |
| `[SCHEDULER]` | `l2-scheduler.md` | Schedule-based trigger source |
| `[NODUS-LANG]` | `crates/nodus/src/vocab.rs` | `@ON:` trigger syntax — implicit mode binding point |
| `[NODUS-OBS]` | `crates/nodus/src/executor.rs` | AuditProvider hook points — AP-6 observability |
| `[ORCHESTRATION]` | `.design/main/specifications/l1-orchestration.md` | Delegation protocol used by action nodes |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — AP-1…AP-7, node taxonomy, trigger types, dual-mode architecture, event payload contract |

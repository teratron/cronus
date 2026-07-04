# Automation Pipeline

**Version:** 1.3.0
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
- [l2-agent-migration.md](l2-agent-migration.md) — staged, non-destructive apply pattern reused by portable-bundle import (AP-11)
- [l1-memory-model.md](l1-memory-model.md) — distinct from durable node memory (AP-8); node memory is pipeline-local, not the office memory store
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

- **AP-8 Durable node memory**: a node MAY retain durable memory that persists across independent firings (e.g., the last-seen value for change detection, a rolling baseline for spike detection, the set of already-emitted keys for deduplication, an accumulator for digests). Node memory is office-scoped (AP-7), schema-bounded, individually resettable, and subject to the same content exclusions as event payloads (AP-4 — never raw user message text, session context, credentials, or memory-store contents). `transform` nodes remain pure and stateless; memory is an opt-in capability of nodes that need continuity, and is distinct from the office memory store.

- **AP-9 Control plane separate from data plane**: pipelines connect through two distinct edge kinds — *data edges*, along which an emitted event flows from one node to another, and *control edges*, along which one automation enables, disables, or manually fires another. The two graphs are independent: a control edge never carries an event payload, and a data edge never changes a target's enabled state. Control actions obey AP-5 (no privilege escalation) and AP-7 (office isolation); a cross-office control edge requires the same explicit gate as a cross-office trigger.

- **AP-10 Event lifecycle**: an emitted event has a bounded lifetime. Each node declares an event-retention policy; events past their retention are expired and garbage-collected — except the single most-recent retained event a downstream change-detector depends on for continuity. Propagation timing is declared per data edge: *immediate* (the event flows the instant it is emitted) or *deferred* (delivered on the receiver's next scheduled evaluation). Every node exposes a health signal — last-evaluated, last-received, last-error, and a working/not-working predicate — derived from its recent run history (AP-6).

- **AP-11 Portable automation bundles**: a set of pipelines and their edges (data and control) MAY be packaged as a portable bundle — a self-contained, attributed artifact (stable identity, optional origin reference, human label, tags) that can be exported, shared, and imported into another office to re-instantiate the same automation graph. Import is non-destructive (it instantiates new pipelines; it never silently overwrites existing ones) and re-validates AP-5/AP-7 in the destination office before activation. A bundle carries *definitions only* — never node memory, event history, credentials, or office-specific identifiers.

- **AP-12 Pipeline composition**: a pipeline MAY be invoked as a single node within another pipeline — a `subpipeline` node references a pipeline by identity, passes an input payload, and receives the sub-pipeline's terminal output as its own. Composition is bounded by a declared maximum nesting depth and is acyclic across the composition graph (a pipeline MUST NOT invoke itself transitively). The sub-pipeline executes under the **caller's** office scope and permission set (AP-5/AP-7) — being invoked grants no broader rights. Composition is the reuse mechanism for portable bundles (AP-11): an imported pipeline becomes a callable building block.

- **AP-13 Pinned partial re-execution (development)**: for authoring iteration, a node's prior output MAY be *pinned* so the engine re-executes only the downstream subgraph against the pinned value, skipping upstream re-computation. A pinned partial run is a **development run**: it reuses the §4.9 dry-run side-effect quarantine — real `action` dispatch is simulated and recorded, not performed, unless the operator explicitly opts a specific action live. It is traced and marked development (AP-6), and never counts as a production pipeline run nor fires production side effects implicitly.

- **AP-14 Scoped automation state with declared durability**: durable node memory (AP-8) generalizes to a small, explicit **scope hierarchy** — *node-private* (the AP-8 default, visible to one node) and *pipeline-shared* (visible to every node of one pipeline definition). Each scope binds to a named **persistence backend** drawn from a registry: at minimum a *volatile* backend (in-process, lost on restart — for caches and within-run scratch) and a *durable* backend (survives restart — for baselines, dedup horizons, digest accumulators), with one configured default and per-scope override. All scoped state remains office-scoped (AP-7), schema-bounded, individually resettable, and bound by the AP-4 content exclusions (never raw user text, session context, credentials, or memory-store contents). Pipeline-shared state is **not** the office memory store (`l1-memory-model.md`): it has no cross-office recall and no semantic indexing. `transform` nodes stay pure and stateless (AP-8) — they declare no scoped state; statefulness is opt-in, so a node that declares none is guaranteed freely retryable (AP-3).

- **AP-15 In-graph lifecycle observers**: a pipeline MAY contain *observer* nodes that subscribe to the **lifecycle events** of a declared scope of other nodes rather than to incoming data. Three observer kinds: *error* (a node failed), *status* (a node reported a status change), and *completion* (a node finished handling an event). An observer's scope is either an explicit set of nodes or the catch-all *unhandled* set; a scoped observer takes precedence over a catch-all one, and an error handled by no observer in the current scope propagates outward to the enclosing scope — across an AP-12 `subpipeline` boundary to the caller — preserving AP-3's stop-on-failure where no handler exists at all. Observers decouple cross-cutting error/status/completion handling from the happy-path graph: one error observer covers many nodes, including ones the author never wired an error branch onto, instead of a branch per node. An observer obeys AP-5/AP-7, is traced like any node (AP-6), and its emitted payload carries descriptors only (AP-4) — never verbatim error context bearing user content. Observers are the inbound mirror of the AP-9 control plane: AP-9 governs other automations (enable/disable/trigger); AP-15 listens to them (error/status/completion).

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
| `subpipeline` | Invokes another pipeline by reference, passing input and receiving its terminal output (AP-12) | Bounded nesting depth; acyclic across the composition graph; runs under the caller's scope/permissions |
| `observer` | Lifecycle-triggered entry point: fires on the error / status / completion of a declared scope of nodes (AP-15) | No data input; scope is an explicit node set or the catch-all *unhandled* set; scoped precedes catch-all; emits a descriptor payload onto its data edges |

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

**Implicit mode** (per-worker automation)

Each worker defines automation rules in its nodus workflow file using `@ON:` trigger blocks. The automation engine binds these blocks as pipeline trigger nodes at worker activation time. The user does not interact with these rules directly — they are part of the worker's role definition.

```text
[REFERENCE]
Implicit mode binding:
  @ON: KANBAN_CARD_DONE              — trigger: kanban_event (state_change → done)
  @ON: SCHEDULE(daily 09:00)         — trigger: schedule (cron)
  @ON: MESSAGE(from:orchestrator)    — trigger: message_received
```

Implicit pipelines are per-worker and per-role. They are not visible in the canvas unless the user explicitly expands a role's embedded rules.

**Explicit mode** (canvas-authored pipeline)

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

### 4.7 Durable Node Memory (AP-8)

Most nodes are stateless: a `filter` or `transform` produces output purely from its input. Some reactive behaviors, however, require continuity across firings, and that state belongs to the node, not to a side channel:

| Stateful behavior | What memory holds |
| --- | --- |
| Change detection | The last-seen value; emit only when the new value differs |
| Spike / peak detection | A rolling baseline and recent window statistics |
| Gap detection | The timestamp of the last received event; alert when the gap exceeds a bound |
| Deduplication | The set of keys already emitted within a horizon |
| Digest accumulation | Pending items collected until a flush condition (time or count) |

Node memory is a schema-bounded map, office-scoped (AP-7), individually resettable, and bound by the same content exclusions as event payloads (AP-4). It is **not** the office memory store (`l1-memory-model.md`): it is pipeline-local working state with no cross-office recall and no semantic indexing. A node that declares no memory is guaranteed stateless and freely retryable. §4.12 generalizes this to a scope hierarchy (node-private vs pipeline-shared) with a declared volatile-vs-durable persistence backend (AP-14).

### 4.8 Control Plane (AP-9)

Beyond the event-flow graph, automations can govern one another through a separate control graph:

| Edge kind | Carries | Effect on target |
| --- | --- | --- |
| Data edge | An event payload | The target evaluates the event (normal §4.3 flow) |
| Control edge | A control verb | The target's enabled state or firing is changed |

Control verbs are `enable`, `disable`, and `trigger` (fire once now). This separates *what flows* from *what governs*: a window-controller automation can enable a set of targets during business hours and disable them after; a commander automation can manually fire a target on demand. Control edges never carry event data, and an enabled/disabled state change never rides a data edge. All control actions are traced (AP-6) and bound by AP-5/AP-7.

### 4.9 Event Lifecycle (AP-10)

```text
[REFERENCE]
Event lifecycle:
  EMIT       — a node emits an event with a retention policy (keep-for duration; 0 = do not store)
  PROPAGATE  — per edge: immediate (flow now) | deferred (flow on receiver's next evaluation)
  RETAIN     — event persists until its retention horizon
  EXPIRE     — past horizon, the event is garbage-collected,
               EXCEPT the single most-recent event a downstream change-detector needs
  HEALTH     — each node reports last-evaluated / last-received / last-error + working? predicate
```

A node may be **dry-run** against a sample event to preview its output and downstream effect without persisting side effects or emitting real events — the safe way to validate a node before activation (rendered interactively in `l1-automation-canvas.md`).

### 4.10 Portable Automation Bundles (AP-11)

A bundle packages a working automation graph for sharing and reuse:

```text
[REFERENCE]
AutomationBundle {
  bundle_id   : Text     — stable identity for update detection across imports
  label       : Text     — human-readable name
  origin      : Text?     — optional source/author reference (provenance)
  tags        : Text[]    — categorization
  pipelines   : Definition[]  — member pipeline definitions (no memory, no history)
  edges       : Edge[]        — data + control edges among members
}
```

Export serializes the member pipeline definitions and their edges — **definitions only**, never node memory, event history, credentials, or office-specific identifiers. Import is non-destructive: it instantiates new pipelines (resolving naming collisions rather than overwriting), re-binds external-service action nodes to the destination office's own credentials, and re-validates AP-5/AP-7 before any pipeline is activated. Import reuses the staged, reversible apply pattern of `l2-agent-migration.md` (dry-run → instantiate → review → activate). `bundle_id` lets an office detect that an imported bundle has a newer revision available.

### 4.11 Composition and Development Execution (AP-12, AP-13)

**Pipeline composition (AP-12).** A `subpipeline` node turns a whole pipeline into a reusable building block. It references a pipeline by identity, hands it an input payload, and adopts that pipeline's terminal output as its own — the same way an `action` node delegates, except the callee is another pipeline rather than a role:

```text
[REFERENCE]
subpipeline node:
  target      : pipeline identity (local or imported via an AP-11 bundle)
  input       : EventPayload passed as the callee's trigger payload
  output      : the callee's terminal output, returned to the caller graph
  depth_guard : declared max nesting depth; transitive self-invocation rejected
  scope       : executes under the CALLER's office + permission set (AP-5/AP-7)
```

Composition makes shared automations DRY: import a bundle once (AP-11), then invoke it from many callers. The acyclic + depth-bounded rule keeps a composed graph finite, the same guarantee §4.3 makes for a single graph.

**Pinned partial re-execution (AP-13).** While authoring, re-running a whole pipeline to test one downstream change is wasteful — and re-hits upstream side effects. Pinning fixes that: the operator pins a node's last observed output, and the engine re-runs only the subgraph downstream of a chosen node against that pinned value.

```text
[REFERENCE]
Pinned partial run:
  PIN        — capture a node's prior output as a fixed value
  SELECT     — choose the node to re-run from
  EXECUTE    — engine runs only the downstream subgraph; upstream is the pinned value
  QUARANTINE — action dispatch is dry-run (simulated + recorded) per §4.9,
               unless a specific action is explicitly opted live
  MARK       — the run is traced (AP-6) and labelled "development", never production
```

This is an engine capability, not a canvas one: the canvas (`l1-automation-canvas.md` AC-7) only *requests* a pinned partial run and renders the result — it never executes (AC-3). Pinned data is explicitly not real triggering data, so a development run can never silently fire a production action.

### 4.12 Scoped State and Lifecycle Observers (AP-14, AP-15)

**Scoped automation state (AP-14).** §4.7 establishes per-node memory. Some reactive behavior needs state shared by a handful of cooperating nodes in one pipeline — a running counter several nodes increment, a token bucket a rate-limiter and its consumers both read. That belongs to a *pipeline-shared* scope, distinct from node-private memory and from the office store:

```text
[REFERENCE]
Scope hierarchy (narrowest → widest within an office):
  node-private    — one node only (the AP-8 default)
  pipeline-shared — every node of one pipeline definition
                    (NOT the office memory store: no cross-office recall, no semantic index)

Persistence backend (named, registry-resolved, per-scope override over a default):
  volatile  — in-process; lost on restart        → caches, within-run scratch
  durable   — survives restart                    → baselines, dedup horizons, digests

Every scoped store: office-scoped (AP-7), schema-bounded, individually resettable,
content-excluded (AP-4). A node declaring no scoped state is freely retryable (AP-3).
```

Choosing *volatile* vs *durable* is a continuity decision the node makes explicitly: a change-detector's last-seen value is durable (it must survive a restart to stay meaningful); a per-run dedup cache is volatile (a restart legitimately resets it). The backend is named, not hard-wired, so an office can route durable state to its own store implementation without changing pipeline definitions.

**Lifecycle observers (AP-15).** Wiring an error branch onto every node (AP-3) is repetitive and silently misses the nodes the author forgot. An `observer` node inverts that: it declares a *scope* of nodes and a *kind*, and the engine routes those nodes' lifecycle events to it.

```text
[REFERENCE]
observer node:
  kind   : error | status | completion
  scope  : explicit node set  |  "unhandled" (catch-all for the enclosing scope)
  input  : none — fired by the engine on a scoped node's lifecycle event
  output : a descriptor payload (AP-4) onto normal data edges
           (error → failing node id + error code; status → reporting node + state;
            completion → finishing node id)

Routing & precedence (per kind):
  1. a scoped observer covering the node handles the event first
  2. else a catch-all ("unhandled") observer in the same scope handles it
  3. else (error only) the event propagates OUTWARD to the enclosing scope —
     across an AP-12 subpipeline boundary to the caller —
     and, if no observer anywhere handles it, AP-3 stop-on-failure stands
```

The error observer is a scoped exception handler for the pipeline; the completion observer fires *after* a node (or subgraph) finishes, enabling "do X once Y is fully done" without threading a data edge; the status observer surfaces a node's self-reported state (e.g. a long `delay` or `aggregate` reporting progress) to a handler. All three are the inbound complement to the AP-9 control plane — that plane *governs* automations, this one *listens* to them — and both are traced (AP-6) and bounded by AP-5/AP-7.

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

**Alternative: stateless-only nodes (no node memory)** — forbid AP-8 and force every continuity need (change detection, deduplication, digests) into the office memory store or external state. Rejected: it conflates pipeline-local working state with durable office knowledge, defeats clean retryability boundaries, and makes common reactive patterns awkward. Pipeline-local memory keeps the boundary crisp — stateless nodes are freely retryable, stateful nodes declare their memory explicitly.

**Alternative: single edge type (no control plane)** — model enable/disable/trigger as ordinary events on data edges. Rejected: it overloads the payload contract (AP-4) with control semantics and makes "is this automation currently active?" unanswerable from the graph. A separate control plane (AP-9) keeps governance legible and auditable.

**Alternative: no portable bundles** — keep every automation graph office-local and non-exportable. Rejected: it blocks templating and sharing of proven automations and forces re-authoring per office. AP-11's definitions-only, non-destructive, credential-rebinding import makes sharing safe without leaking state.

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
| 1.1.0 | 2026-06-25 | Core Team | AP-8…AP-11 added — durable node memory (change/spike/gap/dedup/digest continuity), control plane separate from data plane (enable/disable/trigger edges), event lifecycle (per-node retention, TTL/GC with change-detector continuity, immediate-vs-deferred propagation, node health predicate, dry-run preview), and portable definitions-only automation bundles (export/import, non-destructive, credential-rebinding, staged apply). §4.7–4.10 added. |
| 1.2.0 | 2026-06-25 | Core Team | AP-12…AP-13 added — pipeline composition (`subpipeline` node invokes another pipeline by identity, bounded depth + acyclic, caller-scoped, the reuse mechanism for AP-11 bundles) and pinned partial re-execution for authoring (pin a node's output, re-run only the downstream subgraph against it, reusing the §4.9 dry-run side-effect quarantine, marked development-not-production). `subpipeline` node type added to §4.1 taxonomy; §4.11 added. |
| 1.3.0 | 2026-06-25 | Core Team | AP-14…AP-15 added — scoped automation state with declared durability (node-private vs pipeline-shared scope hierarchy generalizing AP-8, named volatile-vs-durable persistence backends with a default, still office-scoped + content-excluded, distinct from the office memory store) and in-graph lifecycle observers (`observer` node subscribing to error/status/completion of a declared scope, scoped-precedes-catch-all routing with outward error propagation across `subpipeline` boundaries, the inbound mirror of the AP-9 control plane). `observer` node type added to §4.1 taxonomy; §4.7 cross-linked; §4.12 added. |

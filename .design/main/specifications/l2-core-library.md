# Core Library (Foundation)

**Version:** 1.1.1
**Status:** Stable
**Layer:** implementation
**Implements:** l1-architecture.md

## Overview

Architectural layer 1: the **embeddable core library** that is the foundation of Cronus. It contains all domain logic and exposes a stable programmatic contract consumed by every frontend (CLI, TUI, application) and, optionally, by third-party host programs. It has no presentation dependencies.

## Related Specifications

- [l1-architecture.md](l1-architecture.md) - Concept this layer implements.
- [l2-technology-stack.md](l2-technology-stack.md) - Technology choices for the core.
- [l2-cli.md](l2-cli.md) - A consumer of this core.
- [l2-tui.md](l2-tui.md) - A consumer of this core.
- [l2-app-ui.md](l2-app-ui.md) - A consumer of this core.

## 1. Motivation

A reusable foundation lets every surface share one behavior and lets the engine be embedded elsewhere. The core owns agent orchestration, memory, scheduling, model routing, Kanban state, and persistence so that no frontend has to.

## 2. Constraints & Assumptions

- Written in **Rust** as a library crate; exposes a **C-ABI/FFI** surface for embedding (Tauri backend, mobile static lib, external hosts).
- No UI, terminal, or windowing dependencies (INV-1).
- Async via Tokio; long-running work never blocks a host's UI thread.
- Local-first: the default datastore is an embedded file (SQLite + sqlite-vec); remote/sync is optional.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| INV-1 Embeddable core | Rust library crate with C-ABI; compiles for desktop and iOS/Android targets; linkable into external programs. |
| INV-2 Logic in core only | All domain modules (orchestration, memory, router, scheduler, board, persistence) live here; frontends hold none. |
| INV-3 Command parity | The core contract is the single definition of each capability; frontends bind to it, guaranteeing parity. |
| INV-4 Hub-and-spoke autonomy | The autonomous loop is a core service runnable headless on a hub; on spokes the same crate runs in foreground/sync-only mode. |
| INV-5 Durable, restartable state | Persistence module writes durable state (SQLite file); engine reconstructs sessions/memory/tasks on restart. |
| INV-6 Graceful capability scaling | The contract is capability-flagged; a host enables the subset it can support. |
| INV-7 Security of client data | Secret handling and telemetry separation are enforced in the core, not delegated to frontends. |

## 4. Detailed Design

### 4.1 Core modules (domain)

| Module | Responsibility |
| --- | --- |
| Orchestrator | Hierarchical agent coordination (manager → department agents); task delegation; adaptive hiring of roles on demand. |
| Memory | Tiered memory (working / recall / archival), scope-aware decay + prune, knowledge graph of entities/relations, hybrid retrieval. |
| Model router | Selects local vs cloud models by cost/tokens/capability; fallback cascade; semantic cache. |
| Scheduler | Cron + heartbeat wake queue (coalesced) driving autonomous work without spamming the board. |
| Board | Kanban state machine: `triage → todo → ready → running → blocked → done → archive` (+ custom). |
| Persistence | Durable local state (SQLite + sqlite-vec); optional remote sync (libSQL/PostgreSQL). |
| AI gateway | Local inference (llama.cpp via FFI) and cloud API clients behind one interface. |

### 4.2 Contract surface (illustrative)

The core exposes capability operations consumed by frontends; equivalent to the command set in [l1-architecture.md](l1-architecture.md) §4.4. `[REFERENCE]` illustrative shape, not final API:

```text
[REFERENCE]
core.init(config) -> Session
core.idea(text) -> CapturedIntent
core.plan(scope) -> Plan
core.task() -> Tasks
core.run(scope) -> RunHandle
core.status() -> StatusSnapshot
core.memory(query) -> MemoryResult
core.goal(condition) -> GoalHandle    // autonomous loop, judged termination
core.compact() -> void
core.analyze(scope) -> Report
```

### 4.3 Embedding model

```mermaid
graph TD
    HOST[Host program: Tauri / CLI / TUI / 3rd-party] --> FFI[C-ABI boundary]
    FFI --> CORE[Rust core engine]
    CORE --> DB[(SQLite + sqlite-vec)]
    CORE --> AI[AI gateway: local FFI + cloud]
```

### 4.4 Autonomy safety rails

- Per-agent budget checks before each wake; overspend auto-pauses (circuit breaker).
- Goal completion gated by an independent judge before the loop may stop.
- Checkpointing + context reconstruction for long-horizon runs.

### 4.5 Manager Pattern & Resource Guards

Stateful subsystems (model inference, session context, history, audio) are organized as **managers**: independent structs wrapped in `Arc<T>`, registered in the application state container at startup, and accessed by any handler via a shared reference.

#### `Arc<T>` manager pattern

```text
[REFERENCE]
// At startup:
let model_manager = Arc::new(ModelManager::new(...));
app.manage(model_manager);

// In any handler:
let mm = app.state::<Arc<ModelManager>>();
mm.load_model(model_id).await?;
```

Managers are initialized once in a defined order. Later managers may hold `Arc` references to earlier ones. All domain logic lives inside a manager; no domain logic lives in handler glue code.

#### RAII loading guard

When a manager transitions to a "loading" state, wrap the transition in a guard struct that resets the state on drop — ensuring atomic cleanup even if the work panics or returns early:

```text
[REFERENCE]
struct LoadingGuard<'a>(&'a Manager);

impl Drop for LoadingGuard<'_> {
    fn drop(&mut self) {
        self.0.is_loading.store(false, Ordering::SeqCst);
    }
}

fn load_model(&self, id: &str) -> Result<()> {
    self.is_loading.store(true, Ordering::SeqCst);
    let _guard = LoadingGuard(self);    // cleared on ANY exit path
    // ... loading work ...
    Ok(())
}
```

#### Idle watcher thread

When a manager holds a heavy resource (e.g., a model occupying VRAM), spawn a background thread that polls `last_activity` and releases the resource after a configurable inactivity timeout:

```text
[REFERENCE]
idle_watcher(manager_ref, poll_interval = 10 s):
  loop:
    sleep(poll_interval)
    if resource_loaded AND now - last_activity > inactivity_timeout:
      manager_ref.unload_resource()

InactivityTimeout enum: Never | Immediately | Sec15 | Min2 | Min5 | Min10 | Min15 | Hour1

last_activity is updated by every successful use of the resource.
```

#### Backend enum for polymorphic dispatch

Support multiple interchangeable implementations (e.g., different inference engines) using an enum rather than `dyn Trait`, to avoid heap allocation and retain ownership clarity:

```text
[REFERENCE]
enum LoadedBackend {
    EngineA(EngineAState),
    EngineB(EngineBState),
    // New backend = new variant; callers use match, not dyn dispatch.
}

impl LoadedBackend {
    fn run(&mut self, input: &Input) -> Result<Output> {
        match self {
            Self::EngineA(e) => e.run(input),
            Self::EngineB(e) => e.run(input),
        }
    }
}
```

### 4.6 Typed prefixed ID system

Every entity in the core that requires a stable, sortable identifier uses a **typed prefixed ID**:
a short human-readable type prefix followed by a k-sortable body. The prefix makes the
entity type visible in logs and databases without a schema lookup; the k-sortable body
makes IDs usable as monotonic cursors.

#### Prefix table

| Entity | Prefix | Example |
| --- | --- | --- |
| Session | `ses` | `ses0r7k3mQp2…` |
| Message | `msg` | `msg0r7k3mQp3…` |
| Message part | `prt` | `prt0r7k3mQp4…` |
| Permission | `per` | `per0r7k3mQp5…` |
| Question | `que` | `que0r7k3mQp6…` |
| Event | `evt` | `evt0r7k3mQp7…` |
| Job | `job` | `job0r7k3mQp8…` |
| Worktree | `wrk` | `wrk0r7k3mQp9…` |
| Tool call | `tool` | `tool0r7k3m…` |
| PTY | `pty` | `pty0r7k3mQpA…` |

#### ID structure and generation

```text
[REFERENCE]
ID format: <prefix> <body>   (no separator; prefix is 2–4 lowercase chars, body is 26 chars)

Body encoding:
  timestamp_ms × 0x1000 + monotonic_counter   → BigInt
  BigInt is encoded as base-62 (digits 0–9 A–Z a–z), zero-padded to 26 chars

Ascending (creation-order) ID:
  BigInt = timestamp × 0x1000 + counter   → encodes lexicographically as creation time order

Descending (reverse-order) ID:
  BigInt = MAX_VALUE - (timestamp × 0x1000 + counter)
  → encodes lexicographically as newest-first; useful for "latest items first" queries

Monotonic guarantee:
  lastTimestamp + counter pair ensures no two IDs are identical even within the same
  millisecond (counter resets each time the millisecond advances).

Factory functions:
  ascending(entity_type, given?)  -> ID   // validate prefix if given; else generate new
  descending(entity_type, given?) -> ID   // same; different sort encoding

Validation: if a given ID does not start with the expected prefix, return an error.
  This catches accidental entity-type mixups at the call site rather than the database.
```

#### Branded type usage

IDs are **branded** at the type level so that a `SessionId` cannot be passed where a
`MessageId` is expected:

```rust
// [REFERENCE]
type SessionId  = Branded<String, "SessionId">;
type MessageId  = Branded<String, "MessageId">;

impl SessionId {
    pub fn ascending(given: Option<&str>) -> Result<Self> { /* prefix: "ses" */ }
}
```

Cross-entity operations (e.g., "which messages belong to session X") use the unbranded
string form only at persistence boundaries; all in-memory code uses the branded type.

## 5. Drawbacks & Alternatives

- **C-ABI surface is verbose:** mitigated by generating bindings; the embeddability benefit (INV-1) justifies it.
- **Alternative — TypeScript/Node core:** richer AI SDK ecosystem but high memory footprint and poor mobile embedding; rejected for the foundation. <!-- TBD: confirm whether a thin Go alternative core is worth a spike given repo history -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ARCH]` | `.design/main/specifications/l1-architecture.md` | Invariants the core must satisfy |
| `[STACK]` | `.design/main/specifications/l2-technology-stack.md` | Technology choices for the core |

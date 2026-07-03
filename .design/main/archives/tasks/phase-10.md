---
phase: 10
name: "Advanced Office Features (L2) — Foundational Wave"
status: Done
subsystem: "crates/core (office_control, acp, automation, kanban) + apps/desktop + packages/ui (navigation)"
requires:
  - "Phase 4: agent-session, model-router, context-router, inbox, memory-store"
  - "Phase 5: kanban board, scheduler, budget-engine, quality-pipeline, extension-registry"
  - "Phase 6: orchestration, trigger-triage"
  - "Phase 8: app-ui (Tauri v2 + React 19 shell), IPC bridge"
provides:
  - "office_control: OfficeState machine, cooperative drain/checkpoint, hibernation ladder (substitute-before-hibernate, auto-recovery wake), per-subsystem toggles (OC-1…OC-5)"
  - "acp: session store, monotonic event bus, capability/trust gate, pure projections, steering + interrupt (ACP-1…ACP-10)"
  - "automation: node taxonomy, dedup window, payload isolation, scoped state over volatile/durable backends, control plane, lifecycle observers (AP-1…AP-15 core)"
  - "kanban::custom_boards: custom columns with canonical anchor + saved views + re-anchor audit (KAN-8)"
  - "packages/ui navigation: canonical sidebar catalog, four-layer nesting, floor lazy-load, two-tier settings (NV-1…NV-10 pure logic)"
key_files:
  created:
    - crates/core/src/office_control.rs
    - crates/core/src/acp.rs
    - crates/core/src/automation.rs
    - crates/core/src/kanban/custom_boards.rs
    - packages/ui/src/navigation.ts
    - packages/ui/src/navigation.test.ts
  modified:
    - crates/core/src/lib.rs
    - crates/core/src/kanban/mod.rs
patterns_established:
  - "Domain-logic-first: state machines + trait seams tested against in-memory state; real bus/transport/audio wiring deferred as documented seams"
  - "Event-before-commit state transitions (OC-5); in-process event sink stands in for the event mesh"
duration_minutes: ~
---

# Stage 10 Tasks — Advanced Office Features (L2), Foundational Wave

**Phase:** 10
**Status:** Todo
**Strategic Goal:** Implement the four tap-root Phase-10 L2 specs (office-control, acp, navigation, automation-pipeline) plus the kanban KAN-8 custom-boards delta. These unblock the dependent wave (canvas, deliberation, inner-monologue, global-orchestration) authored in a later `/magic.spec` run. All backend logic in `crates/core`; navigation is presentation-only in the desktop shell (INV-2).

## Atomic Checklist

- [x] [T-10A01] Office Control — OfficeState machine + master switch
- [x] [T-10A02] Office Control — hibernation ladder + per-subsystem toggles
- [x] [T-10AT01] Validation — OC-1…OC-5
- [x] [T-10B01] ACP — session store + monotonic event bus + capability/trust gate
- [x] [T-10B02] ACP — projections + cross-office relay + steering/interrupt
- [x] [T-10BT01] Validation — ACP-1…ACP-10
- [x] [T-10C01] Automation — PipelineEngine + node executor + dedup + payload isolation
- [x] [T-10C02] Automation — scoped state + control plane + lifecycle observers
- [x] [T-10C03] Automation — composition + portable bundles + dev runs + isolation/security
- [x] [T-10CT01] Validation — AP-1…AP-15
- [x] [T-10D01] Kanban KAN-8 — custom columns (canonical anchor) + saved views + re-anchor audit
- [x] [T-10DT01] Validation — KAN-8 anchor resolution + view-not-store
- [x] [T-10E01] Navigation — four-layer tree + floor tab bar + lazy load + live OfficeState icons
- [x] [T-10E02] Navigation — two-tier settings + Open-in-IDE launcher
- [x] [T-10ET01] Validation — NV parity + presentation-only (fallow audit)

## Detailed Tracking

### Track A — Office Control (`crates/core`, `office_control` module)

Foundational: navigation (Track E) subscribes to its OfficeState events.

#### [T-10A01] OfficeState machine + master switch

- **Spec:** l2-office-control.md §4.1, §4.2 (OC-1, OC-2, OC-5)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus --lib office_control::` → 8 passed / 0 failed; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean (exit 0).
- **Handoff:** OfficeState events consumed by T-10E01 (nav status icons) and dashboards.
- **Notes:** State enum Active/Idle/Paused/Hibernating/Error/Offline; single `transition()` mutator emits `StateChange` before commit (OC-5); freeze writes checkpoint / restore clears it (OC-1/OC-2); master `pause`/`resume` (Workload-driven Active vs Idle); rejected edges have no side effects. Event sink is an in-process `Vec<StateChange>` (event-mesh binding) and drain-to-checkpoint is modeled as a flag — real orchestration-bus drain + session-checkpoint store wiring is the documented seam for T-10A02.
- **Changes:** `crates/core/src/office_control.rs` (new, +~230); `lib.rs` `pub mod office_control`. 8 unit tests, std-only, no unwrap on prod paths.

#### [T-10A02] Hibernation ladder + per-subsystem toggles

- **Spec:** l2-office-control.md §4.3, §4.4 (OC-3, OC-4)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus office_control::hibernation` green — a seeded `quota-exhausted` with a viable substitute stays Active (asserts `model_router.substitute` called, no hibernation); with no substitute → Hibernating; a seeded `quota-recovered` auto-resumes; `SubsystemPause` bitset survives a master resume.
- **Handoff:** none (leaf).
- **Notes:** Subscribe to budget-engine `quota-exhausted`/`quota-recovered`; `DRAIN_TIMEOUT_MS`=30_000 (PARTIAL marker on timeout); `RECOVERY_POLL_MS`=900_000 backstop. Substitution delegates entirely to model-router.

#### [T-10AT01] Validation — Office Control

- **Goal:** Verify OC-1…OC-5 against l2-office-control.md.
- **Method:** `cargo test -p cronus office_control` — assert no mid-step interruption (OC-1), exact-state resume with no dup/drop (OC-2), substitute-before-hibernate (OC-3), auto-recovery wake (OC-4), no silent transition — every transition emits its event (OC-5).
- **Status:** Todo

### Track B — ACP (`crates/core`, `acp` module)

Independent of the other tracks; deps (agent-session, security, orchestration) all Done. Unblocks global-orchestration (dependent wave).

#### [T-10B01] Session store + monotonic event bus + capability/trust gate

- **Spec:** l2-acp.md §4.1, §4.2, §4.3 (ACP-1, 2, 5, 7, 8)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus acp::session` green — idempotent `create_session` (concurrent create returns one winner's state to both), per-session `seq` strictly increasing with gap surfacing, `capabilities()` reflects trust level, terminal events embed `remaining_budget`.
- **Handoff:** Bus consumed by projections (T-10B02) and the agent-session `/acp` transport.
- **Notes:** SQLite session store keyed on `session_id` (unique PK for ACP-5); `AtomicU64` seq per session; trust resolved once from security identity.

#### [T-10B02] Projections + cross-office relay + steering/interrupt

- **Spec:** l2-acp.md §4.4, §4.5, §4.6 (ACP-3, 4, 6, 9, 10)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus acp::steering` + `acp::projection` green — two projections over one session observe identical ordered events (ACP-9); a steer cancels not-yet-started actions as `action_skipped` and the turn continues (ACP-10); interrupt fences to a partial terminal and stays resumable (ACP-6); same-session messages serialize (WL-4), overflow → `STEER_REJECTED`.
- **Handoff:** none (leaf for this wave).
- **Notes:** `ProjectionAdapter::translate` is a total pure fn — no logic/state; steering queue bound `STEER_QUEUE_MAX`=16; poll points at loop-start/after-action/after-response/pre-finalize.

#### [T-10BT01] Validation — ACP

- **Goal:** Verify ACP-1…ACP-10 against l2-acp.md.
- **Method:** `cargo test -p cronus acp` — session-oriented, capability declaration, streaming termination, delegation opt-in gate, idempotent create, graceful interrupt, budget transparency, monotonic order + gap surfacing, pure adapters, live steering scoping.
- **Status:** Todo

### Track C — Automation Engine (`crates/core`, `automation` module)

Deps (trigger-triage, scheduler, orchestration) all Done. Unblocks automation-canvas (dependent wave). Largest track — AP-1…AP-15.

#### [T-10C01] PipelineEngine + node executor + dedup + payload isolation

- **Spec:** l2-automation-pipeline.md §4.1, §4.2, §4.3 (AP-1, 2, 3, 4, 6)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus automation::executor` green — one engine runs both an implicit `@ON:`-bound and an explicit def identically (AP-1); dedup suppresses a duplicate `(trigger_id, event_key)` in-window (AP-2); a node returns Ok/Err with no partial state (AP-3); a payload with an excluded content class is rejected by the schema validator (AP-4); every node boundary emits an AuditProvider event (AP-6).
- **Handoff:** Engine consumed by canvas (dependent wave) and worker `@ON:` binding.
- **Notes:** Topological executor over the typed node taxonomy; `action` nodes delegate to kanban/orchestration/inbox — no new dispatch logic; build trigger-triage intake + dedup primitive first.

#### [T-10C02] Scoped state + control plane + lifecycle observers

- **Spec:** l2-automation-pipeline.md §4.4, §4.5 (AP-8, 9, 14, 15)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus automation::state` + `automation::observer` green — node-private vs pipeline-shared scopes bind to volatile/durable backends with a default (AP-14); a `transform` declaring no state is freely retryable (AP-8); control edges carry enable/disable/trigger separately from data edges (AP-9); an `observer` routes scoped-before-catch-all with outward error propagation across a `subpipeline` boundary (AP-15).
- **Handoff:** none.
- **Notes:** Backend registry (volatile in-proc, durable survives restart); all scoped state office-scoped + content-excluded.

#### [T-10C03] Composition + portable bundles + dev runs + isolation/security

- **Spec:** l2-automation-pipeline.md §4.5, §4.6 (AP-5, 7, 10, 11, 12, 13)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus automation::bundle` + `automation::compose` green — `subpipeline` runs under caller scope, depth-guarded + acyclic (AP-12); bundle export serializes definitions only (no memory/history/credentials), import is non-destructive + credential-rebinding + re-validates AP-5/AP-7 (AP-11); a pinned partial run quarantines `action` dispatch and is marked development (AP-13); a run executes under the owning role's permissions, never the emitter's (AP-5); cross-office read/mutate requires an explicit gate (AP-7); event retention GC keeps the one change-detector event (AP-10).
- **Handoff:** none.
- **Notes:** Bundle import reuses the staged agent-migration apply (dry-run → instantiate → review → activate).

#### [T-10CT01] Validation — Automation

- **Goal:** Verify AP-1…AP-15 against l2-automation-pipeline.md.
- **Method:** `cargo test -p cronus automation` — full invariant sweep across the three feature tasks.
- **Status:** Todo

### Track D — Kanban KAN-8 delta (`crates/core`, extends the Phase-5 board)

Amendment delta over the Done Phase-5 board; spec already Stable at l2-kanban-board 1.1.0.

#### [T-10D01] Custom columns (canonical anchor) + saved views + re-anchor audit

- **Spec:** l2-kanban-board.md §3 (KAN-8 compliance row), §4.1 (board.json), l1-kanban-model.md §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus kanban::custom_columns` green — a custom column stores a mandatory `anchor` canonical enum; archival/analytics read the anchor (a card in a custom column reports its anchor state); a saved view is a filter over the single card set (no second card store); re-anchoring appends an audit record.
- **Handoff:** Board UI surfaces in the dependent app-ui board line (later).
- **Notes:** Extends existing `board.json` — additive; canonical states remain non-removable.

#### [T-10DT01] Validation — KAN-8

- **Goal:** Verify KAN-8 against l1-kanban-model 1.1.0 / l2-kanban-board 1.1.0.
- **Method:** `cargo test -p cronus kanban` — assert every custom column resolves to exactly one canonical anchor for cross-office consumers; deleting a view never touches cards.
- **Status:** Todo

### Track E — Navigation (`apps/desktop` + `packages/ui`, presentation-only)

Depends on Track A (OfficeState events for live status icons). Presentation-only — no business logic in TS (INV-2).

#### [T-10E01] Four-layer tree + floor tab bar + lazy load + live OfficeState icons

- **Spec:** l2-navigation.md §4.1, §4.2, §4.3 (NV-1, 2, 3, 6, 7, 8, 9, 10)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `pnpm -C packages/ui test` + `pnpm -C apps/desktop test` green — component tree renders Building⊃Floor⊃Subsystem⊃Mechanism; frozen `SIDEBAR_TABS` order asserted; floor lazy-load calls `office.load`/`unload`; status icon re-renders on an injected `OfficeStateChanged`; pinned home floor is non-closable.
- **Handoff:** T-10E02 adds settings + IDE.
- **Notes:** State authority is the app-shell store; navigation reads floor/state via the IPC bridge, holds no office state.

#### [T-10E02] Two-tier settings + Open-in-IDE launcher

- **Spec:** l2-navigation.md §4.4, §4.5 (NV-4, NV-5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `pnpm -C apps/desktop test` green — Global/Local settings render under a tier label and write via `config.set(scope, …)`; `open_in_ide` reads `workspace_root` + `configured_ide` and invokes the Tauri `shell_spawn` command (asserted via a mocked shell bridge).
- **Handoff:** none.
- **Notes:** Local settings files `.gitignore`d by default.

#### [T-10ET01] Validation — Navigation

- **Goal:** Verify NV-1…NV-10 parity + presentation-only boundary.
- **Method:** `fallow audit --changed-since <base>` clean (no logic in TS, inward deps); component tests assert canonical order + lazy load + live icons.
- **Status:** Todo

## Notes

- **Wave scope**: this phase file covers only the foundational wave. The dependent wave (automation-canvas, voice-input, deliberation, version-control, inner-monologue, lookahead-planning, global-orchestration) is authored via `/magic.spec` then appended here (or a phase-10b file) on its own `/magic.task` pass.
- **Execution mode**: Parallel (C3). Tracks A/B/C/D are independent core work; Track E gates on Track A. Suggested order: A+B+C+D in parallel, then E.
- **Domain-logic-first**: follow the Phase-9 precedent — implement each subsystem's algorithm against seeded/mock state; defer real OS/network integration (live provider quota events, real shell spawn) with a documented per-task note where it applies.

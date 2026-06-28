# Project State

<!-- STATE.md — live project memory. Read FIRST in every workflow session. -->
<!-- Maximum 100 lines. Agent updates AFTER each completed action. -->

**Workspace:** main
**Updated:** 2026-06-28 10:21
**Phase:** 7 — Leaf: TUI
**Status:** Done

## Current Position

- **Task:** T-7A02 Event-driven render loop (poll-snapshot fallback)
- **Spec:** l2-tui.md (single spec for Phase 7)
- **Next Action:** Plan complete — author new scope via /magic.spec main (or /magic.status for a briefing)

## Progress

```
Build phases: Phase 7 (TUI) decomposed & ready ▶  | Done: 2–6  | In-progress: 1 (gap)  | Pending: 8–11
```

## Recent Decisions

- 2026-06-28 **Run prep (Phase 7 de-risk, no code yet):** Pre-flight green (ok:true, header parity clean). Resolved the planner-flagged hidden dependency for the TUI render loop: (1) **Binding pattern** — frontends bind to subsystems by calling the public core modules **directly** (`cronus::kanban::{Board,CardState}`, `cronus::session`, `cronus::workspace::WorkspaceManager`, `cronus::roles::RoleManager`, …), exactly as `crates/cli/src/commands.rs` does; the thin `Capabilities` trait is version/status only, NOT the subsystem surface. TUI parity (INV-3) = invoke the same core fns the CLI verbs do. (2) **No core event bus** → render loop uses the poll-snapshot fallback (re-read core state per tick), INV-5 view-only preserved. (3) **Toolchain OK + network available** → add `crossterm` (± `ratatui`) as the terminal backend; NOT blocked (unlike Phase 1's pnpm/Tauri). T-7A01 needs a DI-mockable terminal abstraction (raw mode is untestable in the non-TTY runner). Paused here for context budget — tree left green (`cargo build -p cronus-tui` passes). Resume execution in a fresh session.

- 2026-06-28 **Decision:** Plan v2.5.0 → v2.6.0 registry re-sync (INDEX raced to v1.0.36). 48 orphaned specs absorbed: 39 newly-authored Stable L1 concepts folded into Phase 0; 4 ready Stable L2 subsystems (resource-sharing, notes, file-store, development-workflow) became Phase 11; 5 non-Stable (l1-spec-driven-governance/dynamic-harness/loop-governance RFC, l2-knowledge-store RFC, l2-loop-runner Draft) parked in Backlog. Phase 7 (TUI) decomposed: 9 tasks, render-loop carries a poll-snapshot fallback if the core lacks an event seam. (Revert: git restore .design/main/PLAN.md .design/main/TASKS.md .design/main/tasks/phase-7.md)

- 2026-06-23 **Decision:** nodus workspace created under `.design/nodus/` — PLAN.md (3 phases), TASKS.md, tasks/phase-1.md (7 tasks: T-1A01..T-1T01). Phase 1 covers spec completeness (TBDs in §4.4 parallel error propagation + §4.3 macro invocation), vocabulary alignment, and NL invariant test coverage.

- 2026-06-23 **Decision:** Phase 6 (Orchestration & Autonomy) confirmed Done and Archived — all 5 tasks complete (orchestration engine, trigger triage, mission mode, deep research, validation). TASKS.md auto-repaired: `Active` → `Done (Archived)`.

- 2026-06-22 **Decision:** Phase 4 (Core Subsystems) fully complete — 14 tasks, 453 tests, 0 failures. Key subsystems delivered: memory store + encryption, codegraph crate, model/context router, agent session loop, context management, session checkpoint, inbox, agent autonomy, workspace management, agent constitution. CLI commands added for all subsystems.

## Blockers

- **Phase 1 decomposition gap (non-blocking):** `l2-sandbox-policy` and `l2-multi-user-auth` are in Phase 1 (PLAN.md) but lack `T-1xxx` tasks. Fold them into `tasks/phase-1.md` when Phase 1 is revisited; they do not gate Phase 5.

## Blocking Constraints

## Session Continuity

**Last Session Ended:** 2026-06-23
**Handoff File:** none
**Bootstrap Mode:** false

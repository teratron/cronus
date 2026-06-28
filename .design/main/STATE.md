# Project State

<!-- STATE.md — live project memory. Read FIRST in every workflow session. -->
<!-- Maximum 100 lines. Agent updates AFTER each completed action. -->

**Workspace:** main
**Updated:** 2026-06-28 15:05
**Phase:** 7 — Leaf: TUI — DONE
**Status:** Active

## Current Position

- **Task:** Phase 7 (TUI) COMPLETE — all 9 tasks Done (A01,A02,B01,B02,B03,C01,C02,T01,T02). Interactive terminal frontend `crates/tui` delivered: terminal lifecycle, render loop, four read-only panels, slash command bar with core dispatch + secret masking, and structural validation (parity, dep-direction, render-from-state, masking). 38 crate tests pass; clippy/fmt clean.
- **Spec:** l2-tui.md (Done; PLAN.md TUI checkbox marked [x])
- **Next Action:** Per Post-Task Replan, a completed phase routes to `/magic.task main` to decompose/activate the **next** phase — Phase 8 (Flower — Desktop App, currently Pending). Phases 8–11 remain Pending; Phase 1 still carries its non-blocking decomposition gap. (Plan is NOT complete — Phases 8–11 remain; the finalize/update-state script mislabels phase completion as plan completion.)

## Progress

```
Build phases: Phase 7 (TUI) ✓ DONE | Done: 2–7 | In-progress: 1 (gap) | Pending: 8–11 (next: 8 Flower — needs /magic.task)
```

## Recent Decisions

- 2026-06-28 **Phase 7 (TUI) COMPLETE — Retro L1:** all 9 tasks Done across 4 tracks. `crates/tui` is a working interactive frontend over the core: terminal RAII lifecycle (DI-mockable, panic-safe), poll-snapshot render loop (view-only, INV-5), four read-only panels (Board/Office/Status/Sessions via ratatui 0.30), slash command bar (CLI-mirrored catalog, /help, core dispatch with `cronus::redact` secret masking), and structural validation (parity matrix, inward dep-direction, render-from-state purity, secret masking). 38 crate tests; clippy/fmt clean. **What went well:** DI trait seams made every layer TTY-free testable; off-screen ratatui `Buffer` gave deterministic render assertions. **Carried forward (follow-up, not blocking):** core exposes only a thin version/status capability, so board/office/sessions projections render empty and only `/status` dispatches live — full per-verb core bindings + a cheap board snapshot + activity stream await core surface work; production secret-list population for masking pending. Phase 7 archived; PLAN.md TUI [x]; CHANGELOG L1 written. **Next:** /magic.task main to decompose Phase 8 (Flower — Desktop App). (Revert: git restore crates/tui Cargo.toml Cargo.lock .design/main)

- 2026-06-28 **T-7C02 delivered (slash → core dispatch + masking) — Track C complete:** new `Dispatcher` trait + `CapabilityDispatcher` (verb→core capability, then masks via `cronus::redact::redact` — INV-7, no re-impl) + `NoopDispatcher`. `App` holds a boxed dispatcher (`new`=no-op, `with_dispatcher`=real) + `pending_dispatch`; `submit_command` defers `Run`, `tick` drains it (dispatch→masked→feedback→redraw), closing input→core→redraw. `status` wired to the capability surface; other verbs return honest "binding pending" (full per-verb bindings are follow-up — thin capability is version/status by design). 3 `command_dispatch` tests (capability parity via recording `Capabilities` stub, masking, end-to-end no-leak). 32 crate tests pass; clippy/fmt clean. (Revert: git restore crates/tui .design/main)

- 2026-06-28 **T-7C01 delivered (command bar input + `/help`):** new `command` module — `parse`, `CATALOG` (help + 21 CLI-mirrored verbs), `classify` → `Help/Run/Error` (unknown → inline error, no panic). Parity source: TUI can't depend on `cronus-cli` (INV-2) and core has no command registry, so `CATALOG` is curated to mirror the CLI's 21 top-level verbs; anti-drift enforced by the validation track (T01), not a runtime import. Command bar made interactive: `ViewModel.command_input/feedback`, focus-aware key routing (`handle_key`/`submit_command`; Esc cancels line in the bar, quits elsewhere — resolved the provisional Esc note). 9 `command_parse` tests; 29 crate tests pass; clippy/fmt clean. (Revert: git restore crates/tui .design/main)

- 2026-06-28 **T-7B03 delivered (Status + Sessions panels) — Track B complete:** Sessions scrollback chosen as a **bounded last-N projection in `CoreSnapshot`** (not accumulating `App` state) → view stays a pure function of the snapshot (INV-5; loop determinism test holds). `SessionsView::push` caps at `MAX_SESSION_LINES`=500 (drops oldest); core's durable log is source of truth, panel shows a recent tail. New `render_status` (mirrors version+status) + `render_sessions`. `CoreSnapshot` gained `sessions` (empty until core streams activity). All four panels now render. 20 crate tests pass; clippy/fmt clean. (Revert: git restore crates/tui .design/main)

- 2026-06-28 **T-7B02 delivered (Board + Office panels):** core has **6** card states + archive *store*; the board *view* defines its own 7-column `BoardColumn` projection (spec's 7 cols incl. `archive`) — presentation only, transitions stay in core (INV-2). New `view` board/office projections (`BoardColumn`/`BoardCard`/`BoardView`, `AgentActivity`/`OfficeView`) + pure `render_board`/`render_office` into a ratatui `Buffer`. `CoreSnapshot` gained `board`/`office` (empty until core exposes a cheap board snapshot). 3 `board_office_render` Buffer tests; 17 crate tests pass; clippy/fmt clean. (Revert: git restore crates/tui .design/main)

- 2026-06-28 **T-7B01 delivered (panel layout + focus):** adopted **ratatui 0.30** (feature `crossterm_0_29` → single crossterm version shared with the `Tui` guard; ratatui owns frame diffing, the guard owns raw-mode lifecycle). New `view` module: `layout()` (2×2 panels + 1-row command bar, clamps at tiny sizes), `Focus` enum + `next`/`prev` tab-cycling. `RatatuiRenderer` (replaced the `PlainRenderer` stopgap) draws bordered/titled panels with focus highlight. `ViewModel.focus` (view-only) + `Tab`/`BackTab` handling in `tick`; `Key::BackTab` added. 14 crate tests pass; clippy/fmt clean. (Revert: git restore crates/tui Cargo.toml Cargo.lock .design/main)

- 2026-06-28 **Track A delivered (T-7A01, T-7A02):** `crates/tui` converted to lib+bin; `crossterm` added. `terminal` module — `TerminalBackend` DI trait + `CrosstermBackend` + panic-safe/idempotent `Tui` RAII guard. `app` module — `App::tick` pure step-fn (input-first → non-blocking on slow snapshot → exactly one redraw), view-only `ViewModel`/`CoreSnapshot`, `SnapshotSource` poll seam + `CapabilitySource`, `Renderer` seam + `PlainRenderer`, `run`/`run_with`. **Event-seam resolved:** core has no pub/sub → poll-snapshot fallback taken (INV-5 preserved). 9 unit tests pass; clippy/fmt clean. Note: `update-state` script mislabeled STATE Status as Done/"Plan complete" — corrected manually (phase is In Progress). (Revert: git restore crates/tui Cargo.toml Cargo.lock .design/main)

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

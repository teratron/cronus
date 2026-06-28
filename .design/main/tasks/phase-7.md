---
phase: 7
name: "Leaf — TUI"
status: Todo
subsystem: "crates/tui"
requires:
  - "core::Capabilities contract + Engine (Phase 1)"
  - "CLI command grammar + core bindings (Phase 3) — parity source"
  - "core subsystems exposing observable state/events: kanban-board, agent-session, orchestration (Phases 4–6)"
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 7 Tasks — Leaf: TUI

**Phase:** 7
**Status:** Todo
**Strategic Goal:** An interactive, keyboard-driven terminal frontend (`crates/tui`) over the now-mature core — live Board / Office / Status / Sessions panels plus a slash-command bar with 1:1 parity to the CLI capability set. Pure presentation: rendering and input only, all behavior delegates to the core (INV-2); the TUI holds view state, never domain state (INV-5).

> **Architectural guardrails (l2-tui §3):** the crate links `cronus` (core) and must NOT depend on `cronus-cli` or carry domain logic (INV-2). Slash commands map 1:1 to the shared capability set (INV-3). Secrets are never rendered (INV-7). The render loop is async and never blocks on long core calls (§4.2).
>
> **Open dependency (resolve in T-7A02):** the event-driven loop assumes a core event/subscribe seam. If the core exposes no observer/pub-sub, fall back to polling durable-state snapshots on a tick (still INV-5 view-only). Track B panels render from a view-model snapshot, so they are not hard-blocked on the live subscription.

## Atomic Checklist

Track A — Shell & Render Loop (l2-tui §4.2)

- [ ] [T-7A01] Terminal backend + raw-mode lifecycle (enter/restore, panic-safe teardown, resize)
- [ ] [T-7A02] Event-driven async render loop: core event subscription (poll-snapshot fallback) → view-model update → redraw

Track B — View Panels (l2-tui §4.1)

- [ ] [T-7B01] Panel layout + focus/tab navigation across the four views and the command bar
- [ ] [T-7B02] Board + Office panels (live Kanban columns `triage→todo→ready→running→blocked→done→archive`; agent/task text schema)
- [ ] [T-7B03] Status + Sessions/Log panels (status mirror; live agent-activity / log stream)

Track C — Command Bar & Parity (l2-tui §4.3)

- [ ] [T-7C01] Command bar input + `/help` discovery (slash parser + command catalog)
- [ ] [T-7C02] Slash → core dispatch with CLI parity and secret masking (INV-3 / INV-7)

Track T — Validation

- [ ] [T-7T01] Validate command parity + dependency direction (each `/cmd` ↔ a CLI verb; `tui → core`, not `cli`)
- [ ] [T-7T02] Validate render-from-state + secret masking

## Detailed Tracking

### [T-7A01] Terminal backend + raw-mode lifecycle

- **Spec:** l2-tui.md §2 (Constraints), §4.2 (Render loop)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui terminal_lifecycle` — entering raw mode then dropping the guard restores cooked mode and the alternate screen (no leaked terminal state); a simulated panic still runs the restore path. `cargo build -p cronus-tui` exit 0.
- **Handoff:** Provides the `Terminal`/`Tui` RAII guard that T-7A02 drives.
- **Notes:** Any ANSI terminal; use a single backend (crossterm) behind a `cronus-tui`-local wrapper. Resize events propagate to a redraw request. No core calls here — pure terminal plumbing.

### [T-7A02] Event-driven async render loop

- **Spec:** l2-tui.md §4.2, l1-architecture.md (INV-2 logic-in-core, INV-5 view-only state)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui render_loop` — a stub core event/snapshot source drives the loop; asserting (a) a core state change updates the view-model and schedules exactly one redraw, (b) a slow core call does not block input handling (loop stays responsive), (c) the loop holds no domain state beyond the view-model snapshot.
- **Handoff:** Establishes the `App` view-model + the subscribe/poll seam every panel reads from.
- **Notes:** Prefer a core event/observe subscription; if absent, poll `Engine` state snapshots on a tick. Either way the TUI never mutates domain state (INV-5). This task resolves the open event-seam dependency noted above — record which path was taken in `provides`.

### [T-7B01] Panel layout + focus navigation

- **Spec:** l2-tui.md §4.1 (Views)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui layout_focus` — the layout splits into the four view regions + command bar at representative terminal sizes (no panic on small sizes); tab/focus cycling visits each focusable region in order and wraps.
- **Handoff:** Hosts the panels from T-7B02 / T-7B03 and the command bar from T-7C01.
- **Notes:** Render-only components fed by the view-model (INV-2). Unsupported-capability panels are hidden/disabled, never behaviorally divergent (INV-6).

### [T-7B02] Board + Office panels

- **Spec:** l2-tui.md §4.1 (Board, Office)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui board_office_render` — given a view-model with cards across the seven columns, the Board renders each card under its column and a card moving column in the next snapshot re-renders in the new column; the Office panel renders the agent→current-task text schema from the snapshot.
- **Handoff:** Reads the kanban / office projection from the T-7A02 view-model.
- **Notes:** Columns `triage→todo→ready→running→blocked→done→archive`. Presentation only — no board mutation logic in the TUI.

### [T-7B03] Status + Sessions/Log panels

- **Spec:** l2-tui.md §4.1 (Status, Sessions/Log)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui status_sessions_render` — the Status panel renders current position / progress / blockers mirroring the `status` capability snapshot; the Sessions/Log panel appends streamed activity entries in order and bounds its scrollback (no unbounded growth).
- **Handoff:** Completes the read-only view surface ahead of command dispatch (Track C).
- **Notes:** Status mirrors the same capability the CLI `status` verb reports (INV-3). Log stream is append-with-cap.

### [T-7C01] Command bar input + `/help` discovery

- **Spec:** l2-tui.md §4.1 (Command bar), §4.3 (Parity with CLI)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui command_parse` — the bar parses `/verb arg…` into a structured command; `/help` lists the available slash commands (the discovery surface); unknown `/verb` yields an inline error, never a panic.
- **Handoff:** Produces parsed commands for T-7C02 to dispatch.
- **Notes:** The slash catalog is derived from the shared capability set, not hand-maintained, so it cannot drift from the CLI.

### [T-7C02] Slash → core dispatch with parity + secret masking

- **Spec:** l2-tui.md §4.3 (Parity), l1-architecture.md (INV-3 parity, INV-7 secret safety)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui command_dispatch` — each parsed `/verb` invokes the same core capability the CLI verb binds to (asserted against a stub Engine recording the call), and rendered command output masks known secret patterns (no secret value reaches the screen buffer).
- **Handoff:** Closes the input→core→redraw cycle (feeds back into T-7A02).
- **Notes:** Behavior difference vs the CLI is presentation only (INV-3). Reuse the core redaction path for masking (INV-7); do not re-implement redaction in the TUI.

### [T-7T01] Validation — command parity + dependency direction

- **Goal:** Prove INV-3 (parity) and INV-2 (no domain logic / inward dependency) structurally.
- **Method:** `cargo test -p cronus-tui parity_matrix` — every TUI slash command resolves to a capability also reachable from the CLI verb set (no TUI-only behavior). `cargo tree -p cronus-tui` shows `cronus` (core) among workspace crates and **not** `cronus-cli`. `cargo clippy -p cronus-tui --all-targets -- -D warnings` and `cargo fmt -p cronus-tui --check` clean.
- **Status:** Todo

### [T-7T02] Validation — render-from-state + secret masking

- **Goal:** Prove INV-5 (view-only, render purely from snapshot) and INV-7 (secrets never rendered).
- **Method:** `cargo test -p cronus-tui render_state` — panels are a pure function of the view-model snapshot (same snapshot ⇒ identical frame; no hidden mutable domain state). `cargo test -p cronus-tui mask_secrets` — a snapshot/command output carrying a known secret renders masked.
- **Status:** Todo

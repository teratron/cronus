---
phase: 7
name: "Leaf — TUI"
status: Done
subsystem: "crates/tui"
requires:
  - "core::Capabilities contract + Engine (Phase 1)"
  - "CLI command grammar + core bindings (Phase 3) — parity source"
  - "core subsystems exposing observable state/events: kanban-board, agent-session, orchestration (Phases 4–6)"
provides:
  - "Terminal RAII lifecycle guard (DI-mockable backend, panic-safe + idempotent restore)"
  - "Event-driven poll-snapshot render loop (pure tick step-fn; view-only, INV-5)"
  - "Panel layout + Tab/Shift+Tab focus navigation across four views + command bar"
  - "Read-only Board/Office/Status/Sessions panels (ratatui), bounded sessions scrollback"
  - "Slash command bar: parser, CLI-mirrored catalog, /help discovery, core dispatch with secret masking"
  - "Structural validation: command parity, inward dependency direction, render-from-state purity, secret masking"
key_files:
  created:
    - "crates/tui/src/lib.rs"
    - "crates/tui/src/terminal.rs"
    - "crates/tui/src/app.rs"
    - "crates/tui/src/view.rs"
    - "crates/tui/src/command.rs"
  modified:
    - "crates/tui/Cargo.toml"
    - "crates/tui/src/main.rs"
    - "Cargo.toml"
    - "Cargo.lock"
patterns_established:
  - "DI trait seams (TerminalBackend / SnapshotSource / Renderer / Dispatcher) enable TTY-free unit testing"
  - "Poll-snapshot fallback (no core event bus) keeps the view a pure function of the snapshot (INV-5)"
  - "ratatui 0.30 (feature crossterm_0_29, single crossterm version) as the panel framework; off-screen Buffer for render assertions"
  - "Secret masking at the dispatch boundary via cronus::redact — never re-implemented in the frontend (INV-7)"
  - "Cross-crate command parity validated without the forbidden cli dependency (curated verb set + compile-time manifest check)"
duration_minutes: ~
---

# Stage 7 Tasks — Leaf: TUI

**Phase:** 7
**Status:** Done
**Strategic Goal:** An interactive, keyboard-driven terminal frontend (`crates/tui`) over the now-mature core — live Board / Office / Status / Sessions panels plus a slash-command bar with 1:1 parity to the CLI capability set. Pure presentation: rendering and input only, all behavior delegates to the core (INV-2); the TUI holds view state, never domain state (INV-5).

> **Architectural guardrails (l2-tui §3):** the crate links `cronus` (core) and must NOT depend on `cronus-cli` or carry domain logic (INV-2). Slash commands map 1:1 to the shared capability set (INV-3). Secrets are never rendered (INV-7). The render loop is async and never blocks on long core calls (§4.2).
>
> **Open dependency (resolve in T-7A02):** the event-driven loop assumes a core event/subscribe seam. If the core exposes no observer/pub-sub, fall back to polling durable-state snapshots on a tick (still INV-5 view-only). Track B panels render from a view-model snapshot, so they are not hard-blocked on the live subscription.

## Atomic Checklist

Track A — Shell & Render Loop (l2-tui §4.2)

- [x] [T-7A01] Terminal backend + raw-mode lifecycle (enter/restore, panic-safe teardown, resize)
- [x] [T-7A02] Event-driven async render loop: core event subscription (poll-snapshot fallback) → view-model update → redraw

Track B — View Panels (l2-tui §4.1)

- [x] [T-7B01] Panel layout + focus/tab navigation across the four views and the command bar
- [x] [T-7B02] Board + Office panels (live Kanban columns `triage→todo→ready→running→blocked→done→archive`; agent/task text schema)
- [x] [T-7B03] Status + Sessions/Log panels (status mirror; live agent-activity / log stream)

Track C — Command Bar & Parity (l2-tui §4.3)

- [x] [T-7C01] Command bar input + `/help` discovery (slash parser + command catalog)
- [x] [T-7C02] Slash → core dispatch with CLI parity and secret masking (INV-3 / INV-7)

Track T — Validation

- [x] [T-7T01] Validate command parity + dependency direction (each `/cmd` ↔ a CLI verb; `tui → core`, not `cli`)
- [x] [T-7T02] Validate render-from-state + secret masking

## Detailed Tracking

### [T-7A01] Terminal backend + raw-mode lifecycle

- **Spec:** l2-tui.md §2 (Constraints), §4.2 (Render loop)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui terminal_lifecycle` — entering raw mode then dropping the guard restores cooked mode and the alternate screen (no leaked terminal state); a simulated panic still runs the restore path. `cargo build -p cronus-tui` exit 0.
- **Handoff:** Provides the `Terminal`/`Tui` RAII guard that T-7A02 drives.
- **Notes:** Any ANSI terminal; use a single backend (crossterm) behind a `cronus-tui`-local wrapper. Resize events propagate to a redraw request. No core calls here — pure terminal plumbing.
- **Changes:** Converted `cronus-tui` to lib+bin; added `crossterm` (workspace dep). New `terminal` module: `TerminalBackend` DI trait (enter/leave/size/poll_event), `CrosstermBackend` production impl (Windows Press-only key filter), `Tui<B>` RAII guard with panic-safe + idempotent restore, folded `Key`/`TermEvent` vocab. 4 `terminal_lifecycle` unit tests via recording fake (no TTY). Verify: build exit 0, 4 tests pass, clippy/fmt clean.

### [T-7A02] Event-driven async render loop

- **Spec:** l2-tui.md §4.2, l1-architecture.md (INV-2 logic-in-core, INV-5 view-only state)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui render_loop` — a stub core event/snapshot source drives the loop; asserting (a) a core state change updates the view-model and schedules exactly one redraw, (b) a slow core call does not block input handling (loop stays responsive), (c) the loop holds no domain state beyond the view-model snapshot.
- **Handoff:** Establishes the `App` view-model + the subscribe/poll seam every panel reads from.
- **Notes:** Prefer a core event/observe subscription; if absent, poll `Engine` state snapshots on a tick. Either way the TUI never mutates domain state (INV-5). This task resolves the open event-seam dependency noted above — record which path was taken in `provides`.
- **Event-seam resolved:** the core exposes **no** observer/pub-sub (its `Capabilities` surface is version/status only), so the loop took the **poll-snapshot fallback** — a `SnapshotSource` polled per tick, `App` dedupes by snapshot equality (still INV-5 view-only). An event-driven source can later implement the same trait without changing the loop.
- **Changes:** New `app` module: `App::tick` pure step-function (input-first ordering keeps the loop responsive while a snapshot is in flight; coalesces to exactly one redraw per change), `ViewModel`/`CoreSnapshot` view-only projection, `SnapshotSource` poll seam + `CapabilitySource` production impl over the core, `Renderer` seam (`&ViewModel` enforces render-from-state) + minimal `PlainRenderer`, `run`/`run_with` drivers (RAII-guarded terminal). Binary now launches the loop. 5 `render_loop` unit tests via scripted source + recording renderer. Verify: 9 crate tests pass, clippy/fmt clean.

### [T-7B01] Panel layout + focus navigation

- **Spec:** l2-tui.md §4.1 (Views)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui layout_focus` — the layout splits into the four view regions + command bar at representative terminal sizes (no panic on small sizes); tab/focus cycling visits each focusable region in order and wraps.
- **Handoff:** Hosts the panels from T-7B02 / T-7B03 and the command bar from T-7C01. Panel content builders render into the `view::layout` regions; the `Focus` enum + `ViewModel.focus` drive the highlight.
- **Notes:** Render-only components fed by the view-model (INV-2). Unsupported-capability panels are hidden/disabled, never behaviorally divergent (INV-6).
- **Framework decision:** adopted **ratatui 0.30** (feature `crossterm_0_29`, single crossterm version with the `Tui` guard). ratatui owns frame diffing; the `Tui` guard keeps the raw-mode lifecycle.
- **Changes:** New `view` module — `layout(area)` splits a 2×2 panel grid above a one-row command bar (ratatui `Layout::areas`, clamps at tiny sizes), `Focus` enum + `next`/`prev` tab-cycling, `PanelAreas`. `RatatuiRenderer` (replaces the `PlainRenderer` stopgap) draws bordered/titled panels with a focus highlight + command-bar prompt. `ViewModel` gained view-only `focus`; `App::tick` handles `Tab`/`BackTab` → focus cycle + redraw; `terminal::Key` gained `BackTab`. Verify: 5 `layout_focus` tests; 14 crate tests pass; clippy/fmt clean.

### [T-7B02] Board + Office panels

- **Spec:** l2-tui.md §4.1 (Board, Office)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui board_office_render` — given a view-model with cards across the seven columns, the Board renders each card under its column and a card moving column in the next snapshot re-renders in the new column; the Office panel renders the agent→current-task text schema from the snapshot.
- **Handoff:** Reads the kanban / office projection from the T-7A02 view-model.
- **Notes:** Columns `triage→todo→ready→running→blocked→done→archive`. Presentation only — no board mutation logic in the TUI.
- **Column-model note:** the core models **6** live card states (`Triage→Todo→Ready→Running→Blocked→Done`) plus an archive *store* (archive is not a state). The board *view* defines its own 7-column `BoardColumn` projection (the spec's 7 columns incl. `archive`); the binding maps live cards by state and archived cards into the `Archive` column. Transition logic stays in the core (INV-2).
- **Changes:** `view` module gained the board projection (`BoardColumn` ×7, `BoardCard`, `BoardView::cards_in`, `board_columns` splitter) and office projection (`AgentActivity`, `OfficeView`), plus pure `render_board`/`render_office` into a ratatui `Buffer`. `CoreSnapshot` extended with `board`/`office` (empty until the core exposes a cheap board snapshot; `CapabilitySource` leaves them default). `RatatuiRenderer` now renders Board + Office content; `focus_border_style` centralized in `view`. 3 `board_office_render` tests assert card-under-column, column-move re-render, and agent→task lines via off-screen Buffer. Verify: 17 crate tests pass; clippy/fmt clean.

### [T-7B03] Status + Sessions/Log panels

- **Spec:** l2-tui.md §4.1 (Status, Sessions/Log)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui status_sessions_render` — the Status panel renders current position / progress / blockers mirroring the `status` capability snapshot; the Sessions/Log panel appends streamed activity entries in order and bounds its scrollback (no unbounded growth).
- **Handoff:** Completes the read-only view surface ahead of command dispatch (Track C).
- **Notes:** Status mirrors the same capability the CLI `status` verb reports (INV-3). Log stream is append-with-cap.
- **Scrollback design:** the Sessions log is a **bounded last-N projection inside `CoreSnapshot`** (not accumulating `App` view-state), so the view stays a pure function of the snapshot (INV-5; the loop determinism test holds). `SessionsView::push` caps at `MAX_SESSION_LINES` (500), dropping the oldest. The core's durable activity log remains the source of truth; the panel shows a recent tail.
- **Changes:** `view` module gained `render_status` (mirrors `version` + `status` line) and the `SessionsView` projection (`push` with cap, `entries`, `MAX_SESSION_LINES`) + `render_sessions` (recent tail that fits, oldest-of-window top, newest bottom). `CoreSnapshot` extended with `sessions` (empty until the core streams activity; `CapabilitySource` leaves it default). `RatatuiRenderer` now renders all four panels. 3 `status_sessions_render` tests (status mirror, append-in-order, bounded scrollback). Verify: 20 crate tests pass; clippy/fmt clean. Track B (read-only view surface) complete.

### [T-7C01] Command bar input + `/help` discovery

- **Spec:** l2-tui.md §4.1 (Command bar), §4.3 (Parity with CLI)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui command_parse` — the bar parses `/verb arg…` into a structured command; `/help` lists the available slash commands (the discovery surface); unknown `/verb` yields an inline error, never a panic.
- **Handoff:** Produces parsed commands for T-7C02 to dispatch. `command::classify` returns `Run(SlashCommand)` for known verbs — T-7C02 swaps the acknowledgement for a real core call.
- **Notes:** The slash catalog is derived from the shared capability set, not hand-maintained, so it cannot drift from the CLI.
- **Parity-source note:** the crate cannot depend on `cronus-cli` (INV-2) and the core exposes no enumerable command registry (`Capabilities` = version/status; other ops are direct module calls), so `command::CATALOG` is curated in the TUI to mirror the CLI's 21 top-level verbs. Anti-drift is enforced structurally by the validation track (each slash verb ↔ a CLI verb), not by a runtime import.
- **Changes:** New `command` module — `parse` (`/verb arg…` → `SlashCommand`, whitespace-tolerant), `CATALOG` (help + 21 CLI-mirrored verbs), `lookup`/`is_known`/`names`/`help_lines`, `classify` → `CommandOutcome::{Help, Run, Error}` (unknown verb → inline error, never panic). Command bar made interactive: `ViewModel.command_input`/`command_feedback`, focus-aware key routing in `tick` (`handle_key`/`submit_command`: type/Backspace/Enter; Esc cancels the line in the bar but quits elsewhere — resolves the earlier provisional Esc note); the bar renders the live input or the last feedback. 9 `command_parse` tests (parser, catalog/help, classify, bar typing+enter, unknown+esc-cancel). Verify: 29 crate tests pass; clippy/fmt clean.

### [T-7C02] Slash → core dispatch with parity + secret masking

- **Spec:** l2-tui.md §4.3 (Parity), l1-architecture.md (INV-3 parity, INV-7 secret safety)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-tui command_dispatch` — each parsed `/verb` invokes the same core capability the CLI verb binds to (asserted against a stub Engine recording the call), and rendered command output masks known secret patterns (no secret value reaches the screen buffer).
- **Handoff:** Closes the input→core→redraw cycle (feeds back into T-7A02).
- **Notes:** Behavior difference vs the CLI is presentation only (INV-3). Reuse the core redaction path for masking (INV-7); do not re-implement redaction in the TUI.
- **Dispatch-surface note:** `status` is routed to the `Capabilities` surface; other known verbs are recognized but return an honest "core binding not yet surfaced" line — the thin TUI capability is version/status by design, full per-verb bindings (kanban/goal/… via direct core modules, as the CLI does) are follow-up. The masking path is wired + tested regardless.
- **Changes:** New `Dispatcher` trait + `CapabilityDispatcher` (routes verb→core capability, then masks output via `cronus::redact::redact` — INV-7, no re-implementation) + `NoopDispatcher`. `App` holds a boxed dispatcher (`new` = no-op, `with_dispatcher` = real) + `pending_dispatch`; `submit_command` defers `Run` to the loop, `tick` drains it (dispatch → masked output → feedback → redraw), closing input→core→redraw. `run`/`run_with` thread the dispatcher. 3 `command_dispatch` tests (capability parity via recording `Capabilities` stub; secret masking; end-to-end no-leak through the bar). Verify: 32 crate tests pass; clippy/fmt clean. **Track C complete.**

### [T-7T01] Validation — command parity + dependency direction

- **Goal:** Prove INV-3 (parity) and INV-2 (no domain logic / inward dependency) structurally.
- **Method:** `cargo test -p cronus-tui parity_matrix` — every TUI slash command resolves to a capability also reachable from the CLI verb set (no TUI-only behavior). `cargo tree -p cronus-tui` shows `cronus` (core) among workspace crates and **not** `cronus-cli`. `cargo clippy -p cronus-tui --all-targets -- -D warnings` and `cargo fmt -p cronus-tui --check` clean.
- **Status:** Done
- **Changes:** 3 `parity_matrix` tests — every `CATALOG` verb (bar `help`) has a CLI counterpart (no TUI-only behavior); the full CLI verb set is covered (both-direction parity); the crate manifest links `cronus` and never `cronus-cli` (structural INV-2, compile-time `include_str!` of `Cargo.toml`). Parity asserted against a curated `EXPECTED_CLI_VERBS` set because importing `cronus-cli` would itself break the dependency rule under test. `cargo tree -p cronus-tui -e normal` confirms `cronus-tui → cronus`, no `cronus-cli`; clippy/fmt clean.

### [T-7T02] Validation — render-from-state + secret masking

- **Goal:** Prove INV-5 (view-only, render purely from snapshot) and INV-7 (secrets never rendered).
- **Method:** `cargo test -p cronus-tui render_state` — panels are a pure function of the view-model snapshot (same snapshot ⇒ identical frame; no hidden mutable domain state). `cargo test -p cronus-tui mask_secrets` — a snapshot/command output carrying a known secret renders masked.
- **Status:** Done
- **Changes:** Extracted `render_view(area, buf, &ViewModel)` (pure frame render) from `RatatuiRenderer::draw` so the render path is exercisable against an off-screen `Buffer`. 2 `render_state` tests (same view-model ⇒ byte-identical buffer; a changed snapshot ⇒ a changed buffer) + 1 `mask_secrets` test (a dispatched secret never appears in the rendered buffer, renders `***`). Verify: 38 crate tests pass; clippy/fmt clean. **Track T complete — all Phase 7 tasks Done.**

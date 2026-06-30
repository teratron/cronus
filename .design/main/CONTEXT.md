# Project Context

**Generated:** 2026-06-30

## Active Technologies

- Node.js
- Rust

## Core Project Structure

```plaintext
.
├── .agents/
│   ├── rules/
│   ├── skills/
│   └── workflows/
├── .claude/
│   ├── commands/
│   ├── rules/
│   ├── settings.local.json
│   └── skills/
├── .codex/
│   ├── prompts/
│   ├── rules/
│   └── skills/
├── .design/
│   ├── .cache/
│   ├── .graph-cache/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── desktop.drawio.svg
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── release.drawio.svg
│   └── technology-stack-research.md
├── .env
├── .env.example
├── .gitignore
├── .markdownlint.json
├── .release/
│   ├── program/
│   └── state/
├── AGENTS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── apps/
│   └── desktop/
├── biome.json
├── crates/
│   ├── cli/
│   ├── codegraph/
│   ├── core/
│   ├── nodus/
│   └── tui/
├── docs/
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
└── rust-toolchain.toml
```

## Recent Changes


Internal phase journal. Each entry corresponds to a completed phase.

## Phase 7 — Leaf: TUI (2026-06-28)

- T-7A01: `terminal` module — `TerminalBackend` DI trait + `CrosstermBackend` production impl (Windows Press-only key filter) + `Tui<B>` RAII guard with panic-safe, idempotent restore; converted `cronus-tui` to lib+bin; added `crossterm` workspace dependency
- T-7A02: `app` module — `App::tick` pure step-function (input-first → non-blocking on slow snapshot → exactly one redraw), view-only `ViewModel`/`CoreSnapshot`, `SnapshotSource` poll seam + `CapabilitySource`, `Renderer` seam, `run`/`run_with` drivers. Event-seam resolved: core has no pub/sub → poll-snapshot fallback (INV-5 preserved)
- T-7B01: `view` module — `layout()` (2×2 panels above a one-row command bar, clamps at tiny sizes), `Focus` enum + `next`/`prev` tab-cycling, `RatatuiRenderer` (bordered/titled panels + focus highlight); adopted ratatui 0.30 (feature `crossterm_0_29`, single crossterm version); `Tab`/`Shift+Tab` focus routing; `Key::BackTab` added
- T-7B02: Board + Office panels — 7-column `BoardColumn` presentation projection (core models 6 states + archive store; transitions stay in core, INV-2), `BoardView`/`BoardCard`, `OfficeView`/`AgentActivity`, pure `render_board`/`render_office` into an off-screen ratatui `Buffer`
- T-7B03: Status + Sessions panels — `render_status` mirrors the `status` capability (version + status line); `SessionsView` bounded last-N scrollback projection (`MAX_SESSION_LINES` = 500) in `CoreSnapshot`, keeping the view a pure function of the snapshot (INV-5); all four panels render
- T-7C01: `command` module — slash parser, `CATALOG` (help + 21 CLI-mirrored verbs), `classify` → `Help`/`Run`/`Error` (unknown verb → inline error, never panic); interactive command bar (`command_input`/`command_feedback`, focus-aware key routing; Esc cancels the line in the bar, quits elsewhere)
- T-7C02: `Dispatcher` trait + `CapabilityDispatcher` (verb → core capability, then masks output via `cronus::redact::redact` — INV-7, no re-implementation) + `NoopDispatcher`; loop drains a submitted command and renders masked output, closing the input→core→redraw cycle
- T-7T01: `parity_matrix` validation — every slash command maps to a CLI verb (no TUI-only behavior), full CLI verb coverage, and a compile-time manifest check that the crate links `cronus` and never `cronus-cli` (structural INV-2); `cargo tree` confirms `cronus-tui → cronus`
- T-7T02: `render_state` + `mask_secrets` validation — extracted `render_view` (pure frame render); same view-model ⇒ byte-identical buffer (INV-5), a changed snapshot ⇒ a changed buffer, and a dispatched secret never reaches the rendered buffer (INV-7)
- Verify: `cargo test -p cronus-tui` — 38 passed, 0 failed; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean


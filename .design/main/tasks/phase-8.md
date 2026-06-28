---
phase: 8
name: "Flower — Desktop App"
status: In Progress
subsystem: "apps/desktop, packages/ui"
requires:
  - "core capability surface + subsystems (Phases 1, 4–6)"
  - "CLI/TUI command set as parity reference (Phases 3, 7)"
  - "frontend toolchain: pnpm + Tauri v2 CLI (provisioned in T-8A01 — currently MISSING)"
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 8 Tasks — Flower: Desktop App

**Phase:** 8
**Status:** In Progress
**Strategic Goal:** The full graphical surface — a Tauri v2 desktop shell (`apps/desktop`) wrapping a React 19 UI (`packages/ui`) that renders core state and drives the autonomous office. Pure presentation: the UI calls the core over the IPC bridge and holds no business logic (INV-2); all domain logic stays in the core.

> **Architectural guardrails (l2-app-ui §3):** the UI is a pure consumer of the core via the shell↔core IPC bridge (INV-1/INV-2). No `any` on public surfaces; all user-facing strings externalized (i18n); honor the theme/design-token system. Slash/command surfaces mirror the shared capability set (INV-3 parity with CLI/TUI). Secrets are never rendered (INV-7).
>
> **Planner Audit — execution risks (read before `/magic.run`):**
>
> 1. **Toolchain gate (RESOLVED in T-8A01):** pnpm (via corepack) + the Tauri v2 CLI are provisioned and both crates/packages are scaffolded and build green. Note: native C/Tauri builds (`cargo` on the Tauri crate, `windres`, `gcc`) must run via **PowerShell**, not the Bash tool — Git Bash's MSYS2 environment makes mingw64 `cc1.exe` fail to load (this was the false "[C-801] host gcc broken" finding; gcc is fine in PowerShell).
> 2. **Optimism bias:** `l2-app-ui` spans 14 design sections (§4.1–4.14: shell bridge, settings, tray, shortcuts, overlay, single-instance, per-provider prompts, XML env, MCP client). This is a large phase — run it **incrementally**: complete Track A first, then B/C/D can proceed in parallel.
> 3. **Cascade:** Tracks B (Rust/Tauri shell systems), C (React surfaces + views), D (integrations) all sit on the T-8A01 scaffold + the T-8A02 IPC bridge.

## Atomic Checklist

Track A — Scaffold & Bridge (l2-app-ui §2, §4.2) — **gating**

- [x] [T-8A01] Provision frontend toolchain (pnpm + Tauri v2 CLI) + scaffold `apps/desktop` (Tauri v2 shell) and `packages/ui` (React 19 + Vite + TS + Tailwind v4 + shadcn/ui)
- [ ] [T-8A02] Shell ↔ core IPC bridge: typed UI→core command surface over Tauri IPC (presentation-only; mirrors the capability set)

Track B — Shell Systems (Rust/Tauri side, l2-app-ui §4.7–4.11)

- [ ] [T-8B01] Settings persistence (§4.7): load_or_create, dual deserializer, additive migration, platform defaults, AtomicU8 hot settings
- [ ] [T-8B02] Tray icon state machine + global shortcut binding + overlay window + single-instance (§4.8–4.11)

Track C — UI Surfaces & Views (React side, l2-app-ui §4.1/4.5/4.6 + sibling specs)

- [ ] [T-8C01] Surfaces + theming + i18n (§4.1, §4.5, §4.6): render-from-state surfaces, theme tokens, externalized strings
- [ ] [T-8C02] Office View panel (l2-office-view): graph + spatial projection of agents/tasks
- [ ] [T-8C03] Dashboard panel (l2-dashboard): live read-only statistics projection

Track D — Integrations (l2-app-ui §4.12–4.14)

- [ ] [T-8D01] Per-provider system prompt dispatch (§4.12) + XML structured environment context (§4.13)
- [ ] [T-8D02] MCP client transports (Stdio/SSE/StreamableHTTP) + connection status states + OAuth flow (§4.14)

Track T — Validation

- [ ] [T-8T01] Validate presentation-only + dependency direction (UI → core over IPC, no business logic in TS; `fallow audit` clean; tsc no `any` on public surfaces)
- [ ] [T-8T02] Validate store-compliance + theming/i18n behavior (§4.4): single-authority state, render-from-state, localized strings, theme tokens honored

## Detailed Tracking

### [T-8A01] Provision toolchain + scaffold

- **Spec:** l2-app-ui.md §2 (Constraints), §4.1 (Surfaces); l2-technology-stack.md (UI stack)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `pnpm --version` and the Tauri v2 CLI resolve; `pnpm -C packages/ui build` exits 0; `pnpm -C apps/desktop tauri info` succeeds; `cargo check` on the Tauri crate compiles.
- **Handoff:** Provides the buildable `apps/desktop` (Tauri v2) + `packages/ui` (React 19) workspace every other Phase 8 task builds on. **Tauri crate dir is `apps/desktop/tauri` (renamed from `src-tauri`); the Tauri v2 CLI auto-detects it.**
- **Changes:** pnpm 11.9.0 (corepack) + pnpm workspace (`pnpm-workspace.yaml`, root `package.json`, `biome.json` migrated to biome 2.x, scoped to `packages/**`+`apps/**`). `packages/ui` — React 19 + Vite 8 + TS 6 + vitest 4; render-from-state `App` + 2 tests. `apps/desktop` — Vite/React/Tailwind v4 frontend + Tauri v2 `apps/desktop/tauri` (standalone `[workspace]`, valid v2 config, lib/main/build/capabilities, valid PNG-based `icon.ico`). All deps on latest. Verify: `pnpm -r build` exit 0, `pnpm -r test` 2/2, `biome check` clean, `cargo check` on the Tauri crate compiles (5.14s), `pnpm tauri info` OK.
- **[C-801] was a false alarm:** the earlier "host gcc broken" was an artifact of running `cargo`/`windres` through the **Bash tool** (Git Bash's MSYS2 env makes mingw64 `cc1.exe` fail to load, exit 127). The same `gcc.exe` works in **PowerShell**, so native C/Tauri builds run via PowerShell. gcc is fine; no host repair was needed.
- **Notes:** No domain logic — scaffold only. shadcn/ui component setup (`components.json` + first component) deferred to T-8C01 (surfaces); Tailwind v4 + the shadcn-ready stack are in place.

### [T-8A02] Shell ↔ core IPC bridge

- **Spec:** l2-app-ui.md §4.2 (Shell ↔ core bridge), l1-architecture.md (INV-1 embeddable core, INV-2 logic-in-core)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** A round-trip test — the UI invokes a core capability (e.g. `status`) over the Tauri IPC command and renders the returned value; assert the TS side holds no business logic (it only marshals the call). `cargo test` on the Tauri crate for the command registration; `pnpm -C packages/ui test` for the client wrapper.
- **Handoff:** Establishes the typed command channel every UI surface reads/writes through (INV-2 boundary).
- **Notes:** The bridge marshals UI intents to core capability calls and core state back; mirrors the shared capability set the CLI/TUI bind (INV-3). Secrets masked via the core redaction path (INV-7), never re-implemented in TS.

### [T-8B01] Settings persistence

- **Spec:** l2-app-ui.md §4.7 (Settings Persistence System)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` on the Tauri crate — `load_or_create` returns defaults on a missing file and round-trips a saved file; the dual deserializer reads a legacy shape; an additive migration preserves unknown fields; platform defaults resolve per-OS.
- **Handoff:** Provides the durable settings store the shell + UI read (hot settings via AtomicU8).
- **Notes:** Backward-compatible (dual deserializer); additive migration only. Lives in the Tauri/Rust shell, not the React layer.

### [T-8B02] Tray + shortcuts + overlay + single-instance

- **Spec:** l2-app-ui.md §4.8 (Tray), §4.9 (Global shortcuts), §4.10 (Overlay), §4.11 (Single instance)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — the tray state machine resolves the correct icon/menu for each State×Theme (9 variants) with the copy-last-result fallback; a `ShortcutBinding` registers and auto-rolls-back on conflict; overlay position maps per-OS; a second instance hands off to the running one.
- **Handoff:** Completes the desktop-shell affordances around the main window.
- **Notes:** Rust/Tauri side. Dual shortcut backend with auto-rollback; overlay has an OS escape hatch.

### [T-8C01] Surfaces + theming + i18n

- **Spec:** l2-app-ui.md §4.1 (Surfaces), §4.5 (Theming), §4.6 (Localization)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `pnpm -C packages/ui test` (vitest) — surfaces render from injected state (render-from-state, no domain mutation); switching theme swaps tokens; switching locale swaps all visible strings (no hard-coded user-facing text). `biome` lint + `tsc --noEmit` clean.
- **Handoff:** The surface shell that hosts the Office View and Dashboard panels.
- **Notes:** Render-only components fed by core state over the bridge (INV-2). All strings externalized; theme via design tokens.

### [T-8C02] Office View panel

- **Spec:** l2-office-view.md, l1-office-visualization.md
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `pnpm -C packages/ui test` — given an office projection, the panel renders the agent graph + spatial floor; a state change in the next projection re-renders. biome + tsc clean.
- **Handoff:** Reads the office projection from the IPC bridge view-model.
- **Notes:** Graph + spatial projection; presentation only — no office mutation in TS.

### [T-8C03] Dashboard panel

- **Spec:** l2-dashboard.md, l1-dashboard.md
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `pnpm -C packages/ui test` — the dashboard renders per-office + building-aggregate statistics from a projection; updates on a new projection. biome + tsc clean.
- **Handoff:** Completes the read-only graphical surface ahead of integrations.
- **Notes:** Live read-only statistics projection (mirrors the `dashboard` capability).

### [T-8D01] Per-provider prompt dispatch + XML env context

- **Spec:** l2-app-ui.md §4.12 (Per-provider system prompt dispatch), §4.13 (XML structured environment context)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` / `pnpm -C packages/ui test` — a provider key (anthropic/gpt/gemini/…) selects the matching system-prompt variant; the `env` + `available_references` XML blocks are assembled with the expected structure.
- **Handoff:** Feeds provider-correct prompts + environment context to the core session surface.
- **Notes:** Provider-keyed variants; XML blocks are structural — assert shape, not model output.

### [T-8D02] MCP client transports + status + OAuth

- **Spec:** l2-app-ui.md §4.14 (MCP client connection and status)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — the transport variants (Stdio/SSE/StreamableHTTP) construct with `DEFAULT_TIMEOUT=30_000ms`; the connection-status state machine moves through connected/disabled/failed/needs_auth/needs_client_registration; the OAuth flow tracks a pending-transport map.
- **Handoff:** Connects the desktop app to external MCP servers.
- **Notes:** Roots capability declared at connect; OAuth pending-transport map keyed per connection.

### [T-8T01] Validation — presentation-only + dependency direction

- **Goal:** Prove INV-2 (no business logic in the UI; UI → core inward) structurally.
- **Method:** `fallow audit --changed-since <base>` — no new dead code, duplication, circular deps, or architecture-boundary violations (presentation-only UI, inward-pointing deps). `tsc --noEmit` shows no `any` on public surfaces. The React layer contains no domain logic — behavior delegates to the core over the IPC bridge.
- **Status:** Todo

### [T-8T02] Validation — store-compliance + theming/i18n

- **Goal:** Prove the store-compliance notes (l2-app-ui §4.4) and INV-7/i18n/theming behavior.
- **Method:** `pnpm -C packages/ui test` — single-authority state with render-from-state components (same state ⇒ same render); a known secret in a projection renders masked (INV-7); every visible string resolves through i18n (no hard-coded user text); theme tokens are honored (no literal colors on themed surfaces).
- **Status:** Todo

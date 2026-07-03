# Navigation

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-navigation-model.md

## Overview

The concrete rendering of the four-layer building navigation in the Tauri v2 + React 19 desktop shell: the Building frame (L0 menu + Providers/ACP + Process Monitor), the floor tab bar (L1) with lazy loading and live OfficeState icons, the canonical subsystem sidebar (L2), recursive mechanism sub-navigation (L3+), the two-tier settings surface, and the "Open in IDE" launcher. Navigation is presentation over core state: the sidebar catalog is a fixed frontend constant, but floor load/unload, office state, and settings persistence are core calls (INV-2).

## Related Specifications

- [l1-navigation-model.md](l1-navigation-model.md) — the model this renders (NV-1…NV-10).
- [l2-app-ui.md](l2-app-ui.md) — the frontend runtime shape (state authority, workbench) this composes onto.
- [l2-office-control.md](l2-office-control.md) — the OfficeState source for live tab status icons (NV-3).
- [l2-workspace-management.md](l2-workspace-management.md) — floor creation/deletion and `workspace_root` binding (NV-8).
- [l2-config-hotreload.md](l2-config-hotreload.md) — two-tier settings persistence and live reload (NV-4).
- [l1-process-monitor.md](l1-process-monitor.md) — the Building-level process view (NV-7).

## 1. Motivation

The model guarantees a consistent, memorable structure across platforms. Realizing it as a fixed frontend catalog plus core-backed floor/state calls keeps the navigation identical on every build while the live data (which floors exist, their state, their settings) stays single-sourced in the core.

## 2. Constraints & Assumptions

- The sidebar catalog and its order are a frontend constant; users may pin shortcuts above it but cannot reorder/hide the canonical set (NV-1).
- Floor lazy loading calls the core `office.is_loaded` / `office.load` / `office.unload` capability; the home floor is never unloaded.
- Status icons subscribe to the OfficeState event stream — no polling (NV-3).
- Local settings files are `.gitignore`d by default (machine-specific paths/keys).

## 3. Invariant Compliance (Layer 2)

| L1 Invariant | Implementation |
| --- | --- |
| NV-1 Canonical sidebar order | A frozen `SIDEBAR_TABS` array (Chat…Settings) renders in fixed order; pinned shortcuts render in a separate strip above it, never mutating the canonical array. |
| NV-2 Office tab lazy loading | On start the shell loads the home floor + most-recently-active project floor via `office.load`; other tabs render a placeholder until activated. A closed tab with no running tasks calls `office.unload`. |
| NV-3 Live status indicator | Each floor tab subscribes to `OfficeStateChanged` (office-control §4.1); the icon re-renders on each event, never from a snapshot older than one cycle. |
| NV-4 Two-tier settings | The Settings tab renders a Global section (app config) and a Local section (active office config) with a tier label; each writes through `config.set(scope, …)` and reloads via config-hotreload. |
| NV-5 IDE integration | The floor settings dropdown's "Open in IDE" reads `workspace_root` (local) + `configured_ide` (global) and calls the Tauri `shell_spawn` command `{ide} {root}`. |
| NV-6 Strict layer nesting | The React tree mirrors Building ⊃ Floor ⊃ Subsystem ⊃ Mechanism; each level's router is scoped to one parent instance and cannot address a sibling's subtree. |
| NV-7 Building frame & app menu | The L0 frame hosts File/Edit/View/Help (Tauri native menu) plus Providers/ACP and Process Monitor panels; these act across all floors. |
| NV-8 Floor = disk-bound tab | Floor creation resolves through three affordances (File menu, "+" control, folder drag-drop) all calling `workspace.create`; a project floor binds `workspace_root` at creation, stable for its life. |
| NV-9 Default pinned home floor | The first tab is pinned, non-closable, loaded on start; its dropdown omits close/delete; its files resolve to the state tier, not a disk project. |
| NV-10 Recursive sub-navigation | A subsystem with facets renders an L3 sub-tab strip (e.g. Schedule → Cron/Pulse, Inbox → Messages/Poll-Clarify) scoped to that subsystem; a flat subsystem renders none. |

## 4. Detailed Design

### 4.1 Component tree

```text
[REFERENCE]
<BuildingFrame>                       // L0 — native menu + Providers/ACP + Process Monitor
  <FloorTabBar>                       // L1 — pinned Home + project floors, "+" + drop target
    <Floor office_id>                 // one active floor
      <SubsystemSidebar>              // L2 — frozen SIDEBAR_TABS
        <Subsystem tab>               // active subsystem surface
          <MechanismNav?>             // L3+ — sub-tabs, only where earned (NV-10)
```

State authority is the app-shell store (AS single-authority); navigation components read floor/state/settings from it and dispatch load/unload/create actions.

### 4.2 Floor tab bar

Each entry renders name, live OfficeState icon (NV-3), and a settings dropdown (rename, open-in-IDE, pause/resume via office-control, close, delete). The pinned home floor omits close/delete. The trailing "+" and a full-bar drop target both invoke `workspace.create`; a dropped folder pre-fills `workspace_root`.

### 4.3 Sidebar catalog

`SIDEBAR_TABS = [Chat, Inbox, Channels, Sessions, Schedule, Pulse, Memory, Office, Kanban, Security, Wiki, Settings]` — frozen order, badge counts from per-subsystem live signals. UX-stage candidates (Discover/Graph/Process Monitor) slot in additively without reordering.

### 4.4 Two-tier settings

Global settings persist to the app global config file; local settings travel with the office workspace config. Both render in the Settings tab under a tier label; writes go through the config service so config-hotreload applies them live. Local files carry machine-specific paths/keys and are excluded from the office git repo.

### 4.5 IDE launch

`open_in_ide(office)` → read `workspace_root` (local) + `configured_ide` (global, default `$EDITOR` → platform default VS Code) → Tauri `shell_spawn`. The editor is external; the app is launcher-only.

## 5. Implementation Notes

1. Lazy loading (NV-2) requires the core `office.is_loaded`/`load`/`unload` capability; the shell holds no office state itself.
2. Status icons (NV-3) subscribe to the office-control event stream through the IPC bridge, not a poll timer.
3. NV-7 per-menu leaf contents and the L3 per-subsystem facet catalog remain UX-stage TBDs carried from the L1; this spec fixes the structure, not the final leaf lists.

## 6. Drawbacks & Alternatives

**Alternative — user-customizable sidebar**: fragments the cross-platform mental model (NV-1). Rejected.

**Alternative — floor picker dialog instead of a tab bar**: loses glanceable live status across all floors (NV-3). Rejected.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-navigation-model.md` | Invariants NV-1…NV-10 |
| `[APP-UI]` | `.design/main/specifications/l2-app-ui.md` | Rendering host / state authority |
| `[OFFICE-CTRL]` | `.design/main/specifications/l2-office-control.md` | OfficeState source for NV-3 icons |
| `[WS-MGMT]` | `.design/main/specifications/l2-workspace-management.md` | Floor creation + workspace_root |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-03 | Core Team | Initial implementation spec — four-layer component tree, floor tab bar with lazy load + live OfficeState icons, frozen sidebar catalog, recursive mechanism sub-nav, two-tier settings, IDE launcher; maps NV-1…NV-10. |

# Navigation Model

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

The navigation model defines the two primary navigation surfaces of the application: a vertical sidebar of function-labelled tabs (per-office subsystem access) and a horizontal tab bar for switching between offices (projects). Every primary capability of the office is reachable through one of these two surfaces; no capability is buried more than one click deep from the navigation.

The model also defines the two-tier settings hierarchy (global application settings vs per-office local settings) and the IDE integration entry point accessible from the office tab bar.

## Related Specifications

- [l1-office-model.md](l1-office-model.md) — the office concept the navigation surfaces expose
- [l1-office-control.md](l1-office-control.md) — OfficeState taxonomy driving the office tab status icons
- [l1-project-wiki.md](l1-project-wiki.md) — client-facing living project documentation surfaced through the Wiki tab
- [l2-workspace-management.md](l2-workspace-management.md) — office creation, naming, and lifecycle
- [l2-app-ui.md](l2-app-ui.md) — rendering implementation of this model

## 1. Motivation

A multi-subsystem application with 10+ distinct capabilities needs a consistent, memorable navigation structure. Without a formal model, tab placement drifts arbitrarily across UI iterations and different platform builds. The navigation model guarantees that any user, on any platform, sees the same canonical structure and can locate any capability without exploration.

## 2. Constraints & Assumptions

- The sidebar is scoped to the currently selected office; every tab shows data for that office only.
- The office tab bar is scoped to the whole application (building); it lists all known offices.
- Navigation is read-only with respect to system state — mutations happen within each subsystem surface, not from navigation controls themselves.
- The navigation model is platform-neutral (applies equally to desktop, web, and mobile); platform-specific adaptations are L2 concerns.

## 3. Core Invariants

- **NV-1 Canonical sidebar order**: the sidebar presents exactly the defined set of primary tabs in a fixed order. No tab may be hidden, reordered, or removed at the application level. A user or office may PIN additional shortcut tabs above the canonical set; the canonical set remains intact below.
- **NV-2 Office tab lazy loading**: an office tab is loaded into memory only when the user activates it, or when it holds an actively running task that requires monitoring. An inactive office with no running tasks MUST NOT consume foreground resources.
- **NV-3 Live status indicator**: each office tab displays a status icon drawn from the OfficeState taxonomy (from `l1-office-control.md`). The icon MUST reflect live engine state, not a cached snapshot older than one event cycle.
- **NV-4 Two-tier settings**: the Settings tab exposes global settings (affect the whole application) and local settings (affect only the active office). Both tiers are reachable from the same tab; they are visually separated with a clear tier label.
- **NV-5 IDE integration**: every office exposes an "Open in IDE" action reachable from its tab's settings dropdown. The action shell-spawns the user's configured editor against the office's workspace root path.

## 4. Detailed Design

### 4.1 Sidebar Tab Catalog

Fixed order; badge shows live pending-item count where applicable.

| # | Tab | Subsystem | Badge |
| --- | --- | --- | --- |
| 1 | Chat | Conversation with the active office orchestrator | Unread messages |
| 2 | Notifications / Inbox | Incoming events: messages, alerts, approval requests | Unread count |
| 3 | Channels | Persistent topic threads; deliberation logs; inter-role communication | Active threads |
| 4 | Sessions | Current and historical agent sessions | Running sessions |
| 5 | Schedule | Recurring jobs, one-shot schedules, cron entries | Jobs due today |
| 6 | Pulse | Heartbeat activity: background routine and inner-monologue log | Active pulses |
| 7 | Memory | Office memory store: facts, skills, knowledge items | — |
| 8 | Office | Automation canvas + agent interaction graph | — |
| 9 | Kanban | Work board: triage → todo → ready → running → blocked → done | Active + blocked cards |
| 10 | Security | Sandbox policies, secret vault status, audit log | Policy alerts |
| 11 | Wiki | Client-facing living project documentation: overview, areas, decisions, how-to, changelog (read-only, office-maintained) | Updated since last visit |
| 12 | Settings | Two-tier configuration: global + per-office | — |

### 4.2 Office Tab Bar

The horizontal tab bar lists all offices in the building. Each entry:

- **Office name** — display label (from workspace config, kebab-case slug underneath)
- **Status icon** — live OfficeState: Active / Idle / Paused / Hibernating / Error / Offline
- **Settings dropdown** — per-office quick actions: rename, open in IDE, pause/resume, close tab, delete office

**Lazy loading (NV-2)**: on application start, only the most recently active office is loaded. Switching activates the target office on demand. An office is eligible for unloading when its tab is closed and it has no running background tasks.

### 4.3 Settings Two-Tier Structure

```text
[REFERENCE]
Global Settings (affect the whole application — all offices)
  ├── Appearance: theme (system/light/dark), language, density, configured IDE
  ├── Models: default provider, global cost limits, API key management
  ├── Security: sandbox defaults, secret storage backend, audit retention
  ├── Notifications: delivery targets, quiet hours, escalation rules
  └── Updates: release channel, update policy

Local Settings (affect only the active office)
  ├── Office identity: name, description, icon, workspace root path
  ├── Model overrides: per-role model assignment
  ├── Automation: office-level automation rules and pipeline toggles
  ├── Git: repository path, default remote, branch strategy declaration
  └── Integrations: office-specific MCP server endpoints
```

Global settings are stored in the application's global config file. Local settings travel with the office workspace (stored in the office's local config), enabling portability.

### 4.4 IDE Integration

The "Open in IDE" action (NV-5):
1. Reads the office's `workspace_root` path from Local Settings
2. Reads the `configured_ide` command from Global Settings → Appearance
3. Shell-spawns: `{configured_ide} {workspace_root}`

The editor is not embedded; the application is the launcher only. The configured IDE is any program that accepts a path as its first CLI argument. Default: the system's `$EDITOR` variable, falling back to a platform-appropriate default (VS Code on desktop).

## 5. Implementation Notes

1. Office tab lazy loading (NV-2) requires the engine to expose a per-office `is_loaded` predicate and a `load`/`unload` command.
2. Status icons (NV-3) subscribe to the OfficeState event bus from `l1-office-control.md`; no polling.
3. Local settings files must be excluded from the office's git repository by default (`.gitignore` entry) — they may contain local paths and model keys that are machine-specific.

## 6. Drawbacks & Alternatives

**Alternative: fully customizable sidebar** — let users reorder, hide, or rename any tab. Rejected: NV-1 ensures consistency across users, platforms, and documentation; customization fragments the mental model.

**Alternative: no office tab bar, use a list or picker dialog** — a dropdown or modal replaces the tab bar. Loses the glanceability of NV-3 live status icons across all offices simultaneously.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[OFFICE-CTRL]` | `.design/main/specifications/l1-office-control.md` | OfficeState taxonomy for NV-3 status icons |
| `[WORKSPACE-MGMT]` | `.design/main/specifications/l2-workspace-management.md` | Office naming and root path |
| `[APP-UI]` | `.design/main/specifications/l2-app-ui.md` | Rendering host |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — NV-1…NV-5, 11-tab catalog, office tab bar, two-tier settings, IDE integration |
| 1.1.0 | 2026-06-26 | Core Team | Added the Wiki tab (now #11, Settings → #12) — client-facing living project documentation surface (see l1-project-wiki.md). Additive extension of the canonical set; NV-1 unchanged. |

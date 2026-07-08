# Filesystem Layout (OS-native)

**Version:** 1.1.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-storage-model.md

## Overview

The concrete on-disk realization of the storage model: OS-native install locations for the two tiers, the directory trees of the program and state tiers, the placement of databases (SQLite + sqlite-vec), and the mapping of memory levels to paths. Visualization stubs of both trees live in the repository's `.release/` sandbox.

## Related Specifications

- [l1-storage-model.md](l1-storage-model.md) - The model this layout implements.
- [l2-technology-stack.md](l2-technology-stack.md) - SQLite + sqlite-vec; optional libSQL/PostgreSQL sync.
- [l2-core-library.md](l2-core-library.md) - The core resolves these paths and owns persistence.
- [l2-skill-system.md](l2-skill-system.md) - The two-tier skill stores rooted at `<program>/skills/` and `<state>/skills/`.

## 1. Motivation

The model demands two separated tiers and scoped memory; this spec pins exactly where they live per OS and how the directories are shaped, so implementation and packaging are unambiguous.

## 2. Constraints & Assumptions

- Default deployment is **OS-native** (tiers in their conventional locations); a portable mode (both under one directory) is also supported.
- Cache and logs are placed in OS-specific cache/state locations, outside the main state tier.
- Databases are created at runtime; the repository ships only empty stubs and READMEs.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| STO-1 Two-tier separation | Program tier under the OS program location; state tier under the OS app-data location — distinct roots. |
| STO-2 Durable, restartable state | State tier holds SQLite files + text; runtime rehydrates from them on launch. |
| STO-3 Catalog vs instance | `<program>/employees/` (catalog) and `<program>/templates/` are read-only; hiring/init copies into `<state>/employees/` and `<state>/workspaces/`. |
| STO-4 Multi-level memory | Paths per level: global `<state>/memory/`; workspace `<state>/workspaces/<ws>/memory/`+`graph/`; employee `<state>/employees/<role>/memory/`; session `<state>/workspaces/<ws>/sessions/`. |
| STO-5 Scope-bound lifecycle | Deleting an office/role directory removes its memory; sessions pruned in place; global persists. |
| STO-6 Secret isolation | Secrets in `<state>/.env` (template `.env.example`); excluded from backups and version control. |
| STO-7 Restore-by-copy | Copying `<state>/` minus `.env` and cache restores the system. |
| STO-8 Human-inspectable state | Config as JSON, rules/notes/STATE as Markdown; `*.db` are derived indices alongside `notes/`. |

## 4. Detailed Design

### 4.1 OS-native locations

| Tier | Windows | macOS | Linux (XDG) |
| --- | --- | --- | --- |
| Program (immutable) | `%ProgramFiles%\Cronus\` | `/Applications/Cronus.app/Contents/Resources/` | `/opt/cronus/` or `/usr/local/lib/cronus/` |
| State (mutable) | `%APPDATA%\Cronus\` | `~/Library/Application Support/Cronus/` | `~/.local/share/cronus/` |
| Cache (regenerable) | `%LOCALAPPDATA%\Cronus\Cache\` | `~/Library/Caches/Cronus/` | `~/.cache/cronus/` |
| Logs / runtime | `%LOCALAPPDATA%\Cronus\Logs\` | `~/Library/Logs/Cronus/` | `~/.local/state/cronus/` |

A single path resolver in the core maps the abstract roots (`<program>`, `<state>`, `<cache>`, `<logs>`) to these per-OS paths; portable mode overrides them to one chosen directory.

### 4.2 Program tier tree

```plaintext
<program>/
├── bin/            # cronus (CLI), cronus-tui (TUI), cronusd (always-on service)
├── app/            # core engine library + desktop application shell
├── templates/      # employee/ , workspace/ (blueprints copied on init)
├── employees/      # read-only role catalog (CATALOG.md + role blueprints)
├── skills/         # [ADDED] read-only preset skill store (canonical packages)
├── languages/  themes/
└── VERSION
```

### 4.3 State tier tree

```plaintext
<state>/
├── .env                  # secrets (excluded from backup/VCS)
├── app.json  config.json  auth.json  channels.json  models.json  routing.json  gateway.json
├── AGENTS.md
├── memory/               # GLOBAL: global.db (SQLite+vec), graph.db, notes/
├── skills/               # [MODIFIED] mutable skill store: user-added + generated (canonical packages)
├── employees/<role>/     # EMPLOYEE: config.json, RULES.md, memory/, skills/, skins/
└── workspaces/<ws>/      # WORKSPACE (office)
    ├── config.json  RULES.md  STATE.md
    ├── memory/           #   workspace.db (SQLite+vec) + notes
    ├── graph/            #   graph.db (people/tasks/decisions/artifacts)
    ├── sessions/         #   SESSION (episodic, pruned)
    ├── kanban/  office/  schedules/  hooks/  sandboxes/  snapshots/  dashboard/
```

### 4.4 Database placement (SQLite + sqlite-vec)

| Level | File | Engine |
| --- | --- | --- |
| Global | `<state>/memory/global.db`, `<state>/memory/graph.db` | SQLite + sqlite-vec |
| Workspace | `<state>/workspaces/<ws>/memory/workspace.db`, `<ws>/graph/graph.db` | SQLite + sqlite-vec |
| Employee | `<state>/employees/<role>/memory/employee.db` | SQLite + sqlite-vec |

Physical consolidation (one file with attached schemas vs separate files per level) is an implementation choice deferred to the build phase. Optional remote sync targets libSQL/PostgreSQL; local files remain the source of truth.

### 4.5 Repository visualization stub

```plaintext
.release/
├── program/   # stub of the immutable program tier
└── state/     # stub of the mutable state tier (example office: workspaces/default)
```

`.release/` is a temporary sandbox for discussion/visualization; it is not a build artifact.

## 5. Drawbacks & Alternatives

- **Per-OS path variance:** four location classes per OS add packaging complexity; mitigated by the central path resolver.
- **Strict XDG split (config/data/cache/state in four roots) vs consolidated state:** this spec consolidates mutable state under the data root for a simpler mental model, splitting only cache/logs. Strict XDG separation remains an option. <!-- TBD: confirm consolidated-state vs strict-XDG for Linux v0.1.0 -->
- **Alternative — portable-only:** simpler paths but poor OS integration; rejected as default, kept as a mode.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-storage-model.md` | Invariants this layout satisfies |
| `[STACK]` | `.design/main/specifications/l2-technology-stack.md` | Storage engine choices |
| `[STUB]` | `.release/` | On-disk visualization of both tiers |

## Document History

| Version | Date | Notes |
| --- | --- | --- |
| 1.0.0 | 2026-06-24 | Initial stable spec — OS-native tier locations, program/state trees, database placement, repository visualization stub. |
| 1.1.0 | 2026-07-08 | `[ADDED]` `<program>/skills/` (read-only preset skill store) to the program tier tree; `[MODIFIED]` `<state>/skills/` comment to reflect the mutable skill store (user-added + generated canonical packages); Related Specifications link to the skill system spec. Additive — status remains Stable. |

# Role Catalog

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-roles.md

## Overview

The concrete realization of roles: the read-only preset catalog in the program tier, hired instances in the state tier, the role definition format, how custom roles are created, the hire/release mechanics, and the `role` command surface.

## Related Specifications

- [l1-roles.md](l1-roles.md) - The role model this implements.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - Catalog (`program/employees/`) and instances (`state/employees/`).
- [l2-orchestration.md](l2-orchestration.md) - The manager hires/releases and delegates to roles.
- [l2-cli.md](l2-cli.md) - Command grammar standard the `role` commands follow.

## 1. Motivation

The model needs a concrete place for blueprints versus instances and a uniform definition format so presets and customs are handled identically. File-backed roles keep them inspectable and instances isolated.

## 2. Constraints & Assumptions

- Presets are read-only and shipped under the program tier.
- A hired instance is a mutable copy under the state tier with its own memory.
- Custom roles use the same on-disk shape as presets.
- The frontend renders; hire/fire and definition are core calls (INV-2).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| ROL-1 Role = specialty | Each catalog entry is a role definition; a hired entry is an agent instance. |
| ROL-2 Preset + custom | Presets in `program/employees/`; customs created under `state/employees/` with `hired_from: custom`. |
| ROL-3 Hire = instantiate | Hiring copies the blueprint to `state/employees/<id>/` with its own `memory/`, `skills/`. |
| ROL-4 Fire = release | Releasing moves the instance's memory to an archive and removes it from the active roster; re-hire restores. |
| ROL-5 Manager-driven | `role hire/fire` are issued by the manager; no client action required. |
| ROL-6 Composition | Each role: `config.json` + `RULES.md` + `skills/` + `skins/`. |
| ROL-7 Catalog integrity | Presets are not edited; "customize a preset" writes a new custom under state (`hired_from: <preset>`). |
| ROL-8 Hierarchy placement | A hired instance's `config.json` records `reportsTo`. |

## 4. Detailed Design

### 4.1 Catalog vs instances

```plaintext
<program>/employees/            # read-only PRESET catalog
├── CATALOG.md
└── <role>/                     # blueprint (persona, default config, seed skills)
<state>/employees/              # HIRED instances (mutable)
└── <role-or-custom-id>/
    ├── config.json             # model, budget, reportsTo, hired_from
    ├── RULES.md                # persona
    ├── memory/  skills/  skins/
```

### 4.2 Preset roles (v0.1.0)

Engineering: architect, backend-engineer, frontend-engineer, api-designer, sql-expert.
Quality: code-reviewer, test-writer, debugger, refactorer, performance-optimizer, security-auditor, accessibility-auditor.
Ops & docs: devops-engineer, incident-responder, doc-writer, data-analyst, prompt-engineer.
Memory: archivist.
Business: finance, hr, marketing, support, game-dev.

### 4.3 Custom role creation

A custom role is created under `state/employees/<id>/` with the same shape; `config.json` records `hired_from: custom` (or `hired_from: <preset>` when derived from a preset). Presets are never mutated (ROL-7).

### 4.4 Hire / fire mechanics

```mermaid
graph TD
    HIRE[role hire preset|custom] --> COPY[instantiate into state/employees/id]
    COPY --> PLACE[set reportsTo in office hierarchy]
    FIRE[role fire id] --> ARCHM[archive role memory]
    ARCHM --> REMOVE[remove from active roster]
```

### 4.5 Command surface

Role operations conform to the CLI grammar standard (see `l2-cli.md` §4.4).

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| list (catalog + hired) | `cronus role list [--catalog\|--hired]` | `/role list …` | `roles.list({scope?}) -> Role[]` |
| hire | `cronus role hire <preset> [--as <name>]` | `/role hire <preset> …` | `roles.hire(preset, opts) -> Role` |
| create custom | `cronus role create <name> [--from <preset>]` | `/role create <name> …` | `roles.create(name, opts) -> Role` |
| show | `cronus role show <id>` | `/role show <id>` | `roles.get(id) -> Role` |
| fire | `cronus role fire <id>` | `/role fire <id>` | `roles.fire(id) -> void` |

## 5. Drawbacks & Alternatives

- **Copy-on-hire duplication:** each instance copies the blueprint; acceptable for isolation and per-instance learning.
- **Preset upgrades vs live instances:** when a preset updates with the program, existing instances do not auto-migrate. <!-- TBD: policy for propagating preset updates to already-hired instances -->
- **Alternative — shared role definitions (no copy):** rejected; it couples instances and breaks per-role memory isolation.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ROLES]` | `.design/main/specifications/l1-roles.md` | Invariants this catalog satisfies |
| `[LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | Catalog and instance locations |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

# Dashboard

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-dashboard.md

## Overview

The concrete dashboard: which metrics it computes, the state it reads them from, the per-office and home building-aggregate views, where view layout persists, and the `dashboard` command.

## Related Specifications

- [l1-dashboard.md](l1-dashboard.md) - The model this implements.
- [l2-kanban-board.md](l2-kanban-board.md) - Board state for work metrics.
- [l2-app-ui.md](l2-app-ui.md) - The Dashboard surface in the app shell.
- [l2-cli.md](l2-cli.md) - Command grammar standard.

## 1. Motivation

The model needs concrete metric definitions and sources so the dashboard is a faithful, live projection that costs nothing to keep consistent.

## 2. Constraints & Assumptions

- Metrics are computed on read from existing state; only cosmetic layout is stored under `<ws>/dashboard/`.
- The home aggregate reads across `<state>/workspaces/*` read-only.
- The frontend renders; metric computation is a core call (INV-2).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| DSH-1 Projection | Metrics computed from board, sessions, cost ledger, schedules, memory; nothing authoritative stored. |
| DSH-2 Live | Recomputed on state-change events / refresh. |
| DSH-3 Per-office + building | `dashboard show` for the active office; in home, an aggregate across all offices. |
| DSH-4 Observational | Read-only; no office operations from the dashboard. |
| DSH-5 Privacy | Metrics local; sharing only via telemetry opt-in. |
| DSH-6 Isolation | Office dashboard reads only its workspace; building aggregate reads across offices read-only. |

## 4. Detailed Design

### 4.1 Metrics and sources

| Metric | Source |
| --- | --- |
| cards by state, throughput, cycle time, blocked | `<ws>/kanban/` |
| active agents, running tasks, recent sessions | roster + `<ws>/sessions/` |
| cost/budget usage | cost ledger (per office/agent budgets) |
| schedule status (upcoming/overdue, heartbeat) | `<ws>/schedules/` |
| memory size by scope, recent learnings | memory stores |

### 4.2 Layout storage

```plaintext
<ws>/dashboard/
└── layout.json   # cosmetic widget arrangement; presentation only
```

### 4.3 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| show office dashboard | `cronus dashboard` | `/dashboard` | `dashboard.show() -> Dashboard` |
| building aggregate (home) | `cronus dashboard building` | `/dashboard building` | `dashboard.building() -> Dashboard` |

## 5. Drawbacks & Alternatives

- **Recompute cost on busy offices:** mitigated by event-driven incremental metric updates.
- **Alternative — persist computed metrics:** rejected; storing derived numbers risks drift (DSH-1). Only layout persists.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DASHBOARD]` | `.design/main/specifications/l1-dashboard.md` | Invariants this implements |
| `[BOARD]` | `.design/main/specifications/l2-kanban-board.md` | Work-metric source |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

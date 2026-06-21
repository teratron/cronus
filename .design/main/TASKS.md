# Master Task Index (Registry)

**Version:** 1.4.0
**Generated:** 2026-06-21
**Based on:** .design/main/PLAN.md v2.3.0
**Based on RULES:** .design/RULES.md v1.4.0
**Execution Mode:** Parallel
**Status:** Active

## Overview

Tactical registry of all phases and their statuses, ordered by the growth metaphor (seed → stem → leaf → flower). Atomic checklists live in `tasks/phase-{N}.md`. Phases 1–2 are decomposed into atomic tasks; later phases are decomposed on entry.

## Active Phases

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 1](tasks/phase-1.md) | Seed I — Foundation: workspaces, filesystem, core skeleton, security | `In Progress` |
| [Phase 2](tasks/phase-2.md) | Seed II — Workflow runtime (`crates/nodus`) port | `Done` |
| [Phase 3](archives/tasks/phase-3.md) | Stem — CLI (command framework + grammar + core bindings) | `Done (Archived)` |
| Phase 4 | Core Subsystems: memory store, model/context routers, workspace management | `Pending` |
| Phase 5 | Office Work Engine: role catalog, kanban board, scheduler, quality pipeline, extension registry | `Pending` |
| Phase 6 | Orchestration & Autonomy: delegation, /goal+judge+budget, briefings | `Pending` |
| Phase 7 | Leaf — TUI (interactive terminal) | `Pending` |
| Phase 8 | Flower — Desktop App: application UI, office view, dashboard | `Pending` |
| Phase 9 | Operational Hardening: doctor, backup, error reporting, telemetry | `Pending` |

## Meta Information

- **Last Updated**: 2026-06-21
- **Maintainer**: Core Team

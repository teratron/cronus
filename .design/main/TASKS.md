# Master Task Index (Registry)

**Version:** 1.14.0
**Generated:** 2026-07-03
**Based on:** .design/main/PLAN.md v2.10.0
**Based on RULES:** .design/RULES.md v1.5.0
**Execution Mode:** Parallel
**Status:** Active

## Overview

Tactical registry of all phases and their statuses, ordered by the growth metaphor (seed → stem → leaf → flower). Atomic checklists live in `tasks/phase-{N}.md`. Phases 2-9 are Done; Phase 10 is next and is decomposed on entry, like all later phases.

## Active Phases

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 1](tasks/phase-1.md) | Seed I — Foundation: workspaces, filesystem, core skeleton, security | `Done` |
| [Phase 2](archives/tasks/phase-2.md) | Seed II — Workflow runtime (`crates/nodus`) port | `Done (Archived)` |
| [Phase 3](archives/tasks/phase-3.md) | Stem — CLI (command framework + grammar + core bindings) | `Done (Archived)` |
| [Phase 4](archives/tasks/phase-4.md) | Core Subsystems: memory store, model/context routers, workspace management | `Done (Archived)` |
| [Phase 5](archives/tasks/phase-5.md) | Office Work Engine: role catalog, kanban board, scheduler, quality pipeline, extension registry | `Done (Archived)` |
| [Phase 6](archives/tasks/phase-6.md) | Orchestration & Autonomy: delegation, /goal+judge+budget, briefings | `Done (Archived)` |
| [Phase 7](archives/tasks/phase-7.md) | Leaf — TUI (interactive terminal) | `Done (Archived)` |
| [Phase 8](archives/tasks/phase-8.md) | Flower — Desktop App: application UI, office view, dashboard | `Done (Archived)` |
| [Phase 9](archives/tasks/phase-9.md) | Operational Hardening: sandbox policy, multi-user auth, doctor, config hot-reload, backup, agent migration, GitHub issue reporting, self-improvement, telemetry | `Done (Archived)` |
| Phase 10 | Advanced Office Features (L2): automation engine, canvas, office control, navigation, voice input, deliberation, version control, inner monologue, lookahead planning, ACP, global orchestration — L2 specs pending authoring; + kanban KAN-8 custom-boards delta (spec already Stable at 1.1.0) | `Pending` |
| Phase 11 | Content, Sharing & Dev-Workflow Subsystems: resource-sharing, notes, file-store, development-workflow (L2 specs Stable) — decomposed on entry | `Pending` |

## Meta Information

- **Last Updated**: 2026-07-03 (Registry re-sync to INDEX v1.0.62 / PLAN v2.10.0: 3 orphaned Stable L1 concepts absorbed into Phase 0 (specialty-exemplars, project-priority, project-support); kanban 1.1.0 KAN-8 custom-boards delta tracked as a Phase 10 item. Prior: Phase 9 — Operational Hardening complete: all 10 tasks across 5 tracks done, workspace-wide gates green, archived to archives/tasks/phase-9.md)
- **Maintainer**: Core Team

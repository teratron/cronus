# Master Task Index (Registry)

**Version:** 1.13.0
**Generated:** 2026-07-02
**Based on:** .design/main/PLAN.md v2.9.0
**Based on RULES:** .design/RULES.md v1.5.0
**Execution Mode:** Parallel
**Status:** Active

## Overview

Tactical registry of all phases and their statuses, ordered by the growth metaphor (seed → stem → leaf → flower). Atomic checklists live in `tasks/phase-{N}.md`. Phases 2-8 are Done (Archived); Phase 9 (Operational Hardening) is next and is decomposed on entry, like all later phases.

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
| [Phase 9](tasks/phase-9.md) | Operational Hardening: sandbox policy, multi-user auth, doctor, config hot-reload, backup, agent migration, GitHub issue reporting, self-improvement, telemetry | `Todo` |
| Phase 10 | Advanced Office Features (L2): automation engine, canvas, office control, navigation, voice input, deliberation, version control, inner monologue, lookahead planning, ACP, global orchestration — L2 specs pending authoring | `Pending` |
| Phase 11 | Content, Sharing & Dev-Workflow Subsystems: resource-sharing, notes, file-store, development-workflow (L2 specs Stable) — decomposed on entry | `Pending` |

## Meta Information

- **Last Updated**: 2026-07-03 (Phase 9 — Operational Hardening decomposed: 9 Stable specs -> 10 tasks / 5 tracks in tasks/phase-9.md; Tracks A-D parallel with intra-track serialization only; shared CLI wiring flagged as the cross-track serialization point; T-9D02 flagged for run-time ID-splitting if it overruns. Prior: registry re-sync to INDEX v1.0.61 / PLAN v2.9.0)
- **Maintainer**: Core Team

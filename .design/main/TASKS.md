# Master Task Index (Registry)

**Version:** 1.18.0
**Generated:** 2026-07-10
**Based on:** .design/main/PLAN.md v2.14.0
**Based on RULES:** .design/RULES.md v1.5.0
**Execution Mode:** Parallel
**Status:** Active

## Overview

Tactical registry of all phases and their statuses, ordered by the growth metaphor (seed → stem → leaf → flower). Atomic checklists live in `tasks/phase-{N}.md`. Phases 1–11 are Done; Phase 12 (Skill System) is next. The v2.14.0 plan sync added no tasks: its six absorbed specs are three Stable L1 concepts (Phase 0, no implementation tasks by definition) and three RFC L2s parked in Backlog until they reach Stable.

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
| [Phase 10a](archives/tasks/phase-10.md) | Advanced Office Features (L2) — Foundational Wave: office-control, acp, navigation, automation-pipeline + kanban KAN-8 delta → 15 tasks / 5 tracks (A–E) | `Done (Archived)` |
| [Phase 10b](tasks/phase-10b.md) | Advanced Office Features (L2) — Dependent Wave: automation-canvas, voice-input, deliberation, version-control, inner-monologue, lookahead-planning, global-orchestration → 18 tasks / 7 tracks (F–L) | `Done` |
| [Phase 11](archives/tasks/phase-11.md) | Content, Sharing & Dev-Workflow Subsystems: resource-sharing, file-store, notes, development-workflow → 8 tasks / 4 tracks; +26 tests | `Done (Archived)` |
| [Phase 12](tasks/phase-12.md) | Skill System (L2): two-tier stores, canonical package, built-in command surface, conversion pipeline, prompt synthesis, `cronus skill` verbs → 8 tasks / 5 tracks (A–D, T) | `Todo` |

## Meta Information

- **Last Updated**: 2026-07-10 (v2.14.0 registry sync, no new tasks: 3 Stable L1 concepts → Phase 0 — tokenization-boundary `concept-only`, office-archetype + background-activation not concept-only since each has an authored `Implements:` L2 (C28 §4); 3 RFC L2s → Backlog — crate-topology, service-activation, archetype-catalog; l2-source-layout demoted Stable→RFC at 1.2.0 by the amendment rule, its 1.2.0 delta parked while the Phase 1 `[x]` stands at delivered 1.1.0 scope; Phase 10 KAN-8 checkbox corrected to `[x]` against `crates/core/src/kanban/custom_boards.rs`; INDEX v1.0.96 / PLAN v2.14.0. Prior: v2.13.1 light sync — l1-deep-research absorbed into Phase 0; INDEX v1.0.94 / PLAN v2.13.1)
- **Maintainer**: Core Team

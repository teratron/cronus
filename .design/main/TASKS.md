# Master Task Index (Registry)

**Version:** 1.19.0
**Generated:** 2026-07-10
**Based on:** .design/main/PLAN.md v2.15.0
**Based on RULES:** .design/RULES.md v1.5.0
**Execution Mode:** Parallel
**Status:** Active

## Overview

Tactical registry of all phases and their statuses, ordered by the growth metaphor (seed → stem → leaf → flower). Atomic checklists live in `tasks/phase-{N}.md`. Build phases 1–12 are Done. **Phase 13 (Core Decomposition) is open** — a structural, behavior-preserving crate-topology refactor decomposed from the now-Stable core-decomposition wave; effectively sequential despite Parallel mode.

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
| [Phase 12](archives/tasks/phase-12.md) | Skill System (L2): two-tier stores, canonical package, built-in command surface, conversion pipeline, prompt synthesis, `cronus skill` verbs → 8 tasks / 5 tracks (A–D, T) | `Done (Archived)` |
| [Phase 13](tasks/phase-13.md) | Core Decomposition (Crate Topology): repartition `crates/core` into contract · domain · store-local · auth-local · facade; invert the one domain→infra edge; DN-2 seams as crate boundaries; CI boundary guard → 8 tasks / 5 tracks (A–D, T) | `Todo` |

## Meta Information

- **Last Updated**: 2026-07-10 (v2.15.0: opened **Phase 13 — Core Decomposition** from the now-Stable core-decomposition wave (`l2-crate-topology` 1.0.0 + `l2-source-layout` 1.2.0, promoted RFC→Stable via `/magic.spec`, INDEX v1.0.97). 8 tasks / 5 tracks (A–D, T) mapping the topology spec's ordered §5 migration + validation; a structural behavior-preserving refactor, effectively sequential, pivot at T-13A02 (edge inversion), non-optional CI boundary guard at T-13D01. Wave removed from Backlog; PLAN v2.15.0. Prior: v2.14.1 milestone sync — all 12 build phases Done, no phase open; INDEX v1.0.96 / PLAN v2.14.1)
- **Maintainer**: Core Team

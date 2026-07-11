# Master Task Index (Registry)

**Version:** 1.21.0
**Generated:** 2026-07-11
**Based on:** .design/main/PLAN.md v2.18.0
**Based on RULES:** .design/RULES.md v1.5.0
**Execution Mode:** Parallel
**Status:** Active

## Overview

Tactical registry of all phases and their statuses, ordered by the growth metaphor (seed → stem → leaf → flower). Atomic checklists live in `tasks/phase-{N}.md`. Build phases 1–14 are Done. **Phase 15 (Memory Capture Policy & Metadata) is open** — the MI-6/10/11/12 capture-path delta deferred at Phase 14's delivered scope, decomposed into a 5-task, 3-track build phase over the same Stable `l2-memory-intelligence` spec (behind the `UserDataStore` seam). Domain-logic-first; foundation-then-parallel (Track A capture-metadata schema gates B); generator-optional degrade is the load-bearing acceptance property (MI-10 verbatim, MI-12 raw). Next step: `/magic.run main` to execute.

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
| [Phase 13](archives/tasks/phase-13.md) | Core Decomposition (Crate Topology): repartition `crates/core` into contract · domain · store-local · auth-local · facade; invert the one domain→infra edge; DN-2 seams as crate boundaries; CI boundary guard → 8 tasks / 5 tracks (A–D, T) | `Done (Archived)` |
| [Phase 14](archives/tasks/phase-14.md) | Memory Intelligence & Consolidation (L2): realize `l2-memory-consolidation` (MC-1…MC-10) + `l2-memory-intelligence` at query-surface scope (MI-1/2/3/4/5/7/8/9/13) over the SQLite substrate behind the UserDataStore seam — fact-vs-derived signal table, multiplicative ranking, corpus-maintenance pass, grounded `answer`, temporal modes, conflict routing, lifecycle states, gated experience reuse → 8 tasks / 4 tracks (A schema · B consolidation · C intelligence · T) | `Done (Archived)` |
| [Phase 15](archives/tasks/phase-15.md) | Memory Capture Policy & Metadata (L2): the MI-6/10/11/12 capture-path delta deferred at Phase 14 — salience-gated capture policy + actor/expiry/subject/cross-ref metadata (MI-6), capture-time date normalization (MI-10), caller capture directives (MI-11), raw/inferred mode (MI-12), all behind the UserDataStore seam → 5 tasks / 3 tracks (A capture-metadata schema · B capture path · T) | `Done (Archived)` |

## Meta Information

- **Last Updated**: 2026-07-11 (v1.21.0: **Phase 14 complete + archived; opened Phase 15 — Memory Capture Policy & Metadata** (post-run replan). Phase 14 delivered 9/9 tasks — `l2-memory-consolidation` whole (MC-1…MC-10) and `l2-memory-intelligence` at query-surface scope (MI-1/2/3/4/5/7/8/9/13); workspace `cargo test` 1,333 passed / clippy `-D warnings` / fmt clean. The four capture-path invariants (MI-6/10/11/12) were deferred at delivered scope — each needs net-new capture-metadata schema (actor/expiry/subject/cross-ref) and forms one coherent write-path unit — and carry forward to Phase 15 as an explicit delta (source-layout 1.1.0/1.2.0 precedent: Phase 14 keeps `[x]` at delivered scope, immutable; the remainder is a new line item, never a reopen). Phase 15 = 5 tasks / 3 tracks (A capture-metadata schema · B capture path · T), domain-logic-first, foundation-then-parallel; no `/magic.spec` needed (spec Stable, Invariant Compliance table specifies each net-new item). Pre-flight clean (ok:true, no header drift). Pre-Planning Stabilization: 0 promoted (sole Draft `l2-loop-runner` stays Draft — RFC parent). PLAN v2.18.0. Prior: v1.20.0: **opened Phase 14 — Memory Intelligence & Consolidation**. Both memory L2s (`l2-memory-consolidation` 1.0.1, `l2-memory-intelligence` 1.0.0) promoted RFC→Stable via `/magic.spec` (INDEX v1.0.101, 188 Stable) — one review fix in consolidation (§4.4/MC-8 reworded from "subsumes §4.2" to explicitly compositional, no store edit). Decomposed into 8 tasks / 4 tracks (A schema · B consolidation · C intelligence · T). Domain-logic-first, foundation-then-parallel: Track A (memory_signal table + depth/lifecycle columns) gates B/C; cross-edges MC-8→C-recall and MC-6-archive→MI-9-lifecycle respected; generator-optional degrade paths are build requirements. PLAN v2.17.0. Prior: v1.19.2 plan-sync (memory wave to Phase 0 + Backlog); v1.19.1 milestone (Phase 13 archived) (8/8 tasks Done; `l2-crate-topology` + `l2-source-layout` 1.2.0 realized — five-crate topology, INV-8 compiler-enforced, CI boundary guard live). All thirteen phases Done; every Stable spec implemented; no phase open. Pre-flight clean (ok:true, no drift). Pre-Planning Stabilization: 0 promoted — sole Draft `l2-loop-runner` stays Draft (RFC parent `l1-loop-governance`, layer constraint). Ten Backlog items all RFC/Draft — advance only via `/magic.spec`. DA-3 next target: `l2-service-activation` (direct downstream of Phase 13's crate seam). PLAN v2.15.1. Prior: v2.15.0 opened Phase 13 from the core-decomposition wave; v2.14.1 milestone — all 12 build phases Done)
- **Maintainer**: Core Team

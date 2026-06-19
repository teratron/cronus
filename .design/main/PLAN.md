# Implementation Plan

**Version:** 1.0.0
**Generated:** 2026-06-19
**Based on:** .design/main/INDEX.md v1.0.0
**Status:** Active

## Overview

Implementation plan for Cronus derived from 41 Stable specifications (18 L1 concepts + 23 L2 implementations). Layer-1 concepts are the technology-agnostic contracts (Phase 0); the implementation phases sequence the L2 work by dependency. The critical path runs through `crates/core` — most subsystems and all frontends depend on it. Execution mode: **Parallel** (C3), tracks grouped by file independence.

Phase ordering rationale: scaffold the buildable monorepo + engine skeleton + workflow runtime + security baseline (P1) → core subsystems memory/routing/workspace (P2) → office work engine roles/board/scheduler/quality (P3) → orchestration & autonomy (P4) → frontends (P5) → operational hardening (P6).

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic contracts. All Stable — these gate the implementation phases.*

- [x] **Architecture** ([l1-architecture.md](specifications/l1-architecture.md)) [L1]
- [x] **Office Model** ([l1-office-model.md](specifications/l1-office-model.md)) [L1]
- [x] **Storage & State Model** ([l1-storage-model.md](specifications/l1-storage-model.md)) [L1]
- [x] **Memory Model** ([l1-memory-model.md](specifications/l1-memory-model.md)) [L1]
- [x] **Workspace Lifecycle** ([l1-workspace-lifecycle.md](specifications/l1-workspace-lifecycle.md)) [L1]
- [x] **Kanban Model** ([l1-kanban-model.md](specifications/l1-kanban-model.md)) [L1]
- [x] **Scheduler Model** ([l1-scheduler-model.md](specifications/l1-scheduler-model.md)) [L1]
- [x] **Quality Standards** ([l1-quality-standards.md](specifications/l1-quality-standards.md)) [L1]
- [x] **Office Visualization** ([l1-office-visualization.md](specifications/l1-office-visualization.md)) [L1]
- [x] **Orchestration & Autonomy** ([l1-orchestration.md](specifications/l1-orchestration.md)) [L1]
- [x] **Roles** ([l1-roles.md](specifications/l1-roles.md)) [L1]
- [x] **Smart Routing** ([l1-routing.md](specifications/l1-routing.md)) [L1]
- [x] **Doctor & Self-Healing** ([l1-doctor.md](specifications/l1-doctor.md)) [L1]
- [x] **Client Security** ([l1-security.md](specifications/l1-security.md)) [L1]
- [x] **Error Reporting** ([l1-error-reporting.md](specifications/l1-error-reporting.md)) [L1]
- [x] **Telemetry** ([l1-telemetry.md](specifications/l1-telemetry.md)) [L1]
- [x] **Dashboard & Statistics** ([l1-dashboard.md](specifications/l1-dashboard.md)) [L1]
- [x] **Workflow Language** ([l1-workflow-language.md](specifications/l1-workflow-language.md)) [L1]

## Phase 1 — Foundation & Scaffolding

*Buildable monorepo, engine skeleton, workflow runtime integration, security baseline. Critical path: `crates/core`.*

- [ ] **Source Layout** ([l2-source-layout.md](specifications/l2-source-layout.md)) [L2] — Cargo + pnpm workspaces, polyglot runner
- [ ] **Technology Stack** ([l2-technology-stack.md](specifications/l2-technology-stack.md)) [L2] — toolchain (Rust, Vite/React 19, Tauri v2)
- [ ] **Filesystem Layout** ([l2-filesystem-layout.md](specifications/l2-filesystem-layout.md)) [L2] — OS-native path resolver, state bootstrap
- [ ] **Core Library** ([l2-core-library.md](specifications/l2-core-library.md)) [L2] — engine crate, public contract, durable state
- [ ] **Workflow Runtime** ([l2-workflow-runtime.md](specifications/l2-workflow-runtime.md)) [L2] — external crate integration + step-binding
- [ ] **Security** ([l2-security.md](specifications/l2-security.md)) [L2] — secret store, gitignore, redaction, egress gate, sandbox

## Phase 2 — Core Subsystems

*Memory, routing, and workspace management on top of the core + storage.*

- [ ] **Memory Store** ([l2-memory-store.md](specifications/l2-memory-store.md)) [L2] — SQLite + sqlite-vec + FTS5 + tags
- [ ] **Model Router** ([l2-model-router.md](specifications/l2-model-router.md)) [L2] — local-first, difficulty/cost, fallback, cache
- [ ] **Context Router** ([l2-context-router.md](specifications/l2-context-router.md)) [L2] — memory/rules/session routing
- [ ] **Workspace Management** ([l2-workspace-management.md](specifications/l2-workspace-management.md)) [L2] — tab UX, creation, manager bootstrap

## Phase 3 — Office Work Engine

*Roles, board, scheduling, and quality gates — the machinery that runs work.*

- [ ] **Role Catalog** ([l2-role-catalog.md](specifications/l2-role-catalog.md)) [L2] — presets + custom, hire/fire
- [ ] **Kanban Board** ([l2-kanban-board.md](specifications/l2-kanban-board.md)) [L2] — pipeline, transitions, auto-archival
- [ ] **Scheduler** ([l2-scheduler.md](specifications/l2-scheduler.md)) [L2] — friendly + cron, firing, routines
- [ ] **Quality Pipeline** ([l2-quality-pipeline.md](specifications/l2-quality-pipeline.md)) [L2] — per-language gate runner

## Phase 4 — Orchestration & Autonomy

*The coordination protocol that ties subsystems into an autonomous office.*

- [ ] **Orchestration** ([l2-orchestration.md](specifications/l2-orchestration.md)) [L2] — delegation, /goal+judge+budget, briefings, adaptive topology

## Phase 5 — Frontends

*The surfaces over the core (command parity, INV-3).*

- [ ] **CLI** ([l2-cli.md](specifications/l2-cli.md)) [L2]
- [ ] **TUI** ([l2-tui.md](specifications/l2-tui.md)) [L2]
- [ ] **Application UI** ([l2-app-ui.md](specifications/l2-app-ui.md)) [L2] — Tauri v2 + React 19, theming + i18n
- [ ] **Office View** ([l2-office-view.md](specifications/l2-office-view.md)) [L2] — graph + spatial projection
- [ ] **Dashboard** ([l2-dashboard.md](specifications/l2-dashboard.md)) [L2] — statistics projection

## Phase 6 — Operational Hardening

*Self-healing, backup, error reporting, and telemetry — productionization.*

- [ ] **Doctor** ([l2-doctor.md](specifications/l2-doctor.md)) [L2] — health checks, safe repair, recovery
- [ ] **Backup** ([l2-backup.md](specifications/l2-backup.md)) [L2] — state-tier backup/restore
- [ ] **GitHub Issue Reporting** ([l2-github-issue.md](specifications/l2-github-issue.md)) [L2] — consent + scrub + dedup
- [ ] **Telemetry** ([l1-telemetry.md](specifications/l1-telemetry.md)) [L1] — opt-in program metrics (implementation light)

## Backlog

<!-- All registered specs are scheduled across Phases 0–6; backlog is empty. -->

## Risks (Planning Audit)

- **Critical path = `crates/core`**: Phases 2–5 depend on it; a Phase 1 core slip cascades. Prioritize the core contract surface early.
- **External workflow-runtime crate**: the Rust runtime is a separate deliverable (its own repository) and does not exist yet (the reference runtime is in another language). Phase 1 Track D integrates it; if the Rust crate is not yet available, a minimal in-core stub of the step-binding interface unblocks dependents until it lands.
- **Mobile/Tauri scaffold**: iOS/Android Tauri setup is heavy and toolchain-fragile (see stack §5); validate the build+sign pipeline early, not in Phase 5.
- **Optimism on Phase 1**: it bundles scaffold + engine skeleton + runtime + security; treat tracks A–E as parallel but size generously.

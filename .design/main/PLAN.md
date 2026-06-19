# Implementation Plan

**Version:** 1.1.0
**Generated:** 2026-06-19
**Based on:** .design/main/INDEX.md v1.0.0
**Status:** Active

## Overview

Implementation plan for Cronus from 41 Stable specifications (18 L1 concepts + 23 L2 implementations). Phases follow a **growth order**: the agent grows like a sprout from a seed.

- **Seed = the library** (`crates/core` + `crates/nodus` runtime) — Phases 1–2.
- **Stem = the CLI** — Phase 3, the first usable surface, emerging straight from the seed.
- **Internal growth** = subsystems → office engine → orchestration — Phases 4–6 (the CLI gains commands as each lands).
- **Leaf = the TUI** — Phase 7.
- **Flower = the desktop application** — Phase 8.
- **Hardening** = operational productionization — Phase 9.

Execution mode: **Parallel** (C3); tracks grouped by file independence. Critical path runs through `crates/core` and the `crates/nodus` runtime it depends on.

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic contracts. All Stable — they gate the implementation phases.*

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

## Phase 1 — Seed I: Foundation

*Buildable monorepo + engine skeleton + state + security. The soil and the seed coat.*

- [ ] **Source Layout** ([l2-source-layout.md](specifications/l2-source-layout.md)) [L2] — Cargo + pnpm workspaces (`crates/{core,nodus,cli,tui}`), polyglot runner
- [ ] **Technology Stack** ([l2-technology-stack.md](specifications/l2-technology-stack.md)) [L2] — toolchain (Rust, Vite/React 19, Tauri v2)
- [ ] **Filesystem Layout** ([l2-filesystem-layout.md](specifications/l2-filesystem-layout.md)) [L2] — OS-native path resolver, state bootstrap
- [ ] **Core Library** ([l2-core-library.md](specifications/l2-core-library.md)) [L2] — engine crate, public contract, durable state
- [ ] **Security** ([l2-security.md](specifications/l2-security.md)) [L2] — secret store, gitignore, redaction, egress gate, sandbox

## Phase 2 — Seed II: Workflow Runtime (`crates/nodus`)

*The embeddable workflow-language runtime the core depends on. Behavior-preserving Rust port, built as a vertical slice first (see runtime spec §4.5).*

- [ ] **Workflow Runtime** ([l2-workflow-runtime.md](specifications/l2-workflow-runtime.md)) [L2] — lexer → parser/AST → transpiler → executor → validator/lint; step-binding to core subsystems

## Phase 3 — Stem: CLI

*The first usable surface, growing straight from the seed. Command framework + grammar + core binding; subsystem commands attach in later phases.*

- [ ] **CLI** ([l2-cli.md](specifications/l2-cli.md)) [L2] — `cronus` binary, command grammar, core bindings, initial commands (help/init/status/workflow)

## Phase 4 — Core Subsystems

*Memory, routing, and workspace management. Each lands with its CLI commands.*

- [ ] **Memory Store** ([l2-memory-store.md](specifications/l2-memory-store.md)) [L2]
- [ ] **Model Router** ([l2-model-router.md](specifications/l2-model-router.md)) [L2]
- [ ] **Context Router** ([l2-context-router.md](specifications/l2-context-router.md)) [L2]
- [ ] **Workspace Management** ([l2-workspace-management.md](specifications/l2-workspace-management.md)) [L2]

## Phase 5 — Office Work Engine

*Roles, board, scheduling, quality gates — the machinery that runs work.*

- [ ] **Role Catalog** ([l2-role-catalog.md](specifications/l2-role-catalog.md)) [L2]
- [ ] **Kanban Board** ([l2-kanban-board.md](specifications/l2-kanban-board.md)) [L2]
- [ ] **Scheduler** ([l2-scheduler.md](specifications/l2-scheduler.md)) [L2]
- [ ] **Quality Pipeline** ([l2-quality-pipeline.md](specifications/l2-quality-pipeline.md)) [L2]

## Phase 6 — Orchestration & Autonomy

*Coordination protocol that ties subsystems into an autonomous office.*

- [ ] **Orchestration** ([l2-orchestration.md](specifications/l2-orchestration.md)) [L2] — delegation, /goal+judge+budget, briefings, adaptive topology

## Phase 7 — Leaf: TUI

*Interactive terminal surface over the now-mature core (command parity with the CLI).*

- [ ] **TUI** ([l2-tui.md](specifications/l2-tui.md)) [L2]

## Phase 8 — Flower: Desktop Application

*The full graphical surface — the bloom.*

- [ ] **Application UI** ([l2-app-ui.md](specifications/l2-app-ui.md)) [L2] — Tauri v2 + React 19, theming + i18n
- [ ] **Office View** ([l2-office-view.md](specifications/l2-office-view.md)) [L2] — graph + spatial projection
- [ ] **Dashboard** ([l2-dashboard.md](specifications/l2-dashboard.md)) [L2] — statistics projection

## Phase 9 — Operational Hardening

*Self-healing, backup, error reporting, telemetry.*

- [ ] **Doctor** ([l2-doctor.md](specifications/l2-doctor.md)) [L2]
- [ ] **Backup** ([l2-backup.md](specifications/l2-backup.md)) [L2]
- [ ] **GitHub Issue Reporting** ([l2-github-issue.md](specifications/l2-github-issue.md)) [L2]
- [ ] **Telemetry** ([l1-telemetry.md](specifications/l1-telemetry.md)) [L1] — opt-in program metrics (implementation light)

## Backlog

<!-- All registered specs are scheduled across Phases 0–9; backlog is empty. -->

## Risks (Planning Audit)

- **Critical path = `crates/core` + `crates/nodus`**: Phases 3–8 depend on the library (Phases 1–2). Land the core contract and the nodus vertical slice early.
- **CLI-first surface (stem) is intentionally thin at Phase 3**: it ships the command framework + grammar + the commands available then; subsystem phases (4–6) attach their commands to it, and the TUI (Phase 7) mirrors the matured command set. This staging is the growth model, not scope creep.
- **nodus port size**: ~5k lines across six modules; Phase 2 builds it as a vertical slice (parse → transpile → minimal execute) before completing validator/lint and the full command set. Track parity against the reference test corpus.
- **Mobile/Tauri scaffold**: iOS/Android Tauri setup is toolchain-fragile (stack §5) — smoke-test the build/sign pipeline in Phase 1, not at Phase 8.

# Workspace Specifications Registry

**Version:** 1.0.0
**Status:** Active
**Engine Version:** 2.1.46

## Overview

Local registry of specifications for this workspace.

## Domain Specifications

| File | Description | Status | Layer | Version |
| --- | --- | --- | --- | --- |
| [l1-architecture.md](specifications/l1-architecture.md) | Layered core+frontends and hub-and-spoke topology (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-office-model.md](specifications/l1-office-model.md) | Office/corporation agent model: orchestrator, roles, client interaction, autonomy (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-storage-model.md](specifications/l1-storage-model.md) | Two-tier (immutable program / mutable state) + multi-level memory model (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-memory-model.md](specifications/l1-memory-model.md) | Memory subsystem: 4 scopes, hybrid recall, lifecycle, service+curator ownership (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-workspace-lifecycle.md](specifications/l1-workspace-lifecycle.md) | Home vs project workspaces, creation/edit/delete, default manager, hire/fire (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-kanban-model.md](specifications/l1-kanban-model.md) | Canonical work board: fixed pipeline, office-managed, auto-archival, one board per office (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-scheduler-model.md](specifications/l1-scheduler-model.md) | Schedules: recurring/one-shot, recurrence, fire actions (wake/routine/reminder), autonomy (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-quality-standards.md](specifications/l1-quality-standards.md) | Mandatory tiered quality gates as definition-of-done, role-enforced, universal+dogfood (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-office-visualization.md](specifications/l1-office-visualization.md) | Office as a live projection: graph + spatial floor, per-office + building overview (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-orchestration.md](specifications/l1-orchestration.md) | Coordination protocol: adaptive topology, delegation, /goal+judge+budget, briefings (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-roles.md](specifications/l1-roles.md) | Roles as specialties: preset + custom, hire/fire non-destructive, manager-driven (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-routing.md](specifications/l1-routing.md) | Smart-router pattern: multi-signal selection, fallback, cache, scope resolution (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-doctor.md](specifications/l1-doctor.md) | Self-healing: continuous checks, safe auto-repair, escalation, crash recovery (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-security.md](specifications/l1-security.md) | Client security: secret isolation, safe defaults, no exfiltration, sandboxing (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-error-reporting.md](specifications/l1-error-reporting.md) | Consent-gated, scrubbed, de-duplicated error reporting (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-telemetry.md](specifications/l1-telemetry.md) | Opt-in, program-data-only, privacy-first improvement telemetry (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-dashboard.md](specifications/l1-dashboard.md) | Live read-only statistics projection: per-office + building aggregate (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-workflow-language.md](specifications/l1-workflow-language.md) | Agent workflow DSL: dual-mode, schema contract, constraints, subsystem-bound steps (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l1-extensions.md](specifications/l1-extensions.md) | Unified extensions: skills/MCP/plugins, default-deny+sandboxed lifecycle, skill generation (tech-agnostic) | Stable | 1 | 1.0.0 |
| [l2-technology-stack.md](specifications/l2-technology-stack.md) | Validated cross-platform technology stack | Stable | 2 | 1.0.0 |
| [l2-core-library.md](specifications/l2-core-library.md) | Layer 1: embeddable core library (foundation) | Stable | 2 | 1.0.0 |
| [l2-cli.md](specifications/l2-cli.md) | Layer 2: command-line frontend | Stable | 2 | 1.0.0 |
| [l2-tui.md](specifications/l2-tui.md) | Layer 3: terminal UI frontend | Stable | 2 | 1.0.0 |
| [l2-app-ui.md](specifications/l2-app-ui.md) | Layer 4: desktop/web/mobile application UI/UX (incl. theming + localization) | Stable | 2 | 1.1.0 |
| [l2-filesystem-layout.md](specifications/l2-filesystem-layout.md) | OS-native filesystem layout: tiers, trees, DB placement, memory paths | Stable | 2 | 1.0.0 |
| [l2-memory-store.md](specifications/l2-memory-store.md) | Memory store: SQLite + sqlite-vec + FTS5 + tags, archivist curator, graph deferred | Stable | 2 | 1.0.0 |
| [l2-workspace-management.md](specifications/l2-workspace-management.md) | Desktop tab UX, creation form, kebab naming, template→state, manager bootstrap | Stable | 2 | 1.0.0 |
| [l2-kanban-board.md](specifications/l2-kanban-board.md) | Board storage, transitions, auto-archival store, board command surface | Stable | 2 | 1.0.0 |
| [l2-scheduler.md](specifications/l2-scheduler.md) | Friendly recurrence + raw cron, per-workspace storage, firing, schedule command surface | Stable | 2 | 1.0.0 |
| [l2-quality-pipeline.md](specifications/l2-quality-pipeline.md) | Per-language toolchain map (incl. JS/TS structural analysis), local/pre-commit/CI gate runner, check command surface | Stable | 2 | 1.1.0 |
| [l2-office-view.md](specifications/l2-office-view.md) | Office projection sources, graph+spatial render, home building overview, office command surface | Stable | 2 | 1.0.0 |
| [l2-orchestration.md](specifications/l2-orchestration.md) | Delegation via board, messaging, context isolation, judge+budget /goal loop, adaptive topology | Stable | 2 | 1.0.0 |
| [l2-role-catalog.md](specifications/l2-role-catalog.md) | Preset catalog + hired instances, role definition format, custom roles, hire/fire, role commands | Stable | 2 | 1.0.0 |
| [l2-model-router.md](specifications/l2-model-router.md) | Model selection: local-first, difficulty/cost, fallback cascade, semantic cache, routing commands | Stable | 2 | 1.0.1 |
| [l2-model-error-recovery.md](specifications/l2-model-error-recovery.md) | Error taxonomy (FailoverKind), priority classification pipeline, retry/compress/rotate/fallback loop, credential pool | Stable | 2 | 1.0.0 |
| [l2-agent-session.md](specifications/l2-agent-session.md) | Turn skeleton: TurnContext, IterationBudget, pluggable ContextEngine interface, prologue steps | Stable | 2 | 1.0.0 |
| [l2-context-router.md](specifications/l2-context-router.md) | Memory/rules/session routing, most-specific-first, session continue/retire | Stable | 2 | 1.0.0 |
| [l2-doctor.md](specifications/l2-doctor.md) | Health check suite, safe-repair vs escalate, crash recovery, extensible check registry, doctor command | Stable | 2 | 1.0.1 |
| [l2-security.md](specifications/l2-security.md) | Secret storage, gitignore defaults, egress gate, execution sandbox, audit log | Stable | 2 | 1.0.0 |
| [l2-github-issue.md](specifications/l2-github-issue.md) | Consent + scrub + dedup pipeline, GitHub issue filing, report commands | Stable | 2 | 1.0.0 |
| [l2-backup.md](specifications/l2-backup.md) | State-tier backup minus secrets/cache, restore-by-copy, backup commands | Stable | 2 | 1.0.0 |
| [l2-dashboard.md](specifications/l2-dashboard.md) | Statistics metrics + sources, per-office + home aggregate, dashboard command | Stable | 2 | 1.0.0 |
| [l2-workflow-runtime.md](specifications/l2-workflow-runtime.md) | In-tree Rust runtime crate (lexer/parser/validator/executor/transpiler), in-process, subsystem-bound; port architecture | Stable | 2 | 1.2.0 |
| [l2-extension-registry.md](specifications/l2-extension-registry.md) | Extension manifest, preset/custom locations, MCP client + sandbox, skill generation, plugin config schema, ext commands | Stable | 2 | 1.0.1 |
| [l2-learning-loop.md](specifications/l2-learning-loop.md) | Post-turn background review fork, skill package format, idle-triggered curator with lifecycle transitions | Stable | 2 | 1.0.0 |
| [l2-source-layout.md](specifications/l2-source-layout.md) | Dev monorepo layout: crates/apps/packages, dependency direction, in-tree workflow-runtime crate | Stable | 2 | 1.1.0 |
| [l2-agent-constitution.md](specifications/l2-agent-constitution.md) | Per-workspace identity file system: SOUL, PROFILE, MEMORY, HEARTBEAT, BOOTSTRAP files and bootstrap ritual | Stable | 2 | 1.0.0 |
| [l2-tool-security.md](specifications/l2-tool-security.md) | Two-layer tool defense: static skill scanner (8 categories) + runtime tool guard (10 threat categories, approval escalation) | Stable | 2 | 1.0.0 |
| [l2-mission-mode.md](specifications/l2-mission-mode.md) | Two-phase autonomous goal execution: PRD generation → user checkpoint → story-verified execution loop | Stable | 2 | 1.0.0 |

## Meta Information

- **Maintainer**: Core Team
- **Last Updated**: 2026-06-19
- **Spec Count**: 49 (19 L1 + 30 L2)

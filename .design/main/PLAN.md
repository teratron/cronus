# Implementation Plan

**Version:** 2.3.0
**Generated:** 2026-06-21
**Based on:** .design/main/INDEX.md v1.0.0
**Status:** Active

## Overview

Implementation plan for Cronus from 66 Stable specifications (19 L1 concepts + 47 L2 implementations). Phases follow a **growth order**: the agent grows like a sprout from a seed.

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
- [x] **Extensions** ([l1-extensions.md](specifications/l1-extensions.md)) [L1]

## Phase 1 — Seed I: Foundation

*Buildable monorepo + engine skeleton + state + security. The soil and the seed coat.*

> **Progress (2026-06-21):** Rust foundation complete and verified — filesystem path model, core skeleton + contract, durable state seam, and the security baseline (secret store / redaction / default-deny egress). The JS/Tauri scaffold under source-layout + technology-stack is **deferred to Phase 8** (toolchain `pnpm`/Tauri CLI absent here; off Phase 2's path). `l2-sandbox-policy` and `l2-multi-user-auth` are listed here but **not yet decomposed** — fold them in when Phase 1 is revisited (they do not gate Phase 2). See `tasks/phase-1.md`.

- [ ] **Source Layout** ([l2-source-layout.md](specifications/l2-source-layout.md)) [L2] — Cargo + pnpm workspaces (`crates/{core,nodus,cli,tui}`), polyglot runner
- [ ] **Technology Stack** ([l2-technology-stack.md](specifications/l2-technology-stack.md)) [L2] — toolchain (Rust, Vite/React 19, Tauri v2)
- [ ] **Filesystem Layout** ([l2-filesystem-layout.md](specifications/l2-filesystem-layout.md)) [L2] — OS-native path resolver, state bootstrap
- [ ] **Core Library** ([l2-core-library.md](specifications/l2-core-library.md)) [L2] — engine crate, public contract, durable state
- [ ] **Security** ([l2-security.md](specifications/l2-security.md)) [L2] — secret store, gitignore, redaction, egress gate, sandbox, SSRF guard, internal tool loopback; config integrity shields (three-state lock, SHA-256 seal, drift detection)
- [ ] **Sandbox Policy** ([l2-sandbox-policy.md](specifications/l2-sandbox-policy.md)) [L2] — deny-by-default network egress (named entries + binary allowlists), isolation tiers (restricted/balanced/open), preset catalog, PolicyContext, access failure classification (depends on security)
- [ ] **Multi-User Auth** ([l2-multi-user-auth.md](specifications/l2-multi-user-auth.md)) [L2] — bcrypt passwords, session tokens, TOTP 2FA, privilege map, admin promote/demote, reserved sentinel usernames (depends on security)

## Phase 2 — Seed II: Workflow Runtime (`crates/nodus`)

*The embeddable workflow-language runtime the core depends on. Behavior-preserving Rust port, built as a vertical slice first (see runtime spec §4.5).*

> **Done (2026-06-21):** 12 atomic tasks across 7 tracks complete — lexer → parser/AST → transpiler → executor → validator/lint + library API. 126 tests (83 unit + 26 parity + 17 WFL-invariant), 0 failures. Parity verified against the reference corpus; all WFL-1..9 L1 invariants covered. See [tasks/phase-2.md](tasks/phase-2.md).

- [x] **Workflow Runtime** ([l2-workflow-runtime.md](specifications/l2-workflow-runtime.md)) [L2] — lexer → parser/AST → transpiler → executor → validator/lint; step-binding to core subsystems

## Phase 3 — Stem: CLI

*The first usable surface, growing straight from the seed. Command framework + grammar + core binding; subsystem commands attach in later phases.*

> **Done (2026-06-21):** 8 atomic tasks across 4 tracks complete — clap 4 binary scaffold + help/init/status + workflow (scaffold/validate/run/transpile). 13 unit tests + 6 integration smoke tests = 19 total, 0 failures. See [archives/tasks/phase-3.md](archives/tasks/phase-3.md).

- [x] **CLI** ([l2-cli.md](specifications/l2-cli.md)) [L2] — `cronus` binary, command grammar, core bindings, initial commands (help/init/status/workflow)

## Phase 4 — Core Subsystems

*Memory, routing, and workspace management. Each lands with its CLI commands.*

- [ ] **Memory Store** ([l2-memory-store.md](specifications/l2-memory-store.md)) [L2] — SQLite + sqlite-vec + FTS5 + tags, archivist curator; trust scoring (asymmetric feedback, TRUST_MIN_SEARCH=0.3, retrieval_count), shallow entity links (seed of deferred graph), HRR phase encoding (model-free vector fallback, SNR capacity guard); Bellman propagation (gamma=0.9, alpha=0.1, max_depth=2, 1h temporal credit window), session chaining (2h window, Continuation links), VerificationState weight ladder (Untested=0.30 → ValidatedCrossProject=1.00)
- [ ] **Memory Encryption** ([l2-memory-encryption.md](specifications/l2-memory-encryption.md)) [L2] — AES-256-GCM per-chunk encryption, Argon2id KDF, OS keychain key storage, transactional rotation (depends on memory-store + security)
- [ ] **Code Graph** ([l2-codegraph.md](specifications/l2-codegraph.md)) [L2] — tree-sitter extraction, SQLite + FTS5 + sqlite-vec embeddings, RRF fusion, auto-index (depends on memory-store)
- [ ] **Model Router** ([l2-model-router.md](specifications/l2-model-router.md)) [L2] — local-first, difficulty/cost routing, fallback cascade, semantic cache; semantic router pool (embedding encoder + tolerance threshold, cost-optimal selection)
- [ ] **Model Error Recovery** ([l2-model-error-recovery.md](specifications/l2-model-error-recovery.md)) [L2] — error taxonomy, classification pipeline, credential pool; provider health probe (ProviderHealthStatus, multi-hop subprobes, context window discovery) (depends on model-router)
- [ ] **Agent Session Loop** ([l2-agent-session.md](specifications/l2-agent-session.md)) [L2] — TurnContext, IterationBudget, ContextEngine interface, tool-call loop seams, KV-cache stability, oversized-result summarizer, stop hooks, InterruptFence, post-turn hooks; text loop detection, goal re-entry cap (depends on model-router + context-router)
- [ ] **Agent Autonomy** ([l2-agent-autonomy.md](specifications/l2-agent-autonomy.md)) [L2] — autonomy ladder, SecurityPolicy gate, CommandRiskLevel classifier, ActionTracker rolling cap, approval gate lifecycle; ApprovalRecord manager (create/register separation, 15s grace period) (depends on agent-session + tool-security + scheduler)
- [ ] **Inbox** ([l2-inbox.md](specifications/l2-inbox.md)) [L2] — SQLite inter-actor inbox, send+drain pipeline, GC_TTL_MS=7d, MAX_DRAIN_PER_TURN=100, InboxArrived bus event, synthetic user message injection (depends on agent-session + storage)
- [ ] **Session Checkpoint** ([l2-session-checkpoint.md](specifications/l2-session-checkpoint.md)) [L2] — three-file hierarchy (checkpoint/memory/notes.md), section-budgeted reads, fork-agent prefix-cache parity write, boundary invariant, system reminders, progress reconcile, auto-memory triggers (depends on agent-session + memory-store + agent-registry)
- [ ] **Context Management** ([l2-context-management.md](specifications/l2-context-management.md)) [L2] — adaptive token budget, 8-step trim cascade, LLM-driven compaction, _protected messages; context engine registry (ContextEngineHostCapability, per_turn/thread_bootstrap projection, runtime modes) (depends on agent-session + model-router)
- [ ] **Context Router** ([l2-context-router.md](specifications/l2-context-router.md)) [L2]
- [ ] **Workspace Management** ([l2-workspace-management.md](specifications/l2-workspace-management.md)) [L2]
- [ ] **Agent Constitution** ([l2-agent-constitution.md](specifications/l2-agent-constitution.md)) [L2] — per-workspace identity files (SOUL/PROFILE/MEMORY/HEARTBEAT/BOOTSTRAP), bootstrap ritual (depends on workspace-management + memory-store)

## Phase 5 — Office Work Engine

*Roles, board, scheduling, quality gates, and extensions — the machinery and capabilities that run work.*

- [ ] **Role Catalog** ([l2-role-catalog.md](specifications/l2-role-catalog.md)) [L2]
- [ ] **Kanban Board** ([l2-kanban-board.md](specifications/l2-kanban-board.md)) [L2]
- [ ] **Scheduler** ([l2-scheduler.md](specifications/l2-scheduler.md)) [L2] — recurrence + cron + webhooks + event-driven triggers; cron isolated session execution (session key, model preflight, run log, delivery dispatch, failure notification)
- [ ] **Budget Engine** ([l2-budget-engine.md](specifications/l2-budget-engine.md)) [L2] — hierarchical budget policies, cost events, hard-stop enforcement, monthly reset (depends on roles + kanban)
- [ ] **Execution Workspace** ([l2-execution-workspace.md](specifications/l2-execution-workspace.md)) [L2] — isolated execution environments, no-remote-git contract, finalize write-back gate; git worktree lifecycle (slug naming, boot sequence, isPristine, reset+prune, events) (depends on security + kanban)
- [ ] **Quality Pipeline** ([l2-quality-pipeline.md](specifications/l2-quality-pipeline.md)) [L2]
- [ ] **Extension Registry** ([l2-extension-registry.md](specifications/l2-extension-registry.md)) [L2] — skills / MCP / plugins, sandboxed; skill generation; component auto-discovery (commands/agents/skills/hooks default dirs + manifest overrides, ${PLUGIN_ROOT} portable path); SKILL.md trigger format; agent definition frontmatter (name/description/model/color/tools); command definition frontmatter (dynamic tokens: $ARGUMENTS/$1/@file/!bash!/${PLUGIN_ROOT}); MCP transport variants (stdio/sse/http/ws, tool naming convention `mcp__<plugin>_<server>__<tool>`) (depends on roles + security + workflow runtime)
- [ ] **Plugin Hooks** ([l2-plugin-hooks.md](specifications/l2-plugin-hooks.md)) [L2] — actor.preStop/postStop ReAct loops, ActorMatcher filter, aggregated decision, file hooks auto-discovery, sequential external plugin loading, HookEvent observability; tool-event hook API (9 events: PreToolUse/PostToolUse/Stop/SubagentStop/SessionStart/SessionEnd/UserPromptSubmit/PreCompact/Notification, prompt+command models, parallel execution, hooks.json format, matcher syntax); rule evaluation engine (block/warn, AND conditions, 6 operators); hook security model (input validation, path safety, quoting, timeouts) (depends on extension-registry + agent-session)
- [ ] **Agent Registry** ([l2-agent-registry.md](specifications/l2-agent-registry.md)) [L2] — built-in and custom agent catalog, permission layer stack, fork-agent checkpoint-writer contract, generate-from-description API, default agent resolution (depends on role-catalog + tool-security + model-router + session-checkpoint)
- [ ] **Learning Loop** ([l2-learning-loop.md](specifications/l2-learning-loop.md)) [L2] — post-turn background review fork, skill package format, curator (depends on extension-registry + memory-store + agent-session)
- [ ] **Tool Security** ([l2-tool-security.md](specifications/l2-tool-security.md)) [L2] — two-layer defense: static skill scanner (8 categories) + runtime tool guard (10 threat categories, approval escalation, hard-blocked patterns)

## Phase 6 — Orchestration & Autonomy

*Coordination protocol that ties subsystems into an autonomous office.*

- [ ] **Orchestration** ([l2-orchestration.md](specifications/l2-orchestration.md)) [L2] — delegation, /goal+judge+budget, briefings, adaptive topology, agent tier hierarchy (Chat/Reasoning/Worker), MAX_SPAWN_DEPTH=3, toolkit action ranking
- [ ] **Trigger Triage** ([l2-trigger-triage.md](specifications/l2-trigger-triage.md)) [L2] — TriggerEnvelope intake pipeline, 4-outcome classifier (local CPU + cloud + rule fallback), dedup cache (depends on orchestration + scheduler + agent-session)
- [ ] **Mission Mode** ([l2-mission-mode.md](specifications/l2-mission-mode.md)) [L2] — two-phase autonomous goal execution: PRD generation → user checkpoint → story-verified loop with max-iterations circuit-breaker (depends on orchestration + kanban + tool-security)
- [ ] **Deep Research** ([l2-deep-research.md](specifications/l2-deep-research.md)) [L2] — iterative Think→Plan→Search→Extract→Synthesize engine, date-grounding, untrusted content wrapping, max_rounds circuit breaker (depends on orchestration + tool-security + context-management)

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
- [ ] **Config Hot-Reload** ([l2-config-hotreload.md](specifications/l2-config-hotreload.md)) [L2] — file-watcher with bounded backoff+polling fallback, prefix-keyed reload plan, subsystem action dispatch, skills snapshot invalidation (depends on doctor + scheduler + extension-registry)
- [ ] **Backup** ([l2-backup.md](specifications/l2-backup.md)) [L2]
- [ ] **GitHub Issue Reporting** ([l2-github-issue.md](specifications/l2-github-issue.md)) [L2] — consent + scrub + dedup pipeline, GitHub issue filing; error fingerprinting (BLAKE3 normalized hash, cross-episode dedup, prior-resolution surfacing)
- [ ] **Self-Improvement** ([l2-self-improvement.md](specifications/l2-self-improvement.md)) [L2] — calibration buckets (overconfidence metric, verified-ratio warning), mistake log (project/category/files), should-have-asked (trigger→question→answer), ask-backs (at-most-one-pending per project via partial UNIQUE INDEX), reasoning templates (task_type+domain → JSON steps, dream cycle extracted), brief surface (5-signal join at task start, cross-project mode) (depends on memory-store + learning-loop + agent-session + github-issue)
- [ ] **Telemetry** ([l1-telemetry.md](specifications/l1-telemetry.md)) [L1] — opt-in program metrics (implementation light)
- [ ] **Agent Migration** ([l2-agent-migration.md](specifications/l2-agent-migration.md)) [L2] — migration manifest v1, two-layer import (archives vs memory candidates), staged apply, source adapters (depends on memory-store + extension-registry + backup)

## Backlog

<!-- All registered specs are scheduled across Phases 0–9; backlog is empty. -->

## Risks (Planning Audit)

- **Critical path = `crates/core` + `crates/nodus`**: Phases 3–8 depend on the library (Phases 1–2). Land the core contract and the nodus vertical slice early.
- **CLI-first surface (stem) is intentionally thin at Phase 3**: it ships the command framework + grammar + the commands available then; subsystem phases (4–6) attach their commands to it, and the TUI (Phase 7) mirrors the matured command set. This staging is the growth model, not scope creep.
- **nodus port size**: ~5k lines across six modules; Phase 2 builds it as a vertical slice (parse → transpile → minimal execute) before completing validator/lint and the full command set. Track parity against the reference test corpus.
- **Mobile/Tauri scaffold**: iOS/Android Tauri setup is toolchain-fragile (stack §5) — smoke-test the build/sign pipeline in Phase 1, not at Phase 8.

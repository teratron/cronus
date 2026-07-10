# Implementation Plan

**Version:** 2.13.0
**Generated:** 2026-07-09
**Based on:** .design/main/INDEX.md v1.0.93
**Status:** Active

## Overview

Implementation plan for Cronus from the project registry (187 registered specs: 179 Stable, 7 RFC, 1 Draft). Phases follow a **growth order**: the agent grows like a sprout from a seed. This revision (v2.13.0) re-syncs the registry after a spec-authoring wave: **25 orphaned Stable L1 concepts** absorbed into Phase 0 (concept-only, C28), **1 RFC L1** (dev-office) parked in Backlog, and the **first post-11 implementation phase opened — Phase 12 (Skill System, l2-skill-system)** with 8 tasks / 5 tracks. Phases 1–11 are complete (Seed through Content/Sharing/Dev-Workflow). Prior revision (v2.12.0) completed all 11 Phase-10 L2 specs.

- **Seed = the library** (`crates/core` + `crates/nodus` runtime) — Phases 1–2.
- **Stem = the CLI** — Phase 3, the first usable surface, emerging straight from the seed.
- **Internal growth** = subsystems → office engine → orchestration — Phases 4–6 (the CLI gains commands as each lands).
- **Leaf = the TUI** — Phase 7.
- **Flower = the desktop application** — Phase 8.
- **Hardening** = operational productionization — Phase 9.

Execution mode: **Parallel** (C3); tracks grouped by file independence. Critical path runs through `crates/core` and the `crates/nodus` runtime it depends on.

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic contracts. All Stable — they gate the implementation phases.*

> **Concept-only semantics (RULES §7 C28):** a `[x]` on a Phase 0 L1 means "concept authored & Stable (a gating contract)", **not** "implemented". Many of these concepts have no authored L2 implementation yet — by design, this project builds the conceptual caul ahead of code. Such L1s carry the `concept-only` status: the concept-vs-code gap is named here rather than hidden, and they are exempt from the "Stable L1 without L2 child" coverage-gap advisory until an `Implements:` L2 is authored (at which point the marker auto-reverts). This replaces a hard authoring budget — L1 authoring stays cheap; the delta stays visible.

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
- [x] **Automation Pipeline** ([l1-automation-pipeline.md](specifications/l1-automation-pipeline.md)) [L1] — dual-mode event-driven automation; AP-1…AP-7; 8-node taxonomy; implicit (`@ON:` blocks) + explicit (canvas) surfaces; AP-6 AuditProvider observability
- [x] **Automation Canvas** ([l1-automation-canvas.md](specifications/l1-automation-canvas.md)) [L1] — visual pipeline editor; AC-1…AC-6; three-panel layout; implicit read-only + explicit authoring; inspection via AuditProvider stream
- [x] **Harness Engineering** ([l1-harness-engineering.md](specifications/l1-harness-engineering.md)) [L1] — EVALUATE→ANALYZE→IMPROVE loop; HE-1…HE-9; six-component harness taxonomy; artifact extraction mandatory (HE-8); context freshness per iteration (HE-9)
- [x] **Navigation Model** ([l1-navigation-model.md](specifications/l1-navigation-model.md)) [L1] — canonical 12-tab sidebar (Wiki added as #11, Settings → #12); NV-1…NV-5; office tab bar with lazy loading + OfficeState icons; two-tier settings (Global/Local); IDE integration via shell-spawn
- [x] **Voice Input** ([l1-voice-input.md](specifications/l1-voice-input.md)) [L1] — on-device only speech pipeline; VI-1…VI-5; ACTIVATE→CAPTURE→VAD→TRANSCRIBE→REVIEW→INJECT; push-to-talk + toggle modes
- [x] **Deliberation** ([l1-deliberation.md](specifications/l1-deliberation.md)) [L1] — multi-worker structured debate; DL-1…DL-5; parallel independent arguments; orchestrator finality; append-only log in Channels tab
- [x] **Office Control** ([l1-office-control.md](specifications/l1-office-control.md)) [L1] — OfficeState taxonomy (Active/Idle/Paused/Hibernating/Error/Offline); OC-1…OC-5; master switch; token exhaustion hibernation with model substitution; per-subsystem granularity
- [x] **Version Control** ([l1-version-control.md](specifications/l1-version-control.md)) [L1] — virtual staging area on git worktrees; VC-1…VC-6; trunk-based vs Git Flow; role authority table; Conventional Commits with card reference footer
- [x] **Inner Monologue** ([l1-inner-monologue.md](specifications/l1-inner-monologue.md)) [L1] — heartbeat-driven background cognitive process; IM-1…IM-5; 5 intention types; Pulse log; proactivity threshold with suppression
- [x] **Lookahead Planning** ([l1-lookahead-planning.md](specifications/l1-lookahead-planning.md)) [L1] — pre-execution consequence simulation N steps ahead; LP-1…LP-6; 7-category trigger catalog (branch merge/schema migration/file deletion/dep upgrade/security policy/mass refactor/arch change); CONFIRM/MODIFY/ESCALATE/BUDGET_EXHAUSTED conclusions
- [x] **Agent Client Protocol** ([l1-acp.md](specifications/l1-acp.md)) [L1] — transport-agnostic protocol for external callers; ACP-1…ACP-7; session lifecycle, streaming event taxonomy, capability declaration, trust levels (trusted/restricted/anonymous), cross-office ACP relay
- [x] **Global Orchestration** ([l1-global-orchestration.md](specifications/l1-global-orchestration.md)) [L1] — building-level coordination above individual offices; GO-1…GO-6; phase-awareness enforcement (cross-cutting concern catalog), cross-office delegation via ACP relay, building-level escalation with HITL gate

*Concepts authored since plan v2.5.0 (registry sync INDEX v1.0.4 → v1.0.36). All Stable — they gate later implementation phases.*

- [x] **Execution Graph** ([l1-execution-graph.md](specifications/l1-execution-graph.md)) [L1] — directed graph with typed state channels, superstep atomicity, conditional routing, dynamic spawn, interrupt/resume, checkpoint durability
- [x] **Task Graph Model** ([l1-task-graph-model.md](specifications/l1-task-graph-model.md)) [L1] — decomposition algebra: requirement-to-graph, dependency-DAG integrity, deterministic next-selection, journal, drift-driven re-planning, coordinator/executor split
- [x] **Agent Framework Skeleton** ([l1-agent-framework-skeleton.md](specifications/l1-agent-framework-skeleton.md)) [L1] — paradigm-neutral primitive triad (Agent/Work-Unit/Coordinator), 5-pattern coordination catalog, 3 concrete engines, autonomous self-improvement loop
- [x] **Tool Composition** ([l1-tool-composition.md](specifications/l1-tool-composition.md)) [L1] — toolkit as named group with dependency DAG, single authorization surface, deferred tool resolution at catalog scale (TC-7)
- [x] **Code Execution** ([l1-code-execution.md](specifications/l1-code-execution.md)) [L1] — code-execution-as-tool-use: typed sandboxed program with in-language capability calls, confinement parity, stateful cells, observable replay (CE-1…CE-8)
- [x] **Output Contracts** ([l1-output-contracts.md](specifications/l1-output-contracts.md)) [L1] — inline output validation (schema + callable + LLM criteria), retry budget with verdict injection, escalation to ORC-11
- [x] **Work Liveness** ([l1-work-liveness.md](specifications/l1-work-liveness.md)) [L1] — autonomous-work liveness & ownership: atomic claim, affirmative liveness contract, wake coalescing, stranded-work reconciliation, recovery ladder (WL-1…WL-9)
- [x] **Intent Resolution** ([l1-intent-resolution.md](specifications/l1-intent-resolution.md)) [L1] — ground-before-ask, assume-and-record never silently guess, risk-proportional ask-or-assume, correction re-plans dependents (IR-1…IR-7)
- [x] **Code Intelligence** ([l1-code-intelligence.md](specifications/l1-code-intelligence.md)) [L1] — source tree as queryable semantic graph; typed node/edge taxonomy, structural analysis family, intent-aware context assembly, hybrid retrieval (CI-1…CI-17)
- [x] **Knowledge Base** ([l1-knowledge-base.md](specifications/l1-knowledge-base.md)) [L1] — access-controlled document collections, incremental indexing, hybrid RAG retrieval, authorship zones, curation lifecycle (KB-1…KB-10)
- [x] **File Management** ([l1-file-management.md](specifications/l1-file-management.md)) [L1] — content-addressed dedup (SHA-256), metadata decoupled from blobs, access-controlled download, reference-tracking GC, immutable blobs
- [x] **Notes** ([l1-notes.md](specifications/l1-notes.md)) [L1] — persistent structured artifacts, access control, per-user pinning, append-only history, CRDT concurrent merge, soft deletion
- [x] **Folders** ([l1-folders.md](specifications/l1-folders.md)) [L1] — personal hierarchical session containers, non-destructive delete, sortable, unique sibling names
- [x] **Groups** ([l1-groups.md](specifications/l1-groups.md)) [L1] — flat named user sets as a principal type in the resource-sharing model, admin-managed membership, additive access
- [x] **Resource Sharing** ([l1-resource-sharing.md](specifications/l1-resource-sharing.md)) [L1] — uniform fine-grained access model: single grant primitive, three principal types, default-private, owner-invariant, additive grants, audit trail
- [x] **Operational Ledger** ([l1-operational-ledger.md](specifications/l1-operational-ledger.md)) [L1] — id-addressable operational ground truth: atomic predicates, supersede-don't-mutate, canonical-source precedence, verbatim grounding, required-reading (OL-1…OL-10)
- [x] **User Model** ([l1-user-model.md](specifications/l1-user-model.md)) [L1] — persistent evolving model of the person, inferred/evidence-backed/non-authoritative, anti-drift accretion, privacy-first inspect/erase (UM-1…UM-8)
- [x] **Multi-Device Sync** ([l1-multi-device-sync.md](specifications/l1-multi-device-sync.md)) [L1] — personal multi-device convergence, no central coordinator, data-class reconciliation routing (CRDT / 3-way / supersede), causal metadata (SY-1…SY-9)
- [x] **Change Merge** ([l1-change-merge.md](specifications/l1-change-merge.md)) [L1] — concurrent change as typed delta, sub-unit granularity, base fingerprinting, deterministic three-way merge, no-silent-loss (CM-1…CM-9)
- [x] **Project Wiki** ([l1-project-wiki.md](specifications/l1-project-wiki.md)) [L1] — client-facing office-maintained living docs as a projection, grounded/attributed anti-hallucination, freshness-honest, Wiki nav tab (PW-1…PW-8)
- [x] **Model Runtime** ([l1-model-runtime.md](specifications/l1-model-runtime.md)) [L1] — local-first on-device serving, provider-abstracted backend, content-addressed model store, fit-gated hardware scheduling, multi-device placement (MR-1…MR-14)
- [x] **Generation Budget** ([l1-generation-budget.md](specifications/l1-generation-budget.md)) [L1] — output-side token economy: minimal default reservation, truncation-detect-and-escalate, continue-from-partial, truncation-safe artifacts (GB-1…GB-8)
- [x] **Context Compression** ([l1-context-compression.md](specifications/l1-context-compression.md)) [L1] — third token-economy stage: reversible/bounded re-encoding of bulky structured content, recoverable, content-aware eligibility, runs before eviction (CC-1…CC-8)
- [x] **Retrieval Evaluation** ([l1-retrieval-evaluation.md](specifications/l1-retrieval-evaluation.md)) [L1] — IR-metric measurement of ranked-recall quality (P@K/R@K/MRR/nDCG@K) against labeled fixtures, baseline + regression gate, surface-agnostic (RE-1…RE-11)
- [x] **Application Shell** ([l1-application-shell.md](specifications/l1-application-shell.md)) [L1] — reactive desktop frontend substrate: single-authority state, push reactivity, declarative render, keymap dispatch, workbench composition (AS-1…AS-13)
- [x] **Generative Surface** ([l1-generative-surface.md](specifications/l1-generative-surface.md)) [L1] — agent-rendered interactive visual artifacts as a turn output, sandboxed, closed perception loop, user-controlled, projection-not-source (GS-1…GS-8)
- [x] **Development Workflow** ([l1-development-workflow.md](specifications/l1-development-workflow.md)) [L1] — five-stage agent-assisted pipeline (Design→Plan→Execute→Review→Deliver), task isolation, two-stage quality gate, durable ledger, human checkpoints (DW-1…DW-10)
- [x] **Practice Analytics** ([l1-practice-analytics.md](specifications/l1-practice-analytics.md)) [L1] — diagnostic/coaching engine over session traces: detector/rule separation, portable rule documents, honest data-gap accounting, severity-weighted scoring (PA-1…PA-14)
- [x] **Evaluation Suites** ([l1-evaluation-suites.md](specifications/l1-evaluation-suites.md)) [L1] — declarative version-controlled test suites for customizations, typed grader taxonomy, weighted thresholds, baseline+regression gate (ES-1…ES-16)
- [x] **Evaluations** ([l1-evaluations.md](specifications/l1-evaluations.md)) [L1] — per-message feedback: discrete sentiment + tags + free text, immutable audit, privacy-first, feeds router scoring and analytics
- [x] **Requirement Checklists** ([l1-requirement-checklists.md](specifications/l1-requirement-checklists.md)) [L1] — generated, domain-tailored, falsifiable questions validating requirements quality before code; pre-planning quality gate (RQ-1…RQ-8)
- [x] **Operational Health** ([l1-operational-health.md](specifications/l1-operational-health.md)) [L1] — on-device observability over runtime traces: explainable health score, threshold alerts, trend/anomaly detection, cost accounting, measure-don't-act (OH-1…OH-8)
- [x] **Facilitation** ([l1-facilitation.md](specifications/l1-facilitation.md)) [L1] — advisory thinking-facilitation: method catalog, elicitation loop, three ideation stances, divergence-before-convergence, advisory boundary (FC-1…FC-12)
- [x] **Agent Tool Ergonomics** ([l1-agent-tool-ergonomics.md](specifications/l1-agent-tool-ergonomics.md)) [L1] — design discipline for agent-facing tool surfaces: sufficiency-to-stop, recoverable-as-success, absence-as-signal, adapt-tool-to-agent (ATE-1…ATE-9)
- [x] **Policy Governance** ([l1-policy-governance.md](specifications/l1-policy-governance.md)) [L1] — administrative control over configuration: layered tier precedence, un-overridable managed tier, surface lockdown, integrity-verified source (PG-1…PG-8)
- [x] **Process Integrity** ([l1-process-integrity.md](specifications/l1-process-integrity.md)) [L1] — in-memory hardening of the agent's own process: no crash-dump leak, refuse tracer attach, scrub injection-vector env, early enforcement (PI-1…PI-7)
- [x] **Execution Sandbox** ([l1-execution-sandbox.md](specifications/l1-execution-sandbox.md)) [L1] — OS-level confinement across four deny-by-default axes (operations/privileges/resources/filesystem), capability drop, image pinning, fail-closed (ES-1…ES-9)
- [x] **Messaging Gateway** ([l1-messaging-gateway.md](specifications/l1-messaging-gateway.md)) [L1] — one gateway, many per-platform adapters; normalized contract, identity pairing, principal-keyed continuity, exposure safety, per-platform fault isolation (MG-1…MG-9)
- [x] **Browser Control** ([l1-browser-control.md](specifications/l1-browser-control.md)) [L1] — agent-driven web browser: persistent control daemon, accessibility-tree addressing, side-effect-classified commands, layered injection defense (BC-1…BC-12)

*Concepts authored since plan v2.6.0 (registry sync INDEX v1.0.36 → v1.0.39). All Stable — they gate later implementation phases.*

- [x] **Event Mesh** ([l1-event-mesh.md](specifications/l1-event-mesh.md)) [L1] — single in-process routing substrate: uniform event envelope, topic-based addressing, producer/consumer decoupling (producers name what happened, never a consumer) (EM-1…EM-10)
- [x] **Claim Verification** ([l1-claim-verification.md](specifications/l1-claim-verification.md)) [L1] — runtime hallucination detector over `(claims, sources)`: per-claim verdict (supported/contradicted/unverifiable) with evidence span; grounding-only, never world-truth (CV-1…CV-9)
- [x] **Perspective Model** ([l1-perspective-model.md](specifications/l1-perspective-model.md)) [L1] — theory-of-mind representation: every belief keyed by `(observer → subject)`, generalizing the single-vantage user model to perspectival self/other knowledge (PM-1…PM-8)

*Concepts authored since plan v2.8.0 (registry sync INDEX v1.0.39 → v1.0.61). All Stable, all `concept-only` (C28) — durable design contracts with no authored L2 yet; the marker auto-reverts when an `Implements:` L2 lands.*

- [x] **Work Convergence** ([l1-work-convergence.md](specifications/l1-work-convergence.md)) [L1] — all office activity through one legible surface (the board); materialize/drive/surface relations, exhaustive+exclusive, no shadow work (CONV-1…CONV-8)
- [x] **Context Provenance** ([l1-context-provenance.md](specifications/l1-context-provenance.md)) [L1] — per-fragment trust provenance, untrusted-neutralized-by-default at every composition boundary, sticky monotonic trust, structural injection defense (CP-1…CP-8)
- [x] **Attestation** ([l1-attestation.md](specifications/l1-attestation.md)) [L1] — signed, offline-verifiable artifact witnesses: content-set binding, authorship, typed claims, default-deny unattested, revocation-by-supersession (AT-1…AT-9)
- [x] **Tool Receipts** ([l1-tool-receipts.md](specifications/l1-tool-receipts.md)) [L1] — model-unforgeable per-action execution receipts; narrated-but-unreceipted actions treated as fabricated; ephemeral isolated signing secret (TR-1…TR-9)
- [x] **Issue Reporting** ([l1-issue-reporting.md](specifications/l1-issue-reporting.md)) [L1] — user-initiated report button/function: narrative + bug/feedback/idea categories, preview-before-send consent, opt-in diagnostics, shared error-reporting pipeline (ISS-1…ISS-7)
- [x] **Process Monitor** ([l1-process-monitor.md](specifications/l1-process-monitor.md)) [L1] — read-only live view of the app's own OS process tree with per-process CPU/memory, topology classified against sanctioned boundaries, CLI/TUI/GUI parity (PM-1…PM-7)
- [x] **Harness Optimization** ([l1-harness-optimization.md](specifications/l1-harness-optimization.md)) [L1] — outer-loop search over a harness-candidate space toward a frontier; archivable graded candidates, host-side search policy
- [x] **Harness Composition** ([l1-harness-composition.md](specifications/l1-harness-composition.md)) [L1] — right-sizing discipline for assembling harnesses; justification gates and pruning (HC), applied at role level by ROL-9
- [x] **Context Attachment** ([l1-context-attachment.md](specifications/l1-context-attachment.md)) [L1] — user-attached context artifacts as first-class turn inputs
- [x] **Agent Co-Evaluation** ([l1-agent-coevaluation.md](specifications/l1-agent-coevaluation.md)) [L1] — Performance = f(Model, Harness): the (model, harness) diagnostic matrix, orthogonal task labels, comparability contract, per-layer failure attribution (ACE)
- [x] **Cache-Stable Context** ([l1-cache-stable-context.md](specifications/l1-cache-stable-context.md)) [L1] — prompt-cache stability discipline: provider cache key as a cost lever, frozen prefixes, cache-warmth as a routing signal (CSC)
- [x] **Model Benchmarking** ([l1-model-benchmarking.md](specifications/l1-model-benchmarking.md)) [L1] — hardcoded three-class micro-benchmark (code/content/instruction-compliance) scoring base models on quality + time + tokens/cost into router-consumed fitness profiles (MB-1…MB-9)
- [x] **Crash Recovery** ([l1-crash-recovery.md](specifications/l1-crash-recovery.md)) [L1] — crash-safe writes, verified single-instant snapshots, unclean-shutdown detection, strict recovery ladder, work resumption + honest loss reporting (CR-1…CR-9)
- [x] **Parallel Staffing** ([l1-parallel-staffing.md](specifications/l1-parallel-staffing.md)) [L1] — same-specialty scale-out: ephemeral same-role instances under one accountable lead, parallelism only via disjoint decomposition, bounded width, first-class fan-in (PS-1…PS-9)
- [x] **Employee Availability** ([l1-employee-availability.md](specifications/l1-employee-availability.md)) [L1] — workforce state model: available/working/resting/on-leave/truant + released exit; office-state dominance, detected-never-declared truancy, resource-honest rest, honest metaphor binding (EMP-1…EMP-9)

*Concepts authored since plan v2.9.0 (registry sync INDEX v1.0.61 → v1.0.62). All Stable, all `concept-only` (C28).*

- [x] **Specialty Exemplars** ([l1-specialty-exemplars.md](specifications/l1-specialty-exemplars.md)) [L1] — per-specialty competency instrument, sibling to model-benchmarking pointed at staffing competency: one small concentrated exemplar suite per specialty
- [x] **Project Priority** ([l1-project-priority.md](specifications/l1-project-priority.md)) [L1] — cross-office resource arbitration: explicit user/board-set ordered priority per office/project governing how finite shared resources (building-level token/budget pool) are distributed under scarcity
- [x] **Project Support** ([l1-project-support.md](specifications/l1-project-support.md)) [L1] — the office's operational posture for a delivered/live project: ongoing upkeep (content updates, issue-driven fixes, product improvement) beginning where the build ends

*Concepts authored since plan v2.12.0 (registry sync INDEX v1.0.64 → v1.0.93). All Stable, concept-only until an `Implements:` L2 lands (C28).*

- [x] **Report Prompting & Diagnostic Findings** ([l1-report-prompting.md](specifications/l1-report-prompting.md)) [L1] — system-suggested, user-completed reporting bridging error-reporting and issue-reporting: invitation-never-transmission, closed trigger taxonomy, off/passive/active modes, local findings ledger for user invitations + developer triage (RP-1…RP-8)
- [x] **Work Import** ([l1-work-import.md](specifications/l1-work-import.md)) [L1] — bounded one-directional onboarding migration of an external tracker's backlog into the canonical office model
- [x] **Interception Model** ([l1-interception-model.md](specifications/l1-interception-model.md)) [L1] — interception discipline for cross-cutting behaviour: observe/decide/transform taxonomy, fixed fail-direction per class, transitive guard enforcement (INT-1…INT-8)
- [x] **Diagnostic Log Plane** ([l1-diagnostic-log.md](specifications/l1-diagnostic-log.md)) [L1] — forensic observation plane of last resort: survives native crashes, pre-init boot failures, dependency output; bounded rotating retention, consent-gated egress (DL-1…DL-8)
- [x] **Log Legibility & Economy** ([l1-log-legibility.md](specifications/l1-log-legibility.md)) [L1] — dual-audience (human + AI) contract over every observation channel: one canonical event, faithful projections, overload bounded by construction (LL-1…LL-9)
- [x] **System Readout & Refresh** ([l1-system-readout.md](specifications/l1-system-readout.md)) [L1] — one unified refresh mechanism for every live readout: closed trigger taxonomy, coalescing, honest staleness, visibility-gated economy (SR-1…SR-8)
- [x] **Search** ([l1-search.md](specifications/l1-search.md)) [L1] — application-wide capability to find anything the user can see through one query surface
- [x] **Usage Allowance** ([l1-usage-allowance.md](specifications/l1-usage-allowance.md)) [L1] — external-allowance member of the token economy: quota windows distinct from input-context and output-generation budgets
- [x] **Conversational Control** ([l1-conversational-control.md](specifications/l1-conversational-control.md)) [L1] — the natural-language chat/prompt surface as a first-class control plane over the application's own management operations
- [x] **Simulation** ([l1-simulation.md](specifications/l1-simulation.md)) [L1] — playing out a generated mechanism (workflow / pipeline / role interaction / task graph / schedule) to see how it behaves before it acts
- [x] **Tool-Call Transport** ([l1-tool-call-transport.md](specifications/l1-tool-call-transport.md)) [L1] — the wire seam by which a logical tool invocation crosses the agent↔model boundary and the reply is decoded back
- [x] **Design Identity** ([l1-design-identity.md](specifications/l1-design-identity.md)) [L1] — the office's swappable visual language + craft bar for user-facing visual output
- [x] **Competitive Execution** ([l1-competitive-execution.md](specifications/l1-competitive-execution.md)) [L1] — best-of-N quality-selection fan-out, the third coordination-family mode beside parallel-staffing and deliberation
- [x] **Component Scanning** ([l1-component-scanning.md](specifications/l1-component-scanning.md)) [L1] — admission vetting / threat scanning of third-party components (skill / tool-server / plugin) before trust
- [x] **Iterative Refinement** ([l1-iterative-refinement.md](specifications/l1-iterative-refinement.md)) [L1] — generate–evaluate–refine, the temporal member of the coordination family trading depth for quality
- [x] **Extension Marketplace** ([l1-extension-marketplace.md](specifications/l1-extension-marketplace.md)) [L1] — curated catalog / origin-and-discovery dimension for addressable extensions
- [x] **Inference Cache** ([l1-inference-cache.md](specifications/l1-inference-cache.md)) [L1] — durable tiered prefix-addressed cache of computed inference state, the storage-side companion to cache-stable authoring
- [x] **Progressive Disclosure** ([l1-progressive-disclosure.md](specifications/l1-progressive-disclosure.md)) [L1] — tiered on-demand context loading, the lazy-loading member of the context-economy family
- [x] **Agent Federation** ([l1-agent-federation.md](specifications/l1-agent-federation.md)) [L1] — interoperation with independent external agents as verifiable peers
- [x] **Deployment Neutrality** ([l1-deployment-neutrality.md](specifications/l1-deployment-neutrality.md)) [L1] — local-first server-free default as the whole product, with a pluggable door for auth + remote user-data backend
- [x] **Data Lineage** ([l1-data-lineage.md](specifications/l1-data-lineage.md)) [L1] — derivation traceability across a dataflow: what produced this datum, through which transformations
- [x] **Pattern Codification** ([l1-pattern-codification.md](specifications/l1-pattern-codification.md)) [L1] — the disciplined memory-to-governance pathway by which a stable behavioral pattern becomes an enforceable rule
- [x] **Provenance Taint** ([l1-provenance-taint.md](specifications/l1-provenance-taint.md)) [L1] — at-rest, persistent, propagating origin-trust taint on stored data
- [x] **Action Gating** ([l1-action-gating.md](specifications/l1-action-gating.md)) [L1] — authorization friction proportional to an action's consequence
- [x] **Recursive Decomposition** ([l1-recursive-decomposition.md](specifications/l1-recursive-decomposition.md)) [L1] — the third stance on an over-window input beside compression and progressive disclosure: process it in bounded parts

## Phase 1 — Seed I: Foundation

*Buildable monorepo + engine skeleton + state + security. The soil and the seed coat.*

> **Done (2026-06-30):** Rust foundation complete and verified — filesystem path model, core skeleton + contract, durable state seam, and the security baseline (secret store / redaction / default-deny egress). The JS/Tauri scaffold under source-layout + technology-stack was originally deferred here on a missing toolchain; it has since **landed via Phase 8 `T-8A01`** (pnpm + Tauri v2 provisioned, `apps/desktop` + `packages/ui` scaffolded, full toolchain green) — so the Seed's polyglot workspace is realized. The two remaining security-hardening specs (`l2-sandbox-policy`, `l2-multi-user-auth`) never gated any downstream phase and are **relocated to Phase 9 (Operational Hardening)** where they belong. Phase 1 is closed on its actual scope: the Rust seed. See `tasks/phase-1.md`.

- [x] **Source Layout** ([l2-source-layout.md](specifications/l2-source-layout.md)) [L2] — Cargo + pnpm workspaces (`crates/{core,nodus,cli,tui}`), polyglot runner (JS workspaces scaffolded via Phase 8 T-8A01)
- [x] **Technology Stack** ([l2-technology-stack.md](specifications/l2-technology-stack.md)) [L2] — toolchain (Rust; Vite/React 19 + Tauri v2 provisioned via Phase 8 T-8A01)
- [x] **Filesystem Layout** ([l2-filesystem-layout.md](specifications/l2-filesystem-layout.md)) [L2] — OS-native path resolver, state bootstrap
- [x] **Core Library** ([l2-core-library.md](specifications/l2-core-library.md)) [L2] — engine crate, public contract, durable state
- [x] **Security** ([l2-security.md](specifications/l2-security.md)) [L2] — secret store, gitignore, redaction, egress gate, sandbox, SSRF guard, internal tool loopback; config integrity shields (three-state lock, SHA-256 seal, drift detection)

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

> **Done (2026-06-22):** 14 atomic tasks across 5 tracks complete — memory store + encryption + codegraph (crate), model router + error recovery + context router, agent session loop + context management + session checkpoint + inbox + agent autonomy, workspace management + agent constitution, plus CLI commands and integration tests. 453 tests, 0 failures. See `tasks/phase-4.md`.

- [x] **Memory Store** ([l2-memory-store.md](specifications/l2-memory-store.md)) [L2] — SQLite + sqlite-vec + FTS5 + tags, archivist curator; trust scoring (asymmetric feedback, TRUST_MIN_SEARCH=0.3, retrieval_count), shallow entity links (seed of deferred graph), HRR phase encoding (model-free vector fallback, SNR capacity guard); Bellman propagation (gamma=0.9, alpha=0.1, max_depth=2, 1h temporal credit window), session chaining (2h window, Continuation links), VerificationState weight ladder (Untested=0.30 → ValidatedCrossProject=1.00)
- [x] **Memory Encryption** ([l2-memory-encryption.md](specifications/l2-memory-encryption.md)) [L2] — AES-256-GCM per-chunk encryption, Argon2id KDF, OS keychain key storage, transactional rotation (depends on memory-store + security)
- [x] **Code Graph** ([l2-codegraph.md](specifications/l2-codegraph.md)) [L2] — tree-sitter extraction, SQLite + FTS5 + sqlite-vec embeddings, RRF fusion, auto-index (depends on memory-store)
- [x] **Model Router** ([l2-model-router.md](specifications/l2-model-router.md)) [L2] — local-first, difficulty/cost routing, fallback cascade, semantic cache; semantic router pool (embedding encoder + tolerance threshold, cost-optimal selection)
- [x] **Model Error Recovery** ([l2-model-error-recovery.md](specifications/l2-model-error-recovery.md)) [L2] — error taxonomy, classification pipeline, credential pool; provider health probe (ProviderHealthStatus, multi-hop subprobes, context window discovery) (depends on model-router)
- [x] **Agent Session Loop** ([l2-agent-session.md](specifications/l2-agent-session.md)) [L2] — TurnContext, IterationBudget, ContextEngine interface, tool-call loop seams, KV-cache stability, oversized-result summarizer, stop hooks, InterruptFence, post-turn hooks; text loop detection, goal re-entry cap (depends on model-router + context-router)
- [x] **Agent Autonomy** ([l2-agent-autonomy.md](specifications/l2-agent-autonomy.md)) [L2] — autonomy ladder, SecurityPolicy gate, CommandRiskLevel classifier, ActionTracker rolling cap, approval gate lifecycle; ApprovalRecord manager (create/register separation, 15s grace period) (depends on agent-session + tool-security + scheduler)
- [x] **Inbox** ([l2-inbox.md](specifications/l2-inbox.md)) [L2] — SQLite inter-actor inbox, send+drain pipeline, GC_TTL_MS=7d, MAX_DRAIN_PER_TURN=100, InboxArrived bus event, synthetic user message injection (depends on agent-session + storage)
- [x] **Session Checkpoint** ([l2-session-checkpoint.md](specifications/l2-session-checkpoint.md)) [L2] — three-file hierarchy (checkpoint/memory/notes.md), section-budgeted reads, fork-agent prefix-cache parity write, boundary invariant, system reminders, progress reconcile, auto-memory triggers (depends on agent-session + memory-store + agent-registry)
- [x] **Context Management** ([l2-context-management.md](specifications/l2-context-management.md)) [L2] — adaptive token budget, 8-step trim cascade, LLM-driven compaction, _protected messages; context engine registry (ContextEngineHostCapability, per_turn/thread_bootstrap projection, runtime modes) (depends on agent-session + model-router)
- [x] **Context Router** ([l2-context-router.md](specifications/l2-context-router.md)) [L2]
- [x] **Workspace Management** ([l2-workspace-management.md](specifications/l2-workspace-management.md)) [L2]
- [x] **Agent Constitution** ([l2-agent-constitution.md](specifications/l2-agent-constitution.md)) [L2] — per-workspace identity files (SOUL/PROFILE/MEMORY/HEARTBEAT/BOOTSTRAP), bootstrap ritual (depends on workspace-management + memory-store)

## Phase 5 — Office Work Engine

*Roles, board, scheduling, quality gates, and extensions — the machinery and capabilities that run work.*

> **Done (2026-06-22):** 11 atomic tasks across 4 tracks complete — tool security (skill scanner + runtime guard + guardrail pipeline), role catalog (25 built-in roles, hire/fire/adapter), kanban board (card CRUD + state machine + archival), scheduler (recurring + oneshot + cron + isolated sessions), budget engine (hierarchical policies + hard-stop enforcement + monthly reset), execution workspace (git worktree lifecycle + finalize write-back gate), quality pipeline (per-language gate runner + board integration), extension registry (lifecycle states + component auto-discovery + MCP transport variants), plugin hooks (9 HookEvents + rule evaluation engine + hook security), agent registry (7 built-ins + user config layer + generate-from-description seam), learning loop (post-turn review fork + skill package format + curator approval gate). 36 test suites, 0 failures. See [archives/tasks/phase-5.md](archives/tasks/phase-5.md).

- [x] **Role Catalog** ([l2-role-catalog.md](specifications/l2-role-catalog.md)) [L2]
- [x] **Kanban Board** ([l2-kanban-board.md](specifications/l2-kanban-board.md)) [L2] — implemented at 1.0.x; the 1.1.0 KAN-8 custom-boards delta is tracked as an explicit Phase 10 item
- [x] **Scheduler** ([l2-scheduler.md](specifications/l2-scheduler.md)) [L2] — recurrence + cron + webhooks + event-driven triggers; cron isolated session execution (session key, model preflight, run log, delivery dispatch, failure notification)
- [x] **Budget Engine** ([l2-budget-engine.md](specifications/l2-budget-engine.md)) [L2] — hierarchical budget policies, cost events, hard-stop enforcement, monthly reset (depends on roles + kanban)
- [x] **Execution Workspace** ([l2-execution-workspace.md](specifications/l2-execution-workspace.md)) [L2] — isolated execution environments, no-remote-git contract, finalize write-back gate; git worktree lifecycle (slug naming, boot sequence, isPristine, reset+prune, events) (depends on security + kanban)
- [x] **Quality Pipeline** ([l2-quality-pipeline.md](specifications/l2-quality-pipeline.md)) [L2]
- [x] **Extension Registry** ([l2-extension-registry.md](specifications/l2-extension-registry.md)) [L2] — skills / MCP / plugins, sandboxed; skill generation; component auto-discovery (commands/agents/skills/hooks default dirs + manifest overrides, ${PLUGIN_ROOT} portable path); SKILL.md trigger format; agent definition frontmatter (name/description/model/color/tools); command definition frontmatter (dynamic tokens: $ARGUMENTS/$1/@file/!bash!/${PLUGIN_ROOT}); MCP transport variants (stdio/sse/http/ws, tool naming convention `mcp__<plugin>_<server>__<tool>`) (depends on roles + security + workflow runtime)
- [x] **Plugin Hooks** ([l2-plugin-hooks.md](specifications/l2-plugin-hooks.md)) [L2] — actor.preStop/postStop ReAct loops, ActorMatcher filter, aggregated decision, file hooks auto-discovery, sequential external plugin loading, HookEvent observability; tool-event hook API (9 events: PreToolUse/PostToolUse/Stop/SubagentStop/SessionStart/SessionEnd/UserPromptSubmit/PreCompact/Notification, prompt+command models, parallel execution, hooks.json format, matcher syntax); rule evaluation engine (block/warn, AND conditions, 6 operators); hook security model (input validation, path safety, quoting, timeouts) (depends on extension-registry + agent-session)
- [x] **Agent Registry** ([l2-agent-registry.md](specifications/l2-agent-registry.md)) [L2] — built-in and custom agent catalog, permission layer stack, fork-agent checkpoint-writer contract, generate-from-description API, default agent resolution (depends on role-catalog + tool-security + model-router + session-checkpoint)
- [x] **Learning Loop** ([l2-learning-loop.md](specifications/l2-learning-loop.md)) [L2] — post-turn background review fork, skill package format, curator (depends on extension-registry + memory-store + agent-session)
- [x] **Tool Security** ([l2-tool-security.md](specifications/l2-tool-security.md)) [L2] — two-layer defense: static skill scanner (8 categories) + runtime tool guard (10 threat categories, approval escalation, hard-blocked patterns)

## Phase 6 — Orchestration & Autonomy

*Coordination protocol that ties subsystems into an autonomous office.*

> **Done (2026-06-22):** 5 atomic tasks across 3 tracks complete — orchestration engine (delegation, goal/judge/budget, tier hierarchy, action ranking), trigger triage (4-outcome classifier + dedup + rate limiting), mission mode (two-phase PRD + execution loop), deep research (iterative search engine + untrusted content wrapping), plus validation track. See [archives/tasks/phase-6.md](archives/tasks/phase-6.md).

- [x] **Orchestration** ([l2-orchestration.md](specifications/l2-orchestration.md)) [L2] — delegation, /goal+judge+budget, briefings, adaptive topology, agent tier hierarchy (Chat/Reasoning/Worker), MAX_SPAWN_DEPTH=3, toolkit action ranking
- [x] **Trigger Triage** ([l2-trigger-triage.md](specifications/l2-trigger-triage.md)) [L2] — TriggerEnvelope intake pipeline, 4-outcome classifier (local CPU + cloud + rule fallback), dedup cache (depends on orchestration + scheduler + agent-session)
- [x] **Mission Mode** ([l2-mission-mode.md](specifications/l2-mission-mode.md)) [L2] — two-phase autonomous goal execution: PRD generation → user checkpoint → story-verified loop with max-iterations circuit-breaker (depends on orchestration + kanban + tool-security)
- [x] **Deep Research** ([l2-deep-research.md](specifications/l2-deep-research.md)) [L2] — iterative Think→Plan→Search→Extract→Synthesize engine, date-grounding, untrusted content wrapping, max_rounds circuit breaker (depends on orchestration + tool-security + context-management)

## Phase 7 — Leaf: TUI

*Interactive terminal surface over the now-mature core (command parity with the CLI).*

- [x] **TUI** ([l2-tui.md](specifications/l2-tui.md)) [L2]

## Phase 8 — Flower: Desktop Application

*The full graphical surface — the bloom.*

> **Done (2026-07-02):** all 10 tasks across 5 tracks complete — IPC bridge, settings persistence, shell systems (tray/shortcuts/overlay/single-instance), five-surface workbench with theming + i18n, Office View, Dashboard, provider prompt dispatch + XML env context, MCP client model, and both validation tracks (`fallow audit` clean, store-compliance proven). Tauri crate 34 tests; UI 27 tests; all gates green. See `archives/tasks/phase-8.md`.

- [x] **Application UI** ([l2-app-ui.md](specifications/l2-app-ui.md)) [L2] — Tauri v2 + React 19, theming + i18n
- [x] **Office View** ([l2-office-view.md](specifications/l2-office-view.md)) [L2] — graph + spatial projection
- [x] **Dashboard** ([l2-dashboard.md](specifications/l2-dashboard.md)) [L2] — statistics projection

## Phase 9 — Operational Hardening

*Self-healing, backup, error reporting, telemetry, and the deferred security-hardening layer.*

> **Relocated from Phase 1 (2026-06-30):** `l2-sandbox-policy` and `l2-multi-user-auth` are security-hardening specs that never gated any earlier phase; they sit here with the rest of the productionization work rather than blocking the foundation. Decomposed into tasks on entry.
>
> **Done (2026-07-03):** all 10 tasks across 5 tracks complete — sandbox policy + multi-user auth (security), doctor + config hot-reload (self-healing), backup + agent migration (data safety), GitHub issue reporting + self-improvement + telemetry (reporting/improvement), and a cross-subsystem hardening integration test proving no seeded secret reaches any egress surface. Domain-logic-first scope throughout: each subsystem's algorithm is fully implemented and tested against seeded/mock state; OS-level integration (real file watchers, GitHub API transport, SQLite persistence for dedup/brief tables) is deferred, documented per task. 40 new tests added to `crates/core` this phase (184 lib + 5 integration); workspace-wide gates green. See `archives/tasks/phase-9.md`.

- [x] **Sandbox Policy** ([l2-sandbox-policy.md](specifications/l2-sandbox-policy.md)) [L2] — deny-by-default network egress (named entries + binary allowlists), isolation tiers (restricted/balanced/open), preset catalog, PolicyContext, access failure classification (depends on security)
- [x] **Multi-User Auth** ([l2-multi-user-auth.md](specifications/l2-multi-user-auth.md)) [L2] — bcrypt passwords, session tokens, TOTP 2FA, privilege map, admin promote/demote, reserved sentinel usernames (depends on security)
- [x] **Doctor** ([l2-doctor.md](specifications/l2-doctor.md)) [L2]
- [x] **Config Hot-Reload** ([l2-config-hotreload.md](specifications/l2-config-hotreload.md)) [L2] — file-watcher with bounded backoff+polling fallback, prefix-keyed reload plan, subsystem action dispatch, skills snapshot invalidation (depends on doctor + scheduler + extension-registry)
- [x] **Backup** ([l2-backup.md](specifications/l2-backup.md)) [L2]
- [x] **GitHub Issue Reporting** ([l2-github-issue.md](specifications/l2-github-issue.md)) [L2] — consent + scrub + dedup pipeline, GitHub issue filing; error fingerprinting (BLAKE3 normalized hash, cross-episode dedup, prior-resolution surfacing)
- [x] **Self-Improvement** ([l2-self-improvement.md](specifications/l2-self-improvement.md)) [L2] — calibration buckets (overconfidence metric, verified-ratio warning), mistake log (project/category/files), should-have-asked (trigger→question→answer), ask-backs (at-most-one-pending per project via partial UNIQUE INDEX), reasoning templates (task_type+domain → JSON steps, dream cycle extracted), brief surface (5-signal join at task start, cross-project mode) (depends on memory-store + learning-loop + agent-session + github-issue)
- [x] **Telemetry** ([l1-telemetry.md](specifications/l1-telemetry.md)) [L1] — opt-in program metrics (implementation light)
- [x] **Agent Migration** ([l2-agent-migration.md](specifications/l2-agent-migration.md)) [L2] — migration manifest v1, two-layer import (archives vs memory candidates), staged apply, source adapters (depends on memory-store + extension-registry + backup)

## Phase 10 — Advanced Office Features (L2)

*L2 implementation specs for the L1 concepts added in 2026-06. These specs must be authored via `/magic.spec` before implementation tasks can be generated.*

> **Status:** Done (2026-07-04) — all 11 L2 specs implemented across `crates/core` (office_control, acp, automation, deliberation, version_control, inner_monologue, lookahead, global_orch, voice) + kanban KAN-8 custom_boards + presentation-only navigation/canvas in `packages/ui`. 33 tasks across two workbooks (phase-10.md tracks A–E, phase-10b.md tracks F–L). Domain-logic-first per the Phase-9 precedent: each subsystem's algorithm is implemented and tested against in-memory/mock state; OS/network/audio integration (real event bus, ACP transport, cpal/ONNX, cross-office relay) is deferred as documented seams. Gates green: cargo 250 lib tests + clippy `-D warnings` + fmt; vitest 39 tests (8 files) + biome. Kanban KAN-8 custom-boards delta implemented (spec Stable at 1.1.0).

*Foundational wave — authored & Stable:*

- [x] **Office Control** ([l2-office-control.md](specifications/l2-office-control.md)) [L2] — OfficeState machine, cooperative drain-and-checkpoint, token-exhaustion hibernation ladder (substitute-before-hibernate, auto-recovery wake), per-subsystem toggles; `Implements: l1-office-control.md`; deps orchestration + budget-engine + model-router (all Done)
- [x] **Agent Client Protocol** ([l2-acp.md](specifications/l2-acp.md)) [L2] — session store, monotonic event bus, capability/trust gate, pure projection adapters, cross-office relay, live-steering + interrupt over the agent-session `/acp` transport; `Implements: l1-acp.md`; deps agent-session + security + orchestration (all Done)
- [x] **Navigation** ([l2-navigation.md](specifications/l2-navigation.md)) [L2] — four-layer component tree, floor tab bar with lazy load + live OfficeState icons, frozen sidebar catalog, recursive mechanism sub-nav, two-tier settings, IDE launcher; `Implements: l1-navigation-model.md`; deps app-ui (Done) + office-control (this phase)
- [x] **Automation Engine** ([l2-automation-pipeline.md](specifications/l2-automation-pipeline.md)) [L2] — single PipelineEngine behind both modes, topological node executor, dedup window, scoped state over volatile/durable backends, control plane, lifecycle observers, composition, portable bundles; `Implements: l1-automation-pipeline.md`; deps trigger-triage + scheduler + orchestration (all Done)

*Dependent wave — authored & Stable (intra-phase deps satisfied by the foundational wave):*

- [x] **Automation Canvas UI** ([l2-automation-canvas.md](specifications/l2-automation-canvas.md)) [L2] — three-panel flow-graph projection, node rendering with live state, explicit editing + validation, read-only implicit surfaces, dev-run requests (AC-7), observer scope view (AC-8); `Implements: l1-automation-canvas.md`; deps automation-pipeline (this phase) + app-ui (Done)
- [x] **Voice Input** ([l2-voice-input.md](specifications/l2-voice-input.md)) [L2] — cpal 16 kHz capture, ONNX VAD, pluggable transcription engine, optional consent-gated transform, review overlay, model lifecycle, clipboard-safe injection; `Implements: l1-voice-input.md`; deps technology-stack + security (Done)
- [x] **Deliberation Engine** ([l2-deliberation.md](specifications/l2-deliberation.md)) [L2] — orchestrator-initiated round runner, parallel independent arguments, synthesis with attribution, immutable log over the inbox store, Channels surface; `Implements: l1-deliberation.md`; deps orchestration + inbox (Done) + navigation (this phase)
- [x] **Version Control** ([l2-version-control.md](specifications/l2-version-control.md)) [L2] — virtual staging area over execution-workspace worktrees, orchestrator-enforced role authority, quality-gated commit, Conventional Commits + card footer, trunk-based/Git-Flow; `Implements: l1-version-control.md`; deps execution-workspace + quality-pipeline + kanban-board (Done)
- [x] **Inner Monologue** ([l2-inner-monologue.md](specifications/l2-inner-monologue.md)) [L2] — heartbeat-gated cycle, read-only snapshot, token-bounded reflection, typed intentions, log-before-dispatch to Pulse log over the inbox store, proactivity threshold; `Implements: l1-inner-monologue.md`; deps scheduler + inbox + agent-session (Done) + navigation (this phase)
- [x] **Lookahead Planning** ([l2-lookahead-planning.md](specifications/l2-lookahead-planning.md)) [L2] — trigger-catalog detector, budget-bounded no-real-exec simulator, conclusion dispatcher with ORC-9 fallback, append-only decision log; `Implements: l1-lookahead-planning.md`; deps orchestration + kanban-board + execution-workspace (Done)
- [x] **Global Orchestration** ([l2-global-orchestration.md](specifications/l2-global-orchestration.md)) [L2] — event-bus aggregate view (GO-4), ACP relay routing (GO-5), phase-awareness card annotation (GO-3), building escalation (GO-6); `Implements: l1-global-orchestration.md`; deps orchestration (Done) + l2-acp + deliberation + office-control (this phase)
- [ ] **Kanban Custom Boards (KAN-8 delta)** ([l2-kanban-board.md](specifications/l2-kanban-board.md)) [L2] — spec already Stable at 1.1.0 (no authoring needed); implement the amendment delta over the Done Phase-5 board: custom columns with a mandatory canonical `anchor` in `board.json`, saved views over the single card set (no second store), re-anchor audit records; board UI surfaces follow the existing app-ui board line; depends on kanban-board (Phase 5) + app-ui (Phase 8)

## Phase 11 — Content, Sharing & Dev-Workflow Subsystems

*Ready Stable L2 subsystems authored after the Phase 1–10 narrative was drafted. Access-controlled content stores plus the bundled dev-workflow skill catalog. Decomposed into atomic tasks on entry (like Phases 8–10). Natural build order: resource-sharing (the access layer) before notes/file-store; development-workflow is independent.*

> **Status:** Done (2026-07-04) — all 4 subsystems implemented in `crates/core` domain-logic-first. resource_sharing (access foundation) → file_store + notes; development_workflow independent. 8 tasks; +26 unit tests. SQLite schemas, Yjs binary CRDT, StorageBackend, and SHA-256 are deferred seams; the resolution/dedup/convergence/stage-gate algebra is implemented and tested. Gates green: cargo 276 lib tests + clippy `-D warnings` + fmt.

- [x] **Resource Sharing** ([l2-resource-sharing.md](specifications/l2-resource-sharing.md)) [L2] — uniform access-grant model, `has_access` resolution (owner→user→group→public), write-implies-read, additive grants, audit events (RS-1…RS-8); `Implements: l1-resource-sharing.md` — the access foundation for notes/files/knowledge
- [x] **Notes** ([l2-notes.md](specifications/l2-notes.md)) [L2] — insertion-CRDT with order-independent convergence + idempotent merge, append-only version history, non-destructive soft-delete; `Implements: l1-notes.md`; depends on resource-sharing
- [x] **File Store** ([l2-file-store.md](specifications/l2-file-store.md)) [L2] — content-addressed dedup, reference-tracking GC, immutable blobs, metadata decoupled from bytes; `Implements: l1-file-management.md`; depends on resource-sharing
- [x] **Development Workflow** ([l2-development-workflow.md](specifications/l2-development-workflow.md)) [L2] — five-stage pipeline (Design→Plan→Execute→Review→Deliver), two-stage quality gate, human checkpoint, append-only progress ledger; `Implements: l1-development-workflow.md`; depends on extension-registry + agent-session

## Phase 12 — Skill System (L2)

*First post-11 implementation phase: the skill extension kind gets its concrete two-tier realization on the canonical execution stack.*

- [ ] **Skill System (Two-Tier Stores & Canonical Stack)** ([l2-skill-system.md](specifications/l2-skill-system.md)) [L2] — `Implements: l1-extensions.md, l1-storage-model.md` — preset store (program tier, read-only) + mutable store (state tier) with shadowing precedence; canonical package (no interpreted scripts); built-in command surface registered into the workflow-runtime vocabulary; conversion pipeline (verify → classify → retain → transpile → degrade → report, atomic); prompt synthesis; `cronus skill` import/create/status → 8 tasks / 5 tracks in [tasks/phase-12.md](tasks/phase-12.md)

## Backlog

*Non-Stable specs parked until they reach `Stable` (C6). Promoted into an active phase by a later `/magic.task` run once their status and any parent dependency clear.*

- [ ] **Memory Intelligence** ([l1-memory-intelligence.md](specifications/l1-memory-intelligence.md)) [L1] — `RFC` — active query & intelligence surface over the memory substrate (MI-1…MI-13); backlog until reviewed to Stable
- [ ] **Memory Consolidation** ([l1-memory-consolidation.md](specifications/l1-memory-consolidation.md)) [L1] — `RFC` — consolidation & corpus-maintenance layer between substrate and query surface (MC-1…MC-10); backlog until reviewed to Stable
- [ ] **Spec-Driven Governance** ([l1-spec-driven-governance.md](specifications/l1-spec-driven-governance.md)) [L1] — `RFC` — SDD governance meta-spec (SDG-1…SDG-15); backlog until reviewed to Stable
- [ ] **Dynamic Harness** ([l1-dynamic-harness.md](specifications/l1-dynamic-harness.md)) [L1] — `RFC` — run-time complement to harness engineering (DH-1…DH-12); backlog until reviewed to Stable
- [ ] **Loop Governance** ([l1-loop-governance.md](specifications/l1-loop-governance.md)) [L1] — `RFC` — loop-governance keystone (LG-1…LG-9); backlog until reviewed to Stable
- [ ] **Knowledge Store** ([l2-knowledge-store.md](specifications/l2-knowledge-store.md)) [L2] — `RFC` — `Implements: l1-knowledge-base.md`; pending KB-9/KB-10 compliance before promotion
- [ ] **Loop Runner** ([l2-loop-runner.md](specifications/l2-loop-runner.md)) [L2] — `Draft` — `Implements: l1-loop-governance.md`; blocked: L1 parent is `RFC` (cannot plan until parent reaches Stable)
- [ ] **Dev Office** ([l1-dev-office.md](specifications/l1-dev-office.md)) [L1] — `RFC` — developer (self-hosting) office: a system workspace for maintaining Cronus itself; backlog until reviewed to Stable

## Risks (Planning Audit)

- **Critical path = `crates/core` + `crates/nodus`**: Phases 3–8 depend on the library (Phases 1–2). Land the core contract and the nodus vertical slice early.
- **CLI-first surface (stem) is intentionally thin at Phase 3**: it ships the command framework + grammar + the commands available then; subsystem phases (4–6) attach their commands to it, and the TUI (Phase 7) mirrors the matured command set. This staging is the growth model, not scope creep.
- **nodus port size**: ~5k lines across six modules; Phase 2 builds it as a vertical slice (parse → transpile → minimal execute) before completing validator/lint and the full command set. Track parity against the reference test corpus.
- **Mobile/Tauri scaffold**: iOS/Android Tauri setup is toolchain-fragile (stack §5) — smoke-test the build/sign pipeline in Phase 1, not at Phase 8.
- **Concept-only growth (v2.9.0 sync)**: 17 more specs absorbed (15 Stable L1 concept-only into Phase 0, 2 RFC into Backlog). The concept-only count keeps rising while Phase 8 implementation is in flight — a C28 health signal worth watching: the concept caul is far ahead of code, and several new concepts (crash-recovery, employee-availability, parallel-staffing, model-benchmarking, process-monitor, issue-reporting) are natural L2 candidates for Phases 9–10 when authored.
- **Concept-caul growth continues (v2.13.0 sync)**: 27 more orphans absorbed — 25 Stable L1 concept-only into Phase 0 (the observability/diagnostics family diagnostic-log / log-legibility / system-readout / report-prompting, plus coordination, context-economy, and provenance families), 1 RFC (dev-office) to Backlog, and Phase 12 (skill-system) opened. The Stable-L1-without-L2 delta keeps widening (C28 health signal); the reporting/diagnostics cluster (issue-reporting v1.1.0 + report-prompting) is a natural candidate for the next L2 wave.
- **Registry sync debt (resolved in v2.8.0)**: 48 specs had accumulated outside the plan (INDEX raced ahead to v1.0.36). That revision absorbed them — but the new Phase 0 concept additions (e.g. execution-graph, code-intelligence, model-runtime, knowledge-base) imply future L2 implementation work not yet phased. Most have no authored L2 yet; they will surface as new phases via `/magic.spec` → `/magic.task` when promoted.
- **Amendment deltas over Done phases (v2.10.0)**: the kanban 1.1.0 amendment (KAN-8 custom boards) is the first spec change landing after its implementation phase closed. Pattern: the delta becomes an explicit item in the next open phase (here Phase 10) rather than being silently absorbed or reopening the Done phase — keeps Done phases immutable and the spec-vs-code gap visible.
- **TUI event-seam dependency (Phase 7)**: the render loop assumes a core event/subscribe seam. If the core exposes no pub/sub observer, the TUI must fall back to polling durable-state snapshots (INV-5 view-only). The Phase 7 tasks carry this fallback so the view panels are not hard-blocked on the subscription mechanism. Verify the seam first (T-7A02).

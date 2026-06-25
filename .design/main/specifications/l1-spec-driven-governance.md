# Spec-Driven Governance

**Version:** 1.0.0
**Status:** RFC
**Layer:** concept

## Overview

A paradigm-neutral discipline for governing autonomous agent work. It names the
small set of mechanics that let a system of AI workers produce durable artifacts
(specifications, plans, code) reliably — forcing structured thought before action,
encapsulating the artifact lifecycle so the human is interrupted only when it
matters, and protecting integrity with automatic drift and cascade guards.

This spec is a **meta-spec**, in the same vein as
[l1-agent-framework-skeleton.md](l1-agent-framework-skeleton.md): it does not
introduce a new subsystem. It distills a spec-driven development engine — studied
as a reference for how a *single* operator can stay productive while many agents
work below them — into one coherent vocabulary, a reusable invariant set, and an
ideas-to-adopt mapping onto the project's existing concept specs. Engines are
named by their structural mechanic, never by product. Concrete subsystems remain
authoritative for their own design; this document is the map that shows how the
governance mechanics compose over them and flags the few that are not yet captured.

The load-bearing thesis it borrows: **agents produce better work when forced to
think before they act, and a human stays in control by approving direction — not
by stamping every internal step.** Left unconstrained, an agent jumps to
implementation and produces fragile, misaligned output. A structured pipeline with
encapsulated status transitions makes that failure mode impossible without turning
the human into a bottleneck.

## Related Specifications

- [l1-development-workflow.md](l1-development-workflow.md) - Five-stage Design→Plan→Execute→Review→Deliver pipeline; the execution substrate the governance gates of §5.1 sit on.
- [l1-task-graph-model.md](l1-task-graph-model.md) - Decomposition algebra and status lifecycle; the planning surface the lifecycle of §5.2 governs.
- [l1-quality-standards.md](l1-quality-standards.md) - Tiered quality gates as definition-of-done; the review enforcement of §5.5.
- [l1-orchestration.md](l1-orchestration.md) - Adaptive topology, delegation, error containment; the runtime the decision protocol of §5.4 constrains.
- [l1-harness-engineering.md](l1-harness-engineering.md) - Evidence-backed iterative improvement; the disciplined counterpart to the retrospective of §5.7.
- [l1-roles.md](l1-roles.md) - Roles as specialties; the actors that staff the two-stage review of §5.5.
- [l1-doctor.md](l1-doctor.md) - Self-healing and continuous checks; the runtime home for the drift guards of §5.3.
- [l1-version-control.md](l1-version-control.md) - Commit authority and quality-gated commit boundary; where the finalization mechanic of §5.8 lands.
- [l1-agent-framework-skeleton.md](l1-agent-framework-skeleton.md) - The sibling meta-spec for agent anatomy; this document governs the *artifacts* those agents produce, not the agents themselves.

## 1. Motivation

A productive single operator running many agents needs two things that pull in
opposite directions: **velocity** (do not stop to ask about every step) and
**control** (never lose the thread of what is being built or why). A spec-driven
governance layer reconciles them by making the *artifact* — not the human's
attention — the unit of integrity. Three benefits follow:

1. **Think-before-act is structural, not advisory.** A gate that requires an
   approved concept artifact before a plan, and an approved plan before execution,
   removes the most common autonomous-agent failure: confident implementation of
   the wrong thing. The discipline is enforced by the pipeline, not by reminding
   the agent to be careful.
2. **Human attention is rationed by a whitelist, not spent by default.** Elective
   decisions are resolved autonomously and *narrated*; only a closed set of
   consequential forks (irreversible actions, external releases, genuine
   architectural ambiguity) escalate to a question. The operator approves direction
   at a final gate and audits the trail on demand, instead of rubber-stamping
   internal status changes.
3. **Integrity is defended automatically.** Drift between what is registered and
   what exists, between a foundation and its dependents, between tooling and its
   checksum, is detected and contained before it corrupts downstream work — rather
   than discovered late as a mysterious inconsistency.

The cost of *not* having this layer is the autonomous-agent tar pit: a swarm that
is locally busy and globally adrift, producing artifacts no one registered, on
foundations that quietly changed, gated by a human who is either over-consulted
into uselessness or under-consulted into surprise.

## 2. Constraints & Assumptions

- **Technology-agnostic.** This is a Layer 1 concept. It names no language,
  storage engine, or tool. Concrete bindings (scripts, schemas, file formats) live
  in Layer 2 specs.
- **Source-agnostic.** The mechanics are stated as portable patterns. The reference
  engine that inspired them is named by mechanic, never by product, and is not a
  dependency.
- **Generalization, not replacement.** Where a concrete spec already governs a
  concern (quality gates, task lifecycle, version control), this document defers to
  it and links it. It wins only on the cross-cutting governance vocabulary and the
  gap-filling invariants flagged **New** in §5.10.
- **Human-minimal, not human-absent.** The goal is to minimize *interruptions*, not
  oversight. Every autonomous resolution remains auditable and reversible; the
  escalation whitelist is the hard floor below which the system must ask.
- **Two profiles, one engine.** The same governed pipeline must serve both a
  high-velocity operator who wants the lifecycle hidden and a high-rigor operator
  who wants every artifact and status visible (SDG-15). Integrity guarantees are
  identical across profiles; only surfaced detail differs.
- **Composition over ceremony.** Governance scales down: a small change rides a
  lightweight track and a large one rides the full template, with automatic
  promotion between them (SDG-4). Ceremony that does not earn its keep is a defect.

## 3. Core Invariants

Layer 2 realizations and concrete subsystems MUST NOT violate these.

- **SDG-1 Artifact-before-action.** No execution without an approved structured
  artifact; no plan without an approved concept. The agent records *what* and *how*
  as inspectable artifacts before it acts, so intent is reviewable independently of
  the act.
- **SDG-2 Two-layer artifact model.** Every governed artifact is either a **Concept**
  (technology-agnostic *what*: rules, invariants, contracts — the durable source of
  truth) or an **Implementation** (stack-specific *how*) that declares its Concept
  parent. An Implementation artifact MUST NOT reach a review-ready or approved
  status until its Concept parent is approved. Migrating stacks re-authors only
  Implementation artifacts; Concept artifacts are untouched.
- **SDG-3 Bounded lifecycle with encapsulated promotion.** Artifacts move through a
  bounded status lifecycle (draft → review → approved → deprecated). When no
  objective conflict exists, the system MAY auto-promote silently and surface only a
  final actionable outcome plus a single execution gate. Internal statuses are an
  audit trail, never a mandatory per-step human prompt.
- **SDG-4 Completeness gate with adaptive weight.** Auto-promotion requires a
  minimum content bar — an overview plus at least one substantive design section —
  not every template section. Ceremony scales to task size: a lightweight track for
  small changes, auto-promoted to the full template once it crosses a declared
  size or complexity threshold.
- **SDG-5 Quarantine cascade.** If a Concept artifact is demoted from approved, every
  dependent Implementation artifact and its downstream tasks are automatically
  flagged and blocked, recursively. This downward status change is authoritative —
  execution layers react to it and never silently reverse it. Work never proceeds on
  a destabilized foundation.
- **SDG-6 Registry parity.** Every artifact on disk is registered in a single index,
  and every registered artifact appears in the active plan or an explicit backlog.
  An orphan — registered but absent, or present but unregistered, or registered but
  unplanned — is a critical blocker, not a warning.
- **SDG-7 Drift guards.** Edits made outside the governed lifecycle are detected by
  comparing a recorded baseline against live state across four surfaces: artifact
  version vs registry, convention version vs plan base, tooling integrity vs
  checksum, and configuration vs declared metadata. Detected drift halts dependent
  workflows until reconciled; reconciliation re-applies the amendment rule so the
  unreviewed external change is captured rather than swallowed.
- **SDG-8 Autonomous decision protocol.** Between elective forks the system decides
  and narrates; it does not ask. Each autonomous resolution is one line carrying the
  decision, the deciding criterion, and an override handle. Questions are reserved
  for a closed escalation whitelist — irreversible or destructive actions, external
  release artifacts, hard-fork ambiguity with no objective tiebreaker, governance
  amendments, and routing ambiguity. A whitelist question is single, offers at most
  three fixed options, and marks a recommended default. This decide-don't-ask posture
  persists across task boundaries and does not reset to assistant defaults between
  steps.
- **SDG-9 Two-stage artifact review.** A changed artifact passes an adversarial
  content critic (purity, completeness, coherence, link integrity) and then an
  instruction-quality pass (contradiction, ambiguity, persona consistency, cognitive
  load, semantic coverage, composition coherence) before promotion. The quality pass
  never runs on a critic-rejected artifact; any blocking finding reverts the artifact
  to its pre-promotion status.
- **SDG-10 Navigable artifact graph.** A large artifact corpus is projected into a
  relationship graph that surfaces hub nodes (over-connected), orphans (unreferenced),
  missing parent links, and coverage gaps, plus a generated human-readable index.
  Architectural navigation and recall draw on the graph under a token budget rather
  than re-reading raw artifacts.
- **SDG-11 One-way traceability containment.** Governance artifacts reference product
  work; product artifacts never reference governance artifacts. Provenance (which
  unit produced a change) lives in commit metadata and artifact change-fields, so a
  shipped product that excludes the governance layer carries no dead references.
- **SDG-12 Finalize, never auto-commit.** On workflow completion the system bumps the
  artifact version, appends a release-notes entry, and emits a suggested
  conventional-commit message — but performs no write-side version-control operation.
  Staging, committing, and pushing remain the human's decision.
- **SDG-13 Closed-loop retrospective.** A lightweight retrospective fires per
  completed phase (a health and metrics snapshot) and a deep retrospective fires per
  completed plan (drift, blocked-task patterns, friction, and delivery metrics such
  as deployment frequency and change-failure rate). Findings feed back as actionable
  amendments into the artifact layer; planned-versus-built is measured, not assumed.
- **SDG-14 Session continuity and single next step.** A durable live-memory state
  artifact records position, progress, and blockers; every command updates it. A
  read-only briefing answers "where am I and what is the one next command" without
  recomputation, and resume is plan-state-aware: the next action is computed from
  dependency topology and status maturity, then narrated — never asked.
- **SDG-15 Two operating profiles.** The same governed pipeline serves a
  high-velocity "trust" profile (idea in, lifecycle encapsulated, one final go-gate)
  and a high-rigor "audit" profile (full transparency into every artifact, status,
  and rule). The profile changes only how much of the lifecycle is surfaced, never
  the underlying integrity guarantees.

## 5. Detailed Design

### 5.1 The governance pipeline

Governed work flows through ordered gates, each producing an inspectable artifact:
**specify** (author the Concept and Implementation artifacts) → **plan** (decompose
into an addressable, dependency-ordered task graph) → **execute** (do the work in an
isolated context) → **review** (two-stage critique) → **deliver** (finalize and
hand the commit decision to the human). SDG-1 makes the first two gates
non-skippable: there is no execution path that bypasses an approved plan, and no
plan path that bypasses an approved concept. The pipeline is the spine; the
project's development-workflow concept is its concrete five-stage realization.

The gates are *encapsulated* by default (SDG-3): in the trust profile the operator
sees a single "specs are ready, proceeding to plan" narration and a final execution
gate, not five approval prompts. The audit profile exposes each gate's artifact and
status for inspection at any time. The integrity checks below run identically in
both.

### 5.2 The two-layer artifact model and lifecycle

The Concept/Implementation split (SDG-2) is the structural core. A Concept artifact
states the durable *what* and is portable across stacks; an Implementation artifact
binds one Concept to a concrete realization and names its parent explicitly. The
parent-gated promotion rule (an Implementation cannot be approved before its Concept)
prevents the classic failure of freezing "how we built it" before "what we want" is
settled.

Status moves along a bounded lifecycle. Promotion is governed by a **minimum viable
completeness** bar (SDG-4): an overview plus one substantive design section is enough
to advance; a missing optional section is not a blocker. Ceremony is adaptive — a
micro-track carries small, tactical changes without the full template, and the
engine promotes a micro-artifact to the standard template once it outgrows a size or
complexity threshold. When an approved Concept is later destabilized, the
**quarantine cascade** (SDG-5) walks the dependency graph downward, demoting every
dependent Implementation and blocking its tasks, recursively, so the plan cannot
proceed on a broken foundation. The cascade is the authoritative direction of status
change; upward promotion is owned by the planning and stabilization passes, never by
execution.

### 5.3 Integrity guards

Three guards keep the artifact corpus self-consistent without human policing:

- **Registry parity (SDG-6).** A single index is the source of truth for what
  exists. A reconciliation pass treats any mismatch — a file with no index row, an
  index row with no file, or a registered artifact absent from both plan and backlog
  — as a blocker. Orphans are the most common silent corruption in a large
  agent-built corpus; promoting them to blockers makes them impossible to ignore.
- **Drift family (SDG-7).** Four comparisons run as pre-flight checks: artifact
  version vs registry entry, convention (rules) version vs the base a plan was
  generated against, tooling integrity vs a recorded checksum, and live config vs
  declared spec metadata. Any mismatch halts the dependent workflow with a single
  recommended reconciliation path. Reconciliation does not merely silence the
  warning — it re-applies the amendment rule so the external change enters the
  lifecycle and is reviewed.
- **Tooling integrity.** The governance engine's own files are checksummed; an
  untracked modification halts workflows until the state is reconciled, preventing a
  silently altered engine from corrupting every downstream artifact.

These guards are the runtime responsibility of the self-healing (doctor) concept;
this spec contributes the *family* framing — that registry, version, rules, config,
and tooling drift are one class of defect with one containment pattern.

### 5.4 The autonomous decision protocol

This is the mechanic that makes "human-minimal" safe rather than reckless (SDG-8).
Every fork the system faces is classified as **elective** (a tiebreaker exists by
some objective criterion — pipeline order, dependency topology, status maturity,
coverage) or **escalation** (it belongs to the closed whitelist). Elective forks are
resolved immediately and narrated as a one-line **Decision Record**: the decision,
the criterion that decided it, and an override handle that preserves the operator's
control point. Escalation forks — and only these — produce a single question with at
most three fixed options and a marked default.

The whitelist is deliberately narrow: irreversible or destructive actions, external
release artifacts (anything published outside the local boundary), genuine
architectural hard-forks with no objective tiebreaker, amendments to the project
constitution, and ambiguous routing of work between contexts. Everything else is the
agent's call to make and narrate. Crucially, this posture is **session-persistent**:
the system does not relax back into "ask about everything" at task boundaries. The
project's agent-constitution concept already embodies the spirit of this (draft-first,
at most two questions, optimistic defaults); this spec sharpens it into an explicit
whitelist plus the Decision-Record grammar.

### 5.5 Two-stage review and quality gates

Promotion of any artifact passes two distinct critics in order (SDG-9). The **content
critic** is adversarial about substance: are Concept invariants truly
technology-neutral, are edge cases and error states covered, does an Implementation's
compliance table actually verify each parent point or merely gesture at it, are links
accurate, does the document still read coherently after the edit. The
**instruction-quality pass** then audits the artifact *as a prompt* an agent will
later act on: contradictions, ambiguity, persona drift, cognitive overload, coverage
gaps, composition conflicts. Ordering matters — the quality pass never runs on a
critic-rejected artifact, and either critic's blocking finding reverts status. This
two-stage shape generalizes the project's adversarial code-review and skill-document
semantic-analysis gates from code onto *all* governed artifacts.

### 5.6 The artifact knowledge graph

A corpus of dozens of interlinked specs is not navigable by reading files (SDG-10).
The governance layer projects the corpus into a relationship graph and derives signal
from its shape: **hub nodes** (over-connected artifacts that may be doing too much),
**orphans** (unreferenced artifacts that may be dead or unplanned), **missing parent
links** (an Implementation with no resolvable Concept), and **coverage gaps** (domains
with no artifact). It also emits a generated human-readable index (a wiki) so an
operator and an agent navigate relationships, not raw files. Recall is token-budgeted:
an agent answering an architectural question reads the graph's summary nodes and the
few relevant artifacts, not the whole corpus. This is distinct from the source-code
intelligence graph — it operates on the *design-artifact* layer — but reuses the same
god-node, community, and orphan analyses.

### 5.7 Closed-loop retrospective and delivery metrics

Governance is self-improving (SDG-13). A **level-1** retrospective fires after every
completed phase: a cheap snapshot of artifact health, task throughput, and signal
status. A **level-2** retrospective fires when a plan completes: a deep audit of spec
drift, blocked-task patterns, shadow logic (work done with no governing artifact), and
workflow friction, plus **delivery metrics** borrowed from software-delivery research
— deployment frequency and change-failure rate — applied to the agent-delivery process
itself. The point is to measure planned-versus-built and feed the delta back as
concrete amendments to the artifact layer, closing the loop the project's learning and
self-improvement concepts already open.

### 5.8 Session continuity and finalization

Two mechanics keep long, multi-session autonomous work coherent. **Session
continuity** (SDG-14) maintains a durable live-memory state artifact updated by every
command, and a read-only briefing that reconstructs "where am I, what is blocked, what
is the single next command" without re-deriving it — the next action computed from the
plan's dependency topology and status maturity, then narrated. **Finalization**
(SDG-12) runs at workflow end: detect significant artifact changes, bump the version,
append a release-notes entry, and print a suggested conventional-commit message — and
then stop. The system never stages, commits, or pushes; that decision is always the
human's. This pairs with the project's version-control concept (role-gated commit
authority, quality-gated commit boundary) by supplying the *content* of the suggested
commit without ever taking the commit action.

### 5.9 Two operating profiles

The same engine serves two operators (SDG-15). The **trust** profile takes a
high-level intent, runs the full lifecycle silently, and surfaces only an actionable
outcome and a single go-gate — internal statuses are encapsulated. The **audit**
profile exposes the entire `.design`-style workspace: every artifact, status,
dependency, and rule is inspectable as the structural audit trail. The two are not
different engines with different guarantees; they are two *views* of one engine. This
is what makes the discipline acceptable to both a velocity-first solo builder and a
rigor-first reviewer, and it composes with the project's mission-mode operation ladder
(lite / full / ultra) as the surfacing control.

### 5.10 Ideas-to-adopt mapping

What the reference SDD engine contributes and where it lands in this project.
Mechanics are named structurally, not by product.

| Source mechanic | Idea worth adopting | Where it lands |
| --- | --- | --- |
| Two-layer spec model | Concept/Implementation split with parent-gated promotion; stack migration re-authors only Implementation artifacts. | Already the project's L1/L2 design convention; formalized here as SDG-2 as a *product* governance mechanic, not only a docs convention. |
| Encapsulated lifecycle + trust mode | Auto-promotion through draft→review→approved when non-conflicting; one final go-gate; internal statuses as audit trail. | Mission-mode operation ladder and agent-constitution proactive-drafting; consolidated as SDG-3 / §5.9. |
| Adaptive weight / micro-spec | Ceremony scales to task size; micro-track auto-promoted to full template past a threshold. | Mission-mode lite/full/ultra and the YAGNI decision-ladder; captured as SDG-4. |
| Quarantine cascade | Demoting a Concept recursively blocks dependents and their tasks; downward status change is authoritative. | **New** — no current spec makes foundation-destabilization a first-class, cascading artifact-integrity event. SDG-5. |
| Registry parity | Single index as source of truth; orphans (unregistered / absent / unplanned) are blockers, not warnings. | **New** as a product mechanic — partially echoed by doctor checks; promoted to an invariant as SDG-6. |
| Drift-guard family | Version / rules / tooling / config drift detected against a baseline; halt-and-reconcile with amendment capture. | Doctor self-healing and security config-integrity shields; unified as a *family* under SDG-7. |
| Decision-Record protocol | Decide-and-narrate elective forks; closed escalation whitelist; single ≤3-option question; session-persistent posture. | Agent-constitution (draft-first, ≤2 questions); sharpened with the **New** [DR] one-line grammar and explicit whitelist as SDG-8. |
| Two-stage artifact review | Content critic then instruction-quality pass; ordered; blocking finding reverts status. | Quality-pipeline adversarial review and skill-document semantic analysis; generalized from code to all artifacts as SDG-9. |
| Specification knowledge graph | God-node / orphan / community / coverage analysis over the artifact corpus; generated wiki; token-budgeted recall. | **New** for the design-artifact layer — the code-intelligence graph covers source, not specs. SDG-10. |
| One-way traceability | Product files never reference governance artifacts; provenance lives in commit metadata. | Already a project rule; restated as the portable invariant SDG-11. |
| Finalization protocol | Version bump + changelog + suggested commit message, never an auto-commit. | Version-control concept (role-gated authority); supplies the commit *content* as SDG-12. |
| Two-level retrospective + DORA | Phase snapshot and plan-level deep audit; deployment-frequency and change-failure-rate on the agent process. | Learning-loop and self-improvement concepts; the **New** delivery-metrics framing is SDG-13. |
| Session continuity + status briefing | Durable state artifact; "one next command" resume computed from plan topology. | Session-checkpoint and self-improvement brief surface; consolidated as SDG-14. |

## 7. Drawbacks & Alternatives

- **Over-generalization risk.** A meta-spec can drift into vacuous abstraction.
  Mitigation: it earns its place only through the mechanics flagged **New** in §5.10
  (quarantine cascade, registry parity, artifact knowledge graph, the [DR] grammar,
  DORA-on-agents) and the shared vocabulary; it defers to concrete specs everywhere
  else and links them explicitly.
- **Ceremony tax.** Governance can slow a solo builder. Mitigation: SDG-4 adaptive
  weight and SDG-3 encapsulation exist precisely to keep the common path cheap — the
  full apparatus is opt-in via the audit profile, not imposed on every change.
- **Maintenance coupling.** If a concrete spec changes a concern this document
  summarizes, the summary can go stale. Mitigation: this spec states *principles and
  invariants*, not parameters — concrete thresholds and schemas live in the linked
  specs, so it changes far less often than they do.
- **Alternative — adopt nothing; rely on agent diligence.** Rejected: it is exactly
  how an autonomous swarm goes locally-busy and globally-adrift — unregistered
  artifacts, silently-changed foundations, and a human either over-consulted into
  uselessness or under-consulted into surprise. The mechanics here are the structural
  answer to that failure mode, not optional polish.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DEVFLOW]` | `.design/main/specifications/l1-development-workflow.md` | Authoritative five-stage pipeline that realizes the gates of §5.1. |
| `[TASKGRAPH]` | `.design/main/specifications/l1-task-graph-model.md` | Authoritative decomposition and status lifecycle governed by §5.2. |
| `[QUALITY]` | `.design/main/specifications/l1-quality-standards.md` | Authoritative tiered quality gates enforced by the review of §5.5. |
| `[SKELETON]` | `.design/main/specifications/l1-agent-framework-skeleton.md` | Sibling meta-spec; governs agent anatomy where this governs their artifacts. |

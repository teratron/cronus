# Perspective Model (Theory of Mind)

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

A representation of *who knows or believes what about whom*. Where the user model captures the agent's picture of the person's preferences and traits from one fixed vantage, this concept generalizes representation to a **perspectival** one: every belief is held *by an observer about a subject* and keyed by that `(observer → subject)` pair. When observer and subject coincide it is self-knowledge; when they differ it is a model of what the observer knows or believes about the subject — a *theory of mind*. The same mechanism serves both, and it captures *defeasible belief*, not asserted truth.

For Cronus this is deliberately bounded to two uses, not an open social graph: (a) the agent's model of the **user** — their preferences and traits (the existing user model) *plus the user's epistemic state*: what the user already knows, understands, or misunderstands — so the office communicates at the right level for a possibly-non-technical client; and (b) the orchestrator's model of each **worker agent's** knowledge and context — so it briefs and delegates without duplicating what an agent already holds. The user model is the special case of this concept with `observer = agent, subject = user`.

## Related Specifications

- [l1-user-model.md](l1-user-model.md) - The single-perspective special case (observer=agent, subject=user, preference/trait facets); this concept generalizes it to `(observer → subject)` pairs and adds the epistemic-state facet.
- [l1-operational-ledger.md](l1-operational-ledger.md) - Asserted ground-truth facts; a perspectival belief is the opposite kind — defeasible, perspectival, possibly wrong (PM-2).
- [l1-orchestration.md](l1-orchestration.md) - ORC-8 synchronization-without-duplication is realized by modeling what each agent already knows (PM-6).
- [l1-memory-model.md](l1-memory-model.md) - The scoped store the representations persist in; derivation reuses its consolidation machinery.
- [l2-memory-store.md](l2-memory-store.md) - Derivation (distill/reconcile) and storage of conclusions with their evidence (PM-3).
- [l1-security.md](l1-security.md) - On-device, consent-gated, inspectable/erasable representation, including the agent's model of the user's beliefs (PM-7).

## 1. Motivation

Two real needs in Cronus are not met by a flat, single-perspective model of the user:

- **Communicating at the right level for a non-technical client.** The office's whole premise is that the client need not be technical. To do that well, the agent must track not only what the user *prefers* but what the user *knows*: re-explaining what they already understand wastes their attention and reads as condescension; assuming knowledge they lack produces confusion; leaving a recorded misconception uncorrected compounds error. This is a model of the user's *epistemic state* — a theory of mind about the user — which the preference/trait-focused user model does not hold.
- **Coordinating agents without duplication.** Orchestration already requires that synchronization "produces shared state, never duplicated work" (ORC-8). Achieving that requires the orchestrator to model *what each worker already knows or has context on* — otherwise it re-briefs an agent on what it already holds, or fails to brief one that is missing context. That is a theory of mind about each agent, currently assumed but never modeled.

Both are the same underlying capability: a representation indexed by *who holds the belief* and *about whom*. Generalizing the user model to that `(observer → subject)` keying captures both, and adds a discipline the flat model lacks — a perspectival belief is explicitly *not* asserted truth (the observer can be wrong), so it composes safely with the operational ledger rather than letting soft beliefs harden into facts. The invariants below capture this while bounding it to Cronus's two uses, so it does not sprawl into a general social-belief graph.

## 2. Constraints & Assumptions

- A perspectival representation is a *subject model* layered on the memory store — not a new storage engine (it reuses the memory/derivation machinery).
- Every belief is defeasible and evidence-backed; nothing in it is incontrovertible fact.
- The observer set is bounded to the office's principals (the user) and its agents — this is **not** an open graph of arbitrary parties' beliefs about each other.
- Building and applying any representation is consent-gated, on-device, and fully inspectable/erasable by the user.
- The user model is subsumed as the `observer=agent, subject=user` instance; this concept does not replace it, it generalizes it.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **PM-1 (Perspectival, keyed by observer→subject):** a representation is held *by an observer about a subject* and keyed by the `(observer, subject)` pair. `observer == subject` is self-knowledge; `observer ≠ subject` is a model of what the observer knows/believes about the subject (theory of mind). One mechanism serves both; the perspective is always explicit, never an unattributed global belief.
- **PM-2 (Belief, not asserted truth):** a perspectival representation records the observer's **defeasible belief** — evidence-backed, confidence-scored, revisable — never asserted ground truth. It is a distinct kind from the operational ledger (asserted facts), may be wrong, and is revised or decays as evidence changes. A belief MUST NOT be promoted to a ledger fact without an explicit, evidenced assertion step.
- **PM-3 (Reasoning-first — conclusions with evidence):** the representation holds *conclusions* derived (deductively and inductively) from the subject's messages and events, not merely retrieved raw fragments. Derivation runs as a background process; each conclusion retains the evidence that supports it so it is auditable and revisable (composes with the memory-store distill/reconcile stages).
- **PM-4 (Bounded observer set — two uses, not a social graph):** Cronus maintains exactly two classes of perspectival representation: (a) the agent's model of the **user** (preferences/traits — the user model — *plus the user's epistemic state*), and (b) the orchestrator's model of each **worker agent's** knowledge/context. It MUST NOT build an open graph of many parties' beliefs about each other; the observer/subject set is the office's principals and agents, scoped to these uses.
- **PM-5 (User-knowledge modeling → right-level communication):** the agent's model of *what the user already knows, understands, or misunderstands* is used to calibrate communication — not re-explaining the known, surfacing the unknown, gently correcting a recorded misconception. This serves the user (especially a non-technical client) and MUST NOT be used to manipulate, persuade against interest, or condescend (inherits the service-not-manipulation boundary).
- **PM-6 (Agent-knowledge modeling → non-duplicating coordination):** the orchestrator's model of *what each worker already knows or has context on* is used to brief and delegate without duplication — the mechanism that realizes ORC-8 (shared state, never duplicated work) and avoids re-grounding an agent in what it already holds. A briefing SHOULD carry what the agent lacks, not what it already has.
- **PM-7 (Privacy, consent, inspect/erase):** every perspectival representation is on-device, principal-scoped, consent-gated, and fully inspectable and erasable by the user — **including the agent's model of the user's own knowledge and beliefs**. No representation is egressed without explicit consent; modeling depth is configurable, trading cost and intrusiveness against quality.
- **PM-8 (Bounded, observable application):** only the perspective slice relevant to the current task is surfaced into context (the right `observer → subject` view), never a whole representation; and when a representation influences behavior — e.g. skipping an explanation because the user is modeled as already knowing it — that influence is observable and auditable, so the user can tell when and why perspective-taking occurred.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The (observer → subject) keying

| observer | subject | What it is | In-scope use |
| --- | --- | --- | --- |
| agent | user | preferences/traits + the user's epistemic state | right-level communication (PM-5); the user model is the preference/trait slice |
| orchestrator | worker agent | what that agent knows / has context on | non-duplicating briefing & delegation (PM-6) |
| any | itself (`observer==subject`) | self-knowledge | self-representation (the degenerate, single-perspective case) |

The keying is the whole idea: the same representation machinery, indexed by *whose belief* and *about whom*. Cronus instantiates only the rows above (PM-4).

### 4.2 Belief vs fact

```text
[REFERENCE]
operational ledger : asserted, single-truth, citable, supersede-don't-mutate   (l1-operational-ledger)
perspective model  : "observer believes X about subject", confidence c, evidence E, revisable   (this spec)
promotion          : a belief becomes a ledger fact ONLY via an explicit, evidenced assertion — never silently (PM-2)
```

A representation can be wrong without corrupting ground truth, because the two are different kinds. This is the user model's non-authoritative principle, generalized to every perspective.

### 4.3 Derivation (reasoning-first)

```text
[REFERENCE]
on new messages/events about a subject (background, consent-gated):
    conclusions := derive(subject_messages)        // deductive + inductive (PM-3)
    for c in conclusions:
        merge(observer, subject, c) with confidence + evidence   // PM-1/PM-2
        // for observer=agent,subject=user also tag epistemic facets: knows / unsure / misunderstands (PM-5)
query(observer, subject) -> conclusions (+ evidence), or a representation snapshot, or a natural-language answer
```

Derivation reuses the memory-store distill/reconcile machinery; this concept adds the perspective key and the epistemic-state facet, not a parallel pipeline.

### 4.4 The two applications

- **Right-level communication (PM-5).** Before explaining, the agent consults its model of the user's epistemic state: skip what is modeled as known, expand what is modeled as unknown, correct what is modeled as misunderstood — and record that it did so (PM-8).
- **Non-duplicating coordination (PM-6).** Before briefing a worker, the orchestrator consults its model of that worker's knowledge and includes only the delta — what the worker lacks — realizing ORC-8.

### 4.5 Relationship to the user model

The user model is this concept's `observer=agent, subject=user` instance, restricted to preference/trait facets. This spec does not amend or replace it; it (a) generalizes the keying to other `(observer, subject)` pairs (the agent↔agent case) and (b) adds the user's *epistemic-state* facet to the user-row. Implementations MAY realize the user model as the concrete store and extend it with the epistemic facet and the agent-subject rows.

## 5. Drawbacks & Alternatives

- **Inference error / creepiness.** A wrong or over-confident belief about what the user knows degrades trust. Mitigated by PM-2 (defeasible, evidenced), PM-7 (inspect/erase, including the belief model), and PM-8 (observable application). The user is always the authority on themselves.
- **Scope creep into a social belief graph.** Modeling "who believes what about whom" can sprawl. Mitigated structurally by PM-4 — exactly two bounded uses, the office's principals and agents, never an open graph.
- **Derivation cost.** Reasoning-first conclusions cost more than storing chunks. Mitigated by background, consent-gated, configurable-depth derivation (PM-3, PM-7), reusing existing memory machinery rather than a new pipeline.
- **Alternative — rely on the user model alone.** Rejected: it is single-perspective and preference-focused; it cannot model the user's *knowledge* (for right-level communication) nor any agent's knowledge (for non-duplicating coordination).
- **Alternative — treat beliefs as operational-ledger facts.** Rejected: a perspectival belief is defeasible and possibly wrong; recording it as asserted truth lets soft inferences masquerade as fact (violates OL semantics) — PM-2 keeps them distinct.
- **Alternative — full multi-peer social theory of mind.** Rejected for Cronus: an open graph of arbitrary parties' beliefs is a social-platform feature beyond a personal office; PM-4 bounds the concept to the two uses that earn their place here.

## nodus-relevance mapping

Largely a main-workspace concept; the portable runtime touches it only as a participant.

| Element | nodus seam | Note |
| --- | --- | --- |
| Perspectival key (PM-1) | per-participant state tagged `(observer, subject)` in `StorageProvider` | A multi-participant workflow can hold a per-peer view; bounded by PM-4. |
| Belief vs fact (PM-2) | belief state distinct from asserted run facts on the audit stream | Defeasible step-derived conclusions never overwrite asserted run outputs. |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[USER-MODEL]` | `.design/main/specifications/l1-user-model.md` | The single-perspective special case this concept generalizes. |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | Asserted-fact kind a perspectival belief is deliberately distinct from. |
| `[ORCH]` | `.design/main/specifications/l1-orchestration.md` | ORC-8 non-duplication realized by agent-knowledge modeling (PM-6). |
| `[MEMORY]` | `.design/main/specifications/l2-memory-store.md` | Derivation/storage of conclusions with evidence (PM-3). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-27 | Core Team | Initial spec — perspectival representation / theory of mind: beliefs keyed by (observer→subject) with self==self degenerate case (PM-1), belief-not-asserted-truth (PM-2), reasoning-first conclusions-with-evidence (PM-3), bounded observer set — two uses not a social graph (PM-4), user-knowledge modeling for right-level communication (PM-5), agent-knowledge modeling for non-duplicating coordination realizing ORC-8 (PM-6), privacy/consent/inspect/erase incl. the belief model (PM-7), bounded observable application (PM-8); generalizes l1-user-model (its observer=agent/subject=user special case) and adds the epistemic-state facet; distinct from operational-ledger by the belief-vs-fact boundary; nodus-relevance mapping. Mined from an external peer-representation memory engine. |

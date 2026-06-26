# User Model

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The agent's evolving, persistent representation of the *person* it serves — their stable preferences, recurring goals, working style, domain context, and relationships — built up across sessions and used to personalize how the agent helps. It is a distinct knowledge kind: not the conversation memory (what was said), not the operational ledger (exact ground truth for dispatch), and not knowledge-base documents (external reference material). It answers "who is this user and how do they want to be served," and every belief in it is an inference the user can see, correct, or erase.

## Related Specifications

- [l1-memory-model.md](l1-memory-model.md) - Memory scoping/lifecycle; the user model persists in the user/global memory scope.
- [l2-memory-store.md](l2-memory-store.md) - The store the model is derived from and persisted in.
- [l1-operational-ledger.md](l1-operational-ledger.md) - Exact operational facts; the user model is inferred and provisional, the ledger is asserted ground truth — different kinds.
- [l1-practice-analytics.md](l1-practice-analytics.md) - Analyzes the user's *practice* (how they work); the user model captures *who they are* — complementary.
- [l1-security.md](l1-security.md) - Consent, on-device storage, and inspect/erase rights for the model.

## 1. Motivation

A personal agent that forgets who you are every session cannot compound. Re-explaining your preferences, your projects, your tone, and your constraints on every interaction is the coordination tax of a stateless assistant. Conversation memory helps within a thread, and the operational ledger holds exact facts, but neither captures the durable, inferred picture of the person: that they prefer terse answers, work mostly in Rust, dislike being asked before small edits, are preparing a launch this quarter.

Modeling that explicitly lets the agent anticipate and personalize — but it is also the most sensitive thing the agent holds, and the easiest to get subtly wrong. So the model must be evidence-backed and provisional (never asserted as fact), correctable by the user, privacy-first (on-device, inspectable, erasable), consent-gated, and strictly in service of the user — never a lever for manipulation. Those constraints are what separate a helpful user model from a creepy or harmful one.

## 2. Constraints & Assumptions

- The user model is a *subject model* about a person, layered on top of the memory store — it is not a new storage engine.
- Every attribute is inferred or stated; nothing in the model is treated as incontrovertible fact.
- The model is per-principal and private by default; it is never required — the agent must function (less personally) without it.
- Building and applying the model is governed by user consent and is fully inspectable.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **UM-1 (Distinct subject model):** the user model represents the *person* — stable preferences, goals, traits, working style, domain context, relationships — and is distinct from conversation memory (events), the operational ledger (asserted facts), and knowledge collections (documents). It answers "who is this user," not "what happened."
- **UM-2 (Inferred & non-authoritative):** every attribute is evidence-backed, confidence-scored, and provisional — never asserted as truth. The model records the evidence behind a belief and revises or decays it as evidence changes.
- **UM-3 (Cross-session accretion, anti-drift):** the model persists and deepens across sessions; a single session refines it incrementally but MUST NOT wholesale-rewrite it without supporting evidence, so one anomalous interaction cannot redefine the person.
- **UM-4 (Explicit overrides inferred):** a user-stated preference takes precedence over an inferred one. The model records provenance (stated vs inferred) and resolves conflicts in favor of the explicit, most-recent statement.
- **UM-5 (Privacy-first, principal-scoped):** the model is bound to a principal, stored on-device, and never egressed without explicit consent (consistent with SEC-3). The user can inspect the full model (see exactly what the agent believes about them) and erase any part of it.
- **UM-6 (Bounded, observable application):** the model personalizes behavior via bounded selection — only attributes relevant to the current task are surfaced into context, never the whole profile — and its influence is observable, so the user can tell when and why personalization occurred.
- **UM-7 (Consent-gated formation):** building a persistent user model is opt-in; with it disabled the agent operates without a stored profile. Modeling depth (how much inference effort is spent) is configurable, trading cost and intrusiveness against personalization quality.
- **UM-8 (Service-not-manipulation boundary):** the model informs *service* — better, more anticipatory help — and MUST NOT be used to manipulate, persuade against the user's interest, or upsell. Personalization that works against the user's stated interests is a violation, not a feature.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 Attribute Shape

```text
[REFERENCE]
UserAttribute {
  subject     : principal
  facet       : "preference" | "goal" | "trait" | "style" | "context" | "relationship"
  statement   : string          // e.g. "prefers terse answers"
  provenance  : "stated" | "inferred"
  confidence  : float           // 0..1, revisable (UM-2)
  evidence    : ref[]           // pointers to the interactions that support it
  updated     : timestamp
}
```

### 4.2 Formation & Revision

```text
[REFERENCE]
on interaction (if consented, UM-7):
    candidates := distill(interaction)              // inferred attributes
    for c in candidates:
        merge(c) with confidence + evidence         // UM-2; incremental, not wholesale (UM-3)
on user statement:
    upsert(attribute, provenance="stated")          // explicit wins (UM-4)
```

Confidence rises with corroborating evidence and decays without it; a contradicting user statement immediately supersedes an inferred attribute.

### 4.3 Application

When composing a response, the agent selects only the attributes relevant to the task (UM-6) — e.g. tone preference for phrasing, domain context for examples — and records that it personalized. The full model is never dumped into context.

### 4.4 Inspection & Erasure

The user can view the complete model with provenance and confidence, correct any attribute (which becomes `stated`), and erase any facet or the whole model (UM-5). Erasure removes the derived attributes; the underlying raw memory follows its own lifecycle.

## 5. Drawbacks & Alternatives

- **Inference error & creepiness:** a wrong or over-confident belief degrades trust; mitigated by UM-2 (provisional + evidence), UM-4 (explicit wins), and UM-5 (inspect/erase). The user is always the authority on themselves.
- **Privacy surface:** a rich user model is sensitive; mitigated by on-device storage, consent-gating (UM-7), and no egress without consent (UM-5).
- **Alternative — rely on conversation memory only:** rejected; memory captures events, not the durable, queryable picture of the person, and re-derives preferences from scratch each time.
- **Alternative — store the model as plain operational-ledger facts:** rejected; the ledger asserts ground truth, while the user model is explicitly inferred and provisional — conflating them would let soft inferences masquerade as hard facts (violates OL semantics).

## Canonical References

| Alias | Path | Purpose |
|---|---|---|
| `[MEMORY]` | `.design/main/specifications/l2-memory-store.md` | Store the model is derived from and persisted in. |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | Asserted-fact kind the user model is deliberately distinct from. |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | Consent, on-device storage, and inspect/erase rights. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-26 | Core Team | Initial spec — persistent user model: distinct subject-model kind, inferred & non-authoritative, cross-session anti-drift accretion, explicit-overrides-inferred, privacy-first inspect/erase, bounded observable application, consent-gated formation, service-not-manipulation boundary (UM-1…UM-8). |

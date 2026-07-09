# Action Gating

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

When an agent takes real-world actions — sending an email to a client, moving a file, booking a meeting, changing a permission, spending money — some of those actions are safe and reversible and some are consequential and irreversible. This spec names the discipline that decides **how much authorization friction each action must pass through**: gating that is **proportional to consequence**. A read-only, reversible, in-boundary action executes immediately; a write or an externally-visible action takes a single lightweight confirmation; an irreversible, high-value, or authority-changing action takes a full approval.

The load-bearing insight is that **uniform friction is the failure mode in both directions**. Gate nothing and dangerous actions slip through. Gate everything and the approver is trained to rubber-stamp — a blanket-approval habit that silently nullifies the gate on the actions that actually mattered. Proportional gating spends the approver's scarce attention only where the consequence warrants it: safe actions stay frictionless, and the friction rises, in a closed ordered ladder, exactly as the blast radius, irreversibility, external visibility, and value at stake rise. This spec is the *how-much-friction* contract; the enforcement mechanism (the runtime guard, the approval gate) and the config governance around it are owned by their own specs.

## Related Specifications

- [l2-tool-security.md](l2-tool-security.md) — the runtime guard mechanics (allow / escalate-to-approval / hard-block) that **enforce** the tier this spec assigns; l2-tool-security is a threat guard (block malicious), this L1 is the friction calibrator (how much gate a *legitimate* action warrants). They compose: the guard hard-blocks threats, action-gating tiers the rest.
- [l1-interception-model.md](l1-interception-model.md) — the decide-class seam (INT-1) that intercepts an effect before it happens and enforces the assigned tier; INT-3 fail-closed is AG-4's fail-safe-to-friction.
- [l1-policy-governance.md](l1-policy-governance.md) — the consequence→tier thresholds are administrator-governable (AG-8, composing PG-4/PG-6); weakening the gate below its floor is a governed escape-hatch.
- [l1-security.md](l1-security.md) — SEC-3 default-deny and the egress gate (external-visibility axis, AG-2/AG-4); SEC-9 learnable permission (AG-6 de-escalation); SEC-10 human-rooted authority (AG-6/AG-8).
- [l2-orchestration.md](l2-orchestration.md) — the approval gate the top tier routes into.
- [l1-operational-ledger.md](l1-operational-ledger.md) — the auditable record of each gate decision, tier, and approver (AG-7).
- [l2-agent-autonomy.md](l2-agent-autonomy.md) — durable allow-rules are the concrete de-escalation mechanism (AG-6).
- [../../nodus/specifications/l1-nodus-portability.md](../../nodus/specifications/l1-nodus-portability.md) — LP-16 effect risk-class declaration is the nodus-workflow realization: an effectful step declares its consequence descriptors so a host graduated gate can tier it.

## 1. Motivation

Two opposite mistakes make an agent either dangerous or useless. The dangerous one gates too little: it lets the agent send an irreversible external email, or grant a permission, or spend money, with the same zero friction as reading a file — so a hallucinated or hijacked step does real, unrecoverable damage before anyone sees it. The useless one gates too much: it asks the human to approve *everything*, including reading a document and rephrasing a note — so the human, facing a hundred approvals a day, stops reading them and clicks "approve" reflexively. The second mistake is subtler and more corrosive, because it *feels* safe (there's a gate!) while actually being unsafe (nobody reads it) — the gate on the one dangerous action in the hundred is rubber-stamped along with the ninety-nine trivial ones.

Proportional gating is the resolution. Classify each action by its real consequence — can it be undone, whose state does it touch, does it leave the trust boundary, what value is at stake — and match the friction to that consequence. The frictionless majority (safe reversible reads and in-boundary work) runs immediately, so the human's attention is not spent there. The rare consequential action (irreversible, external, high-value, authority-changing) is gated heavily, so the human's attention lands exactly where it matters and is not yet exhausted. And because a proven-safe action can earn its way to a lower tier over time (never the reverse), the friction keeps shrinking toward only what genuinely needs a human.

## 2. Constraints & Assumptions

- Gating decides **how much friction**, not **whether an action is a threat** — malicious actions are hard-blocked by the security guard regardless of tier; gating calibrates friction for *legitimate* actions.
- The consequence classification must be **legible** — a human can see why an action landed in its tier; an opaque score alone is insufficient.
- The safe default for an unclassifiable action is **more** friction, never less (fail-safe-to-friction).
- This spec owns the friction-calibration contract; the gate mechanics, the approval UI, and the config governance are owned by tool-security, orchestration, and policy-governance respectively.
- Layer 1: it names no concrete threshold, score, or approval UI. The tier thresholds and mechanics are Layer-2 / governable.

## 3. Core Invariants

Rules every Layer 2 realization MUST NOT violate. They are technology-neutral.

- **AG-1 (Friction proportional to consequence):** the authorization friction on an action MUST be **proportional to its consequence**. A safe, reversible, in-boundary action passes no gate; a write or externally-visible action takes a lightweight confirmation; an irreversible, high-value, or authority-changing action takes a full approval. **Uniform friction — gating everything or gating nothing — is the failure this forbids.**

- **AG-2 (Consequence classified by explicit, legible axes):** an action's tier is derived from **explicit consequence axes** — **reversibility** (can it be undone?), **blast radius** (self / shared / others' state), **external visibility** (does it cross the trust boundary?), and **value at stake** (money, credentials, permissions). The classification is **legible and attributable** — a human can see *which axes* put an action in its tier — not an opaque number alone.

- **AG-3 (Closed, ordered tier ladder):** gate tiers form a **closed, ordered ladder** of increasing friction — at minimum **auto** (execute immediately), **confirm** (one lightweight acknowledgement), and **approval** (an explicit authorization by an authorized principal, possibly out-of-band). Higher tiers strictly dominate lower; an action cannot reach a lower-friction path by re-routing around its tier.

- **AG-4 (Read-safe frictionless, irreversible always gated, unknown fails to friction):** a purely read-only, side-effect-free, in-boundary action is **auto** by default — friction there is pure cost. An **irreversible** action, or one that **crosses the trust boundary** or **puts value at stake**, is **never auto**: it takes at least confirm, and by AG-1 usually approval. An **unclassifiable** action defaults to the **higher** tier — fail-safe-to-friction, never fail-open.

- **AG-5 (Friction-fatigue is a first-class failure):** **over-gating is a defect, not caution.** Gating everything trains the approver to rubber-stamp, which nullifies the gate on the actions that mattered — a blanket-approval habit is *worse* than calibrated gating because it *looks* safe while being unsafe. The tier ladder exists precisely to spend the approver's attention only where consequence warrants; a realization that gates indiscriminately violates this contract as surely as one that gates too little.

- **AG-6 (Learned de-escalation, never self-escalation-bypass):** a repeatedly-approved, proven-safe action MAY be **de-escalated** to a lower tier through an explicit, **scoped, revocable, human-ratified** allow-rule (composing SEC-9), so friction shrinks over time toward only what still needs a human. But the ladder is **never bypassed upward**: an action can **never lower its own tier**, a de-escalation is always a human-rooted grant (SEC-10), and a hard-forbidden or top-tier-floored action is **non-de-escalatable** (fail-closed). Trust is earned downward, never seized.

- **AG-7 (Every gate decision is auditable):** each gated action records the **tier assigned**, the **consequence axes** that placed it there, the **decision** (auto / confirmed-by-whom / approved-by-whom / denied), and the outcome — so "why did this need approval" and "who authorized this" are always answerable (composing the operational ledger and the interception observe-after). A silent gate, or one whose tier rationale is unrecorded, is a defect.

- **AG-8 (Governable stricter, never silently laxer):** the mapping from consequence to tier — which axes count, the thresholds, the per-tier friction — is **administrator-governable** (composing policy-governance): an operator can **raise** friction. But the gate is a **safety-reducing escape hatch when weakened**, so lowering it below the safe floor is itself a governed act, and the top tier (approval for irreversible / high-value / authority-changing actions) has a floor **no lower tier of policy can silently remove**. You can make gating stricter, never quietly laxer.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The tier ladder

| Tier | For | Friction | Examples |
| --- | --- | --- | --- |
| **auto** | reversible, in-boundary, no value at stake | none — executes immediately | read a file, search, draft (unsent), in-scratch edit |
| **confirm** | writes, externally-visible, recoverable | one lightweight acknowledgement | send an internal message, create a calendar hold, write a shared doc |
| **approval** | irreversible, high-value, authority-changing, others' state | explicit authorization by an authorized principal | email an external client, spend money, grant a permission, delete shared data |

The ladder is closed and ordered (AG-3); an action's tier is the **highest** any of its consequence axes demands (AG-2) — one high-value axis lifts the whole action to approval regardless of the others.

### 4.2 Classifying an action

```text
[REFERENCE]
tier(action):
    if not classifiable(action):                 return APPROVAL      // AG-4 fail-safe-to-friction
    t := AUTO
    if action.writes or action.external_visible:  t := max(t, CONFIRM) // AG-2
    if not action.reversible:                     t := max(t, APPROVAL)
    if action.crosses_trust_boundary:             t := max(t, APPROVAL) // e.g. external send
    if action.value_at_stake(money|creds|perms):  t := max(t, APPROVAL)
    t := apply_allow_rules(action, t)             // AG-6: de-escalate only, human-ratified, scoped
    t := max(t, governance_floor(action))          // AG-8: policy may raise, never silently lower
    return t
```

The tier is the max across axes (AG-2), de-escalated only by a human-ratified allow-rule (AG-6), then floored by governance (AG-8). Every branch is recorded (AG-7).

### 4.3 Why friction-fatigue is the load-bearing rule

The subtle failure this spec guards against is not too little gating — that is obvious and everyone builds a gate. It is **too much**. A system that asks for approval on every action produces an approver who has stopped reading, so the gate that fires on the one dangerous action in a hundred is clicked through with the ninety-nine trivial ones. Such a system passes a naive audit ("there's an approval gate!") while being *less* safe than one with no gate, because it manufactures false confidence. AG-5 makes over-gating a defect precisely so a realization cannot buy the appearance of safety with friction that destroys the attention the gate depends on.

## 5. Drawbacks & Alternatives

**Alternative: gate every action (max safety).** Rejected by AG-5 — it manufactures rubber-stamping and nullifies the gate on what matters; calibrated friction is what keeps approvals meaningful.

**Alternative: gate nothing an allow-rule permits.** Rejected by AG-6 — de-escalation must stay scoped, revocable, and human-ratified, and top-tier actions are non-de-escalatable; blanket self-permitting is the dangerous extreme.

**Alternative: a single opaque risk score.** Rejected by AG-2 — a number alone is not legible; the human must see *which* consequence put an action in its tier to trust and tune the gate.

**Risk: mis-classification.** An action mis-tiered too low is dangerous. Mitigation: AG-4 fails unclassifiable actions to the higher tier, and AG-8 lets an operator raise thresholds — the calibration errs toward friction where uncertain.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[GUARD]` | `.design/main/specifications/l2-tool-security.md` | The runtime guard that enforces the assigned tier (allow / escalate / hard-block) |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | Egress/default-deny (AG-4), SEC-9 de-escalation (AG-6), SEC-10 authority (AG-8) |
| `[GOVERNANCE]` | `.design/main/specifications/l1-policy-governance.md` | The governable thresholds and the un-disable-able floor (AG-8) |
| `[NODUS]` | `.design/nodus/specifications/l1-nodus-portability.md` | The host-neutral realization: LP-16 effect risk-class declaration |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-09 | Core Team | Initial stable spec — action gating: authorization friction proportional to an action's consequence. Friction proportional to consequence, uniform friction forbidden (AG-1); consequence classified by explicit legible axes — reversibility, blast radius, external visibility, value at stake (AG-2); closed ordered tier ladder auto/confirm/approval (AG-3); read-safe frictionless, irreversible always gated, unknown fails to friction (AG-4); friction-fatigue a first-class failure — over-gating is a defect not caution (AG-5); learned scoped revocable human-ratified de-escalation, never self-escalation-bypass, top tier non-de-escalatable (AG-6); every gate decision auditable with tier + axes + approver (AG-7); governable stricter never silently laxer, top-tier floor un-removable (AG-8). Composes l2-tool-security / l1-interception-model / l1-policy-governance / l1-security / l2-orchestration / l2-agent-autonomy. Distilled from an adoption pass over an external business-operations agent-harness reference (three-tier none/confirm/approval execution gates by action risk). |

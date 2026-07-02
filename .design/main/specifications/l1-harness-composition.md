# Harness Composition

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Every existing harness concept in this project *grows* the harness: engineering adds
an evidence-backed change, optimization searches for a better configuration,
co-evaluation measures a pair. None of them *trims* it. Yet the most common failure of
a real agent harness is not that it is too weak — it is that it is **bloated**: too
many overlapping specialists, skills that duplicate what the base model now does
unaided, hooks that guard against a host feature that already guards itself, delegation
chains so deep they lose the thread. Bloat is not neutral surplus; it costs tokens,
latency, and clarity, and it hides the components that actually matter.

This spec defines the missing direction: **harness composition** — assembling and
maintaining the **minimal, continuously-justified** set of components (roles, skills,
hooks, rules, workflow steps and macros) that a given project or office actually needs.
Its central claim: **every component exists to compensate for a specific capability gap
— something the base model or the host runtime cannot do adequately on its own — and a
component whose gap has closed is dead weight to be pruned.** Because models and hosts
improve, justification is not a one-time decision at creation; it is continuous. It is
the right-sizing / anti-bloat counterpart to `l1-harness-engineering`.

## Related Specifications

- [l1-harness-engineering.md](l1-harness-engineering.md) — the *grow* counterpart (evidence-backed amendment HE-4); this spec is the *trim* counterpart. A harness is right when growth and pruning are both disciplined.
- [l1-harness-optimization.md](l1-harness-optimization.md) — searches the candidate space; a smaller, right-sized candidate is cheaper to evaluate and less prone to overfit.
- [l1-agent-coevaluation.md](l1-agent-coevaluation.md) — measures whether a pruned/added component actually moved a slice, gating a composition change honestly.
- [l1-roles.md](l1-roles.md) — ROL-9 anti-sprawl justification gate is the role-level application of HC-5; hire/fire (ROL-3/ROL-4) is the mechanism, this spec supplies the *criteria*.
- [l1-orchestration.md](l1-orchestration.md) — delegation topology (ORC-2); HC-6 bounds its depth.
- [l1-extensions.md](l1-extensions.md) — the component registry; HC-4 dedup against host-native and existing extensions.
- [l1-office-model.md](l1-office-model.md) — the office this composition right-sizes; a small project gets a small office.

## 1. Motivation

A harness component is never free. An extra specialist role fragments the workforce and
dilutes delegation; an extra skill is one more trigger to mis-fire; an extra hook is
latency on every action; an extra rule is cognitive load in every prompt. The reason
these accumulate is that **adding is easy and removing is scary** — nobody is sure a
component is safe to delete, so it stays forever, long after its reason to exist has
evaporated.

Two forces evaporate a component's justification, and neither is visible without
looking for it:

1. **The model improves.** A component encodes an assumption about what the model
   *cannot* do — a planning scaffold, a self-verification loop, a "think step by step"
   nudge. As the base model gets better at planning, self-correction, and long context,
   that assumption goes stale and the component becomes a no-op that still costs.
2. **The host improves.** A custom guard, command, or memory shim compensates for a
   capability the host runtime lacked — until the host ships it natively. Now the custom
   component *duplicates* a native feature, adding maintenance and a second, divergent
   source of truth.

Without a discipline that *continuously re-justifies* every component against current
model and host capability, a harness only ever grows. This spec makes pruning a
first-class, safe, auditable activity, and makes "minimal and justified" the default
shape rather than a lucky accident.

## 2. Constraints & Assumptions

- Composition is orthogonal to quality-of-a-component (`l1-harness-engineering` evolves
  a component's content; this decides *whether it should exist at all*) and to search
  (`l1-harness-optimization` explores configurations; this bounds the space to justified
  ones).
- "Justified" is defined against a *named capability gap*; a component with no nameable
  gap it closes is unjustified regardless of how well-written it is.
- Pruning is safe by construction: a removed component's knowledge is archived, never
  destroyed (consistent with non-destructive role release, ROL-4 / MEM-5), so a wrong
  removal is reversible.
- This is an authoring/maintenance discipline over a single office/harness on-device;
  it is not a distributed-fleet concern (INV-8).

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **HC-1 Minimal justified composition**: a harness includes only components that are
  justified; the default is to include nothing, and each component earns its place.
  Bloat — a component that does not earn its place — is a first-class defect, not a
  neutral surplus. A larger harness is a cost to be justified, never a default virtue.

- **HC-2 Component = capability-gap compensation**: every component exists to compensate
  for a specific, nameable gap — something the base model or the host runtime cannot do
  adequately on its own. That named gap is the component's justification; a component
  with no gap it demonstrably closes is unjustified and MUST NOT be added (or MUST be
  removed).

- **HC-3 Continuous re-justification (prune when the gap closes)**: a component's
  justifying gap can close over time — the base model improves and now handles the task
  unaided, or the host runtime adds a native capability that supersedes it. The harness
  is periodically re-evaluated against **current** model and host capability; a
  component whose gap has closed is dead weight and is pruned. Justification is
  continuous, not one-time at creation.

- **HC-4 Dedup against host-native and existing components**: a component MUST NOT
  duplicate a capability the host runtime provides natively, nor duplicate an existing
  component. Adding is preceded by a check against host built-ins (and their evolution)
  and the current component set; when a host feature comes to supersede a custom
  component, the custom one is removed with the superseding native feature named.

- **HC-5 Anti-sprawl justification gate**: adding a *specialist* component (a new role,
  agent, or narrowly-scoped skill) requires it to justify itself on at least a threshold
  number of **independent** axes — for example distinct expertise, a parallelism
  benefit, context isolation, or genuine reuse. One weak reason is insufficient; reusing
  or extending an existing component is preferred when the gate is not cleared. This
  prevents fragmentation into many thin, overlapping specialists.

- **HC-6 Bounded composition complexity**: composition complexity is bounded.
  Delegation/nesting depth is capped (a deep chain loses context and compounds errors),
  and component count is proportional to the subject's real scale — a small project gets
  a small harness. An unbounded delegation chain or a maximal fixed component catalog
  applied regardless of scale violates this invariant.

- **HC-7 Right-size to the subject**: the composed harness is sized to the actual
  project or office it serves. Components are selected by the subject's real traits from
  a catalog of patterns, not shipped as a one-size-fits-all maximal set. Two different
  subjects get two different harnesses; the same maximal set for both is a composition
  failure.

- **HC-8 Deferred-with-trigger, not speculative**: a pattern or component that does not
  apply *yet* is not added speculatively. It is recorded as **deferred** with an
  explicit **activation trigger** — the condition under which it becomes justified
  (e.g. "add a verification loop when a test suite exists"). Speculative inclusion is
  bloat (HC-1); deferral preserves the idea at zero standing cost until its trigger
  fires.

- **HC-9 Lifecycle disposition with reasons, safe removal**: maintenance dispositions
  every component into keep / update / add / remove, each with a stated reason (a
  removal names the closed gap or the superseding feature). Removal is safe and
  reversible — a removed component's knowledge is archived, not destroyed (ROL-4). A
  component is never silently dropped nor silently retained; both retention and removal
  are decisions with reasons.

- **HC-10 Composition is observable and auditable**: each included component's
  justification, the deferred set with its triggers, and every add/keep/update/remove
  decision with its reason are recorded, so the harness's shape is explainable and its
  drift toward bloat is detectable from the record rather than only felt as slowness.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The Composition Loop

```text
[REFERENCE]
compose(subject):                                   // create
    gaps := analyze(subject)                         // what the model/host can't do here
    candidates := select_from_catalog(subject.traits)   // HC-7 right-size by traits
    for c in candidates:
        if duplicates_host_native(c) or duplicates_existing(c):  skip        // HC-4
        if c is specialist and not justified_on(c, ≥N axes):     skip        // HC-5
        if c.gap ∈ gaps:  include(c, justification=c.gap)                    // HC-1/HC-2
        else:             defer(c, trigger=activation_condition(c))          // HC-8
    bound(delegation_depth, component_count ∝ subject.scale)                 // HC-6

remaintain(harness):                                 // update (periodic)
    for c in harness.components:
        if host_now_provides(c.gap):        remove(c, reason=superseding_native)  // HC-3/HC-4
        elif model_now_handles(c.gap):      remove(c, reason=gap_closed)          // HC-3
        elif c improvable:                  update(c)
        else:                               keep(c, reason=still_justified)       // HC-9
    for d in harness.deferred:
        if d.trigger fired and d.gap now open:  add(d)                            // HC-8
    record(dispositions)                                                          // HC-10
```

### 4.2 Justification as a Named Gap

A component's justification ledger entry is the gap it closes:

| Component | Named gap (justification) | Prune when |
| --- | --- | --- |
| self-verification skill | model does not reliably self-check | model self-corrects reliably (HC-3) |
| destructive-command hook | host does not guard dangerous ops | host ships a native guard (HC-4) |
| planner role | model does not plan multi-step work alone | model plans adequately unaided (HC-3) |
| memory shim | host has no scoped memory | host adds native scoped memory (HC-4) |

A component whose "Named gap" column would be empty fails HC-2 and is not added.

### 4.3 The Two Supersession Sources (HC-3/HC-4)

```text
[REFERENCE]
model-supersession:  the base model improved → it now does X unaided → the X-compensator is dead weight
host-supersession:   the host added native X → the custom X-shim now duplicates it → remove, name the native feature
```

Both are invisible unless actively checked; HC-3 makes the check periodic, not
incidental.

### 4.4 Division of Labour

| Concern | Owner |
| --- | --- |
| Which components should exist; prune the unjustified; right-size; defer | this spec |
| Improve a component's content against evidence | `l1-harness-engineering` |
| Search over harness configurations | `l1-harness-optimization` |
| Measure whether a change moved a slice | `l1-agent-coevaluation` |
| Instantiate/release a role | `l1-roles` (ROL-3/ROL-4); this spec supplies ROL-9 criteria |

This spec adds the *right-sizing + continuous-justification* discipline; it never
evolves a component's content, searches configurations, or scores runs.

## 5. Implementation Notes

1. Store each component's justifying gap as first-class metadata (HC-2); a component
   with an empty gap field is a pruning candidate by construction.
2. Schedule re-justification against current model/host capability as a recurring
   maintenance pass (HC-3), not only at creation.
3. Make removal archive-then-remove (HC-9) so it is always reversible; this is what
   makes pruning safe enough to actually do.
4. Keep the deferred set with triggers next to the active set (HC-8/HC-10) so a
   just-became-relevant pattern is added on time, not forgotten.

## 6. Drawbacks & Alternatives

- **Re-justification cost.** Periodically re-evaluating every component against current
  model/host capability is work. Justified: bloat's cost is paid on *every* run, while
  re-justification is paid occasionally, and HC-10's record makes the pass cheap to
  scope to what changed.
- **Aggressive pruning risk.** Removing a component whose gap had not fully closed can
  regress behavior. Mitigated by HC-9 safe reversible removal and by gating a removal
  through `l1-agent-coevaluation` (a removal that regresses a slice is reverted).
- **Alternative — only grow (engineering/optimization).** Rejected: a harness that only
  grows becomes a bloated museum of compensations for model weaknesses that no longer
  exist; the trim direction is a distinct, necessary discipline.
- **Alternative — a fixed maximal catalog for everyone.** Rejected (HC-7): a payments
  service and a docs site need very different harnesses; the same maximal set for both
  is bloat for one and mis-fit for the other.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[HARNESS-ENG]` | `.design/main/specifications/l1-harness-engineering.md` | The grow counterpart this trims against. |
| `[ROLES]` | `.design/main/specifications/l1-roles.md` | Hire/fire mechanism + ROL-9 justification gate (HC-5). |
| `[EXTENSIONS]` | `.design/main/specifications/l1-extensions.md` | Component registry; HC-4 dedup surface. |
| `[COEVAL]` | `.design/main/specifications/l1-agent-coevaluation.md` | Gates a prune/add by whether it moved a slice. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-02 | Core Team | Initial spec — harness composition as the right-sizing / continuous-justification / anti-bloat counterpart to harness-engineering's grow: minimal justified composition, bloat is a defect (HC-1); component = named capability-gap compensation (HC-2); continuous re-justification, prune when the model or host closes the gap (HC-3); dedup against host-native + existing components (HC-4); anti-sprawl ≥N-axis justification gate for specialists (HC-5); bounded complexity — capped delegation depth + scale-proportional component count (HC-6); right-size to the subject from a trait-selected catalog (HC-7); deferred-with-trigger not speculative (HC-8); lifecycle disposition keep/update/add/remove with reasons + safe reversible removal (HC-9); observable/auditable composition, drift-to-bloat detectable (HC-10). Composes harness-engineering / harness-optimization / agent-coevaluation / roles (ROL-9) / orchestration / extensions / office-model; resolves the long-standing l1-roles custom-role-sprawl TBD via ROL-9. |

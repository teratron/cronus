# Capability Reachability

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

The project has a complete account of *how* a capability is disclosed — a small resident descriptor, an on-demand body, deeper tiers below it — and a complete account of *how* one is contributed at a seam. Neither answers a question that decides both: **who is allowed to invoke it.**

There are exactly two answers, and they are not two settings of one dial. An **agent-reachable** capability keeps a resident descriptor, so the actor can fire it on its own and other capabilities can compose it; it pays standing context on every turn, whether or not it fires. A **human-only** capability has no resident descriptor at all: only a person naming it can start it, nothing else can reach it, and it costs the context window nothing. What it costs instead is the person's memory — they are the index that has to know it exists.

That second cost is the one this concept refuses to treat as waste. Standing context is a budget to minimize. A human's recall is a budget to **spend deliberately**: it is the price of keeping a decision in a person's hands, and it is worth paying exactly where their judgement is the point. The failure the project would otherwise walk into is making everything agent-reachable because that reads as "more capable", and thereby handing the agent every judgement call the human meant to keep.

## Related Specifications

- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — PD-1's resident descriptor and PD-4's routing contract describe the *agent-reachable* class in full. This spec adds the second class, which has no descriptor and therefore falls outside PD entirely, and states what changes when a capability moves between them.
- [l1-extension-points.md](l1-extension-points.md) — EP-2's *contribute* kind adds an invocable surface; EP-3 makes the core the sole mediator of invocation. Reachability declares **who may ask** for that invocation; EP governs what happens once someone does.
- [l1-surface-parity.md](l1-surface-parity.md) — INV-9 shipped-surface honesty and its retirement rule; a capability's reachability class is part of what a surface honestly ships, and a class change is a surface change.
- [l1-harness-composition.md](l1-harness-composition.md) — HC-1/HC-4 prune components against the host's native provision; this prunes against a different axis — whether anything but a person needs to reach the component at all.
- [l1-derived-instructions.md](l1-derived-instructions.md) — DIN-6's self-locating instructions and DIN-9's authored guidance; a router (REA-7) is authored guidance about reach, and it is subject to DIN's staleness discipline.
- [l1-context-degradation.md](l1-context-degradation.md) — the behaviour of a capability whose ambient configuration is missing; REA-8 classifies that absence before it happens.
- [l1-declarative-configuration.md](l1-declarative-configuration.md) — the ambient configuration REA-8's precondition classes are declared against.
- [l1-action-gating.md](l1-action-gating.md) — gating decides how much friction an *act* passes; reachability decides whether an actor may *initiate* it at all. A human-only capability is not a gate: it is an absent path.
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — REA-7's stale-router condition is a checkable invariant, not a documentation preference.
- [l1-instruction-economy.md](l1-instruction-economy.md) — the sentence-grain companion: REA decides whether a descriptor exists at all, IEC decides whether each line inside one earns its place.

## 1. Motivation

Left undeclared, reachability defaults to *everything is agent-reachable*, because that is what adding a description does. Five costs follow, and only the first is obvious:

- **Standing cost on capabilities that never fire autonomously.** A descriptor is loaded every turn. One that exists only so a person can type its name has bought nothing and is charged for continuously.
- **Judgement quietly transferred.** A capability that exists to make a person decide something — an interview, a scoping call, a destination-setting session — becomes something the actor can start by itself. The moment it can, it will, and the decision the capability existed to elicit gets made by the wrong party.
- **Uncomposable material with nowhere to live.** Two human-only capabilities that need the same reference cannot reach each other, by construction. Without a rule, the material gets duplicated into both, and the two copies drift.
- **Preconditions that fail as the wrong error.** A step whose precondition is a human-only capability, written as an invocation, fails as *capability not found*. The reader diagnoses a broken system when what is missing is a person's action.
- **Routers that lie.** Once human-only capabilities outnumber what a person holds, someone writes an index of them. Nothing keeps that index current, so it eventually routes to something withdrawn or omits something new — and it is trusted precisely because it is the thing a person reaches for when they cannot remember.

None of these is a disclosure failure or a seam failure. They are all consequences of an unstated property.

## 2. Constraints & Assumptions

- **A capability has exactly one reachability class at a time.** Classes are a partition, not a spectrum; a capability that is "mostly human-only" is agent-reachable.
- **Agent-reachability is not a permission.** It states that the actor *may initiate*; whether the resulting act is permitted is gating's question, answered separately and afterwards.
- **The host enforces the class.** Making a capability unreachable to the actor is a property the surrounding system provides; where a host cannot enforce it, the class is unavailable and the spec says so rather than pretending.
- **A person is present for the human-only class to mean anything.** In a fully unattended run, human-only capabilities are simply unavailable — which is the correct behaviour, not a degradation to route around.
- **Recall cost is real and finite.** A person holds a modest number of these before the set stops being usable. That ceiling is what makes REA-7 necessary rather than optional.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **REA-1 (Reachability is declared, from a closed two-class set):** every invocable capability declares itself **agent-reachable** or **human-only**. Agent-reachable means it carries a resident descriptor, the actor may initiate it, and other capabilities may compose it — the human can always name it too, since agent reach *includes* human reach and never replaces it. Human-only means it carries **no** resident descriptor, only a person naming it may start it, and **nothing else can reach it, including by naming it to the invocation mechanism**. There is no third class and no partial state; an undeclared capability is agent-reachable by default, which is why the declaration matters.

- **REA-2 (Two budgets, and only one of them is minimized):** the classes spend different resources. Agent-reachable spends **standing context** — the descriptor's tokens and the attention it takes, on every turn, whether or not it fires. Human-only spends **recall** — a person must know it exists and when to reach for it. Standing context is minimized. **Recall is not**: it is the price of human agency, spent deliberately where a person's judgement is the point of the capability, and removed only where it is not. An implementation that treats recall as pure waste will convert every human-only capability to agent-reachable and hand away the judgement each one existed to keep.

- **REA-3 (The test is autonomy, not reuse):** a capability is agent-reachable when the actor must be able to reach it **on its own**, or another capability must compose it. **Reuse is a reason to extract a capability; it is not a reason to make it agent-reachable** — the two questions have different answers and conflating them is how standing cost accumulates without anyone deciding to spend it. A capability that only ever fires because a person typed it is human-only, however many places its content is relevant to.

- **REA-4 (Human-only capabilities are not composable, so shared material is externalized):** because nothing but a person can reach a human-only capability, material that **two** human-only capabilities both need can live in **neither** — placing it in one makes it unreachable from the other. It is moved to plain external reference that any capability may point at. This is a structural consequence of REA-1, not a stylistic preference, and duplicating the material into both is the failure it forecloses: two copies with no notification channel between them.

- **REA-5 (A human-only precondition is an instruction to the person, never an attempted invocation):** where a step's precondition is a human-only capability, it is phrased as **an action for the human to take**, and the system does not attempt to invoke it. An attempted invocation fails as a missing capability, which is a diagnosis of the wrong thing: what is absent is a person's action, and reporting it as a broken surface sends the reader to fix the system instead of to do the step.

- **REA-6 (Composition is an explicit invocation act, one target per act, never a mention or a path):** a capability that depends on another expresses it as an **explicit instruction to invoke that capability through the invocation mechanism**, naming the target. It MUST NOT be expressed as a bare textual mention left to be interpreted as a command, and MUST NOT reach into the target's internal files by path — the first is unreliable and the second bypasses the target's contract (composing EP-1/EP-3). An act invokes exactly **one** target: a step needing two states two acts, because a single act phrased with two names reads as one invocation carrying both and reliably does neither.

- **REA-7 (A router hints and cannot fire; a stale router is a defect):** where human-only capabilities exceed what a person holds, the cure is a **router** — itself human-only — that names the others and when to reach for each. It can only **hint**: by REA-1 it cannot invoke what it lists. Two obligations bind it. Adding, renaming, withdrawing, or re-scoping any human-reachable capability **obliges a router update in the same change**. And a router that names a withdrawn capability, or omits a live one, is a **correctness defect** rather than documentation debt — it is consulted precisely by the person who cannot remember, so its errors land on the reader least able to catch them.

- **REA-8 (Precondition classes: hard carries a remediation pointer, soft carries none):** a capability that depends on ambient configuration declares which failure it takes when that configuration is absent. **Hard** — the output is *wrong*, not merely less sharp — and the capability carries an explicit pointer naming what to run to supply it. **Soft** — the output degrades gracefully — and the capability refers to the material generically and carries **no** pointer. The split is load-bearing in both directions: a hard dependent without a pointer produces confidently wrong output, and a pointer cargo-culted onto every soft dependent spends standing cost everywhere and trains the reader to skip the line that mattered.

- **REA-9 (The discoverable set is declared once and rendered, never restated per surface):** which capabilities are offered to a person — the shipped set, as distinct from what merely exists in the source — is declared in **one** place and rendered into every surface that presents it: the distribution manifest, the router, the help listing, the completion set. A surface that maintains its own hand-written list is a second source of truth for membership, and it drifts from the day it is written (composing DIN-1/DIN-10).

- **REA-10 (A change of class is a visible change, never a silent edit):** moving a capability between classes changes who can start it, what it costs every turn, and whether anything may compose it. It is therefore an **announced change to the shipped surface**, carrying the same discipline as adding or withdrawing the capability itself (composing INV-9): the router is updated, the manifest is updated, and dependents that composed it as agent-reachable are re-pointed or refused. Silently removing a descriptor breaks every composition of it in a way that reads as an intermittent failure to find a capability that is plainly still there.

- **REA-11 (Reachability is a property of the capability *and* the executor, established in the mode it will be used):** [ADDED v1.1.0] REA-1's declaration answers *whether an actor may start this*; where the work may be carried out by **more than one executor** — a second bench, a delegated peer, an external runtime — it does not answer *which* actor. Presence and reachability are then properties of the **(capability, executor) pair**: the same capability may be resident on one executor, absent on another, and present-but-differently-shaped on a third. Three consequences bind. A plan or composition that names a capability **names the executor it was established on**; establishing it on the commissioning side proves nothing about the side that will run it. **Asymmetry is recorded, never averaged away** — "present on one bench only" is a stated fact about the plan, not a detail to be discovered when the work is handed over, and it is a legitimate reason to route the work to the bench that holds it. And a capability whose behaviour under the **invocation mode it will actually be used in** — most often a non-interactive or headless mode, which loads and resolves differently from the interactive one — has never been exercised is **not established**: it is an assumption, and it is exercised before anything is allowed to depend on it. Discovery of what an executor holds is **input to a plan, never an activation**: nothing loads because it was found, only because the plan names it.

## 4. Detailed Design

### 4.1 The two classes

| | Agent-reachable | Human-only |
| --- | --- | --- |
| Who may initiate | The actor, or a person | Only a person |
| Resident descriptor | Yes — the trigger surface (PD-1, PD-4) | None |
| Composable by other capabilities | Yes | **No, ever** |
| Standing context cost | The descriptor, every turn | Zero |
| Recall cost on the person | Low — the actor finds it | The person is the index |
| Description is written for | The actor, carrying trigger conditions | A person browsing, one line, no triggers |
| Right when | The actor must reach it autonomously, or another capability must | Its point is that a person decides to start it |

### 4.2 Where shared material goes

REA-4 produces a small decision table, and the middle row is the one that surprises:

| Both consumers are | Shared material lives | Why |
| --- | --- | --- |
| Agent-reachable | Inside the capability that owns it; the other invokes it | Owner keeps one copy; reach is available |
| One of each | Inside the agent-reachable one | The human-only one may invoke it; the reverse is impossible |
| **Both human-only** | **Outside both, as plain external reference** | Neither can reach the other, so neither can own it |

### 4.3 Precondition declaration

REA-8's classification is made once, at the capability, and it answers exactly one question: *with this configuration absent, is the output wrong or merely blunt?*

```text
declare precondition:
    subject   := <the ambient configuration relied on>
    class     := hard | soft
    on_absent := hard -> name the action that supplies it, in the capability body
                 soft -> refer to the subject generically; emit no pointer
```

The temptation is to mark everything hard "to be safe". That inverts the cost: the pointer appears on every capability, is read as boilerplate, and stops being seen on the ones where the output really is wrong without it.

### 4.4 Router obligations

A router is a human-only capability whose whole content is *which capability, and when*. Its correctness condition (REA-7) is mechanical enough to check:

- every human-reachable capability in the declared set (REA-9) appears in the router;
- every capability the router names exists and is still reachable;
- a withdrawn capability appears, if at all, only as a named supersession, never as a live route.

Both directions are failures. A router that omits a capability makes it effectively unreachable — a person who cannot remember it and cannot find it has the same experience as one where it does not exist.

## nodus-relevance mapping

- **The language's callable vocabulary is the same partition.** A construct the runtime may select on its own and one an author must write explicitly are two reachability classes over one vocabulary; declaring which is which is what stops the schema from implying the runtime may reach everything it can name.
- **Hard and soft configuration.** The config surface already distinguishes required from optional fields; REA-8 supplies the missing consequence — a required field's absence must name what supplies it, an optional one's must not, or every field's diagnostic reads the same.
- **Composition by invocation, not by path.** A macro reaching another macro through the registry rather than by file path is REA-6 at the language grain: the registry is the contract, and a path into another unit's internals is the bypass this forbids.

## 5. Implementation Notes

1. **Declare the class at the capability, in one field.** A class inferred from the presence or absence of a description is a class nobody chose; make it explicit so REA-10's change is a visible diff.
2. **Enforce, do not merely document.** REA-1's "nothing else can reach it" has to be real: an attempt by another capability to invoke a human-only target is refused with a message that says *this is a human action*, not *not found* (REA-5's diagnosis, applied at the mechanism).
3. **Keep the two description registers apart.** An agent-facing descriptor carries trigger conditions; a human-facing one is a single line with the triggers stripped. Reusing one for the other either leaks trigger prose into a menu or leaves a descriptor that cannot route.
4. **Check the router in the gate** (REA-7). The three conditions in §4.4 are a set comparison against the declared set (REA-9); a router that lies should fail a build, not surprise a person.
5. **Move classes deliberately, and one at a time.** A batch reclassification hides which composition broke; REA-10's announcement is worth little if it names ten changes at once.

## 6. Drawbacks & Alternatives

- **Two classes is a decision authors must make.** Real, and small: it is one field with two values, and REA-3 gives the test in one question. The alternative is not *no decision* but *a default nobody examined*.
- **Human-only capabilities can be forgotten.** The honest cost of REA-2, mitigated but not eliminated by REA-7. A router reduces the recall load to one item; it does not reduce it to zero, and pretending otherwise would be the argument for making everything agent-reachable.
- **A router is one more thing to keep current.** Accepted, and made mechanical by REA-7 plus the §4.4 check. An unchecked router is worse than none, which is precisely why the check is an invariant rather than advice.
- **Alternative — a single class, everything agent-reachable.** Rejected by REA-2 and REA-3: it charges standing context for capabilities that never fire autonomously, and it silently transfers to the actor every judgement a human-only capability existed to keep with the person.
- **Alternative — a permission flag instead of a class.** Rejected: a permission is evaluated when an act is attempted and can be granted at the moment of attempt. Reachability is about whether the path exists at all — there is nothing to grant, and modelling it as a gate invites a bypass that a class does not have.
- **Alternative — let the descriptor's wording do the work (write it so the actor never triggers it).** Rejected by REA-1: wording biases a trigger, it does not remove a path. A capability the actor should never start on its own is not a capability with a discouraging description; it is one with no descriptor.
- **Alternative — fold into `l1-progressive-disclosure`.** Rejected: PD's entire model presumes a resident descriptor and governs what happens below it. The human-only class has no descriptor, so it is not a disclosure tier at all — it is a different answer to a question PD does not ask.
- **Alternative — fold into `l1-surface-parity`.** Rejected: parity governs whether the same capability appears consistently across surfaces and honestly reports what it binds. It has no notion of an actor initiating an invocation, which is the whole subject here. The two compose at REA-10 and are otherwise orthogonal.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DISCLOSURE]` | `.design/main/specifications/l1-progressive-disclosure.md` | The agent-reachable class in full: descriptor, body, tiers, routing contract |
| `[SEAMS]` | `.design/main/specifications/l1-extension-points.md` | Contribution kinds and core-mediated invocation |
| `[PARITY]` | `.design/main/specifications/l1-surface-parity.md` | Shipped-surface honesty and retirement discipline (INV-9) |
| `[DERIVED]` | `.design/main/specifications/l1-derived-instructions.md` | Single-source rendering that REA-9 composes |
| `[ECONOMY]` | `.design/main/specifications/l1-instruction-economy.md` | The sentence-grain companion to this capability-grain contract |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-26 | Core Team | Initial concept — **who may invoke a capability**, the property disclosure and seam models both presume and neither declares. A closed two-class partition, agent-reachable (resident descriptor, actor may initiate, composable, standing cost) versus human-only (no descriptor, only a person, reachable by nothing else, zero standing cost); undeclared defaults to agent-reachable (REA-1); two budgets of which only one is minimized — standing context is minimized, a person's **recall is spent deliberately** as the price of human agency, and treating it as waste converts every human-only capability and hands away the judgement it kept (REA-2); the class test is **autonomy, not reuse** — reuse justifies extraction, never reach (REA-3); human-only capabilities are structurally uncomposable, so material two of them share lives in **neither** and is externalized (REA-4); a human-only precondition is phrased as an action for the person, never attempted, since an attempt diagnoses a broken surface where a person's action is what is missing (REA-5); composition is an explicit invocation act naming one target, never a bare mention nor a path into internals (REA-6); a router hints and cannot fire, must be updated in the same change as any reachable-capability change, and a stale one is a **correctness defect** because it is read by the person least able to catch it (REA-7); precondition classes — **hard** (output wrong) carries a remediation pointer, **soft** (output blunt) carries none, and marking everything hard destroys the pointer's signal (REA-8); the discoverable set is declared once and rendered into every surface, never restated per surface (REA-9); a class change is an announced surface change with dependents re-pointed, never a silent descriptor removal (REA-10). Concept-only. |
| 1.1.0 | 2026-09-01 | Core Team | Amended — REA-11: reachability is a property of the **(capability, executor) pair** wherever more than one executor may do the work. REA-1 answers whether an actor may start a capability, not which actor holds it; a plan naming a capability names the executor it was established on, asymmetry between benches is recorded rather than discovered at handover, and a capability never exercised in the **invocation mode it will actually run in** (typically a non-interactive/headless mode that loads and resolves differently) is an assumption, not an establishment. Discovery is input to a plan, never an activation. Mined from an external cross-model plan-hardening skill collection. |

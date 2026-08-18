# Scoped Capability Layers

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The model of **how one shared capability registry presents a different world to each actor** without forking into one registry per actor. Where [l1-extension-points.md](l1-extension-points.md) defines *where* a contribution attaches and [l1-extensions.md](l1-extensions.md) defines *what* a contribution is, this concept defines *whom a contribution is visible to*: the **scope** — a named region of visibility and lifetime that every registration is filed under — and the composition rules by which one actor's effective capability set is derived from the shared set plus the scopes it belongs to.

The problem is specific and structural. An office runs many agents at once against one core. Each needs its own tool set, persona, prompt sections, and restrictions, yet they must share one registry: forking a registry per actor multiplies the contribution surface by the actor count, drifts the moment one copy is amended, and makes "which capabilities does this actor actually have" answerable only by inspecting N copies. Scoped layers give one registry with per-actor overlays — a contribution is either **shared** (every actor sees it) or **scoped** (exactly the scope that owns it sees it), and an actor's view is an ordered merge of its scope chain over the shared layer.

Two properties do most of the work, and both are easy to get wrong. First, the registration site determines **visibility and lifetime together** — one fact, never two knobs. Second, the scope chain runs in **opposite directions** for supply and for observation: capability flows down to what a composition composed, activity flows up from it.

## Related Specifications

- [l1-extension-points.md](l1-extension-points.md) - The seam side: EP-2's point-kind taxonomy and EP-4's deterministic composition govern *how* contributions compose at a point; this governs *whose view they compose into*. EP-11's reserved-namespace rule is the anti-shadowing guarantee between contributor and core; SCL-4's shadowing is the deliberate, scoped opposite — a same-name replacement inside one actor's view only.
- [l1-extensions.md](l1-extensions.md) - The artifact and its lifecycle (EXT-2); a scope is the region an activated extension's contributions land in.
- [l1-interception-model.md](l1-interception-model.md) - INT-4's nested interception scopes are the guard-plane analogue of this chain, and INT-6's strip/deny axis is what SCL-7 resolves: a scoped removal is a *strip* that also refuses on invocation. INT-8's honest-coverage rule is the discipline SCL-10 applies to isolation claims.
- [l1-composition-binding.md](l1-composition-binding.md) - How a named composition acquires a scope and how actors join it; that spec owns the binding, this owns what the binding means for visibility.
- [l1-security.md](l1-security.md) - The authority plane. SCL-9 declares that scoping is explicitly **not** part of it.
- [l1-orchestration.md](l1-orchestration.md) - Delegation topology; SCL-13 keeps lineage a data fact rather than a scope-structure fact, so changing the tree never silently changes capability.
- [l1-roles.md](l1-roles.md) / [l1-office-archetype.md](l1-office-archetype.md) - The catalog whose blueprints become per-actor layers at runtime; ROL-6's composition contract is what a scope materializes for one live instance.
- [l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) - ATE-3 (absence is the unmissable signal) is what SCL-7 enforces at the scope boundary; ATE-13's *no such target* is the outcome class a filtered-away capability must produce.

## 1. Motivation

Every capability plane in an office is registry-shaped — tools, prompt sections, commands, variables, restrictions, listeners. Every one of them is asked the same question by the office: *give this agent a different set than that agent*, without the two agents diverging into two products.

Three failures follow from leaving the visibility half of the registry unmodeled.

**A registry per actor, arrived at by default.** The obvious implementation of "agent A has a different tool set" is a second tool registry. It works until the first amendment, at which point one registry has it and the other does not, and nothing reports the divergence. The cost is paid continuously and invisibly, and it scales with the number of concurrently-staffed roles — exactly the axis an office is supposed to grow along.

**Visibility and lifetime configured separately.** If a contribution can be *visible* to one actor while *owned* by another, two states become representable that should not be: a capability the actor still sees after its owner is gone, and a capability withdrawn while its owner is alive and expects it present. Both surface as the model calling a tool that is not there, or not calling one that is — failures that look like model defects and are not.

**Isolation assumed, never declared.** The moment a handle exists that "belongs to" an actor, every capability reached through it is assumed to be that actor's own. Most are not: a registry isolates only if it files by scope. An undeclared assumption here is how one agent's state leaks into another's while every component involved behaves exactly as written.

The resolving idea is to make **scope a first-class property of a registration**, with one fact driving both visibility and lifetime, an explicitly directional chain, and declared answers to "which planes actually isolate" and "what is this mechanism not for".

## 2. Constraints & Assumptions

- The actor may be an agent, a session, a run, or a composition — the model is actor-agnostic and names no specific one. What matters is that an actor has an identity a scope can be keyed by.
- Scopes are **within one trust domain**. Everything here routes contributions among components already permitted to run; nothing here decides what is permitted (SCL-9).
- The chain is a chain, not a graph: a scope has at most one parent. Multi-membership policy sets are out of scope; where a design needs one, the answer is a contribution that composes the sets, not a second parent.
- A scope's identity is **opaque** and compared by identity rather than by name matching, so two scopes cannot collide by coincidence of naming.
- This concept is additive to `l1-extension-points`: it changes nothing about *how* contributions at one point compose, only *which contributions are in that composition for a given actor*.

## 3. Core Invariants (Layer 1 only)

Rules every Layer 2 implementation MUST NOT violate:

- **SCL-1 (One registry, per-actor layers — never one registry per actor):** a capability plane has exactly one registry. Per-actor difference is expressed as **layers over it**, never as a duplicate registry, a forked catalog, or a copied contribution set. The effective set for any actor is derived on demand from the shared layer plus that actor's scope layers; it is never materialized as an independent, separately-amendable collection.

- **SCL-2 (One fact drives visibility and lifetime):** the scope a registration is made through determines **both** who can see it **and** when it is torn down. These are never independently configurable. The states this forbids are the ones that actually ship: a capability an actor still sees after its owner is gone, and one withdrawn while its owner is live and assumes it present. Unifying the two facts makes both unrepresentable rather than merely discouraged.

- **SCL-3 (One chain, two opposite directions):** scopes form a parent chain, and it is traversed in **opposite directions** by the two things that traverse it. **Supply inherits downward** — a descendant sees its ancestors' layers. **Observation admits upward** — an observer registered at an ancestor is admitted to events about a descendant, never the reverse. The asymmetry is the design, not an accident: a composition supplies capability *to* what it composed and observes activity *from* it. A design in which one direction mirrors the other has conflated the supply relation with the observation relation, and will either leak a descendant's capabilities to its ancestor or deafen a composition to the work it is running.

- **SCL-4 (Shadowing by name; nearest wins; replacement, never merge):** where a scoped contribution and a farther one (an ancestor's, or the shared layer's) claim the same name, the **nearest** wins for that scope alone, and it **replaces** the farther one entirely. This is the per-actor persona and per-actor variant mechanism. It is never a field-wise merge of two contributions: a surface assembled from halves of two authored things was authored by nobody, and its behavior becomes a property of the merge algorithm rather than of any contributor's intent.

- **SCL-5 (Governing contributions are read exact-scope, never inherited):** contributions that **govern** a scope — restrictions, guards, policy filters — are read at the exact scope and MUST NOT resolve along the chain. Supply inherits; governance does not. A scope that inherited its ancestor's restrictions would acquire a governance decision nobody made for it, and could not express being *differently* restricted without appearing to escape a guard. Where an ancestor's restriction must genuinely bind descendants, it is enforced by the guard plane ([l1-interception-model.md](l1-interception-model.md) INT-5), whose transitivity is a security property — not by making a visibility mechanism impersonate one.

- **SCL-6 (Restriction narrows the shared set by intersection; scope-local merges after):** a restriction narrows the **shared** set for one scope; multiple restrictions compose by **intersection** — never union, never last-writer-wins, so adding a restriction can only ever remove. Scope-local contributions merge **after** the filter, so a scope may hold a capability the filter removed only by contributing it **itself**, under its own identity, as a deliberate act. Restriction and contribution are therefore never in a race whose winner depends on registration order.

- **SCL-7 (A filtered-away capability is indistinguishable from a nonexistent one):** a capability removed from an actor's view is **absent from that actor's affordance surface and refused on invocation**, producing the same outcome class as a capability that never existed. Both halves are required. Leaving it visible-but-failing teaches the actor to abandon a capability that works fine elsewhere ([l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) ATE-2). Removing it from the surface without refusing invocation leaves a bypass for any actor that learned the name from a transcript, a prior session, or its own training.

- **SCL-8 (Bounded setup window — compose before publish; setup registers, never drives):** an actor's scoped world is composed inside a **bounded window** that opens once the scope and the actor exist and closes **before** the actor is published to observers, before its first request is assembled, and before any input can reach it. Two rules bound the window: everything the actor's world needs is registered **inside** it, and its occupant **registers only** — it does not send input, start work, or drive the actor. A composition that fails inside the window rolls the whole creation back rather than leaving a half-composed actor; a registration landing after the window produces an actor whose world differs from the one its first work was planned against, which is unreproducible by construction.

- **SCL-9 (Scoping is routing among trusted components — the non-goal is part of the contract):** scoping decides **who sees what**; it never decides **who is allowed what**. Authority, confinement, and least privilege belong to the security plane ([l1-security.md](l1-security.md), [l1-extensions.md](l1-extensions.md) EXT-3/EXT-4, [l1-interception-model.md](l1-interception-model.md)). This non-goal is part of the published contract and MUST be stated wherever the mechanism is documented, because a mechanism that *looks* like isolation is used as isolation unless it says otherwise — and the resulting confinement is one nobody implemented and no review would think to check.

- **SCL-10 (Honest isolation boundary — only scope-aware planes isolate):** a plane isolates by scope only if it **files** by scope. A capability that does not remains shared even when reached through a scoped handle. Which planes are scope-aware is **declared**, and "this actor has its own X" is never inferred from the fact that X was reached through the actor's handle. An undeclared isolation assumption is the same defect class as an unnamed guard-coverage gap (INT-8) and is surfaced the same way: in the plane's own contract and in diagnostics.

- **SCL-11 (Reachability comes from the minter; a broad handle cannot be narrowed by its holder):** a scoped handle carries the **minting** component's resolution surface — whatever the minter could reach, the holder reaches through the handle. Two consequences bind. The handle is minted from the component whose dependencies the scoped registrations actually need, not from whichever component happened to be holding a context. And narrowing is done by **minting narrower**, never by the holder restraining itself: a broadly-minted handle passed onward is an authority widening that no grant recorded and no audit shows.

- **SCL-12 (The default membership of a plane is declared, not inherited by accident):** for every plane, whether an actor that joins no composition sees the shared layer or sees **nothing** is an explicit, declared property of that plane. Where the whole of a plane is contributed per-actor, its shared layer is **empty** and a joinless actor reaches the model with nothing. Both defaults are defensible; an undeclared one is not, because its failure mode is silent over-capability — an actor holding capabilities nobody decided to give it, which no test asks about and no surface shows.

- **SCL-13 (Lineage is data; nesting is capability — never one knob for both):** facts about an actor's position in a delegation tree — who spawned it, how deep it sits, which run owns it — are carried as **data** and MUST NOT be expressed by nesting scopes. Nesting changes visibility and lifetime; recording a parent changes neither. Conflating them turns a topology change into a silent capability change: re-parenting a sub-agent for reporting reasons would hand it a different world, and restricting a sub-agent's world would rewrite the delegation record.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The layer model

Each capability plane holds one **shared layer** and zero or more **scope layers**, each keyed by an opaque scope identity. A registration is filed into the layer of the scope it was made through; a registration made through no scope lands in the shared layer.

```plaintext
plane: tools
  shared layer          read_file, search, notify
  scope layer :alpha    search(variant), review_diff        (shadows shared `search` — SCL-4)
  scope layer :beta     deploy
  scope layer :alpha/1  —                                   (child of :alpha)

view(:alpha/1) = shared  ∩ restrictions(:alpha/1)   ← exact-scope only (SCL-5, SCL-6)
               ⊕ layer(:alpha)                      ← inherited supply (SCL-3)
               ⊕ layer(:alpha/1)                    ← nearest, shadows all (SCL-4)
```

A scope layer is created lazily on first registration and reclaimed when it becomes empty, so an actor that contributes nothing costs nothing — the scalability property EP-9 states for contributions, applied to their visibility.

**A disposed ancestor shrinks its descendants' view; it does not silently invalidate them.** SCL-2 makes each registration unwind with the scope it was made through, so releasing a composition removes *its* layer and leaves every joined actor alive with a smaller view. That is the correct behavior and it is not a quiet one: the removal is the same observable event as any other contribution withdrawal, and an actor whose remaining view no longer satisfies what it was composed for surfaces that as an unavailable capability (SCL-7), never as a capability that appears present and fails. Where an implementation instead intends a composition's release to end the actors composed under it, that cascade is a **declared property of the composition** ([l1-composition-binding.md](l1-composition-binding.md)), not an implicit consequence of scope nesting — otherwise SCL-13's separation collapses and disposing a parent for lifetime reasons becomes an unrecorded termination of its descendants.

### 4.2 Why the chain runs both ways

The chain expresses one relation — *this scope was composed inside that one* — and two different things read it.

| Reader | Direction | Rule | Why |
| --- | --- | --- | --- |
| Capability resolution | ancestor → descendant | a descendant sees ancestors' layers; nearest shadows farthest | A composition exists to supply the actors composed under it; denying that denies the composition's purpose. |
| Event admission | descendant → ancestor | an ancestor-registered observer is admitted to a descendant's events | A composition must be able to watch the work it runs; the reverse would let a composed actor observe its siblings through their shared parent. |
| Governance (restrictions, guards) | neither | read at the exact scope only (SCL-5) | Inherited governance is a decision nobody made, and it makes "less restricted than my parent" indistinguishable from "escaping my parent's guard". |

The third row is the one most designs get wrong, because inheritance *feels* uniform. It is not: supply and governance travel in opposite moral directions. Supplying more downward is a gift; governing more downward is an authority claim, and authority claims belong to a plane that can enforce them transitively and audit them (INT-5).

### 4.3 Composing one actor's view

The resolution is total and order-independent:

1. Take the **shared layer**.
2. Apply the actor scope's **own** restrictions (exact-scope, SCL-5), composed by **intersection** (SCL-6). The result is the actor's permitted slice of the shared set.
3. Overlay the scope chain's layers, **farthest ancestor first**, so nearer layers shadow farther ones by name (SCL-4).
4. Anything removed at step 2 and not re-contributed at step 3 is absent **and** refuses invocation (SCL-7).

Because step 2 reads only the exact scope and step 3 is a fixed traversal order, the same chain and the same registrations always yield the same view — the EP-4 determinism property, preserved through the visibility layer.

### 4.4 The setup window

The window (SCL-8) is the only place an actor's world is authored, and its two rules solve two different failures.

**Compose before publish** solves reproducibility. Once an actor is published, an observer may read its capability set, a request may be assembled from it, and work may be planned against it. A registration arriving after any of those has produced an actor whose history was made under a world that no longer describes it — the same hazard as a composition edited mid-run ([l1-composition-binding.md](l1-composition-binding.md) CBD-2), arriving one layer lower.

**Register, never drive** solves ownership. The window's occupant is a composer, not a caller. If setup could send input or start work, the actor's first turn would have an author other than the one the office believes started it, and cancelling or re-planning that work would have no owner to address.

A failure inside the window is a **whole-creation rollback**. A half-composed actor is worse than no actor: it is present, addressable, and wrong.

### 4.5 What this mechanism is not

SCL-9 and SCL-10 are the two honesty rules, and they answer the two questions a reader forms immediately on seeing the model.

*"Is a scope a sandbox?"* — No. It is trusted-component routing. Untrusted code is confined by the security plane, out of process where that plane says so, and a scope boundary neither blocks a call nor limits a reach. The rule is stated in the contract because the mechanism's shape invites the opposite conclusion.

*"Does everything the actor touches through its handle belong to the actor?"* — Only where the plane declares it files by scope. A shared counter, a process-wide cache, or a global client reached through a scoped handle is still shared. The declaration is what turns this from a trap into a documented boundary, exactly as INT-8 turns an incomplete guard into an honest one.

### 4.6 Worked shape: an office of many agents

An office runs a manager and four specialists concurrently against one core.

- The **office composition** mounts once and holds a scope. Its tools, prompt sections, and norms are registered there — **once**, not five times (SCL-1).
- Each agent's scope is **parented** to the office's (SCL-3), so all five see the office's contributions, and the office's observers see all five agents' activity.
- One specialist is restricted to a read-only slice: its own scope carries a restriction narrowing the shared set (SCL-5, SCL-6). The removed tools are absent from its prompt and refuse invocation (SCL-7) — the specialist behaves exactly as though the office never had them.
- Another specialist needs a differently-tuned variant of a shared tool: it contributes the variant under the same name in its own scope, shadowing the shared one for itself alone (SCL-4). No other agent is affected and the shared registration is untouched.
- A sub-agent spawned by a specialist records its parent as **data** (SCL-13) and joins the office composition through the binding rules of [l1-composition-binding.md](l1-composition-binding.md); its delegation depth is a fact, not a visibility structure.
- When an agent is released, everything registered through its scope unwinds with it (SCL-2), and the office's registrations — owned by the office's scope — are untouched.

## 5. Drawbacks & Alternatives

- **The chain's two directions are a learning cost.** A contributor must know that supply inherits down and observation admits up. The alternative — uniform inheritance — is simpler to state and wrong in one of the two directions whichever way it is chosen.
- **Exact-scope governance (SCL-5) surprises.** "My parent restricted this, why can I still see it?" is a fair question, and the answer — *because visibility is not authority; the guard plane is what makes a restriction transitive* — requires holding two planes in mind. The alternative, inheriting restrictions, buys an intuitive read and pays with a governance decision nobody made and an inexpressible "differently restricted" case.
- **Replacement rather than merge (SCL-4) forbids partial overrides.** A scope wanting to change one field of a shared contribution must restate the whole thing. This is the same trade [l1-composition-layering.md](l1-composition-layering.md) LAY-2 makes for the same reason, and it is deliberate: a merged surface has no author.
- **Rejected — a registry per actor.** Simplest to reason about per actor and unmaintainable across actors: N copies to amend, N places for drift, and no answer to "what does the office contribute" that does not enumerate its staff.
- **Rejected — scope as an authority boundary.** Tempting, because the shape is right there. Rejected because an authority boundary must survive an adversary and a routing mechanism is not built to; claiming it would leave the product with confinement nobody implemented and no review would check. SCL-9 states the refusal so it cannot be re-derived accidentally.

## nodus-relevance mapping

Nodus already resolves providers by **locus proximity** (LP-21) — the nearest declaring scope beats declared priority. That is this model's SCL-3/SCL-4 read at the DSL grain; the gaps this concept names are the ones LP-21 does not reach.

| Element | nodus seam | Note |
| --- | --- | --- |
| Nearest-wins supply (SCL-3/SCL-4) | LP-21 step 2, locus proximity | Already present and already host-ordered; this concept supplies the reason the ordering is *supply-only*. |
| Exact-scope governance (SCL-5) | per-effect authorization gate (LP-11), `@needs` disclosure (NL-16) | A gate is read for the step that declares it; a nested run declares its own rather than inheriting one. |
| Filtered-away ⇒ nonexistent (SCL-7) | LP-8 capability manifest + LP-21 declared refusal | A role with no provider in the run's locus fails pre-run; an unimplemented operation names what the provider does offer rather than falling through. |
| Setup window (SCL-8) | validate-before-run stage | The bounded window in which a run's world is fixed; a capability arriving after validation would invalidate the satisfiability claim LP-8 made. |
| Declared non-goal (SCL-9) | LP-10 host-granted authority | Authority sits entirely with the host; locus proximity selects an implementation, it never grants one. |
| Lineage as data (SCL-13) | run/parent identifiers in the trajectory (NE-3) | Sub-run relationships travel as observability data, never as a resolution structure. |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[POINTS]` | `.design/main/specifications/l1-extension-points.md` | The seam taxonomy (EP-2) and composition determinism (EP-4) this layers visibility onto. |
| `[EXTENSIONS]` | `.design/main/specifications/l1-extensions.md` | The artifact lifecycle (EXT-2) whose contributions land in a scope. |
| `[INTERCEPTION]` | `.design/main/specifications/l1-interception-model.md` | INT-5 transitive enforcement — why governance is the guard plane's, not this one's; INT-8, the honesty model SCL-10 reuses. |
| `[BINDING]` | `.design/main/specifications/l1-composition-binding.md` | How a scope is acquired and joined; the binding half of this pair. |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | The authority plane SCL-9 defers to. |
| `[ERGONOMICS]` | `.design/main/specifications/l1-agent-tool-ergonomics.md` | ATE-2/ATE-3/ATE-13 — the outcome classes SCL-7 must produce. |
| `[NODUS-PORTABILITY]` | `.design/nodus/specifications/l1-nodus-portability.md` | LP-21 locus-proximity resolution, the DSL-grain prior instance. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-19 | Core Team | Initial spec — scoped capability layers: the visibility half of the plugin architecture, previously unmodeled. One registry with per-actor layers, never a registry per actor (SCL-1); the registration site drives visibility **and** lifetime as one fact, making "visible here, disposed there" unrepresentable (SCL-2); one parent chain traversed in **opposite** directions — supply inherits down, observation admits up — because a composition supplies what it composed and observes activity from it (SCL-3); nearest-wins shadowing as replacement rather than merge, since a surface assembled from halves of two authored things was authored by nobody (SCL-4); governing contributions read **exact-scope**, never inherited, because inherited governance is a decision nobody made and makes "differently restricted" indistinguishable from "escaping a guard" — transitivity belongs to the guard plane (SCL-5); restrictions narrowing the shared set by **intersection** with scope-local contributions merged after, so restriction and contribution never race (SCL-6); a filtered-away capability absent from the surface **and** refused on invocation, both halves required (SCL-7); a bounded setup window closing before publication, whose occupant registers but never drives, with whole-creation rollback on failure (SCL-8); scoping declared **not** an authority boundary, stated in the contract because a mechanism shaped like isolation is used as isolation unless it says otherwise (SCL-9); isolation claimed only by planes that declare they file by scope (SCL-10); reachability inherited from the minter and narrowed by minting narrower rather than by holder restraint (SCL-11); per-plane default membership declared rather than accidental, the failure mode of an undeclared default being silent over-capability (SCL-12); lineage carried as data so a topology change is never a silent capability change (SCL-13). Distilled from an adoption pass over an external plugin-framework-based agent-harness reference. Concept-only. |

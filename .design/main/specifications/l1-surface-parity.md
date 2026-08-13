# Surface Parity

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Surface parity is the discipline that keeps **one product from becoming several products that share a name**. The architecture already states the rule — domain logic lives in the core, and every frontend is a thin binding over it. This spec answers the question that rule leaves open: *what keeps it true on the day someone adds the fifth surface, in a hurry, to a behavior the core does not quite expose yet.*

The answer is not a stronger rule. Boundary checks — "the core must not import the renderer" — pass cleanly while two surfaces compute the same span with two different formulas and disagree by one line. That divergence is invisible to every import gate, every type check, and every unit test written on either side, because each side is internally consistent. It is visible only when the same input is driven through both surfaces' **real** projections and the answers are compared.

So this spec supplies four mechanisms: a **duplication inventory** that treats each re-derivation as a tracked defect with a definition of *repaid*; **tombstones** so a deleted copy stays deleted; a **shared conformance corpus** every surface registers its real projection against; and **preemptive extraction**, which is the only one that is cheap — the shared primitive is created before the second implementation exists, because after it exists, someone has to decide which of the two was right.

## Related Specifications

- [l1-architecture.md](l1-architecture.md) — INV-8/INV-9/INV-10 state the boundary and the honest-verb rule; this spec is the maintenance contract that keeps them true past the first surface.
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — the general enforcement model this composes. TW-6 already separates structural checks from behavioral tests; SP-6 names the third kind neither catches alone — **cross-consumer behavioral agreement** — and the conformance corpus is its tripwire.
- [l1-derived-instructions.md](l1-derived-instructions.md) — the sibling: parity keeps surfaces from *behaving* differently, derived instructions keep their *descriptions* from drifting from the behavior. SP-11's catalog is the artifact both consume.
- [l1-semantic-addressing.md](l1-semantic-addressing.md) — a re-derived address is simultaneously an SA-1 and an SP-2 violation; addressing is the highest-value single instance of this class.
- [l1-host-native-rendering.md](l1-host-native-rendering.md) — outbound materialization into a foreign host is a surface too, and it joins the corpus like any other consumer.
- [l1-extension-points.md](l1-extension-points.md) — EP-12's dogfooding rule is parity applied to the seam: the host's own implementations go through the public contract, so the contract cannot quietly become insufficient.
- [l1-change-containment.md](l1-change-containment.md) / [l1-solution-frugality.md](l1-solution-frugality.md) — SP-10's *converge first, correct second* is containment applied to unification: an extraction that also changes behavior is two changes wearing one diff.
- [l1-convergence-gate.md](l1-convergence-gate.md) — where the corpus runs; parity checks are gate content, not a separate ceremony.
- [l1-project-vocabulary.md](l1-project-vocabulary.md) — one canonical term per concept is the same discipline in prose; a forked action vocabulary (SP-11) is a forked term with keys attached.

## 1. Motivation

**The second surface is where a product forks, and nobody notices for a year.** The first surface *is* the behavior — there is nothing to disagree with. The second is written by someone reading the first, re-implementing what they need, and getting most of it right. "Most" is the problem: the two implementations agree on every case anyone tested and diverge on pure-insertion, empty, zero-count, renamed, oversized, and every other case that lives in the corners. Those are precisely the cases users hit and cannot describe.

**This divergence class is structurally invisible to the checks already in place.** An import gate proves the core does not depend on the renderer; it says nothing about a renderer that re-derives what the core already computed. A unit test on each side proves each side matches its own author's belief. A type check proves both compile. The disagreement exists only *between* them, and only a test that drives both through one input can see it.

**Re-derivation is locally rational every single time.** The core's function is one import away but returns slightly the wrong shape; the local math is four lines and obviously correct; the deadline is real. Each instance is a good decision. The aggregate is a product whose two interfaces answer the same question differently, and where fixing either one is now a behavior change for someone.

**By the time the duplication is found, correcting it is a second, riskier change.** Three implementations of one range calculation, one of them wrong — the extraction is easy, but adopting the *correct* one changes visible output for the surfaces that shipped the wrong one. If those two changes travel in one diff, the review sees a refactor and the users see a regression. They have to be staged, which means the finding has to be *recorded* rather than fixed on sight.

**Not every similarity is duplication, and unstated exceptions become the next fork.** Row measurement in a terminal and height estimation in a browser genuinely differ. If the difference is merely tolerated rather than *named*, the next reader either unifies them wrongly or takes them as licence for the next divergence. The exceptions have to be as explicit as the rules.

## 2. Constraints & Assumptions

- **The product has, and will keep adding, surfaces.** A library API, a command line, a terminal UI, a desktop UI, an external-caller protocol, an agent-facing surface, and outbound materialization into foreign hosts. This is not a transitional state to be minimized away.
- **Surfaces legitimately own presentation.** Layout, measurement, input mechanics, animation, per-client view state, and platform idioms are surface property and are not candidates for unification.
- **Extraction has a cost and cannot be unbounded.** The mechanisms here are for behavior a user can observe through more than one surface; internal helper similarity is not in scope.
- **A conformance corpus is only as good as its adversarial cases.** Fixtures drawn from the happy path prove nothing, since the happy path is where the implementations already agree.
- **Some divergence is discovered as a live defect.** The contract must handle "we found it in production and one side is wrong" as the normal case, not the exception.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **SP-1 (One decision point per observable behavior):** any behavior a user can observe through **more than one** surface is decided in exactly one place — the shared core. A surface owns presentation, input mechanics, and client-local view state, and nothing else. Where a surface needs a decision the core does not expose, the remedy is to extend the core, never to decide locally "for now".

- **SP-2 (Derived facts are consumed, never recomputed):** a fact derived from the shared model — a range, a span, an ordering, a default target, a fallback choice, an empty-state explanation, a match rule — is computed **once** and consumed everywhere. A surface that recomputes it is defective **even while it agrees**, because agreement today is a coincidence that nothing preserves and nothing measures.

- **SP-3 (Divergence is a tracked defect class, not a review opinion):** each discovered duplication is recorded as a **finding** naming its sites, the divergence observed or reachable from it, and the single primitive that will replace it. A duplication mentioned in a review and fixed later from memory is a duplication that will be rediscovered; the inventory is what makes the class shrink monotonically instead of oscillating.

- **SP-4 (A finding is repaid only when all four hold):** copies **deleted**; their deletion **pinned** so they cannot return (SP-5); an **adversarial fixture** landed for the divergence the finding named; and every consumer **registered** against the corpus that runs it. Three of four is not partial credit — a deleted copy with no fixture returns as soon as someone needs the behavior again, and a fixture no consumer runs proves nothing about the consumers.

- **SP-5 (Deletion is pinned; the ledgers move one way):** a removed duplicate is recorded in an **append-only** tombstone list (the file, or the symbol, that must not come back), and any list of accepted outstanding debt is **shrink-only**. Both directions are the point: the tombstone list only grows, the debt list only shrinks, and a change that reverses either is visible as such rather than arriving as a quiet re-addition.

- **SP-6 (Parity is proven by one corpus through real projections):** agreement between surfaces is established by driving **shared fixtures through each surface's actual projection** and asserting identical semantic outcomes. A boundary check, a type check, and per-surface unit tests are all structurally blind to this class: each side is internally consistent, and the defect lives only in the comparison. The corpus is that comparison, and it is the tripwire for a rule no import gate can express.

- **SP-7 (A new consumer registers before it ships):** a new surface joins the conformance corpus **as part of becoming a surface**, not after it has diverged. Registration is what converts "we intend to stay in agreement" into a mechanically failing condition when it stops being true, and it is an order-of-magnitude cheaper before the surface has behavior to preserve.

- **SP-8 (Legitimate difference is named, with its reason):** an explicit **do-not-unify** record lists what is genuinely surface-specific and why. An unstated exception is read by the next author either as an oversight to be unified or as licence for the next divergence; naming it removes both readings and keeps the inventory honest about its own boundary.

- **SP-9 (Extract before the second implementation exists):** when a second surface is planned, the shared primitive is extracted **first** — the vocabulary, the catalog, the derivation — even where only one surface consumes it yet. Preemptive extraction costs one refactor with one caller; the same extraction after the fact costs a reconciliation, a decision about which behavior was correct, and a staged behavior change (SP-10). This is the only cheap point in the lifecycle.

- **SP-10 (Converge first, correct second — unification never smuggles a behavior change):** where extraction reveals that the surfaces agreed on something **wrong**, the extraction makes them agree on the *existing* behavior and the correction ships as a **separate, disclosed change**. A residual known-incorrect behavior is recorded at the primitive that owns it. A diff that both unifies and corrects presents a regression to users as a refactor to reviewers, and neither audience can see what actually happened.

- **SP-11 (Shared vocabulary, declared exposure — omission never means absence):** the actions a product offers are described **once**, in a catalog carrying each action's identity, human-readable name, and **resolution locus** — semantic (belongs to the core), client-local (belongs to a surface), or host-only (never remotely invocable). Menus, help, palettes, bindings, and agent-facing instructions all render from that catalog. An action a given surface deliberately does not expose is **declared** as such in the catalog: a capability boundary stated as an exclusion is a scope decision, while the same boundary expressed by silence is indistinguishable from a missing feature and will be "fixed" by someone.

## 4. Detailed Design

### 4.1 Anatomy of a finding (SP-3)

A finding is a short, durable record — not a ticket that closes and disappears:

| Field | Content |
| --- | --- |
| **Sites** | every implementation, named, with which one is authoritative |
| **Divergence** | the observed or reachable disagreement, in user-visible terms |
| **Class** | *observed* (found in production or in review) or *preemptive* (SP-9, the copy does not exist yet) |
| **Primitive** | the single derivation that replaces the copies |
| **Repayment** | the four conditions of SP-4, each checked off individually |
| **Residual** | any known-incorrect behavior the extraction preserved, and why (SP-10) |

The *class* field matters more than it looks. Preemptive findings are the cheap ones and the ones an inventory drawn only from bug reports will never contain — an inventory with no preemptive entries is an inventory that has given up on SP-9.

### 4.2 What each check can and cannot see

| Check | Catches | Blind to |
| --- | --- | --- |
| Import / boundary gate | core depending on a surface; layering violations | a surface re-deriving what the core already computes |
| Type check | shape mismatch | two correct-shaped answers that differ |
| Per-surface unit test | that surface matching its author's belief | the other surface's belief |
| **Conformance corpus (SP-6)** | **the surfaces disagreeing on one input** | whether the agreed answer is *right* — that is a fixture's job |

The last row is the reason the corpus is not redundant with the other three, and the reason it must run each surface's **real** projection: a corpus that tests a shared helper both surfaces already call proves nothing about the surface that quietly does not call it.

### 4.3 The extraction ladder

```
second surface planned
   │
   ├─ primitive extracted first ──────────▶ SP-9: one refactor, one caller.  Cheapest point.
   │
   └─ not extracted
        │
        ├─ copy written, agrees today ────▶ SP-2 defect, invisible.  Costs a finding.
        │
        └─ copy written, disagrees ───────▶ observed divergence.  Costs a finding,
                                              a decision about which was right,
                                              and a staged correction (SP-10).
```

Every step down the ladder multiplies the cost of the same extraction. Nothing about the extraction itself gets harder — what gets harder is everything that has come to depend on the disagreement.

### 4.4 Residuals and staged correction (SP-10)

An extraction that finds a wrong shared answer faces a fork, and only one branch is safe:

- **Correct it inside the extraction.** The diff reads as "unify three copies"; the release changes what users see. When something breaks, nobody can tell whether the unification or the correction did it.
- **Preserve the existing behavior, record the residual at the primitive, and ship the correction separately.** The unification is provably behavior-preserving and can be reviewed as such. The correction is small, isolated, and disclosed, and it lands against a codebase where exactly one place decides the answer.

Recording the residual **at the primitive** — not in a tracker — is what makes the second step happen: the next reader of that function sees the known-wrong behavior stated where they are already looking.

### 4.5 The catalog as shared vocabulary (SP-11)

The single artifact that most reliably forks is the list of things the product can *do*. Each surface grows its own names, its own groupings, and eventually its own semantics for the same verb, and the drift shows up as a user who cannot follow their own instructions from one interface to another.

One catalog, three consumers, one addition point:

```
                    ┌── surface menus / palettes / help  (names + current bindings)
action catalog ─────┼── key and command resolution        (identity, never a hard-coded chord)
 (id, name,         └── agent-facing instructions          (derived, l1-derived-instructions)
  locus, default
  binding)
```

Two consequences fall out. A component that owns a key asks the catalog about a **command identity**, never a hard-coded chord, so a user's rebinding is honored everywhere the action appears. And an action's *locus* is declared, which is what lets a remote or agent-facing surface expose the semantic actions while structurally refusing the host-only ones — an exclusion stated in the catalog rather than implied by an omission nobody documented.

### 4.6 Nodus relevance

**No new language invariant.** The parity discipline is a property of the host's surfaces, and nodus is one consumer of the core rather than a surface over it. Two alignments are worth stating so the boundary stays honest:

- **NL-6 (dual representation)** is surface parity inside the language: compact and human forms are two presentations of one AST, and the round-trip requirement (`compact → human → compact` yields an AST-equal result) is exactly SP-6's shape — a shared corpus driven through both real projections, asserting semantic identity. The language got there first; the rest of the product adopts the same proof.
- **NL-1/NL-16 (schema-first, selective vocabulary disclosure)** make the workflow vocabulary a single declared catalog, which is SP-11 for the language: commands exist once in the schema, and disclosure decides what a given run *sees* rather than each surface restating what exists.

## 5. Implementation Notes

- Seed the inventory from a single deliberate audit, not from incoming bug reports. The findings that matter most are the ones with no user-visible symptom **yet**, and no report will ever name them.
- Write the adversarial fixture from the divergence, not from the feature: empty, single-element, zero-count, boundary-crossing, renamed, oversized, absent. The happy path is where the implementations already agree, so it proves nothing.
- Register the corpus in the ordinary quality gate rather than as a separate ritual. A parity suite that runs on request runs never.
- When the corpus is added to a codebase that already has several surfaces, expect it to fail immediately and in an unflattering way. That output *is* the initial inventory; convert it into findings before fixing anything, or the repairs will be rediscovered as duplicates.
- Keep the tombstone list boringly literal — a path, or an owner-qualified symbol name, and the finding it belongs to. Cleverness here produces a check nobody trusts and everybody bypasses.

## 6. Drawbacks & Alternatives

- **The corpus is a maintenance surface of its own.** Every new semantic behavior wants a fixture, and every new surface pays a registration cost. That cost is the price of the guarantee, and it scales with surfaces rather than with features.
- **SP-9 asks for extraction on the strength of a plan.** A second surface that never arrives leaves a primitive with one caller — a small, real waste. Accepted: the reverse error costs a reconciliation and a staged behavior change, which is strictly larger.
- **SP-10 slows the satisfying part.** Finding a wrong answer and being required to preserve it for one more release is genuinely annoying, and it is the whole reason the residual is recorded at the primitive rather than left to memory.
- **Alternative — one surface, and everything else scripts it:** rejected. It collapses the class by amputation, and the product's surfaces exist because their audiences are genuinely different (a terminal, a desktop, an external caller, an agent).
- **Alternative — rely on import boundaries and code review:** rejected by §4.2. Both are structurally blind to a re-derivation that currently agrees, which is the state every divergence passes through before it becomes visible.
- **Alternative — generate all surfaces from one declaration:** rejected as over-reach. It buys parity by giving up the platform-idiomatic presentation each surface exists to provide; the catalog (SP-11) takes the part of that idea that pays — shared vocabulary — without the part that does not.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ARCH]` | `.design/main/specifications/l1-architecture.md` | INV-8/9/10 — the boundary this spec maintains over time |
| `[TRIPWIRE]` | `.design/main/specifications/l1-invariant-tripwires.md` | The enforcement model SP-6 instantiates for cross-consumer drift |
| `[INSTRUCTIONS]` | `.design/main/specifications/l1-derived-instructions.md` | The description-side sibling; consumes the SP-11 catalog |
| `[ADDRESS]` | `.design/main/specifications/l1-semantic-addressing.md` | The highest-value single instance of the re-derivation class |
| `[GATE]` | `.design/main/specifications/l1-convergence-gate.md` | Where the conformance corpus runs |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-13 | Core Team | Initial concept: **what keeps "thin bindings over one core" true past the first surface**, given that boundary gates, type checks, and per-surface tests are all structurally blind to a surface re-deriving what the core already computes. One decision point per observable behavior, with "extend the core" as the only remedy for a missing decision (SP-1); derived facts **consumed, never recomputed** — a recomputation is defective *while it still agrees*, because agreement is a coincidence nothing preserves (SP-2); divergence as a **tracked defect class with findings**, since a duplication fixed from memory is rediscovered (SP-3); **repaid means all four** — copies deleted, deletion pinned, adversarial fixture landed, every consumer registered (SP-4); **append-only tombstones and shrink-only debt**, so reversal is visible rather than quiet (SP-5); parity proven by **one corpus through each surface's real projection**, the third check kind neither structural gates nor unit tests can express (SP-6); a new consumer **registers before it ships** (SP-7); **legitimate difference named with its reason**, because an unstated exception is read as either an oversight or a licence (SP-8); **extract before the second implementation exists** — the only cheap point on the ladder (SP-9); **converge first, correct second**, so unification never presents a regression to users as a refactor to reviewers, with the residual recorded at the primitive (SP-10); **shared action catalog with declared locus**, where an unexposed action is declared rather than merely absent (SP-11). §4.2 tabulates what each check can and cannot see; §4.3 the extraction cost ladder; §4.5 the catalog as the vocabulary three consumers share; §4.6 records the nodus alignment — NL-6's round-trip is SP-6's shape and NL-1/NL-16 are SP-11 inside the language. |

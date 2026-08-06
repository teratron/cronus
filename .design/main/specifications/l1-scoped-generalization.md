# Scoped Generalization

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Scoped generalization answers the question the codification pathway never asks: **where does a learned pattern apply?**

Pattern codification governs *whether* an observed behaviour becomes a norm — recurrence raises a candidate, durability and human ratification make it binding. Every step of that pathway measures evidence **across time**. None of it measures evidence **across contexts**. Five observations on five different days inside one project satisfy the recurrence test perfectly, and the rule that results is, by default, a rule about everything.

That default is the defect. A convention learned in one codebase, one domain, or one principal's way of working is *correct there* and confidently wrong elsewhere, and at the point of application it is indistinguishable from a rule that earned its breadth. This spec fixes the missing axis: a pattern is born **narrow**, widens only by recurring in **independent** contexts, narrows again when contradicted, and is checked against its scope **when it is used** rather than only when it was written.

## Related Specifications

- [l1-pattern-codification.md](l1-pattern-codification.md) — the pathway this spec adds an axis to. PC-1's recurrence and PC-4's dated observations establish *that* a pattern is real; SG-3 establishes *where* it is true. The two are orthogonal (SG-11) and both gates must clear before a pattern binds anywhere.
- [l1-memory-model.md](l1-memory-model.md) — MEM-1's scopes are the storage-side partition; SG scopes are an **applicability** claim carried by the pattern itself. A pattern stored in a wide scope may still apply narrowly, and storing it wide does not widen it.
- [l1-memory-consolidation.md](l1-memory-consolidation.md) — MC-6's corpus maintenance operates within a corpus; scope decides which corpus a pattern is even a candidate for, and SG-9's decay is the candidate-tier counterpart of MC-6's staleness→archive.
- [l1-corpus-originality.md](l1-corpus-originality.md) — ORI-8's *declare the relationship rather than relax the gate* is the same discipline SG-7 applies to imports: an incoming pattern declares where it came from instead of arriving pre-widened.
- [l1-context-provenance.md](l1-context-provenance.md) — an imported pattern is externally-authored content and carries its origin as provenance (SG-7).
- [l1-host-native-rendering.md](l1-host-native-rendering.md) — HNR-6's stable-slug discipline is the same argument SG-2 makes for context identity: an identity that is really a location silently changes when the thing moves.
- [l1-improvement-loop.md](l1-improvement-loop.md) — IMP-2 generalizes a finding **at capture** so user content never travels; SG governs the opposite direction (how far a generalization may be *applied*). Demarcated in §4.5.
- [l1-office-archetype.md](l1-office-archetype.md) — an archetype is a *prior* whose applicability is declared by domain; OA-9's recorded deviations are exactly the contradiction evidence SG-5 narrows on.
- [l2-self-improvement.md](l2-self-improvement.md) — the concrete corpus (mistake log, reasoning templates, ask-backs) whose per-project partition and cross-project mode this contract governs.
- [l1-user-model.md](l1-user-model.md) — a pattern about *the principal* and a pattern about *a project* have different natural scopes; conflating them widens a project convention into a claim about the person.
- [../../nodus/specifications/l1-nodus-portability.md](../../nodus/specifications/l1-nodus-portability.md) — **LP-3 is the prior exemplar of SG-3**: a pattern observed in one host enters the portable library only when it is demonstrably useful in **two independent** host contexts, with §4.14 supplying the falsifiable admission record. That rule was written for library extraction; this spec is the same principle stated once for every learned artifact.

## 1. Motivation

Everything that learns accumulates two kinds of knowledge and stores them identically. Some of what it learns is about *the world* — validate input at boundaries, read before editing. Some is about *one place* — this repository formats that way, this domain calls it that, this principal wants terse answers on Fridays. Both arrive through the same observations, both recur, both satisfy every existing promotion test.

Left unscoped, the second kind leaks, and the leak has an unusually bad shape:

- **It is confident.** An out-of-scope pattern is applied with exactly the authority of one that earned its breadth, because at the point of use nothing distinguishes them.
- **It is invisible.** Nothing fails. The agent simply behaves, in a new project, according to conventions from an old one, and the resulting friction reads as the agent being wrong in general rather than misapplied in particular.
- **It compounds.** The mis-scoped rule shapes new observations, which recur, which produce more rules with the same inherited scope.
- **It discredits the mechanism.** The user's conclusion is not "that rule was mis-scoped", it is "the learning is unreliable, turn it off" — and the well-earned rules are switched off along with it.

The naive fix — learn only universal things — throws away most of the value, because the place-specific knowledge is precisely what makes an agent feel like it knows *this* codebase. The right fix is to keep learning everything and be honest about where each thing applies: **narrow by default, widened only by evidence from somewhere else.**

There is already a worked instance of exactly this rule inside the project. The portable workflow library admits a pattern observed in one host only when it is demonstrably useful in **two independent** hosts; single-host observations stay in that host's adaptor layer. That rule was written to keep a library clean. It is the same rule, and this spec states it once for everything that learns.

## 2. Constraints & Assumptions

- **Narrow knowledge is valuable, not second-class.** This contract exists to make place-specific learning *safe to keep*, not to discourage it.
- **Independence is a judgment, not a count.** Two checkouts of one repository, or two workspaces of one project, are one context; deciding otherwise requires a stated reason (SG-3).
- **Contexts are not strictly nested.** A pattern may be scoped to a project, a domain, a principal, or a task class, and these overlap; the model is a set of applicability claims, not one tree.
- **Scope is carried by the pattern, not inferred at read time.** A consumer must be able to ask "does this apply here" without re-deriving the pattern's history.
- **This spec governs applicability, not admission.** Whether a pattern becomes binding at all remains PC's question; whether it is a near-duplicate of an existing one remains ORI's.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **SG-1 (Born narrow; unscoped is unusable, not universal):** every learned pattern carries an explicit **applicability scope**, and its initial value is the **narrowest context it was observed in** — never global, never absent. A pattern with no scope MUST NOT be applied anywhere: absence of a claim is not a claim of universality, and treating it as one is precisely how a project convention becomes a law.

- **SG-2 (Durable context identity):** a context's identity is **stable across relocation and distinct between coinciding things** — derived from something durable about the context, not from where it currently sits. A context identified by its filesystem location silently changes identity when moved (orphaning every pattern that named it) and silently merges two different things that occupy the same location (cross-contaminating both). This is HNR-6's argument applied to contexts rather than to definitions.

- **SG-3 (Widening requires independent recurrence):** repetition **within** one context is evidence that the pattern is **real**; it is not evidence that it **generalizes**. A pattern earns a wider scope only by being independently observed in **at least two independent contexts**, and independence is a **declared judgment** with a stated reason — two checkouts of one repository are one context, and counting them as two manufactures generality out of nothing. This is the load-bearing invariant: PC-4's recurrence-across-dates and this recurrence-across-contexts are different measurements, and only the second one licenses breadth.

- **SG-4 (Widening is a distinct, recorded, ratified act):** a scope change is an explicit act — never a side effect of a counter crossing a threshold — and it records **which contexts supplied the evidence** and who ratified it (PC-2). An automatic widening is an agent granting its own conclusions a reach the human never agreed to, and it is unauditable afterwards because the evidence that justified it was never named.

- **SG-5 (Contradiction narrows, it does not except):** a widened pattern **contradicted** in a new context is **demoted back** toward the contexts where it demonstrably holds, and the contradiction is recorded as evidence. A rule that claims to hold everywhere is falsified by one place that disagrees; the honest response is a **narrower rule**, not a wide rule with a growing list of suppressed exceptions, which is a rule nobody can predict the behaviour of. (Composes PC-5 reversibility.)

- **SG-6 (Scope is checked at application, not only at storage):** the consumer asks, at the moment of use, **whether this pattern applies here**, and an out-of-scope pattern is **not offered** — not offered-and-ignored, not offered-with-a-caveat. A corpus filtered only on write is permanently contaminated by everything written before the scope model existed, and by every import; the check must sit where the pattern is consumed.

- **SG-7 (Imported patterns arrive narrow):** a pattern received from **another corpus** — another principal, a shared bundle, an exported library — enters at the **receiving** side's narrowest scope with its **origin recorded**, regardless of the scope it claimed at its source. Another corpus's "global" means *observed across contexts you have not seen*, which is evidence about their world and a hypothesis about yours. Widening it here follows SG-3 like anything else, on evidence gathered here.

- **SG-8 (Cross-context leakage is the named failure):** applying a pattern outside its earned scope is a **first-class defect**, not an inefficiency — worse in effect than never having learned the pattern, because it is applied confidently, is indistinguishable at the point of use from a well-earned rule, and produces the *why did it do that* failure whose usual resolution is disabling the entire learning mechanism. An implementation MUST be able to detect and report an application that occurred out of scope.

- **SG-9 (Candidates decay):** a candidate pattern that stops recurring within a **declared window** expires and is removed together with its evidence. An unbounded pool of indefinitely-pending candidates becomes a second, unratified, unaudited rule corpus that grows monotonically and is consulted by nobody — and whose sheer size later gets used as an argument to promote in bulk.

- **SG-10 (Scope is explainable at the point of application):** whenever a pattern influences behaviour, the system can answer **why it applied here** — its current scope, the contexts whose evidence earned that scope, and the widening record (SG-4). A pattern that cannot explain its own applicability cannot be argued with, and an unarguable rule is either obeyed blindly or disabled wholesale.

- **SG-11 (Scope and bindingness are orthogonal axes):** how far a pattern applies (this spec) and how strongly it binds (PC-3's observation → advisory → ratified rule ladder) are **independent**. A narrowly-scoped **ratified rule** is normal and common — the strictest conventions are usually the most local. A widely-scoped **advisory** is equally normal. Collapsing the two axes forces "important" to mean "everywhere", which is exactly the pressure that produces mis-scoped rules.

## 4. Detailed Design

### 4.1 The two independent gates

```
observations (dated, contextualized)
        │
        ├── PC gate:  did it recur over TIME, durably, and was it ratified?
        │             → decides BINDINGNESS  (observation → advisory → rule)
        │
        └── SG gate:  did it recur across INDEPENDENT CONTEXTS?
                      → decides SCOPE        (this context → wider → global)

both gates are evaluated; neither substitutes for the other (SG-11)
```

The failure mode this diagram exists to prevent is reading the left branch as the whole picture. Under PC alone, a pattern that recurred ten times in one project on ten distinct days is a well-evidenced, durable, ratifiable rule — and it is a rule about that project, which nothing in the pathway records.

### 4.2 Scope kinds and what makes them independent

| Scope kind | Independent instances are… | Not independent |
| --- | --- | --- |
| Project / codebase | different projects | two checkouts, two branches, two workspaces of one project |
| Domain / trade | different domains of work | two tasks in one domain |
| Principal | different people | one person across their projects (that is *project* recurrence, not *principal* recurrence) |
| Task class | different classes of work | two instances of one class |

The table's right-hand column is the whole point. Independence is the property that makes a second observation *informative*, and an implementation that counts instances instead of judging independence will generalize on the cheapest available evidence — which is always the correlated kind.

### 4.3 The widening record

A scope change records, at minimum:

- the previous and new scope,
- **each context that supplied evidence**, and what was observed there,
- the independence judgment and its reason (SG-3),
- who ratified it and when (SG-4, PC-2).

This record is what makes SG-5's demotion possible: narrowing a pattern requires knowing which contexts it actually held in, and a widening that recorded only a count leaves nothing to narrow *back to*.

### 4.4 Application-time check

```
consumer needs guidance in context C
   → candidate patterns retrieved
   → for each: does its scope claim cover C?          (SG-6)
        no  → not offered (and, if it was, that is a reportable defect — SG-8)
        yes → offered, with its scope answerable on request (SG-10)
```

Filtering here rather than at write time is what makes the model survive an import (SG-7), a scope model introduced after the corpus already existed, and a demotion (SG-5) that must take effect immediately rather than at the next write.

### 4.5 Demarcation — generalize-at-capture is the other direction

The improvement loop generalizes a finding **at capture** (IMP-2) so that a product-level observation never carries the user's content outward. That is a rule about *what leaves*. This spec is a rule about *how far what stays may reach*. They meet only in that both refuse to let a specific thing masquerade as a general one, and an implementation should not reuse one's machinery for the other: the improvement loop's generalization is lossy and deliberate, this one's is a claim that must remain falsifiable.

## 5. Implementation Notes

- Record the context on the **observation**, not on the pattern derived from it. Scope is computed from where the evidence came from, and a pattern that lost its observations' contexts can never be widened or narrowed honestly again.
- The independence judgment (SG-3) is the piece most likely to be quietly automated into a count. If it is automated, the heuristic and its reason belong in the widening record, so a wrong generalization can be traced to the rule that produced it rather than to the pattern.
- SG-8's out-of-scope detection is worth building even though it should never fire: the case where it fires is the case where the scope model has a hole, and that is the only way to find one.
- Decay (SG-9) should remove evidence with the candidate. A candidate deleted but whose observations remain will be re-derived on the next pass, producing an expiry loop that looks like activity.

## 6. Drawbacks & Alternatives

- **Narrow-by-default slows down genuinely universal learning.** A truly universal convention needs two independent contexts before it applies broadly, which means the second project pays full price for something the first already learned. That is the intended trade: the alternative is that the first project's *non*-universal conventions also apply broadly, and those are the majority.
- **Alternative — global by default, narrow on conflict:** rejected (SG-1). It applies mis-scoped rules until something visibly breaks, and most mis-application does not visibly break — it just produces work in the wrong style, which is noticed as a general failing of the agent rather than as a scope error.
- **Alternative — count observations instead of judging independence:** rejected (SG-3). Correlated observations are the cheapest to accumulate, so a counting rule generalizes fastest exactly where the evidence is weakest.
- **Alternative — keep a wide rule and accumulate exceptions:** rejected (SG-5). An exception list makes behaviour unpredictable and hides the fact that the rule's real scope is narrower than claimed; it also grows without bound because nothing ever triggers a re-statement.
- **Independence is a judgment and judgments can be wrong.** Mitigated by SG-4's record (a wrong widening is traceable) and SG-5 (contradiction narrows it back), not by pretending a mechanical test exists. <!-- TBD: whether a default independence heuristic ships, and what it keys on — the risk is that shipping one turns SG-3's judgment back into the count it exists to replace -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CODIFY]` | `.design/main/specifications/l1-pattern-codification.md` | The bindingness axis this spec is orthogonal to (PC-1…PC-8, SG-11) |
| `[MEMORY]` | `.design/main/specifications/l1-memory-model.md` | Storage scopes, distinct from applicability scope |
| `[ORIGINALITY]` | `.design/main/specifications/l1-corpus-originality.md` | Declare-the-relationship discipline SG-7 mirrors for imports |
| `[SELFIMP]` | `.design/main/specifications/l2-self-improvement.md` | The concrete corpus this contract partitions |
| `[PORTABILITY]` | `.design/nodus/specifications/l1-nodus-portability.md` | LP-3, the prior worked instance of SG-3 (two independent host contexts) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-06 | Core Team | Initial concept: **where a learned pattern applies**, the axis the codification pathway never carried. PC-1/PC-4 measure recurrence across **time** and produce a binding rule that is, by default, a rule about everything — so five observations on five days inside one project satisfy every existing gate and yield a project convention with the authority of a law. Born narrow, with an absent scope **unusable rather than universal** (SG-1); durable context identity that survives relocation and distinguishes coinciding things, HNR-6's argument applied to contexts (SG-2); **widening requires independent recurrence** — repetition within one context evidences that the pattern is *real*, never that it *generalizes*, and independence is a declared judgment, since two checkouts of one repository are one context (SG-3); widening as a distinct ratified act recording **which contexts supplied the evidence** (SG-4); **contradiction narrows rather than excepts**, because a wide rule with a growing exception list is a rule whose behaviour nobody can predict (SG-5); scope checked **at application**, since a write-time-only filter leaves the corpus permanently contaminated by everything predating the model and by every import (SG-6); **imported patterns arrive narrow** regardless of the scope claimed at their source — another corpus's "global" is evidence about their world and a hypothesis about yours (SG-7); cross-context leakage named a **first-class defect** worse than not learning, because it is confident, indistinguishable at the point of use, and usually resolved by disabling the whole mechanism (SG-8); candidate decay, so the pending pool never becomes a second unaudited rule corpus (SG-9); scope explainable at the point of application (SG-10); and **scope orthogonal to bindingness**, since collapsing the axes forces "important" to mean "everywhere" (SG-11). §4.5 demarcates it from IMP-2's generalize-at-capture, which governs what *leaves* rather than how far what stays may *reach*. Notes the prior worked instance: nodus LP-3 already requires two independent host contexts before a pattern enters the portable library — the same rule, previously stated only for library extraction. |

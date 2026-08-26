# Concept Grounding

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Anything a reader moves through **in order** — a report, a walkthrough, an onboarding path, a generated explanation, a sequence of wiki pages, an answer that builds toward a conclusion — carries a dependency structure that nobody declares. Each unit **requires** some concepts and **establishes** others. A unit that requires a concept the reader does not yet hold loses them, and that is the single move an ordered artifact cannot make: everything after it is read by someone who is behind.

The structure is invisible for one specific reason, and it is the reason vocabulary checks do not catch it. **The dependency is the concept, not the word for it.** A passage containing no unfamiliar term can still lean on an idea the reader has never met, and it fails exactly as hard as one full of jargon. Scanning for undefined terms finds the second and misses the first.

This concept makes the structure explicit. A concept is **grounded** in one of two ways: **assumed** — the audience brings it, fixed and declared before the first unit — or **established** — an earlier unit landed it. A unit is admissible only when everything it requires is already grounded. The set of admissible next units is therefore a **frontier** over the grounded set, and establishing a concept widens it. The artifact's central design lever is not the order of units at all: it is the split between what is assumed and what is established, because that split decides who the audience is.

## Related Specifications

- [l1-order-independent-production.md](l1-order-independent-production.md) — the complement, and the reason this matters at scale. OIP requires a unit to be a pure function of its declared position; GRD-5 makes the **grounded set part of that position**, which is what allows units of an ordered artifact to be produced concurrently at all.
- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — a different axis, easy to conflate. Disclosure governs what is **loaded** into an actor's context on demand; grounding governs what a **reader** has been given by the units before this one. Neither implies the other.
- [l1-project-vocabulary.md](l1-project-vocabulary.md) — VOC decides which term is canonical for a concept; GRD-6 decides that the term and its idea land together, in one unit, and that a name introduced without its idea is worse than no name.
- [l1-content-language.md](l1-content-language.md) — the register an artifact is written in; grounding is orthogonal to register and fails identically in every one.
- [l1-content-segmentation.md](l1-content-segmentation.md) — how an artifact is cut into units; GRD takes the cut as given and governs what each resulting unit may lean on.
- [l1-hierarchical-summarization.md](l1-hierarchical-summarization.md) — a summary that drops an establishing unit silently ungrounds every unit downstream of it; the grounded set is part of what a reduction must preserve.
- [l1-report-prompting.md](l1-report-prompting.md) — the produced report is the most common instance; its sections are units with requirements nobody currently declares.
- [l1-project-wiki.md](l1-project-wiki.md) — a navigable set of pages read in many orders; GRD-8's published assumption set is what lets a page state who it is for.
- [l1-computed-grounding.md](l1-computed-grounding.md) — a distinct contract despite the shared word: that one grounds a *claim* in computed fact, this one grounds a *concept* in the reader. Both are named grounding and neither implies the other.
- [l1-generation-shaping.md](l1-generation-shaping.md) — the production controls a unit is generated under; the requirement/establishment declaration is an input to shaping, not a substitute for it.

## 1. Motivation

The failure is common, costly, and almost never diagnosed as itself:

- **The unmarked leap.** A unit uses an idea the reader has not met. The reader does not stop — they carry on, decreasingly attached to the material, and the loss is attributed to the artifact being "dense" or the reader "not being technical enough".
- **Jargon-free and still lost.** A passage written in plain words leans on a distinction the audience does not hold. Every vocabulary check passes. This is the class that motivates the whole contract: the unit of dependency is not the word.
- **The drowning opening.** Corrected by over-assumption's opposite: everything is established up front, and the artifact opens with a wall of definitions for concepts nobody has yet needed. Readers who *did* bring the concepts abandon it before the first substantive unit.
- **The unstated audience.** Two readers approach the same artifact with different backgrounds. Neither can tell whether they are the intended one, because what the artifact assumes was never written down — so the mismatch surfaces as confusion rather than as a decision either of them could have made.
- **The name without the idea.** A term is introduced as a label and used from then on, while the idea behind it was never landed. Every later unit that leans on the term is leaning on nothing, and the artifact reads as coherent to its author and as circular to everyone else.
- **Termination by exhaustion.** An artifact continues until the source material runs out rather than until it reaches what it set out to reach, and the last third is material that was gathered rather than material that was needed.

## 2. Constraints & Assumptions

- **The artifact has an order.** This governs artifacts a reader traverses in a defined sequence. A reference set read in any order is out of scope except at its entry points, where GRD-8's declared assumptions still apply.
- **Concepts are identifiable.** Requirements and establishments can be named at a useful grain. Perfect granularity is not required; the contract fails gracefully toward coarser concepts and only breaks if none can be named at all.
- **The audience is knowable enough to decide about.** GRD-3's assumed set is a decision about who this is for. It can be wrong; it cannot be absent.
- **Units are produced with a bounded view.** A unit's producer does not read every prior unit — that is exactly what GRD-5 exists to avoid — so the grounded set must be supplied rather than derived.
- **More source material than the artifact needs is normal.** Termination is a destination question (GRD-9), not an input-coverage question.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **GRD-1 (Every unit declares what it requires and what it establishes):** a unit carries two explicit sets — the concepts it **leans on**, and the concepts it **lands** for everything downstream. Both are declared, not inferred from the text: a requirement discovered by reading the finished unit is discovered too late to place it, and an establishment nobody declared cannot be depended on by anything.

- **GRD-2 (The dependency is the concept, never its name):** a requirement is an **idea the reader must already hold**, not a term that must already have appeared. A unit written entirely in familiar words can require an unheld concept and fails exactly as a jargon-heavy one does. Consequently, a check that scans for undefined *terms* does not satisfy this invariant and MUST NOT be presented as satisfying it — it catches the visible half of the failure and reports clean on the half that matters more.

- **GRD-3 (Two grounding origins, and the set is monotone within an artifact):** a concept is grounded either as **assumed** — brought by the audience, fixed before the first unit — or as **established** by an earlier unit. There is no third origin, and in particular *mentioned* is not *grounded*. Within one artifact the grounded set only **grows**: a concept once established is not established again, and a second establishment is a signal that the **first one failed** — repaired at the first, never patched by adding a second.

- **GRD-4 (A unit is admissible only if everything it requires is grounded):** placing a unit whose requirements are not all in the grounded set is the one move an ordered artifact cannot make. Everything after it is read by someone who is behind, so the cost is not local to the unit — it compounds across the remainder. Where a unit is desirable and inadmissible, GRD-7 governs; **proceeding anyway is not among the options.**

- **GRD-5 (The grounded set travels with the position; it is never inferred by reading back):** whatever produces or validates a unit is **given** the grounded set as of that position, explicitly. Deriving it by reading every prior unit is forbidden for two independent reasons: it does not fit a bounded working context as the artifact grows, and it forces sequential production of something whose units could otherwise be produced concurrently. This is the clause that makes an ordered artifact compatible with order-independent production — the grounded set is part of what a position *is*.

- **GRD-6 (A term is landed with its idea, in one unit):** where a concept has a name, the unit that establishes it lands **the idea and the name together**. Introducing a term whose idea has not been established grounds nothing while appearing to: every later unit leaning on that term leans on a label, and the artifact reads coherent to its author and circular to everyone else. A name is never an establishment on its own.

- **GRD-7 (Exactly two remedies for an ungrounded requirement, and one of them changes the audience):** when a wanted unit requires something not yet grounded, there are two admissible moves. **Establish it first** — insert a unit that lands the concept, which lengthens the path. Or **promote it to an assumption** — which is not a local edit but a **change to who the artifact is for**, and is therefore a declared decision recorded in the assumption set (GRD-8), not a quiet convenience. Re-ordering that leaves the gap in place is not a third remedy; it relocates the failure.

- **GRD-8 (The assumption set is published as part of the artifact):** what the artifact assumes its reader brings is stated **in the artifact**, before the first unit that depends on it. This is what lets a reader determine whether they are the audience, rather than discovering they were not somewhere in the middle. An unpublished assumption set turns an authoring decision into the reader's private confusion, and it is the reason the same artifact is called clear by one reader and impenetrable by another.

- **GRD-9 (Termination is by destination, not by exhaustion of source material):** the artifact ends when it has reached what it set out to reach. **Leftover source material is expected and is not a defect** — gathering more than is needed is what makes selection possible. An artifact that runs until its inputs are consumed has substituted an input-coverage criterion for a purpose criterion, and its final stretch is material that was available rather than material that was required.

- **GRD-10 (A gap in the source is named, never invented over):** when a unit needs something the source material does not contain — an example, a figure, a case — the gap is **stated as a gap**, and resolved by one of exactly two acts: supply the missing material, or cut the unit that depended on it. Filling it with plausible invented content is forbidden: it grounds a concept on something that is not so, and every downstream unit that leans on it inherits the fabrication silently.

## 4. Detailed Design

### 4.1 The position of a unit

Under GRD-5, a unit's position is not an index. It is the tuple its producer is handed:

```text
position:
    index          := <where in the sequence>
    grounded       := <every concept assumed, plus every concept established before here>
    requires       := <concepts this unit leans on>        # GRD-1, must be subset of grounded
    establishes    := <concepts this unit lands>           # GRD-1, added to grounded after
    destination    := <what the artifact is reaching for>  # GRD-9
```

The admissibility check is then a set containment, evaluated before the unit is produced rather than after:

```text
admissible(unit, position) := unit.requires ⊆ position.grounded      # GRD-4
```

### 4.2 The frontier

The admissible next units are exactly those whose requirements sit inside the current grounded set. Establishing a concept widens the frontier by admitting everything that was waiting on it. This gives the sequencing decision a shape:

| State | What it means | Move |
| --- | --- | --- |
| Frontier non-empty | Several units are placeable now | Choose by destination proximity, not by convenience |
| Frontier empty, destination unreached | Everything left requires something ungrounded | GRD-7: establish, or promote |
| Frontier empty, destination reached | Done | GRD-9: stop, leftovers and all |
| Frontier non-empty, destination reached | Done | GRD-9: stop — the remaining units are available, not required |

The last row is the one that gets violated in practice, and GRD-9 exists for it.

### 4.3 The assumption lever

The split between assumed and established is the artifact's most consequential decision, and it trades in one direction:

| Assume more | Assume less |
| --- | --- |
| Shorter path to the destination | Longer path |
| Narrower audience | Wider audience |
| Opening gets to substance immediately | Opening spends units on definitions |
| Readers without the concepts are excluded silently unless GRD-8 is honoured | Readers who had the concepts may leave early |

Neither end is correct in general. What GRD-8 requires is that whichever was chosen is **visible**, so a reader's mismatch is a fact they can check rather than an experience they suffer.

### 4.4 Concept, not word

GRD-2's distinction, made concrete:

| Passage | Undefined terms | Ungrounded concept | Verdict |
| --- | --- | --- | --- |
| "The write model is event-sourced." | *event-sourced* | Yes, and it is named | Caught by a term scan |
| "So the read side can lag, and you have to decide what a stale read costs." | None | Yes — that reads and writes are separate at all | **Missed by a term scan** |
| "The frontier is the set of admissible next units." | *frontier* | No — established here, with its idea | Fine |

The second row is the failure class this invariant exists for, and the reason GRD-2 explicitly refuses to let a term scan stand in for the check.

## nodus-relevance mapping

- **Generated workflow documentation is an ordered artifact.** An explanation of a workflow walks its steps in order; each step's description leans on concepts the earlier ones established, and the ones the reader brings are exactly the language's own vocabulary — the assumption set is the schema's declared constructs.
- **The grounded set fits the trace.** Producing a per-step narration concurrently, one worker per step, is only sound if each worker is handed the grounded set for its position rather than reading every earlier step — GRD-5 at the runtime grain.
- **A construct's name is landed with its meaning.** GRD-6 is the authoring rule behind the language's own documentation: a keyword introduced as a label, with its semantics arriving later, grounds nothing and produces workflows written by pattern-matching on shape.

## 5. Implementation Notes

1. **Declare requirements before generating the unit, not after.** GRD-4's check is worth nothing once the text exists; the point is to refuse placement, not to review prose.
2. **Keep the concept grain coarse at first.** A dozen well-chosen concepts sequence an artifact; a hundred fine ones produce a dependency graph nobody maintains. Refine only where an admissibility question is actually contested.
3. **Publish the assumption set where the reader lands** (GRD-8), not in an appendix. Its whole job is to be read before the reader has invested anything.
4. **Treat a second establishment as a bug report on the first** (GRD-3). The instinct is to add a reminder; the correct move is to fix the unit that failed to land the concept.
5. **Record leftovers as leftovers** (GRD-9). An explicit note that material was gathered and not used prevents the next pass from reading its absence as an omission and re-adding it.
6. **Make GRD-10's gap visible in the artifact's production record**, not only in the conversation that produced it — a gap resolved by cutting a unit is a decision someone may want to revisit with better material.

## 6. Drawbacks & Alternatives

- **Declaring requirements is authoring overhead.** Real, and bounded by §5.2's coarse grain. The overhead buys the one thing prose review cannot: a placement decision made before the text exists.
- **Concept boundaries are judgement calls.** Accepted. Two authors may split a region of meaning differently and both sequence correctly; the contract needs the sets to be *usable*, not canonical.
- **The assumption set can be wrong about the audience.** Yes — and GRD-8 is what makes that wrongness discoverable instead of silent. A published wrong assumption is corrigible; an unpublished right one is luck.
- **Alternative — check for undefined terms instead.** Rejected by GRD-2: it catches the visible half of the failure and returns clean on the half that costs more, while creating a strong impression that the artifact was checked.
- **Alternative — let each unit define what it needs, in place.** Rejected by GRD-3's monotonicity and by the drowning-opening failure: definitions repeated per unit inflate every unit, and the reader who already holds a concept meets its definition repeatedly.
- **Alternative — infer the grounded set by reading the prior units.** Rejected by GRD-5: it does not fit a bounded working context as the artifact grows, and it forecloses concurrent production of an ordered artifact, which is the property that makes long ones feasible.
- **Alternative — fold into `l1-order-independent-production`.** Rejected: OIP's subject is that a unit must not depend on its siblings' *content*. This spec's subject is that a unit legitimately depends on its predecessors' *effect on the reader*. They compose exactly at GRD-5 — the grounded set is the dependency made positional — and merging them would collapse a dependency contract into an independence one.
- **Alternative — fold into `l1-project-vocabulary`.** Rejected: VOC governs which term names a concept across the project, a global and durable decision. Grounding governs whether *this reader, at this point in this artifact*, holds the concept — a local and per-artifact one. GRD-6 is the seam, not a merge point.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORDER]` | `.design/main/specifications/l1-order-independent-production.md` | The complement; position-as-pure-input, which GRD-5 extends |
| `[VOCABULARY]` | `.design/main/specifications/l1-project-vocabulary.md` | Canonical naming; the term half of GRD-6 |
| `[SEGMENT]` | `.design/main/specifications/l1-content-segmentation.md` | How the artifact is cut into the units this sequences |
| `[REPORT]` | `.design/main/specifications/l1-report-prompting.md` | The most common ordered artifact in the system |
| `[WIKI]` | `.design/main/specifications/l1-project-wiki.md` | Multi-entry artifacts where GRD-8's declared assumptions do the work |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-26 | Core Team | Initial concept — the undeclared dependency structure inside every artifact a reader traverses in order. Each unit declares what it **requires** and what it **establishes** (GRD-1); the dependency is the **concept, never its name**, so a passage with no unfamiliar vocabulary can still lose the reader and a term scan explicitly does not satisfy the check (GRD-2); two grounding origins only — **assumed** (brought by the audience, fixed and declared) and **established** (landed by an earlier unit) — *mentioned* is not grounded, and the set is monotone, so a second establishment is a bug report on the first (GRD-3); a unit is admissible only when all its requirements are grounded, and proceeding anyway is not an option because the cost compounds across everything downstream (GRD-4); the grounded set **travels with the position and is never inferred by reading back**, which is what makes an ordered artifact compatible with concurrent, bounded-context production (GRD-5); a term is landed with its idea in one unit, since a name alone grounds nothing while appearing to (GRD-6); exactly two remedies for an ungrounded requirement — establish before, or promote to an assumption, the second being a **change of audience** and therefore a declared decision (GRD-7); the assumption set is published in the artifact so a reader can tell whether they are its audience (GRD-8); termination is by **destination, not by exhaustion of source material**, and leftovers are expected rather than defective (GRD-9); a gap in the source is named and resolved by supplying or cutting, never by plausible invention that every downstream unit then inherits (GRD-10). Concept-only. |

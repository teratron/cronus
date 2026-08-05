# Order-Independent Production

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Some deliverables are not one artifact but **many units that must line up**: a long report
produced section by section, a batch of records processed in ranges, a rendered sequence, a
corpus transformed page by page. The office already knows how to split such work across
workers and how to own the fan-in. What it has never stated is the property that makes the
split **safe**.

That property is narrow and absolute: **the unit at position *i* is a pure function of *i*
and a frozen input set — and of nothing else.** Not of which units were produced before it.
Not of the wall-clock. Not of a source that can change mid-run. Not of an unseeded draw.

Everything valuable follows from it. Units can be produced in any order, on any number of
machines, and joined; an interrupted run resumes at the next position instead of restarting;
a damaged range is re-produced alone and dropped back into place; and the result is
identical whether it was made by one worker or fifty.

The reason it needs stating is the shape of its failure. A violation does not crash. It
produces units that are each individually plausible and **mutually inconsistent** — the
defect lives at the seams, which is exactly where nobody looks. Worse, the obvious remedy
makes it invisible: producing serially usually *looks* correct, because the units then
happen to be made in an order that hides the dependence. The system then ships with the
defect intact, the parallelism it paid for turned off, and a result that will differ on the
next machine.

## Related Specifications

- [l1-parallel-staffing.md](l1-parallel-staffing.md) — the **scheduling** half: PS-2 achieves parallelism only through decomposition, PS-5 isolates siblings, PS-6 owns the fan-in. Isolation prevents siblings from *interfering*; it does not prevent a sibling from depending on **when** it ran. OIP is the production-side property that makes PS's fan-in lossless rather than merely orderly.
- [l1-incremental-execution.md](l1-incremental-execution.md) — the closest sibling and a useful contrast: IE-4 gates *memoization* on a step being a deterministic function of its declared inputs. OIP applies the same instinct **across positions of one artifact**, and adds what memoization never needs — a global index, uniform partitioning, and a seam-safe join.
- [l1-recursive-decomposition.md](l1-recursive-decomposition.md) — RD-7's deterministic slicing is the precondition OIP-5 builds on: the same input must always partition the same way, or two runs are not comparable and a re-produced range does not fit back.
- [l1-execution-graph.md](l1-execution-graph.md) — EG-12's deferred asynchronous execution with correlated resumption is the mechanism OIP-9 constrains: a named, bounded hold that gates one unit's completion.
- [l1-reproduction-recipe.md](l1-reproduction-recipe.md) — RR-8 records the determinism controls in force; OIP-2(c) is where the seed acquires its structure (derived from position, so it is stable per unit and different across units).
- [l1-competitive-execution.md](l1-competitive-execution.md) — **deliberately different parallelism**: there, N attempts produce *alternatives* and one wins; here, N producers make *disjoint parts* and all of them survive into the result. Confusing the two produces either a discarded majority or an incoherent join.
- [l1-change-merge.md](l1-change-merge.md) — merging *edits* against a base is a different problem from assembling *disjoint parts* of one artifact: OIP's units never overlap by construction, so its join has an order but no conflict resolution.
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — the OIP-2 prohibitions (wall-clock, unseeded randomness, live reads inside a unit producer) are exactly the shape a structural check catches, and exactly the shape a behavioural test does not.
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.6): a positional loop is the natural carrier, the deferred-step mechanism is the named hold, and the validation stage can flag the prohibitions statically.

## 1. Motivation

**The capability is bought entirely by the constraint.** Arbitrary concurrency, resumption,
partial repair, and machine-independent results are not four features to be built — they are
four consequences of one property. A system that holds the property gets them for free; a
system that does not cannot obtain any of them at any price, because each one assumes that a
unit can be made in isolation.

**The failure is silent, and it is located where nobody inspects.** Each unit passes its own
validity check, because each unit is internally fine. What is wrong is the *relationship*
between units, and relationships have no per-unit gate. The artifact reads as complete,
ships, and is discovered later by whoever notices that the numbering restarts, that a
running total resets, that the tone changes at a boundary, or that two sections describe the
same state differently.

**The intuitive fix is worse than the defect.** Faced with inconsistency, the first move is
to stop producing in parallel — and it works, apparently. It works because serial production
supplies the very ordering the units were illegitimately relying on. Nothing was repaired:
the artifact is now correct by coincidence, the concurrency was surrendered, and the result
is still machine-dependent. Naming this explicitly is most of the value of the spec, because
the wrong fix is the one a careful person reaches for first.

**Order-dependence enters through four ordinary conveniences.** Continuing from the previous
unit's leftover state; reading the clock; drawing a random value; fetching a source at
production time. None of these looks like a mistake, and each one, used once, converts a
distributable artifact into a serial one.

**The join is a hazard in its own right.** Parts that are individually correct can still be
lossy or artefacted where they meet, depending on how they were produced. A production
format chosen without regard to concatenation turns a correct parallel run into a subtly
corrupt artifact, and the corruption is concentrated precisely at the boundaries the
parallelism created.

## 2. Constraints & Assumptions

- The concept applies to an artifact that **decomposes into positioned units** with a
  declared total extent. Work that is one indivisible unit is out of scope.
- Units are **disjoint**: two units never claim the same position. Overlapping contributions
  are a merge problem, owned elsewhere.
- The input set is **frozen for the run**. This spec does not define how it is frozen; it
  requires that it is, and that its identity is shared.
- Position may be any total order the artifact declares — an index, a range, a page, a key.
  Nothing here presumes time.
- Detection of a violation is assumed to be possible only **across** units (OIP-7); the spec
  does not require that every violation be mechanically detectable.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **OIP-1 A unit is a pure function of its position and the frozen input set**: the content
  of the unit at position *i* is determined by **i** and by the run's frozen inputs, and by
  nothing else. This is the whole concept; every other invariant either follows from it or
  protects it. A producer that cannot state its output as such a function has not earned any
  of the capabilities the rest of this spec describes, whatever its concurrency settings say.

- **OIP-2 Four named prohibitions inside a unit producer**: a producer MUST NOT (a) depend on
  **production order** or on any residue left by a neighbouring unit; (b) depend on **elapsed
  or absolute wall-clock time** — position is the clock, and a unit's content may not change
  because it was made later; (c) consume **unseeded randomness** — a deliberately varied
  element draws from a **seed derived from the position**, so the variation is stable per
  unit and still differs across units; (d) read a **source that can change during the run**
  without pinning it into the frozen input set. Each is an ordinary convenience, each looks
  harmless once, and each alone converts a distributable artifact into a serial one.

- **OIP-3 One frozen input identity, shared identically by every producer**: every unit is
  produced against the **same** parameters and the **same** input identity, referenced by
  content. A parameter that differs between producers does not yield a faster artifact — it
  yields a **different** one, assembled from parts of two artifacts, and the difference will
  read as an inexplicable inconsistency rather than as a configuration error.

- **OIP-4 Every producer receives its global position, not merely its local extent**: a
  producer handed a partition receives both the partition's own range **and its offset in
  the whole**, because unit content is a function of the *global* index. A producer told
  only "make twenty-five units" for a partition that begins at position 100 will produce
  twenty-five wrong units and report success — the most expensive failure in this family,
  because it is invisible until assembly.

- **OIP-5 Partitions are uniform in size, with the remainder in the last**: equal partition
  size — the final partition taking whatever remains — is a **production** constraint, not a
  scheduling preference. Partition size decides where the boundaries land, and boundary
  placement is exactly what the join must reconcile; irregular partitions move the seams
  around between runs and make a re-produced range stop fitting.

- **OIP-6 The join is declared, ordered, and seam-safe**: assembly is an owned step with a
  **declared order**, and the **production form is chosen so that independently produced
  parts combine without boundary artifacts**. A form that is correct per part and lossy at
  the seam converts a correct parallel run into a subtly corrupt artifact whose damage is
  concentrated at the boundaries the parallelism introduced. Seam-safety is a property of how
  the parts were **produced**, and it cannot be recovered at assembly time.

- **OIP-7 The characteristic failure is cross-unit inconsistency, not a crash**: a violation
  yields units that are each individually valid and **mutually inconsistent**. Detection is
  therefore a **cross-unit consistency check** — continuity across boundaries, monotonicity
  of any running quantity, agreement on shared facts, uniformity of form — and a per-unit
  validity gate passes every unit of a thoroughly broken artifact. A system whose only checks
  are per-unit has no detector for this failure class at all.

- **OIP-8 Reducing concurrency is a diagnosis, never a remedy**: if producing serially makes
  the artifact correct, the units are **not** order-independent — the serial run merely
  supplied the ordering they were illegitimately relying on. Shipping the serial setting as
  the fix leaves the defect in place, forfeits every capability OIP-1 buys, and yields a
  result that still differs on another machine or under different timing. A serial run is a
  legitimate way to **confirm** the diagnosis and is never the resolution recorded against it.

- **OIP-9 Asynchronous readiness is a named, bounded hold on one unit**: where producing a
  unit requires asynchronous work, the producer takes a **named hold** that blocks *that
  unit's* completion, releases it on success, and cancels explicitly on unrecoverable
  failure. Several holds may be outstanding at once and the unit completes only when all are
  released. Two properties are mandatory: the hold carries a **name**, so a timeout is
  attributable to the work that stalled rather than to the unit as a whole; and the wait is
  **bounded**, because an unbounded hold silently converts a parallel run into a hang.

- **OIP-10 Any subset is independently re-producible**: because a unit depends only on its
  position and the frozen inputs, a single unit or range can be re-produced later, alone, and
  dropped back into place — the basis for partial repair, resumption after interruption, and
  re-deriving a damaged region without re-making the whole. A system that must re-produce
  everything to correct one unit has lost the property, and has almost always lost it to a
  violation of OIP-2.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The production contract

```text
[REFERENCE]
produce(i, frozen) -> unit                    // OIP-1: i and frozen are the ONLY inputs

run(extent, frozen, width):
    parts := partition(extent, width)          // OIP-5: uniform, remainder last
    for p in parts (in any order, anywhere):   // OIP-1 makes "any" legitimate
        emit(p.global_offset, [produce(i, frozen) for i in p.range])   // OIP-4
    return join(all parts by position)         // OIP-6: declared order, seam-safe
```

Read the loop's freedom as the specification's product: *in any order, anywhere* is not an
implementation liberty, it is the property being asserted. Every invariant here exists to
keep that phrase true.

### 4.2 The four prohibitions and their tells (OIP-2)

| Prohibited | Why it is tempting | How it shows up |
| --- | --- | --- |
| Order / neighbour residue | "continue from where the last one left off" | a running quantity resets or doubles at a partition boundary |
| Wall-clock | "stamp it with now", "animate over elapsed time" | the same position renders differently on a second run |
| Unseeded randomness | "vary it a bit" | re-producing one unit changes it; two runs disagree |
| Live source read | "fetch the current value" | boundaries disagree because the source moved mid-run |

The right form of the third row is worth stating positively: variation is legitimate, and
it is obtained by **deriving the seed from the position**. The unit is then stable when
re-produced and still different from its neighbours — variation without order-dependence.

### 4.3 Why serialization is not a fix (OIP-8)

```text
[REFERENCE]
symptom  := inconsistency across unit boundaries
observe  := run with width=1
    if symptom disappears  -> CONFIRMED violation of OIP-1        // a diagnosis
    if symptom persists    -> look elsewhere (frozen-set drift, join form)
resolution := repair the producer's purity — never "keep width=1"
```

The serial run is a **probe**, and it is a good one: it separates an order-dependence bug
from a join-form bug in one measurement. What it must never become is the recorded
resolution. Three things are lost by shipping it: the concurrency, the ability to re-produce
a damaged range (OIP-10), and machine-independence — and the third is lost *silently*, since
the artifact still looks right on the machine that made it.

### 4.4 Detecting what per-unit checks cannot (OIP-7)

Cross-unit checks that catch this class:

- **Continuity at boundaries** — the last position of one partition and the first of the
  next agree on whatever crosses them.
- **Monotonicity** — any running or accumulating quantity moves in one direction across the
  whole, never resetting at a partition boundary.
- **Shared-fact agreement** — a fact appearing in several units is identical in all of them.
- **Re-production equality** — re-producing a sampled unit yields a byte-identical result
  (this one detects all four prohibitions at once and is the cheapest high-value check).

The last is the practical recommendation: sample a few positions, re-produce them, compare.
It costs a fraction of a run and catches the entire failure class.

### 4.5 Boundary with the other parallelisms

| | Units | Outcome | Join |
| --- | --- | --- | --- |
| **Order-independent production** | disjoint parts of one artifact | **all** survive | ordered assembly, no conflicts |
| **Competitive execution** | alternative attempts at the same thing | **one** survives | selection, the rest discarded by design |
| **Change merge** | overlapping edits to a common base | reconciled | conflict resolution required |

Choosing the wrong model has a characteristic cost: treating disjoint parts as competitors
discards most of the work; treating alternatives as parts assembles an incoherent artifact;
treating either as a merge invents conflicts that cannot occur.

### 4.6 nodus projection

No new language primitive is required:

1. **A positional loop is the carrier.** A loop over a finite collection is intrinsically
   bounded, and its loop variable *is* the position — so OIP-1 states exactly that the loop
   body may read the loop variable and the workflow's declared inputs, and nothing else.
2. **The deferred-step mechanism is the named hold.** The runtime already lets a step suspend
   pending an externally-supplied completion with correlated resumption; OIP-9 adds only the
   two obligations — the hold is named, and the wait is bounded — both of which sit
   comfortably in the existing error taxonomy as a typed timeout.
3. **The prohibitions are statically flaggable.** A loop body that reads a live host source
   or draws an unseeded value is detectable at the validate-before-run stage — the same stage
   that hosts the project's structural checks — which turns three of the four prohibitions
   from review advice into a pre-execution refusal. Uniform partitioning, identical
   parameters, and the global offset stay host-side, consistent with how every other
   orchestration concern maps onto the provider surface.

## 5. Implementation Notes

1. Pass the global offset explicitly and early (OIP-4); a producer that has to *derive* its
   offset from context will eventually derive it wrong, and the error is invisible until
   assembly.
2. Make the position-derived seed the **only** available source of variation inside a
   producer — an ambient generator that also works is an ambient generator that will be used.
3. Add the re-production equality check (§4.4) to the standard run, sampled: it is the one
   check that catches all four prohibitions and it costs almost nothing.
4. Record the width the artifact was produced at. When an inconsistency is reported later, the
   first question is whether width was ever greater than one, and the answer should not
   require re-running anything.

## 6. Drawbacks & Alternatives

- **The constraint is genuinely restrictive.** Some work is honestly sequential, and forcing
  it into this shape is worse than accepting a serial pipeline. The remedy is to say so at
  design time rather than to declare a sequential producer order-independent and discover it
  at the seams.
- **Deriving a seed from position is more ceremony than calling a generator.** Accepted: it
  is a few lines against the entire re-producibility and repair story, and §5.2's advice
  (remove the ambient alternative) is what makes the ceremony stick.
- **Cross-unit checks cost a pass over the whole artifact.** Bounded by sampling (§4.4);
  full-artifact checking is available when the stakes justify it.
- **Alternative — just produce serially and avoid the whole subject.** Rejected as a
  *general* answer by OIP-8: it is a legitimate choice for genuinely sequential work, and a
  silent defect whenever it is adopted as a fix for an inconsistency.
- **Alternative — reconcile inconsistencies at assembly time.** Rejected by OIP-6/OIP-7:
  assembly sees only the parts it was given, cannot know what a correct neighbour would have
  contained, and a seam artifact is not recoverable after production.
- **Alternative — fold into parallel staffing.** Rejected: PS governs *who* works and how the
  fan-in is owned; this governs *what a producer may depend on*. A perfectly staffed,
  perfectly isolated set of workers still produces an incoherent artifact if every one of
  them reads the clock.
- **Alternative — fold into incremental execution.** Rejected: IE-4's determinism gate
  protects a *cache*; it needs no global index, no uniform partitioning, and no join. The
  overlap is the instinct, not the contract.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[STAFFING]` | `.design/main/specifications/l1-parallel-staffing.md` | The scheduling half; PS-2/PS-5/PS-6 boundary. |
| `[INCREMENTAL]` | `.design/main/specifications/l1-incremental-execution.md` | IE-4 determinism gate — the closest sibling contract. |
| `[DECOMPOSITION]` | `.design/main/specifications/l1-recursive-decomposition.md` | RD-7 deterministic slicing, the precondition for OIP-5. |
| `[GRAPH]` | `.design/main/specifications/l1-execution-graph.md` | EG-12 deferred execution, the mechanism OIP-9 constrains. |
| `[RECIPE]` | `.design/main/specifications/l1-reproduction-recipe.md` | RR-8 determinism controls; where the seed is recorded. |
| `[COMPETITIVE]` | `.design/main/specifications/l1-competitive-execution.md` | The other parallelism (§4.5) — alternatives, not parts. |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the concept projects onto (§4.6). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — order-independent production: the property that makes splitting a multi-unit artifact safe, which the staffing and decomposition specs assumed without ever stating. The unit at position *i* is a pure function of *i* and a frozen input set and of nothing else (OIP-1), enforced by four named prohibitions — production order or neighbour residue, wall-clock, unseeded randomness (variation comes from a **position-derived seed**), and live sources not pinned into the frozen set (OIP-2); one frozen input identity shared identically, since a differing parameter yields a *different artifact* assembled from parts of two (OIP-3); every producer receives its **global** offset, not merely a local extent, or it produces wrong units and reports success (OIP-4); uniform partitions with the remainder last, a production constraint because partition size decides where seams land (OIP-5); a declared, ordered, **seam-safe** join, seam-safety being a property of how parts were produced and unrecoverable at assembly (OIP-6); the characteristic failure is **cross-unit inconsistency, not a crash** — each unit individually valid, so a per-unit gate passes a thoroughly broken artifact and detection must be cross-unit (OIP-7); **reducing concurrency is a diagnosis, never a remedy** — serial production supplies the very ordering the units illegitimately relied on, so shipping it leaves the defect, forfeits the capabilities, and stays machine-dependent (OIP-8); asynchronous readiness as a **named, bounded** hold on one unit, the name making a timeout attributable and the bound preventing a parallel run from becoming a hang (OIP-9); and any subset independently re-producible, the basis for partial repair and resumption (OIP-10). Nodus projection needs no new primitive — a positional loop is the carrier, the deferred-step mechanism is the named hold, and three of the four prohibitions are statically flaggable at the validate-before-run stage. Concept-only. |

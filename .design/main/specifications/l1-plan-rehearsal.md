# Plan Rehearsal

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Once a plan exists and before any of it is built, the office **plays the plan out** — not
the journeys a person will take through the finished product, but the **seams between the
planned units**: what one unit hands the next, what the next does when the hand-off is
malformed, who owns the recovery when a unit fails after its neighbour already committed,
and whether the declared build order actually leaves each unit buildable when its turn
arrives.

Nothing executes, because nothing exists yet. The play-out is **reasoned from the plan's
own declarations**, and its first finding is always about those declarations: a seam that
cannot be reasoned about is a seam that was not specified, and that is information about
the plan, available at the one moment it costs a sentence to fix.

This is the second of the two things planning owes the work. Scenario derivation asks
*"can a person get this done, and how would we know?"* and produces the journeys. This
asks *"do these units actually fit together, including when one of them fails?"* and
produces findings against the plan. Both are derived before the code, from the same
planning act, and neither substitutes for the other — a plan can pass one completely while
failing the other, because they are blind to different things.

## Related Specifications

- [l1-scenario-derivation.md](l1-scenario-derivation.md) — the co-product and nearest sibling. SD's subject is the **person's journey** across units; this spec's subject is the **machine's seams** between them. SD-3 routes an underivable obligation upward to the *requirement*; PR-2 routes an unrehearsable seam back to the *plan*. Two upward paths, two different desks.
- [l1-task-graph-model.md](l1-task-graph-model.md) — the artifact under rehearsal. TG-3's per-unit verification strategy is what a seam finding is *not* (it verifies a unit; a seam has no owning unit), and TG-9's drift-driven re-planning is what PR-9 rides on.
- [l1-simulation.md](l1-simulation.md) — the sibling that plays out a **generated mechanism that exists** and can be walked, with effects suppressed (SIM-2). A plan cannot be walked, so this is not that: SIM-5 already names a task graph as an addressable mechanism, but its structural fidelity parses and traverses, where a plan must be *reasoned about* because the thing it describes has not been written.
- [l1-usage-simulation.md](l1-usage-simulation.md) — the instrument that answers the same family of questions against the **built** product. A rehearsal finding that survives to the shipped surface becomes that instrument's problem; the point of rehearsing is that most of them should not survive that far.
- [l1-development-workflow.md](l1-development-workflow.md) — the five-stage pipeline. Rehearsal closes the Plan stage; DW-2's design gate is where its findings are cheapest, and DW-4's two verdicts are a per-task lens that no seam falls under.
- [l1-remedy-authority.md](l1-remedy-authority.md) — what may be done about a rehearsal finding. PR-6 keeps the instrument from editing its own subject; RA governs who edits it afterwards and on what proof.
- [l1-acceptance-oracle.md](l1-acceptance-oracle.md) — AO-1: criteria precede the work. A seam finding raised before implementation is a criterion the implementation is then judged against, rather than an observation about what got built.
- [l1-exploratory-planning.md](l1-exploratory-planning.md) — the stage before a requirements artifact exists. Rehearsal has nothing to read until planning has produced a decomposition.
- [l1-lookahead-planning.md](l1-lookahead-planning.md) — predicts **one proposed action's** consequence to gate a commit, inside a run. This plays out a **whole planned decomposition** before any run begins; the grain and the moment are both different.
- [l1-whole-system-rehearsal.md](l1-whole-system-rehearsal.md) — the same question asked of the assembled system rather than of the plan. The two bracket the build: one before it, one repeatedly after.

## 1. Motivation

**Units are verified; seams are not, and seams are where the plan's real errors live.**
Every planned unit declares how it will be verified, and a decomposition can satisfy that
rule perfectly while containing a hand-off that neither side owns, an ordering that makes
a unit unbuildable when its turn arrives, or a failure with no declared owner. Each of
those is invisible to per-unit verification by construction — it is not *in* a unit — and
each is discovered, absent rehearsal, by the implementer of the unit that happens to hit
it, mid-build, at the worst moment and by the least-equipped person.

**The failure case is where planning is thinnest.** A plan describes what each unit does.
It very rarely describes what the neighbour sees when the unit does not. Ask of any four
planned units what happens when the third fails after the second already committed, and
the honest answer is usually that the plan does not say — not because anyone decided it
did not matter, but because nothing in the planning act ever asks. Rehearsal is the act
that asks, and it asks while the answer is still a sentence rather than a rewrite.

**A journey does not cross every seam, and the ones it misses are not the safe ones.**
Derived scenarios follow the routes a person takes. Background work, recovery paths,
concurrent arrivals, and the state left behind by a half-completed operation are crossed
by the *system* and by no journey at all. A plan can carry a complete, well-derived
scenario set and still have no artifact whose subject is any of them.

**Reasoning about a plan is the cheapest diagnostic the lifecycle offers, and it grades
the plan while it works.** A seam that cannot be played out is a seam that was not
specified. That finding costs one clarifying sentence during planning; the same finding
after the build costs the interface, both units, and whatever was written against them.

## 2. Constraints & Assumptions

- **Rehearsal needs a decomposition.** Absent a task graph with declared unit boundaries,
  rehearsal is refused rather than improvised — the same posture derivation takes toward a
  missing requirements artifact.
- **Nothing executes.** There is no code, no world, and no effect to suppress or contain.
  The honesty of the play-out rests on the plan's declarations and on the reasoner, and
  both limits are declared rather than assumed away.
- **The seam space is combinatorial.** Every pair of units times every failure mode is not
  reachable and not worth reaching; what was covered and what was not must both be stated.
- **A rehearsal finding is about the plan, never about a person's experience.** The
  journey question belongs to derivation and stays there.
- **Findings are cheap only before the build.** Rehearsal run after implementation exists
  is a different, weaker act: the reasoner now knows which seams were actually built and
  will reason about those.
- **The record ships with the plan, not with the office.** It is versioned alongside the
  decomposition it examined and carries no reference to the planning machinery itself.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **PR-1 (The subject is the seam, and nothing executes):** a rehearsal plays out the
  **relations between planned units** — hand-offs, shared state, ordering dependencies,
  concurrent arrivals, and failure propagation — not the internals of any one unit and not
  the journeys a person takes across them. Because the units do not exist, the play-out is
  **reasoned from the plan's declarations**; a rehearsal that claims to have observed
  behaviour has claimed to observe something that has not been written.

- **PR-2 (Findings are filed against the plan; requirement defects take the other path):**
  a rehearsal finding names a defect in the **decomposition** — an unowned hand-off, an
  unbuildable order, an unhandled failure, a contradiction between two units' declared
  interfaces. A finding about what was *wanted* rather than about how it was *divided* is
  a requirement defect and travels SD-3's upward path instead. Sending a design finding to
  the requirement, or a requirement finding to the plan, guarantees it is answered by
  someone who cannot answer it.

- **PR-3 (Failure is rehearsed, not only success):** every rehearsed seam is played at
  least once with the upstream unit **failing, partial, slow, or repeated** — and the
  finding is what the downstream unit sees, what state survives, and **which unit owns the
  recovery**. A rehearsal that plays only the success path has rehearsed the case the plan
  already handles, and the missing owner is the single most common seam defect there is.

- **PR-4 (A finding is falsifiable against the plan as written):** every finding cites
  **what the plan does and does not say** — the declaration it rests on, or the absence it
  rests on. A finding that cannot be checked against the plan text is the reasoner's
  speculation about an implementation nobody has written, and admitting it teaches readers
  to discount the whole set. *"Unit B may mishandle this"* is not a finding; *"neither A
  nor B declares who retains the detached items when B fails"* is.

- **PR-5 (Bounded, with the unreached set named):** a rehearsal declares its bound — which
  seams and which failure modes it played — and **names what it did not reach**. The seam
  space is combinatorial and exhausting it is neither possible nor wanted; what is refused
  is the silent version, in which an unexamined seam and a seam examined and found sound
  are indistinguishable in the record.

- **PR-6 (A rehearsal reports; it does not edit the plan):** the deliverable is findings.
  Changing the decomposition is the planning act's own move, performed afterwards, with the
  findings as input. A rehearsal that edits as it goes has silently replaced a diagnosis
  with an unreviewed re-plan, and has removed the evidence that the original plan had the
  defect. What may then be done about a finding is `l1-remedy-authority`'s question — and
  **not on the same terms as a finding against built code**: a plan has no build for a pin
  to fail against, so RA-2 caps a rehearsal finding at *propose* outside a chain grant, and
  its resolution belongs to the planning act that owns the decomposition.

- **PR-7 (An unrehearsable seam is a planning defect, never a skipped seam):** when a seam
  cannot be played out because the plan does not say enough to reason about it, that is
  **the finding** — recorded against the plan as underspecified. It is never resolved by
  omitting the seam from the bound (PR-5) or by inventing the missing declaration and
  rehearsing against the invention, which produces a confident play-out of a plan that does
  not exist.

- **PR-8 (Rehearsal closes the Plan stage, before the design gate):** a rehearsal runs on
  the decomposition **before implementation begins**, and its findings are resolved or
  explicitly accepted at the gate where the plan is approved. Run later it still has value
  and loses its main one: after the build, the reasoner reasons about the seams that were
  actually built, which is precisely the set that needed no rehearsing.

- **PR-9 (The record is a versioned companion of the plan, and re-planning re-rehearses):**
  the rehearsal record lives with the decomposition, versioned alongside it. When
  implementation drift causes downstream units to be re-planned, the seams touching those
  units are **re-rehearsed in the same act**, and a seam that passes through a re-plan
  unexamined is **flagged for confirmation** rather than assumed still sound — the same
  silent-survivor hazard SD-6 names, at the seam grain.

> L2 specs cannot reach RFC status until all invariants here are addressed in their
> "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 What a seam is (PR-1)

```text
[REFERENCE]
a seam is any relation the plan declares between two or more units:

  hand-off        A produces something B consumes
  shared state    A and B both read or write the same thing
  ordering        B is unbuildable, or meaningless, until A exists
  concurrency     A and B may be in flight at the same time
  propagation     A's failure reaches B, or is meant not to

a seam is NOT:
  a unit's internals          — TG-3's per-unit verification strategy covers it
  a person's route            — l1-scenario-derivation covers it
  one action's consequence    — l1-lookahead-planning covers it, inside a run
```

The list is closed on purpose. Without it, "rehearse the plan" expands until it means
"imagine the whole product", which produces volume rather than findings and exhausts the
patience the discipline needs to survive its second use.

### 4.2 The failure repertoire (PR-3)

| Played against the upstream unit | The rehearsal asks |
| --- | --- |
| fails outright | what does the downstream unit see, and does it say so? |
| fails **after** committing part of its work | what state survives, and **who owns putting it back**? |
| returns something malformed | is it rejected, or consumed as if valid? |
| is slow, or never answers | does the downstream unit wait forever, or does the plan say? |
| runs twice | is the second run harmless, or does it double the effect? |
| runs at the same time as its neighbour | do they contend for the same thing, and does the plan say who wins? |

The second row is the one that finds the most and is asked the least. Total failure tends
to be handled because it is easy to imagine; *partial* failure leaves a state nobody
planned for and an owner nobody named.

### 4.3 A worked seam (PR-2 / PR-3 / PR-4)

Take the same four planned units a derived journey would cross:

| Unit | Its verification strategy |
| --- | --- |
| create a board | a board exists after the create call |
| list boards | the listing returns what was created |
| add a card | the card is attached to the board |
| archive a board | the board leaves the active set |

A derived scenario crosses these as a person: keep three things going, stop caring about
one, check the other two survived. That is the journey question and it is well covered.

Rehearsal asks a different one and finds something the journey cannot:

```text
[REFERENCE]
seam        archive a board  ->  the cards attached to it
mode        partial failure: the cards are detached, then archiving fails

finding     neither unit declares who retains the detached cards.
            "archive a board" declares its post-condition as the board leaving
            the active set; it says nothing about a run that does not reach that
            post-condition. "add a card" declares attachment and says nothing
            about detachment it did not perform.

grounded in the plan says (archive): the board leaves the active set
            the plan says (add):     the card is attached to the board
            the plan does not say:   what holds a card whose board neither
                                     archived nor remained

not a finding about the requirement — the requirement wanted the board gone,
and it still does. it is a finding about the division of the work.
```

No journey reaches this: a person archiving a board does not experience the archive
failing halfway, and if they did, the scenario's obligations are about their two surviving
boards rather than about the orphaned cards of the third. The seam is crossed by the
system and by nobody else, which is exactly the class PR-1 exists to cover.

### 4.4 Bound and the unreached set (PR-5)

```text
[REFERENCE]
a rehearsal record states:

  played      the seams examined, each with the failure modes played against it
  unreached   the seams NOT examined, and why — out of bound / judged low risk /
              unrehearsable (which is itself a PR-7 finding, not an exemption)
  findings    each grounded per PR-4, each filed against the plan per PR-2

what is refused: a record in which "examined and sound" and "never looked at"
                 render identically. an unreached seam is a known unknown; an
                 undeclared one is an unknown the reader will mistake for coverage.
```

### 4.5 Demarcation

| Artifact | Subject | Asks | When |
| --- | --- | --- | --- |
| **Plan rehearsal** (this spec) | the seams between planned units | do these fit together, including when one fails? | closing the Plan stage |
| Derived scenario set | the promised behaviour, across units | can a person get this done, and how would we know? | with the plan |
| Per-unit verification strategy | one unit in isolation | does this piece do its job? | with the task |
| Simulation of a mechanism | a generated mechanism that exists | how does this thing behave when played out? | once it exists |
| Lookahead | one proposed action, inside a run | should I commit this next action? | at the commit point |
| Usage-simulation run | the built product | what happens when someone tries? | at review, and after |

The three that plausibly collapse into each other are the first, second, and fourth. The
distinguishing test is what the subject *is at that moment*: a plan is a set of
declarations that cannot be walked, a mechanism is an artifact that can, and a journey is
a person's path across whatever got built. An instrument that treats a plan as though it
could be walked will report a confident play-out of an implementation it invented.

## 5. Drawbacks & Alternatives

- **A reasoned play-out can invent seams that do not exist, or miss ones that do.** This is
  the real limit and PR-4 is the only defence: a finding must cite the plan's own text or
  a specific absence in it, which makes an invented seam visible as one. The miss side has
  no structural defence at all — PR-5's declared bound is an honesty measure, not a
  completeness one.
- **Volume is the likely failure in practice.** A reasoner asked for seam findings will
  produce one per unit pair per failure mode, which is both expensive and unreadable. The
  closed seam definition (§4.1) and the declared bound are the pressures against it, and
  they are real design constraints rather than stylistic notes.
- **Alternative — fold seam questions into each unit's verification strategy.** Rejected
  (the same reason SD-2 gives for journeys): a seam has no owning unit, so it would be
  duplicated across both sides or arbitrarily assigned to one, and both produce the drift a
  single artifact avoids.
- **Alternative — rehearse after implementation, where there is something real to run.**
  Rejected as the primary placement (PR-8): at that point the seams that were built are the
  only ones that can be examined, and the seams that were never built are exactly the
  finding. Retained as a secondary use against the assembled system, which is a different
  contract.
- **Alternative — treat this as a fidelity level of mechanism simulation.** Rejected
  (§4.5): that contract's cheapest level parses and traverses a mechanism that exists. A
  plan supports neither, and stretching the contract to cover it would make "structural
  fidelity" mean two incompatible things in one vocabulary.
- **The gate's teeth are unspecified.** Whether an unresolved seam finding blocks the design
  gate or is merely recorded at it is left to the workflow that owns the gate. <!-- TBD: whether an unowned-recovery finding specifically should be blocking, given it is both the most common and the most consequential class -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[PLAN]` | `.design/main/specifications/l1-task-graph-model.md` | The decomposition under rehearsal; TG-3 per-unit field, TG-9 drift that PR-9 rides. |
| `[JOURNEYS]` | `.design/main/specifications/l1-scenario-derivation.md` | The co-product; SD-3's upward path that PR-2 deliberately parallels rather than shares. |
| `[MECHANISM]` | `.design/main/specifications/l1-simulation.md` | SIM-5/SIM-3 — the play-out contract for mechanisms that exist, which a plan is not. |
| `[WORKFLOW]` | `.design/main/specifications/l1-development-workflow.md` | The five stages; DW-2's design gate where PR-8 places the findings. |
| `[REMEDY]` | `.design/main/specifications/l1-remedy-authority.md` | What may be done about a rehearsal finding once PR-6 has refused to do it. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-13 | Core Team | Initial spec — playing the plan out before the code exists, as the seam-side companion to scenario derivation's journey side. The subject is the relation between planned units and nothing executes, so the play-out is reasoned from the plan's declarations (PR-1); findings are filed against the decomposition while requirement defects keep SD-3's separate upward path (PR-2); every rehearsed seam is played with the upstream unit failing, partial, slow or repeated, and the finding names who owns the recovery (PR-3); a finding cites what the plan does or does not say, so an invented seam is visible as one (PR-4); the bound is declared and the unreached seams named, since an unexamined seam and a sound one must not render identically (PR-5); the rehearsal reports and never edits the plan, leaving the remedy question to `l1-remedy-authority` (PR-6); a seam that cannot be reasoned about is itself the finding, never a skipped seam or an invented declaration (PR-7); rehearsal closes the Plan stage before the design gate, because afterwards only the seams that were built can be examined (PR-8); the record is versioned with the plan and re-planning re-rehearses, flagging silent survivors at the seam grain (PR-9). Post-Update Review (Spec Council) corrected one cross-spec defect: PR-6 had routed rehearsal findings to `l1-remedy-authority` "on the same terms as any other finding", which was wrong — that contract's RA-2 requires a pin failing against the current build, and a plan has no build, so the terms are materially different. Both sides were fixed in the same pass: RA-2 gained the unbuilt-subject cap, and PR-6 now states the difference instead of asserting sameness. `[DR]` Promoted `Draft → Stable` in the authoring pass under Trust Mode (MVC satisfied, no RULES conflict, no hard-dependency cycle, Council pass adversarial and consequential). |

# Instruction Economy

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

The project bounds bloat at two grains already. Harness composition prunes **components**: a role, a tool, a check earns inclusion by compensating a real capability gap and is removed when the gap closes. Capability reachability prunes **descriptors**: a capability keeps a resident trigger only when something other than a person must reach it. Neither reaches the grain where standing instructions actually accumulate — **the sentence**.

That grain has its own economics, and they are not the component ones scaled down. A line in a standing instruction is charged on every turn whether or not it fires, competes for attention with every other line, and is almost never removed, because adding one feels safe and deleting one feels like discarding a lesson. What survives is therefore not what is live but what nobody dared delete.

The test this concept installs is narrow and unusual: a line earns its place by **changing the actor's behaviour against its own default**. That default is a property of the actor, not of the reader, which makes the test empirical rather than editorial — two people disagreeing about whether a line does anything are disagreeing about a fact, settled by running the instruction both ways. It also makes the verdict **perishable**: swap the actor and the default moves, so the no-op set is re-measured rather than inherited.

## Related Specifications

- [l1-harness-composition.md](l1-harness-composition.md) — the component-grain sibling. HC-4 dedups against what the **host** already provides; IEC-1 dedups against what the **actor** already does. Two different baselines, and the second one moves.
- [l1-capability-reachability.md](l1-capability-reachability.md) — the capability-grain sibling: REA decides whether a descriptor exists at all, this decides whether each line inside one earns its place.
- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — IEC-11's structural remedy is disclosure: material only some paths need moves behind a pointer rather than being trimmed in place.
- [l1-derived-instructions.md](l1-derived-instructions.md) — DIN-1 renders the derivable half from a declaration; IEC-4 is the reason that matters economically — a restated declaration is a cache of a lookup that is not expensive.
- [l1-harness-optimization.md](l1-harness-optimization.md) — HX's search accepts a candidate on measured behaviour; IEC-2 applies the same discipline to a single line, and supplies the measurement HX would otherwise have to guess at.
- [l1-negative-specification.md](l1-negative-specification.md) — NEG-5 requires a negative to name its alternative; IEC-7 states the mechanism and hardens it into an ordering: the positive target is the primary form, the prohibition the fallback.
- [l1-project-vocabulary.md](l1-project-vocabulary.md) — VOC governs which term is canonical; IEC-8 governs whether coining a term is worth its definition cost at all.
- [l1-roles.md](l1-roles.md) — the roles IEC-6 places standing rules across; placement by capacity is a property of the role, not of the rule.
- [l1-attention-steering.md](l1-attention-steering.md) — the mechanism IEC-7 and IEC-9 exploit; what is in context is what competes, whether it was stated as a target or as a ban.
- [l1-model-adaptation.md](l1-model-adaptation.md) — IEC-2's perishability: an actor change invalidates the no-op set, so re-measurement belongs to the adaptation path.

## 1. Motivation

Standing instruction sets degrade in a way that is invisible from inside them, because every individual line looks defensible:

- **Lines that do nothing.** Guidance the actor already follows unprompted — be careful, be thorough, write clear code — is charged every turn and changes nothing. Worse than free: it dilutes the lines beside it that do change behaviour.
- **Restated environment.** Commands, configuration, and layout are copied into prose from the declarations that already define them. The copy is authoritative for nobody, goes stale on its own schedule, and displaces the guidance that could not have been looked up.
- **Bans that summon.** A rule phrased as a prohibition puts the forbidden thing into context, where it competes as any other available behaviour does, and the negation is the weakest word in the sentence. The rule reads as caution and behaves as suggestion.
- **Rules loaded onto the busiest actor.** Standards, conventions, and review criteria are placed with the role that is simultaneously exploring, producing, and debugging, because that is the role that writes the code. It is also the role with the least attention to spare, so the rules are skipped exactly when the work is hardest.
- **Sediment.** Nothing is ever removed. A reader has to core down through stale layers to find the guidance that still applies, and eventually stops reading and works from priors — at which point the whole document is a no-op.
- **Sprawl.** A document grows past the point where attention holds, while every line in it is live, unique and true. Line-level editing cannot fix this, and repeated attempts at it are how a bloated document becomes a bloated document written tersely.

## 2. Constraints & Assumptions

- **The actor has a default.** There is behaviour it produces without any instruction at all. That default is observable and is the baseline everything here measures against.
- **The default is not stable across actors.** A different model, version, or configuration has a different default. Any verdict derived from a measurement is scoped to the actor it was measured on.
- **Behaviour can be observed.** Running an instruction with and without a line, and comparing what happened, is available. Where it is not, this contract degrades to judgement, and says so rather than pretending to a measurement.
- **Standing instructions are resident.** They are loaded whether or not they apply; if a body of guidance is loaded on demand, it is disclosure's subject, not this one's.
- **Removal is safe by construction.** A line's removal can be undone. The asymmetry that produces sediment is psychological, not technical, and the scheduled review (IEC-10) exists to counter it.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **IEC-1 (A line earns its place by changing behaviour against the actor's default):** the test for any standing instruction is whether the actor behaves **differently with it than without it**. A line the actor already obeys unprompted is a **no-op**: it spends standing cost to say nothing, and it dilutes the lines around it that do work. Plausibility, agreement, and good intentions are not the test; a behavioural difference is.

- **IEC-2 (The no-op verdict is actor-relative, measured by running, and expires with the actor):** the default IEC-1 measures against is a property of the **actor**, not of the reader. Two people disagreeing about whether a line is a no-op are disagreeing about that default, and the disagreement is settled by **running the instruction with and without the line**, never by argument. Three consequences bind. A verdict names the actor it was measured on. **An actor change invalidates it** — a swapped or upgraded model has a different default, so the set is re-measured rather than inherited. And an unmeasured claim that a line is load-bearing is an opinion, recorded as one.

- **IEC-3 (A failing line is deleted whole, never trimmed):** when a line fails IEC-1 it is removed in its entirety. Shortening a no-op yields a shorter no-op, and a document edited that way converges on a dense collection of statements that still change nothing — which reads as economical and is not. Trimming is for lines that survive the test.

- **IEC-4 (The environment is a source of truth; an instruction restating it is a cache):** anything the actor can determine by looking — declared operations and their arguments, configuration values, the layout, a surface's own help output — is **already authoritative there**. An instruction that restates it is a **cache of a lookup**, and a cache earns its cost only when the lookup is expensive or impossible. What belongs in the instruction is what cannot be looked up: the unwritten convention, the reason a choice was made, the trap no declaration confesses. A restated one-lookup fact buys nothing and goes stale on a schedule its author does not control.

- **IEC-5 (One meaning, one place — duplication and scattering are opposite failures with opposite fixes):** **duplication** places one meaning in two locations: it costs maintenance, drifts, and inflates the meaning's apparent rank above its real one. **Scattering** fragments one meaning across many locations — the definition here, its constraint two sections away, its exception in a third — so reading one part does not bring its neighbours with it. The remedies point in opposite directions and MUST NOT be confused: duplication is **removed**, scattering is **co-located**. Treating scattering as duplication deletes half of a meaning; treating duplication as scattering gathers two copies into one place and keeps both.

- **IEC-6 (Standing rules are placed with the role that has attention to spare):** where work passes through roles carrying different loads — one that must explore, produce, and diagnose, and one that receives a finished delta and judges it — a body of standing rules is placed with the role that has **room to attend to it**. Loading conventions and review criteria onto the actor that is simultaneously navigating, writing, and debugging spends attention where there is least of it, and the rules are dropped precisely when the work is hardest. **Placement is decided by capacity as well as by relevance**; a rule that is relevant to a role with no attention left is a rule that will not fire.

- **IEC-7 (Steer by the target behaviour; a bare prohibition raises what it forbids):** an instruction states **what to do**. Steering by prohibition places the forbidden behaviour into context, where it becomes *more* available rather than less, with the negation acting as a weak modifier over a strongly activated concept. A prohibition is admissible only as a hard guardrail that genuinely cannot be phrased positively — and even then it is **paired** with its positive target, so attention lands on the action to take rather than on the one named to be avoided. This is the mechanism behind the exclusion contract's requirement to name the alternative, hardened into an ordering: the positive form is primary, the ban is the fallback.

- **IEC-8 (Density is bought with terms the actor already holds; a coinage pays for itself in definition):** a compact term the actor already carries anchors a region of behaviour in one word, because it recruits meaning nobody had to supply. A **coined** term recruits nothing: its definition is paid for in the document, and every use of it depends on that definition having been read and retained. Reach for an existing term first; coin only where no existing one carries the distinction, and then define it **once**, in the place the vocabulary contract designates. A term too weak to beat the default is itself a no-op under IEC-1, and its fix is a **stronger term**, not more sentences around a weak one.

- **IEC-9 (A completion criterion is both checkable and demanding, and its demand is the work it produces):** an instruction that ends in a state of doneness names the condition that makes it done, and two properties of that condition are separately load-bearing. **Checkable** — the actor can tell done from not-done; a vague bound invites stopping while work remains, with attention sliding to *being done*. **Demanding** — the wording determines how much digging happens beneath it: a criterion naming the exhaustive set produces thorough work where one naming an artifact does not. Demand is not confined to sequences: an exhaustiveness bar binds a flat body of rules exactly as it binds a series of steps.

- **IEC-10 (Accumulation is the default fate; pruning is scheduled, not incidental):** instruction sets grow monotonically because adding feels safe and removing feels risky, so what survives is what nobody dared delete rather than what is still live. A standing instruction set therefore carries a **scheduled** review whose explicit output includes **removals**, measured under IEC-2. A review that produced only additions has not run; it has been attended.

- **IEC-11 (Length is a failure mode independent of per-line quality, and its remedy is structural):** a document can be too long while every line in it is live, unique, and non-obvious. Attention thins across the excess, and each additional line is one more that has to be kept true. Line-level editing cannot fix this and repeated attempts produce a bloated document written tersely. The remedy is **structural**: push what only some paths need behind a pointer, and split by branch or by sequence so each path carries only what it needs.

- **IEC-12 (A tiered instruction set names the condition that opens each deeper tier, and the condition is a trigger, not an appetite):** [ADDED v1.1.0] IEC-11's structural remedy for length is a **bounded primary path** plus deeper documents held in reserve. That structure only works if each deeper tier declares **the specific condition under which it is read** — a named diagnostic the primary path cannot resolve, a user request for a capability the primary path does not cover, a stated number of failed focused repairs. "Read this when you need more detail" is not such a condition: *need* is unfalsifiable, every actor reads everything to be safe, and the tiering that was supposed to reduce the standing cost restores it in full while adding a second place for the same rule to drift. Two rules bind it. The primary path states, positively, **which files are read for an ordinary task and which are not** — an enumerated reading set, since a prohibition alone leaves the actor guessing at the boundary (IEC-7). And a tier whose opening condition is a **failure count** states the count, so escalation is a fact about the work rather than a judgement about how stuck the actor feels.

## 4. Detailed Design

### 4.1 Admitting a candidate line

```text
admit(line, actor):
    if actor.default already produces the behaviour:      reject   # IEC-1 no-op
    if the fact is discoverable by looking:               reject   # IEC-4 cheap cache
    if the meaning already exists elsewhere:              co-locate or drop  # IEC-5
    if phrased as a ban and a positive target exists:     rewrite as the target  # IEC-7
    if the role holding it has no attention to spare:     re-place  # IEC-6
    otherwise: admit, and record the actor it was measured on   # IEC-2
```

The order matters only in that the first two tests are the cheapest and reject the most.

### 4.2 The measurement

IEC-2's test is small enough to run routinely:

1. Fix a representative task and the actor under test.
2. Run it with the instruction set **including** the line; record what happened.
3. Run it with the line **removed** and nothing else changed; record what happened.
4. **No observable difference** ⇒ the line is a no-op *for this actor*, and IEC-3 deletes it whole.

Two properties of this procedure are the point. It is decidable, so a disagreement terminates. And its result is **scoped and perishable** — recorded against the actor, invalidated when the actor changes, and re-run rather than inherited.

### 4.3 Placement by context pressure

| Role | What it carries | Attention available | What belongs here |
| --- | --- | --- | --- |
| Producing | Exploration, writing, diagnosing failures | Least | The few rules that must hold *while* producing |
| Reviewing | A finished delta and its context | Most | The body of conventions and criteria |
| Deciding | A prepared brief and a bounded choice | Moderate | Rules about the decision, not about the work |

IEC-6 is not an argument that standards do not matter to the producer. It is an argument that a standard the producer has no room to attend to is enforced nowhere, and that the role receiving a finished delta has both the room and a complete view of what to check it against.

### 4.4 Cache or content

IEC-4's test, applied to a candidate line:

| The line states | Discoverable by looking? | Disposition |
| --- | --- | --- |
| An operation's name, arguments, or output shape | Yes, in its declaration | Drop; point at the declaration |
| A configuration value or default | Yes, in the configuration | Drop |
| Where something lives in the layout | Yes, in the layout | Drop |
| Why the layout is that way | No | Keep |
| A convention nothing declares | No | Keep |
| A trap the declaration does not confess | No | Keep — this is the highest-value class |

### 4.5 Failure modes named

| Failure | Tell | Which invariant closes it |
| --- | --- | --- |
| No-op | Removing the line changes nothing | IEC-1, IEC-2 |
| Trim theatre | Shorter lines, same behaviour | IEC-3 |
| Stale cache | The instruction and the declaration disagree | IEC-4 |
| Split meaning | A rule read alone gives the wrong answer | IEC-5 |
| Misplaced standard | A rule everyone endorses and nobody follows | IEC-6 |
| Summoned behaviour | The banned thing appears more often, not less | IEC-7 |
| Definition tax | A coined term re-explained at each use | IEC-8 |
| Early stop | Work reported done with the criterion unmet | IEC-9 |
| Sediment | Nobody can say which lines are still live | IEC-10 |
| Sprawl | Every line defensible; the document unread | IEC-11 |

## nodus-relevance mapping

- **The schema is the environment.** Authoring guidance for the language must not restate what the schema already declares — construct names, arguments, allowed shapes are one lookup away and go stale the moment the schema moves. What guidance owes the author is the part the schema cannot carry: which construct to reach for, in what order, and what a good workflow looks like.
- **Construct naming buys density.** IEC-8 applies directly to the language's own vocabulary: a keyword whose meaning an author already holds needs no explanation, while a coined one is paid for in documentation at every encounter and in every misuse.
- **Diagnostics steer positively.** A validation message that names what to write is IEC-7 at the error grain; one that only names what is forbidden leaves the author holding the prohibited form as the most available thing in mind.

## 5. Implementation Notes

1. **Record the actor with every admitted line** (IEC-2). Without it, the set cannot be invalidated on an actor change, and the perishability rule becomes unenforceable.
2. **Make the review produce a removal list** (IEC-10). A review template whose output is only "additions" institutionalizes the accumulation it was meant to counter.
3. **Run the measurement on a small fixed task set.** The point is a decidable comparison, not a benchmark; a handful of representative tasks settles most lines.
4. **Treat co-location as a first-class edit** (IEC-5). Gathering a scattered meaning under one heading is the fix, and it is easy to misfile as duplication and half-delete.
5. **Split by branch before trimming again** (IEC-11). When a document is long and every line survives IEC-1, further editing is the wrong tool; disclosure and splitting are the right ones.
6. **Do not measure taste.** Lines that encode a preference the project has chosen — a house style, a naming convention — are decided, not measured. IEC-1 governs whether they *change behaviour*; whether the project wants that behaviour is a different question and is not settled by running anything.

## 6. Drawbacks & Alternatives

- **Measurement costs runs.** Real. Bounded by §5.3 and by IEC-1's cheap pre-tests: most rejected lines are rejected as restated environment or as obvious defaults without any run at all.
- **The no-op set expires.** By design (IEC-2), and inconvenient: an actor upgrade invalidates prior verdicts. The alternative is worse — carrying lines that were load-bearing for a model nobody uses any more, and calling that a standard.
- **Deleting a line that was doing something subtle.** Mitigated by IEC-2's procedure being an observation rather than a judgement, and by removal being reversible. A subtle effect that no run can show is, for this contract's purposes, not yet demonstrated.
- **Alternative — judge lines by reading them.** Rejected by IEC-2: a reader's model of the default is exactly the thing in dispute, and reading has no way to settle it. This is the failure that produces documents everyone agrees with and nobody's behaviour reflects.
- **Alternative — a length budget on the instruction set.** Rejected: it prices the symptom. A budget forces trimming (IEC-3's failure) rather than deletion, and leaves no-ops in place as long as they are short.
- **Alternative — put every rule everywhere, so nothing is missed.** Rejected by IEC-5 and IEC-6: duplication drifts, and a rule placed with a role that has no attention for it is not enforced by being present.
- **Alternative — fold into `l1-harness-composition`.** Rejected: HC's unit is a component and its baseline is the host's native provision. This unit is a sentence and its baseline is the actor's own default — a baseline that moves when the actor changes, which HC has no notion of.
- **Alternative — fold into `l1-derived-instructions`.** Rejected: DIN governs how the derivable half of an instruction is generated and kept true. It is silent on whether the **authored** half earns its place, which is the entirety of this contract; IEC-4 is precisely the economic argument DIN-1 leaves implicit.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[COMPOSITION]` | `.design/main/specifications/l1-harness-composition.md` | The component-grain sibling and its host-native baseline |
| `[REACH]` | `.design/main/specifications/l1-capability-reachability.md` | The capability-grain sibling; whether a descriptor exists at all |
| `[DERIVED]` | `.design/main/specifications/l1-derived-instructions.md` | Generation of the derivable half; the declarations IEC-4 defers to |
| `[NEGATIVE]` | `.design/main/specifications/l1-negative-specification.md` | The exclusion contract IEC-7 supplies the mechanism for |
| `[VOCABULARY]` | `.design/main/specifications/l1-project-vocabulary.md` | Where a coined term is defined, once (IEC-8) |
| `[OPTIMIZE]` | `.design/main/specifications/l1-harness-optimization.md` | The measured-acceptance discipline IEC-2 applies at line grain |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-26 | Core Team | Initial concept — the sentence-grain member of the anti-bloat family, below harness composition (components) and capability reachability (descriptors). A line earns its place by **changing behaviour against the actor's default**, and plausibility is not the test (IEC-1); the no-op verdict is **actor-relative, settled by running rather than arguing, and expires when the actor changes** — a swapped model has a different default, so the set is re-measured, never inherited (IEC-2); a failing line is deleted whole, since trimming a no-op yields a shorter no-op (IEC-3); the environment is a source of truth and an instruction restating it is a **cache of a lookup** — cache what cannot be looked up: the unwritten convention, the reason, the trap no declaration confesses (IEC-4); duplication and **scattering** are opposite failures with opposite fixes, removal versus co-location, and confusing them deletes half a meaning (IEC-5); standing rules are placed with the role that has **attention to spare** — a body of standards loaded onto the actor that is exploring, writing and debugging is dropped exactly when the work is hardest (IEC-6); steer by the target behaviour, since a bare prohibition makes the forbidden thing *more* available and the negation is the weakest word in the sentence — bans are a paired fallback, never the primary form (IEC-7); density is bought with terms the actor already holds, a coinage is paid for in definition at every use, and a term too weak to beat the default is itself a no-op (IEC-8); a completion criterion is both **checkable** and **demanding**, and its demand is the legwork it produces, binding flat rule bodies as readily as sequences (IEC-9); accumulation is the default fate, so pruning is **scheduled** and a review that produced only additions has not run (IEC-10); length is a failure mode independent of per-line quality and its remedy is structural — disclosure and splitting, not another trimming pass (IEC-11). Concept-only. |
| 1.1.0 | 2026-09-04 | Core Team | Added IEC-12 — a tiered instruction set names the condition that opens each deeper tier, and the condition is a trigger, not an appetite. IEC-11 prescribes a bounded primary path plus reserve documents as the structural remedy for length; that structure only works if each deeper tier declares the specific condition under which it is read — a named diagnostic the primary path cannot resolve, a user request for an uncovered capability, a stated number of failed focused repairs. "Read this when you need more detail" is not such a condition: *need* is unfalsifiable, a cautious actor reads everything, and the tiering restores the full standing cost while adding a second home for the same rule to drift. Two rules: the primary path enumerates positively which files are read for an ordinary task and which are not, since a bare prohibition leaves the boundary to guesswork (IEC-7); and a failure-count trigger states the count, so escalation is a fact about the work rather than a judgement about how stuck the actor feels. From an external artifact-generation skill whose primary path forbids reading renderer, validator, or test source before a first candidate exists and opens that tier only on an unsupported diagnostic or after two focused repairs fail. |

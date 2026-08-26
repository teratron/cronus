# Exploratory Planning

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The task-graph model states its own precondition plainly: a graph is **generated from an explicit requirements artifact** (TG-1). That is the right rule, and it leaves a phase unowned — the one where no such artifact exists and none can yet be written, because the way from here to the intended end is not visible. Work of that shape arrives regularly: a greenfield effort, a large migration, a subsystem nobody has decided the shape of. Handed to a planner that requires a requirements artifact, it produces one anyway, out of guesses, and every unit below it inherits the guessing.

This concept owns that phase. Its unit of progress is a **decision**, not a deliverable: a unit completes when a question has been answered and recorded, and producing the thing itself is out of contract. Its plan is **deliberately incomplete** — beyond the units it can state, there is a declared region of questions it can tell are coming and cannot yet phrase, and the admission test between them is precise: *can the question be stated sharply now*, not *can it be answered now*. And it ends by **handing off** rather than by building: when the last decision lands, its output is collapsed into the artifact TG-1 requires, and execution begins there.

One invariant here is not about planning at all and matters beyond it. A unit declared to need a human **cannot be completed by the actor supplying the human's side of it**. That is a false completion the whole completion-verification family misses, because the actor genuinely did the work; what it fabricated was the counterparty.

## Related Specifications

- [l1-task-graph-model.md](l1-task-graph-model.md) — the downstream phase and the reason this one exists. TG-1 requires an explicit requirements artifact; XPL-1 owns the phase that produces one, and TG-15 is the refusal that keeps the two from blurring.
- [l1-work-convergence.md](l1-work-convergence.md) — CONV-1/CONV-3: no shadow work, one kind of work unit. XPL's decision unit converges on the same surface as every other stream (CONV-9); it is a completion criterion of a different shape, never a parallel queue.
- [l1-work-liveness.md](l1-work-liveness.md) — WL-1's atomic claim is exactly XPL-9's claim-before-work; XPL adds only that unclaimed-ness is part of what makes a unit takeable.
- [l1-completion-verification.md](l1-completion-verification.md) — CMP-10 is XPL-4 generalized: a completion requiring a counterparty is not satisfied by the actor's own contribution.
- [l1-facilitation.md](l1-facilitation.md) — FC-13's frontier-batched elicitation is how a converse-shaped unit is actually worked; FC-5's declared stance and FC-11's interactive/headless split are what XPL-4 makes binding at the unit grain.
- [l1-rejection-memory.md](l1-rejection-memory.md) — the adjacent record, and a boundary. REJ stores *declined proposals* across the project; XPL-7's out-of-scope record is **effort-scoped**, defined by this effort's destination, and dissolves with it.
- [l1-deep-research.md](l1-deep-research.md) — the investigate-shaped unit's mechanism, including its parallel dispatch and cost roll-up.
- [l1-lookahead-planning.md](l1-lookahead-planning.md) — the sibling that simulates *consequences of a known action sequence*; this one runs where the sequence is not yet known and simulation has nothing to simulate.
- [l1-host-native-rendering.md](l1-host-native-rendering.md) — XPL-10: structure expressed in the host's own relations is what makes the frontier visible in surfaces people already look at.
- [l1-log-legibility.md](l1-log-legibility.md) — XPL-11 at the planning grain: what a human reads is named, never enumerated by bare identifier.
- [l1-context-transition.md](l1-context-transition.md) — a decision unit is sized to one working session, so its completion is a natural boundary for the transition ladder.

## 1. Motivation

Every failure below comes from applying an execution-shaped planner to work that has not been decided yet:

- **A requirements artifact assembled from guesses.** TG-1 is honoured in form: an artifact exists, a graph is generated from it. Nothing records that half its statements were invented to satisfy the precondition, so every unit downstream carries invented scope with the authority of a plan.
- **Decisions filed as tasks.** "Decide how sessions are stored" enters as a work unit, gets picked up by an executor, and is resolved by *building one of the options*. The decision was made, unrecorded, by whoever happened to take the unit.
- **Fog pre-sliced into units.** Knowing something is coming, a planner writes units for it. The units are precise and the precision is fabricated; they are then worked, blocked, re-planned, and eventually deleted, having consumed real attention.
- **Scope creep with nowhere to put a boundary.** Something turns out to sit past what this effort is for. With only *done* and *not done* available, it stays open forever, or it is closed with no record, and it returns.
- **The simulated counterparty.** A unit needing a person is worked by an actor that produces both sides: the questions and the answers. It completes, it looks complete, and the human whose judgement the unit existed to capture was never involved. This is the most damaging failure here, and the one nothing currently detects.
- **The map consumed instead of collapsed.** The decisions land, and the units are fed straight to execution. The linked reasoning that made each decision legible is dropped, and the executing phase works from titles.

## 2. Constraints & Assumptions

- **The destination is nameable even when the route is not.** If the intended end cannot be stated at all, this phase has no scope and no termination condition; naming it is the first act, not a formality.
- **This phase is expensive and is not the default.** It is for work whose route is genuinely not visible. A well-scoped feature runs through ordinary planning; routing it here spends the cost of an exploration on something already decided.
- **A human is available for the units that declare one.** XPL-4 has no fallback: where no human is available, the unit does not complete, and saying so is the correct outcome.
- **The surrounding system can express relations between units.** Blocking and parent/child are assumed available in some form; XPL-10 governs which form, not whether.
- **The record outlives the sessions.** Multiple sessions work one effort, concurrently or over time, so everything an XPL session needs is on the record rather than in a context.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **XPL-1 (A distinct phase whose output is decisions, and whose completion signal is the pull to build):** exploratory planning **produces decisions, not deliverables**. Building the thing is out of contract here, and a unit that produced one has left the phase rather than advanced it. The phase's own completion has a **behavioural tell** worth stating because it is reliable: the **urge to just do the work** is the signal that the route is now clear and the phase should hand off (XPL-12), not a temptation to resist. An effort MAY declare an exception carrying execution into the map; absent that declaration, deliverables are out of scope.

- **XPL-2 (The destination is named first, and it is what fixes the scope):** before any unit exists, the effort names its **destination** — what reaching the end looks like: the artifact to hand off, the decision to lock, the change to have made. Scope is **derived from the destination**, never declared separately, and every session orients to it before choosing work. An effort without a named destination has no termination condition, no basis for XPL-7's boundary, and no way to tell fog (in scope, unsharp) from out-of-scope (past the end).

- **XPL-3 (The decision unit: completion is a recorded answer, sized to one session):** the unit of progress declares a **question**, and completes when that question has an **answer recorded on the unit**. It is sized to one working session, and typed by **how it resolves** — investigate (find a fact a decision waits on), make-something-to-react-to (raise the fidelity of the discussion with a cheap concrete artifact), converse (the default: a decision reached with a person), or unblock-by-doing. The last is the one type that *acts* rather than decides, and it earns its place **only by unblocking a decision** — never by delivering any part of the destination.

- **XPL-4 (A unit needing a human is never completed by the actor supplying the human's side):** a unit MAY declare that it resolves **only through live exchange with a person**. Such a unit is **not resolvable by the actor producing both sides of that exchange** — an actor that asks its own questions and answers them has not completed the unit, it has fabricated the counterparty. The check is mechanical and cheap: a unit declared to need a human whose record contains only the actor's own contributions is **not resolved**, whatever it claims. Two consequences bind. Where the person is unavailable, the honest outcome is an **unresolved unit**, never a resolved one with an inferred answer. And a resolution recorded against a person **names them**, so the claim is attributable and falsifiable rather than ambient. (The unit-grain member of the completion-verification family, and the one class CMP-1…CMP-9 cannot see, because the actor's work was real and its report was honest.)

- **XPL-5 (Deliberate incompleteness: fog is declared, and the admission test is sharpness, not answerability):** the plan is **knowingly partial**. Beyond the units it can state lies a declared region of questions the effort can tell are coming and cannot yet phrase, recorded as a coarse description rather than as units. The test between the two is exactly this and nothing else: **can the question be stated precisely now?** — *not* can it be answered now. A question that is sharp but unanswerable is a **blocked unit**; a question that cannot yet be phrased sharply is **fog**. **Pre-slicing fog into units is forbidden**: it manufactures a precision that does not exist, and the resulting units are worked, blocked, re-planned, and discarded at full cost.

- **XPL-6 (Fog graduates one patch at a time, and leaves the fog when it does):** resolving a unit clears the fog ahead of it. Whatever has become statable **graduates into units** — one patch at a time, since a patch may yield several units or none — and the graduated patch is **removed from the fog region** at the same moment, so it lives in exactly one place. A patch that survives in the fog after graduating is a second copy of live work, and it drifts.

- **XPL-7 (Out of scope is terminal, is not fog, and is not part of the route):** work found to sit **past the destination** is neither fog nor a unit: it is **out of scope**, and it **never graduates** — the frontier stops at the destination, so it returns only if the destination is redrawn, and then as a fresh effort rather than a resumption. **Scope, not sharpness, lands it there.** When an existing unit turns out to be mis-scoped it is **closed** — a closed unit is unambiguously off the frontier — and leaves **one line with its reason** in the out-of-scope record. It stays **out of the record of decisions made**, which records the route actually walked: a scope boundary is not a step on it.

- **XPL-8 (The map is an index; each decision lives in exactly one place):** the effort's map carries the destination, the standing notes, a **one-line gist plus link per resolved decision**, the fog region, and the out-of-scope record. It does **not** restate the decisions themselves — each lives on its own unit, and the map points at it. Two properties follow: the map loads at low resolution once per session, with any unit's full body fetched on demand; and a decision restated on the map is a second copy that drifts from the one that is authoritative.

- **XPL-9 (Claim before work; the frontier is open, unblocked, and unclaimed):** a session **claims** a unit before doing anything on it (composing WL-1's atomic claim), so concurrent sessions skip it. The **frontier** — what is takeable now — is therefore the units that are open, have every blocker resolved, **and** are unclaimed. Unclaimed-ness is part of the definition, not an afterthought: without it two sessions compute the same takeable set and both act on it.

- **XPL-10 (Structure is expressed in the host's native relations wherever it has them):** blocking, parent/child, and ownership are expressed through the **surrounding system's own relationship mechanisms** wherever those exist, rather than through a convention inside a unit's body. The reason is not tidiness: native relations render the frontier **in the surfaces people already look at**, so a person sees what is takeable without opening the map. A body convention is the declared fallback for a host that lacks the relation, never the default.

- **XPL-11 (What a human reads names things; a bare identifier never stands in for a name):** in every human-facing rendering — narration, the map's decision record, a session's report — units are referred to by **name**. A list of bare identifiers is unreadable, and reading it requires opening each one to learn what it was. The identifier and its address are not discarded: they **ride inside the name** as its link, and never replace it.

- **XPL-12 (Completion hands off; it does not roll into execution):** when the frontier empties at the destination, the phase's output is **collapsed into the artifact the execution phase's precondition names** (TG-1), and execution begins from that artifact. Feeding the map's units directly to an executor **skips the collapse** and discards the linked reasoning that made each decision legible, leaving the executing phase working from titles. The one exception is the effort that turned out small enough that no map was needed, which is a finding about the effort rather than a shortcut through the handoff.

## 4. Detailed Design

### 4.1 The three regions

The single most common error is treating these as one thing with degrees. They are three, with different membership tests and different exits:

| Region | Membership test | Exits by | Ever becomes a unit? |
| --- | --- | --- | --- |
| **Unit** | The question is statable sharply now | Being answered and recorded | It is one |
| **Fog** | In scope, but not yet statable sharply | Graduating (XPL-6) | Yes, one patch at a time |
| **Out of scope** | Past the destination | Nothing — terminal | **Never** (XPL-7) |

A blocked unit is a **unit**, not fog: sharpness, not actionability, is what admits it. Conversely a patch of fog is not a small unit — it is coarser than a unit, and may yield several or none.

### 4.2 The unit

```text
unit:
    question     := <the decision or investigation this resolves>     # XPL-3
    resolves_by  := investigate | make-to-react-to | converse | unblock-by-doing
    needs_human  := true | false                                       # XPL-4
    blocked_by   := <native relations to other units>                  # XPL-10
    claimed_by   := <session or actor, set before any work>            # XPL-9
    answer       := <recorded on resolution; absent while open>
```

`needs_human` is the field XPL-4 acts on, and it is worth being blunt about why it is a declared field rather than an inference: an actor deciding for itself whether a human is needed will decide correctly right up to the point where deciding otherwise is convenient.

### 4.3 The counterparty check

XPL-4's enforcement is deliberately shallow, because a deep one is not available and a shallow one is enough:

```text
resolvable(unit):
    if not unit.needs_human:            return true
    if unit.record contains only the actor's own contributions:
                                        return false   # fabricated counterparty
    if unit.answer has no named person attached:
                                        return false   # unattributable claim
    return true
```

It does not attempt to detect a *convincing* fabrication. It detects the shape that actually occurs: a transcript with one participant, and an answer attributed to nobody.

### 4.4 A session

```text
1. Load the map at low resolution (XPL-8); orient to the destination (XPL-2).
2. Take the first frontier unit — open, unblocked, unclaimed — and claim it (XPL-9).
3. Resolve it, zooming into related or closed units on demand.
4. Record the answer on the unit; close it; append a one-line gist + link to the map.
5. Graduate whatever fog the answer made statable, clearing those patches (XPL-6).
   If the answer reveals a unit sits past the destination, rule it out of scope (XPL-7).
6. Frontier empty and destination reached -> hand off (XPL-12).
```

Step 2's *claim first* and step 5's *clear as you graduate* are the two steps most often skipped, and each has a specific consequence: duplicated work, and a fog region that slowly becomes a second copy of the plan.

### 4.5 Failure modes named

| Failure | What it looks like | Which invariant closes it |
| --- | --- | --- |
| Guessed requirements | A plan whose scope nobody decided | XPL-1, TG-15 |
| Decision resolved by building | The option that got built is the decision | XPL-1, XPL-3 |
| Fabricated counterparty | A one-participant transcript, resolved | **XPL-4** |
| Pre-sliced fog | Precise units for undecided questions | XPL-5 |
| Fog shadow | A graduated patch still described in the fog | XPL-6 |
| Boundless scope | Off-destination work that never closes | XPL-7 |
| Map as store | Decisions restated on the map, drifting | XPL-8 |
| Double-take | Two sessions on one unit | XPL-9 |
| Invisible frontier | Takeability only computable from the map | XPL-10 |
| Identifier wall | A report a human cannot read | XPL-11 |
| Skipped collapse | Execution working from titles | XPL-12 |

## nodus-relevance mapping

- **The counterparty rule already holds at the language grain.** The dialog spec's rejection of satisfying a human-interaction step through a model call is XPL-4 expressed as a language constraint; XPL-4 is the same rule one level up, where the unit rather than the step is what claims to be complete.
- **Fog has no construct, and should not get one.** A workflow definition is an execution artifact; a knowingly-partial plan is not something the runtime should be able to represent, or partial plans become runnable. The handoff (XPL-12) is precisely the boundary at which a decision map becomes something a workflow may reference.
- **Native relations over body conventions.** XPL-10 mirrors the language's own preference for declared structure over conventions parsed out of free text: a relation the host understands is inspectable by every surface, and one encoded in prose is inspectable by whoever wrote the parser.

## 5. Implementation Notes

1. **Make `needs_human` a declared field, checked at resolution** (XPL-4). Inferring it, or checking it only at review, both fail in the same direction and at the same moment.
2. **Do not model fog as units with a flag.** A `status: fog` unit is a unit, and it will be worked, ranked, and counted. Fog is a region of the map, deliberately coarser than the unit grain (XPL-5).
3. **Close mis-scoped units rather than parking them** (XPL-7). Open-but-not-really-open is exactly the state whose ambiguity the out-of-scope record exists to remove.
4. **Wire relations in a second pass.** Units generally need identities before they can reference each other, so create-then-wire is the ordinary shape, not a workaround.
5. **Keep the effort's standing notes on the map** (XPL-8) — the domain, what every session should consult, standing preferences. It is the one thing every session reads and the one place it is cheap to keep current.
6. **Resolve one unit per session by default**, with investigate-shaped units the exception (they parallelize and return facts rather than decisions). The bound is about the quality of a decision, not about throughput.

## 6. Drawbacks & Alternatives

- **This phase is slow and dense.** Honestly so, and XPL-2 is the bound: an effort whose destination is already clear should not be here. The cost is justified only where the route genuinely is not visible.
- **The fog/unit test is a judgement.** Accepted (XPL-5 states the test, not a procedure). It is a *sharper* judgement than the alternative, which is deciding whether something is "ready to plan" with no test at all.
- **XPL-4 can block an effort when no human is available.** By design. The alternative is a resolved unit whose answer nobody gave, which is worse in the exact way that is hardest to detect later.
- **A map is a second artifact to maintain.** Bounded by XPL-8: the map is an index, so maintaining it is appending one line per resolved unit. A map that has become expensive to maintain has become a store, which is the violation.
- **Alternative — plan it as tasks and let the decisions emerge.** Rejected by XPL-1 and XPL-3: decisions filed as tasks are made by whoever picks the task up, are not recorded as decisions, and are discovered later as unexplained constraints in the code.
- **Alternative — chart the whole map up front.** Rejected by XPL-5: the parts that cannot yet be stated sharply would be stated anyway, and the fabricated precision is indistinguishable from the real kind once it is written down.
- **Alternative — one region for "not now", covering both fog and out-of-scope.** Rejected by XPL-7: they differ in *membership test* and in *exit*, and merging them means either off-destination work keeps resurfacing or genuinely in-scope questions are silently dropped.
- **Alternative — fold into `l1-task-graph-model`.** Rejected: TG's subject is a graph generated from a requirements artifact, and its unit is a task. This phase exists because that artifact does not exist yet, and its unit is a decision whose product is explicitly not a deliverable. Folding them would put TG-1's precondition inside the phase whose job is to satisfy it.
- **Alternative — fold into `l1-facilitation`.** Rejected: facilitation is the *technique layer* for running an interview or an ideation session, advisory by contract (FC-12). This is a planning phase with a destination, a frontier, claims, scope boundaries and a handoff obligation. Facilitation is how a converse-shaped unit is worked; it is not the phase.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[TASKGRAPH]` | `.design/main/specifications/l1-task-graph-model.md` | The downstream phase and TG-1's precondition this one satisfies |
| `[CONVERGE]` | `.design/main/specifications/l1-work-convergence.md` | The single activity surface a decision unit converges on (CONV-9) |
| `[LIVENESS]` | `.design/main/specifications/l1-work-liveness.md` | WL-1's atomic claim, composed by XPL-9 |
| `[COMPLETION]` | `.design/main/specifications/l1-completion-verification.md` | CMP-10, the general form of XPL-4 |
| `[FACILITATION]` | `.design/main/specifications/l1-facilitation.md` | How a converse-shaped unit is actually worked (FC-13) |
| `[REJECTION]` | `.design/main/specifications/l1-rejection-memory.md` | The project-scoped record XPL-7's effort-scoped boundary is distinct from |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-26 | Core Team | Initial concept — the phase **before** TG-1's precondition holds, where no requirements artifact exists and none can yet be written. Output is **decisions, not deliverables**, and the phase's completion tell is the **urge to build** rather than a temptation to resist (XPL-1); the **destination is named first and fixes the scope**, giving the effort its termination condition and the basis for every boundary (XPL-2); the decision unit completes on a **recorded answer**, sized to one session, typed by how it resolves, with the acting type earning its place only by unblocking a decision (XPL-3); **a unit needing a human is never completed by the actor supplying the human's side** — an actor that asks its own questions and answers them has fabricated the counterparty, the check being a record containing only the actor's contributions or an answer attributed to nobody, and the honest outcome where no person is available is an unresolved unit (XPL-4); deliberate incompleteness, with the admission test being **whether the question can be stated sharply now, not whether it can be answered** — sharp-but-blocked is a unit, not-yet-sharp is fog, and pre-slicing fog manufactures precision that does not exist (XPL-5); fog graduates one patch at a time and leaves the fog region as it does (XPL-6); **out of scope is terminal, is not fog, and is not part of the route** — scope not sharpness lands it there, a mis-scoped unit is closed to leave the frontier unambiguously, and it stays out of the decisions record because a scope boundary is not a step on the walked route (XPL-7); the map is an **index, not a store** — one gist plus link per decision, each decision authoritative in exactly one place (XPL-8); claim before work, with the frontier defined as open, unblocked **and unclaimed** (XPL-9); structure expressed in the host's **native relations**, so the frontier renders in surfaces people already look at (XPL-10); human-facing renderings **name** units, with identifiers riding inside names rather than replacing them (XPL-11); completion **hands off** by collapsing into the artifact TG-1 names, since feeding units straight to an executor discards the linked reasoning and leaves execution working from titles (XPL-12). Concept-only. |

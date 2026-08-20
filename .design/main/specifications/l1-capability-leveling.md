# Capability Leveling

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Capability leveling is the **closed loop that narrows the spread between models on large-scale construction work** by replaying reconstructed episodes of exemplary work and converting each divergence into a durable, transferable correction.

The loop takes an episode whose production chain has been reconstructed backwards ([l1-inverse-derivation.md](l1-inverse-derivation.md)), replays it forward with several models, compares each replay against the episode's recorded outcome **and against its recorded trajectory**, classifies every divergence by its cause, and routes the classified divergence to the carrier appropriate to that cause. Then it runs again. It terminates not when a replay reproduces the original — that never happens and is not wanted — but when further iteration stops raising the weakest performer and stops narrowing the spread.

Two commitments make this something other than an expensive way to generate advice. **Cause determines carrier**: a divergence caused by not knowing becomes recorded knowledge, a divergence caused by not being able becomes environmental scaffolding that removes the need for the ability, and a divergence caused by nothing at all — the exemplar simply chose one of several equally valid options — is discarded rather than learned. And **leveling is achieved mainly by what the environment executes, not by what the model is told**: instruction is read by every model and executed differently by each, which is the very spread the loop exists to remove.

The loop produces no new learning machinery. The office already distils experience into memory, skills, and reusable playbooks; this is a **new source of experience** for that machinery — exemplary work by others, rather than the office's own sessions.

## Related Specifications

- [l1-inverse-derivation.md](l1-inverse-derivation.md) — The supplier. Provides the graded chain, the episode's before/after states, its executable checks, and its trajectory (IVD-1/IVD-2/IVD-3). The seam is one-directional: this spec never reaches back into reconstruction.
- [l2-learning-loop.md](l2-learning-loop.md) — The existing distillation machinery this loop feeds rather than duplicates: session review writing to memory and skills. Here the reviewed experience is a replayed exemplary episode instead of an own session (LVL-13).
- [l2-self-improvement.md](l2-self-improvement.md) — The consuming surface: mistake logs, calibration, and accumulated reasoning templates joined into a start-of-task brief. Lessons land here, not in a parallel store.
- [l1-pattern-codification.md](l1-pattern-codification.md) — The promotion pathway: PC-1 repetition yields a candidate, PC-2 forbids agent self-promotion into binding norms, PC-5 retires what stops holding. LVL-10 places the human ratification at the laboratory boundary rather than on each candidate.
- [l1-scoped-generalization.md](l1-scoped-generalization.md) — SG-3/SG-11: recurrence across **independent contexts** decides a lesson's scope, which is the gate that separates "this is how construction works" from "this is how those authors worked" (LVL-11).
- [l1-convergence-gate.md](l1-convergence-gate.md) — CG-3 judges the delta a change introduces, CG-5 deferred obligation, CG-9 authority-protected gate definition; the laboratory boundary is such a gate placed where every lesson must cross to become real (LVL-10).
- [l1-dynamic-harness.md](l1-dynamic-harness.md) — The scaffolding carrier for capability gaps: what the environment enforces rather than what the model is asked to remember (LVL-6).
- [l1-evaluation-suites.md](l1-evaluation-suites.md) — ES-3 grader machinery reused rather than redefined; ES-11 sandboxed run. Demarcation: suites grade a customization's marginal effect, this loop closes a correction cycle (§5.7).
- [l1-simulation.md](l1-simulation.md) — SIM-2 suppressed real effects, the mode every replay runs in; SIM-7 demarcation — simulation reveals behaviour, this loop grades and corrects it.
- [l1-specialty-exemplars.md](l1-specialty-exemplars.md) — The measurement sibling at micro grain (SE-2/SE-5): competency of a model in a specialty against an authored reference. This loop measures spread across models on a recovered arc of construction work.
- [l1-model-benchmarking.md](l1-model-benchmarking.md) — MB-1 fixed probes, MB-4 deterministic-first grading, MB-8 bounded isolated runs, MB-9 honest failure; the comparison disciplines this loop inherits.
- [l1-model-adaptation.md](l1-model-adaptation.md) — The model-plane sibling explicitly **not** used here: MA-5 governs reversible weight adaptation; this loop is confined to the context plane (LVL-14).
- [l1-corpus-originality.md](l1-corpus-originality.md) — ORI-2/ORI-10 applied to the episode corpus: near-identical episodes must not dominate the measurement (LVL-16).
- [l1-host-native-rendering.md](l1-host-native-rendering.md) — HNR-1/HNR-12 canonical-source-with-derived-renditions and honest degradation; how one lesson reaches structurally different hosts without a second authored copy (LVL-14).
- [l1-solution-frugality.md](l1-solution-frugality.md) — The counterweight to scaffolding accumulation: every hardening is a cost, and LVL-8 refuses hardening that degrades strong performers.
- [l1-outcome-attributed-cost.md](l1-outcome-attributed-cost.md) — Run cost attributed to what the loop actually produced; a lesson's price is knowable (LVL-15).
- [l1-security.md](l1-security.md) — SEC-9/SEC-10 human-rooted authority: the agent proposes lessons and never grants itself binding ones (LVL-10).

## 1. Motivation

Models differ enormously on long-horizon construction work, and the difference is not well described as "one is better". One holds an architecture across many steps and stays inside it; another loses the frame after a few steps and starts improvising locally-plausible work that does not compose. The gap is in **process**, not in any single output, and it is invisible to instruments that grade one response at a time.

Instruction alone does not close it. Written guidance is read by every model and executed differently by each — thoroughly, partially, or in appearance only — and that execution variance **is** the spread. Adding more guidance often widens it: longer instructions are followed even less uniformly, and the strongest performer, who needed none of it, now spends its attention on compliance.

Meanwhile the knowledge that would close the gap exists, unwritten, in work already done well by others. Not in its finished artifacts — those show conclusions — but in how the work moved: what was settled first, where it doubled back, what framing made a decision tractable. Recovering that is inverse derivation's job. Turning it into something that measurably raises the weakest performer is this loop's job, and it needs three things the recovered chain does not supply: a way to find out where models actually fail, a principled choice of what to do about each failure, and a stopping rule that is not wishful.

The stopping rule is where the naive version of this idea collapses. "Iterate until the reproduction matches the original" never terminates: two competent engineers solving one problem do not produce identical work, and neither will two runs. Reframing the goal as **spread collapse** replaces an unreachable target with a measurable one, and — usefully — makes the exemplar a **bar** rather than a thing to be copied.

## 2. Constraints & Assumptions

- **The corpus is a measuring instrument.** Comparisons hold only within a corpus version (IVD-5); a corpus that changes between runs makes every trend uninterpretable.
- **Replays run with real effects suppressed.** Every replay is a simulation run (SIM-2); nothing a replay does reaches production state.
- **The loop is internal.** It is preparation of how the office works, not a capability surfaced to a principal. Its outputs reach working practice only through ratification (LVL-10).
- **Lessons must be transferable by copying.** Anything that cannot be carried into a different model or a different host as text or as environment cannot level anything.
- **Budgets are finite and the work is expensive.** Multi-model replay of many episodes over many iterations is the dominant cost of the whole mechanism.
- **A reconstructed episode's framing knows the answer.** It was recovered from completed work. This contaminates the replay by construction, and the design accounts for it rather than pretending it away (LVL-3).

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **LVL-1 (The objective is spread collapse at a bar, not reproduction):** the loop optimizes for **the weakest participating model reaching the episode's bar** and for **the spread across participating models narrowing**. An episode's **bar** is defined by the episode itself, not by resemblance to it: the episode's executable checks pass, and the replay's outcome satisfies the same obligations the after-state satisfied. Reproducing the exemplar's artifact is never an objective and never a success criterion. An implementation that optimizes similarity-to-exemplar will select for imitation of one body of work, which is the failure this mechanism most needs to avoid.

- **LVL-2 (Termination is measured, two-part, marginal — and states which of two endings it reached):** iteration stops when the last round **neither raised the weakest performer nor narrowed the spread**, against a declared and published marginality threshold. Both terms are required: raising everyone uniformly leaves the spread intact, and narrowing the spread by degrading strong performers is not leveling (LVL-8). Termination is reported as one of two distinct endings — **converged at the bar** (the floor reached it) or **plateaued below the bar** (iteration stopped helping while the floor is still short) — and the second is never reported as success. A collapsed spread among models that all fail the episode is the loop's most misleading state: its headline metric is at its best precisely where nothing was achieved. A loop whose stopping rule is "until it matches" does not terminate at all; a loop whose threshold is unpublished can terminate on its first round and call it convergence.

- **LVL-3 (Comparison covers outcome and trajectory, and a suspiciously smooth path is a finding):** every replay is compared against the episode's recorded outcome **and** against its recorded trajectory (IVD-3). A replay that reaches a comparable outcome by a path markedly straighter than the original — no reversals where the original reversed, no exploration where the original explored — is recorded as a **contamination signal**, not as an outstanding result. The reconstructed framing carries knowledge of the answer; the trajectory comparison is the mechanism that detects when a replay is executing that answer rather than finding it.

- **LVL-4 (Every divergence is classified before it is acted on):** a divergence is assigned exactly one cause — **knowledge** (the model did not know the practice), **capability** (it knew and could not execute reliably), **framing** (the task as posed admitted more than one reading), or **arbitrary** (the exemplar made one of several equally valid choices) — and the cause determines the carrier (LVL-6). Acting on unclassified divergence produces guidance that mixes real practice with one team's habits, and the mixture cannot be separated afterwards.

- **LVL-5 (Arbitrary divergences are discarded, and the bias is toward retention):** a divergence classified **arbitrary** yields no lesson. Where classification is genuinely uncertain between arbitrary and substantive, the divergence is **retained as a low-scope candidate** rather than discarded. The two errors are asymmetric: a wrongly-kept lesson is visible, contestable, and eventually falsified by its own scope gate (LVL-11); a wrongly-discarded lesson is silently gone, and nothing downstream can recover what was never recorded.

- **LVL-6 (Carrier is chosen by who executes the lesson):** the carrier is selected by which party must act for the lesson to hold — the environment, the orchestrator, or the model — never by storage convenience. The ordering, strongest first: **enforced check** (environment executes; no model variance, no context cost), **enforced procedure** (orchestrator executes an ordering), **triggered instruction** (loaded on a condition), **standing rule** (resident instruction), **precedent** (retrieved by relevance). An implementation that files every lesson as text has chosen the layer with the highest execution variance for the problem defined by execution variance.

- **LVL-7 (Executable carriers require an objective oracle):** a lesson may occupy an enforced carrier only if satisfaction is decidable **without judgment**. Lessons that require judgment take an instructional carrier and are recorded as **weaker on weak models** rather than being converted into a mechanical proxy for a judgment the proxy cannot make. A gate that approximates a judgment enforces the approximation, and the approximation is what gets optimized against.

- **LVL-8 (Hardening must not degrade the strong):** a candidate correction that raises the weakest performer while measurably degrading a strong one is **rejected in that form**, and either narrowed in scope, made conditional, or discarded. Leveling by lowering the ceiling produces a uniform pipeline that no participant exceeds, which trades the mechanism's purpose for its metric.

- **LVL-9 (The laboratory is isolated from working practice):** in-loop corrections are applied **only inside the loop's own execution context** and are invisible to offices doing real work. An unratified correction that reaches working practice is indistinguishable, at the point of use, from a ratified one — so the failure is silent, and the behaviour it causes is attributed to the model rather than to the experiment.

- **LVL-10 (One human gate, placed at the boundary, on the body of lessons):** corrections apply automatically **within** the loop — the loop cannot iterate otherwise — and cross into working practice only through a single explicit human ratification of the **assembled body of lessons**, carrying each lesson's evidence, scope, and proposed carrier. Per-lesson ratification does not scale to the volume this loop produces and degrades into rubber-stamping, which is worse than a single considered review. The gate is placed where all lessons converge (CG-3), and the agent never self-promotes across it (PC-2, SEC-10).

- **LVL-11 (Scope is earned across independent contexts):** a lesson's declared scope is bounded by the number of **independent contexts** in which it was corroborated — different technology, different domain, different exemplar source — not by how often it recurred within one. Recurrence inside a single source measures that source's consistency (SG-3/SG-11). Without this gate, one exemplar's house style is promoted to a universal construction practice with a confident evidence count behind it.

- **LVL-12 (Lessons inherit the evidentiary grade of their source):** a lesson derived from an **assumed** link in a reconstruction (IVD-2) carries that grade and cannot outrank one derived from an attested link. Grades survive aggregation: a lesson corroborated only by assumed links remains assumption-grade however many times it recurs. Otherwise the loop's most confident conclusions accumulate on its weakest evidence, since unrecorded framing is exactly what gets assumed most often.

- **LVL-13 (One learning machinery; this is a source, not a second loop):** lessons are written into the office's **existing** memory, skill, rule, and scaffolding surfaces. The loop introduces no parallel store of behavioural knowledge. A second store diverges from the first, and the copy that actually reaches a working context is decided by retrieval order rather than by anyone's intent.

- **LVL-14 (Context plane only; carriers are canonical and rendered):** the loop changes what a model is given and what its environment enforces. It **never** alters model weights (MA-5 governs that plane and is out of scope). Each lesson exists once canonically and is materialized per host by rendition (HNR-1/HNR-12), with honest degradation reported where a host cannot express it — a lesson that survives only in one host's format levels one host.

- **LVL-15 (Runs are budgeted and report incompleteness):** every iteration declares a budget; a run that exhausts it reports **which episodes and which models it did not reach**, and results are never aggregated across a partially-executed round as though complete. Partial coverage silently averaged with full coverage produces a spread number that no one can interpret and that moves for reasons unrelated to learning.

- **LVL-16 (The episode set is de-duplicated before it is measured):** near-identical episodes are detected and down-weighted or excluded (ORI-2/ORI-10). A source contributes many structurally identical arcs; unchecked, the corpus measures one situation repeatedly and reports the result as breadth.

- **LVL-17 (Measurements are comparable only within a corpus version):** every recorded measurement names the corpus version it was taken against (IVD-5), and comparisons across versions are refused rather than approximated. Without the version, an accumulated history of spread numbers is a series of measurements of different things presented as a trend.

## 4. Detailed Design

### 4.1 The loop

```
reconstructed episode (graded chain, before-state, checks, trajectory)
   ──▶ de-duplicate against the episode set                      [LVL-16]
   ──▶ replay forward, N models, effects suppressed              [SIM-2]
   ──▶ compare: outcome vs after-state, path vs trajectory       [LVL-3]
   ──▶ classify each divergence by cause                         [LVL-4]
   ──▶ discard arbitrary; route the rest to carriers             [LVL-5, LVL-6]
   ──▶ apply inside the laboratory only                          [LVL-9]
   ──▶ re-measure: weakest performer, spread                     [LVL-2]
        │
        ├── still improving ──▶ iterate
        └── marginal ──▶ assemble body of lessons ──▶ human gate ──▶ practice
                                                      [LVL-10]
```

### 4.2 Divergence taxonomy and routing

| Cause | Diagnostic question | Carrier | Effect on spread |
| --- | --- | --- | --- |
| knowledge | Would the model have acted correctly had it known this practice? | precedent, then standing rule if corroborated | Moderate — depends on retrieval and on compliance |
| capability | Did it know, and fail to execute reliably? | enforced check or enforced procedure | **Largest** — removes the need for the ability |
| framing | Did the posed task admit a second reasonable reading? | framing template for the task class | Large — eliminates a whole divergence class at the source |
| arbitrary | Was the exemplar's choice one of several equally valid? | none — discarded (LVL-5) | None; keeping it *widens* effective spread by adding noise |

Classification is evidence-driven: a divergence is **capability** rather than **knowledge** when the model demonstrably held the practice — stated it, applied it elsewhere in the same replay — and still failed to sustain it. Where evidence does not separate the two, the divergence takes the knowledge carrier first, because the instructional carrier is cheap and reversible while scaffolding is neither.

### 4.3 The carrier ladder

| Carrier | Executed by | Spread contribution | Context cost | Requires an oracle |
| --- | --- | --- | --- | --- |
| Enforced check | Environment | none | none | yes (LVL-7) |
| Enforced procedure | Orchestrator | low | low | partial |
| Triggered instruction | Model, on a guaranteed load | moderate | on demand | no |
| Standing rule | Model, always resident | high | continuous | no |
| Precedent | Model, on relevance retrieval | high | on retrieval | no |

The ladder is a preference order, not a hierarchy of importance. Most lessons cannot descend it — judgment does not become decidable because it would be convenient (LVL-7) — and the loop is expected to produce mostly instructional carriers, with the smaller executable set doing a disproportionate share of the leveling. The counterweight is LVL-8: each descent adds enforcement that every participant pays, including the one that never needed it.

### 4.4 The stop criterion

Two quantities per round, both against a fixed corpus version (LVL-17):

| Quantity | What it answers |
| --- | --- |
| Weakest performer's distance to the episode bar | Did the floor rise? |
| Spread across participating models | Did the models converge? |

Stop when the round's improvement in both is marginal, against a threshold declared before the campaign and published with its results. Three degenerate outcomes are named explicitly so they are not misread as success:

| Degenerate ending | What it looks like | Why it is not success |
| --- | --- | --- |
| Uniform lift without convergence | Everyone improved, the gap is unchanged | The lessons are being executed as unevenly as before — the spread is the target, and it did not move |
| Convergence by ceiling loss | The gap closed because a strong performer got worse | Rejected under LVL-8; the metric improved by damaging what it was meant to preserve |
| **Plateau below the bar** | Spread is small, the floor still fails the episode | The headline metric is at its best precisely where nothing was achieved — models agreeing on failure is agreement, not capability |

The third is the one an implementation is most likely to report as a win, because a small spread reads as convergence in every summary that does not also carry the floor's distance to the bar. Both quantities are reported together, always.

Spread is undefined below two participants and unstable at two; the participating set is declared with the campaign, and a campaign whose set shrinks mid-run reports the change rather than continuing the series (LVL-17's reasoning applied to the participant axis).

### 4.5 The laboratory boundary

```
┌─ laboratory ───────────────────────────────────────────┐
│ corrections applied automatically; replays sandboxed;   │
│ carriers materialized into the loop's own context only  │
└──────────────────────┬─────────────────────────────────┘
                       │  assembled body of lessons
                       │  (evidence, grade, scope, carrier)
                       ▼
              ── human ratification ──          [LVL-10, CG-3]
                       │
                       ▼
        existing memory / skill / rule / scaffolding surfaces
                                                  [LVL-13]
```

Isolation is a property of where carriers are materialized, not a label on them. A correction written into a shared surface with a "draft" marker is already in working practice, because retrieval does not read markers.

### 4.6 The generalization gate

A lesson's scope is bounded by corroboration across **independent** contexts:

| Corroborated in | Admissible scope |
| --- | --- |
| One episode | Candidate; no scope claim |
| Several episodes, one source | Local to that source's domain and conventions |
| Independent sources, one technology | The technology and its ecosystem |
| Independent sources, independent technologies | General construction practice |

This is what gives varied technologies and domains a job in the corpus: they are the instrument that distinguishes a practice from a habit. It is also why a corpus concentrated in one ecosystem cannot produce general lessons, however large it grows.

### 4.7 Demarcation — five neighbours that are not this

- **Simulation** plays a mechanism out to reveal its behaviour and returns no verdict (SIM-7). This loop grades a replay against a recovered reference and acts on the difference. A replay is a simulation run; the loop is what surrounds it.
- **Evaluation suites** measure a customization's marginal effect with reusable grader machinery (ES-3). This loop borrows that machinery and closes a correction cycle around it — measurement is a step here, not the product.
- **Model benchmarking** ranks base-model fitness per task class to inform routing. This loop does not rank models; it uses the differences between them as the signal it is trying to eliminate.
- **Specialty exemplars** measure one model's competency in one specialty against an authored gold standard (SE-2/SE-5), at micro grain. This measures spread across models on a recovered arc of construction work, and its output is corrections rather than a competency profile.
- **Competitive execution** runs candidates in parallel and keeps the winner, discarding the rest. Here nothing is discarded on the basis of winning: divergences from **every** participant are evidence, and the weakest performer's failures are the most informative signal the round produces.

## 5. Implementation Notes

- Fix the corpus version before a campaign and leave it fixed until the campaign ends. Adding episodes mid-campaign is the most common way to make a spread trend meaningless.
- Record the divergence taxonomy counts per round. A round dominated by **arbitrary** classifications means the exemplar is contributing style rather than method — a corpus-composition signal (IVD-7), not a loop failure.
- Keep replay budgets per-episode rather than per-round, so an expensive episode cannot silently consume the round and trigger LVL-15 incompleteness for everything after it.
- Re-run retired lessons occasionally against later rounds. A lesson that no longer changes any outcome is a candidate for retirement (PC-5); accumulated inert scaffolding is pure cost under LVL-8's ledger.
- Prefer few, well-chosen participating models over many. Spread is measured across the set, and an unrepresentative set produces a number that improves without meaning anything.
- Report the grade mix behind each promoted lesson (LVL-12). A body of lessons resting largely on assumed links is a signal to improve corpus record quality, not to tighten thresholds.

## 6. Drawbacks & Alternatives

- **Contamination is bounded, not eliminated.** Trajectory comparison detects replays that are implausibly smooth; it cannot detect a replay that is contaminated *and* imperfect, which is the common case. The residual is a systematic optimism in the loop's own results, and it should be stated wherever those results are reported.
- **Trajectory data is the least reliable input.** Abandoned work is under-recorded at the source (IVD-3), so the reference path is systematically straighter than the real one — which biases the contamination test toward silence. <!-- TBD: whether a per-source trajectory-fidelity factor is estimable, or whether the bias is reported as a known floor -->
- **Classification is a judgment made by the system about itself.** Distinguishing "did not know" from "could not sustain" is not always decidable from a replay. The retention bias (LVL-5) and the cheap-carrier-first rule (§4.2) bound the damage in the direction that is recoverable.
- **Cost is dominated by multi-model replay** and grows with episodes × models × rounds. LVL-15 makes the cost visible rather than smaller; corpus curation (IVD-7) is the real lever.
- **Alternative — iterate until the artifact matches:** rejected (LVL-1, LVL-2). Non-terminating, and it optimizes for imitating one body of work.
- **Alternative — file every lesson as instruction:** rejected (LVL-6). It puts every lesson in the highest-variance carrier, so the spread the loop exists to remove is precisely what determines whether its output takes effect.
- **Alternative — convert every lesson into enforcement:** rejected (LVL-7, LVL-8). Judgment lessons become mechanical proxies that get optimized against, and accumulated enforcement degrades exactly the participants who did not need it.
- **Alternative — ratify each lesson individually:** rejected (LVL-10). It does not scale to this volume and degrades into rubber-stamping, which reads as human authority while providing none.
- **Alternative — a dedicated store for loop-derived behaviour:** rejected (LVL-13). A second store diverges from the first, and which copy reaches a working context becomes a retrieval accident.
- **Alternative — adapt weights instead:** out of scope by construction (LVL-14). It is a different plane with a different governance regime (MA-5), and its output cannot be carried into another model — which is the requirement that defines this loop.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[SUPPLIER]` | `.design/main/specifications/l1-inverse-derivation.md` | The graded chain, before-state, checks, and trajectory this loop consumes |
| `[DISTILLATION]` | `.design/main/specifications/l2-learning-loop.md` | The existing machinery lessons are written into (LVL-13) |
| `[BRIEF]` | `.design/main/specifications/l2-self-improvement.md` | The consuming surface: mistake log, calibration, reasoning templates |
| `[PROMOTION]` | `.design/main/specifications/l1-pattern-codification.md` | PC-2 no self-promotion, PC-5 retirement; the ratification pathway |
| `[SCOPE]` | `.design/main/specifications/l1-scoped-generalization.md` | SG-3/SG-11 independent-context gate behind LVL-11 |
| `[GATE]` | `.design/main/specifications/l1-convergence-gate.md` | CG-3 delta judgment; the boundary placement behind LVL-10 |
| `[SCAFFOLD]` | `.design/main/specifications/l1-dynamic-harness.md` | The enforced-carrier surface for capability gaps |
| `[GRADERS]` | `.design/main/specifications/l1-evaluation-suites.md` | ES-3 grader machinery reused rather than redefined |
| `[RENDITION]` | `.design/main/specifications/l1-host-native-rendering.md` | HNR-1/HNR-12 canonical lesson, per-host rendition, honest degradation |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-20 | Core Team | Initial concept: the closed replay loop that levels model capability on large-scale construction work using reconstructed episodes of exemplary work. Objective stated as spread collapse at a bar rather than reproduction, with similarity-to-exemplar explicitly rejected as an optimization target (LVL-1); a measured two-part marginal stopping rule against a **published** threshold, replacing the non-terminating "until identical", reporting which of two endings it reached — converged at the bar, or **plateaued below it**, the latter being the loop's most misleading state since a collapsed spread among models that all fail puts its headline metric at its best exactly where nothing was achieved (LVL-2); comparison over outcome **and** recorded trajectory, with an implausibly smooth path recorded as a contamination signal rather than a success — the structural answer to a reconstructed framing that knows the answer (LVL-3); mandatory four-way cause classification of every divergence (LVL-4); arbitrary divergences discarded with an explicit retention bias on uncertain calls, justified by the asymmetry between a visible wrong-keep and a silent wrong-discard (LVL-5); carrier selected by who executes the lesson, with the five-rung ladder from enforced check to precedent and the reason instruction-only filing defeats the loop's purpose (LVL-6); executable carriers gated on an objective oracle, judgment lessons kept instructional rather than proxied (LVL-7); hardening refused where it degrades strong performers, forbidding leveling by ceiling loss (LVL-8); laboratory isolation defined by where carriers are materialized rather than by draft markers (LVL-9); a single human ratification gate on the assembled body of lessons at the boundary, with per-lesson ratification rejected as rubber-stamping (LVL-10); scope earned across independent contexts, giving varied technologies their instrumental role (LVL-11); evidentiary grades inherited from the reconstruction and surviving aggregation (LVL-12); one learning machinery, this being a new source of experience rather than a second loop (LVL-13); context plane only, canonical lessons rendered per host with honest degradation (LVL-14); budgeted runs reporting unreached episodes and models, never aggregating partial rounds as complete (LVL-15); episode de-duplication before measurement (LVL-16); measurements comparable only within a named corpus version (LVL-17). Demarcated in §4.7 from simulation, evaluation suites, model benchmarking, specialty exemplars, and competitive execution. |

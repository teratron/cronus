# Evaluation Suites

**Version:** 1.2.0
**Status:** Stable
**Layer:** concept

## Overview

An evaluation suite is a declarative, version-controlled test suite for an agent **customization** — a skill, role, workflow, or harness. It pairs a set of **golden tasks** (representative inputs with declared expectations) with **graders** (typed validators that judge a run), and produces an immutable, diffable results artifact. Suites make customization quality measurable: instead of "this skill seems better", a suite answers "this skill version scores 0.84 on task-completion across 40 tasks, up from 0.79, with no regressions".

A suite is the **dynamic** half of customization quality. The **static** half — reading the customization document for contradictions, ambiguity, persona drift, cognitive overload, coverage gaps, and composition conflicts — is the document lint defined in `l2-quality-pipeline.md`. Linting reads the prompt; a suite runs it. Neither substitutes for the other.

This spec defines the suite model and the grader taxonomy. It does not define a runtime; the frozen evaluation pipeline that executes a suite is `l1-harness-engineering.md` (HE-3), and concrete validators are reused from `l1-output-contracts.md` rather than redefined here.

## Related Specifications

- [l1-harness-engineering.md](l1-harness-engineering.md) — the frozen evaluation pipeline (HE-3) that runs a suite; held-out/search-set discipline (HE-6) a suite's task split honors
- [l1-quality-standards.md](l1-quality-standards.md) — a passing suite is a quality gate / definition-of-done for a customization
- [l1-output-contracts.md](l1-output-contracts.md) — schema/callable/criteria validators reused as output-shape graders (ES-3); a suite composes them, never redefines them
- [l1-extensions.md](l1-extensions.md) — skills/customizations are the units under test; a suite lives alongside its skill
- [l1-automation-pipeline.md](l1-automation-pipeline.md) — the dry-run side-effect quarantine (AP-13 §4.9) precedent reused for sandboxed evaluation (ES-11)
- [l1-practice-analytics.md](l1-practice-analytics.md) — normalized session traces a grader reads to judge a run; complementary diagnostic layer
- [l2-quality-pipeline.md](l2-quality-pipeline.md) — static document lint (the complementary half) and the behavior-probe grader precedent
- [l2-self-improvement.md](l2-self-improvement.md) — skill-evolution training runs suites each rollout and gates on their scores; baseline comparison is the best-vs-current check
- [l1-agent-coevaluation.md](l1-agent-coevaluation.md) — the model×harness co-evaluation methodology this suite's grading machinery drives; ES-17 per-slice reporting shares its orthogonal-label discipline (ACE-3/ACE-4)

## 1. Motivation

Customizations (skills, roles, workflows) shape agent behavior, but their quality is usually asserted, not measured. The office already has scattered evaluation: per-story independent tests, behavior probes, training rollouts, output validators. What is missing is a **first-class, reusable evaluation artifact bound to the customization it tests** — authored once, run on demand, compared across versions.

Three gaps follow from its absence:

1. **No regression gate.** Editing a skill to fix one failure can silently break three others. Without a suite run before/after, the regression is invisible until it reaches a user.
2. **No activation testing.** A skill can be perfectly written yet trigger on the wrong prompts (or fail to trigger on the right ones). Output-only checks never catch mis-activation.
3. **No stability signal.** A customization that passes once but fails one run in four is flaky; a single pass/fail hides that. Repeated trials surface variance.

A declarative suite with a typed grader taxonomy closes all three, and gives skill evolution (`l2-self-improvement.md`) and quality gates (`l1-quality-standards.md`) a concrete signal to consume.

## 2. Constraints & Assumptions

- A suite tests a customization, not the base model; it measures the customization's marginal effect on behavior.
- A suite is data (declarative tasks + graders), not code. Graders are typed and configured, not free-form scripts (though a `program` grader may shell out to a validator).
- Evaluation is non-deterministic: the same task may score differently across runs. The suite measures distributions, not single points.
- A suite never runs against production state; it executes in an isolated workspace with declared fixtures.
- Authoring a suite is optional but recommended for any customization whose behavior matters; the office does not block a skill that lacks one, but flags it.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **ES-1 Suite as companion artifact**: an evaluation suite is stored alongside the customization it tests and versioned with it. It is independently runnable on demand and identifies its target by stable identity. The unit under test is the customization; the suite is declarative data.

- **ES-2 Golden tasks with declared expectations**: a task is an addressable (stable id), taggable, individually enable-able unit declaring inputs (a prompt plus optional context and fixture files) and zero or more expectations (should-trigger, output-must-contain / must-not-contain, expected outcomes, behavior limits) and zero or more graders. A task with no grader and no expectation is inert and MUST be reported as non-asserting.

- **ES-3 Typed grader taxonomy, composed not redefined**: a grader is a typed, named, declarative validator producing a pass/fail or scored verdict for a run. The taxonomy spans five families — *activation* (did the customization trigger), *invocation* (were the right skills/tools called, in order), *behavior-budget* (within token/time/tool-call limits), *output-shape* (schema, text, diff, file presence), and *judgment* (LLM-as-judge, version comparison). Output-shape graders REUSE the validators of `l1-output-contracts.md`; behavior-budget graders reuse the budget model — a suite COMPOSES existing validators and MUST NOT define a parallel validation engine.

- **ES-4 Two grader scopes**: suite-global graders apply to every task; task-local graders apply to one task. Both run on a task; every grader's verdict is attributed to its stable name in the results.

- **ES-5 Weighted thresholded metrics**: a suite declares named metrics, each with a weight and a pass threshold. The suite verdict aggregates per-metric scores by weight; a metric below its threshold fails the suite regardless of the weighted total (a configurable hard-gate). Metric definitions are explicit — a suite never reports a single opaque score.

- **ES-6 Stability via repeated trials**: a task MAY run more than one trial. The suite reports per-task outcome distribution (pass rate, variance), not only a single verdict, so a flaky customization is visible. Determinism MUST NOT be assumed; a single trial is a special case, not the model.

- **ES-7 Frozen and isolated per run**: within one evaluation run the suite definition, task set, graders, and the customization-under-test are frozen (consistent with HE-3). Each task trial executes in an isolated workspace seeded only by declared fixtures, with state reset between trials. No task or trial may observe another's side effects.

- **ES-8 Immutable, attributed, diffable results**: a run emits a timestamped, immutable results artifact recording per-task, per-grader, per-trial verdicts plus aggregated metric scores, plus provenance (suite version, customization version, model, executor). Results are archivable and diffable across runs; a results artifact is never edited in place.

- **ES-9 Baseline comparison and regression gate**: a run MAY compare against a baseline — a prior results artifact or a baseline customization version. A metric drop beyond a declared tolerance is flagged as a regression. This is the mechanism by which a suite gates customization evolution and definition-of-done.

- **ES-10 Activation correctness is first-class**: a suite MAY assert whether the customization *activates* for a prompt — positive cases must trigger, negative cases must not, against a confidence threshold — independently of output correctness. Over-triggering and under-triggering are first-class suite failures, not warnings.

- **ES-11 No production side effects**: an evaluation run is sandboxed. Side-effecting actions are dry-run, simulated, and recorded (reusing the quarantine of `l1-automation-pipeline.md` §4.9); a grader that requires a real side effect (a file created, a config written) observes the isolated workspace, never production state. An eval run MUST NOT mutate user data.

- **ES-12 Scaffold then maintain**: a suite MAY be bootstrapped from its customization — a scaffold derives starter tasks and default graders from the customization's declared triggers and intents — but the scaffold is a starting point, never authoritative coverage. Coverage gaps are the author's responsibility; the suite is a maintained artifact, not a generated one.

- **ES-13 Control-arm attribution**: a run MAY evaluate, alongside the candidate customization and the no-customization baseline (ES-9), one or more *control arms* — cheap instructions that state the same surface intent without the customization's structure. A metric gain is credited to the customization only to the extent it exceeds the **best** control arm, not merely the baseline; a customization that does not beat a naive prompt stating the same intent has not earned its complexity. A control arm that matches the candidate is a reportable finding (the effect is achievable without the customization), not a harness failure.

- **ES-14 Separated non-negotiable quality tier**: metrics partition into an efficiency/behavior tier and a quality/safety tier, scored independently. The quality tier is a hard floor — a run where the candidate improves efficiency metrics while regressing the quality tier **fails outright**, regardless of weighted total (no efficiency gain buys back a quality regression). Quality-tier tasks MAY state their requirement *implicitly* (as a real task reads) so a customization that silently drops it is caught, and MAY execute the produced artifact against adversarial input within the ES-11 sandbox.

- **ES-15 Judge trust gate**: a judgment grader's verdicts are trusted for a run only after the judge passes a discrimination self-test — for a reference pair it MUST rank the known-worse artifact strictly worse than the known-better one on the judged axis; a judge that cannot tell the references apart is not used. A trusted judgment grader is auditable: fixed judge model and settings, a published rubric, and every verdict cites the specific construct/evidence it scored (or "none").

- **ES-16 Signed metrics**: a metric declares its direction, and a minimization metric measures *unnecessary* volume, not total volume. An output that is a positive signal — a test written, an explanation the user explicitly requested — MUST NOT be counted against a size/cost metric. A suite never penalizes a necessary addition as bloat.

- **ES-17 Sliceable results by orthogonal labels**: [ADDED v1.2.0] a task carries labels on a fixed set of **orthogonal** dimensions (ES-2 tags, sharpened into declared slice axes — e.g. capability, complexity, scenario, modality, environment, source), and a run reports **per-slice macro-averaged** scores over those dimensions, not only the aggregate suite verdict. A regression MUST be localizable to a slice; the aggregate is a summary, never the only signal. Macro-averaging (mean of per-value means) prevents a populous slice from masking a small brittle one. This is the suite-level companion to the cross-harness co-evaluation matrix (`l1-agent-coevaluation.md`): a suite slices one customization's results; the co-evaluation matrix slices across the model×harness grid using the same label discipline.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 Suite and Task Structure

A suite is a definition plus a set of task units:

```text
[REFERENCE]
EvaluationSuite:
  target        : customization identity (skill / role / workflow under test)
  version       : suite spec/version label
  config        : run settings — trials_per_task, timeout, parallelism, executor,
                  model, judge_model, fail_fast, max_attempts, required/disabled skills
  metrics       : [ { name, weight, threshold, description } ]   -- ES-5
  graders       : [ Grader ]                                     -- suite-global (ES-4)
  tasks         : [ Task ]   (inline or referenced by glob/external file)

Task:
  id            : stable identifier (appears in results)
  name, tags, group, enabled
  inputs        : { prompt, context?, fixture_files? }
  expected?     : { should_trigger?, output_contains?, output_not_contains?,
                    outcomes?, behavior_limits? }                -- ES-2
  graders       : [ Grader ]   -- task-local (ES-4)
```

### 4.2 Grader Taxonomy

A grader is a typed validator. The five families and what each judges:

| Family | Judges | Example grader kinds | Reuse |
| --- | --- | --- | --- |
| Activation | Did the customization trigger for this prompt? | trigger (positive/negative + threshold) | ES-10; `l1-extensions.md` trigger model |
| Invocation | Were the right skills/tools called, in order? | skill-invocation (ordered/unordered), action-sequence, tool-constraint (expect/reject patterns) | `l1-tool-composition.md`, `l2-sandbox-policy.md` |
| Behavior-budget | Did the run stay within limits? | behavior (max tokens / duration / tool-calls) | `l2-budget-engine.md` |
| Output-shape | Is the output well-formed and correct? | json-schema, text (regex match/not-match), diff (expected file edits), file (must-exist), code (assertions) | **`l1-output-contracts.md`** validators |
| Judgment | Subjective quality, version comparison | prompt (LLM-as-judge pass/fail), comparison (A vs B) | `l2-quality-pipeline.md` LLM-as-judge |

A grader produces a verdict (pass/fail) and optionally a score in `[0,1]` that feeds metric aggregation (ES-5). Output-shape and behavior-budget graders are thin adapters over validators that already exist elsewhere — the suite is the composition layer, not a new validation engine (ES-3).

### 4.3 Metrics and Verdict Aggregation

```text
[REFERENCE]
Per task:   each grader → verdict (+ optional score)
Per metric: metric_score = weighted/normalized roll-up of the graders mapped to it
Suite:      total = Σ (metric_score × weight)
            PASS iff total ≥ overall threshold
                 AND every metric ≥ its own threshold   (ES-5 hard-gate)
```

A suite reports the total, every metric score, and every grader verdict — never a single opaque number. This lets an author see *why* a suite passed or failed, not just *that* it did.

### 4.4 Trials and Stability

Each task runs `trials_per_task` times (default 1). The suite reports, per task, the pass rate and outcome spread across trials. A task that passes 3/4 trials is materially different from one that passes 4/4; ES-6 makes that difference visible so flakiness is treated as a defect, not noise. Stability itself can be a metric (e.g. "≥ 0.9 pass rate across trials").

### 4.5 Baseline Comparison and Regression (ES-9)

```text
[REFERENCE]
RUN      — evaluate the candidate customization → candidate results
BASELINE — load a prior results artifact OR evaluate a baseline version
DIFF     — per metric: candidate_score − baseline_score
REGRESS  — any metric drop beyond its declared tolerance → flagged regression
VERDICT  — improvement / neutral / regression, per metric and overall
```

Regression gating is what lets skill evolution (`l2-self-improvement.md`) accept an edit only when the suite shows no regression — the dynamic analogue of its best-vs-current comparison.

### 4.6 Run Lifecycle

```text
[REFERENCE]
1. RESOLVE   — locate the suite for the target customization; load tasks + graders
2. FREEZE    — snapshot suite, tasks, graders, customization version (ES-7)
3. EXECUTE   — for each task × trial: isolated workspace, seed fixtures, run the
               customization under the configured executor/model, sandboxed (ES-11)
4. GRADE     — run suite-global + task-local graders on each trial (ES-4)
5. AGGREGATE — roll graders → metrics → suite verdict (ES-5); compute stability (ES-6)
6. COMPARE   — optional baseline diff + regression flags (ES-9)
7. EMIT      — immutable timestamped results artifact with full provenance (ES-8)
```

### 4.7 Scaffold-then-Maintain Authoring (ES-12)

A scaffold reads the customization's declared triggers and intents and emits starter tasks (one positive activation case per declared trigger, a default behavior-budget grader, a default output presence check). The author then adds negative activation cases, edge cases, domain-specific graders, and fixtures. The scaffold accelerates the first version; it never certifies coverage. A suite that still contains only scaffolded tasks is reported as under-developed.

### 4.8 Static and Dynamic Quality Are Two Halves

| | Static analysis (`l2-quality-pipeline.md`) | Dynamic evaluation (this spec) |
| --- | --- | --- |
| Operates on | the customization document | the customization's behavior on tasks |
| Catches | contradiction, ambiguity, persona drift, cognitive overload, coverage gaps, composition conflicts | wrong activation, broken invocation, output regressions, budget blowups, flakiness |
| Cost | cheap, no execution | requires running the customization |
| Blindspot | cannot see runtime behavior | cannot see latent document defects that this task set happens not to exercise |

A mature customization pairs both: lint the document, then run the suite. Neither half's job is satisfied by the other.

### 4.9 Control Arms, Quality Tier, and Judge Trust (ES-13…ES-16)

**Control arms (ES-13).** Baseline comparison (ES-9) answers "is the customization better than nothing?" — but a topic-shaped instruction with no real structure can move a metric too. To attribute a gain to the customization's *design*, a run adds control arms: cheap prompts stating the same surface intent ("be concise", "prefer the simplest solution") without the customization's machinery. The credited effect is the candidate's margin over the best control, not over baseline.

```text
[REFERENCE]
arms = { baseline (no customization), candidate, control₁…controlₙ (naive, same intent) }
credited_effect(metric) = candidate_score − max(control_scores)     -- not − baseline_score
A candidate that fails to beat its best control on its headline metric has not earned its complexity.
```

**Quality tier as a hard floor (ES-14).** Metrics split into two independently scored tiers:

| Tier | Measures | Gate |
| --- | --- | --- |
| Efficiency / behavior | size, tokens, cost, latency, budget adherence | weighted + thresholded (ES-5) |
| Quality / safety | correctness, validation, error-handling, security, accessibility | hard floor — any regression fails the run |

Quality-tier tasks may leave the requirement **implicit** (the way a real ticket reads) so an arm that silently drops it is caught, and may execute the produced artifact against adversarial input inside the ES-11 sandbox. No efficiency gain buys back a quality-tier regression — the run fails even if the weighted total rose.

**Judge trust (ES-15).** Before a judgment grader's verdicts count, the judge runs a self-test on reference pairs: it must rank a known-worse artifact strictly below a known-better one on the judged axis. A judge that cannot separate the references is not trusted for that run. Trusted judges are auditable — fixed model and settings, a published rubric, and every verdict cites the specific construct it scored (or "none").

**Signed metrics (ES-16).** A minimization metric measures *unnecessary* volume, not total volume. Positive-signal outputs — a test written, an explicitly requested explanation — are excluded from size/cost metrics, never counted as bloat. This keeps a "less code" metric from punishing the check that makes the code safe.

This methodology is customization-agnostic: it scores a skill, a role, or a workflow definition (including a workflow authored in the agent workflow DSL) the same way — candidate against baseline and naive controls, efficiency separate from a quality floor.

## 5. Drawbacks & Alternatives

- **Maintenance cost.** Suites rot if the customization changes and tasks do not. Mitigation: scaffold-assisted authoring (ES-12), and reserve suites for customizations whose behavior materially matters rather than every trivial skill.
- **LLM-judge non-determinism.** `prompt`/`comparison` graders inherit model variance. Mitigation: prefer structured graders (schema/text/trigger/behavior) where possible; use trials (ES-6) and a separate `judge_model` to bound variance; treat judge verdicts as scored, not binary, where possible.
- **Alternative — per-story independent tests only** (`l2-quality-pipeline.md` §4.19): adequate for one-off story verification, but gives no skill-level coverage, no regression gate, and no stability signal. Rejected as the sole mechanism; the two compose (a story test is a task, a suite is the skill-level aggregate).
- **Alternative — static lint only:** cheap but blind to behavioral regressions and mis-activation. Rejected as sufficient; §4.8 makes the case for pairing.
- **Alternative — record/replay golden transcripts:** pin exact transcripts and diff. Brittle under model non-determinism; the grader taxonomy (semantic/structural assertions) tolerates benign variation that transcript-diffing would flag as failure.

## 6. Ideas to Adopt (Mapping)

| Idea | Where it lands |
| --- | --- |
| Declarative eval suite bound to a skill | ES-1, ES-2 — new artifact |
| Typed grader taxonomy (5 families) | ES-3, §4.2 — composing existing validators |
| Activation (trigger) testing | ES-10 — first-class, new |
| Invocation / action-sequence graders | ES-3 Invocation family — new, over `l1-tool-composition.md` |
| Weighted thresholded metrics with hard-gate | ES-5, §4.3 |
| Trials-per-task stability | ES-6, §4.4 — new |
| Baseline / regression comparison | ES-9, §4.5 — feeds `l2-self-improvement.md` |
| Sandboxed, side-effect-free runs | ES-11 — reuses AP-13 §4.9 quarantine |
| Scaffold-then-maintain authoring | ES-12, §4.7 |
| Static lint as complementary half | §4.8 — `l2-quality-pipeline.md` |
| Control-arm attribution (beat a naive control, not just baseline) | ES-13, §4.9 — extends ES-9 |
| Separated non-negotiable quality/safety tier (implicit requirement, adversarial exec) | ES-14, §4.9 |
| Judge trust self-test + cite-the-construct auditability | ES-15, §4.9 — strengthens ES-3 Judgment |
| Signed metrics (tests / requested explanation never counted as bloat) | ES-16, §4.9 |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[HARNESS]` | `.design/main/specifications/l1-harness-engineering.md` | Frozen evaluation pipeline (HE-3) that runs a suite; held-out discipline (HE-6) |
| `[OUTPUT-CONTRACTS]` | `.design/main/specifications/l1-output-contracts.md` | Output-shape grader validators reused by ES-3 |
| `[QUALITY-STD]` | `.design/main/specifications/l1-quality-standards.md` | A passing suite as definition-of-done gate |
| `[QUALITY-PIPE]` | `.design/main/specifications/l2-quality-pipeline.md` | Static document lint (complementary half); behavior-probe precedent |
| `[SELF-IMPROVE]` | `.design/main/specifications/l2-self-improvement.md` | Training rollouts run suites; baseline comparison consumer |

## Document History

| Version | Date | Notes |
| --- | --- | --- |
| 1.0.0 | 2026-06-25 | Initial spec — evaluation suites as companion artifacts (ES-1, ES-2); typed grader taxonomy across five families composing existing validators (ES-3); global/task grader scopes (ES-4); weighted thresholded metrics with hard-gate (ES-5); stability via trials (ES-6); frozen+isolated runs (ES-7); immutable diffable results (ES-8); baseline/regression gate (ES-9); first-class activation-correctness testing (ES-10); sandboxed no-production-side-effects (ES-11); scaffold-then-maintain authoring (ES-12); static-vs-dynamic quality framing (§4.8). |
| 1.1.0 | 2026-06-25 | Minor — control-arm attribution: credit a customization only for its margin over the best naive same-intent control (ES-13); separated non-negotiable quality/safety tier with implicit-requirement adversarial tasks, no efficiency gain buys back a quality regression (ES-14); judge trust self-test + auditable cite-the-construct verdicts (ES-15); signed metrics that never penalize necessary additions as bloat (ES-16); §4.9 added; fixed a stray "ES-13" phrasing in §4.8. Mined from an external code-minimalism customization's benchmark methodology. Re-reviewed (spec-critic + prompt-engineer PASS). |
| 1.2.0 | 2026-07-02 | Minor — ES-17 sliceable results: a task carries orthogonal declared slice labels and a run reports per-slice macro-averaged scores (not only the aggregate), so a regression localizes to a slice; macro-average prevents a large slice masking a small brittle one. Suite-level companion to the new l1-agent-coevaluation model×harness matrix (shared ACE-3/ACE-4 label discipline). Related Specification link added. Mined from an external model×harness benchmark methodology. |

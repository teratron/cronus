# Agent Co-Evaluation

**Version:** 1.0.1
**Status:** Stable
**Layer:** concept

## Overview

The same model behaves very differently once it is placed inside a real agent
runtime. A failure may come from the model's reasoning, from the harness (missing
tools, weak skill discovery, poor workspace awareness, brittle web access), or from
the grader (a completion check too loose or too strict). A single aggregate pass-rate
cannot separate these causes — it reports *that* the agent failed, never *which layer*
did.

This spec defines the discipline that makes agent quality **diagnostic** rather than
merely ranked. It rests on one claim:

> **Agent performance is a function of both the model and the harness.**

The unit of evaluation is therefore the **(model, harness) pair**, evaluated over a
task set whose every task is labeled on a fixed set of **orthogonal dimensions**, so
results can be sliced to localize *where* a harness is brittle. A **comparability
contract** keeps cross-harness numbers meaningful, and every failure is **attributed**
to one of the model / harness / grader layers. It is the evaluation *methodology*
layer; `l1-evaluation-suites` supplies the grading machinery it runs, and
`l1-harness-engineering` supplies the frozen pipeline it evolves against.

## Related Specifications

- [l1-evaluation-suites.md](l1-evaluation-suites.md) — the grading machinery (golden tasks, typed grader taxonomy, metrics, judge trust) this methodology drives; ES-17 per-slice reporting is its suite-level companion.
- [l1-harness-engineering.md](l1-harness-engineering.md) — HE-1 (harness≠model) is the premise; HE-3 frozen pipeline and HE-5 single-component change are composed by ACE-8 targeted-slice iteration.
- [l1-dynamic-harness.md](l1-dynamic-harness.md) — DH-6 contract-preserving substitution; ACE-5 strengthens it into an evaluation-comparability contract, and the M×H matrix is the diagnostic complement to runtime substitution.
- [../../nodus/specifications/l1-nodus-environment.md](../../nodus/specifications/l1-nodus-environment.md) — a graded run (reset/step/evaluate + reward + trajectory) is one matrix cell's execution; grading-mode + task-label slicing there feed the slices here.
- [l1-practice-analytics.md](l1-practice-analytics.md) — normalized traces a slice reads; the analytics surface the matrix renders onto.
- [l1-quality-standards.md](l1-quality-standards.md) — a co-evaluation matrix is a definition-of-done signal for a harness change.

## 1. Motivation

cronus builds agent *offices* — model + harness + tools + workspace — and evolves
them. Two evaluation capabilities already exist: `l1-evaluation-suites` scores a
*customization's* marginal effect, and `l1-harness-engineering` *evolves one harness*
against evidence. Neither answers the diagnostic question that matters when a whole
office underperforms:

- **Is it the model or the harness?** Swapping the model is expensive; rebuilding the
  harness is expensive; guessing wrong wastes both. Only varying each independently,
  and reading the two gaps separately, tells you which lever to pull.
- **Where exactly is it weak?** A single 68% hides that skill-heavy tasks score 41%
  while text tasks score 74%. Without an orthogonal slice taxonomy, the weak
  capability is invisible and the fix is a guess.
- **Did my harness change actually help?** A change that raises the aggregate might
  have helped one slice and regressed another. Only rerunning the *targeted* slice,
  holding everything else frozen, credits the change honestly.
- **Was it even a real failure?** A grader false-positive looks identical to a model
  failure in an aggregate. Attribution across the model / harness / grader triad is
  the difference between fixing the agent and "fixing" a correct answer the grader
  wrongly rejected.

The invariants below make these four questions answerable by construction.

## 2. Constraints & Assumptions

- The unit of evaluation is the (model, harness) pair; the model alone is never the
  subject when a harness is present.
- Evaluation is non-deterministic; the matrix reports distributions per cell, reusing
  the trials/stability model of `l1-evaluation-suites` (ES-6). This spec adds the
  cross-factor structure, not a second grading engine.
- Comparability is a precondition, not a nicety: numbers from harnesses that do not
  share the frozen contract are not placed in the same matrix.
- This is a diagnostic methodology, not a public benchmark: it runs on-device against
  the user's own offices; there is no leaderboard-submission obligation. (The
  comparable-result-schema idea is adopted; the public-leaderboard machinery is not.)

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **ACE-1 Two-factor performance**: agent performance is a function of both the model
  and the harness. The unit of evaluation is the **(model, harness) pair**; an
  aggregate score that conflates the two factors MUST NOT be the only reported signal.

- **ACE-2 Independent-factor matrix**: evaluation varies model and harness
  independently so that a **harness gap** (fixed model, vary harness) and a **model
  gap** (fixed harness, vary model) are both readable from one run set. A
  one-dimensional sweep (models only, or harnesses only) is a projection of the
  matrix, never a substitute for it, and MUST be labeled as such.

- **ACE-3 Orthogonal task taxonomy**: every task is labeled on a fixed set of
  orthogonal dimensions (for example scenario, capability, complexity, modality,
  environment, and provenance/source). Labels are declared with the task, not inferred
  per run, so a slice means the same thing across runs and across harnesses.

- **ACE-4 Per-slice macro-averaged reporting**: results are reported per slice as a
  **macro-average** (the mean of per-slice means, so a large slice cannot mask a small
  weak one), not only as one aggregate. Brittleness MUST localize to a slice; the
  aggregate is a summary, never the diagnostic. Micro- and macro-averages, when both
  are shown, are labeled distinctly.

- **ACE-5 Harness comparability contract**: harnesses placed in one matrix MUST share
  a frozen evaluation contract — identical task prompt, workspace contract, timeout
  behavior, transcript format, and result schema. A harness that alters any of these
  is **not comparable**: its scores are excluded from that matrix and reported
  separately with the deviation named. Comparability is checked, not assumed.

- **ACE-6 Triad failure attribution**: a failed task is attributed to a cause class —
  **model** reasoning, **harness** capability, or **grader** defect — never collapsed
  into "the agent failed." The grader is a fallible subject: a suspected
  false-positive or false-negative is a distinct, first-class finding (composing the
  judge-trust self-test of `l1-evaluation-suites` ES-15), not silently counted as a
  model or harness failure.

- **ACE-7 Comparable result schema & normalized import**: a task may be imported from
  an external evaluation set and **normalized** into the one comparable result schema
  and taxonomy rather than authored from scratch. Provenance (source) is a first-class
  slice dimension so an imported set's bias is visible, and a normalized task carries
  the same prompt/workspace/grader contract as a native one or it is not admitted.

- **ACE-8 Targeted-slice iteration**: a harness change is validated by rerunning the
  **targeted slice** and confirming the targeted score actually moved, with the model,
  task set, and grader held frozen (composing `l1-harness-engineering` HE-5
  single-component change and HE-3 frozen pipeline). A change that does not move its
  targeted slice earns no credit; a change that moves the target but regresses another
  slice is reported as a trade, never as an unqualified win.

- **ACE-9 Diagnostic preservation**: every evaluated cell preserves its transcript,
  workspace artifacts, and final environment state, keyed to `(model, harness, task,
  trial)`, so a matrix cell is replayable and a failure attributable after the fact
  (composing observability and the run trajectory). A score without a preserved,
  replayable trace is a ranking, not a diagnostic.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The Model × Harness Matrix

```text
[REFERENCE]
            harness_A      harness_B      harness_C     ← model gap (read a row)
model_1   [ cell(1,A) ]  [ cell(1,B) ]  [ cell(1,C) ]
model_2   [ cell(2,A) ]  [ cell(2,B) ]  [ cell(2,C) ]
model_3   [ cell(3,A) ]  [ cell(3,B) ]  [ cell(3,C) ]
             ↑ harness gap (read a column)

cell(m,h) := distribution over trials of (m, h) on the frozen task set,
             graded, sliced, with preserved traces (ACE-9)
harness_gap(m) := max_h cell(m,h) − min_h cell(m,h)      // fixed model (ACE-2)
model_gap(h)   := max_m cell(m,h) − min_m cell(m,h)      // fixed harness (ACE-2)
```

The harness gap is often "comparable to many model upgrades" — a large harness gap on
a fixed model is the signal that the cheapest capability win is a harness fix, not a
model swap.

### 4.2 Orthogonal Taxonomy and Slicing

```text
[REFERENCE]
task.labels := { scenario, capability, complexity, modality, environment, source }
                                                                    // ACE-3, orthogonal

slice_score(dim, value, cell) := macro_avg( cell.task_scores where task.labels[dim]==value )
report(cell) := { aggregate, per_dim: { dim → { value → slice_score } } }   // ACE-4
```

Macro-averaging (mean of per-value means) prevents a populous slice from hiding a
small brittle one — the reason a single aggregate is a summary, not a diagnostic.

### 4.3 The Comparability Contract

Two harnesses are comparable iff they agree on all five frozen elements (ACE-5):

| Frozen element | Why it must be identical |
| --- | --- |
| Task prompt | different prompts test different tasks |
| Workspace contract | different starting files change difficulty |
| Timeout behavior | a longer budget is a different task |
| Transcript format | attribution reads the transcript; format drift breaks it |
| Result schema | scores are only comparable in one schema |

A harness that deviates is scored on its own, with the deviation named — never
silently mixed into the shared matrix.

### 4.4 Failure Attribution Triad

```text
[REFERENCE]
attribute(failed_task, cell):
    if grader_suspect(failed_task):      return GRADER   // ES-15 self-test flags it
    elif harness_capability_missing(failed_task): return HARNESS  // tool/skill/workspace gap in trace
    else:                                 return MODEL    // reasoning error with capability present
```

The order matters: rule the grader out first (a false rejection would otherwise be
mis-attributed downstream), then the harness (a missing tool is a harness fault, not a
model one), leaving genuine model-reasoning failures last. Each attribution cites the
trace evidence it used (ACE-9).

### 4.5 Reuse & Normalize Import (ACE-7)

Rather than author every task, a task is pulled from an external evaluation set and
normalized into the one comparable schema: its prompt, workspace, expected behavior,
and grader are re-expressed in the native contract, and its `source` label is
recorded. Provenance-as-a-slice makes a source's bias legible (e.g. one imported set
being disproportionately skill-heavy). A task that cannot be normalized to the shared
contract is not admitted to the matrix.

### 4.6 Division of Labour

| Concern | Owner |
| --- | --- |
| The (model, harness) matrix, slices, comparability, attribution | this spec |
| Golden tasks, grader taxonomy, metrics, trials, judge trust | `l1-evaluation-suites` |
| The frozen evaluation pipeline + single-component-change loop | `l1-harness-engineering` |
| One graded run (reset/step/evaluate/reward/trajectory) | `l1-nodus-environment` |
| Runtime substitution over interchangeable runtimes | `l1-dynamic-harness` |

This spec adds the *cross-factor diagnostic structure*; it never re-implements a
grader, a pipeline, or a run.

## 5. Implementation Notes

1. Store each cell's result keyed by `(model, harness, task, trial)` with its trace
   pointer — the matrix, the slices, and attribution are all views over that table.
2. Compute the comparability check before admitting a harness to a matrix; a late
   check corrupts every comparison already drawn.
3. Default to macro-average in the headline; offer micro-average as a labeled
   secondary view.
4. Attribution is advisory and evidence-cited; it guides a fix, it does not overwrite
   a grader verdict.

## 6. Drawbacks & Alternatives

- **Matrix cost.** M models × H harnesses × T tasks × trials is expensive. Mitigated
  by targeted-slice iteration (ACE-8): after a baseline matrix, most reruns are a
  single column against one slice, not the full grid.
- **Attribution is heuristic.** The triad rule can mis-assign a borderline failure.
  Mitigated by evidence-citation (ACE-9) and by ruling the grader out first (ACE-6),
  so the expensive-to-fix layers are only blamed on positive evidence.
- **Alternative — fold into `l1-evaluation-suites`.** Rejected: eval-suites tests a
  *single customization's* marginal effect against a baseline; the two-factor matrix,
  comparability contract, and triad attribution are a distinct cross-cutting
  methodology that *drives* suites, the same way the mesh drove the inbox.
- **Alternative — a public leaderboard.** Rejected as scope: cronus is a
  single-principal on-device product; the comparable-result-schema idea is adopted
  (ACE-7), the submission/leaderboard machinery is not.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[SUITES]` | `.design/main/specifications/l1-evaluation-suites.md` | Grading machinery this methodology drives; ES-17 per-slice reporting companion. |
| `[HARNESS-ENG]` | `.design/main/specifications/l1-harness-engineering.md` | Frozen pipeline (HE-3) + single-component change (HE-5) composed by ACE-8. |
| `[DYN-HARNESS]` | `.design/main/specifications/l1-dynamic-harness.md` | Contract-preserving substitution (DH-6) ACE-5 strengthens. |
| `[NODUS-ENV]` | `.design/nodus/specifications/l1-nodus-environment.md` | One graded run = one matrix cell's execution. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-02 | Core Team | Initial spec — agent co-evaluation methodology: performance as a function of both model and harness with the (model, harness) pair as the unit of evaluation (ACE-1); the independent-factor M×H matrix reading harness-gap and model-gap separately (ACE-2); orthogonal declared task taxonomy (ACE-3); per-slice macro-averaged reporting that localizes brittleness (ACE-4); harness comparability contract freezing prompt/workspace/timeout/transcript/result-schema (ACE-5); failure attribution across the model/harness/grader triad, grader-first (ACE-6); reuse-&-normalize external task import with provenance-as-slice (ACE-7); targeted-slice iteration crediting a harness change only when its targeted slice moves (ACE-8); diagnostic preservation of transcript/workspace/final-state per cell for replay (ACE-9). Drives l1-evaluation-suites, composes l1-harness-engineering / l1-dynamic-harness / l1-nodus-environment; public-leaderboard machinery rejected as out of scope. |
| 1.0.1 | 2026-07-10 | Core Team | Fixed broken cross-workspace Related Specifications link to l1-nodus-environment — it lives in the nodus workspace, so the bare same-directory path resolved to nothing; corrected to the workspace-relative path (the Canonical References entry already used the correct full path). |

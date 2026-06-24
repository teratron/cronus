# Harness Engineering Pattern

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

Harness engineering is the practice of systematically improving the code and configuration *around* a fixed AI model — rather than the model itself. The harness is the totality of what a model sees, how it acts, what it remembers, and which constraints govern it. Improving the harness improves model behaviour without retraining.

This spec defines the harness engineering pattern as a universal protocol applicable to any host system that orchestrates AI models. Within Cronus, the pattern is embodied by the combination of nodus workflow files (as portable harness specifications) and an outer evolution loop that evaluates, analyzes, and improves those files.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) — office-model orchestration; the harness orchestrator is one executor within it
- [l1-workflow-language.md](l1-workflow-language.md) — Cronus workflow layer; uses harness-engineered nodus files as its execution units
- [l2-workflow-runtime.md](l2-workflow-runtime.md) — Rust runtime that executes harness-specified workflows
- [../../nodus/specifications/l1-nodus-language.md](../../nodus/specifications/l1-nodus-language.md) — nodus DSL; a workflow file is the harness spec format
- [../../nodus/specifications/l1-nodus-observability.md](../../nodus/specifications/l1-nodus-observability.md) — trace output that the evolution loop analyzes

## 1. Motivation

AI systems improve along two axes: the model and the harness. Model improvement requires training and is controlled by the model provider. Harness improvement is always within the host system's control. Structured harness engineering extracts the maximum reliable performance from any fixed model by:

- Making failure analysis systematic: traces, not just scores, are the unit of evidence
- Making improvements falsifiable: each change predicts specific task flips; predictions are checked
- Making improvements transferable: a harness evolved on one model generalises to another if it encodes general engineering experience rather than model-specific prompt tricks
- Making improvements auditable: every change has a documented motivation and a recorded outcome

## 2. Constraints & Assumptions

- The base model (or model set) is fixed for a given evolution session; harness changes must not assume a specific model version
- The evaluation pipeline (the logic that decides whether a workflow run succeeded) is frozen from the harness's perspective; a harness amendment cannot alter the scoring signal
- Harness evolution requires a labelled task set with a clear success criterion per task
- The search set and held-out test set must be disjoint; a harness must not be evaluated on the same tasks it was evolved against

## 3. Core Invariants

Rules that any conforming harness engineering implementation MUST honour:

- **HE-1 Harness ≠ model**: the base model is held fixed during a harness evolution session. Harness improvements are orthogonal to model quality. A change that requires the base model to change is an infrastructure change, not a harness improvement.

- **HE-2 Workflow file as harness spec**: a nodus workflow file (`.nodus`) is the authoritative harness specification. It encodes all six harness components (§4.1) in a language-agnostic, schema-validated, version-controlled artifact. Any system that uses nodus workflows inherits harness specification for free.

- **HE-3 Frozen evaluation pipeline**: the scoring signal — the logic that determines whether a workflow run succeeded on a given task — is immutable from the harness's perspective. It may be a unit test suite, a judge model, a human reviewer protocol, or a deterministic oracle. Whatever it is, harness evolution must not modify it.

- **HE-4 Evidence-backed amendment**: a workflow amendment is accepted only when accompanied by four fields:
  1. *Failure evidence* — the task identifiers and trace excerpts (citing `run_id`, `step_index`) that motivate the change
  2. *Root cause* — the harness component (step sequence, schema, macro, memory contract, policy, trace config) that caused the failure and why
  3. *Targeted fix* — the specific diff to the harness component that addresses the root cause
  4. *Predicted impact* — which tasks are expected to flip from fail to pass, and which may regress

- **HE-5 Single-component change**: each evolution iteration changes exactly one harness component per amendment. Mixing a step-sequence change with a schema vocabulary change in the same iteration prevents attribution — it is unknown which change caused the observed flip. Compound changes are valid in a release merge but not in a traceable evolution iteration.

- **HE-6 Transfer validity**: after evolution on the search set, the harness must be evaluated on the held-out test set before adoption. If held-out performance is materially lower than search-set performance, the harness has overfit and must be rolled back or re-evolved with stricter evidence requirements.

- **HE-7 Append-only evolution history**: each iteration's harness artifacts (workflow files, traces, scores) are stored immutably. Prior iterations are never overwritten. The history is the audit trail.

- **HE-8 Artifact extraction mandatory**: the ANALYZE phase MUST produce at least one durable artifact — an external, persisted object (compressed failure pattern set, extracted rule list, causal failure model) — as its primary output. Analysis that remains only as accumulated conversational context is not sufficient; without an external artifact, the analysis is lost when the context window closes, and the next iteration degenerates to the same strategy. The artifact is the compressed knowledge that carries the iteration's learning forward.

- **HE-9 Context freshness per iteration**: each new EVALUATE→ANALYZE→IMPROVE iteration MUST open with a fresh context window loaded with the artifact produced by the previous ANALYZE phase. It MUST NOT inherit the accumulated conversational history from all prior iterations. Accumulated history causes context rot: noise accumulates faster than signal, the model's effective attention on the current task shrinks, and successive iterations converge toward identical outputs regardless of artifact content. The artifact IS the compressed continuity; raw history is noise.

## 4. Detailed Design

### 4.1 Harness Component Taxonomy

A complete harness consists of exactly six independently evolvable components:

| # | Component | Description | nodus representation |
| --- | --- | --- | --- |
| 1 | Workflow spec | Step sequence, declarations, input/output contract | `§wf:` file: `@steps`, `@in`, `@out`, `@err` |
| 2 | Schema | Command vocabulary, reserved variables, rule definitions | `§schema:` file: `KNOWN_COMMANDS`, rule blocks |
| 3 | Macro library | Reusable step patterns; named sub-harnesses | `@macro:` declarations within a workflow |
| 4 | Memory contract | Context shape, session variables, persistent state references | `@ctx:` field declarations + runtime `$ctx` bindings |
| 5 | Policy config | Hard constraints, soft preferences, approval gates, spend caps | `!!NEVER`/`!PREF` blocks + PolicyProvider parameters |
| 6 | Trace config | Observability depth, sampling rate, AuditProvider binding | `@runtime:` section + AuditProvider registration |

Each component can be evolved independently; HE-5 requires evolving only one per iteration.

### 4.2 Evolution Loop Protocol

The harness evolution loop follows three phases repeated until a target criterion is met or a budget is exhausted:

```text
[REFERENCE]
Phase 1 — EVALUATE
  1. Run the current harness over the task search set
  2. Collect per-run manifests and per-step traces via AuditProvider
  3. Score each run against the frozen evaluation pipeline
  4. Record: (task_id, run_id, score, trace) for every run

Phase 2 — ANALYZE
  5. Compress raw traces into a structured analysis report:
     - Cross-task root-cause summary
     - Per-task failure attribution (which component, which step, which error code)
     - Pattern detection: recurring failure classes across tasks
  6. Identify the highest-frequency failure class
  7. WRITE the analysis as a durable artifact to external storage (HE-8):
     artifact contains: iteration number, top failure classes with frequencies,
     root-cause hypotheses, and proposed amendment direction.
     A cycle that does not write this artifact is incomplete regardless of IMPROVE output.

Phase 3 — IMPROVE
  8. Open a FRESH context window loaded with the artifact from Phase 2 (HE-9).
     Do NOT carry forward the raw trace history or prior-iteration chat context.
  9. Propose an amendment targeting the identified root cause
     — must satisfy HE-4 (four-field evidence requirement)
     — must target exactly one harness component (HE-5)
  10. Apply the amendment to produce the next harness candidate
  11. Evaluate candidate on the search set
  12. Check: did the predicted tasks flip?
       — Yes → adopt as the next iteration's harness
       — No  → discard candidate; investigate mis-prediction; revise root cause
             → revision uses the artifact (HE-8/HE-9), not replayed history

Termination:
  Stop when target score is reached, budget is exhausted,
  or K consecutive iterations produce no improvement.

Post-evolution:
  Evaluate the final harness on the held-out test set (HE-6).
```

### 4.3 Workflow File Versioning

Each evolved harness iteration is a distinct version of the workflow file. Version bumps follow the nodus portability contract (`l1-nodus-portability.md` LP-6):

- `patch` — typo-only, no semantic change
- `minor` — additive changes to step sequence, macros, or schema extension
- `major` — change to `@in`/`@out`/`@err` contract, schema vocabulary removal, or policy constraint removal

Each iteration's workflow file is committed with a message that includes the evidence fields from HE-4, so the amendment is traceable through git history without reading the trace store.

### 4.4 Transfer Validity Protocol

A harness that was evolved against one base model is transferable to a different model if:

- The harness changes encode structural improvements (step decomposition, macro extraction, policy tightening) rather than prompt-text tuning
- The harness passes HE-6 on the held-out set with the new model without re-evolution
- The harness makes no assumptions about model-internal behaviour (no references to model-specific response formats or quirks)

A harness that fails HE-6 transfer requires one of:

1. Re-evolution from the baseline harness using traces from the new model
2. Addition of a model-specific adapter layer in the schema extension (not in the nodus core)

### 4.5 Budget and Termination Criteria

Evolution budgets should be declared before a session begins:

| Dimension | Description |
| --- | --- |
| `max_iterations` | Hard cap on `evaluate → analyze → improve` cycles |
| `target_score` | Stop when held-out performance exceeds this threshold |
| `patience` | Stop after K consecutive iterations without improvement on the search set |
| `token_budget` | Optional: hard cap on model API calls across all evaluation runs |

Budget exhaustion without reaching `target_score` is not a failure — it is a signal to increase the budget or re-examine the harness taxonomy (a component may have been misidentified as evolvable when it is actually frozen).

## 5. Implementation Notes

The order of implementation for a conforming harness evolution system:

1. Establish the frozen evaluation pipeline (HE-3) first — nothing can proceed without a stable scoring signal
2. Wire the AuditProvider (from `l1-nodus-observability.md`) to produce structured traces — traces are the analysis input
3. Build the ANALYZE phase as a standalone tool that consumes traces and produces a structured root-cause report
4. Build the IMPROVE phase as a constrained proposer that must emit all four HE-4 evidence fields before writing any workflow file
5. Wire the three phases into the loop; add budget tracking last

## 6. Drawbacks & Alternatives

**Alternative: model fine-tuning instead of harness evolution** — improve the model itself to perform better on the task set. Not excluded, but orthogonal: harness evolution is always within the host's control; fine-tuning requires model-provider involvement. Both can be applied.

**Alternative: unconstrained free-form amendment** — allow the proposer to change any aspect of the harness in any iteration. Rejected: violates HE-5. Without single-component attribution, failures are uninterpretable and improvements are unreliable.

**Alternative: continuous online learning** — evolve the harness in real time during task execution rather than in discrete iteration loops. Acceptable if the frozen evaluation pipeline is maintained and HE-7 (append-only history) is preserved. The discrete loop is the baseline; continuous variants must satisfy all same invariants.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[NODUS-LANG]` | `.design/nodus/specifications/l1-nodus-language.md` | The harness spec format (nodus DSL invariants NL-1…NL-10) |
| `[NODUS-OBS]` | `.design/nodus/specifications/l1-nodus-observability.md` | The trace format that feeds Phase 2 (ANALYZE) |
| `[ORCHESTRATION]` | `.design/main/specifications/l1-orchestration.md` | The office-model context in which the evolution loop operates |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — HE-1…HE-7, six-component harness taxonomy, nodus-workflow mapping, three-phase evolution loop, transfer validity protocol, budget criteria |
| 1.1.0 | 2026-06-24 | Core Team | Added HE-8 (artifact extraction mandatory in ANALYZE phase) and HE-9 (context freshness per iteration via fresh context + artifact load); updated §4.2 ANALYZE/IMPROVE phases to reflect artifact write and fresh-context open steps |

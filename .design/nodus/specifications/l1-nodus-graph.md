# Nodus Workflow Graph

**Version:** 1.0.0
**Status:** RFC
**Layer:** concept

## Overview

A conforming nodus implementation already parses a workflow into an AST and validates it. This spec adds one more view over that same definition: the workflow as a **typed directed graph** — steps, macros, and `§wf:` files as nodes; control flow, data flow, context access, macro calls, workflow routes, and error dispatch as typed edges — and the family of **model-free structural analyses** that view unlocks (step reachability, impact / blast-radius, loop-bound and branch-exhaustiveness checks, a macro call graph).

The graph is a **pure, deterministic projection of the workflow definition** (NL-6 determinism, HO-20 content identity): the same definition yields the same graph, it is never a second source of truth, and it is never hand-edited. It is the workflow-side realization of the relevance that [../../main/specifications/l1-code-intelligence.md](../../main/specifications/l1-code-intelligence.md) §6 records — a parser-agnostic graph core with structural analysis layered on top, transferred from a source tree to a workflow definition — and the analyses it defines are the contract behind the analysis-flag seam the runtime already exposes.

This L1 owns *what the workflow graph is and the invariants it obeys*. It does not own the language it observes (that is `l1-nodus-language`), the execution trace of a run (that is `l1-nodus-observability`), or any concrete store, renderer, or interchange-format library (host concerns per `l1-nodus-portability` LP-1/LP-2).

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — the language whose definition this spec projects: `@steps:` sequence, `→pipeline` / `→ $name` data flow, `@ctx:` fields, `RUN(@macro)`, `ROUTE(wf:name)`, `?IF`/`?SWITCH` branches, `~FOR`/`~UNTIL` loops, `@err:` dispatch, `~compensate` (NL-22). NG-7 defers **branch-exhaustiveness on a generated-value guard** to NL-25/NL-26 — not loop-bound, which NL-25 does not address.
- [l1-nodus-testing.md](l1-nodus-testing.md) — NT-10 route-coverage advisory (`W006`); NG-4 is its **structural-reachability companion** — NT-10 asks whether a routed target has a test, NG-4 whether a step has a path from entry at all.
- [l1-nodus-observability.md](l1-nodus-observability.md) — the run-time counterpart. Observability traces *one execution*; this spec analyses *the definition* before any run. HO-20 content identity is what makes NG-2's projection deterministic.
- [l1-nodus-portability.md](l1-nodus-portability.md) — LP-1/LP-2 host-neutrality: the core computes the graph and every model-free analysis; a model-bearing pass is host-supplied (NG-11).
- [../../main/specifications/l1-code-intelligence.md](../../main/specifications/l1-code-intelligence.md) — the concept this realizes for workflows (CI-1 parser-agnostic core, CI-5 structural analysis family, CI-14 node summaries, CI-15 vocabulary-grounded query, CI-12 export).
- [../../main/specifications/l1-change-merge.md](../../main/specifications/l1-change-merge.md) — NG-5's change-kind classification and NG-6's diff blast-radius feed the delta-granularity model (delta ↔ step / macro).
- [../../main/specifications/l1-automation-canvas.md](../../main/specifications/l1-automation-canvas.md) — a consumer of the NG-10 export: a render source without bespoke serialization.
- [../../main/specifications/l1-requirement-checklists.md](../../main/specifications/l1-requirement-checklists.md) — loop-bound and branch-exhaustiveness coverage (NG-3) are the executable form of checklist items.
- [../../main/specifications/l1-derived-artifact-handoff.md](../../main/specifications/l1-derived-artifact-handoff.md) — DAH-1 derived-only-never-authority: NG-2 is that discipline applied to the workflow graph.

## 1. Motivation

A workflow author asks the same relational questions a code author asks: which steps feed which; what a change to this step can reach; whether this loop can run forever; whether this switch covers its cases; which steps a macro pulls in. The AST answers none of these directly — reconstructing them by re-reading the `@steps:` body is the workflow-scale version of grepping a source tree.

Left unspecified, three failures recur:

1. **Dead steps ship.** A step made unreachable by an earlier `?IF` refactor stays in the file, is never executed, and is never noticed — the route-coverage advisory (NT-10) catches the routed-target case but not the general one.
2. **Blast radius is guessed.** A change to what one step binds — a renamed `→ $name`, a dropped `@ctx:` write — breaks a downstream consumer three steps away, and the author finds out at run time because nothing computed the reachable set.
3. **Unbounded loops and non-exhaustive switches are found in production.** A `~UNTIL` with no `MAX:n`, or a `?SWITCH` with no `*` over an open domain, is a structural defect that a graph check finds statically and a test suite finds only by luck.

Naming the graph once — typed nodes and a closed edge vocabulary, a deterministic projection, a model-free analysis family, a diff blast-radius — makes every one of these a check rather than a surprise.

## 2. Constraints & Assumptions

- **Projection, not state.** The graph is recomputed from the definition; it is cheap enough (a single pass over an already-parsed AST) that no persistent store is required, and no store is authoritative if one is added for caching (NG-2).
- **Model-free core.** Every analysis in NG-3 is derivable from graph topology alone. A model pass may *upgrade* a node summary (NG-8) or *widen* a natural-language query (NG-9); it is never on the path of a structural verdict.
- **Advisory, not blocking.** Graph findings are diagnostics (warning severity), consistent with NT-10. A workflow with a dead step or an unbounded loop still parses and still runs; the finding tells the author, it does not halt the executor. (A hard structural error — a dependency cycle in the macro call graph — is a validation error, surfaced at load, exactly as `l1-nodus-testing` treats a `@test:` dependency cycle.)
- **Host-neutral.** The core names no graph-interchange format, no renderer, and no query engine. NG-10 export and any model-bearing pass are host-supplied (LP-1/LP-2).
- **Deterministic.** Same definition → same graph → same analysis results, byte for byte (NL-6), so a graph or a finding set can be diffed across revisions.

## 3. Core Invariants

Rules that Layer 2 implementations MUST NOT violate:

- **NG-1 Typed graph over a closed edge vocabulary.** A workflow projects to a directed graph whose nodes are steps, `@macro:` definitions, and `§wf:` files, and whose edges belong to a **closed set of verbs, each answering a question a workflow author or reviewer actually asks**: `contains` (a file/macro contains its steps — *where does this live?*), `sequences` (step order in `@steps:` — *what runs next?*), `pipes` (`→pipeline` / `→ $name` data flow — *what breaks if I change this output?*), `reads` / `writes` (`@ctx:` field access — *who depends on this context field?*), `branches-to` (`?IF`/`?ELIF`/`?ELSE`/`?SWITCH` arms — *which paths exist here?*), `loops` (`~FOR`/`~UNTIL` body membership, carrying the declared bound — *is this bounded?*), `calls` (`RUN(@macro)` — *what does this macro pull in?*), `routes-to` (`ROUTE(wf:name)` — *what other workflow does this reach?*), `dispatches-on-error` (`@err:` handler dispatch), `configures` (`§config:` / `§runtime:` field governing a step — *what changes its behavior without a step change?*), and `compensates` (`~compensate`, NL-22). A verb that answers no such question is inadmissible; every edge carries the source position (step index / label) it was projected from.

- **NG-2 Pure deterministic projection; never a second source of truth.** The graph is a deterministic function of the workflow definition's content identity (HO-20, NL-6). It MUST NOT be hand-edited, MUST NOT be an input to validation or execution, and MUST NOT be a place any fact about the workflow is *declared* — losing the graph costs a recomputation and nothing else (DAH-1). Every analysis result cites the step index / label of any concrete claim, so a finding is traceable back to the definition without trusting the graph.

- **NG-3 Model-free structural analysis family.** From the graph alone, with no model, the subsystem derives: **step reachability / dead-step** detection (NG-4); **impact / blast-radius** — the forward closure from a changed step over `pipes` + `writes`→`reads` + `calls`, depth-bounded, each hit tagged with depth and the edge it arrived by, classified by change kind (NG-5); **loop-bound** check — a `~UNTIL` reachable without a `MAX:n` bound on its `loops` edge (this check is independent of NG-7: a `~UNTIL` whose guard reads a generated value still needs a bound, and NL-25/NL-26 supplies none); **branch-exhaustiveness** — a `?SWITCH` with no `*` default over a value whose domain is not closed (NG-7 scopes this to author-written guards); **macro call graph** with hot-path ranking (macros by transitive `calls`-in count) and **god-macro** ranking (a macro referenced far above the median); **context-field lifecycle** — a `@ctx:` field `read` with no prior `write` on any path (use-before-def), or `write` with no `read` (dead write); and a **`calls`-graph cycle** check — advisory today, since recursive `RUN(@macro)` body expansion is itself a deferred nodus feature, and a hard load-time error only on a realization that actually executes a recursive expansion. Traversals are depth-limited to stay bounded on large or recursive workflows (NL-18).

- **NG-4 Dead-step detection.** The **workflow entry** is the first step of `@steps:`; a step with **no path from that entry under any combination of branch outcomes** is a dead step — the workflow analog of unreferenced code. (Where a nodus realization dispatches entry from triggers — `@ON`, a deferred feature — each trigger target is an additional root, and the check is run from the full root set.) NG-4 is the structural-reachability companion to NT-10's route-coverage advisory, not a superset of it: NT-10 asks whether a routed target is *tested*, NG-4 whether a step is *reachable*. It is emitted at **warning severity** with a diagnostic code, is **advisory** (it does not block parse or execution, matching NT-10), and names the step by index and label.

- **NG-5 Impact is classified by change kind.** A change to a step is classified as a **contract change** — its **binding surface**: which `→ $name` it binds, which `@ctx:` fields it `writes`, its argument arity (nodus step outputs carry no statically-declared value shape — NL-7's value space is closed but a step's produced type is not declared, so the contract is *which names and fields it binds*, not a schema) — or a **body-only change**. A contract change propagates along `pipes` and `writes`→`reads` to every downstream consumer; a body-only change usually has an **empty** structural blast radius. This is the high-signal / low-signal split, and it is the input the delta model in `l1-change-merge` consumes at step / macro granularity.

- **NG-6 Blast radius of a workflow diff is a review artifact.** Given two revisions of a `§wf:` file, the subsystem computes the set of steps, macros, routes, and `@ctx:` fields the change can reach (NG-5 applied to the diff's changed steps) and renders it as a review artifact — minimally a text summary attributable per changed step, optionally a rendered graph view (NG-10). The diff blast-radius is the workflow counterpart of a source-tree change/PR bundle; it never modifies the workflow.

- **NG-7 Branch-exhaustiveness defers to the generated-value branch contract.** NG-3's **branch-exhaustiveness** check covers **author-written** control flow. Where a branch is decided by a **generated** value (a `GEN`/`REFINE` `Text`, a delegated external step's reply), the outcome-set obligation is owned entirely by NL-25/NL-26 (a closed, anchored, total outcome set routed to `@err:` on zero or multiple matches); NG applies **no second exhaustiveness check** to that site — the branch contract already does, and double-jeopardy would train authors to suppress it. The deferral is scoped to *which arm*: NG-3's other checks are **not** deferred. **Loop-bound** in particular still applies — a `~UNTIL` whose continuation is decided by a generated value needs a `MAX:n` bound exactly as an author-written one does, and NL-25/NL-26 says nothing about termination bounds; and NG-2's determinism, NG-4's reachability, and NG-5's impact all still project the generated-value site normally.

- **NG-8 Node summaries, deterministic-first, never the authority.** A step or macro node MAY carry a bounded (~one-line) summary of its responsibility, derived **deterministically first** from local signal — the step's command, its inline comment, and its I/O contract (`→ $name`, `@ctx:` reads/writes) — and optionally upgraded by a host-supplied model pass (NG-11). A summary records how it was produced, is off by default, is never the authority (the definition is), and is surfaced under the same token budget as the NG-9 query surface.

- **NG-9 Vocabulary-grounded query.** A natural-language question is expanded against the workflow's **own** vocabulary — its step labels, macro names, and `@ctx:` field names — before traversal, so a wording mismatch between the question and the definition does not collapse recall. The navigable surface offers at least: bounded-neighborhood context around a step, a traced path between two steps, and single-step explanation — each bounded by an explicit token budget and citing the source position of any concrete claim (NG-2).

- **NG-10 Export is a pure projection.** The graph exports to a standard graph-interchange format for the automation canvas (`l1-automation-canvas`) and external tools, carrying each edge's verb (NG-1). Syntax-extracted edges and inferred edges (a resolved generated-value branch outcome per NL-25/NL-26) are **distinguishable** in the export (the provenance distinction). The export is a one-way projection, never a second source of truth, never an import path back into the definition.

- **NG-11 Host boundary.** The nodus core computes the graph (NG-1), the deterministic projection (NG-2), and every model-free analysis (NG-3). A model-bearing pass — a summary upgrade (NG-8), a richer natural-language expansion (NG-9) — is **host-supplied** (LP-1/LP-2); the core names no store, no renderer, and no interchange-format library, and a host that supplies none gets the graph and the structural analyses unchanged.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The node and edge taxonomy

| Node kind | Projected from |
| --- | --- |
| Step | one entry in an `@steps:` / `@macro:` body |
| Macro | a `@macro:` definition |
| File | a `§wf:` file (the module node; `contains` its top-level steps) |

| Edge verb | Projected from | The question |
| --- | --- | --- |
| `contains` | file→step, macro→step | where does this live? |
| `sequences` | adjacent steps in a body | what runs next? |
| `pipes` | `→pipeline`, `→ $name` → later use of `$name` | what breaks if I change this output? |
| `reads` / `writes` | `@ctx:` field access in a step | who depends on this context field? |
| `branches-to` | `?IF`/`?ELIF`/`?ELSE`/`?SWITCH` arm | which paths exist here? |
| `loops` | `~FOR`/`~UNTIL` body membership (+ bound) | is this bounded? |
| `calls` | `RUN(@macro)` | what does this macro pull in? |
| `routes-to` | `ROUTE(wf:name)` | what other workflow does this reach? |
| `dispatches-on-error` | `@err:` handler dispatch | where do failures go? |
| `configures` | `§config:` / `§runtime:` field bound to a step | what changes behavior without a step change? |
| `compensates` | `~compensate` (NL-22) | what undoes this on rollback? |

### 4.2 The structural analysis catalog (NG-3)

All derivable from topology alone:

- **Reachability / dead step** — BFS from the entry over `sequences` + `branches-to` + `loops`; a step in no traversal is dead (NG-4).
- **Impact (blast radius)** — forward closure from a changed step over `pipes` + (`writes` then matching `reads`) + `calls`, depth-limited, each hit tagged `{step, depth, via_verb}`; classified contract-vs-body (NG-5).
- **Loop bound** — every `loops` edge to a `~UNTIL` body carries the declared bound; a missing `MAX:n` on a reachable `~UNTIL` is a finding.
- **Branch exhaustiveness** — a `?SWITCH` with no `*` arm over a value whose type is not a closed enum; author-written sites only (NG-7).
- **Macro call graph** — the `calls` subgraph; hot paths = macros by transitive callers; god macro = caller count far above the median.
- **Context lifecycle** — per `@ctx:` field, the set of `writes` and `reads` steps and the paths between them; use-before-def and dead-write are findings.
- **`calls`-graph cycle** — an SCC over `calls` is a finding; **advisory** while recursive `RUN(@macro)` body expansion remains a deferred nodus feature, promoted to a load-time validation error only on a realization that actually executes recursive expansion.

### 4.3 Diff blast-radius (NG-6)

```text
[REFERENCE]
diff_blast(wf_before, wf_after) -> {
  changed_steps:   [ step_id ],                      // from the definition diff
  reachable:       [ { kind: step|macro|route|ctx_field, id, depth, via_verb } ],
  classification:  contract_change | body_only,      // per changed step (NG-5)
  summary:         "step 4 GEN output binding changed — reaches steps 7, 9 and macro @score",
}
```

Rendered as a text summary (attributable per changed step) and, on request, as an NG-10 graph view highlighting the reachable set. Purely a read over two revisions.

### 4.4 Why a projection, not a stored index

A source-tree code graph earns a persistent store because extraction is expensive (parsing thousands of files) and the corpus changes under it continuously. A workflow definition is a single already-parsed file; the projection is one pass over its AST. So the workflow graph follows the derived-artifact discipline at its strongest: recompute on demand, never persist as authority, never hand-edit (NG-2, DAH-1). A host MAY cache the projection keyed by the definition's content identity (HO-20), but the cache is a convenience, never consulted for a fact the definition itself answers.

## 5. Drawbacks & Alternatives

- **Branch-combination explosion.** Reachability under "any combination of branch outcomes" (NG-4) is exponential in nested branches in the worst case. Mitigated by treating each branch arm as independently reachable (a step reachable under *some* assignment is live) rather than enumerating assignments — the cheap over-approximation that never reports a live step as dead.
- **Alternative — reuse the execution trace (`l1-nodus-observability`).** A trace shows which steps ran *in one execution*; it cannot show a dead step (one that never runs is absent from every trace, indistinguishable from one that simply was not exercised), nor a blast radius before a change is made. The graph analyses the definition; the trace analyses a run. They are complementary.
- **Alternative — fold these checks into the validator (`l1-nodus-language`).** Rejected for the advisory family: a dead step or a dead context write is a smell, not an error, and NT-10 already establishes that coverage-style findings are advisory (`W00x`), not parse failures. A `calls`-graph cycle is the one check that *would* be a hard validation error — but only on a realization that executes recursive macro expansion, which nodus does not yet do, so it too is advisory today (NG-3).
- **Model-free ceiling.** Reachability cannot see that two branch conditions are mutually exclusive by domain logic, so it may call a genuinely-dead step live. Accepted: the over-approximation's only failure is a missed finding, never a false one, and a model pass to prune it is a host concern (NG-11), never a gate.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[LANG]` | `.design/nodus/specifications/l1-nodus-language.md` | The projected language; NL-25/NL-26 own generated-value branch-exhaustiveness (NG-7), not loop-bound |
| `[TESTING]` | `.design/nodus/specifications/l1-nodus-testing.md` | NT-10 route-coverage advisory; NG-4 is its structural-reachability companion |
| `[OBSERVE]` | `.design/nodus/specifications/l1-nodus-observability.md` | The run-time counterpart; HO-20 content identity behind NG-2 |
| `[CONCEPT]` | `.design/main/specifications/l1-code-intelligence.md` | The source-tree concept this realizes for workflows |
| `[MERGE]` | `.design/main/specifications/l1-change-merge.md` | Consumer of NG-5 / NG-6 at step / macro delta granularity |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 (RFC) | 2026-09-09 | Core Team | Post-Update Review (`@role:spec-critic` 5-lens Council + `@role:prompt-engineer`): **PASS-WITH-REWRITES**, `Draft → RFC`. Four corrections applied inline: **NG-7 narrowed** — it deferred *both* loop-bound and branch-exhaustiveness to NL-25/NL-26 for generated-value control flow, but NL-25 owns only *which arm* (outcome-set totality), not termination bounds; loop-bound (NG-3) now explicitly still applies to a generated-value `~UNTIL`. **NG-4** — "workflow entry" defined (first step of `@steps:`; trigger targets as additional roots if `@ON` is ever realized), and "generalizes NT-10" softened to "structural-reachability companion" since NG-4 does not subsume NT-10's *test*-coverage question. **NG-3 `calls`-graph cycle** — downgraded from "load-time validation error" to advisory, since recursive `RUN(@macro)` expansion is a deferred nodus feature; hard-error only on a realization that executes it. **NG-5 / Motivation / §4.3** — "output shape" reworded to "binding surface" (nodus step outputs carry no statically-declared value shape; the contract is *which names and `@ctx:` fields a step binds*). Held at RFC rather than Stable: one authoring pass + one adversarial pass; Stable awaits a second review or an `l2-nodus-graph` authoring pass that stress-tests the concept against real crate shape. Concept-only (no `l2-nodus-graph`), so nothing is gated by RFC-vs-Stable. |
| 1.0.0 | 2026-09-09 | Core Team | Initial spec — the workflow-graph view the `l1-code-intelligence` §6 nodus-relevance section deferred to this workspace, seeded from a fourth code-graph engine's model. NG-1 typed graph over a closed edge-verb vocabulary where each verb answers an authoring/review question; NG-2 pure deterministic projection, never a second source of truth, never hand-edited (DAH-1); NG-3 model-free structural analysis family (reachability/dead-step, impact/blast-radius, loop-bound, branch-exhaustiveness, macro call graph + god-macro, context-field lifecycle, call-graph cycle); NG-4 dead-step generalizes NT-10 route coverage; NG-5 impact classified contract-vs-body feeding `l1-change-merge`; NG-6 diff blast-radius as a review artifact; NG-7 defers generated-value control flow entirely to NL-25/NL-26 (no double-jeopardy); NG-8 deterministic-first node summaries; NG-9 vocabulary-grounded query; NG-10 export as a pure projection carrying edge verbs and extracted-vs-inferred provenance; NG-11 host boundary (model-free core, model-bearing passes host-supplied per LP-1/LP-2). |

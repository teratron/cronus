# Solution Frugality

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

An autonomous coding office generates code. Left undisciplined, a capable model
*over-builds*: it installs a dependency for what a native platform feature already does,
writes a wrapper class around a one-line call, introduces an interface with a single
implementation, and scaffolds "for later" that never comes. Over-building is not a
stylistic preference — it costs review time, review attention, running cost, latency, and
every extra line is surface area for a future bug or vulnerability. The most common
failure of a generated diff is not that it is wrong; it is that it is **larger than the
task needs**.

This spec defines the missing discipline: **solution frugality** — for any construction
task (write, add, refactor, fix), produce the **least code that fully solves the
problem**, resolved through an ordered *reuse-before-build* decision procedure and bounded
by a hard **negligence floor** below which "less" becomes a defect. Its central claim:
**the frugal solution is chosen after the problem is fully understood, never instead of
understanding it** — the discipline shortens the solution, never the reading. It is the
output-side counterpart to `l1-harness-composition`: composition right-sizes the agent's
own *tooling*; frugality right-sizes the *code the agent produces for the user's task*.

## Related Specifications

- [l1-harness-composition.md](l1-harness-composition.md) — the sibling right-sizing discipline, applied to a **different subject**: composition trims the agent's own harness (roles/skills/hooks); this trims the *deliverable code*. Same anti-bloat spirit, non-overlapping targets.
- [l1-quality-standards.md](l1-quality-standards.md) — **orthogonal, composed**: quality asks "is it correct, tested, clean?"; frugality asks "is it the least code that is correct?". Frugality never overrides a quality gate (FR-3/FR-9); the two run together.
- [l1-code-intelligence.md](l1-code-intelligence.md) — the reuse rung (FR-2) needs a queryable view of the existing corpus to answer "does this already live here?"; code-intelligence is that lookup surface.
- [l1-memory-intelligence.md](l1-memory-intelligence.md) — MI-13 experience reuse is the *procedure-level* reuse-before-build (reuse a proven run); FR-2's reuse rung is the *solution-shape-level* counterpart. Same instinct at two granularities.
- [l1-operational-ledger.md](l1-operational-ledger.md) — the disclosed-simplification debt ledger (FR-6) is recorded as ledger entries, not a new store.
- [l2-self-improvement.md](l2-self-improvement.md) — a simplification deferred with an upgrade trigger is kin to the should-have-asked / mistake ledger; a deferral that rots (no trigger) is a self-improvement signal.
- [l1-security.md](l1-security.md) — the negligence floor (FR-3) never cuts security controls or trust-boundary validation; intensity (FR-5) is authority the agent cannot self-elevate (SEC-10).
- [l1-development-workflow.md](l1-development-workflow.md) — the frugality review/audit pass (FR-7) slots into the Review stage as a complexity-only lens, distinct from the correctness/security review.
- [l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) — "recoverable-as-success / sufficiency-to-stop" shares the do-only-what-is-needed instinct at the tool-surface level.
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.6): the reuse rung maps onto `RUN(@macro)` reuse, and the discipline realizes as a frugality-lint family over authored workflows, not a new language primitive.

## 1. Motivation

A generated line of code is never free. It is reviewed, maintained, executed, and
carried forward; each is a place a defect can hide and a cost paid on every run. Yet a
model's default bias is to *add*: adding demonstrates effort, covers speculative futures,
and pattern-matches the over-built codebases it trained on. Nothing in a plain quality
gate pushes back — a 400-line date-picker component and a one-line native input field
both pass tests and lint. Quality answers "is it correct?"; nothing answers "is this the
least that is correct?".

Three forces make over-building the default without a discipline against it:

1. **Adding reads as diligence.** More code, more abstraction, and more configuration
   look like thoroughness, so the model reaches for them to seem helpful.
2. **Speculative generality is invisible until it rots.** An interface with one
   implementation, a config nobody sets, a factory for one product — each looks
   harmless at authoring time and becomes dead flexibility nobody dares delete later.
3. **Re-implementation is easy to do accidentally.** Re-writing a helper that already
   lives a few files over, or hand-rolling what the standard library ships, is the most
   common slop — not because the model can't reuse, but because it doesn't look first.

Without a discipline that makes *reuse-before-build* a reflex and *minimality* the
default shape, generated code only ever grows. This spec makes frugality a first-class,
bounded, auditable property of every construction — and makes the hard floor beneath it
(never cut validation, error handling, security, accessibility, or comprehension)
equally first-class, so frugality can never degrade into negligence.

## 2. Constraints & Assumptions

- Frugality is **orthogonal to correctness**: it governs the *size and shape* of a
  solution, never its correctness, security, or performance. A frugality review
  (FR-7) explicitly does not hunt bugs; those route to the normal correctness pass.
- Frugality is **subordinate to the negligence floor** (FR-3): where minimality and a
  floor concern conflict, the floor wins without argument. "Frugal" never means
  "flimsier" — between two equally small correct options, the one correct on edge cases
  wins.
- The subject is the **deliverable code the office produces**, not the office's own
  harness (that is `l1-harness-composition`). The two are siblings, not the same rule.
- "The least code that works" is defined **only after comprehension** (FR-1): the
  smallest diff in the wrong place is a second defect, not a frugal fix. This spec
  never trades reading for a shorter diff.
- Frugality's headline benefit (less code) is **measurable and falsifiable** (FR-10);
  its secondary benefits (cost, latency) are generator-dependent and are claimed only
  where measured, never asserted doctrinally. Near-zero savings on already-minimal code
  is an honest outcome, not a failure of the discipline.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **FR-1 Comprehension precedes frugality**: the discipline runs *after* the problem is
  understood and every site the change touches has been read and the real flow traced —
  never as a substitute for that reading. A minimal diff produced without understanding
  the change is not frugal; it is a confident wrong fix dressed as efficiency. The
  discipline shortens the solution, never the comprehension.

- **FR-2 The reuse-before-build ladder**: before authoring new code, the solution is
  resolved at the **first satisfied rung** of an ordered ladder, highest wins:
  (1) does it need to exist at all? — speculative need is skipped (YAGNI);
  (2) does it already exist in this codebase? — reuse it, never re-implement;
  (3) does the language's standard library do it? — use it;
  (4) does a native platform capability cover it? — use it;
  (5) does an already-present dependency solve it? — use it, never add a new dependency
  for what a few lines do;
  (6) can it be a single expression? — make it one;
  (7) only then, the minimum new code that works. Two rungs both hold → take the higher
  one. A new dependency added for capability the platform or an installed dependency
  already provides violates this invariant.

- **FR-3 The negligence floor (lazy, not negligent)**: frugality MUST NOT remove
  trust-boundary input validation, error handling that prevents data loss, security
  controls, accessibility basics, explicitly-requested scope, or the calibration a
  physical/external system needs (the real world is never the model's idealization).
  Below this floor, "less" is a defect. When the user insists on a fuller version, it is
  built without re-litigation.

- **FR-4 Root-cause locality**: a frugal correction is applied at the single shared point
  every affected path routes through, not duplicated at each call site. The smallest
  *correct* diff is usually the root-cause one; patching only the path a report names,
  while leaving sibling paths broken, is both larger in aggregate and wrong. Frugality
  and root-cause correctness point the same direction.

- **FR-5 Tunable, non-self-elevated intensity**: the discipline exposes a small ordered
  set of intensity levels — roughly *advisory* (build as asked, name the leaner
  alternative), *enforced* (the ladder applied, the default), *aggressive* (deletion
  before addition, challenge the requirement itself) — plus *off*. The level is set by
  the human principal per office/session, defaults to the enforced middle, and is never
  elevated or relaxed by a model-produced action (composes SEC-10). The active level is
  always self-announced; "off" is honest, not silent.

- **FR-6 Disclosed simplification with an upgrade trigger**: a deliberate corner-cut with
  a known ceiling (a coarse lock, a naive scan, a heuristic that holds only at small
  scale) is recorded *at the cut site* naming both the ceiling and the trigger that
  should reopen it, and is harvestable into a debt ledger so a deferral cannot quietly
  become permanent. A cut recorded without an upgrade trigger is flagged as rot-risk. An
  *undisclosed* corner-cut is a defect, not frugality — the discipline is honest about
  what it skipped.

- **FR-7 Frugality review is complexity-only, scoped, and non-mutating**: an
  over-engineering review is a distinct pass from the correctness/security/performance
  review. It emits findings in a closed vocabulary — *delete* (dead / speculative),
  *stdlib-covered*, *native-covered*, *speculative-abstraction*, *shrinkable* — each
  naming the location, what to cut, and what replaces it, scoped to either a change-set
  (diff) or the whole corpus (audit). It produces a delete-list and a net-lines-saveable
  figure; it does **not** apply the cuts, and it never flags the mandatory minimum check
  (FR-9) as bloat. "Lean already" is a valid, complete result.

- **FR-8 Output economy**: the explanation accompanying a construction never exceeds what
  the construction needs — unrequested justification prose is complexity in another form
  (every paragraph defending a simplification smuggles it back as words). Explicitly
  requested reports, walkthroughs, or per-step notes are **exempt** and given in full;
  the rule is only against *unrequested* prose.

- **FR-9 Proportional verification**: non-trivial logic (a branch, a loop, a parser, a
  money or security path) leaves exactly **one** runnable check that fails if the logic
  breaks — the smallest such check, not a per-function suite unless asked. Trivial
  constructions need none: YAGNI applies to verification too. This composes, and never
  contradicts, the project's mandatory-test quality gate (`l1-quality-standards`) — it
  governs *proportion*, and never licenses skipping a test the quality gate requires.

- **FR-10 Measured, not asserted; and honest about the counterfactual**: the discipline's
  value is measurable against a no-discipline baseline (code volume always; cost and
  latency where the generator follows the ladder) and is claimed only where measured. A
  **per-instance** savings number is never fabricated — the un-built larger version was
  never written, so there is no real baseline to subtract from in a live task; the only
  honest per-project figure is the counted debt ledger (FR-6). The claim is falsifiable,
  not doctrinal.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The ladder as a post-comprehension reflex

```text
[REFERENCE]
construct(task):
    understand(task)                       // FR-1 — read every touched site, trace the real flow
    for rung in [ need_to_exist?,          // 1 YAGNI
                  reuse_in_codebase,        // 2 (needs l1-code-intelligence lookup)
                  language_stdlib,          // 3
                  native_platform,          // 4
                  installed_dependency,     // 5
                  single_expression,        // 6
                  minimum_new_code ]:       // 7
        if rung.satisfies(task):
            return rung.solution            // first hold wins; ties resolve to the higher rung
    // never reached: rung 7 always terminates
```

The ladder is a reflex, not a research project — but it is gated behind `understand()`.
Skipping comprehension to reach a small diff is the dangerous failure mode FR-1 forbids.

### 4.2 The negligence floor (FR-3)

Six things are never on the chopping block, regardless of intensity:

| Floor concern | Why it is below the floor |
| --- | --- |
| Trust-boundary input validation | untrusted input is a security surface, not surplus |
| Data-loss-preventing error handling | a lost write is unrecoverable; "less" here is destruction |
| Security controls | a removed control is a vulnerability, not a saved line |
| Accessibility basics | correctness for all users, not an optional feature |
| Explicitly requested scope | the user's stated intent overrides the default bias to less |
| Physical/external calibration | the real world drifts; a minimal model can't see the tuning it needs |

A simplification that touches any of these is a defect. Frugality is *lazy about the
solution shape, never about these*.

### 4.3 Intensity levels (FR-5)

| Level | Behavior | Set by |
| --- | --- | --- |
| off | discipline dormant; announced, not silent | human principal |
| advisory | build as asked; name the leaner alternative in one line, user picks | human principal |
| enforced (default) | the ladder applied; stdlib/native first; shortest correct diff | human principal |
| aggressive | deletion before addition; ship the minimum and challenge the requirement in the same response | human principal |

The level is office/session-scoped policy authored by the human, never a model
self-grant (SEC-10). A worked example across levels for "add a cache": advisory ships the
cache and names the one-line library alternative; enforced uses the one-line memoization
and skips the custom class; aggressive ships no cache until a profiler demands one.

### 4.4 The two review scopes (FR-7)

```text
[REFERENCE]
frugality_review(scope):                    // scope ∈ { diff, whole-corpus }
    findings := []
    for unit in scope:
        if dead_or_speculative(unit):   findings += (delete,  unit, replacement=∅)
        if reinvents_stdlib(unit):      findings += (stdlib,  unit, name_the_fn)
        if duplicates_native(unit):     findings += (native,  unit, name_the_feature)
        if single_use_abstraction(unit):findings += (yagni,   unit, inline_it)
        if shrinkable(unit):            findings += (shrink,  unit, shorter_form)
    return rank(findings, by=lines_saved_desc), net_lines_saveable   // never applies them
```

Diff-scope is the review of a change in flight (the Review stage of
`l1-development-workflow`); whole-corpus is a standing audit. Both are **complexity-only**
— a correctness bug found here is routed to the normal review, not fixed here. The
mandatory FR-9 check is never a deletion target.

### 4.5 The disclosed-simplification debt ledger (FR-6)

A deliberate cut is marked in-place with its ceiling and upgrade trigger. Harvesting all
such markers yields a ledger:

```text
[REFERENCE]
<site>, <what was simplified>. ceiling: <named limit>. upgrade: <trigger to revisit>.
```

A marker naming no trigger is tagged `no-trigger` — the ones that silently rot. The
ledger composes `l1-operational-ledger` (it is ledger entries, not a new store) and feeds
`l2-self-improvement` (a rotting deferral is a signal). This is the *only* honest
per-project frugality figure (FR-10) — a counted ledger, never an invented savings number.

### 4.6 nodus projection

The discipline projects onto the nodus workflow layer in two concrete ways, needing **no
new language primitive**:

1. **Reuse rung = macro reuse.** FR-2's rung 2 (reuse-in-codebase) maps onto nodus's
   existing `RUN(@macro)` invocation — a workflow that re-expresses steps an existing
   macro already performs should invoke the macro, mirroring MI-13 experience reuse at
   the language level rather than the memory level.
2. **Frugality lint family.** The invariants realize as **lint rules over authored
   workflows** in the runtime's existing validator/lint stage: flag a workflow step that
   re-implements an existing macro (FR-2), a single-use parameterization nobody sets
   (FR-7 *yagni*), a hand-rolled sequence a native step covers (FR-2 rung 4). Authored
   workflows over-build exactly as code does; the same ladder applies to them.

The gating and level policy are host-side (a StorageProvider/PolicyProvider concern),
consistent with how other disciplines map to nodus — the language contributes the reuse
primitive and the lint hooks, not the judgement of when to be frugal.

## 5. Implementation Notes

1. The reuse rung (FR-2 rung 2) is only as good as the corpus lookup behind it — wire it
   to `l1-code-intelligence`, not to a from-scratch grep, so "already here?" is answered
   reliably.
2. Store the FR-6 marker convention as first-class (a recognizable in-code marker naming
   ceiling + trigger) so the debt harvest is a mechanical scan, not a judgement call.
3. Keep the frugality review (FR-7) a separate role/pass from correctness review so the
   two lenses do not blur; the delete-list is advisory output, never an auto-applied edit.
4. Intensity (FR-5) lives in office/session policy the human authors; the agent reads it,
   never writes it.

## 6. Drawbacks & Alternatives

- **Under-building risk.** An over-eager frugality could cut something that was actually
  needed. Mitigated by the negligence floor (FR-3), by comprehension-first (FR-1), and by
  disclosed-with-trigger deferral (FR-6) rather than silent omission.
- **Reasoning-token cost on some models.** A model that spends thinking tokens
  deliberating the rungs can cost more, not less — the cost/latency benefit is
  generator-dependent (FR-10) and claimed only where measured, never assumed.
- **Alternative — rely on the quality gate alone.** Rejected: quality gates pass
  correct-but-bloated code unchanged; frugality is the orthogonal missing lens (§2).
- **Alternative — a blanket "write less / fewest tokens" instruction.** Rejected: a naive
  terseness rule cuts validation and error handling too (it has no floor), and optimizes
  token count rather than necessity. The floor (FR-3) and the reuse ladder (FR-2) are
  exactly what a blanket rule lacks.
- **Alternative — fold into harness-composition.** Rejected: composition right-sizes the
  agent's tooling; this right-sizes the produced code. Same spirit, different subject;
  merging them would blur two distinct disciplines.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[COMPOSITION]` | `.design/main/specifications/l1-harness-composition.md` | The sibling right-sizing discipline (harness, not deliverable). |
| `[QUALITY]` | `.design/main/specifications/l1-quality-standards.md` | The orthogonal correctness gate frugality composes with, never overrides. |
| `[CODE-INTEL]` | `.design/main/specifications/l1-code-intelligence.md` | The corpus lookup the reuse rung (FR-2) depends on. |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | Where the disclosed-simplification debt (FR-6) is recorded. |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the discipline projects onto (§4.6). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-11 | Core Team | Initial spec — solution frugality as the output-side anti-over-engineering discipline (sibling to harness-composition's tooling-side right-sizing): comprehension precedes frugality (FR-1); the reuse-before-build ladder — YAGNI / reuse-in-codebase / stdlib / native / installed-dep / one-expression / minimum, first-hold-wins (FR-2); the hard negligence floor — never cut validation/data-loss-handling/security/accessibility/requested-scope/calibration (FR-3); root-cause locality (FR-4); tunable non-self-elevated intensity off/advisory/enforced/aggressive (FR-5, composes SEC-10); disclosed simplification with an upgrade trigger, harvestable debt ledger, no-trigger rot flag (FR-6, composes operational-ledger + self-improvement); complexity-only scoped non-mutating frugality review/audit with a closed finding vocabulary (FR-7); output economy — unrequested justification prose is complexity, requested reports exempt (FR-8); proportional verification — one runnable check for non-trivial logic, composes quality-standards' mandatory-test gate (FR-9); measured-not-asserted with an honest counterfactual — never fabricate a per-instance savings number (FR-10). Projects onto nodus as macro-reuse + a frugality-lint family with no new language primitive (§4.6). Composes harness-composition / quality-standards / code-intelligence / memory-intelligence (MI-13) / operational-ledger / self-improvement / security (SEC-10) / development-workflow / workflow-language. Derived from studied prior art on agentic code minimalism. |

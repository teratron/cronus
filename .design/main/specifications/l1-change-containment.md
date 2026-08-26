# Change Containment

**Version:** 1.2.0
**Status:** Stable
**Layer:** concept

## Overview

Solution frugality asks *how much code should this solution be*. This spec asks the
separate, unanswered question: **which existing lines is a change entitled to touch at
all?**

A change can be perfectly frugal in the code it adds and still arrive as an unreviewable
diff — because on the way to the fix it reformatted a neighbouring function, rewrote a
comment whose purpose it never established, added type annotations nobody asked for,
"improved" the validation next to the bug, and deleted code that looked dead. Every one
of those edits is defensible in isolation; together they are the single most common
reason a small correct change becomes an expensive one. The reviewer loses the two lines
that matter inside two hundred that do not, the history stops answering *why did this
line change*, and a regression rides in on a hunk nobody read.

**Change containment** is the edit-footprint discipline: every changed line carries a
**warrant** tracing to the stated goal, unrelated observations become **findings rather
than edits**, mechanical transformations are **separated** from semantic ones, and
nothing whose purpose has not been established is removed or rewritten. It is the
diff-side counterpart of `l1-solution-frugality` — frugality bounds the *size of the
solution*, containment bounds the *footprint of the change* — and the two are
independent: a bloated contained change and a lean sprawling one are both real failures.

## Related Specifications

- [l1-solution-frugality.md](l1-solution-frugality.md) — the sibling discipline on the **other axis**: frugality right-sizes the solution (how much code should exist), containment right-sizes the edit (which existing lines may be touched). FR-1's comprehension-before-frugality is extended here to *removal* (CTN-2); FR-7's non-mutating findings are the model CTN-8 follows; FR-4 root-cause locality is the warranted expansion CTN-6 permits.
- [l1-development-workflow.md](l1-development-workflow.md) — DW-4's "nothing extra" spec-compliance verdict is exactly the containment verdict; the CTN-8 review is a distinct lens at the Review stage, beside the correctness and complexity lenses.
- [l1-review-checkpoint.md](l1-review-checkpoint.md) — RC-7 (the reviewer sees the artifact, not a claim about it) is what makes containment matter: a noisy diff defeats an honest reviewer. Containment findings feed the request-revision arm (RC-4).
- [l1-version-control.md](l1-version-control.md) — VC-3's card-aligned commit boundary is the delivery-side expression of CTN-5: a mechanical transformation and a semantic change are different units of work and therefore different commits.
- [l1-code-intelligence.md](l1-code-intelligence.md) — the lookup that answers "what is this construct for, and who depends on it" before CTN-2 permits a removal; also what distinguishes genuinely unreferenced code from code whose only caller is dynamic.
- [l1-operational-ledger.md](l1-operational-ledger.md) — the findings channel CTN-7 routes deferred observations into; a containment finding is a ledger predicate, not a new store.
- [l1-quality-standards.md](l1-quality-standards.md) — **orthogonal, composed**: quality gates ask "is the change correct, tested, clean?"; containment asks "does the change touch only what it is entitled to?". Containment never licenses skipping a gate; a test the gate requires carries a warrant by construction (CTN-1b).
- [l1-intent-resolution.md](l1-intent-resolution.md) — when the goal's surface is under-specified, the change proceeds on a **recorded assumption** about its boundary (IR-3) rather than silently widening it; a corrected boundary re-plans the change (IR-7).
- [l1-pattern-codification.md](l1-pattern-codification.md) — the pathway by which an observed local convention (CTN-4) is promoted, with human ratification, into an explicit project standard that then outranks it.
- [l1-practice-analytics.md](l1-practice-analytics.md) — the CTN-9 containment signals are detector metrics of exactly this shape: computed from the record, rolled into findings, never self-asserted.
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.8): containment realizes as a lint family plus a *decidable* mechanical/semantic split, needing no new language primitive.

## 1. Motivation

Four costs, none of them stylistic.

**Review attention is the scarcest resource in the pipeline.** A reviewer's capacity to
catch a real defect degrades with the number of hunks they must classify as harmless.
Padding a two-line fix with sixty lines of reformatting does not merely waste time — it
statistically buys a missed defect.

**An unrelated hunk is an unreviewed hunk, and unreviewed hunks carry regressions.** The
drive-by "improvement" is written with less care than the requested change (it was never
the task), tested with less care (no test was asked for), and read with less care (the
reviewer is looking at the fix). It is the highest-risk code in the diff and receives the
least scrutiny.

**Removal without comprehension is the most expensive single failure mode.** Code that
looks redundant is very often load-bearing: an ordering dependency, a guard against an
input that occurs only in production, a workaround for a defect in a dependency. The
comment that looks stale is frequently the *only* surviving record of why the guard is
there. Deleting either is not simplification; it is discarding information that cost
someone a production incident to acquire.

**History legibility is a durable asset that a noisy change spends.** When a
reformatting pass and a behavior change land together, every later "why is this line the
way it is" query returns the reformat. The cost is not paid at review time; it is paid
by everyone who investigates that region for the rest of the project's life.

The failure this spec addresses is not incompetence — it is a *helpful* actor with broad
capability and no boundary. Given the ability to improve anything it reads, an
undisciplined generator improves everything it reads. The discipline supplies the missing
boundary without suppressing the value: what is not edited is **reported**, never
silently dropped.

## 2. Constraints & Assumptions

- Containment presumes a **stated goal**: the request, plus whatever the mandatory gates
  require of it. Where no goal is stated, containment has no warrant test and does not
  apply (an open-ended "clean up this module" *is* the goal, and its surface is the
  module).
- Containment is **not** a rule against large changes. A large change with a large
  warranted surface is contained; a one-line change with an unwarranted reformat is not.
  Size is not the measured quantity.
- Containment is **not** conservatism about correctness. Reaching the root cause,
  updating every caller of a changed contract, and fixing sibling paths that share a
  defect are all in-scope by warrant (CTN-1b / CTN-6). Shipping a knowingly partial fix
  to keep a diff small is a defect, not containment.
- The discipline binds the **actor producing the change**, whichever it is. Nothing here
  is agent-specific; a human contributor is held to the same warrant test.
- Findings routing assumes a durable channel exists. Where none does, containment
  degrades to reporting in the change's own delivery note — it never degrades to silence.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **CTN-1 Every changed line carries a warrant**: a hunk is admissible only if it traces
  to exactly one of three warrants — **(a) realization**: it implements the stated goal;
  **(b) necessity**: the goal cannot be correct or pass its mandatory gates without it
  (the root-cause site, a caller updated for a changed contract, the test the quality gate
  requires, a sibling path sharing the same defect); **(c) self-repair**: it fixes
  something this change's own edits stranded. There is no fourth warrant. "It was better
  this way", "I was already in the file", and "the linter suggested it" are not warrants.
  A hunk that cannot name its warrant is out of scope by definition, not by judgement.

- **CTN-2 Comprehension gate on removal and rewrite**: code, a comment, a branch, a
  configuration value, or a test whose **purpose has not been established** MUST NOT be
  deleted, rewritten, condensed, or "simplified" — in any change, at any size. Where the
  purpose cannot be established within the change's budget, the construct is left exactly
  as found and recorded as an open question (CTN-7). Apparent redundancy is evidence of
  *missing comprehension*, not evidence of dead code: the most expensive removals are the
  ones that looked obviously safe. This extends FR-1 (comprehension precedes frugality)
  to the removal side, which the frugality ladder does not reach.

- **CTN-3 Foreign dead code is reported, never removed in the same change**: pre-existing
  unused or superseded code encountered while doing something else is a **finding**, not
  an edit. Orphans **this change itself created** — an import it made unused, a helper its
  rewrite left uncalled, a fixture its deletion abandoned — are cleaned up *in this same
  change* and are never findings; leaving them is the change's own mess. The dividing line
  is **authorship of the orphaning**, not the current state of the code.

- **CTN-4 Local idiom outranks personal preference; an explicit standard outranks both**:
  changed and added code adopts the conventions actually in force at the site — naming,
  formatting, error-handling shape, comment density, test idiom. Resolution order is
  strict: **declared project standard > local convention at the site > anything else**. A
  divergence between the local idiom and the project standard is real work and is
  recorded as such (CTN-7), but reconciling it is **its own change**; it is never folded
  into an unrelated one. Style is not a matter on which a change may express an opinion in
  passing.

- **CTN-5 Mechanical transformation is never bundled with semantic change**: a reformat, a
  rename, a file move, an import reorder, a mass annotation pass, an automated migration
  — each is a separable unit with a purely mechanical warrant, and each is delivered
  **separately** from any change that alters behavior. Bundling is what makes a diff
  unreviewable, and it is the direct cause of the history-legibility loss in §1. Where a
  tool cannot avoid emitting both, the semantic hunks are enumerated explicitly so the
  reviewer can find them without reading the mechanical ones.

- **CTN-6 Expansion is declared and warranted, never taken silently and never refused
  when correctness needs it**: when the correct change reaches beyond the surface the
  request names — the real cause is elsewhere (FR-4), a contract change ripples to its
  callers, sibling paths share the defect — the change **does** expand, and the expansion
  is **stated with its warrant** as part of delivery. Both failure directions are
  defects: silently widening the surface hides scope from the reviewer, and narrowing the
  fix to the named symptom to keep the diff small ships a change that is knowingly wrong.
  Containment governs *disclosure and warrant*, never *timidity*.

- **CTN-7 Declining to edit is never declining to report**: every observation containment
  keeps out of the change — foreign dead code, an idiom divergence, an over-engineered
  neighbour, a construct nobody could explain, a suspected defect out of scope — is
  recorded as an **actionable finding** carrying location, observation, and suggested
  action, routed to the standing findings channel. The discipline **relocates** the work;
  it MUST NOT be usable as a licence for silence. A containment that quietly drops what it
  saw is strictly worse than the drive-by edit it replaced, because the knowledge was
  acquired and then thrown away.

- **CTN-8 The containment review is a distinct, non-mutating pass with a closed
  vocabulary**: reviewing a change for containment is a separate lens from the
  correctness, security, and complexity lenses, and emits findings drawn from a closed set
  — *unwarranted-hunk*, *style-drift*, *foreign-cleanup*, *bundled-mechanical*,
  *comprehension-gap*, *undeclared-expansion*. Each names the location, the warrant it
  fails, and one disposition (revert-the-hunk / split-it-out / declare-the-expansion). The
  pass **reports**; it does not rewrite the change. It MUST NOT flag a hunk carrying a
  valid (b) or (c) warrant as creep — a required test and a root-cause repair are contained
  by construction. "Fully contained" is a valid and complete result.

- **CTN-9 Containment is measured on the diff, never asserted**: the discipline's effect
  is computed from the change itself against its stated goal — share of hunks with no
  named warrant, count of files touched outside the goal's surface, ratio of mechanical to
  semantic changed lines, count of removals with no established purpose, count of findings
  filed versus edits deferred. These are comparable across time and across actors, so
  "this office produces clean diffs" is falsifiable or is not claimed. No per-change
  "noise avoided" figure is fabricated: the sprawling version was never written, so there
  is no baseline to subtract from (the same honesty rule as FR-10).

- **CTN-10 The rules are uniform; the ceremony is proportional**: CTN-1…CTN-8 hold at
  every change size — a one-line fix may not smuggle a reformat, and a trivial task is
  not an exemption from the comprehension gate. What scales with size and blast radius is
  the **ceremony**: a small contained change needs no declared expansion, no separate
  review pass, and no findings write; a large or high-impact one gets all three. Turning
  the discipline into ritual on every typo is itself a cost of the kind the discipline
  exists to remove.

- **CTN-11 A declared edit boundary is enforced by mechanism, blocks rather than warns, and is never self-relaxed**: [ADDED v1.1.0] CTN-1…CTN-10 are a discipline the producer applies to itself; for work where the cost of straying is high — a live system, a narrow debugging session, a module under active parallel work — the human principal MAY declare an **edit boundary**: the paths this work may write to. Four properties make it real rather than advisory. It is enforced **at the write gate by mechanism**, not by instructing the producer to stay inside it (an instruction is the enforcement that already failed). It **blocks**, it does not warn: a warned-past boundary is not a boundary, and a producer that can proceed after a warning will. It is **declared by the human and never widened by the producer** — narrowing is always available, widening is a principal decision (composing the authority-not-self-authored and governable-stricter-never-laxer rules), so a boundary cannot be dissolved by the actor it constrains. And it is **scoped and visible** — bound to a session or a task rather than standing forever, and announced while active, since a silently-enforced boundary produces failures the producer cannot explain. The boundary is a *floor* under containment, never a substitute for it: staying inside a declared path does not make an unwarranted hunk warranted (CTN-1 still applies within it), and its absence does not relax anything.

- **CTN-12 A wide mechanical change is sequenced expand → migrate → contract, never forced into one contained hunk**: [ADDED v1.2.0] CTN-1…CTN-11 assume a change whose blast radius fits a reviewable unit. One class defeats that by construction: a **single mechanical transformation** — renaming a shared symbol, retyping a shared value, relocating a shared boundary — whose call sites number in the hundreds, so every correct version of it is enormous and every small version of it is broken. Cutting it into "contained" pieces yields pieces that individually do not build, which is worse than the large diff rather than better. The sanctioned shape is **expand → migrate → contract**: introduce the new form **beside** the old so nothing breaks; migrate call sites in **batches sized by blast radius** (per package, per directory), each batch its own unit and each leaving the system green because the old form still exists; then **remove the old form** once no caller remains, in a final unit gated on every migration batch. Two rules keep the exception from becoming a loophole. It is **declared, not assumed** — the change must actually be one mechanical transformation, because "this is a wide refactor" is the most available excuse for an unwarranted hunk (CTN-1), and containment applies unchanged *within* each batch. And where even a batch cannot stand green alone, the batches share an integration line gated by a single **verify** unit: green is then promised **at that unit, explicitly**, rather than silently promised nowhere.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The warrant test (CTN-1)

Containment is decided per hunk, mechanically, against the stated goal:

```text
[REFERENCE]
classify(hunk, goal):
    if realizes(hunk, goal):                      return in_scope        // (a)
    if required_for_correctness(hunk, goal):      return in_scope        // (b) root cause,
                                                                         //     caller, gate-test,
                                                                         //     sibling path
    if repairs_orphan_created_by(this_change):    return in_scope        // (c)
    if outside_goal_surface(hunk):                return expansion       // → CTN-6: declare or drop
    return unwarranted                                                   // → revert; file as finding
```

The test is deliberately *not* "is this hunk an improvement?" — almost every drive-by
edit is one. It is "does this hunk exist because of the goal?". An improvement that would
have been made anyway, on a different day, in a different change, belongs to that change.

### 4.2 The four dispositions

Every observation made during a change lands in exactly one of four buckets:

| Disposition | What it covers | What happens |
| --- | --- | --- |
| **In-scope** | warrants (a), (b), (c) | edited in this change |
| **Warranted expansion** | correctness needs a surface the request did not name | edited **and declared** (CTN-6) |
| **Deferred finding** | foreign dead code, idiom divergence, neighbouring bloat, suspected out-of-scope defect | **not** edited; filed (CTN-7) |
| **Forbidden** | rewriting or removing what was not understood | **not** edited, **not** deferred as a deletion candidate — recorded as an open question (CTN-2) |

The fourth row is the one that distinguishes this discipline from a scope policy. The
others say *not now*; this one says *not until someone understands it*, and it applies
even when the change is explicitly about cleanup.

### 4.3 The comprehension gate (CTN-2)

```text
[REFERENCE]
may_remove(construct):
    p := establish_purpose(construct)     // corpus lookup: references (incl. dynamic/reflective),
                                          // history, tests exercising it, the comment's claim
    if p is unknown:      return false    // leave as-is; record an open question
    if p is still served: return false    // it is not dead; it is load-bearing
    if removal_requested_or_warranted:  return true
    return false                          // understood-and-dead, but not this change's business (CTN-3)
```

Two properties matter. First, "no references found" is a *lookup result*, not a purpose —
dynamic dispatch, reflective access, configuration-driven wiring, and external callers all
produce zero static references for live code. Second, the gate applies to **comments**
identically: a comment contradicting the code is a finding to raise, not a line to
silently correct, because either the comment or the code is wrong and the change does not
yet know which.

### 4.4 Idiom resolution (CTN-4)

```text
[REFERENCE]
convention_for(site):
    if declared_standard covers(site):  return declared_standard   // explicit wins
    if local_pattern is discernible:    return local_pattern       // the site's own idiom
    return house_default                                           // fall back, do not invent
```

The rule's force is in what it forbids: an actor with a strong style preference
propagating that preference through unrelated code, one visit at a time. Where the local
idiom genuinely conflicts with the declared standard, the conflict is a finding; if it
recurs and holds, it is a candidate for codification into an explicit standard through the
ratified pathway — never resolved unilaterally in the middle of a feature change.

### 4.5 The containment review (CTN-8)

```text
[REFERENCE]
containment_review(change, goal):
    findings := []
    for hunk in change:
        c := classify(hunk, goal)                                   // §4.1
        if c == unwarranted:      findings += (unwarranted_hunk, hunk, revert)
        if c == expansion and not declared(hunk):
                                  findings += (undeclared_expansion, hunk, declare)
        if style_only(hunk) and not mechanical_change(change):
                                  findings += (style_drift, hunk, revert)
        if removes_foreign_dead(hunk):
                                  findings += (foreign_cleanup, hunk, split_out)
        if removes_or_rewrites_unestablished(hunk):
                                  findings += (comprehension_gap, hunk, revert)
    if mixes_mechanical_and_semantic(change):
                                  findings += (bundled_mechanical, change, split_out)
    return findings                                                 // never applies them
```

It runs beside — never instead of — the correctness and complexity lenses. A correctness
defect noticed here is routed to the correctness review, exactly as a frugality review
routes one. The pass is cheap: it reads the diff and the goal, not the whole corpus.

### 4.6 Boundary with frugality

The two disciplines are independent axes, and the four combinations are all real:

| | **Contained** | **Sprawling** |
| --- | --- | --- |
| **Lean** | the target | 50 good lines plus 200 lines of drive-by |
| **Bloated** | 400 lines where 40 would do, all of them warranted | the worst case |

Three specific seams:

1. **FR-7 emits *delete* findings; CTN-3 decides who may act on them.** The frugality
   review is already non-mutating. Containment supplies the missing rule for the
   consumer: a delete-finding against code the current change did not orphan is scheduled
   as its own change, never folded into the one in flight.
2. **FR-4 root-cause locality is a CTN-1(b) warrant, not creep.** Fixing the shared
   upstream site rather than the reported symptom expands the surface *and* shrinks the
   aggregate diff. CTN-6 requires it be declared; CTN-8 must not flag it.
3. **FR-1 and CTN-2 are the same instinct on two sides.** Frugality forbids a short
   solution written without understanding; containment forbids a deletion made without
   understanding. Both say: the discipline shortens the output, never the reading.

### 4.7 Measurement (CTN-9)

```text
[REFERENCE]
containment_signals(change, goal):
    unwarranted_hunk_share  := |hunks with no warrant| / |hunks|
    off_surface_files       := |files touched outside goal surface, undeclared|
    mechanical_line_ratio   := mechanical_lines / (mechanical_lines + semantic_lines)
    blind_removals          := |removals with no established purpose|
    deferral_discipline     := findings_filed / observations_deferred   // CTN-7 honesty check
```

The last signal is the one that keeps the discipline honest in the other direction: an
actor whose unwarranted-hunk share is zero *and* whose deferral discipline is zero is not
contained — it is blind, or it is silent. Both are read as findings, per the honest
data-gap and minimum-sample rules the analytics layer already enforces.

### 4.8 nodus projection

The discipline projects onto the nodus workflow layer with **no new language primitive**,
in three concrete ways:

1. **A decidable mechanical/semantic split (CTN-5).** nodus's dual-representation
   invariant guarantees that compact and human forms are semantically equivalent and that
   a round-trip is AST-equal. That gives the containment review something no
   general-purpose language offers: for an authored workflow, "is this edit purely
   mechanical?" is **decidable** — an edit whose compact-form AST is unchanged is
   mechanical by construction, and any other edit is semantic. `bundled_mechanical` stops
   being a heuristic and becomes a check.
2. **Containment lints over authored workflows.** Authored workflows accumulate drive-by
   edits exactly as code does: a step reordered for tidiness, a variable renamed across an
   unrelated branch, a `!PREF` block reflowed while fixing a different step. These realize
   as lint rules in the runtime's existing validator stage — flag changed steps outside
   the declared goal surface, flag a rename that touches steps unrelated to the change.
3. **Declared effect surfaces as mechanical containment.** Where a host already declares a
   step's effect classes and capability manifest, the *entitled* surface of a run is
   already a machine-checkable object. Containment reuses it rather than restating it: a
   step touching an artifact outside its declared effect surface is refused **by
   mechanism**, not by instructing the model — the structural-enforcement pattern this
   project prefers wherever a rule can be made unrepresentable instead of merely stated.

As with the frugality projection, the judgement half stays host-side: the language
contributes the AST-equality oracle, the lint hooks, and the effect-surface declaration;
what counts as the goal surface is a host policy concern.

## 5. Implementation Notes

1. The comprehension gate (CTN-2) is only as good as the lookup behind it — wire it to the
   code-intelligence surface, not to a from-scratch text search, so "who uses this" accounts
   for dynamic and configuration-driven references rather than reporting a false zero.
2. The goal surface (§4.1) should be **recorded at the start of the change**, not
   reconstructed at review time. A surface inferred after the fact rationalizes whatever
   the change happened to touch, which defeats the test.
3. Findings (CTN-7) are ledger predicates, not a new store; one observation per entry, so
   they stay greppable and can be picked up as their own work items.
4. The review (CTN-8) belongs in the Review stage next to the frugality lens; running both
   in one pass is fine, mixing their finding vocabularies is not.
5. Delivery-side separation (CTN-5) lands naturally on the card-aligned commit boundary —
   a mechanical transformation is a different card, therefore a different commit.

## 6. Drawbacks & Alternatives

- **The discipline can be used to justify a knowingly partial fix.** Bounded by CTN-6's
  second half: narrowing the change below what correctness requires is explicitly a defect,
  and CTN-1(b) makes the root-cause site in-scope by warrant. Containment never means
  *less correct*.
- **Findings can accumulate faster than anyone acts on them.** Real, and accepted: an
  unactioned finding is strictly better than an unreviewed edit, and the backlog is
  visible rather than smeared through diffs. The backlog's own health is a self-improvement
  signal, not a reason to resume drive-by editing.
- **The comprehension gate slows down genuine cleanup.** Intended. Cleanup is a legitimate
  goal with its own surface — where cleanup *is* the request, removal is warrant (a) and
  the gate only requires that the purpose was established, which any responsible cleanup
  does anyway.
- **Alternative — a size cap on diffs.** Rejected: it measures the wrong quantity. A
  large warranted change would be blocked and a small unwarranted one waved through, and
  actors would learn to split by size rather than by warrant.
- **Alternative — forbid touching any file the request does not name.** Rejected by
  CTN-1(b)/CTN-6: it forbids root-cause repair and caller updates, which is how symptom
  patching becomes policy.
- **Alternative — fold it into solution frugality.** Rejected: frugality bounds the size of
  the solution and has no notion of a warrant, an idiom, or a foreign-versus-self-created
  orphan. The four-cell table in §4.6 is the argument — the two axes fail independently.
- **Alternative — leave it to the reviewer.** Rejected: it spends the exact resource (§1)
  the discipline exists to protect, and it detects the failure after the cost is paid.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[FRUGALITY]` | `.design/main/specifications/l1-solution-frugality.md` | The sibling axis (solution size); source of FR-1/FR-4/FR-7/FR-10 seams. |
| `[WORKFLOW]` | `.design/main/specifications/l1-development-workflow.md` | Where the containment lens runs (Review stage) and DW-4's "nothing extra" verdict. |
| `[CODE-INTEL]` | `.design/main/specifications/l1-code-intelligence.md` | The lookup the comprehension gate (CTN-2) depends on. |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | Where deferred findings (CTN-7) are recorded. |
| `[VCS]` | `.design/main/specifications/l1-version-control.md` | Commit-boundary realization of the mechanical/semantic split (CTN-5). |
| `[ANALYTICS]` | `.design/main/specifications/l1-practice-analytics.md` | The detector/finding shape the CTN-9 signals adopt. |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the discipline projects onto (§4.8). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.1.0 | 2026-08-05 | Core Team | Added CTN-11 (a declared edit boundary is enforced by mechanism, blocks rather than warns, and is never self-relaxed) — the mechanical floor under a discipline that was until now entirely self-applied: for high-cost work the human principal declares which paths the work may write to, enforced **at the write gate** rather than by instructing the producer (instruction being the enforcement that already failed); **blocking, not warning**, since a producer that can proceed past a warning will; **narrowable by the producer, widenable only by the principal** (composing authority-not-self-authored and governable-stricter-never-laxer), so the constrained actor cannot dissolve its own constraint; and **scoped and announced**, since a silently-enforced boundary produces failures the producer cannot explain. Explicitly a floor, never a substitute: staying inside the path does not make an unwarranted hunk warranted, and the boundary's absence relaxes nothing. |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — change containment as the edit-footprint discipline, the diff-side counterpart to solution frugality (which bounds solution size and has no notion of an edit's entitlement): every changed line carries one of exactly three warrants — realization, correctness-necessity, self-repair — with no fourth (CTN-1); a comprehension gate forbidding removal or rewrite of any construct, comment included, whose purpose was never established, since apparent redundancy is evidence of missing comprehension rather than of dead code (CTN-2); foreign dead code reported never removed in the same change, with the dividing line being authorship of the orphaning rather than the code's current state (CTN-3); local idiom over personal preference and an explicit standard over both, a divergence being its own change (CTN-4); mechanical transformation never bundled with semantic change, the direct cause of unreviewable diffs and lost history legibility (CTN-5); expansion declared and warranted, with silent widening and correctness-narrowing both defects (CTN-6); declining to edit never a licence for silence — every deferred observation filed as an actionable finding (CTN-7); a distinct non-mutating containment review with a closed finding vocabulary that must not flag warranted root-cause or gate-test hunks (CTN-8); measured on the diff with no fabricated per-change savings figure (CTN-9); uniform rules with proportional ceremony (CTN-10). nodus projection needs no new primitive: the dual-representation AST-equality guarantee makes the mechanical/semantic split *decidable* for authored workflows, plus a containment lint family and reuse of declared effect surfaces as mechanical enforcement. Concept-only. |
| 1.2.0 | 2026-08-26 | Core Team | Amended — CTN-12: the **wide mechanical change** exception. A single mechanical transformation whose call sites number in the hundreds cannot be made both correct and contained, and splitting it into "contained" pieces produces pieces that do not build. Sanctioned shape is **expand → migrate → contract** — new form beside the old, call sites migrated in blast-radius-sized batches each leaving the system green, old form removed in a final unit gated on every batch. The exception is **declared, not assumed** (it is the most available excuse for an unwarranted hunk, so CTN-1 applies unchanged within each batch), and where a batch cannot stand green alone the batches share an integration line gated by one explicit **verify** unit, so green is promised somewhere rather than nowhere. Distilled from an adoption pass over an external engineering-skills reference. |

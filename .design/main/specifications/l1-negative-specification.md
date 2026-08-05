# Negative Specification

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Every guidance surface in the office states what to **aim for**: a rubric, a gold-standard
exemplar, a token contract, a stated requirement, a quality gate. Almost none of them
states, as a first-class artifact, what to **stay away from**.

That gap is not cosmetic, because the two are not derivable from each other. A positive
target says where to go; an output can satisfy it completely and still land squarely on the
one thing the principal specifically did not want. "Make it look like this" does not say
"and never like that", and a checker that only knows the positive will pass the output
every time.

The shape already exists in the system, invented locally in four places and named nowhere:
a repeatedly-corrected choice promoted to a recorded anti-pattern; a fixed list of
generic-default tells blocked at the craft bar's must-fix tier; a forbidden construct
guarded by a source-level tripwire; a banned phrasing in generated prose. Each was built
for its own corner.

**Negative specification** is the shared concept: a declared, provenanced, bounded
statement of what an output must **not** be. Three rules make one work rather than
decorate a document — it reaches the **generator** before production rather than the
reviewer after it; it names the **alternative** wherever one exists; and it never overrides
the principal's own explicit request, because an exclusion is not a veto over its author.

## Related Specifications

- [l1-specialty-exemplars.md](l1-specialty-exemplars.md) — the **positive** counterpart and the clearest statement of NEG-2: SE-2 grades a candidate by distance from a gold-standard exemplar (SE-5's compare-to-exemplar, never compare-to-open-ideal). Distance from a good example is silent about proximity to a bad one; the two instruments answer different questions and neither substitutes.
- [l1-design-identity.md](l1-design-identity.md) — the worked instance: DI-6's blocked *default tells* are the built-in negative layer of a craft bar, and DI-5's named auto-vs-advisory boundary is the discipline NEG-9 generalizes. A project-declared exclusion is the **user layer** DI-7's layered rule catalog already anticipates.
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — the **mechanical enforcement** of a source-level negative: TW turns "this shape must never appear" into a running check. NEG is the specification form; TW is one of its enforcement mechanisms, and NEG-4 adds the half TW does not cover — reaching the generator *before* the shape is written.
- [l1-intent-resolution.md](l1-intent-resolution.md) — IR-7's *a repeatedly-corrected assumption becomes a recorded anti-pattern* is the **derived** origin class (NEG-3); this spec gives that anti-pattern a form, a lifecycle, and a place in the generation context.
- [l1-pattern-codification.md](l1-pattern-codification.md) — the ratification pathway a derived exclusion travels to gain binding authority (PC-1/PC-2), and PC-5's re-validation is what NEG-6's dating serves.
- [l1-content-language.md](l1-content-language.md) — a language policy is a positive specification with an obvious negative twin (a banned register, a phrasing to avoid); CL-6's structural-vs-checked tiering and NEG-9 are the same discipline applied to two different constraints.
- [l1-user-model.md](l1-user-model.md) — a principal-declared exclusion is a stated preference with UM-4 precedence; taste expressed as avoidance is still taste, and it accretes the same way.
- [l1-cache-stable-context.md](l1-cache-stable-context.md) — the exclusion set is stable across requests and therefore belongs in the cacheable prefix, not appended per call (§4.5).
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.5): the validated configuration surface carries the set, the validation stage hosts the checkable subset, and prompt composition places it in the stable prefix.

## 1. Motivation

**Passing every positive check is not the same as being acceptable.** The output that
satisfies the rubric and still reaches for the stack default, the summary that meets every
stated requirement in the register the client asked never to hear again, the fix that is
correct and written in the pattern the team retired last year — each is a pass by the
system's own instruments and a failure by the principal's.

**A correction that is not written down is a correction that will be needed again.** The
client says "not like that" once, the producer adjusts, and the knowledge lives exactly as
long as the conversation. Next session, next artifact, next agent: the same output, the
same correction, and the principal learns that saying it does not help.

**Review-time enforcement pays for the work twice.** A negative discovered after
production means the artifact exists, the cost was spent, and the remedy is a rewrite. The
same exclusion placed in front of the generator costs a few tokens and prevents the
artifact from being wrong in the first place. This is the single highest-leverage property
of the whole concept, and it is the one most often skipped, because writing a checker is
easier than routing a constraint into a prompt.

**A prohibition without an alternative pushes the generator into the neighbouring
failure.** Told only "not this", a generator moves to whatever is closest — which is
usually the second-most-default option. "Not this, use that instead" is a different
instruction with a different outcome.

**An unbounded avoid-list is self-cancelling.** Exclusions accumulate, nobody removes them,
and eventually the constraint set is large enough that no output can satisfy it. What
follows is not careful work; it is a generator that has learned the list is not really
binding.

## 2. Constraints & Assumptions

- A negative specification constrains **output**; it is not a permission control. What an
  actor may *do* is the authority layer's question — this governs what a produced artifact
  may *be*.
- Exclusions are **local-first project and user data**, carrying no authority of their own
  and never a source of truth about behaviour.
- Some negatives are mechanically detectable and some are matters of taste. The concept
  covers both and requires the boundary to be stated (NEG-9); it does not require every
  exclusion to be checkable.
- This spec defines no new checker, no new store, and no new rule engine. Exclusions live
  where the corresponding positive rules already live; the checkable subset runs in the
  gates that already run.
- The set is assumed small. Every property here degrades if it is not, which is why
  boundedness is an invariant rather than advice.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **NEG-1 A negative is a first-class artifact with identity and provenance**: an exclusion
  is recorded as its own item — **what** is excluded, **why**, **who or what established
  it**, and **when it was last affirmed** — not as a footnote on a positive rule, a line in
  a review comment, or a sentence in a conversation. Without an identity it cannot be
  cited, honoured, contested, or retired, and it degrades into folklore within one session.

- **NEG-2 Positive and negative are independent; neither implies the other**: satisfying
  every positive criterion does **not** discharge the negatives, and a negative cannot be
  inferred from a positive. A system that models only the target will produce outputs that
  pass all its checks and are precisely what the principal asked to avoid. Both instruments
  are required, and a claim of conformance names which of the two it was measured against.

- **NEG-3 Three declared origins, with different authority on conflict**: every exclusion
  declares its origin — **principal-declared** (the client stated it; the office's own
  judgement never overrides it), **derived** (a repeatedly-corrected choice promoted to an
  anti-pattern; advisory until it passes the ratification pathway), or **built-in** (a
  known-bad default shipped with the system; the weakest, always overridable per project).
  Origin is recorded because it is what decides the outcome when two exclusions, or an
  exclusion and a request, collide.

- **NEG-4 A negative reaches the generator, not only the checker**: an exclusion is placed
  in the **production context before the artifact is made**. Enforcement at review is the
  **second** line, never the only one: a negative discovered after production means the work
  exists, the cost is spent, and the remedy is a rewrite. An exclusion that is only ever
  checked is a complaint with a schedule.

- **NEG-5 A negative names the alternative wherever one exists**: the actionable form is
  *not this — that instead*. A bare prohibition leaves the generator to move to whatever is
  nearest, which is reliably the next-most-default option. An exclusion for which no
  alternative can be named is legitimate and is **marked as such**, so its weaker guidance
  value is visible rather than assumed.

- **NEG-6 Bounded, dated, and revisited**: the exclusion set is **finite**, each entry
  carries the date it was last affirmed, and stale entries are re-affirmed or retired. An
  unbounded avoid-list becomes a constraint no output can satisfy, after which it is
  ignored wholesale — and a stale exclusion silently forbids something the principal now
  wants. The set's size is itself a health signal.

- **NEG-7 An explicit request outranks an exclusion, and the conflict is surfaced once**:
  when the principal asks for exactly what an exclusion forbids, **the request wins** — an
  exclusion is not a veto over its own author. The conflict is stated once, plainly ("this
  is on the avoid-list; applying it because you asked"), never silently resolved in either
  direction, and the exclusion is offered for update rather than quietly abandoned or
  quietly enforced.

- **NEG-8 A violation reports the matched signal and its origin**: when a negative fires,
  the finding names **what matched**, **where**, and **which exclusion with which origin**
  it came from. "This looks generic" is not a finding — it is an opinion wearing a finding's
  clothes, and it cannot be acted on, contested, or measured.

- **NEG-9 The checkable subset is checked; the remainder is named advisory**: exclusions
  expressible as a detectable signal are enforced mechanically; the taste-level remainder is
  guidance, and **the boundary between the two is stated, never blurred**. An advisory
  exclusion presented as if mechanically guaranteed is the same dishonesty the craft bar and
  the completion discipline already forbid elsewhere.

- **NEG-10 Firing rate is measured, and both extremes are informative**: how often each
  exclusion fires is counted. One that **never** fires is either fully internalized or dead
  weight, and only the record distinguishes them; one that fires **constantly** means the
  **positive** specification is missing something the negative is compensating for — the
  fix belongs on the positive side, not in a stricter prohibition.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The exclusion record

```text
[REFERENCE]
Exclusion {
  id            : stable identity                       // NEG-1
  excludes      : the thing to avoid (signal or description)
  instead       : the named alternative | NONE(marked)   // NEG-5
  rationale     : why
  origin        : principal | derived | built-in         // NEG-3
  checkable     : true | false                           // NEG-9
  affirmed_at   : date                                   // NEG-6
  fired_count   : n                                      // NEG-10
}
```

The `instead` field being explicitly `NONE(marked)` rather than empty is deliberate: an
absent alternative is a known weakness of that exclusion, and the record says so instead of
looking complete.

### 4.2 Origin and conflict resolution (NEG-3 / NEG-7)

```text
[REFERENCE]
resolve(candidate_output, exclusions, request):
    if request explicitly asks for X and exclusion(X) exists:
        surface_conflict_once(X, exclusion.origin)       // NEG-7 — the request wins
        offer_update(exclusion)
        return allow
    for e in exclusions ordered by origin_authority:      // principal > derived(ratified) > built-in
        if matches(candidate_output, e):  return violation(e)   // NEG-8
    return allow
```

Two exclusions can also collide with each other — a built-in that forbids what a
principal-declared entry requires. The ordering settles it, and the losing entry is
surfaced rather than silently skipped: a built-in that a project has effectively repealed
should be visibly repealed.

### 4.3 Placement: before, then after (NEG-4)

| Placement | Cost of a violation | Role |
| --- | --- | --- |
| **In the production context** | a few tokens | primary — the artifact is never wrong |
| **At the review gate** | a rewrite | secondary — catches what the generator missed |
| **At review only** | a rewrite, every time | the failure mode this invariant forbids |

The asymmetry is the whole argument. The second line is genuinely necessary — a generator
does not reliably honour every constraint in its context — but a system that has only the
second line has chosen to pay for every violation at full price.

### 4.4 The four existing instances

| Instance | Origin class | Checkable | Where it lives today |
| --- | --- | --- | --- |
| A corrected assumption promoted to an anti-pattern | derived | rarely | the resolution/learning path |
| Generic-default tells blocked at a craft bar | built-in | mostly | the craft rule catalog |
| A forbidden construct guarded by a source check | built-in or derived | yes | the tripwire suite |
| A banned register or phrasing in generated prose | principal or built-in | partly | the content policy |

Naming the shared shape is what lets a fifth instance be added without inventing a fifth
mechanism — and what lets a single reader answer "what is this office avoiding, and why?"

### 4.5 The exclusion set is stable context

The set changes on the scale of weeks and is identical across the requests of a session, so
it belongs in the **stable, cacheable prefix** of a model-facing composition rather than
appended per request. Placing it late — after the volatile part — pays its token cost fresh
on every call for a constraint that never changed.

### 4.6 nodus projection

No new language primitive is required:

1. **The set is a configuration surface.** The validated declarative configuration already
   supplies typed, defaulted, validated fields checked *before* any value becomes visible
   to a workflow — an exclusion set is an ordinary member of it, host-rendered and
   host-validated.
2. **The checkable subset runs in the validation stage.** The same stage that hosts
   authored-workflow tripwires hosts the mechanically-detectable exclusions; the language
   contributes the stage, the host contributes the entries.
3. **Placement is a prompt-composition concern the language already solves.** The
   cache-stable composition rule puts reusable segments in a byte-stable leading prefix, and
   the exclusion set is exactly such a segment (§4.5). One caution rides with it: an
   exclusion sourced from user or external content is **content**, and carries provenance
   and origin taint like any other value — an avoid-list is not automatically trusted
   instruction merely because it arrived as configuration.

## 5. Implementation Notes

1. Store exclusions beside the positive rules they shadow, not in a separate registry — the
   pair is read together, and a split guarantees one of them is forgotten.
2. Route the set into the production context through the same path as the positive
   guidance, so NEG-4 cannot be satisfied by accident on one surface and missed on another.
3. Count firings from the start (NEG-10); retrofitting the counter loses the baseline that
   makes "never fires" interpretable.
4. Make the affirmation date a required field at write time — an optional date is an absent
   date, and NEG-6 then has nothing to work with.

## 6. Drawbacks & Alternatives

- **Exclusion sets grow and rot.** The central risk, met by NEG-6 (bounded, dated,
  revisited) and NEG-10 (a never-firing entry is visible). The concept does not pretend the
  risk is eliminated; it makes it measurable.
- **Injecting exclusions into every generation costs context.** Real, and bounded by NEG-6's
  finiteness plus §4.5's cache-stable placement, which makes the cost near-zero on the
  repeat calls that dominate.
- **A negative can encode prejudice as policy.** Which is why origin (NEG-3) and rationale
  (NEG-1) are mandatory and why a principal's explicit request always wins (NEG-7): the
  exclusions are visible, attributable, and contestable rather than an invisible taste
  baked into the generator.
- **Alternative — express everything as positive requirements.** Rejected by NEG-2: it is
  not expressible. "Never resemble this" has no positive form that is not an enumeration of
  everything else.
- **Alternative — check negatives at review only.** Rejected by NEG-4: it is the same
  constraint at many times the price, and it trains the reviewer to be the constraint.
- **Alternative — fold into the craft bar.** Rejected: the craft bar is one domain's
  instance (§4.4). The concept spans prose register, code constructs, corrected assumptions,
  and visual defaults, and each would otherwise re-invent origin, alternative, boundedness,
  and conflict resolution.
- **Alternative — fold into the tripwire concept.** Rejected: a tripwire is an *enforcement
  mechanism* for the mechanically-checkable subset, and it acts after the shape is written.
  It has no notion of an alternative, a taste-level exclusion, or reaching the generator
  first — the three properties that carry most of this spec's value.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[EXEMPLARS]` | `.design/main/specifications/l1-specialty-exemplars.md` | The positive counterpart NEG-2 contrasts with. |
| `[IDENTITY]` | `.design/main/specifications/l1-design-identity.md` | The worked instance: default tells, tiered bar, auto-vs-advisory boundary. |
| `[TRIPWIRES]` | `.design/main/specifications/l1-invariant-tripwires.md` | The enforcement mechanism for the checkable source-level subset. |
| `[INTENT]` | `.design/main/specifications/l1-intent-resolution.md` | IR-7 — the derived origin class (corrected assumption → anti-pattern). |
| `[CODIFICATION]` | `.design/main/specifications/l1-pattern-codification.md` | The ratification pathway a derived exclusion travels. |
| `[CACHE]` | `.design/main/specifications/l1-cache-stable-context.md` | Why the set belongs in the stable prefix (§4.5). |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the concept projects onto (§4.6). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — negative specification as the named shared shape behind four locally-invented instances (a corrected assumption promoted to an anti-pattern, generic-default tells blocked at a craft bar, a forbidden construct under a source tripwire, a banned register in generated prose): an exclusion is a first-class artifact with identity, rationale, origin, and an affirmation date rather than a footnote or a conversation (NEG-1); positive and negative are independent and neither implies the other, so a system modelling only the target passes every check while producing exactly what the principal asked to avoid (NEG-2); three origins with different conflict authority — principal-declared, derived, built-in (NEG-3); the exclusion reaches the **generator before production**, with review enforcement the second line and never the only one, since a negative caught after the fact costs a rewrite while the same constraint in context costs tokens (NEG-4); the alternative is named wherever one exists, because a bare prohibition moves the generator to the next-most-default option (NEG-5); bounded, dated, revisited, since an unbounded avoid-list is unsatisfiable and then ignored wholesale (NEG-6); an explicit principal request outranks an exclusion with the conflict surfaced once, an exclusion being no veto over its author (NEG-7); a violation names the matched signal and its origin, not an opinion in a finding's clothes (NEG-8); the checkable subset checked and the taste remainder named advisory with the boundary stated (NEG-9); and firing rate measured, where never-fires and always-fires are both actionable — the latter indicting the *positive* spec (NEG-10). Nodus projection needs no new primitive — the validated config surface carries the set, the validation stage hosts the checkable subset, cache-stable composition places it in the byte-stable prefix, and an externally-sourced avoid-list carries provenance like any other content. Concept-only. |

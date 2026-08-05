# Invariant Tripwires

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The project is good at *producing* rules. An observed pattern is ratified into a standing
convention; a review finds a defect and the fix lands with an explanation; a specification
states an invariant every implementation must respect. What none of that establishes is
**who notices when the rule is broken next time**.

In practice the answer is: whoever happens to be reading. The rule is enforced by memory —
a contributor's, a reviewer's, an agent's — and memory is exactly the mechanism that
failed the first time. So the pattern returns: a refactor reintroduces the call the rule
forbade, a new entry point skips the helper everything was supposed to route through, a
convenience shortcut re-opens the hole a past incident closed. Nothing fails, because
nothing was watching.

A **tripwire** is the missing enforcement half: a narrow, automated check that lives with
the code and **fails when one specific forbidden shape reappears**. It is not a linter and
not a behavior test. It is a named check for one rule, written at the moment the rule was
earned, that states the pattern, the reason, and the sanctioned alternative.

The characteristic tripwire is embarrassingly simple — search a declared scope for a shape
that must not exist there, fail if it is found. Its value has nothing to do with
sophistication. Its value is that the rule stops depending on anyone remembering it.

## Related Specifications

- [l1-pattern-codification.md](l1-pattern-codification.md) — the **producer** this concept completes: codification promotes an observed pattern into a ratified rule (PC-1/PC-2) but never asks *how the rule is enforced afterwards*. TW-1 answers that, and TW-8 is the enforcement-side counterpart of PC-5's demote-when-it-stops-holding.
- [l1-convergence-gate.md](l1-convergence-gate.md) — where a tripwire is **placed**: at the boundary all paths converge on, not at each entrance (CG-1). TW-10 adopts its delta discipline directly — a gate that fails on accumulated pre-existing state gets disabled, so the tripwire gates new violations.
- [l1-quality-standards.md](l1-quality-standards.md) — tripwires are a **tier** in the gate model, not a parallel one: they run where the other mandatory checks run and block on the same terms.
- [l1-change-containment.md](l1-change-containment.md) — the discipline a structural tripwire mechanizes: CTN-5's mechanical-vs-semantic separation and CTN-4's idiom precedence are exactly the kind of rule that decays under memory-only enforcement.
- [l1-completion-verification.md](l1-completion-verification.md) — CMP-2's *the claim names its check* is what a tripwire supplies for an architectural rule: the rule becomes a claim with a runnable proof rather than an assertion in a document.
- [l1-evaluation-suites.md](l1-evaluation-suites.md) — the **behavioral** sibling: suites ask *does it do the right thing*, tripwires ask *is it built the right way*. TW-6 states why neither substitutes for the other.
- [l1-optimization-integrity.md](l1-optimization-integrity.md) — OI's unverified-is-never-rendered-as-verified is the honesty rule TW-9 applies to governance itself: a rule with no tripwire is *stated*, not *enforced*, and the difference is reported.
- [l1-solution-frugality.md](l1-solution-frugality.md) — FR-7's closed finding vocabulary is the model TW-4 follows; a tripwire failure is a finding with a location and a replacement, not a bare match.
- [l1-spec-driven-governance.md](l1-spec-driven-governance.md) — the specification layer states invariants; this concept is how a stated invariant acquires a mechanical guardian outside the document that states it.
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.6): the language's validate-before-run stage is the natural host for authored-workflow tripwires, and its canonical-form guarantee gives them a reformat-proof target.

## 1. Motivation

**Every rule has a half-life under memory-only enforcement.** The rule is vivid the week
the incident happened and invisible six months later, and the people most likely to break
it are the ones who were not there. This is not a discipline problem to be solved by
trying harder; it is a property of enforcement mechanisms that require recall.

**Agents make the decay faster, not slower.** A generating agent works from what is in its
context. A rule that lives in a document it did not load, or in a review comment from
three months ago, has no effect on what it writes — and it writes a lot. The volume that
makes agentic development fast is the same volume that reintroduces a forbidden pattern at
scale, quietly, in a diff that otherwise looks fine.

**Behavioral tests are structurally blind to structural rules.** "Every write must go
through the sanctioned helper", "this module must not import that one", "this call must be
paired with its cleanup" — a system violating any of these can pass every behavior test it
has, because the behavior is still correct *today*. The defect is in the shape, and it
surfaces later as a class of bugs, not as a failing assertion.

**The moment of maximum information is the moment of the fix.** When a defect is being
repaired, the exact bad pattern, the reason it was wrong, and the correct alternative are
all present and precise. A day later they are a summary; a month later they are folklore.
Any tripwire written later is written from a worse position, which is why it usually is not
written at all.

**Governance without enforcement inventory is self-deception.** A project with fifty
stated rules and six enforced ones behaves like a project with six rules, but reasons about
itself as if it had fifty. The gap is invisible unless something names it.

## 2. Constraints & Assumptions

- A tripwire targets a rule that is **mechanically checkable**: a shape that can be found,
  or an absence that can be verified. Rules requiring judgement (is this the right
  abstraction?) are out of scope and belong to review.
- The check runs where the project's other mandatory checks run, on the same failure terms.
  This concept defines no new execution machinery.
- Tripwires are **cheap by construction**. A check expensive enough to be argued about is
  one that will eventually be removed for being slow.
- The concept is technology-neutral: "search a scope for a shape" covers a source grep, a
  dependency-direction assertion, a schema check, or a static query — the mechanism is not
  specified here.
- False positives are the primary adoption risk and are managed by narrowness (TW-5) and
  declared exemptions (TW-7), not by loosening the rule.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **TW-1 A mechanically checkable rule carries a mechanical check**: when a rule states
  that a shape must never appear, or may appear only behind a designated entry point, it is
  accompanied by an **automated check** that fails when the rule is violated. A rule of that
  form enforced only by review, documentation, or instruction is enforced by **memory** —
  the mechanism whose failure created the need for the rule.

- **TW-2 The tripwire is authored when the rule is earned, in the same change**: a
  tripwire is written **at the moment** the defect is fixed or the rule ratified, not
  scheduled for later. The information required to write a good one — the exact offending
  shape, why it was wrong, what replaces it — is at its maximum in that moment and decays
  immediately. A tripwire deferred is, in practice, a tripwire never written.

- **TW-3 One tripwire, one rule, named for the rule**: a tripwire checks exactly **one**
  invariant and its identity names that invariant, so a failure reads as *this rule was
  violated* rather than *a check failed*. A bundled guardian of several rules produces a
  failure that cannot be acted on without first reverse-engineering the check.

- **TW-4 A failure names the rule, the site, and the sanctioned alternative**: the failure
  message states **what may not appear**, **where it appeared**, **why the rule exists**,
  and **what to do instead**. A tripwire that fails with a bare pattern match teaches
  nothing, and a check that teaches nothing is one that gets suppressed rather than
  satisfied.

- **TW-5 Targeted and cheap, never a general style rule**: a tripwire is a **narrow** check
  for a specific known-bad shape in a **declared scope**. Broad stylistic rules belong to
  the formatter and the linter. The tripwire's authority comes from its precision and its
  stated reason; a broad, slow, or noisy check spends that authority and is eventually
  deleted — taking the rule with it.

- **TW-6 Structural checks and behavioral tests are distinct kinds, and both are
  required**: a behavioral test asks *does it do the right thing*; a tripwire asks *is it
  built the right way* — a dependency direction, a single-entry-point rule, a forbidden
  call outside its wrapper, a required pairing. A system can satisfy every behavioral test
  while violating every structural rule, because the behavior is still correct **today**.
  Neither kind substitutes for the other, and a suite with only behavioral tests has no
  defense against structural drift.

- **TW-7 Exemptions are explicit, narrow, and reasoned**: where a rule has a legitimate
  exception, the exception is declared **inside the tripwire** as a named allowlisted site
  carrying its justification — never by widening the pattern until it stops matching, and
  never by an anonymous blanket suppression. An undeclared exemption silently generalizes
  into the rule's repeal.

- **TW-8 A rule that no longer holds is retired deliberately, never weakened quietly**:
  when the underlying hazard is genuinely gone, the tripwire is **deleted together with the
  rule**, in a change that says so. Progressively loosening a pattern until it matches
  nothing is how a rule dies while continuing to look enforced — the worst state, because
  the inventory (TW-9) still counts it. This is the enforcement-side counterpart of
  demoting a codified rule that stops holding.

- **TW-9 The enforcement inventory is legible**: which stated rules carry tripwires — and,
  crucially, **which do not** — is inspectable. The uncovered set is the honest measure of
  how much of the project's governance is running on memory, and publishing it is what
  turns "we have rules" into "these rules are enforced; those are aspirations".

- **TW-10 A tripwire gates the delta, never the accumulated state**: pointed at a corpus
  that already violates the rule, a tripwire that fails on the **existing total** blocks
  every change including unrelated ones, and is therefore disabled — which removes the rule
  entirely. It fails on a **new** occurrence; the pre-existing backlog is recorded and
  reduced as its own work. Adoptability is a correctness property here, not a convenience:
  an unadoptable gate protects nothing.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The shape of a tripwire

```text
[REFERENCE]
Tripwire {
  rule_id      : the invariant it guards          // TW-3, one rule
  scope        : where the rule applies           // TW-5, declared and narrow
  forbidden    : the shape that must not appear   // or: the pairing that must exist
  exemptions   : [(site, justification)]          // TW-7, named and reasoned
  message      : rule + site + why + alternative  // TW-4
  basis        : delta                            // TW-10, new occurrences only
}
```

Everything in that record exists to make a failure *actionable by someone who has never
heard of the rule*. That is the design target: the tripwire is read almost exclusively by
people encountering it for the first time, usually while trying to do something else.

### 4.2 What a tripwire catches that a test cannot

| Rule kind | Example shape | Why behavior tests miss it |
| --- | --- | --- |
| Single entry point | a resource acquired outside its designated wrapper | both paths work today; only one cleans up on the error path |
| Dependency direction | an inner layer importing an outer one | the code runs; the coupling surfaces at the next refactor |
| Required pairing | an acquire with no matching release in scope | the leak is invisible until sustained load |
| Forbidden construct | a broad process-matching kill, a raw link where a helper is required | works on the author's machine, harms a sibling process or platform |
| Provenance rule | a sink writing content that skipped the sanitizing path | output looks fine until the one input that does not |

The common thread: **the system is correct today and wrong by construction**. That is
precisely the region a behavioral suite cannot reach, and precisely the region where the
expensive incidents live.

### 4.3 Delta gating (TW-10)

```text
[REFERENCE]
verdict(tripwire, change):
    new_hits := matches(tripwire, change.added_or_modified) - exemptions
    if new_hits ≠ ∅:  return Fail(message_for(new_hits))     // blocks this change
    return Pass                                              // pre-existing hits: recorded, not blocking
```

The recorded pre-existing set is a **backlog with a count**, reduced deliberately. Two
properties follow: the tripwire is adoptable on day one against a corpus that violates it,
and the backlog's trend is visible — a rule whose backlog grows despite the gate means the
gate's scope is wrong, not that the team is undisciplined.

### 4.4 Placement (composing the convergence gate)

A tripwire is placed at the **convergence point** the rule protects, not at every entrance
that might reach it. If five call sites can acquire the resource and one wrapper is
sanctioned, the tripwire guards *"acquisition outside the wrapper"* — one rule, one check —
rather than five per-site checks that a sixth site will silently escape.

This is why the enforcement inventory (TW-9) is small even when the rule set is large: one
well-placed structural check subsumes an unbounded number of individual violations.

### 4.5 The failure message is the whole product

```text
[REFERENCE]
"<rule name>: <shape> found at <site>.
 Why: <the hazard this rule prevents, in one sentence>.
 Instead: <the sanctioned alternative, named concretely>.
 If this is a legitimate exception, add it to the tripwire's exemption list with a reason."
```

The last line matters as much as the first. Without a declared path for a legitimate
exception, the only available action is to weaken the pattern (TW-8's failure mode), and
the rule dies through the exact door the message left open.

### 4.6 nodus projection

The workflow layer already has the machinery and gains three concrete applications, none
requiring a new primitive:

1. **The validate-before-run stage is the natural host.** The language already refuses to
   execute a workflow carrying a block-class validation error, and already treats unknown
   commands and undefined variables as **validation-time** failures rather than run-time
   surprises. A tripwire over authored workflows is another validation-stage rule: a
   forbidden step shape, a step that must only be reached through a declared macro, a
   required pairing of an effectful step with its compensating action.
2. **Canonical form makes tripwires reformat-proof.** Because the compact and human forms
   are semantically equivalent and round-trip to an equal structure, a rule can be stated
   over the **canonical structure** rather than over surface text — so a reformat, a
   rename of whitespace, or a switch of representation cannot evade a check the way a
   text-level pattern can. This is a materially stronger foundation than a source grep, and
   it is available for free.
3. **Host-supplied rules, language-supplied hook.** Which shapes are forbidden is a host
   policy concern (a project's own architecture rules), consistent with how every other
   policy maps onto the provider surface: the language contributes the validation stage and
   the canonical target, the host contributes the rule set and its severity.

## 5. Implementation Notes

1. Write the tripwire in the same change as the fix (TW-2). A backlog item named "add a
   guard for this later" is the observable form of a tripwire that will not exist.
2. Prefer a check over the **canonical or structural** form wherever one is available; a
   surface-text pattern is the fallback, and it is the one that decays under refactoring.
3. Keep the exemption list in the tripwire, not in the source (TW-7): an exemption beside
   the violation is invisible to anyone auditing the rule, and it multiplies.
4. Publish the inventory (TW-9) next to the rule set, not in a separate report — the point
   is that a reader of the rules sees immediately which ones are real.
5. Give each tripwire the rule's own identifier so a failure, a specification invariant, and
   a codified convention all name the same thing.

## 6. Drawbacks & Alternatives

- **Tripwires can become their own maintenance burden.** Bounded by TW-5 (narrow and cheap)
  and TW-8 (retired deliberately when the rule dies). A tripwire that is expensive or noisy
  is a defect in the tripwire, and the fix is to sharpen or delete it — never to loosen it.
- **A pattern check can be evaded trivially by a determined author.** True and accepted:
  this is a guard against *forgetting*, not against *intent*. The honest claim is that it
  catches the reintroduction that nobody meant to make, which is the overwhelming majority.
- **False positives train people to bypass checks.** The primary adoption risk, addressed by
  narrowness, by a declared exemption path (TW-7), and by delta gating (TW-10) so the check
  never fails for reasons unrelated to the change in hand.
- **Alternative — rely on code review.** Rejected by TW-1: review is memory with extra
  steps, it is exactly what failed the first time, and it scales worst precisely when
  generated volume is highest.
- **Alternative — a general linter with many rules.** Rejected by TW-5/TW-4: a rule buried
  in a generic ruleset loses its reason, and a failure without its reason is suppressed
  rather than fixed. Broad style belongs to the linter; earned architectural rules do not.
- **Alternative — express the rule in the type system so violation is unrepresentable.**
  **Preferred wherever possible** and not an alternative at all: a rule made structurally
  impossible needs no tripwire. TW-1 applies to the large remainder that cannot be encoded
  that way.
- **Alternative — fold into the quality-standards gate model.** Rejected: quality standards
  say *which gates must pass*; this says *how an earned rule acquires a gate at all*, and
  it has its own lifecycle (authored at the fix, exempted explicitly, retired with the
  rule).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CODIFICATION]` | `.design/main/specifications/l1-pattern-codification.md` | The rule producer this concept completes (PC-1/PC-2/PC-5). |
| `[CONVERGENCE]` | `.design/main/specifications/l1-convergence-gate.md` | Placement (CG-1) and the delta-gating discipline TW-10 adopts. |
| `[QUALITY]` | `.design/main/specifications/l1-quality-standards.md` | The gate model tripwires run inside, not beside. |
| `[EVAL]` | `.design/main/specifications/l1-evaluation-suites.md` | The behavioral sibling TW-6 distinguishes from. |
| `[COMPLETION]` | `.design/main/specifications/l1-completion-verification.md` | CMP-2 — the claim names its check; a tripwire is that check for a structural rule. |
| `[CONTAINMENT]` | `.design/main/specifications/l1-change-containment.md` | A discipline whose mechanical half a tripwire supplies. |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the discipline projects onto (§4.6). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — invariant tripwires as the enforcement half the rule-producing layers never supplied: a mechanically checkable rule carries a mechanical check, because a rule enforced by review or documentation is enforced by *memory*, the mechanism whose failure created the rule (TW-1); the tripwire is authored in the same change that earns the rule, when the offending shape, the reason, and the alternative are all still precise (TW-2); one tripwire per rule, named for the rule, so a failure reads as a rule violation rather than a failed check (TW-3); the failure names rule, site, reason, and sanctioned alternative, since a check that teaches nothing is suppressed rather than satisfied (TW-4); targeted and cheap rather than a general style rule, because precision is where its authority comes from (TW-5); structural checks and behavioral tests are distinct kinds and both required, since a system can pass every behavior test while violating every structural rule — the behavior is still correct *today* (TW-6); exemptions explicit, narrow, and reasoned inside the tripwire, never by widening the pattern (TW-7); a rule that stops holding is retired with its tripwire rather than quietly weakened, the worst state being a rule that looks enforced while matching nothing (TW-8); the enforcement inventory is legible, the uncovered set being the honest measure of how much governance runs on memory (TW-9); and the gate is on the **delta**, never the accumulated state, since a tripwire that fails on pre-existing debt is disabled and protects nothing (TW-10). Nodus projection needs no new primitive — the validate-before-run stage hosts authored-workflow tripwires, and the canonical-form guarantee gives them a reformat-proof structural target rather than a surface-text pattern. Concept-only. |

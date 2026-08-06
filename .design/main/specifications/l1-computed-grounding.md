# Computed Grounding

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Computed grounding is the rule that **when an answer is mechanically computable from state the system already holds, the system computes it and the model only phrases it**. The model narrates; the code decides.

The class of question this applies to is narrow and extremely common: *which of my things are in state X*, *how many are there*, *what is the total*, *when did it last happen*. These have exactly one correct answer, the system holds every fact needed to produce it, and a language model asked to derive it from a context dump will — reliably, not occasionally — miscount, include a neighbouring category, or return a different set in a different language than it returned in the first.

The fix is not a better prompt or a bigger context. It is to compute the answer first, inject it as an authoritative constraint, and let the model do the one thing it is actually better at: saying it well, in the right language, in the right tone. This makes the wrong count **unrepresentable** rather than detectable, which is a different and cheaper guarantee than verifying the answer afterwards.

## Related Specifications

- [l1-claim-verification.md](l1-claim-verification.md) — the **after-the-fact** counterpart: CV checks a produced claim against sources and downgrades confidence (CV-3/CV-6, advisory, never edits). Computed grounding acts **before** generation and prevents the class outright; §4.4 states why both are wanted and neither replaces the other.
- [l1-operational-ledger.md](l1-operational-ledger.md) — OL-7's *exact grounding, not similarity recall* is the retrieval-side sibling: retrieve the exact fact rather than something similar. This spec is the generation-side sibling: **compute** the exact answer rather than asking for a derivation.
- [l1-report-prompting.md](l1-report-prompting.md) / [l1-system-readout.md](l1-system-readout.md) — surfaces where a stated count or set is user-facing and a drift between the readout and the narration is directly visible.
- [l1-content-language.md](l1-content-language.md) — CGR-6's extension/expression split is what makes a grounded answer stable across locales: the phrasing is the model's and the language is the user's, but the set is neither's to change.
- [l1-outcome-confidence.md](l1-outcome-confidence.md) — a computed-grounded answer's factual core carries the certainty of the computation, not of the generation; conflating the two mis-states confidence in both directions.
- [l1-output-repair.md](l1-output-repair.md) — the sibling deterministic layer on the **output** side; this one acts on the **input** side of the same call. Both replace a round trip with computation, and both must stay strictly inside form/extension so they never invent content.
- [l1-negative-specification.md](l1-negative-specification.md) — NEG-4's *reach the generator before production* is the same economics: a constraint in the request costs tokens, a correction after the fact costs a rewrite.
- [l1-context-degradation.md](l1-context-degradation.md) — CGR-7's ungrounded fallback is a declared degradation, not a silent one.
- [l1-intent-resolution.md](l1-intent-resolution.md) — CGR-3's detector is an intent classification, and its conservatism requirement is the same one that keeps an imperative from being read as a question.

## 1. Motivation

Give a model a snapshot of two hundred entities and ask how many lights are on. It will answer with a number and a list. The number and the list will frequently disagree with each other, will include an item from an adjacent category, and — asked the same question in another language — will produce a different set from the same snapshot.

None of this is a prompt-quality problem, and it does not go away with a stronger model; it gets rarer, which is worse, because it stops being reproducible while staying wrong. The operation being asked for is *filter a collection and count it* — an operation the system performs exactly, instantly, and for free, and which the model is being asked to simulate by reading a serialized list.

Left unaddressed, the failure has an unusually damaging shape for trust: the answer is **specific**. A vague answer invites checking. "Three lights are on: kitchen, hallway, porch" invites belief, and when the porch light is off the user learns not that the count was wrong but that the system does not know its own state — a conclusion that generalizes to everything else it says.

The alternatives all cost more and deliver less. Verifying afterwards means generating a wrong answer, detecting it, and regenerating — two calls to arrive where computation arrives in microseconds. Adding "be careful to count accurately" to the prompt spends tokens on an instruction the model cannot reliably follow. Dumping more context makes the derivation harder, not easier.

## 2. Constraints & Assumptions

- **Only for the mechanically computable.** The scope is answers derivable by a deterministic function over state the system holds — filters, counts, sums, extremes, lookups. Anything requiring judgment is out.
- **The system holds the state.** This is a contract about a system answering questions about **itself** and the things it manages, not about the world.
- **The model is still doing real work.** Phrasing, language, tone, and the surrounding explanation remain the model's, and they are the reason the answer is not just a table.
- **The detector is fallible.** Deciding "is this that kind of question" is itself a classification, and this spec assumes it will sometimes be wrong in both directions.
- **Computation is cheap relative to generation.** If a grounding computation is expensive, the trade this spec assumes no longer holds and the case is not a candidate.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **CGR-1 (Compute the answer, generate the wording):** where a request's factual core is **mechanically computable** from state the system already holds, the system MUST compute it deterministically and the model's role is reduced to **expressing** it. Asking a model to derive a filter-and-count from serialized context, when the same operation is one exact function call away, is a design defect and not a prompting problem.

- **CGR-2 (The computed result is an authoritative constraint, explicitly outranking derivation):** the computed value enters the request as a constraint that **states its own authority** — it is the answer, it was computed from live state, and it outranks whatever the model would conclude from the surrounding context. A computed value offered as merely more context is one input among many and will lose to a plausible-looking derivation; the constraint must say what exactly must be reproduced (the members, the count, the value) and that nothing may be added or dropped.

- **CGR-3 (Narrow, conservative, observable detection):** grounding applies only on an explicit detected trigger, and the detector MUST be **conservative** — an over-eager detector that grounds a request it misread is **worse than no grounding at all**, because it forces a confidently-stated wrong answer with the full authority of CGR-2. Requiring an unambiguous signal (a question form rather than an instruction, an identifiable target category) is the shape of that conservatism, and misfires in both directions MUST be observable so the detector can be corrected against real traffic rather than intuition.

- **CGR-4 (Computed from the same state the model sees):** the deterministic computation and the context supplied to the model MUST derive from the **same snapshot**. A constraint computed from a later or earlier instant contradicts the context the model is reading, forcing it to choose between two authorities and producing a new failure class — an answer that is internally inconsistent and wrong in a way neither source explains.

- **CGR-5 (Empty is a first-class answer):** a computed result of **nothing** — an empty set, a zero count — is stated plainly and completely. This is the case most at risk: a model that has been handed a question and a context tends to find *something*, and an empty result is exactly the one it is most tempted to fill. The constraint MUST say the result is empty, and MUST say that nothing is to be listed.

- **CGR-6 (Extension is fixed, expression is free):** the constraint fixes the **extension** — which members, how many, what value. The model owns the **expression** — wording, language, register, and any surrounding explanation. This split is what makes a grounded answer stable across locales and phrasings: the same question in another language produces the same set, because the set was never the model's to derive. It is also what keeps the answer from degenerating into a table.

- **CGR-7 (Ungrounded is a declared degradation, never a silent one):** if the deterministic path cannot resolve — unavailable state, an unsupported target, an ambiguous request — the turn proceeds **ungrounded** rather than failing, and the fact that it was ungrounded is **recorded**. An answer silently downgraded from computed to derived is indistinguishable from a computed one at the point of use, which converts a known reliability boundary into an unknown one.

- **CGR-8 (Prevention, not verification — and both):** this contract makes the wrong count **unrepresentable at generation time**; claim verification catches it **afterwards** and downgrades confidence. They are different instruments with different costs, and the cheap deterministic one runs first. Neither substitutes for the other: verification alone pays for a wrong answer before detecting it, and grounding alone covers only the computable subset. An implementation MUST NOT drop verification on the grounds that grounding exists.

- **CGR-9 (Grounded-ness is inspectable):** whether an answer's factual core was computed or derived, and by which computation, MUST be inspectable. Without it nobody — user, developer, or evaluator — can distinguish a model that got it right from a model that was told, which makes both the model's real accuracy and the grounding layer's real coverage unmeasurable.

## 4. Detailed Design

### 4.1 The flow

```
request
   │
   ├─▶ detector (conservative, CGR-3) ── no match ──▶ ordinary generation
   │                                                   (recorded as ungrounded, CGR-7)
   └─ match
        │
        ├─▶ deterministic computation over the snapshot (CGR-4)
        │        └─ result: members + count + value (possibly empty, CGR-5)
        │
        ├─▶ constraint injected with declared authority (CGR-2)
        │
        └─▶ generation: model phrases it in the user's language (CGR-6)
                 │
                 └─▶ answer, marked as computed-grounded (CGR-9)
```

### 4.2 Anatomy of the constraint

An effective constraint carries four things, and dropping any one of them reintroduces the failure:

| Element | Without it |
| --- | --- |
| The result itself (members, count, value) | Nothing to reproduce |
| A statement of **authority** — computed from live state, trust over your own reading | Becomes one context item among many and loses to a plausible derivation |
| An **exactness** clause — reproduce these and only these, add nothing, drop nothing | Neighbouring items get helpfully included |
| The **empty case** handled explicitly | A zero result gets filled in (CGR-5) |

### 4.3 The two directions the detector can fail

| Failure | Effect | Severity |
| --- | --- | --- |
| **Miss** — a groundable request is not detected | Falls back to ordinary generation; the old failure mode, no worse | Tolerable |
| **False fire** — a non-groundable request is grounded | A confidently-stated wrong answer carrying CGR-2's authority | **Worse than having no grounding** |

The asymmetry is why CGR-3 requires conservatism rather than coverage. A detector tuned for recall imports a new, higher-severity failure to remove a lower-severity one.

### 4.4 Why both prevention and verification

Prevention and verification look redundant and are not. Verification is **general** — it applies to any claim against any source — and **expensive**, because the wrong answer must be produced before it can be caught. Grounding is **narrow** — only the computable subset — and **nearly free**, because it replaces a derivation with a function call. Keeping both means the computable core is right by construction while everything outside it still gets checked; dropping verification because grounding exists leaves the much larger non-computable remainder unguarded, and dropping grounding because verification exists pays generation twice for answers the system already knew.

### 4.5 Nodus relevance

None new. A workflow step that needs a computed value binds it as an ordinary input before the generation step — the language already expresses *compute here, interpolate there*, and the authority framing is prompt content the host composes. Adding a grounding vocabulary would name host-specific state kinds the portable core must not know about.

## 5. Implementation Notes

- Compute and inject **at the same point** the context snapshot is assembled; splitting them across stages is how CGR-4's same-snapshot requirement quietly breaks under later refactoring.
- The constraint's wording is load-bearing and worth testing as behaviour, not prose: an assertion that the model reproduces the exact set, that it does not add a neighbouring category, and that it handles the empty case is three cheap tests that catch the regressions this layer exists to prevent.
- Track detector fires and misses separately from repair-style counts. The interesting number is not how often grounding ran but how often it *should* have and did not, and that requires sampling ungrounded traffic.
- Resist widening the detector to improve coverage. §4.3's asymmetry means each widening trades a tolerable failure for a severe one, and the pressure to widen arrives as a user report about a miss, never about a false fire — which is invisible.

## 6. Drawbacks & Alternatives

- **Coverage is inherently narrow.** Only mechanically computable cores qualify, which is a small fraction of what a user asks. That fraction is disproportionately trust-defining, because it is where being wrong is checkable.
- **The detector is a new component that can be wrong.** Mitigated by conservatism (CGR-3) and observability, not eliminated — this spec buys a large reduction in one failure class for a small, bounded new one.
- **Alternative — prompt the model to be careful:** rejected. It spends tokens on an instruction the model cannot reliably follow, and produces no signal about when it failed.
- **Alternative — verify and regenerate:** rejected as the primary mechanism (CGR-8). Two calls to reach what computation reaches immediately, and the user still sometimes sees the wrong answer first on a streaming surface.
- **Alternative — answer these questions deterministically end to end, with no model:** tempting and rejected by CGR-6. It loses the language, the register, and the surrounding explanation, and it forces the user to know which of their questions are the "structured" kind — which is exactly the seam a conversational surface exists to remove.
- **A constraint the model disobeys is a silent failure.** Nothing in this contract forces compliance; it makes compliance overwhelmingly likely and non-compliance detectable at verification (CGR-8). <!-- TBD: whether the reproduced set should be mechanically checked against the computed one at the output boundary, and whether that check is a repair (form) or a contract retry (semantics) -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[VERIFY]` | `.design/main/specifications/l1-claim-verification.md` | The post-hoc counterpart; CGR-8 keeps both |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | OL-7 exact grounding — the retrieval-side sibling |
| `[REPAIR]` | `.design/main/specifications/l1-output-repair.md` | The output-side deterministic sibling of the same call |
| `[LANGUAGE]` | `.design/main/specifications/l1-content-language.md` | Why extension/expression separation stabilizes across locales |
| `[NEGATIVE]` | `.design/main/specifications/l1-negative-specification.md` | NEG-4 reach-the-generator-before-production economics |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-06 | Core Team | Initial concept: **the model narrates, the code decides** for any answer whose factual core is mechanically computable from state the system already holds. Compute deterministically and reduce the model to expression, since asking it to derive a filter-and-count from serialized context when the exact operation is one call away is a design defect, not a prompting problem (CGR-1); the computed result enters as a constraint that **states its own authority**, because a computed value offered as mere context is one input among many and loses to a plausible derivation (CGR-2); **narrow, conservative, observable detection** — a false fire is *worse than no grounding*, forcing a confidently wrong answer with full authority, so the detector is tuned against the asymmetry rather than for coverage (CGR-3); computed from the **same snapshot** the model reads, or the constraint contradicts the context and forces a choice between two authorities (CGR-4); **empty as a first-class answer**, the case a model handed a question is most tempted to fill (CGR-5); **extension fixed, expression free** — the split that makes the same question in another language return the same set while keeping the answer from degenerating into a table (CGR-6); ungrounded as a **declared** degradation, since a silent downgrade turns a known reliability boundary into an unknown one (CGR-7); **prevention and verification both kept** — grounding is narrow and nearly free, verification is general and pays for the wrong answer before catching it, and dropping either leaves a real gap (CGR-8); grounded-ness inspectable, without which neither the model's accuracy nor the layer's coverage is measurable (CGR-9). §4.4 argues the non-redundancy; §4.5 records the nodus disposition — no new invariant, a computed value is an ordinary bound input. |

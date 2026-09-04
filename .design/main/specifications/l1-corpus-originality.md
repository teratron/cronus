# Corpus Originality

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

Corpus originality is the **admission gate on sameness**: when a new definition enters a curated corpus that is *selected from by description* — a role catalog, a skill library, an archetype set, a persona collection, a template or extension catalog — the corpus must reject the candidate that is a **re-skin of a member it already has**.

The failure this prevents is specific and invisible to every other gate. A re-skin is well-formed, passes schema validation, reads well, and is individually useful; it is a find-and-replace of an existing member with a swapped proper noun, platform, market, or domain word. Nothing in a correctness check can see it. What it costs is not disk: it is **routing**. Two near-identical descriptors make selection between them arbitrary, and an arbitrary selection is indistinguishable from a wrong one.

The gate has two moments and one mechanism: it **blocks at admission**, and the same measure run over the whole corpus **audits for convergence** — the point at which the corpus is telling you two members should have been one.

## Related Specifications

- [l1-roles.md](l1-roles.md) — ROL-9 is the *justification* half of role admission (a custom role must justify itself on ≥2 independent axes); this spec is the *content* half. A candidate can articulate two good reasons and still be a re-skin, and a re-skin's reasons are as duplicated as its body.
- [l1-office-archetype.md](l1-office-archetype.md) — OA-10 requires an archetype to compose the role catalog rather than fork it; an inline near-copy of a preset role is the forking OA-10 forbids, and this gate is how it is detected rather than merely prohibited.
- [l1-extension-marketplace.md](l1-extension-marketplace.md) — XM-5's publishing gate judges description-matches-behavior and responsible handling; originality is the third admission signal a curated catalog needs, because re-skin flooding degrades discovery without any entry being malicious.
- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — PD-4 makes descriptor accuracy a **correctness** property; two accurate descriptors that describe the same thing break routing exactly as one inaccurate descriptor does. This is the cost model behind ORI-11.
- [l1-memory-consolidation.md](l1-memory-consolidation.md) — MC-6's redundancy→merge corpus-maintenance action is the *remediation* this gate's audit mode feeds; demarcated in §4.5 (memory consolidates recorded observations, this gate admits authored definitions).
- [l1-pattern-codification.md](l1-pattern-codification.md) — PC-2 human-sole-ratification: an override of a duplicate finding is a human act with a recorded reason, never an agent self-admission (ORI-9). A corpus converging toward its own warn band is a codification signal (ORI-6).
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — TW-4 (a check that teaches nothing gets suppressed) shapes ORI-7's finding format; TW-7 (exemptions inside the check, never by widening the pattern) shapes ORI-8's refusal to relax thresholds.
- [l1-agent-framework-skeleton.md](l1-agent-framework-skeleton.md) — AFS-13's anti-collapse novelty rule guards *generational* diversity in a self-improvement lineage; demarcated in §4.5 — same word, different corpus and different failure.
- [l1-evaluation-suites.md](l1-evaluation-suites.md) — the calibration discipline (ORI-5): a threshold is a measurement result, not a preference.
- [l1-host-native-rendering.md](l1-host-native-rendering.md) — every duplicate admitted here is rendered to every target host, so the cost is multiplied across the whole outbound surface.
- [l1-solution-frugality.md](l1-solution-frugality.md) — the same instinct at a different scale: prefer extending what exists to adding a near-copy of it.

## 1. Motivation

Curated corpora grow by contribution, and contribution has a gradient: writing a genuinely new definition is hard, adapting an existing one by swapping its domain nouns is easy, and the adapted result is *mergeable* — well-formed, plausible, individually defensible. A reviewer comparing the candidate against the guidelines sees nothing wrong, because nothing is wrong with it in isolation. The problem is only visible against the corpus, and no human reviewer holds a several-hundred-member corpus in their head.

The result, unchecked, is a corpus that grows in count while shrinking in coverage. Selection degrades first — a query that should resolve to one member resolves to five, and whichever is picked, four were equally good candidates the router had no principle for rejecting. Maintenance degrades next: an improvement to one member does not reach the four near-copies, so the corpus quietly develops four stale variants of a fixed idea. Finally, trust degrades: a catalog that looks padded is treated as padded, including the parts that are not.

The counter-mechanism has to be **mechanical** (a human cannot do a corpus-wide comparison), **noun-blind** (the swapped word is exactly what a naive comparison sees as difference), and **calibrated** (a threshold nobody measured is a number nobody can defend when it fires).

## 2. Constraints & Assumptions

- **The corpus is curated and selected-from.** This gate applies where members compete for selection by description. It does **not** apply to logs, observations, evidence, or any append-only record, where repetition is data.
- **Legitimate near-neighbours exist.** Deliberate variants — a localization, a specialization, a reduced-fidelity twin — are real and must have a path (ORI-8) that is not "relax the threshold".
- **Similarity is a signal, not a verdict about intent.** The gate reports measured overlap. It does not accuse anyone of copying, and its language should not.
- **The measure runs on content, not on metadata.** Declared fields (name, category, colour, tags) are short, templated, and shared by design; comparing them produces noise in both directions.
- **The corpus's own distribution is the reference point.** What counts as anomalous overlap is a property of *this* corpus, measured, not a constant borrowed from elsewhere.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **ORI-1 (Novelty is an admission criterion):** admission to a curated corpus requires the candidate to be **substantively new relative to the existing members**. Well-formedness, usefulness, and a clear description are **necessary and never sufficient**. A definition that is an existing member with its nouns swapped is refused as a definition, however good it is as a document.

- **ORI-2 (Compared against the corpus *and* the pending set):** a candidate is measured against **every admitted member** and against **every other candidate in the same change set**. A corpus-only comparison is defeated by two identical newcomers arriving together — each is novel against the corpus and neither is novel against the other.

- **ORI-3 (Neutralized comparison):** the measure is computed over the candidate's **substantive body** with the tokens most likely to be find-and-replaced **neutralized** — proper nouns, platform/market/domain names, and the unit's own identifiers. Declared metadata is excluded. A comparison that treats a swapped domain word as difference measures the disguise instead of the content, and will pass every re-skin it is meant to catch.

- **ORI-4 (Two-threshold disposition, never a boolean):** the outcome is graded: **pass**, **warn** (surfaced for human review, non-blocking), **fail** (blocking). A single boundary forces one of two bad choices — a strict one that blocks legitimate neighbours, or a lax one that admits re-skins — and the warn band is where the genuinely ambiguous cases go to be looked at by a person.

- **ORI-5 (Thresholds are calibrated and the calibration is published):** the thresholds are derived from the **measured similarity distribution of the already-admitted corpus** — at minimum its maximum and median pairwise overlap — and are set with a **stated margin** above the observed baseline. The gate publishes that baseline alongside its thresholds wherever it reports. A threshold with no published baseline is a guess wearing the authority of a number, and the first time it fires on a legitimate contribution it will be deleted rather than defended.

- **ORI-6 (Re-measure the baseline; convergence is a signal, not a licence):** the baseline is **re-measured as the corpus grows**. A baseline drifting upward toward the warn band means **the corpus is converging** — its members are becoming versions of each other — and the correct responses are consolidation (MC-6 redundancy→merge) or codification of the shared core into one member the others compose. Raising the thresholds to keep the gate quiet is forbidden: it converts the corpus's most important health signal into silence.

- **ORI-7 (A finding names its nearest neighbour and its score):** every warn or fail identifies **the specific closest existing member**, the measured overlap, and where the candidate sits relative to the published thresholds. A finding that reports only "too similar" cannot be acted on and will be routed around rather than resolved (TW-4).

- **ORI-8 (Legitimate overlap is resolved on the content, never on the threshold):** an intended near-neighbour is admitted by **making the body genuinely distinct** (different substance, different procedure, different examples) or by **declaring the relationship explicitly** — a variant, a specialization, or an extension of a named member, which changes what the corpus stores from a second copy into a delta. It is **never** admitted by raising a threshold, adding a blanket exemption, or excluding a region of the corpus from measurement (TW-7). Exemptions, where truly needed, are narrow, reasoned, and recorded against the specific pair.

- **ORI-9 (Blocking to the agent, overridable by the human, with a reason):** an agent MUST NOT self-admit a candidate that the gate failed, and MUST NOT tune the gate to make its own candidate pass. A human MAY override a finding; the override is **recorded with its reason** and remains visible against the admitted member (PC-2 — the human is the sole author of what the corpus accepts as binding content).

- **ORI-10 (One mechanism, two moments — gate and audit):** the same measure runs (a) on candidates at admission and (b) over the **whole corpus** as a periodic audit. The audit's output is a ranked list of the corpus's own most-similar pairs, feeding merge/split/summarize maintenance (MC-6). A gate without the audit only prevents *new* duplication and never repairs the duplication that predates it or arrived through an override.

- **ORI-11 (The cost is routing ambiguity, not storage):** the harm this gate prevents is **selection becoming arbitrary** — two near-identical descriptors give a router no principled basis to choose, so a correct query returns an effectively random member (PD-4). Any implementation that justifies the gate on size, tidiness, or aesthetics has mis-stated its purpose and will trade it away the first time it is inconvenient.

- **ORI-12 (The duplicate search covers withdrawn members, and is keyed on the candidate's identifying features rather than its own phrasing):** [ADDED v1.1.0] ORI-2 measures a candidate against every admitted member and every pending sibling. Two retrieval rules make that measurement actually reachable. The search **includes retired, withdrawn, resolved, and rejected members**, not only the live corpus — a candidate matching a *closed* member is not a duplicate, it is a **regression signal** (the thing that was fixed, removed, or decided against is back), and that finding is worth strictly more than the duplicate it superficially resembles. And the search is keyed on the candidate's **identifying features** — the entities it names, the symbols it carries, the conditions it fires under — **never on the wording of the candidate's own title or summary**, because a self-phrased query retrieves the neighbours of the author's phrasing rather than the neighbours of the content, and reliably misses the near-duplicate written by someone who said the same thing differently. Where the corpus is one another party maintains, one further asymmetry holds: a duplicate costs the receiving party more than silence, so a candidate that adds nothing the existing member does not already contain is **withheld rather than submitted as agreement** — restating that a known thing is also true here is not a contribution.

## 4. Detailed Design

### 4.1 What the measure looks at

```
candidate ──▶ strip declared metadata
          ──▶ neutralize find-replace-prone tokens (proper nouns, platforms, markets, own name)
          ──▶ normalize (case, punctuation, whitespace)
          ──▶ overlapping fixed-length word windows
          ──▶ set-overlap ratio against every member + every sibling candidate
          ──▶ max overlap + the member it was against
```

Two properties matter more than the specific algorithm. **Windowed comparison** (contiguous multi-word spans rather than a bag of words) is what makes the signal specific: shared vocabulary is expected between two members of the same domain, shared *sentence spans* are not. **Neutralization before comparison** is what makes it re-skin-proof: it removes precisely the tokens a re-skin changes, so the disguise stops registering as difference.

### 4.2 Calibration, concretely

Calibration is a measurement, published with the gate:

| Published figure | Why it is published |
| --- | --- |
| Maximum pairwise overlap among admitted members | The observed ceiling of *legitimate* similarity in this corpus |
| Median pairwise overlap | Shows the ceiling is an outlier, not the norm |
| Warn / fail thresholds and their margin above the ceiling | Makes the gate's strictness a defensible number rather than a preference |

A healthy corpus shows a wide gap between the observed ceiling and the warn band. That gap is the gate's whole credibility: it is what lets a maintainer say a firing is an anomaly rather than a policy disagreement. ORI-6 exists because that gap is also a **measurement of corpus health over time** — when it closes, the corpus, not the gate, is what changed.

### 4.3 The three dispositions

| Band | Meaning | Action |
| --- | --- | --- |
| pass | Overlap within the corpus's normal range | Admit |
| warn | Above normal, below the duplicate threshold | Surface with the nearest neighbour named; a human decides; overlap is often legitimate specialization that ORI-8 wants declared |
| fail | At or above the duplicate threshold | Refuse; the resolution paths are "make it distinct" or "declare the relationship" |

### 4.4 The audit mode

Run over the whole corpus, the same measure produces the corpus's most-similar pairs ranked. This is the input to consolidation: the top of that list is where two members should become one, or where a shared core should be extracted into a member both compose (OA-10's compose-never-fork applied within the catalog itself). Running the audit only at admission time leaves the corpus's existing convergence permanently unmeasured.

### 4.5 Demarcation — three neighbours that are not this

- **Memory consolidation (MC-6)** de-duplicates *recorded observations* inside a growing corpus of evidence, after the fact, with confidence-proportional actions. This gate refuses *authored definitions* at the boundary, before admission. The audit mode (ORI-10) is the seam where this gate hands findings to that machinery.
- **Generational novelty (AFS-13)** prevents a self-improvement lineage from collapsing toward a single behaviour across generations. Its corpus is a lineage in time; this one is a catalog at rest. Same word, different failure.
- **Justification gates (ROL-9)** ask whether a new specialty is *warranted* — the reasons for its existence. This gate asks whether its *content* is new. Both are required: a re-skin usually has excellent warranted-sounding reasons, borrowed along with everything else.

## 5. Implementation Notes

- Compute the corpus's window sets once and reuse them; admission cost is then one candidate against a prepared index rather than a full pairwise pass.
- The neutralization list is **maintained**, not fixed: as the corpus takes on new domains, the nouns those domains swap join the list. A stale list quietly degrades the gate toward a naive comparison, which passes exactly the cases it exists to catch.
- Report every candidate's score, not only the failures. A run showing a column of small numbers is what makes a large number legible as the anomaly it is, and it is also the cheapest continuous calibration check (ORI-6).
- Wording matters: the finding describes *measured overlap with a named member*, never intent. The gate has no evidence about intent and does not need any to be correct.

## 6. Drawbacks & Alternatives

- **False positives on genuinely close specialties.** Two adjacent specialties in one narrow domain can legitimately overlap. Mitigated by the warn band (ORI-4), by the published margin (ORI-5), and by ORI-8's declare-the-relationship path, which turns a near-copy into an explicit delta.
- **The measure is syntactic.** A semantic re-write of the same definition — same substance, different sentences — passes. This gate catches the cheap copy, which is the common one; it does not claim to catch a determined one. Deeper semantic comparison is a possible later layer and would need its own calibration. <!-- TBD: whether an embedding-based second signal earns its cost and false-positive profile -->
- **Alternative — human review only:** rejected. It does not scale past a corpus a single reviewer can hold in memory, and it fails silently and unevenly (whichever member the reviewer happened to remember).
- **Alternative — a single blocking threshold:** rejected (ORI-4). It forces a choice between blocking legitimate neighbours and admitting re-skins, and gets tuned toward whichever error was most recently annoying.
- **Alternative — forbid variants entirely:** rejected. Localizations and specializations are legitimate corpus growth; ORI-8 gives them a path that keeps the relationship visible instead of hidden inside a duplicate body.
- **Ongoing calibration cost.** Re-measuring the baseline (ORI-6) is recurring work. It is also the only way the thresholds stay defensible, and its output doubles as a corpus-health metric worth having independently.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ROLES]` | `.design/main/specifications/l1-roles.md` | ROL-9 justification half of admission; this spec is the content half |
| `[ARCHETYPE]` | `.design/main/specifications/l1-office-archetype.md` | OA-10 compose-never-fork, detected rather than merely prohibited |
| `[MARKET]` | `.design/main/specifications/l1-extension-marketplace.md` | XM-5 publishing gate; the third admission signal a curated catalog needs |
| `[DISCLOSURE]` | `.design/main/specifications/l1-progressive-disclosure.md` | PD-4 descriptor accuracy as correctness — the ORI-11 cost model |
| `[CONSOLIDATION]` | `.design/main/specifications/l1-memory-consolidation.md` | MC-6 redundancy→merge, the remediation the audit mode feeds |
| `[TRIPWIRES]` | `.design/main/specifications/l1-invariant-tripwires.md` | TW-4 findings that teach, TW-7 exemptions inside the check |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-06 | Core Team | Initial concept: near-duplicate admission gate for curated, selected-from corpora (roles, skills, archetypes, personas, templates, catalog entries). Novelty as an admission criterion beside well-formedness (ORI-1); measured against the corpus **and** the pending change set, closing the two-identical-newcomers hole (ORI-2); comparison over the substantive body with find-replace-prone tokens **neutralized**, so a swapped domain noun cannot hide a copy (ORI-3); graded pass/warn/fail rather than a boolean (ORI-4); thresholds **calibrated against the corpus's own measured similarity distribution with a published baseline and stated margin** (ORI-5), the baseline re-measured over time with upward drift read as corpus convergence — a consolidation/codification signal, never grounds to relax the gate (ORI-6); findings that name the nearest neighbour and the score (ORI-7); legitimate overlap resolved by distinct content or a **declared** variant/specialization relationship, never by threshold relaxation or blanket exemption (ORI-8); blocking to the agent, human-overridable with a recorded reason (ORI-9); one mechanism at two moments — admission gate and whole-corpus convergence audit feeding MC-6 (ORI-10); and the stated cost model: the harm is **routing ambiguity** under PD-4, not storage (ORI-11). Demarcated in §4.5 from memory consolidation (observations, after the fact), AFS-13 generational novelty (lineage in time), and ROL-9 justification (reasons, not content). |
| 1.1.0 | 2026-09-04 | Core Team | Added ORI-12 — the duplicate search covers withdrawn members and is keyed on the candidate's identifying features, never its own phrasing. ORI-2 states *what* a candidate is measured against; ORI-12 states how that set is actually retrieved, and closes two ways the measurement silently misses. Retired, withdrawn, resolved, and rejected members are searched too, because a match among *closed* members is a **regression signal** (what was fixed, removed, or decided against has returned) and is worth strictly more than the duplicate it resembles. And the key is the candidate's identifying features — the entities it names, the symbols it carries, the conditions it fires under — not the wording of its own title, since a self-phrased query retrieves the neighbours of the author's phrasing rather than of the content and reliably misses the near-duplicate that said the same thing differently. For a corpus another party maintains, the asymmetry is stated: a duplicate costs the receiver more than silence, so a candidate adding nothing the existing member lacks is withheld rather than submitted as agreement. |

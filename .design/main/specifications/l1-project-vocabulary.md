# Project Vocabulary

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

An office is dropped into a project and has to infer the project's jargon from what it can
read. Lacking a settled term, it describes the concept instead — "the problem that happens
when a lesson inside a section of a course is given a real position in the file system"
where the people who built it would say "the materialization cascade". The description is
correct, it is twenty words long, and it is rebuilt from scratch every time the subject
comes up.

The cost compounds in four directions at once. Every statement about the concept is longer,
in every prompt and every artifact, forever. Two descriptions of the same concept do not
match, so nobody — human or agent — can tell whether a difference in wording is a
difference in meaning. Identifiers drift, because a producer that has no settled term
invents one per file. And the project loses the ability to say something short and exact,
which is the thing a shared language is actually for.

**Project vocabulary** is the governing artifact that fixes this: for each domain concept,
one **canonical term**, its definition, the **synonyms it displaces**, its **relationships**
to other terms, and the record of **ambiguities that were resolved**. It is authored
*during* the work — at the moment a distinction is live and the participants can settle it
— and it governs what gets produced, not merely how people talk: identifiers, module names,
work-item titles, documentation, and durable records all draw from it.

It is deliberately **not** the client-facing glossary. That one explains the project to an
outsider in plain language and is a derived projection. This one is authored, authoritative,
and internal — the source those projections are made from.

## Related Specifications

- [l1-project-wiki.md](l1-project-wiki.md) — the **boundary that matters most**: PW-1's glossary page explains the project to a non-technical client in plain language, and PW-3 makes the whole wiki a *projection, never a source of truth*. The governing vocabulary is the opposite on both axes — internal, authored, authoritative — and the wiki glossary is one of its derived renderings.
- [l1-pattern-codification.md](l1-pattern-codification.md) — the pathway a term travels to become canonical (VOC-7): repetition makes a candidate, durability plus human ratification makes it binding (PC-1/PC-2), and PC-5's demote-when-it-stops-holding is what retires a dead term.
- [l1-negative-specification.md](l1-negative-specification.md) — the displaced-synonym list (VOC-2) is a negative specification in miniature: NEG-5's *name the alternative* is exactly what a vocabulary entry does, and NEG-4's reach-the-generator-first is why the list belongs in the production context rather than in a review comment.
- [l1-operational-ledger.md](l1-operational-ledger.md) — OL-4's verbatim citation exists because paraphrase drifts; a settled term is the upstream cure, removing the paraphrase rather than policing it.
- [l1-content-language.md](l1-content-language.md) — the **orthogonal** axis: content language fixes *which language* an artifact is written in, this fixes *which terms* it uses. Both are declared per artifact kind and both are retrieval-integrity concerns.
- [l1-code-intelligence.md](l1-code-intelligence.md) — grounding a query in the corpus's actual vocabulary is the consumer side of this artifact; a canonical term makes the grounding exact instead of approximate.
- [l1-cache-stable-context.md](l1-cache-stable-context.md) — the vocabulary is stable across a session's requests and belongs in the cacheable prefix; VOC-10's compression benefit is realized against that placement.
- [l1-knowledge-base.md](l1-knowledge-base.md) / [l1-memory-model.md](l1-memory-model.md) — durable records written in canonical terms are findable by those terms; records written in ad-hoc paraphrase are not (VOC-5).
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — a displaced synonym appearing in a produced identifier is exactly the shape a structural check catches and a behavioural test does not.
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.6): the schema vocabulary is already a validated controlled vocabulary, and selective disclosure is already VOC-9's volume control.

## 1. Motivation

**A settled term is compression that pays every time.** Replacing a twenty-word description
with a two-word term shortens every prompt, every artifact, every record, and every future
statement about the concept — not once, but on every occurrence for the life of the project.
Few interventions in the context economy have that shape; most save once and are spent.

**Without a settled term, sameness is undecidable.** Two descriptions of one concept are
never identical, and a reader cannot tell whether the difference is deliberate. That
uncertainty is expensive precisely where it matters: in a specification, in a work item, in
a memory record that a later session will retrieve and have to reconcile against something
worded differently.

**Producers invent terms at the rate they produce.** A generator with no canonical term
coins one — per file, per session, per agent. The naming does not converge, because nothing
is converging it, and the codebase ends up carrying three names for one thing plus the
navigational cost of knowing that they are the same.

**The moment to settle a term is when it is contested, not later.** During the work, the
distinction is live and the people who understand it are present. A vocabulary compiled
afterwards records the terms everyone already agreed on and silently omits the ones that
were argued about — which are the only ones that needed writing down.

**A glossary of isolated definitions answers the wrong questions.** Confusion is rarely
about what a word means on its own; it is about how two concepts relate — whether this
contains that, whether these are the same thing at different stages. A vocabulary without
relationships leaves exactly that unanswered.

## 2. Constraints & Assumptions

- The vocabulary is **local-first project data**, authored and owned by the project, carrying
  no authority beyond naming.
- It covers **domain concepts** — the things the project is about. General technical
  vocabulary is not in scope; an entry exists because *this* project uses a word in a
  particular way or needed to distinguish two things.
- It is **small by intent** (VOC-9). Every property here degrades as it grows, which is why
  boundedness is an invariant rather than advice.
- Adherence in produced artifacts is partly mechanically checkable (a displaced synonym in an
  identifier) and partly not (a concept described rather than named). The concept covers both
  and does not require full mechanization.
- The spec defines no new store: the vocabulary lives with the project's other authored
  design artifacts.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **VOC-1 One canonical term per concept, declared with its definition**: each domain concept
  has exactly **one** canonical term and a definition stating what it is. Two live terms for
  one concept is not a stylistic difference: every reader and every producer must then decide
  whether the difference carries meaning, and they will decide inconsistently. The
  vocabulary's job is to remove that decision, not to catalogue both options.

- **VOC-2 Displaced synonyms are listed with the entry, never merely absent**: a canonical
  entry names the alternatives it **replaces**. An absent synonym is invisible guidance that
  the producer cannot follow; a listed one is a correction delivered *before* the wrong word
  is written. This is a negative specification at term granularity, and it obeys the same
  rule — name the alternative, and put it where the producer will see it.

- **VOC-3 Relationships between terms are part of the vocabulary**: entries state how the
  concepts relate — contains, is-a, one-to-many, precedes. A set of isolated definitions
  cannot answer the questions that actually cause confusion, which are almost always about
  connection rather than about a single word's meaning.

- **VOC-4 Ambiguities are resolved on the record**: when a term is found to carry two
  meanings, the vocabulary records **what was ambiguous and which meaning won**. Two things
  follow: the same ambiguity is not re-litigated in a later session, and a reader of older
  material can decode which sense was meant when it was written. An ambiguity resolved only
  in conversation is an ambiguity that returns.

- **VOC-5 The vocabulary governs produced artifacts, not only conversation**: identifiers,
  module and file names, work-item titles, commit messages, documentation, and durable memory
  records use the canonical terms. A vocabulary honoured in prose while the code says
  something else has **doubled** the project's terminology rather than unified it — and the
  code's version is the one future readers will believe.

- **VOC-6 Authored during the work, never as a documentation phase**: terms are proposed,
  challenged, and settled **at the moment the concept is being discussed or built**, when the
  distinction is live and the participants can adjudicate it. A vocabulary compiled afterwards
  captures what was already agreed and omits precisely the contested terms that needed
  settling.

- **VOC-7 A term earns canonical status; it is never self-decreed**: entry follows the
  codification discipline — repeated use raises a **candidate**, durability plus **human
  ratification** makes it canonical. A generating actor MUST NOT promote its own coinage into
  the governing vocabulary; it may propose, and the proposal is evidence-backed by the
  occurrences that raised it.

- **VOC-8 Authoritative source, distinct from any client-facing glossary**: the governing
  vocabulary is **authored and authoritative**; a plain-language glossary that explains the
  project to an outsider is a **derived projection** of it. Conflating them yields the worst
  of both — a governing artifact rewritten for readability, and an outward-facing document
  nobody is allowed to edit.

- **VOC-9 Precision is the goal and volume is a cost**: an entry exists because a distinction
  was genuinely being lost, not because a word occurred. An over-large vocabulary is not
  consulted, and an unconsulted vocabulary is **worse than none**, because artifacts still
  claim to follow it. Terms that fall out of use are retired rather than accumulated.

- **VOC-10 The benefit is measured, and compression is one of the measures**: adherence
  (canonical term versus a listed synonym in produced artifacts) and the compression a settled
  term buys are **countable**, and a claim that the project has a shared language is made with
  those numbers or not at all. The compression measure is the honest one: a term that replaces
  a paraphrase saves symbols on every occurrence, and a term nobody uses saves nothing.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The entry

```text
[REFERENCE]
Term {
  canonical    : the one word or phrase                    // VOC-1
  definition   : what it is
  displaces    : [synonyms this replaces]                  // VOC-2
  relations    : [(predicate, other-term)]                 // VOC-3
  resolutions  : [(what was ambiguous, which sense won, when)]   // VOC-4
  status       : candidate | canonical | retired           // VOC-7, VOC-9
}
```

`displaces` and `resolutions` are what distinguish this from a glossary. A glossary tells a
reader what a word means; those two fields tell a **producer** which word to use and tell a
**later reader** how to decode what was written before the question was settled.

### 4.2 Where the leverage is

| Without a canonical term | With one |
| --- | --- |
| the concept is re-described each time, at length | named in two words |
| two descriptions cannot be compared for sameness | identity is textual and exact |
| identifiers diverge per file and per session | naming converges by construction |
| retrieval misses records worded differently | records share the term that finds them |
| every statement costs more symbols, forever | the saving recurs on every occurrence |

The last row is the one that justifies the artifact economically, and it is the reason the
vocabulary belongs in the **stable, cacheable** part of a model-facing composition: it is
identical across a session's requests and pays its cost once.

### 4.3 Settling a term in flight (VOC-6 / VOC-7)

```text
[REFERENCE]
during work, on encountering a contested or re-described concept:
    propose(term, definition, displaces?)        // evidence: the occurrences that raised it
    challenge(term)                              // does it survive the edge cases?
    if ratified_by_human: status := canonical    // VOC-7 — never self-promoted
    else:                 status := candidate    // still usable, not yet binding
```

The challenge step is not ceremony. A term that has not been stress-tested against the edge
cases usually turns out to name two things, and discovering that after it is embedded in
identifiers is the expensive path.

### 4.4 Boundary with the client-facing glossary (VOC-8)

| | Governing vocabulary | Client-facing glossary |
| --- | --- | --- |
| Audience | the project's builders, human and agent | an outsider |
| Authority | authored, source of truth | derived projection |
| Optimized for | precision and distinction | plain-language accessibility |
| Edited by | the project, through ratification | the office, as a by-product |

Both should exist. Neither can do the other's job: a term precise enough to govern
identifiers is usually not the phrasing that explains the project to someone who has never
seen it, and the plain-language rendering must be free to change without renaming anything.

### 4.5 Adherence, and what it can and cannot check

Mechanically checkable: a **listed displaced synonym** appearing in a produced identifier,
title, or record. That check is narrow, cheap, and exactly the shape a structural tripwire
handles — the rule names the forbidden word and the sanctioned replacement, which is the
whole of a good failure message.

Not mechanically checkable: a concept **described instead of named**. That is a review-level
observation and is reported as guidance, never dressed up as an enforced rule — the same
auto-versus-advisory boundary the craft and negative-specification disciplines already draw.

### 4.6 nodus projection

The workflow layer already holds most of the machinery:

1. **A schema vocabulary is a controlled vocabulary, already validated.** Command names are
   declared in a schema and an unknown command fails at *validation*, before execution —
   which is VOC-1 enforced by construction for the command surface: there is exactly one name
   for each operation, and using another is not a style deviation but a refusal to run.
2. **Selective disclosure is VOC-9's volume control.** A workflow declares which vocabulary
   units it needs rather than loading the whole surface, so vocabulary breadth costs nothing
   until it is used — the same economy this spec asks for at the domain-term level.
3. **The gap is author-chosen names, and it is lintable.** Variable, macro, and workflow names
   are the author's, not the schema's, and a displaced synonym appearing among them is
   detectable at the same validate-before-run stage that hosts the project's other structural
   checks — host-supplied rules, language-supplied hook, consistent with every other policy
   concern.

## 5. Implementation Notes

1. Keep the vocabulary in one authored file that the production context loads (VOC-2/§4.2),
   not scattered across design documents; a term that is not in the loaded set does not exist
   as far as a producer is concerned.
2. Record `displaces` at the moment of resolution — it is the field that decays first, because
   once everyone has stopped saying the old word it feels redundant, and that is precisely
   when a new participant will reintroduce it.
3. Derive the client-facing glossary from the vocabulary rather than maintaining both (VOC-8);
   two hand-kept term lists diverge on the first rename.
4. Count adherence from the start (VOC-10). Retrofitting the measure loses the baseline that
   makes "the vocabulary is working" a statement rather than an impression.

## 6. Drawbacks & Alternatives

- **Vocabularies rot and grow.** The main risk, addressed by VOC-9 (volume is a cost, unused
  terms retired) and VOC-7's ratification, which stops casual coinages entering. The concept
  does not claim to prevent rot; it makes rot visible and retirement normal.
- **Settling terms mid-work interrupts the work.** Real, and cheaper than the alternative:
  the interruption is minutes, and the cost of embedding an ambiguous term in identifiers is
  paid on every later read and every rename.
- **A canonical term can be wrong.** Which is why VOC-4 records resolutions rather than
  silently replacing them, and why an entry can be retired. A wrong term that everyone uses is
  still better than three right ones nobody agrees on, and it is fixable in one place.
- **Alternative — rely on the client-facing glossary.** Rejected by VOC-8: it is a derived,
  plain-language projection optimized for an outsider, and making it authoritative means
  either the governing terms get simplified or the client document gets frozen.
- **Alternative — let naming conventions in code carry it.** Rejected by VOC-3/VOC-4: code
  names carry no definitions, no displaced synonyms, and no record of what was ambiguous — and
  the concepts that need the vocabulary most are usually the ones with no single code
  representation.
- **Alternative — compile the vocabulary at the end of a project.** Rejected by VOC-6: it
  records what was already agreed and omits every term that was contested, which inverts the
  artifact's purpose.
- **Alternative — fold into pattern codification.** Rejected: codification is the general
  *pathway* by which an observation becomes binding, and VOC-7 uses it. What it does not
  supply is the artifact's shape — one canonical term, its displaced synonyms, its relations,
  and its resolved ambiguities — nor the rule that produced artifacts must draw from it.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[WIKI]` | `.design/main/specifications/l1-project-wiki.md` | The client-facing glossary boundary (PW-1/PW-3). |
| `[CODIFICATION]` | `.design/main/specifications/l1-pattern-codification.md` | The ratification pathway VOC-7 uses. |
| `[NEGATIVE]` | `.design/main/specifications/l1-negative-specification.md` | The displaced-synonym list as a negative specification. |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | OL-4 paraphrase drift — the problem a settled term removes upstream. |
| `[LANGUAGE]` | `.design/main/specifications/l1-content-language.md` | The orthogonal axis: which language, versus which terms. |
| `[CACHE]` | `.design/main/specifications/l1-cache-stable-context.md` | Why the vocabulary belongs in the stable prefix (§4.2). |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the concept projects onto (§4.6). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — the governing project vocabulary, the artifact that lets a project say something short and exact: one canonical term per concept with its definition, because two live terms force every reader and producer to decide whether the difference carries meaning and they decide inconsistently (VOC-1); **displaced synonyms listed with the entry rather than merely absent**, since an absent synonym is invisible guidance while a listed one corrects before the wrong word is written (VOC-2); relationships between terms, because confusion is almost always about connection rather than about one word's meaning (VOC-3); ambiguities resolved **on the record**, so the same question is not re-litigated and older material stays decodable (VOC-4); the vocabulary governs **produced artifacts** — identifiers, titles, records — since one honoured only in prose has doubled the terminology and the code's version is what future readers believe (VOC-5); authored **during the work**, when the distinction is live, because a vocabulary compiled afterwards omits exactly the contested terms that needed settling (VOC-6); a term **earns** canonical status through the codification pathway and a generator never self-promotes its own coinage (VOC-7); authoritative and distinct from the client-facing plain-language glossary, which is a derived projection (VOC-8); precision as the goal and volume as a cost, an unconsulted vocabulary being worse than none since artifacts still claim to follow it (VOC-9); and measured adherence with **compression as the honest measure** — a settled term saves symbols on every occurrence, forever, while a term nobody uses saves nothing (VOC-10). Nodus projection needs no new primitive — the schema vocabulary is already a validated controlled vocabulary enforcing one name per operation, selective disclosure is already the volume control, and author-chosen names carrying a displaced synonym are lintable at the validate-before-run stage. Concept-only. |

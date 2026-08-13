# Semantic Addressing

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Semantic addressing is the contract for **naming a place inside an artifact so that the name survives the artifact changing, the surface changing, and the process restarting**.

Everything in the system that points *into* something — a review note on a line, a memory anchored to a symbol, a deep link to a card, an agent told to look at hunk three, a log entry citing a paragraph — is carrying an address. Left ungoverned, each of these is minted by whoever needed it, in whatever coordinates were closest to hand: an array index, a rendered row number, a scroll offset, a runtime handle. Those coordinates are correct exactly once, at the moment of minting, on the surface that minted them.

This spec fixes one grammar of **durable semantic addresses**, one resolution discipline with a ternary outcome, and one placement rule: where an address cannot be placed unambiguously, the container falls back to a presentation where every address can be placed — never a guess, never a partial placement, never a silent drop.

## Related Specifications

- [l1-attention-steering.md](l1-attention-steering.md) — every steering act names its target with an address from this grammar; a steering channel addressing by screen position breaks on the first reload.
- [l1-surface-parity.md](l1-surface-parity.md) — the sibling failure. Parity keeps surfaces from *deciding* differently; addressing keeps them from *pointing* differently. A re-derived address is simultaneously an SP-2 violation and an SA-1 one.
- [l1-code-intelligence.md](l1-code-intelligence.md) — CI-10's memories anchored to graph entities and CI-3's staleness signaling are this contract applied to one artifact kind; SA-5's ternary resolution generalizes their invalidation.
- [l1-document-understanding.md](l1-document-understanding.md) — positional anchoring of a parsed source document is where addresses are *minted*; this spec governs what happens to them afterwards.
- [l1-change-merge.md](l1-change-merge.md) — CM-10's self-mis-anchoring repair and SA-5's relocated/orphaned outcomes are the same event seen from the producing and consuming sides.
- [l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) — ATE-11's opaque reference passing is SA-9 for the agent-facing case: identifiers are copied verbatim, never retyped from memory.
- [l1-notes.md](l1-notes.md) / [l1-review-checkpoint.md](l1-review-checkpoint.md) — the artifacts most damaged when an address silently lands somewhere else.
- [l1-declarative-configuration.md](l1-declarative-configuration.md) — DC-8's stable hierarchical identifier is this contract for configuration surfaces; the two grammars must not diverge into competing dialects.
- [l1-multi-device-sync.md](l1-multi-device-sync.md) — addresses cross device boundaries and must resolve on a replica that never saw the surface that minted them.

## 1. Motivation

**Every convenient coordinate is a lie with a short half-life.** An array index is correct until the array is filtered. A rendered row is correct until the presentation changes, the window resizes, or a fold opens. A runtime handle is correct until the process restarts. Each of these is *free* to produce at the moment of use, which is why each of them gets used, and each becomes wrong at a moment nobody observes.

**Wrong addresses do not fail — they land somewhere.** This is the property that makes the class expensive. A stale identifier that resolves to nothing produces an error someone fixes. A stale index resolves to a *different, valid* target: the note about the security check appears beside the logging call, the memory about one function attaches to its neighbour, the human is steered to a line that says something else. The system stays green while its statements become false.

**The same address is re-derived at every boundary, and the derivations disagree.** A grammar that lives in nobody's file is restated by every producer and every consumer — one clamps and another wraps, one prefers the added side and another the first line, one is zero-based and another one-based. Every restatement is a coin flip on agreement, and the disagreements surface as "the note is in the wrong place in the other view", which reads as a rendering bug and is not one.

**Partial placement is worse than no placement.** When an alternate presentation can place four of five annotations, showing four looks like success. The reader has no way to know a fifth existed. A presentation that silently drops authored content has converted a visible limitation into an invisible data loss — and the reader's confidence in what they *do* see is now unearned.

**Nothing else in the corpus owns this.** Anchoring appears as a clause in the code-intelligence, document-understanding, and change-merge specs, each solving it for its own artifact and each with its own vocabulary. Three partial solutions with three grammars is the condition this spec ends.

## 2. Constraints & Assumptions

- **Artifacts change under their addresses.** Files are edited, documents regenerated, boards reordered, graphs re-indexed. An address that assumed stability was never an address.
- **Not every target survives.** Some addresses become genuinely unresolvable. The contract is about handling that honestly, not about preventing it.
- **Address minting is cheap; address repair is not.** The discipline has to be cheaper at the point of minting than the convenient coordinate, or it will not be followed.
- **Consumers may never have seen the producer.** A different surface, a different device, a later version of the program. An address that needs the producing renderer to be interpreted is not portable.
- **Address grammars are per-artifact-kind in their units, universal in their shape.** A line in a file, a paragraph in a document, a node in a graph, a card in a column — different units, one structure and one resolution discipline.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **SA-1 (One grammar, one serializer, one parser):** there is exactly **one** address grammar and one implementation of writing and reading it, shared by every producer and consumer. A boundary that restates the grammar — a wire format that re-lists the fields, a client that re-parses by hand — is the defect, not a convenience: each restatement is an independent chance to disagree, and the disagreements manifest as misplaced content rather than as errors.

- **SA-2 (Semantic keys only — never position, never runtime identity):** an address names its target by **durable semantic identity**: a stable key for the container, a named side or dimension where the artifact has more than one, and a one-based ordinal in the artifact's own units. Array indices, rendered row numbers, screen coordinates, scroll offsets, and process-local handles MUST NOT appear in an address that outlives the call that produced it.

- **SA-3 (Exactly one target — ambiguity is a rejection, never a precedence rule):** an address resolves to exactly one target, and a request carrying **more than one** targeting dimension is **refused**. Defining a precedence between competing targets ("line wins over unit") makes a caller's mistake silently productive; refusing it makes the mistake one corrected call. Under-specification is refused for the same reason.

- **SA-4 (An address records what it was resolved against):** an address carries the **identity of the content it was minted against** alongside the identity of its container, so a later resolution can distinguish *the same place* from *the same text*. Without it, relocation and coincidence are indistinguishable, and the system cannot tell a note that followed its subject from one that landed on an unrelated line that happens to look similar.

- **SA-5 (Resolution is ternary, and none of the three outcomes is silent):** resolving an address yields exactly one of **resolved** (found, unchanged), **relocated** (found elsewhere, with the evidence that justified the move), or **orphaned** (the target no longer exists). An orphan is **retained and disclosed**, never discarded: authored content whose anchor died is still authored content, and deleting it converts a recoverable state into data loss. Relocation states its evidence so a reader can disagree with it.

- **SA-6 (Clamp inside a resolved container; refuse an unresolved one):** where the **container** resolves but the ordinal within it is out of range, the ordinal is clamped into the container's real range — the caller is off by a little, and the nearest valid place is the answer they meant. Where the **container itself** does not resolve, the request is **refused** rather than redirected to a neighbouring container. Clamping across containers is how content lands in the wrong file.

- **SA-7 (Placement is all-or-nothing per container):** where a set of addresses must be placed into a presentation of a container and **any one of them cannot be placed unambiguously**, the container falls back to a presentation in which *all* of them can be placed. Guessing, dropping the unplaceable ones, or rendering a partial set is prohibited: a viewer cannot detect what is missing, so partial placement is silent data loss wearing the appearance of success. The preferred presentation returns as soon as the mapping becomes resolvable.

- **SA-8 (Presentation-independent — resolvable by a consumer that never saw the producer):** an address MUST be resolvable using only the artifact and the grammar, with no access to the presentation that minted it. This is what makes one address work in a terminal, a graphical view, a link, a log line, and an agent instruction; an address requiring the producing renderer is a private coordinate with a public appearance.

- **SA-9 (An address is opaque to whoever carries it):** an actor that receives an address **copies it verbatim** and never reconstructs, abbreviates, or beautifies it; the resolving side treats a look-alike as **invalid rather than approximately correct**. Fuzzy matching on addresses converts a transcription error into a confident wrong answer — the exact failure mode SA-2 removed from the coordinates and must not reintroduce at the transport. (The addressing case of ATE-11.)

- **SA-10 (One address, every surface — non-display is stated, never silent):** the same address denotes the same target in **every** surface that can present it; a surface that cannot present the target says so explicitly rather than resolving to something adjacent or ignoring the request. Cross-surface addressability is the property that lets a decision, a link, an instruction, and a log line all refer to one place, and it is destroyed by a single surface that interprets the grammar for itself.

## 4. Detailed Design

### 4.1 Address shape

An address is a small record, not a string convention that happens to parse:

| Element | Role | Example unit vocabularies |
| --- | --- | --- |
| **container key** | durable identity of the thing addressed into | file key, document id, card id, graph entity id |
| **dimension** | which side/axis, where the artifact has more than one | old/new side, revision, language variant |
| **ordinal** | one-based position in the artifact's own units | line, paragraph, step, column position |
| **content identity** | what the address was minted against (SA-4) | digest of the addressed unit's content |
| **minted-at identity** | the container state it was resolved against (SA-4) | source/version identity of the container |

The serialized form is a single token produced and consumed by one implementation (SA-1), opaque to everyone else (SA-9).

### 4.2 The resolution ladder

```
address + current artifact
   │
   ├── container key resolves? ── no ──▶ ORPHANED (retained, disclosed)   [SA-5]
   │                                     (never redirected to a neighbour) [SA-6]
   └── yes
        │
        ├── ordinal in range? ── no ──▶ clamp into the container's range   [SA-6]
        │
        ├── content identity matches at that ordinal? ── yes ──▶ RESOLVED
        │
        └── no ──▶ search the container for the minted content identity
                     ├── found, unique  ──▶ RELOCATED (evidence: the match) [SA-5]
                     └── not found / not unique ──▶ ORPHANED (retained)
```

The ladder is deliberately conservative at the bottom: an ambiguous match is an orphan, not a coin flip. An orphan is a visible, recoverable state; a wrong relocation is an invisible false statement.

### 4.3 Why placement is all-or-nothing (SA-7)

Consider a container that can be shown two ways, and a set of five addresses bound to it. Four place cleanly in the preferred presentation; one does not.

| Strategy | What the viewer sees | What the viewer can detect |
| --- | --- | --- |
| Place four, drop one | four annotations | **nothing** — the set looks complete |
| Place four, guess the fifth | five annotations, one wrong | nothing — the wrong one looks authored |
| Fall back to the presentation where all five place | five annotations, plainer view | the presentation changed, and why |

Only the third leaves the viewer with a true picture. The cost is a less pleasant rendering in a rare case; the alternative cost is that no rendering can be trusted, because none of them announce when they are incomplete. The fallback is temporary by construction: the preferred presentation returns the moment the mapping resolves.

### 4.4 Relocation evidence and the coincidence trap

Relocation without evidence is indistinguishable from coincidence. If an address is repaired by "find the nearest unit whose content matches", and the container has three identical units, the repair is a guess that will present itself as a fact. SA-4's minted content identity and SA-5's uniqueness requirement close this: a match that is not unique produces an orphan, and a match that is unique carries the evidence that justified it, so a reader or a later check can disagree.

This is also why an orphan must be retained rather than deleted (SA-5). The content that lost its anchor is frequently the most valuable content in the artifact — someone wrote it deliberately about something that has since moved. Retention plus disclosure lets a human or an agent re-anchor it; deletion makes that impossible and leaves no trace that it was ever there.

### 4.5 Nodus relevance

**No new language invariant.** An address is an ordinary typed value (NL-7's `Text` or `Map`) crossing a step boundary through NL-9's typed I/O contract, and the grammar itself is host-supplied — a portable workflow language must not learn the unit vocabularies of a host's artifact kinds.

Two existing invariants carry the weight, and one alignment is worth recording:

- **NL-8 (reserved variable namespace)** and **NL-10 (sequential pipeline)** already forbid the language's own analogue of the defect: steps are referred to by declared name and pipeline target, never by ordinal position in the source. A workflow addressing "the third step" would break under any edit, which is SA-2 stated in the language's own terms.
- **NL-9's typed I/O** is where an address arrives and leaves; SA-9's opacity means a workflow **passes an address through** rather than composing one from parts, and a host that accepted a workflow-assembled address would be reintroducing the restatement SA-1 forbids.

## 5. Implementation Notes

- Mint the address at the moment the target is *identified*, not at the moment it is *used*. An address assembled later from whatever is on hand is exactly the convenient-coordinate defect arriving through the back door.
- Make the ternary outcome (SA-5) impossible to ignore in the type: a resolution that can return "the target" flattens relocated and resolved into one value and loses the evidence, and callers will forget the orphan case entirely.
- Test the coincidence trap directly: a container with repeated identical units, an address into one of them, an edit that removes it. The correct answer is an orphan, and the naive implementation confidently returns the wrong sibling.
- Pin the clamp/refuse boundary (SA-6) with adversarial fixtures on both sides. It is the rule most likely to be "simplified" into uniform clamping by someone who has only seen the in-range case.
- Where a legacy surface already mints its own coordinates, convert at the boundary and record it as a parity finding ([l1-surface-parity.md](l1-surface-parity.md) SP-3) rather than tolerating a second grammar indefinitely.

## 6. Drawbacks & Alternatives

- **Addresses are heavier than indices.** They carry identity as well as position, and they must be stored and transported. The weight is the cost of surviving the next edit; an index is cheaper only until the first one.
- **Relocation is a heuristic wearing a contract.** SA-5's evidence requirement and the uniqueness rule bound it, but a unique content match can still be a coincidence in a small artifact. The mitigation is conservatism, not certainty.
- **Alternative — immutable artifacts, so addresses never move:** rejected. It is available only where the artifact is genuinely append-only, and it makes the anchoring problem someone else's by making the artifact useless for the mutable cases that dominate.
- **Alternative — re-anchor by asking a model where the note now belongs:** rejected as the primary mechanism. It converts a cheap deterministic resolution into a generation, and it produces confident relocation without evidence — the exact failure SA-4 exists to prevent. It remains legitimate as an explicit, disclosed repair offered on an *orphan*, after the deterministic ladder has declined.
- **Alternative — let each surface own its own addressing and translate at the seams:** rejected by SA-1. Translation between N grammars is N² agreements to maintain, and every one of them fails by misplacing content rather than by erroring.
- **All-or-nothing placement will occasionally show a plainer view than the user wanted.** Accepted deliberately (§4.3): the alternative is a view that cannot announce its own incompleteness. <!-- TBD: whether the fallback should surface *why* it fell back inline, or only on request — the disclosure is required, the placement of the disclosure is not yet decided -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[STEERING]` | `.design/main/specifications/l1-attention-steering.md` | The steering acts that target with these addresses |
| `[PARITY]` | `.design/main/specifications/l1-surface-parity.md` | SP-2 — why a re-derived address is a parity defect too |
| `[CODEINTEL]` | `.design/main/specifications/l1-code-intelligence.md` | CI-10/CI-3 — anchoring and staleness for one artifact kind |
| `[DOCS]` | `.design/main/specifications/l1-document-understanding.md` | Where positional anchors are minted |
| `[MERGE]` | `.design/main/specifications/l1-change-merge.md` | CM-10 self-mis-anchoring, the producing side of SA-5 |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-13 | Core Team | Initial concept: **naming a place so the name survives the artifact, the surface, and the process**. One grammar with one serializer and one parser, because every restatement at a boundary is an independent chance to disagree and the disagreements misplace content instead of erroring (SA-1); **semantic keys only** — no indices, rendered rows, coordinates, or runtime handles in an address that outlives its call (SA-2); **exactly one target**, with ambiguity refused rather than resolved by precedence, so a caller's mistake is one corrected call instead of silently productive (SA-3); an address **records what it was minted against**, without which relocation and coincidence are indistinguishable (SA-4); resolution is **ternary — resolved / relocated-with-evidence / orphaned** — and an orphan is retained and disclosed because authored content that lost its anchor is still authored content (SA-5); **clamp inside a resolved container, refuse an unresolved one**, since clamping across containers is how content lands in the wrong file (SA-6); **placement is all-or-nothing per container** — partial placement is silent data loss wearing the appearance of success, so the container falls back to a presentation where every address places (SA-7); addresses are **presentation-independent**, resolvable by a consumer that never saw the producer (SA-8); an address is **opaque to whoever carries it** and a look-alike is invalid rather than approximately correct (SA-9); **one address, every surface**, with non-display stated rather than silently resolved to something adjacent (SA-10). §4.2 gives the resolution ladder, §4.3 the placement argument, §4.4 the coincidence trap; §4.5 records the nodus disposition — no new invariant, NL-8/NL-10 already forbid ordinal addressing inside the language and NL-9 passes host addresses through opaquely. |

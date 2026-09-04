# Diagram Authoring

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

A diagram is an argument drawn in space. When a system generates one — a component map, a request sequence, a pipeline, a state machine, a process flow — it is not producing a picture of data it already has; it is **choosing what a reader will understand**, and every choice it makes silently (which path dominates, which relationship gets a label, which colour means something, what the export keeps) is a claim the reader will believe without being able to check it.

The corpus already owns the neighbouring layers. A live projection of office state is a **view of a running system**, refreshed and clickable. A design identity owns **tokens and craft conformance**. Host-native rendering owns **one authored document rendered into several hosts**. None of them owns the object in the middle: a **generated explanatory diagram, authored once, exported as an artifact of record, and read by someone who was not present when it was made.**

That reader cannot ask a follow-up question. They cannot hover. Frequently they see a cropped export pasted into a review, a slide, or a message thread, detached from every surface that would have explained its scope. This is what makes a diagram's failure modes specific: **they are all silent, and they all look like competence.** A dense map reads as thorough. A label deleted to fix a collision reads as a relationship that needed no explanation. A card saying "retry" reads as a recovery path that exists. A cropped subgraph reads as the whole system. Nothing in the artifact says otherwise, and the artifact is what travels.

## Related Specifications

- [l1-office-visualization.md](l1-office-visualization.md) — OVZ owns the **live projection of a running office** (OVZ-1 projection-not-source, OVZ-2 live, OVZ-8 drill-down). This spec owns an **authored, exported artifact** whose reader has no engine behind it. Where a projection can answer a question by refreshing, an artifact must have answered it before it was saved. The two share the projection-not-source instinct and nothing else.
- [l1-automation-canvas.md](l1-automation-canvas.md) — AC is one specific live canvas over the pipeline engine, with its own fidelity and staleness rules (AC-1, AC-6). A diagram authored *about* an automation is this spec's subject; the canvas *of* the running automation is AC's.
- [l1-design-identity.md](l1-design-identity.md) — DI owns the **token contract and craft bar** (DI-3 tokens as the single source of visual truth, DI-6 distinctiveness over default). DGM-2 consumes that contract and adds the rule tokens alone cannot state: which channels carry *meaning* in this artifact, and therefore which channels decoration may not spend.
- [l1-host-native-rendering.md](l1-host-native-rendering.md) — HNR-1's one-authored-source/derived-renditions and HNR-4's honest-loss are the general form; DGM-7 and DGM-8 are what they mean when the rendition is a **static export of an interactive surface** and the loss is *runtime state* rather than *host capability*.
- [l1-computed-grounding.md](l1-computed-grounding.md) — CGR-1 (compute the answer, generate the wording) is the parent of DGM-4: a legend, a count, or a receipt inside a diagram is a computed set that the artifact may only phrase, never assert independently.
- [l1-code-intelligence.md](l1-code-intelligence.md) — where a diagram is derived from repository evidence, CI owns what the extraction may claim; DGM-9 owns the **vocabulary** the diagram is allowed to use when reporting a traversal of it.
- [l1-negative-specification.md](l1-negative-specification.md) — NEG owns stating what a design must *not* become. A diagram's anti-references (a themed beautifier, a drawing suite, a motion demo) are authored under NEG; DGM-1's node budget and DGM-2's channel rule are the enforceable residue.
- [l1-acceptance-oracle.md](l1-acceptance-oracle.md) — AO-12's counterfeit rule governs the *check* that a diagram fits its viewport; this spec governs what the diagram must be once it fits.
- [l1-evidence-currency.md](l1-evidence-currency.md) — the evidence discipline that binds a delivered diagram to the specification that was validated.

## 1. Motivation

Left unspecified, generation converges on the same failures, and every one of them produces an artifact that looks finished:

- **The complete map.** Everything the model knows becomes a node, because omission feels like inaccuracy. The reader cannot find the primary path, so they read the diagram as "complicated" and take nothing from it.
- **The label deleted to fix a collision.** Two labels overlapped; one was removed. The relationship is still drawn, so nothing looks missing — and the protocol, direction, or cross-boundary mechanism that label carried is now unrecoverable from the artifact.
- **The annotation standing in for topology.** A card reads "retries on failure" and no edge returns to the active state. The claim is in the picture and absent from the model, so every derived query — reachability, path count, validity — disagrees with what the reader was told.
- **The legend that describes what is not there.** Written by hand to look complete, it lists categories the diagram contains no instance of, and readers spend attention looking for them.
- **The cropped view that travels as the whole.** A subgraph exported for one question is pasted somewhere else. Nothing in the image says it was scoped, so it is read as the system.
- **The export that needs the viewer.** Meaning was carried by a hover, a focus glow, or an animation, and the still frame that reaches the reader is missing the half that explained it.
- **The traversal reported as impact.** A walk over authored edges is captured as "blast radius" or "what breaks", and a reader makes a deployment decision on a claim about the world that was only ever a claim about the drawing.
- **The counterfeit fit.** The measurable check said the content must not overflow; the content was clipped, scrolled, or shrunk until it did not. The check passes and the artifact is worse than before it ran.

## 2. Constraints & Assumptions

- **The artifact outlives its context.** It is exported, pasted, and cropped. Any qualification that lives outside the image is lost on the first re-share.
- **The reader cannot interrogate it.** There is no engine, no hover, no follow-up. What is not in the frame does not exist for them.
- **Visual channels are scarce and interfering.** Position, colour, shape, weight, and motion are a small fixed set; anything decorative that uses one has taken it from meaning.
- **Layout constraints are real and will conflict with content.** Labels collide, nodes overlap, viewports overflow. Something must give, and the order in which things give is a design decision, not an implementation detail.
- **Generation is cheaper than deliberation.** For a spatial artifact, producing a candidate and measuring it is faster and more accurate than reasoning about coordinates in prose.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **DGM-1 (One primary narrative, bounded, legible before anything secondary):** a diagram carries **one main path** that a reader can follow without instruction, and every secondary relationship, branch, or detail is subordinate to it in position, weight, or disclosure. The primary set is **explicitly bounded** — a declared budget of primary elements, not "everything known" — and a candidate exceeding it is reduced by **removing low-value relationships**, never by shrinking the elements until they fit. Completeness is not the goal and is not a defence: an artifact whose main path cannot be found has communicated nothing, however accurate every node in it is.

- **DGM-2 (Meaning-bearing channels are a closed vocabulary; decoration never spends one):** the artifact declares which visual channels carry semantics — typically colour category, node shape, edge style, and weight — and each is a **closed, documented mapping**. Nothing decorative may consume a semantic channel: an accent added to make a surface lively, an identity mark that recolours the element it sits on, a highlight that means nothing. Where an identity or brand mark is present it is **confined to its own plate**, keeps its colour inside that plate, and **never replaces** the element's semantic type, label, or relationships. Identity is authored as an explicit fact and is **never inferred from label text** — a match on a name that changes what an element appears to be is a semantic change made by a string comparison.

- **DGM-3 (A claimed relation lives in the topology, never only in an annotation):** if the diagram asserts that two things are related — a retry returns to an active state, a service calls another, a stage feeds a consumer — that relation exists **in the model as a relation**. A card, caption, legend line, or note asserting it is **decoration, not structure**: it is invisible to every derived query, survives no transformation, and disagrees with the model the moment either changes. A grouping construct (a boundary, a lane, a container) expresses containment and **does not stand in for a relationship** between the things it contains.

- **DGM-4 (Derived summaries are computed from the model, never authored beside it):** legends, counts, receipts, path summaries, and category lists are **computed from the artifact's own model**, so what they claim and what the diagram contains cannot diverge. A hand-authored summary is admitted only as a **wording override on a computed entry** — it may change how a category is named, never whether it is present. A summary MUST NOT be used to compensate for something the model lacks: listing a category with no instance, or naming a flow that no edge carries, is the artifact asserting content it does not have.

- **DGM-5 (Labels are content, and the repair precedence yields geometry before meaning):** relationship labels, node names, and stage descriptions are **semantic data, not decoration to be reclaimed under pressure**. When a layout constraint conflicts with content, the declared repair order gives way in this sequence: **move the element, adjust the route or spacing, shorten the wording while preserving meaning — and only then consider removing wording**. Deleting a label is **not a layout repair**, and an implementation that treats it as the cheapest fix will reliably strip exactly the protocol, direction, synchrony, and cross-boundary facts a reader cannot reconstruct.

- **DGM-6 (Removing meaning is a declared semantic decision, distinguishable from never having had it):** where content is genuinely omitted — a label whose wording is fully implied by both endpoints, a detail moved to disclosure — the omission is **recorded as an authoring decision with its reason**, because the artifact's end state is identical to the state produced by a silent geometry repair and a reader cannot tell them apart. The two routes to "no label here" mean opposite things: one says *this needs no explanation*, the other says *an explanation was discarded to make the picture fit*.

- **DGM-7 (The canonical export is complete without the runtime, and carries none of its state):** whatever the artifact means, it means **in a still frame with no interaction**. Motion, focus, hover, search state, camera position, and transient overlays are **reader affordances that add no meaning of their own**, and they are **stripped from every canonical export** rather than baked into it. Two rules follow: motion is finite, reader-controlled, and never the sole carrier of a fact; and a rendition produced from a live surface removes the live surface's marks instead of freezing them into the record (the export-direction form of AST-10's live-view-is-not-the-record).

- **DGM-8 (A scoped view names its scope inside the artifact, for a reader and for a machine):** an intentionally partial rendition — one path, one subgraph, one query's result, one crop — **declares its scope within the artifact itself**: human-visible in the frame, and machine-readable in its accompanying record. The declaration names what selected it (the origin, the direction, the filter) and the size of what it shows (element and relationship counts, depth). Scope stated only in surrounding prose is lost on the first re-share, which is the normal fate of an exported image, and a scoped view that does not say so **is read as canonical** — the coverage-travels-with-the-number discipline applied to an artifact that travels alone.

- **DGM-9 (The vocabulary of a derived query never overstates what it measured):** a traversal of authored relationships reports itself in the **terms of the graph it walked** — elements, relationships, direction, depth. It MUST NOT be labelled with terms that assert facts about the running world — impact, blast radius, breakage, causality, dependency risk — unless independent evidence of that kind was actually produced. The graph is a model of what someone drew; the words *impact* and *breakage* describe what happens when a system runs, and the gap between them is where a reader makes a decision the artifact never supported (the diagram-grain form of RLV-6: evidence proves what it names and nothing wider).

- **DGM-10 (Presentation variants preserve category identity and information priority, and vary on independent axes):** themes, colour modes, and stylistic presets may change material, contrast, and atmosphere; they **MUST NOT change which category an element belongs to, which path is primary, or what is disclosed**. A variant that reorders information priority is a different diagram wearing the same name. Presentation axes that a reader can switch independently (colour mode and stylistic preset, say) stay **independent**: changing one never silently resets the other.

- **DGM-11 (An exemplar contributes shape, never facts):** where generation is seeded by an example, template, or prior artifact, the exemplar supplies **structure — field shape, arrangement, conventions — and no content**. Identifiers, wording, domain facts, and layout are authored fresh. The failure this prevents is the quiet one: an example's node names, its sample services, or its illustrative labels surviving into a delivered artifact, where they read as findings about the reader's own system.

- **DGM-12 (Generate the candidate, then diagnose it):** for a spatial artifact, the **first action is producing a candidate**, not reasoning about its geometry in prose. Coordinates, routes, and spacing are measured, not predicted, and a candidate that exists can be checked; a plan for one cannot. Deeper implementation detail — layout internals, solver behaviour, renderer source — is consulted **only when a diagnostic names it or focused repair has already failed**, never as preparation. The corollary is that the artifact's own diagnostics are the authoring interface: the engine's internals are not authoring controls, and steering an artifact by reaching into them produces a result that no later version of the engine will reproduce.

> An L2 implementation cannot reach RFC until every invariant above is addressed in its Invariant Compliance section.

## 4. Detailed Design

### 4.1 The repair precedence

The order matters more than any individual rule, because pressure arrives late and whatever is cheapest at that moment is what gets sacrificed:

```
1. structural validity      (schema, placement, required fields)
2. element collisions       (overlap, out-of-range placement)
3. relationship integrity   (an edge crossing an unrelated element; endpoint direction)
4. route quality            (crossings, ambiguous corridors, rhythm)
5. label clearance          (to elements, to other labels, to routes)

within every step:  move > re-route > re-space > re-word > (declared) omit
```

Content is last in both dimensions. A diagram that fits by having lost its labels has traded the thing it was made for against the thing that was easy to measure.

### 4.2 What travels, and what does not

| Lives in the artifact | Lives outside it |
| --- | --- |
| The topology and its labels | The conversation that produced it |
| The scope declaration (DGM-8) | The caption in the message that shared it |
| Computed legends and counts (DGM-4) | The reader's memory of what was asked |
| The semantic channel vocabulary (DGM-2) | Viewer state, focus, motion (DGM-7) |

Everything in the right column is gone by the second re-share. Any qualification the artifact needs to be read correctly belongs in the left one.

### 4.3 Failure modes named

| Mode | What the reader sees | Closed by |
| --- | --- | --- |
| The complete map | Accurate, unreadable, primary path unfindable | DGM-1 |
| Decorative accent in a semantic channel | A colour that appears to mean something | DGM-2 |
| Annotation as topology | A recovery path that exists only in a caption | DGM-3 |
| Legend describing absent content | Attention spent looking for what is not there | DGM-4 |
| Label deleted to fix a collision | A relationship that appears to need no explanation | DGM-5, DGM-6 |
| Runtime-only meaning | A still frame missing the half that explained it | DGM-7 |
| Cropped view read as canonical | A subgraph mistaken for the system | DGM-8 |
| Traversal reported as impact | A deployment decision on a claim never made | DGM-9 |
| Variant reordering priority | The "same" diagram arguing something else | DGM-10 |
| Exemplar facts in a delivered artifact | Someone else's services named as yours | DGM-11 |
| Coordinate planning in prose | Confident geometry that does not survive rendering | DGM-12 |

## 5. Implementation Notes

- DGM-2's vocabulary is best expressed as a token-level binding into the identity contract (DI-3) rather than as a per-diagram convention, so the same category means the same thing across every artifact the system produces.
- DGM-4's computation is the cheap half; the expensive half is refusing hand-authored summaries at all, including the seemingly harmless "list every category the renderer supports". That variant is admissible only as an explicit, declared reference mode — never as the default, which must show what is present.
- DGM-6's record belongs with the artifact's specification, not in its visual frame: the reader of the picture does not need it, and the next author does.
- DGM-8's machine-readable half should reuse the artifact's own receipt rather than inventing a sidecar, so a scope declaration cannot be dropped while the image survives.

## 6. Drawbacks & Alternatives

- **DGM-1's budget will sometimes exclude a true relationship.** Accepted, and it is the invariant most argued with. The excluded relationship is recoverable from the model; a reader's failure to find the main path is not recoverable at all.
- **DGM-3 makes some artifacts harder to author.** By design: an assertion cheap enough to type into a card is exactly the assertion nobody checks.
- **DGM-5's precedence sometimes leaves a diagram larger than one would like.** Held: the alternative is smaller diagrams that have quietly lost their protocols and directions.
- **DGM-7 forbids a class of genuinely delightful artifacts** where motion carries the story. Not quite: motion may carry the *story*, never the *fact*. What is forbidden is a fact that exists only while something is moving.
- **Alternative — qualify a scoped export in the surrounding message instead of in the frame:** rejected (DGM-8). The message is the first thing lost, and the image is the thing that travels.
- **Alternative — let the layout engine drop labels automatically when they do not fit:** rejected (DGM-5). It is the single most common way a generated diagram silently becomes less true than the model behind it.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[VISUALIZATION]` | `.design/main/specifications/l1-office-visualization.md` | The live-projection sibling; OVZ-1 projection-not-source |
| `[IDENTITY]` | `.design/main/specifications/l1-design-identity.md` | DI-3 token contract that DGM-2 binds its vocabulary into |
| `[RENDERING]` | `.design/main/specifications/l1-host-native-rendering.md` | HNR-1/HNR-4 — one source, derived renditions, honest loss |
| `[GROUNDING]` | `.design/main/specifications/l1-computed-grounding.md` | CGR-1 — compute the answer, generate the wording |
| `[ORACLE]` | `.design/main/specifications/l1-acceptance-oracle.md` | AO-12 — the counterfeit rule for the fit check |
| `[CURRENCY]` | `.design/main/specifications/l1-evidence-currency.md` | Binding a delivered artifact to the specification that passed |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-04 | Core Team | Initial concept — the **generated explanatory diagram as an artifact of record**, the object between a live projection (OVZ), a token contract (DI), and a host rendition (HNR), none of which owns a picture authored once and read by someone who cannot interrogate it. Mined from an external diagram-generation skill whose entire discipline is organized around the fact that a diagram's failures are silent and look like competence. Twelve invariants: one bounded primary narrative, reduced by removing relationships rather than shrinking elements (DGM-1); a closed semantic-channel vocabulary that decoration may not spend, with identity marks confined to their own plate and **never inferred from label text** (DGM-2); **a claimed relation lives in the topology, never only in an annotation** — a card saying "retry" is not a transition, and a boundary is not a relationship (DGM-3); legends, counts, and receipts computed from the model, hand authorship admitted only as a wording override on a computed entry (DGM-4); labels as content with a declared repair precedence — move, re-route, re-space, re-word, and only then omit — since deleting a label is not a layout repair (DGM-5); an omission recorded as a semantic decision because its end state is identical to a silent geometry repair and a reader cannot tell them apart (DGM-6); the canonical export complete without the runtime and stripped of its state (DGM-7); a scoped view naming its scope **inside** the artifact, human-visible and machine-readable, because the qualifying prose is lost on the first re-share (DGM-8); a traversal reported in the terms of the graph it walked and never as impact, blast radius, or breakage (DGM-9); presentation variants preserving category identity and information priority on independent axes (DGM-10); an exemplar contributing shape and never facts (DGM-11); and generate-then-diagnose, with engine internals explicitly not authoring controls (DGM-12). Concept-only; no L2 yet. |

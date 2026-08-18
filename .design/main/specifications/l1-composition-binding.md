# Composition Binding

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The model of **how a named composition binds to running actors, and what happens when the composition changes underneath them**. A *composition* is a named, discoverable declaration of an actor's world — which capabilities it holds, which persona it wears, which norms it obeys. [l1-roles.md](l1-roles.md) and [l1-office-archetype.md](l1-office-archetype.md) own the catalog such declarations live in; [l1-scoped-capability-layers.md](l1-scoped-capability-layers.md) owns what a binding means for visibility. This spec owns the binding itself: mounting, joining, generation identity, rebinding, discovery, and authoring.

The catalog side is well understood — preset versus custom, copy rather than fork, one active archetype. The runtime side is not, and it is where the failures live. A composition is a **file** that a human edits while sessions are running. The moment it becomes a live binding, four questions appear that no catalog rule answers: does a running actor follow the edit or keep what it started with; does a child re-resolve its parent's composition by name or inherit the exact one its parent is running; when a session is reconstructed from its record, which composition does it rebuild under; and what does the system do with a composition that is present on disk but cannot be loaded.

The organizing answer is a **generation**: the binding names not a composition but a *specific loaded state of it*. Actors join a generation; the generation outlives its source file changing or disappearing; and every reconstruction resolves the generation the actor actually ran, never the one its creation record names.

## Related Specifications

- [l1-scoped-capability-layers.md](l1-scoped-capability-layers.md) - The visibility half of this pair. A standing mount holds a scope (SCL-3), and joining a composition is parenting an actor's scope to that mount's; SCL-8's setup window is where CBD-1's join is installed.
- [l1-roles.md](l1-roles.md) - ROL-2/ROL-6/ROL-7 — the catalog, the composition contract, and preset integrity. This spec binds what that catalogs.
- [l1-office-archetype.md](l1-office-archetype.md) - OA-6/OA-8/OA-10 — preset+custom with catalog integrity, one active archetype revisable non-destructively, compose-never-fork. CBD-6 supplies the runtime condition under which OA-8's revision is safe.
- [l1-declarative-configuration.md](l1-declarative-configuration.md) - DC-5's single/template/instance kinds and DC-8's stable addressing; a composition is a *template*, a binding produces an *instance*, and CBD-11 hardens DC-8's identity discipline into a containment property.
- [l1-composition-layering.md](l1-composition-layering.md) - How a composition's content is assembled from layers before it is bound; that spec ends where this one begins.
- [l1-context-provenance.md](l1-context-provenance.md) - A composition's personas and norms are untrusted content that becomes agent instruction; CBD-10's copy-only authoring is what keeps a locally-authored composition from carrying text nobody vetted.
- [l1-conversation-rewind.md](l1-conversation-rewind.md) / [l1-crash-recovery.md](l1-crash-recovery.md) - Fork and resume are the reconstruction paths CBD-4 governs; both must resolve rather than replay a creation header.
- [l1-extension-marketplace.md](l1-extension-marketplace.md) - Distribution of compositions; CBD-8's two dispositions are what a delivered-but-unloadable artifact must produce.

## 1. Motivation

Four failures, each invisible until it has already corrupted a record.

**The edit that reaches a running actor.** A human edits an office's composition to add a tool. Three sessions are mid-conversation. If the edit reaches them, each has a history whose earlier turns were produced under a different capability set than the one now described — and the record no longer explains the behavior it contains. If the edit reaches none of them ever, the file has stopped being the way compositions are changed. The only coherent answer is a versioned binding: running actors keep what they joined, new ones get the new state, and both facts are legible.

**The child that re-resolves.** A sub-agent composed by name rather than by reference will, if the file changed since its parent started, run a **different** composition than the parent whose work it is continuing — and will fail outright if the composition was deleted, while its parent runs on undisturbed. Both are silent: nothing in the delegation record says the child ran a different world.

**The record that rebuilds wrong.** A session's creation record names the composition it *started with*. If the session later switched, every reconstruction that reads that record — a resume, a fork, a transcript summary, an evaluation replay — rebuilds it under a composition whose capabilities cannot account for the tool calls in its own history.

**The composition that is present but broken.** A malformed composition that discovery *skips* still occupies its identity in its root while every surface shows nothing to remove. The user sees an unusable name they cannot delete and an empty list that claims nothing is wrong.

## 2. Constraints & Assumptions

- A composition is **discovered**, not registered: it is found by scanning declared roots, so it may appear, change, or vanish at any moment without the system being told.
- Roots carry **trust** (shipped versus locally authored), and a composition inherits the trust of the root it was found under; trust is never self-declared.
- The composition's content model is out of scope — this spec governs the binding regardless of whether the content is a role definition, an archetype, a capability list, or a profile.
- Actors are cheap and numerous; compositions are few and shared. The model optimizes for many actors on one composition, not for one actor per composition.
- Every rule here is about **binding**, never about permission. What a composition may grant is settled by [l1-security.md](l1-security.md) and [l1-extensions.md](l1-extensions.md); binding cannot widen it.

## 3. Core Invariants (Layer 1 only)

Rules every Layer 2 implementation MUST NOT violate:

- **CBD-1 (One standing mount, many joiners):** a composition is mounted **once** and **joined** by every actor that names it. Its contributions exist exactly once and cover every joined actor; per-actor state is keyed by actor inside the shared mount, never by duplicating the mount. Mounting a composition per actor is forbidden: it multiplies identical registrations by the actor count and makes "what does this composition contribute" a question about N copies.

- **CBD-2 (A binding names a generation, and a joined generation outlives its source):** what an actor binds to is not a composition but a **generation** — one identified loaded state of it, stamped with the source state it was loaded from. An actor keeps the generation it joined for its whole life. The source changing, being replaced, or being **deleted** does not disturb an actor already running on it; the next actor that finds the stamp stale starts the next generation. A design in which a running actor's world follows its file has made every in-flight history unexplainable by its own record.

- **CBD-3 (A child joins by reference to the parent's generation, never by re-resolving the name):** an actor composed beneath another joins the **exact generation its parent is running**, by reference. Re-resolving the composition by name is forbidden, and for two independent reasons: a source edited since the parent started would hand the child a different generation than the one its parent's history was produced under, and a source deleted since would fail the child outright while its parent keeps running. The bind is also the only form available inside a synchronous creation window, which is where children are composed.

- **CBD-4 (Started-with is a frozen creation fact; runs-on is resolved):** the creation record names the composition an actor **started with** and is frozen, because it is a fact about creation. Which composition an actor **runs on** is a separate, resolved answer. Every reconstruction path — resume, fork, summary, replay, evaluation — **resolves**; none reads the creation record as the answer. Reading the header alone rebuilds a switched actor under a composition that cannot account for its own recorded actions.

- **CBD-5 (A change of binding is recorded after it commits):** rebinding an actor to a different composition is a **durable, appended event**, written after the swap commits, never a silent mutation and never a rewrite of the creation record. The composition determines what the model sees, so it falls under the model-visible-means-recorded rule: anything that shaped a request must be reconstructable from the record ([l1-context-provenance.md](l1-context-provenance.md), [l1-diagnostic-log.md](l1-diagnostic-log.md)).

- **CBD-6 (Rebinding only while nothing has been produced, checked where history is in hand):** an actor may be rebound only while it has **produced nothing** — no recorded action, no model-visible output. This is a product rule, not a mechanical one: swapping an actor's capabilities mid-work leaves recorded actions the new composition cannot make, and a record whose steps are unreproducible under its own stated world. The check is enforced at the boundary that **holds the history**, not at the one that happens to receive the request, and its refusal names the reason rather than reporting a generic failure.

- **CBD-7 (A rebind is atomic and restoring; two compositions never coexist):** rebinding **releases the old and installs the new** — the two are never mounted simultaneously, because both would claim the same names in one layer. The replacement is prepared and validated **before** the old is released; a failure restores the previous composition rather than leaving the actor with none, and an unknown target is refused before anything is torn down.

- **CBD-8 (Two dispositions for an unusable composition — skip only what could never be claimed):** an artifact whose **identity** is unusable (it could never be a valid name) is skipped outright, because nothing could ever claim it. An artifact whose identity is valid but whose **content** cannot be loaded is **listed with its reason**, never skipped: a skipped artifact still occupies its identity in its root while every surface shows nothing to remove, so the user faces a name they cannot use and cannot delete. The reason is shown verbatim on the surface that lists it, and every operation that does not require loading — reading it, reporting it, deleting it — remains available on a broken entry.

- **CBD-9 (Discovery re-reads; it is never memoized):** the set of available compositions is re-read from its roots on every query. A composition authored while the system runs is visible on the next read, and a deleted one disappears from it. Caching discovery makes an externally-authored composition invisible until a restart — and since files are the authoring surface, that is the same as saying authoring does not work.

- **CBD-10 (Authoring is copy-only; a copy is exactly as loadable as its source):** a new locally-authored composition is produced by **copying an existing one whole**; no caller supplies composition content through the authoring interface. Two properties follow and are the reason for the rule: a copy is exactly as loadable as its source, so authoring cannot mint a broken artifact; and a copy grants nothing the catalog did not already carry, so authoring cannot mint capability. Everything after creation happens by editing the copy's own files, under the ordinary vetting rules for content that becomes agent instruction. The copy **drops the fields that distinguish it from its source** (its display name and its ordering position) and keeps the editable description — a copy presenting itself identically to its source makes the catalog stop distinguishing them.

- **CBD-11 (Containment is a property of the identity, not of a later path check):** where an identity becomes a storage location, the **identity itself** is constrained to a form that cannot escape — traversal segments, separators, and absolute forms are rejected **as identities**, before anything is created. A path check applied after an identity is accepted is a second decision point for one property, and it is the one that gets forgotten on the next code path that builds a location from the same identity.

- **CBD-12 (Display metadata can never claim identity or trust):** a composition's human-facing metadata (title, description, presentation) is separate from its identity and its trust, and neither may be written there. Identity comes from where the artifact lives; trust comes from the root it was discovered under. A locally-authored composition able to declare its own identity or trust could name itself into the shipped set, which is a supply-chain claim made by the artifact about itself — exactly what [l1-attestation.md](l1-attestation.md) exists to refuse.

- **CBD-13 (Resolution anchors are declared per reference kind):** a composition references components, and where each kind of reference resolves **from** is declared, not inherited from wherever the composition file happens to sit. Shared-component names resolve against the **host installation**, so a composition authored anywhere can reference what the product ships; composition-relative names resolve against the composition's **own** location, so its private parts travel with it; absolute references keep their own location. Without the declaration, a composition authored outside the installation's tree cannot reference a single shipped component, and the failure appears as a load error naming an unrelated resolution mechanism.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 Mount, join, generation

```plaintext
composition "reviewer"  (source: catalog root, trust: shipped)
   │
   ├── generation g1   stamp: <source state at load>          ← standing mount, holds a scope
   │      ├── actor A   joined g1     (running)
   │      └── actor B   joined g1     (running)
   │            └── child B'  joined g1 BY REFERENCE (CBD-3), never re-resolved
   │
   └── [source edited] ─────────────────────────────────────►
          generation g2   stamp: <new source state>
                 └── actor C   joined g2     (created after the edit)

A and B keep g1 for life (CBD-2). Deleting the source now removes nothing from A, B, or B'.
```

The stamp is whatever cheaply identifies the source's state; its only requirement is that a change to the source produces a different stamp. A joiner compares the stamp, not the content: an unchanged source costs one comparison, a changed one costs one load.

The join is installed inside the actor's setup window ([l1-scoped-capability-layers.md](l1-scoped-capability-layers.md) SCL-8), which is what makes CBD-7's rollback total — a rejected composition rolls the whole actor creation back rather than leaving a half-composed actor addressable.

### 4.2 Started-with versus runs-on

| Question | Answer source | Why |
| --- | --- | --- |
| What did this actor start with? | the frozen creation record | A creation fact. It never changes, because what happened at creation never changes. |
| What is this actor running on? | resolved: the last recorded rebind, else the creation record | The only answer that accounts for the actor's whole recorded history. |

Every reconstruction path asks the second question. The temptation to read the first is strong because it is one field and always present — which is exactly why CBD-4 states the rule rather than leaving it to each path's author. A summary, a resume, a fork, a replay, and an evaluation are five independent code paths, and it takes only one of them reading the header to produce a reconstruction that cannot execute its own history.

### 4.3 The two dispositions for an unusable artifact

```plaintext
artifact found in a root
   │
   ├── identity is not a well-formed name  ──► SKIP silently
   │        (nothing could ever claim it; it is not a broken composition,
   │         it is not a composition)
   │
   └── identity is well-formed
            │
            ├── content loads      ──► LIST as usable
            │
            └── content fails      ──► LIST as broken, with the reason verbatim
                                        · read / report / delete remain available
                                        · every load-requiring operation refuses
                                          up-front with the same reason
```

The asymmetry is the whole point. Skipping a well-formed identity hides a name that is *occupied* — the user cannot use it and cannot free it, and the surface that would tell them so shows an empty row. Listing it with its reason puts the problem and its remedy on the same page.

CBD-8's second half matters as much as the first: refusing a broken composition **up front, with the discovery-reported reason**, means every path fails identically. Otherwise one path refuses early with a clear reason, another gets deep into loading and fails with a parser message, and the same artifact appears to have two different problems.

### 4.4 Authoring by copy

Copy-only authoring (CBD-10) is a narrow interface with three refusals, and each refusal closes a different hole:

| Refusal | Hole closed |
| --- | --- |
| Identity not well-formed | Containment (CBD-11) — the identity becomes a location, so it is constrained as an identity. |
| Identity already taken in **any** root | A local artifact named like a shipped one would be permanently shadowed by it; a directory already occupying the name would be listed as broken (CBD-8), so the refusal's remedy is on the page that reported it. |
| Source does not exist | A half-made artifact that discovery cannot see; a failed copy rolls its partial state back. |

The copy is self-contained (links dereferenced, permissions tightened to the owner) so that a copy of a shipped composition is a *thing the user owns*, not a view onto something the next product update will replace.

### 4.5 What this does not decide

Binding is not granting. A composition names capabilities; whether an actor may hold them is settled by the authority plane before any of them becomes usable ([l1-extensions.md](l1-extensions.md) EXT-3, [l1-security.md](l1-security.md)). A composition's prose — personas, briefs, norms — is untrusted content that becomes agent instruction and is vetted as such ([l1-office-archetype.md](l1-office-archetype.md) OA-5, [l1-component-scanning.md](l1-component-scanning.md)). CBD-10's copy-only rule reduces the surface those planes must cover; it does not replace them.

## 5. Drawbacks & Alternatives

- **Generations accumulate.** Every edit to a live composition can leave an older generation held by running actors. This is the cost of CBD-2 and it is bounded by actor lifetime, not by edit count — a generation is released when its last joiner ends. The alternative, one live version, buys memory and pays with histories that cannot explain themselves.
- **CBD-6 blocks a wanted operation.** "Change this agent's role mid-conversation" is a reasonable request and this spec refuses it. The refusal is narrow — an actor that has produced nothing may switch freely — and the honest alternative is to start a new actor, which is what the user meant anyway when the conversation so far was produced under a role they no longer want.
- **Copy-only authoring is a coarse interface.** A user who wants a composition with three fields changed must copy and edit. The alternative — an authoring interface that accepts composition content — reintroduces exactly the two properties CBD-10 buys: it can mint an unloadable artifact, and it can mint capability the catalog never carried.
- **Rejected — mount per actor.** Uniform and simple, and it multiplies identical registrations by the staff count while making a composition's contribution unenumerable without enumerating its joiners.
- **Rejected — a running actor follows its file.** The most intuitive behavior and the one that makes every in-flight record unexplainable. It also cannot answer what happens when the file is deleted.

## nodus-relevance mapping

A workflow definition is a composition and a run is an actor, so the same four failures apply at the DSL grain — and nodus's existing seams already hold most of what is needed.

| Element | nodus seam | Note |
| --- | --- | --- |
| Generation binding (CBD-2) | validate-before-run stage, LP-8 capability manifest | A run's satisfiability is decided once, against one definition state; the run must keep that state even if the definition file is edited or removed while it executes. This is the one gap worth carrying into nodus explicitly. |
| Child joins by reference (CBD-3) | nested/imported workflow invocation (LP-13) | A nested run inherits the parent's resolved bundle rather than re-resolving the name, or a mid-run publish silently changes what the nested step executes. |
| Started-with vs runs-on (CBD-4) | paused-run resume descriptor (LP-15) | A resume descriptor names the definition the run *is executing*, not the one it was launched from. |
| Two dispositions (CBD-8) | LP-12 imported-bundle admission vetting | A bundle that fails vetting is reported with its reason and stays addressable for removal; a malformed identity is not a bundle at all. |
| Resolution anchors (CBD-13) | LP-13 addressable versioned import resolution | Already declared: a named import resolves by the host's rules, a relative one against the bundle. |
| Copy-only authoring (CBD-10) | host-side bundle authoring | Nodus supplies the seam, the host owns the policy; the invariant belongs on the host side. |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[SCOPES]` | `.design/main/specifications/l1-scoped-capability-layers.md` | The visibility half; SCL-8's setup window is where a join is installed. |
| `[ROLES]` | `.design/main/specifications/l1-roles.md` | ROL-2/ROL-6/ROL-7 — the catalog this binds. |
| `[ARCHETYPE]` | `.design/main/specifications/l1-office-archetype.md` | OA-5/OA-6/OA-8/OA-10 — content vetting, catalog integrity, and non-destructive revision. |
| `[LAYERING]` | `.design/main/specifications/l1-composition-layering.md` | How a composition's content is assembled before it is bound. |
| `[DECLARATIVE-CONFIG]` | `.design/main/specifications/l1-declarative-configuration.md` | DC-5 template/instance kinds and DC-8 stable addressing. |
| `[PROVENANCE]` | `.design/main/specifications/l1-context-provenance.md` | Why a rebind must be recorded: model-visible means reconstructable. |
| `[NODUS-PORTABILITY]` | `.design/nodus/specifications/l1-nodus-portability.md` | LP-8/LP-13/LP-15 — the DSL-grain seams this projects onto. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-19 | Core Team | Initial spec — composition binding: the runtime half the catalog specs never covered. A composition mounts **once** and is joined by many, contributions existing exactly once (CBD-1); a binding names a **generation** stamped from the source state, and a joined generation outlives its source being edited or deleted, because an actor whose world follows its file has a history its own record cannot explain (CBD-2); a child joins its parent's exact generation **by reference**, never by re-resolving the name — re-resolution hands it a different generation after an edit and fails it outright after a delete, while its parent runs on undisturbed (CBD-3); **started-with** is a frozen creation fact and **runs-on** is resolved, with every reconstruction path resolving, since reading the header rebuilds a switched actor under a composition that cannot account for its own recorded actions (CBD-4); a rebind is a durable event appended after it commits, because the composition decides what the model sees (CBD-5); rebinding permitted only while the actor has produced nothing, checked where the history is in hand (CBD-6); rebind atomic and restoring, with the two compositions never coexisting since both would claim the same names (CBD-7); **two dispositions** for an unusable artifact — an unclaimable identity is skipped, an unloadable content is *listed with its reason*, because a skipped artifact still occupies its identity while every surface shows nothing to delete (CBD-8); discovery re-read rather than memoized, since files are the authoring surface (CBD-9); **copy-only authoring**, which cannot mint an unloadable artifact and cannot mint capability, dropping the fields that would let a copy impersonate its source (CBD-10); containment as a property of the identity rather than a later path check, a second decision point being the one that gets forgotten (CBD-11); display metadata forbidden from claiming identity or trust, which would be a supply-chain claim an artifact makes about itself (CBD-12); and per-kind declared resolution anchors, without which a composition authored outside the installation cannot reference a single shipped component (CBD-13). Distilled from an adoption pass over an external plugin-framework-based agent-harness reference. Concept-only. |

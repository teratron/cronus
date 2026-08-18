# Composition Layering

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The model of **how the set of components a system runs is assembled from ordered layers**, and how that set is changed, inspected, and reconciled while the system is live. Where [l1-declarative-configuration.md](l1-declarative-configuration.md) DC-11 resolves one **field value** from a stack of layers with per-field authority, this concept resolves a **set of components** from a stack of layers: which components exist at all, in what order, with which configuration, and which are present but not running.

The two are different operations on different objects and neither substitutes for the other. Value resolution asks *what is this setting?* and answers with a scalar. Set composition asks *what is running?* and answers with an ordered list whose entries appear, disappear, and change identity — which makes **reconciliation** its central problem: given the previous list and the next one, what must actually be started, stopped, or reconfigured.

The failure this spec exists to prevent is the quiet one: a system that restarts components nobody changed, because the composition algorithm cannot tell an edit from a removal-plus-addition; or a system whose "show me what is running" command computes its answer differently from the path that actually runs things, so the two agree until precisely the moment someone needs them to.

## Related Specifications

- [l1-declarative-configuration.md](l1-declarative-configuration.md) - The value-resolution sibling. DC-11's layered stack with per-field authority governs *what a setting is*; LAY governs *what exists*. DC-1's one-declaration rule and DC-8's stable addressing are what LAY-1's entry identity extends into the composition dimension.
- [l1-composition-binding.md](l1-composition-binding.md) - What happens once an assembled composition binds to running actors; this spec ends where that one begins.
- [l1-extension-points.md](l1-extension-points.md) - EP-4's deterministic composition and declared precedence at one point; LAY is the same discipline one level up, over the *set of contributors* rather than the contributions at a point.
- [l1-extensions.md](l1-extensions.md) - EXT-2's lifecycle; LAY-3's disabled state sits between *discovered* and *activated* and is durable, not a runtime toggle.
- [l1-extension-marketplace.md](l1-extension-marketplace.md) - Distribution; LAY-7 constrains what a distributed unit may do to a composition — contribute entries that remain overridable, never entries the consumer cannot reach.
- [l1-system-readout.md](l1-system-readout.md) / [l1-doctor.md](l1-doctor.md) - The inspection surfaces LAY-6 binds to the composition algorithm; a readout computed by a second implementation is a readout that will eventually lie.
- [l1-staged-rollout.md](l1-staged-rollout.md) - LAY-10's transactional recomposition and last-good retention is the same "never swap in place" discipline applied to configuration rather than to a rollout.
- [l1-change-attribution.md](l1-change-attribution.md) - LAY-8's ownership determination is an attribution question answered structurally rather than by a stored flag.

## 1. Motivation

A system whose components are declared rather than compiled in gains three things — a user-visible composition, a distribution format, and an override mechanism — and inherits four problems that nothing in the value-resolution model addresses.

**Reconciliation needs identity, and identity is not free.** When a declaration is re-read, the system must decide, per entry, whether it is the same component reconfigured or a different one. Without an **author-assigned** identity the only available answer is positional or content-derived, and both mean that editing one entry restarts every entry after it. The symptom is bizarre and hard to attribute: a user changes one line and their whole tree cycles.

**Override semantics are a fork in the road, and the intuitive branch is wrong.** Deep-merging an override into a base configuration feels helpful and produces configurations no layer authored, whose behavior depends on the merge algorithm's treatment of absent keys, empty collections, and nulls. Whole replacement is more verbose and is the only rule under which a reader of the top layer knows what is in effect.

**Inspection drifts from execution.** "Show me the effective composition" is implemented once as a boot path and once as a reporting path, and they agree until the day someone is debugging — which is the only day the answer matters.

**Ownership of a shipped file is unstated.** The system ships a default composition; the user edits it; the next update must decide whether to overwrite. Storing a "user modified this" flag is a second source of truth that can be wrong in both directions. The composition itself already carries the answer.

## 2. Constraints & Assumptions

- A composition is an **ordered list of entries**; each entry names a component and carries that component's configuration. The entry's own configuration is a value-resolution problem and belongs to `l1-declarative-configuration`.
- Layers are **ordered and total** — every layer has a declared position, and there is no "same-priority" case to resolve by chance.
- Composition is **offline-computable**: the effective set can be derived from the layers without starting anything, which is what makes LAY-6's inspection possible.
- This spec governs composition, not authority. What a composed component may do is settled by the extension and security planes; a layer that can compose cannot thereby grant.
- Nothing here presumes a file format, a distribution mechanism, or a process model.

## 3. Core Invariants (Layer 1 only)

Rules every Layer 2 implementation MUST NOT violate:

- **LAY-1 (Entry identity is author-assigned, stable, and the sole basis for reconciliation):** every entry carries an identity assigned by whoever wrote it, stable across reads, and reconciliation between two composition states is computed **by that identity alone** — never by position, never by content hash, never by a derived identity generated at read time. A derived identity makes every edit anywhere in the declaration a removal-plus-addition for every entry, so unchanged components restart because something else changed. An entry without an author-assigned identity is either refused or reported as unreconcilable; it is never given a silently-generated one.

- **LAY-2 (An override replaces its target's configuration wholly; there is no deep merge):** a layer that overrides an entry replaces that entry's configuration **entirely**, and an override that wishes to keep a field restates it. Partial or deep merging is forbidden. The cost is verbosity; what it buys is that the effective configuration of any entry is **authored by exactly one layer** and is readable there, rather than being a function of a merge algorithm's treatment of absent keys, empty collections, and explicit nulls — a function no layer's author can see.

- **LAY-3 (Disabled is a durable state of the entry, not its removal):** an entry may be present and **not running**. Disabling preserves the entry, its identity, and its configuration, and is reversible without reauthoring; it is not a runtime toggle that a restart forgets, and it is not achieved by deleting the entry. The state is visible in every inspection of the composition, because an entry that is present-and-off and an entry that is absent are different facts with different remedies.

- **LAY-4 (An override matching nothing is reported, never a silent no-op):** an override that targets an identity absent from the composed set is **surfaced**, naming the identity and the layer that wrote it. Silently discarding it is forbidden: the two things it means — a typo, or a base layer that changed underneath the override — are both actionable, and both present identically as "my override did nothing" if the system says nothing.

- **LAY-5 (An empty declaration and an absent declaration are different):** a layer that exists and declares nothing is distinct from a layer that does not exist, and a declaration that **parses to nothing** is an error rather than an empty layer. Turning a layer off is done by declaring it **explicitly empty**, which is a statement; a file that has become blank or comment-only is not a statement and is far more often a mistake than an intention.

- **LAY-6 (One composition algorithm; every surface that reports the effective set uses it):** there is exactly **one** implementation that composes layers into the effective set, and every consumer of that set — the path that actually runs the components, the inspection command, the diagnostic readout, any derived flag or capability list — obtains it from that implementation. A second implementation for reporting is forbidden. Two implementations agree until they diverge, and they diverge silently at exactly the moment someone is using the report to understand a discrepancy.

- **LAY-7 (A distribution unit contributes entries that remain overridable from above):** a unit that is distributed in order to insert composition entries contributes them **into the layer stack**, at a declared position, such that every layer above it can override, reorder, or disable what it inserted. A distributed unit MUST NOT install components outside the stack, at a position no layer can reach, or in a form the consumer's own layers cannot address — a contribution the consumer cannot override is a component they cannot remove without removing the whole unit.

- **LAY-8 (Ownership is determined structurally: system-owned until deviated):** whether the system may update a declaration in place is decided by **comparing it to what the system shipped**, not by a stored flag. A declaration that still matches the shipped form exactly is system-owned and may be updated in place; **any** deviation — an addition, a removal, a reordering — makes it user-owned and it is left untouched thereafter. The comparison is structural and total: there is no partial ownership, because a rule that updates *some* of a file the user edited produces a result neither party authored.

- **LAY-9 (Layer order is declared, from least specific to most; precedence never depends on load order):** the stack's order is an explicit, declared property — typically shipped defaults, then distributed units in their declared sequence, then the deployment's own layer, then the operator's, then a per-invocation overlay. Two layers never occupy one position, and precedence is never a consequence of which layer happened to be read first, installed first, or discovered first.

- **LAY-10 (Recomposition is transactional; a rejected one leaves the last good set running and reports):** applying a changed layer recomposes the **whole** stack and applies the result as one transaction. A recomposition that fails to read, parse, or validate leaves the **previously effective composition running, undisturbed**, and reports the failure through a channel the operator sees. Partially applying a recomposition is forbidden — it produces a running set that no layer stack describes, which is the one state from which neither retrying nor reverting is well-defined.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The stack

```plaintext
    ┌──────────────────────────────┐  most specific
    │  per-invocation overlay      │  transient, one run
    ├──────────────────────────────┤
    │  operator layer              │  this machine, this human
    ├──────────────────────────────┤
    │  deployment layer            │  this installation
    ├──────────────────────────────┤
    │  distributed unit N          │  ← LAY-7: everything it inserts stays
    │  …                           │           overridable from above
    │  distributed unit 1          │
    ├──────────────────────────────┤
    │  shipped defaults            │  least specific
    └──────────────────────────────┘

applied to an initially empty entry list, in declared order (LAY-9)
```

Each layer performs one of three operations on the accumulating list: **insert** a new entry, **override** an existing entry by identity (wholly, LAY-2), or **disable** one (LAY-3). Removal is deliberately not among them — an entry the user wants gone is disabled, which is reversible and legible, or removed from the layer that inserted it.

### 4.2 Reconciliation, and what identity buys

Given the previously effective list and the newly composed one, reconciliation is a three-way partition by identity (LAY-1):

| Identity is in | Action |
| --- | --- |
| both, configuration unchanged | nothing — the component is not touched |
| both, configuration changed | reconfigure that one component |
| new only | start it |
| previous only | stop it |

Everything about this table depends on the identity being **the author's**. With a positional or generated identity, "configuration unchanged" is unrecognizable — inserting one entry near the top shifts everything below it, and the reconciler correctly concludes that every one of them was removed and a different one added. The system is not wrong; it was given no way to be right.

This is why LAY-1 refuses to invent an identity rather than generating one silently. A generated identity produces a system that *works* — it converges to the right set — while restarting components for no reason, which is a performance and correctness problem nobody can attribute to its cause.

### 4.3 Why replacement rather than merge

Deep merge fails on the cases that matter, and its failures are unattributable:

- **Absent versus explicitly empty.** Does an override omitting a collection mean "keep the base's" or "make it empty"? Both readings are defensible and the choice is invisible at the call site.
- **Explicit null.** Does it clear the field or is it a value? The merge algorithm decides, and no layer's author reads the merge algorithm.
- **Order inside collections.** Merging two ordered collections has no canonical answer, so the algorithm picks one and it becomes load-bearing.

Whole replacement (LAY-2) makes the effective configuration of an entry a thing exactly one layer wrote, which any reader can find by looking at the highest layer that mentions the identity. The price is real — an override keeping nine fields restates nine fields — and it is the same price [l1-scoped-capability-layers.md](l1-scoped-capability-layers.md) SCL-4 pays for the same reason.

### 4.4 One algorithm, two consumers

LAY-6 is a `l1-surface-parity` SP-1 instance: one decision point per observable behavior. The effective composition is one behavior with at least two consumers — the path that runs things and the path that reports them — and giving each its own implementation guarantees they will diverge, silently, in the direction of the one exercised less.

The rule has a concrete shape: the inspection surface **composes the same layers with the same algorithm** and renders the result, rather than describing what it believes the algorithm would do. This makes inspection output a load-bearing artifact — what it shows is by construction what will run — and it makes the layered composition debuggable at all, since an override that lands nowhere (LAY-4) is visible in the same output that shows the entry it missed.

### 4.5 Ownership without a flag (LAY-8)

```plaintext
on update, for a shipped declaration:

    declaration == shipped form (exact, structural)
        └─ yes ──► system-owned      ──► replace with the new shipped form
        └─ no  ──► user-owned        ──► leave entirely untouched
                                          (report that it was skipped and why)
```

The structural comparison is the whole mechanism, and its virtue is that it cannot be wrong in the way a stored flag can. A flag says "unmodified" about a file the user edited outside the product; a flag says "modified" about a file the user edited and then reverted. The declaration itself is the record of whether it was touched.

"Any deviation" is deliberately strict — an added entry, a removed one, a reordering, all make the whole declaration user-owned. Partial ownership is the tempting refinement and it is exactly wrong: updating the parts the user did not touch produces a composition assembled by no one, at the moment the user is least expecting their configuration to change.

### 4.6 Demarcation from value resolution

| Question | Owner | Object | Failure it prevents |
| --- | --- | --- | --- |
| What is this setting's value? | `l1-declarative-configuration` DC-11 | one field | A low-trust layer setting an exec-adjacent or operator-owned field. |
| Which components exist, in what order, with what configuration? | this spec | the entry set | Reconciliation churn, unauthored merged configurations, and inspection that disagrees with execution. |

The two compose: LAY produces the entry set, and each entry's configuration is then subject to DC's declaration, validation, and authority rules. A layer that may insert an entry does not thereby gain the authority to set that entry's operator-owned fields (DC-11.1).

## 5. Drawbacks & Alternatives

- **Whole replacement is verbose.** An override changing one field restates the rest. Accepted deliberately (§4.3); the alternative trades verbosity for configurations nobody authored.
- **Author-assigned identity is a burden on the author.** Every entry needs a name. The alternative — generating identities — produces silent restart churn that is nearly impossible to attribute to its cause, so LAY-1 makes the cost explicit and up front.
- **LAY-8 is all-or-nothing.** A user who added one entry to a shipped declaration stops receiving updates to the rest of it. This is the correct trade — the alternative silently rewrites parts of a file the user is editing — but it means the product must be able to tell the user *that* their declaration is now user-owned, which is why the skip is reported rather than silent.
- **Rejected — a "user modified" flag.** A second source of truth about a fact the artifact already carries, wrong in both directions, and desynchronized by any edit made outside the product.
- **Rejected — removal as a layer operation.** Superficially symmetric with insert. Rejected because a removal is indistinguishable from an override that targets a missing identity (LAY-4), so the two cannot both be expressible; disabling is reversible, legible, and covers the intent.

## nodus-relevance mapping

Nodus assembles a run's world from host-supplied layers, so the composition dimension applies directly; most of it is already present in adjacent form.

| Element | nodus seam | Note |
| --- | --- | --- |
| Declared total layer order (LAY-9) | LP-21 locus ordering | Already declared and host-ordered; LAY-9 is the same rule for the entry set rather than for provider candidates. |
| One algorithm, one answer (LAY-6) | validate-before-run stage | The satisfiability check and the executor must resolve the same set, or a run validated against one world executes in another. |
| Override matching nothing (LAY-4) | `§config` acceptance (NL-20) | A configuration key targeting nothing is a validation finding, not a silent drop. |
| Empty vs absent (LAY-5) | schema-vocabulary declaration | A declared-and-empty host vocabulary layer differs from an undeclared one; the first is a statement about scope. |
| Distributed unit stays overridable (LAY-7) | LP-13 versioned import resolution, LP-12 admission vetting | An imported bundle contributes into the host's stack; the host's own layers must be able to override what it contributed. |
| Transactional recomposition (LAY-10) | LP-15 host-supplied durable state | A failed reload of a workflow's configuration leaves the last good definition running rather than a partially-applied one. |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DECLARATIVE-CONFIG]` | `.design/main/specifications/l1-declarative-configuration.md` | DC-1/DC-8/DC-11 — the value-resolution sibling and the demarcation in §4.6. |
| `[BINDING]` | `.design/main/specifications/l1-composition-binding.md` | What an assembled composition does once it binds to actors. |
| `[POINTS]` | `.design/main/specifications/l1-extension-points.md` | EP-4 — deterministic composition at one point; LAY is the same discipline over the contributor set. |
| `[SCOPES]` | `.design/main/specifications/l1-scoped-capability-layers.md` | SCL-4 — the same replacement-not-merge trade, made for the same reason. |
| `[PARITY]` | `.design/main/specifications/l1-surface-parity.md` | SP-1 — one decision point per observable behavior, which LAY-6 instantiates. |
| `[READOUT]` | `.design/main/specifications/l1-system-readout.md` | The inspection surface LAY-6 binds to the composition algorithm. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-19 | Core Team | Initial spec — composition layering: assembling the *set of components* from ordered layers, the set-valued sibling of DC-11's field-valued resolution (demarcated in §4.6). Author-assigned stable entry identity as the sole reconciliation basis, because a generated identity turns every edit into remove-plus-add and restarts components nobody changed — a correctness-preserving failure nobody can attribute (LAY-1); whole replacement with no deep merge, since merge semantics for absent keys, explicit nulls, and collection order are decided by an algorithm no layer author reads, and replacement makes each entry's effective configuration a thing exactly one layer wrote (LAY-2); disabled as a durable state of the entry rather than its removal, present-and-off and absent being different facts with different remedies (LAY-3); an override matching nothing reported rather than silently dropped, since typo and base-changed-underneath are both actionable and present identically (LAY-4); empty declaration distinguished from absent, with a parse-to-nothing treated as an error because a blanked file is far more often a mistake than a statement (LAY-5); **one** composition algorithm serving both the run path and every inspection surface, two implementations agreeing until the moment someone needs the report (LAY-6); a distributed unit contributing into the stack so everything it inserts stays overridable from above (LAY-7); ownership decided **structurally** — exact match to the shipped form is system-owned and updatable, any deviation is user-owned and untouched — because a stored "modified" flag is a second source of truth wrong in both directions, and partial ownership produces a file neither party authored (LAY-8); declared total layer order, precedence never a consequence of load order (LAY-9); transactional recomposition leaving the last good set running on failure, a partially-applied composition being the one state from which neither retry nor revert is defined (LAY-10). Distilled from an adoption pass over an external plugin-framework-based agent-harness reference. Concept-only. |

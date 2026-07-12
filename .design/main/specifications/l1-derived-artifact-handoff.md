# Derived-Artifact Handoff

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Several of this project's subsystems build an expensive, deterministically-rebuildable
**derived artifact** from source-of-truth data: the code-intelligence knowledge graph
(`l1-code-intelligence`), the memory ranking signals and embeddings
(`l2-memory-store` / `l1-memory-consolidation`), and any future precomputed index. These
artifacts are costly to compute the first time — indexing a large repository, embedding a
corpus, recomputing centrality — yet they are pure projections of data that already
exists, so a second device or a teammate that has the *same source* is forced to redo the
same expensive work to reach the same derived state.

This spec defines the discipline that removes that waste: a derived artifact can be
**handed off** — shared as a single portable, integrity-checked snapshot — so a peer
imports it and fills only its local diff incrementally, instead of paying the full
recompute. Its central claim: **because a derived artifact is always rebuildable from
source, it is never authoritative and never merged — a handoff is a cache bootstrap, not
a data sync.** That single property (derived, not authoritative) is what makes the handoff
safe, mergeless, and discardable, and is exactly what distinguishes it from
`l1-multi-device-sync`, which converges *authoritative* data and must never route a
derived artifact through its merge machinery.

## Related Specifications

- [l1-multi-device-sync.md](l1-multi-device-sync.md) — **the deliberate contrast**: that concept converges *authoritative* data (CRDT / reviewable-merge / supersede, SY-4) where a conflict is a real semantic conflict; this concept bootstraps a *rebuildable derived cache* where a "conflict" is meaningless (either side can regenerate it), so it is mergeless (DAH-4) and MUST NOT be routed through SY-4.
- [l1-code-intelligence.md](l1-code-intelligence.md) — the primary producer: the code knowledge graph is the archetypal handoff artifact; CI-3 (cache-not-authority) and CI-4 (persistent/incremental/fingerprinted) are exactly the properties DAH-1/DAH-2 build on.
- [l1-memory-consolidation.md](l1-memory-consolidation.md) — MC-5's fact-vs-derived-signal separation is the same principle at the memory layer; the derived-signal layer (centrality/cluster/recency, embeddings) is a handoff artifact, the authored fact layer is not.
- [l1-storage-model.md](l1-storage-model.md) — the artifact is state-tier, derived from program/source; STO-8 durability and the "rebuild is always possible" stance ground DAH-1/DAH-3.
- [l1-deployment-neutrality.md](l1-deployment-neutrality.md) — local-first-by-default and no-forced-egress: the handoff is opt-in (DAH-6) and travels only through user-controlled channels (DAH-7).
- [l1-security.md](l1-security.md) — the artifact inherits the source's confidentiality and the secret-exclusion floor (CI-13); a handoff never egresses source-derived data to a third party as a condition of sharing (DAH-7).

## 1. Motivation

An expensive derived artifact recomputed independently on every device and by every
teammate is pure duplicated work. Indexing the Linux kernel's call graph, embedding a
large memory corpus, or recomputing community structure can take minutes to hours; a
freshly-cloned repository or a newly-paired device pays that cost in full before the agent
can answer a single structural query — even though the derived state is *identical* to
what another replica already computed from the same source.

The naive fixes are both wrong:

1. **Route the artifact through data sync.** Treating the derived index as authoritative
   data and merging replicas is category error: two indexes of the same source are not in
   *conflict* — they are the same projection, trivially regenerable. Merging them wastes
   the merge machinery on data that has no independent truth, and risks a merged index
   that matches neither source.
2. **Recompute always, share nothing.** Correct but wasteful — it discards a completed,
   verifiable computation that a peer could simply import.

The right model is a **handoff**: publish the derived artifact as a portable snapshot,
let a peer import it and run only incremental catch-up for its local diff, and treat the
whole thing as a cache that can be regenerated at any time. Because the artifact is
derived, the hard problems of data sync (conflict resolution, causal ordering, tombstones)
simply do not arise — the resolution for any discrepancy is "take one and rebuild the
difference from source." This spec makes that model a first-class, safe, opt-in capability
rather than an ad-hoc optimization each producer reinvents.

## 2. Constraints & Assumptions

- **Derived, not authoritative.** This concept applies *only* to artifacts fully
  rebuildable from source-of-truth data. Authoritative data (authored memories, specs,
  the operational ledger, user files) is out of scope and belongs to
  `l1-multi-device-sync`. Misclassifying authoritative data as a handoff artifact would
  lose edits; misclassifying a derived index as authoritative data wastes merge effort.
- **Same-source assumption.** A handoff only helps a peer that has (approximately) the
  same source; the imported artifact is a *starting point* that incremental update
  reconciles to the peer's actual source (DAH-2), never a substitute for it.
- **Opt-in.** Sharing is never forced; the default may be either share or local-only, but
  a peer can always decline the artifact and rebuild from scratch (DAH-6).
- **On-device / user-controlled transport.** The artifact travels through a channel the
  user controls (a repository they own, their own paired devices); it is not egressed to
  a third party as a condition of the handoff (DAH-7).
- **No new source of truth.** The artifact is never consulted as authority for a fact it
  encodes; the source is (composes CI-3). A stale or absent artifact degrades speed only,
  never correctness.

## 3. Core Invariants

Layer 2 implementations MUST NOT violate these. They are technology-neutral.

- **DAH-1 Derived-only, never authority.** A handoff artifact is a deterministically
  rebuildable projection of source-of-truth data. It is always reconstructable from
  source, and every consumer treats it as a cache (composes CI-3, MC-5). Discarding it
  costs only recompute time, never data. An artifact that cannot be regenerated from
  source is not a handoff artifact and MUST NOT be shared as one.

- **DAH-2 Import-then-incremental bootstrap.** A peer with no local artifact but a present
  shared one imports it, then runs incremental update to reconcile only its *local diff*
  against source — never trusting the imported artifact wholesale, never paying the full
  recompute. An absent shared artifact falls back to a full build; a present one never
  makes the peer worse off than a full build.

- **DAH-3 Integrity-checked import with rebuild fallback.** The artifact is
  integrity-checked (checksum) and version/schema-checked on import. A corrupt, truncated,
  or incompatible artifact is **rejected**, and the consumer falls back to a full rebuild
  from source. A bad artifact never poisons local state, never silently degrades results,
  and never half-imports.

- **DAH-4 Mergeless by construction.** Because the artifact is derived (DAH-1), concurrent
  versions are never three-way-merged. A discrepancy is resolved by "take one version and
  rebuild the local diff from source" (an ours-wins choice over the binary artifact), and
  the transport is configured so the artifact never produces a merge conflict. This is the
  deliberate opposite of authoritative-data convergence (`l1-multi-device-sync` SY-4); a
  derived artifact MUST NOT be routed through that merge machinery.

- **DAH-5 Two-tier export (thorough vs incremental).** The artifact supports at least two
  write tiers: a **thorough** form (compacted, integrity-optimized, maximally compressed)
  written on an explicit full build, and a **fast** form (low-latency, lightly compressed)
  written by the background updater on incremental change — so the shared artifact stays
  current without paying the thorough cost on every small change. Both tiers import
  identically (DAH-2/DAH-3); the tier is a write-side cost choice, never a read-side
  semantic difference.

- **DAH-6 Opt-in and inspectable.** Producing and consuming a handoff artifact is opt-in,
  never forced. The artifact's presence, its tier, its staleness relative to source, and
  its provenance (what produced it, at what source revision) are inspectable, and a
  consumer can always decline it and rebuild from scratch. A handoff is an offered
  shortcut, never a mandatory dependency.

- **DAH-7 Source-confidentiality-inheriting, user-controlled transport.** A derived
  artifact is a projection of source and inherits the source's confidentiality and the
  unconditional secret-exclusion floor (CI-13) — it MUST NOT embed material the
  source-of-truth ingestion excludes. Sharing travels only through channels the user
  controls and is never a condition for egressing source-derived data to a third party
  (composes SY-8 / `l1-security` no-exfiltration, `l1-deployment-neutrality`).

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The handoff lifecycle

```text
[REFERENCE]
produce(source):                              // on explicit full build
    artifact := build_derived(source)
    write(artifact, tier=thorough)            // DAH-5 compacted + integrity-optimized

update(source_delta):                         // background updater, on change
    apply_incremental(source_delta)
    write(artifact, tier=fast)                // DAH-5 low-latency

consume(peer):                                // a peer arrives with source but no local artifact
    if shared_artifact_present():
        if integrity_ok(shared) and schema_compatible(shared):   // DAH-3
            import(shared)                                        // DAH-2 bootstrap
            incremental_update(peer.source_diff)                 // DAH-2 fill local diff only
        else:
            full_build(peer.source)                              // DAH-3 fallback, never poison
    else:
        full_build(peer.source)                                  // DAH-2 absent ⇒ full build
```

Every path terminates in a correct local artifact; the shared one only ever *saves time*.

### 4.2 Why mergeless is correct (DAH-4)

Two replicas that indexed the same source hold the *same* projection — there is no
independent truth to reconcile, so a "merge" would be busywork at best and a
matches-neither hybrid at worst. The only meaningful operation on a divergence is: pick
either version and let incremental update (DAH-2) reconcile it to the local source. This
is why the transport is configured to never conflict on the artifact (an ours-wins
resolution over the binary blob is always safe), and why this concept is the categorical
opposite of `l1-multi-device-sync`:

| | Authoritative data (`l1-multi-device-sync`) | Derived artifact (this spec) |
| --- | --- | --- |
| Source of truth | itself | source it is derived from |
| A divergence is | a real conflict needing resolution | meaningless — regenerable either way |
| Resolution | CRDT / reviewable-merge / supersede (SY-4) | ours-wins + rebuild the diff (DAH-4) |
| Loss cost | data loss | recompute time only |

### 4.3 Tiering (DAH-5)

The thorough tier pays a one-time cost (full compaction, index stripping, strongest
compression, integrity metadata) so the artifact a teammate first imports is small and
verifiable. The fast tier keeps the shared artifact *current* between full builds at low
latency, so a peer importing mid-stream still gets a near-fresh starting point and a small
incremental diff. Both are the same artifact at different write-cost points.

## 5. Ideas to Adopt

Adjacent mechanics mined alongside this concept, mapped to their Cronus home. Recorded
here as adoption candidates so the neighboring Stable specs are not churned in this pass.

| Mined mechanic | Adoption in Cronus |
| --- | --- |
| **Type-aware resolution as an embedded pass, without a language-server process** — refine call/usage edges with type inference (generics, inheritance, return-type propagation, method dispatch through an inferred receiver type) to "go-to-definition" accuracy, embedded in the extractor rather than an external LSP, tiered per-language with textual fallback, and honoring silent-beats-wrong. | **[extend `l2-codegraph`]** sharpens CI-16's resolution pass with a *type-resolution tier*. Both `l1-code-intelligence` Drawbacks and `l2-codegraph` Drawbacks currently frame the choice as binary (self-contained-syntactic vs heavy-LSP); this is the third path — embedded type resolution, no server process. An L2 implementation strategy, not a new L1 invariant (CI-16 already leaves the resolver unspecified). |
| **Transparent search-reflex augmentation** — intercept the agent's default text-search reflex (grep/glob) and inject structured graph results as additional context, non-blocking, and never gate the read path (gating read breaks read-before-edit). | **[extend `l1-interception-model` + `l1-agent-tool-ergonomics`]** an observe/transform interceptor (INT taxonomy) that augments rather than replaces the default search, fail-open. Realizes CI-6/CI-8's token-economy win *transparently*, without the agent having to remember to call a graph tool. |
| **Interactive graph-explorer UI surface** — a navigable visual explorer of the knowledge graph (node/edge browse, focus + neighborhood, community/cluster layout) as a first-class read surface. | **[extend `l2-office-view` + CI-12 export]** CI-12 already admits a web-visualization projection; the interactive explorer is that projection made navigable, a pure read view (never a second source of truth), sibling to the office/automation-canvas visual surfaces. |
| **Payload-free diagnostics trajectory** — an opt-in resource-counter time series (memory/fd/query counts, no source or query text) safe to share for leak/perf diagnosis. | **[extend `l1-operational-health` / `l1-diagnostic-log`]** a shareable-by-construction trajectory: the privacy property (counters only, never payload) is the reusable idea. |

## 6. Nodus Relevance

A compiled/validated nodus workflow is itself a **derived artifact** of its source
`.nd` definition: the transpiler output and the validated workflow graph
(`l2-workflow-runtime`) are deterministically rebuildable from the source workflow, so
they are handoff artifacts by DAH-1. Two concrete applications:

- **Transpile-cache handoff.** A validated/transpiled workflow can be shared as a derived
  snapshot so a peer skips re-transpilation and re-validation, importing then reconciling
  only its local `.nd` diff (DAH-2) — with the source `.nd` remaining the sole authority
  (DAH-1).
- **Workflow-graph artifact.** The workflow-graph view (steps as nodes, control/data/
  pipeline edges) that `l1-code-intelligence` §6 proposes for nodus is exactly a derived
  index; if it is ever shared to seed the automation canvas or an analysis surface, it is
  handed off under these invariants, never merged.

The gating and transport policy are host-side (StorageProvider/PolicyProvider), consistent
with how other concepts map to nodus; the language contributes the deterministic,
rebuildable output, not the handoff policy.

## 7. Drawbacks & Alternatives

- **Staleness of the shared artifact.** A teammate may import an artifact behind the tip
  of source; DAH-2's incremental reconciliation closes the gap, and DAH-6's inspectable
  staleness makes the lag visible. The artifact only ever saves time, so a stale one is
  never worse than a full build.
- **Artifact size in the transport.** A committed binary snapshot has size; mitigated by
  the thorough tier's compaction + compression (DAH-5) and by DAH-6 opt-in (a project that
  prefers everyone rebuild simply does not share it).
- **Alternative — route through `l1-multi-device-sync`.** Rejected (§4.2): a derived
  artifact has no independent truth to converge; merging it is category error.
- **Alternative — never share, always recompute.** Rejected: correct but discards a
  completed, verifiable computation a peer could import in seconds instead of minutes.
- **Alternative — treat the artifact as authority to skip source.** Rejected (DAH-1/CI-3):
  the artifact is a cache; a fact is always verified against source before a destructive
  action.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[SYNC]` | `.design/main/specifications/l1-multi-device-sync.md` | The authoritative-data-convergence contrast this concept must never be routed through. |
| `[CODE-INTEL]` | `.design/main/specifications/l1-code-intelligence.md` | The archetypal producer (the code graph); CI-3/CI-4 cache + incremental properties. |
| `[MEMORY-CONSOLIDATION]` | `.design/main/specifications/l1-memory-consolidation.md` | MC-5 fact-vs-derived separation — the derived-signal layer is a handoff artifact. |
| `[DEPLOY]` | `.design/main/specifications/l1-deployment-neutrality.md` | Local-first / opt-in / no-forced-egress posture (DAH-6/DAH-7). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-12 | Core Team | Initial spec — derived-artifact handoff as the "share the expensive rebuildable index instead of recomputing it, without ever merging it" discipline, distinct from authoritative-data multi-device-sync: derived-only never authority (DAH-1, composes CI-3/MC-5); import-then-incremental bootstrap (DAH-2); integrity-checked import with full-rebuild fallback, never poison (DAH-3); mergeless by construction — ours-wins + rebuild the diff, never routed through SY-4 (DAH-4); two-tier thorough/fast export (DAH-5); opt-in + inspectable presence/tier/staleness/provenance (DAH-6); source-confidentiality-inheriting + secret-exclusion floor + user-controlled transport, no third-party egress (DAH-7). Ideas-to-Adopt records three adjacent mechanics as [extend] candidates without churning Stable neighbors — embedded type-aware resolution without an LSP process (sharpens CI-16 / l2-codegraph, the third path past the binary self-contained-vs-heavy-LSP framing), transparent search-reflex augmentation (interception-model + agent-tool-ergonomics, fail-open, never gate read), and an interactive graph-explorer UI (l2-office-view + CI-12 projection). nodus: a validated/transpiled workflow + workflow-graph is itself a derived artifact, handoff-eligible with the source .nd as sole authority. Derived from studied prior art on a portable code-intelligence index. |

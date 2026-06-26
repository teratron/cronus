# Code Intelligence

**Version:** 1.2.0
**Status:** Stable
**Layer:** concept

## Overview

Code intelligence is the subsystem that turns a source tree into a **queryable semantic graph** of code entities (files, functions, classes, modules, types, interfaces) and their relationships (calls, imports, containment, inheritance, implementation, references). It is the agent-facing ground truth for *"where is X defined, who depends on it, what would break if I change it, and what minimal context do I need to act"* — replacing brittle full-text grep and context-window-flooding whole-file reads with precise, sub-second, structure-aware answers.

The concept is parser-agnostic: extraction (language-specific) is decoupled from the graph core (language-neutral), so adding a language never touches the graph, query, or analysis layers. On top of the graph sit three capability families: **structural analysis** (impact/blast-radius, call/dependency graphs, cycles, hot paths, dead code, complexity), **intent-aware context assembly** (a budget-bounded projection of the graph that reports how much it compressed), and **change/design analysis** (PR blast-radius bundles, docs↔code drift verification).

This L1 captures the technology-neutral invariants. The graph index realization — store schema, fusion ranking, community detection, deduplication — lives in [l2-codegraph.md](l2-codegraph.md).

## Related Specifications

- [l2-codegraph.md](l2-codegraph.md) — the implementation of this concept (graph index store, extraction pipeline, hybrid retrieval, graph analysis).
- [l1-storage-model.md](l1-storage-model.md) — the index is state-tier mutable data over program-tier immutable source; persistence follows STO-8.
- [l1-memory-model.md](l1-memory-model.md) / [l2-memory-store.md](l2-memory-store.md) — memories anchor to graph entities; entity change drives memory revalidation (CI-10).
- [l1-lookahead-planning.md](l1-lookahead-planning.md) — blast-radius (CI-5) feeds pre-execution consequence simulation for refactor/migration/deletion triggers.
- [l1-version-control.md](l1-version-control.md) / [l1-development-workflow.md](l1-development-workflow.md) — change/PR analysis (CI-7) supplies blast radius, test-gap, and reviewer signals to the commit/review gates.
- [l1-knowledge-base.md](l1-knowledge-base.md) — design-document indexing and hybrid recall reuse the knowledge-collection pattern; CI-9 cross-references docs against the graph.
- [l1-spec-driven-governance.md](l1-spec-driven-governance.md) — docs↔code verification (CI-9) is the product-level analog of spec/registry drift detection.
- [l1-tool-composition.md](l1-tool-composition.md) — capability-surface profiling (CI-11) reuses the deferred-tool / named-group reduction model (TC-7).
- [l1-quality-standards.md](l1-quality-standards.md) — complexity, dead code, and cycle findings (CI-5) are quality signals consumable by the gates.
- [l1-security.md](l1-security.md) — secret-exclusion at ingestion (CI-13) is an instance of the no-exfiltration / on-device discipline.

## 1. Motivation

Agents doing code work repeatedly need answers that are **relational**, not textual: the callers of a function, the files that transitively import a module, the blast radius of a rename, the tests that exercise a symbol, the minimal neighborhood relevant to a debugging task. Text search cannot answer these; reading whole files to reconstruct them burns the context budget and scales badly.

A pre-built semantic graph answers them directly and cheaply. Crucially, the graph is also a **context compressor**: given a focus entity and an intent, it can project a tiny relevant subgraph out of tens of thousands of entities and *report that ratio*, turning "how much did we trim" from a guess into a measured number. The same graph, joined with version-control and the memory subsystem, lets a single call describe the full risk surface of a change. And because design documents describe the same entities the graph already holds, the graph can verify whether documented design still matches the code — a continuous, automatic drift check.

Without an L1 concept, these capabilities have no shared contract: the existing index spec is (mis-)parented to the storage model, the impact/context/drift ideas have no home, and the relationship to memory, planning, review, and security is implicit. This spec makes code intelligence a first-class subsystem.

## 2. Constraints & Assumptions

- **On-device, no egress.** Indexing, embedding, and querying happen locally; the index never leaves the device. Cross-project graphs live in the user home tier, still on-device.
- **Best-effort, not authoritative.** The index trails the source by at most one change cycle. It is a fast cache, never the source of truth for a code location (CI-3).
- **Bounded ingestion.** Generated, vendored, and build artifacts are excluded by default; secret-bearing paths are excluded unconditionally (CI-13). A workspace-scoped ignore file (gitignore syntax) tunes the rest.
- **Extraction is pluggable and may be partial.** A language without a full grammar degrades to coarse extraction (names only) rather than failing; the graph core is unaffected.
- **Model-optional.** Every structural capability (CI-5) must work with no embedding model and no network — semantic retrieval (CI-8) is an enhancement, not a prerequisite.

## 3. Core Invariants

Layer 2 implementations MUST NOT violate these. They are written in technology-neutral terms.

- **CI-1 — Parser-agnostic graph core.** The graph, query, and analysis layers are independent of any source language. Ingestion is a pluggable extraction step that emits typed nodes and typed edges; adding or changing a language never modifies the core. ("Bring your own parser; the core owns the graph.")
- **CI-2 — Typed entity + relationship taxonomy.** Nodes belong to a closed kind set (at minimum: file, function/method, class/struct, module/namespace, type, interface/trait, variable/constant) and edges to a closed directed-relationship set (at minimum: contains, calls, imports, implements, extends, references; with optional runtime-call edges). Every node and edge carries a flexible property bag (e.g., line span, visibility, signature, call-site line, imported symbols). Edges are directed and may be queried by direction (incoming / outgoing / both).
- **CI-3 — Cache-not-authority discipline.** The index is best-effort and may be stale. Consumers treat it as a fast lookup and MUST validate a returned location against ground-truth source before any destructive action. The index never blocks or gates correctness on its own freshness. When a response may reference a not-yet-reindexed item, the subsystem signals that staleness **explicitly and per item** (naming the pending item, directing the consumer to verify it directly) rather than returning a possibly-wrong answer silently — the agent-facing form of this signal is governed by [l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) ATE-6.
- **CI-4 — Persistent, incremental, namespace-isolated storage.** The graph persists across restarts with no full re-parse on boot. Updates are incremental, keyed by a content/stat fingerprint so unchanged inputs are skipped. Multiple projects coexist in one store via namespace isolation (per-project key/ID prefix); a merged cross-project graph deduplicates external/library entities by stable label and skips unchanged sources by hash.
- **CI-5 — Structural analysis family (model-free).** From the graph alone, without any model, the subsystem derives: depth-bounded directional traversal; **impact / blast-radius** (the reverse-dependency closure of a seed), classified by change kind (modify / delete / rename, and signature-change vs body-only-change); **circular-dependency** detection; **hot paths** (entities ranked by transitive caller count); **dead/unused** detection (imports or symbols with no referencing edge); **entry-point** discovery (mains, request handlers, CLI commands, event handlers); per-entity **complexity** metrics; and per-module summaries. Traversals are depth-limited to stay bounded on large or cyclic graphs.
- **CI-6 — Intent-aware, budget-bounded context assembly with reported compression.** Given a focus entity, a declared intent (e.g., explain / modify / debug / test), and a token budget, the subsystem assembles the *minimal relevant neighborhood* by graph traversal + ranking + truncation, and returns a **compression accounting** (entities-in-graph / entities-traversed / entities-kept). The assembly is a deterministic projection over the graph; it spends no model call to decide what to include.
- **CI-7 — Composite context bundles.** Higher-order single-call bundles compose the graph with adjacent subsystems. An *edit bundle* returns source + callers + related tests + anchored memories + change history for an entity. A *change/PR bundle* returns, for a diff against a base: blast radius (impacted callers), test coverage and **test gaps** (changed entities with zero exercising tests), affected modules, diff-aware change classification (signature vs body), stale-documentation warnings, complexity, a commit-message hint, and suggested reviewers from change-history ownership.
- **CI-8 — Hybrid, lazy, optionally-disabled retrieval.** Symbol search fuses a lexical signal (keyword / full-text rank) with a semantic signal (embedding similarity) via rank fusion. Embeddings are computed lazily and are optional; a **graph-only mode** serves every structural capability (CI-5) and every exact lookup with no embeddings and no model load — the fast path for CI and constrained environments. Disabling semantics degrades ranking quality, never structural correctness.
- **CI-9 — Docs↔design verification (drift, never auto-edit).** Indexed design documents are cross-referenced against the live graph in both directions: forward (documented identifiers must exist in code), reverse (code coverage by docs), and **gap detection** (identifiers described in docs but absent from code). The subsystem can generate an architecture summary *from* the graph, and a human-readable **audit report** of the graph's own findings (top-connected entities, surprising cross-boundary connections, suggested questions). The audit report obeys honesty rules: never assert an unverified edge (uncertain relations are surfaced as ambiguous-for-review, not hidden), always disclose extraction cost, and show analysis scores raw rather than behind symbols. All findings are reported as drift or advisories; the subsystem never auto-modifies code or docs to resolve them.
- **CI-10 — Memory anchoring.** Durable agent memories may anchor to graph entities and be recalled by graph proximity to the working entity. A change to an anchored entity (especially deletion or signature change) drives revalidation/invalidation of the dependent memories. Code intelligence owns the anchor and proximity signal; the memory subsystem owns storage and recall policy.
- **CI-11 — Capability-surface profiling.** The agent-facing capability set is exposed through named profiles (e.g., lookup-only, structural, memory) so the surface — and its context cost — can be narrowed to the task. The active reduction is observable. Profiling never changes results, only which capabilities are reachable.
- **CI-12 — Interoperable export.** The graph exports to standard interchange formats for external visualization and analysis (a graph-description format, a web-visualization object format, a tabular format, and a subject-predicate-object triple format) without coupling external consumers to the internal store. The export catalog is open-ended: additional targets — a property-graph database projection (with optional direct push), a navigable per-community knowledge vault/wiki, a diagram render (architecture / call-flow) — are admissible as long as each is a pure projection of the graph, never a second source of truth.
- **CI-13 — Secret-exclusion at ingestion.** Credential directories and secret-bearing file classes are excluded from extraction and are never embedded or persisted into the index, regardless of other ignore configuration. Defense in depth against leaking secrets into a searchable artifact.
- **CI-14 — Node summaries for navigation.** A node may carry a compact, bounded summary (≈ one sentence) describing its responsibility, so an agent can decide whether a node is relevant *without reading the source*. Summaries are generated deterministically from local signal first (docstring/leading comment, exported symbols, dominant relations, key imports, community/hub context) and may be upgraded by an optional model pass; each records how it was produced and its format version. Summaries are optional, off by default, never the authority (the source is), and are surfaced in node lookup and context assembly under the same token budget as CI-6.
- **CI-15 — Vocabulary-grounded query.** Natural-language queries are expanded against the graph's *own* label vocabulary before traversal, so a wording mismatch between the question and the indexed entities does not collapse recall to noise. The navigable query surface offers at least: breadth traversal (broad neighborhood context), depth traversal (trace a specific path), shortest-path between two entities, and single-node explanation — each bounded by an explicit token budget and citing the source location of any concrete claim.
- **CI-16 — Reference resolution & indirect-edge synthesis with provenance.** Extraction emits raw entities and *direct* edges; a distinct resolution pass turns names into real connections — imports to their sources (including path aliases and workspace members), calls to definitions, inheritance — and *synthesizes* the indirect/dynamic-dispatch edges that static parsing misses (callback/observer registration, event channels, framework re-render and child-render, ORM descriptors, cross-language bridges). Every synthesized or heuristic edge carries explicit **provenance** — how it was wired and the site that wired it — distinct from directly-extracted edges, and is surfaced inline wherever a path crosses it. Governing principle: **a partially-bridged flow is worse than an unbridged one** — closing a flow end-to-end is required, because a half-bridged flow exposes a hop the consumer then resolves manually; an unbridged flow stays silent (silent beats wrong).
- **CI-17 — Measured resolution coverage with an honest frontier.** Edge-resolution completeness is *measured, not asserted*: per language and framework, the share of symbol-bearing files with at least one resolved cross-file dependent, on a real benchmark corpus. The unresolved residual is disclosed as a genuine static-analysis frontier (runtime dynamic dispatch, reflection / dependency-injection containers, framework-convention entry points, vendored third-party code) and is **never** improved by shrinking the denominator. Coverage is a first-class quality signal of the graph, distinct from retrieval-ranking quality.

## 4. Concept Detail

### 4.1 Graph model

The graph is a directed, typed, attributed multigraph. The minimum taxonomy:

| Node kind | Covers |
| --- | --- |
| File | A source file |
| Function | Free functions, methods, closures bound to a name |
| Class | Classes, structs, records |
| Interface | Traits, interfaces, protocols, abstract types |
| Module | Files-as-modules, namespaces, packages |
| Type | Type aliases, enums, primitive references |
| Variable | Constants, statics, fields, named bindings |

| Edge kind | Meaning |
| --- | --- |
| Contains | Parent contains child (file→function, class→method) |
| Calls | Caller invokes callee (carries call-site line) |
| Imports | File/module imports another (carries imported symbols) |
| Implements | Type implements an interface/trait |
| Extends | Type inherits from another |
| References | Non-call, non-import use of a symbol |
| RuntimeCalls | Resolved runtime coupling (e.g., HTTP client → route handler) |

Properties are an open key→value bag so extractors attach language-specific detail (visibility, async/test flags, signatures, docstrings, line spans) without schema changes. Nodes and edges carry stable identity so the graph is diffable across rebuilds.

**Multi-modal and external-system breadth.** Code is the primary but not the only source. The same graph admits non-code artifact nodes — documents, papers, images, transcripts, design rationale — tagged by a `file_type` attribute, and cross-modal edges (e.g., a `semantically-similar-to` edge linking a code symbol to a paper concept) so non-obvious couplings across artifact kinds surface in the same analysis. Beyond files, structured external systems are first-class ingestion sources: a database schema (tables/views/foreign-keys/join relationships), a dependency manifest, a code-intelligence-protocol index, change/PR history, and query logs each contribute typed nodes and edges. Non-code and external sources flow through the model-bearing extraction pass; pure-code corpora skip it entirely (CI-8 graph-only). This breadth is what makes the graph a general knowledge graph rather than a code-only index, while every invariant above (identity, confidence, cache discipline, secret exclusion) applies uniformly across sources.

### 4.2 Structural analysis catalog (CI-5)

All derivable from topology alone:

- **Neighbors / traversal** — incoming, outgoing, or both, depth-bounded.
- **Impact (blast radius)** — reverse-dependency closure from a seed over a configurable relation set, depth-limited, each hit tagged with depth and the relation it arrived by. Classified by change kind: a *rename* or *signature change* propagates differently than a *body-only* edit (the latter usually has empty structural blast radius — a key input to memory invalidation policy).
- **Circular dependencies** — strongly-connected components over the import/reference subgraph, collapsed to file granularity, rotation-deduplicated, ranked tightest-first.
- **Hot paths** — entities ranked by transitive caller count; the most-depended-upon code.
- **Dead / unused** — imports or symbols with no incoming reference edge.
- **Entry points** — nodes matching role heuristics (mains, request handlers, CLI commands, event handlers).
- **Complexity** — per-entity metric with a breakdown (branches, loops, nesting, early returns, exception paths).
- **Module summary** — per-directory rollup (entity counts, language mix, top-complexity and top-connected entities).

### 4.3 Context assembly + compression accounting (CI-6)

The assembly pipeline is: resolve focus entity → select relation set and depth by intent → traverse → rank candidates → truncate to the token budget → emit. The emitted result always carries the accounting triple so the caller (and any analytics) can see the realized compression:

```text
[REFERENCE]
context(focus, intent, budget) -> {
  payload:  <source + ranked neighborhood + imports + siblings + intent-specific hints>,
  stats:    { entities_in_graph, entities_traversed, entities_kept },
}
# intent shapes the relation set + depth:
#   explain → containment + outgoing calls + types
#   modify  → callers + tests + same-file siblings
#   debug   → callers + callees + error-handling edges
#   test    → callees + existing related tests
```

The projection is deterministic and model-free; an optional design-doc augmentation (CI-9) may attach matching documentation sections.

### 4.4 Change / PR analysis (CI-7)

One call over a diff-against-base composes graph + version-control + tests + docs into a risk surface: impacted callers (blast radius), test coverage with explicit **gaps** (changed entities reachable by no test), affected modules, **diff-aware classification** (signature change vs body-only — the high-signal vs low-signal distinction), stale-doc warnings (entities described in indexed docs whose code changed), complexity, a commit-message hint, and reviewer suggestions from change-history ownership. This is the structural counterpart to the human review gate.

### 4.5 Docs↔design verification (CI-9)

Design documents are chunked (heading tree) and indexed for hybrid recall, then verified against the graph: forward (every documented identifier resolves to code), reverse (code coverage by docs), and gap detection (documented-but-unimplemented identifiers → a TODO list straight from the spec). An architecture summary can be generated *from* the graph (modules, hot paths, complexity hotspots, cycles). The subsystem reports drift; resolution is always a human/agent action, never an automatic edit.

### 4.6 Storage, namespacing, interop (CI-4, CI-12)

The store is a backend abstraction so the same graph runs in-memory (tests, ephemeral runs) or persistent (durable, instant restart). A namespace wrapper prefixes all keys/IDs so many projects share one store with strict isolation, and scans stay scoped to the active project. A merged cross-project graph dedups external/library entities by label and skips unchanged sources by content hash. The graph exports to standard interchange formats (graph-description, web-visualization, tabular, and subject-predicate-object triples) for external tooling without exposing the internal store.

## 5. Ideas to Adopt

How each mined mechanic maps onto Cronus specs. Items marked **[new]** are not yet captured anywhere; **[reparent]**/**[extend]** adjust existing specs.

| Mined mechanic | Adoption in Cronus |
| --- | --- |
| Parser-agnostic graph core + typed node/edge taxonomy | **[reparent]** `l2-codegraph.md` realizes this concept and should declare `Implements: l1-code-intelligence.md` (it was previously parented to the storage model). CI-1/CI-2 become its invariant contract. |
| Best-effort cache discipline | Already in `l2-codegraph.md §2`; promoted to invariant CI-3. |
| Persistent + incremental + namespaced multi-project store | Already in `l2-codegraph.md §4.5/§4.15`; generalized as CI-4 (the namespace-prefix isolation is the same mechanism as the global-graph node prefixing). |
| Impact / blast-radius with change-kind classification | **[new]** CI-5; feeds `l1-lookahead-planning.md` (refactor/migration/deletion triggers) and the memory invalidation map in `l2-memory-store.md` (signature-change vs body-edit already distinguished there — CI-5's classification is its upstream source). |
| Intent-aware budgeted context assembly + **compression accounting** | **[new]** CI-6; complements the token-budget context engines in `l2-context-management.md` / `l2-agent-session.md` with a *structural* projection and a measured trim ratio. |
| Edit-context + PR-context composite bundles | **[new]** CI-7; supplies blast-radius / test-gap / reviewer / commit-hint signals to `l1-version-control.md` + `l1-development-workflow.md`. |
| Hybrid lexical+semantic retrieval, lazy embeddings, graph-only mode | Hybrid fusion already in `l2-codegraph.md §4.4`; **[new]** the *graph-only, model-free* fast path is promoted to CI-8 for CI/constrained use. |
| Docs↔code verification, design gaps, architecture-doc generation | **[new]** CI-9; product-level drift detection paralleling `l1-spec-driven-governance.md`; reuses `l1-knowledge-base.md` doc indexing + hybrid recall. |
| Memory anchored to graph entities, proximity recall, change-driven invalidation | Already partially in `l2-memory-store.md` (CodeLink, graph-proximity boost, CodeChangeType→SuggestedAction); CI-10 names code intelligence as the owner of the anchor + proximity signal. |
| Capability-surface profiles | CI-11; reuses the deferred-tool reduction model of `l1-tool-composition.md` (TC-7) and the `ToolSurfaceProfile` in `l2-agent-session.md`. |
| Interoperable export (graph-viz / web / tabular / triples) | **[new]** CI-12; the triple (subject-predicate-object) export is a clean bridge to any external knowledge-graph tooling. |
| Secret-exclusion at ingestion | CI-13; an instance of `l1-security.md` no-exfiltration; `l2-codegraph.md` already excludes via ignore rules, hardened to unconditional for secret classes. |
| Node summaries for AI navigation | **[new]** CI-14; compact deterministic-first per-node summaries so an agent skips source reads; directly amplifies CI-6 budgeted assembly. Roadmap in `l2-codegraph.md`. |
| Vocabulary-grounded query + navigable query surface (BFS/DFS/path/explain, budgeted) | **[new]** CI-15; query-expansion against the graph's own vocabulary is a recall-robustness mechanic measurable by `l1-retrieval-evaluation.md`. Roadmap in `l2-codegraph.md`. |
| Multi-modal + external-system ingestion (docs/papers/images/audio + DB schema / dependency manifest / code-intel-protocol index / PR history / query logs; cross-modal edges) | **[extend]** §4.1 breadth note; doc/paper/image/audio ingestion already in `l2-codegraph.md §4.8` three-pass — the DB-schema/manifest/PR/query-log external sources and the `file_type`-tagged general-graph framing are the new part. |
| Human-readable audit report with honesty rules (top entities, surprises, questions; ambiguous-for-review; raw scores; disclosed cost) | **[extend]** CI-9; `l2-codegraph.md §4.13` already computes the analyses — the generated report artifact + honesty rules are the new framing. |
| Richer export targets (property-graph DB push, per-community vault/wiki, diagram render, GraphML/SVG) | **[extend]** CI-12 open-ended catalog; each is a pure projection, never a second source of truth. |
| Reference resolution + indirect-edge synthesis (callbacks/observers/event channels/framework re-render/cross-language bridges) with provenance | **[new]** CI-16; the resolution depth that lets a flow connect end-to-end across dynamic dispatch; provenance distinguishes synthesized from extracted edges. Roadmap in `l2-codegraph.md`. |
| Close-flow-end-to-end principle (partial bridging worse than none) | **[new]** CI-16; a coverage discipline — never ship a half-bridged flow. |
| Measured cross-file resolution coverage with honest frontier | **[new]** CI-17; graph-completeness quality signal, distinct from `l1-retrieval-evaluation.md` ranking metrics; never gamed by shrinking the denominator. |
| Per-item staleness signaling to the agent | **[extend]** CI-3; the agent-facing form is `l1-agent-tool-ergonomics.md` ATE-6. |

## 6. Nodus Relevance

The reference's core idea — *a parser-agnostic graph core fed by per-language extractors, with structural analysis layered on top* — transfers directly to the nodus DSL, which already parses workflows into an AST and validates them. A **workflow graph** view treats a nodus workflow as a typed directed graph (steps as nodes; control/data/pipeline edges as edges) and unlocks the same analyses:

- **Step-dependency & reachability** — which steps feed which; unreachable steps are the dead-code analog (see `l1-nodus-testing.md` route-coverage advisory).
- **Impact analysis over a workflow** — what downstream steps a change to one step affects (blast radius for `→pipeline` and `@ctx` data flow), a natural input to the change-merge delta model in `l1-change-merge.md` (delta granularity ↔ step/macro).
- **Cycle / bound checks** — detect unbounded loops without `~UNTIL MAX:n`, mirroring circular-dependency detection; complements the requirement checklists in `l1-requirement-checklists.md` (loop-bound, branch-exhaustiveness coverage).
- **Macro call graph** — `RUN(@macro)` edges form a call graph; hot-path and "god macro" ranking surface over-coupled definitions.
- **Export for visualization** — exporting a workflow graph to a standard graph format gives the automation canvas (`l1-automation-canvas.md`) and external tools a render source without bespoke serialization.
- **Node summaries (CI-14)** — a bounded one-line summary per step/macro lets an agent navigate a large workflow without reading every line; deterministic-first from the step's command + comment + I/O contract.
- **Vocabulary-grounded query (CI-15)** — expanding a natural-language question against a workflow's own step/macro labels before traversal makes "how does this workflow handle X?" robust to wording, useful for explaining or debugging a workflow.

These are candidate capabilities for a future nodus analysis surface (likely an `l1-nodus-*` companion); this spec records the relevance, the nodus workspace owns any realization.

## 7. Drawbacks & Alternatives

- **Staleness vs. cost.** A fully-fresh graph would require synchronous re-extraction on every change; the best-effort + incremental + content-fingerprint approach (CI-3/CI-4) trades a one-cycle staleness window for responsiveness. CI-3 makes the trade explicit and forces validation before destructive use.
- **Extraction fidelity.** Heuristic call/reference resolution produces some inferred (uncertain) edges; the implementation labels confidence and suppresses low-confidence cross-language artifacts. The L1 only requires that uncertainty be representable, not a specific resolver.
- **Alternative — language servers (LSP).** Richer type-aware analysis, but per-language server processes, heavy lifecycle, and IPC. The self-contained graph is more portable and embeddable, and the graph-only mode (CI-8) keeps it fast and dependency-light.
- **Alternative — pure text search.** Cheaper to build, but cannot answer relational questions and floods context. The graph exists precisely to make those answers cheap (CI-6 compression accounting quantifies the win).

## Document History

| Version | Change |
| --- | --- |
| 1.2.0 | Added CI-16 (reference resolution & indirect-edge synthesis with provenance; close-flow-end-to-end principle) and CI-17 (measured resolution coverage with an honest frontier); extended CI-3 with explicit per-item staleness signaling (→ ATE-6); Ideas-to-Adopt rows extended |
| 1.1.0 | Added CI-14 (node summaries for navigation) and CI-15 (vocabulary-grounded query + navigable query surface); enriched CI-9 (audit report + honesty rules) and CI-12 (open-ended export catalog: property-graph push / vault-wiki / diagram render); added §4.1 multi-modal + external-system ingestion breadth note; Ideas-to-Adopt + nodus-relevance rows extended |
| 1.0.0 | Initial spec — CI-1…CI-13; code intelligence as a queryable semantic graph; ideas-to-adopt + nodus-relevance mapping |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CODEGRAPH]` | `.design/main/specifications/l2-codegraph.md` | The implementation realizing this concept |
| `[STORAGE]` | `.design/main/specifications/l1-storage-model.md` | Program/state tiering and durable-store invariant |
| `[MEMORY]` | `.design/main/specifications/l2-memory-store.md` | Memory anchoring + change-driven invalidation map |

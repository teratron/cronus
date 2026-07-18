---
phase: 21
name: "Knowledge Store"
status: Todo
subsystem: "crates/store-local/src/knowledge"
requires: [11, 16, 17]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 21 Tasks — Knowledge Store

**Phase:** 21
**Status:** Todo
**Strategic Goal:** Ship `l2-knowledge-store` — the agent-queryable RAG subsystem: named, access-controlled document collections with hybrid semantic (sqlite-vec ANN) + keyword (FTS5) retrieval fused by RRF, an async ingestion pipeline (file/URL/record), incremental re-indexing, soft-delete GC, storage-enforced authorship zones, and a curation lifecycle. Foundation-then-parallel; Track A gates B/C/D.

## Atomic Checklist

- [x] [T-21A01] Schema + contract types + store scaffolding (`knowledge` module)
- [ ] [T-21B01] Ingestion core: chunking + File/Record adapters + embed seam + transactional write + KB-3 re-index
- [ ] [T-21B02] URL adapter + KB-5 source-type completeness + document status lifecycle
- [ ] [T-21C01] Hybrid retrieval: ANN + FTS5 + RRF fusion (KB-1/KB-6/KB-7)
- [ ] [T-21C02] Query preparation (KB-11) + `min_curation` retrieval filter (KB-10 read side)
- [ ] [T-21D01] Authorship zones + curation write-gates + soft-delete + GC (KB-8/KB-9/KB-10 write side)
- [ ] [T-21D02] Access gate (`ResourceKind::Knowledge`) + facade `KnowledgeStore` + `cronus knowledge` CLI (KB-4)
- [ ] [T-21T01] Validation sweep: KB-1…KB-11 acceptance

## Detailed Tracking

### [T-21A01] Schema + contract types + store scaffolding

- **Spec:** l2-knowledge-store.md §4.1 (Schema), §4.3 (RetrievalRequest/RetrievedChunk types), §3 (Invariant Compliance types), §4.4 (KB-9/KB-10 write gates), §4.6 (KB-8 soft-delete/GC)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local knowledge::schema` — 15/15 pass (Evidence Capsule: command `cargo test -p cronus-store-local knowledge::schema`, exit_code 0, key_findings: "15 passed; 0 failed", "sqlite_vec_extension_is_registered ok", "kb1_ann_search_finds_the_nearest_neighbour ok"). `cargo check --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --all -- --check` all clean. `cargo test --workspace` green across 3 consecutive runs (0 failed each).
- **Handoff:** Gates T-21B01/C01/D01. **Scope note:** this task delivered more than the original schema+round-trip Verify line specified — it also delivered the full store-tier retrieval primitives (`ann_search`/`fts_search`/`hydrate_chunks`) and the KB-8/KB-9/KB-10 write/gc gates, since the sqlite-vec dependency decision (below) resolved toward building the real thing now. This narrows Track C's remaining scope to RRF fusion + domain-tier composition over these already-built primitives, not raw ANN/FTS implementation.
- **Notes:** New single-file `crates/store-local/src/knowledge.rs` (matching the `wiki.rs` simple-subsystem precedent, not a directory — refactor to a directory later only if it grows past the memory/-store threshold). DDL: `knowledge_collection`, `knowledge_directory`, `knowledge_document` (`origin` NOT NULL default `agent`, `curation` nullable + `ix_kdoc_curation`/`ix_kdoc_collection`/`ix_kdoc_status`), `knowledge_chunk`, `knowledge_chunk_fts` (FTS5 standalone, manually synced — memory/wiki pattern), `knowledge_chunk_vec` (real `vec0`, TEXT primary key, `FLOAT[768]`). Contract types + `KnowledgeStore` port trait in `crates/contract` (no `Send + Sync` bound — `rusqlite::Connection` isn't `Sync`, the `WikiCache`/`WikiReadSurface` precedent, corrected after an initial clippy/compile failure copied from `InferenceBackend`'s bound by mistake). KB-9 enforced via a typed `WriteOverride::HumanDirected { audit_ref }` at the single write seam (`write_document`); KB-10 via `set_curation` requiring `human_auth` for `Reviewed`/`Stable`; KB-8 via `soft_delete_document` + `gc`; KB-3 via `delete_chunks`. `ann_search`/`fts_search` over-fetch from `vec0`/FTS5 then post-filter by `collection_id`/`status='ready'` (l2-knowledge-store §2's documented "post-ANN filtering" — `vec0` has no per-row collection scoping). All 15 tests, incl. a real nearest-neighbour proof and both KB-9 refuse/override paths, pass against the genuine `vec0` extension (not a stub).

  **Dependency decision (user-directed):** flagged via `AskUserQuestion` (defer-to-FTS-only vs. build real `vec0` now); the user explicitly directed "decide as the engineer — optimize for quality/scalability/flexibility, not time" — added `sqlite-vec = "0.1.10-alpha.4"` to workspace + `store-local` deps and built the real ANN half this task, not a deferred stub.

  **Real bug found and fixed during verification (not a stub, not swept under the rug):** `sqlite-vec 0.1.10-alpha.4` (the latest published version — no newer stable release exists) has a genuine **upstream packaging bug**: its C source (`sqlite-vec.c`) enables `SQLITE_VEC_ENABLE_DISKANN` and `SQLITE_VEC_ENABLE_RESCORE` by default (`#ifndef`-guarded, defaulting to `1`), but the published crates.io tarball omits their companion source files (`sqlite-vec-diskann.c`, `sqlite-vec-rescore.c`) — the build fails with "file not found" out of the box. The crate's own `build.rs` exposes no way to disable these from outside. Root-caused by reading the vendored `.c` source directly (not guessed); fixed by adding `.cargo/config.toml` with `CFLAGS = "-DSQLITE_VEC_ENABLE_DISKANN=0 -DSQLITE_VEC_ENABLE_RESCORE=0"` (durable, applies automatically to every `cargo` invocation in this workspace — not a manual per-session env var). Neither disabled feature is used by standard `vec0` KNN search. Verified this global `CFLAGS` does not affect any other C compilation in the workspace (`rusqlite`'s bundled SQLite, etc.) via a full `cargo check --workspace` + `cargo test --workspace` pass. Also confirmed via the vendored C source (not assumed) that this `vec0` build supports a `TEXT PRIMARY KEY` (`pkIsText`) exactly as the original spec's illustrative DDL used, and that the KNN query returns the declared primary-key column directly (`SELECT chunk_id, distance FROM knowledge_chunk_vec WHERE embedding MATCH ?1 AND k = ?2`) with no rowid join needed.

  **Also fixed:** `RetrievedChunk` could not derive `Eq` (contains `f32`, which has no total ordering / NaN != NaN) — dropped to `PartialEq` only, caught immediately by `cargo check`.

### [T-21B01] Ingestion core + File/Record adapters + KB-3 re-index

- **Spec:** l2-knowledge-store.md §4.2 (Ingestion Pipeline); KB-3, KB-5, KB-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain knowledge::ingest` — a document ingests to `ready` against a deterministic fake `EmbeddingBackend`; re-index deletes the document's chunks/fts/vec before re-insert (no duplicate chunks); a failed extract leaves the document in `error` status with the collection still queryable. `cargo test --workspace` exit 0.
- **Handoff:** Retrieval (C) needs stored chunks; D01 guards the write path this establishes.
- **Notes:** Domain `knowledge::ingest` — `chunk_text` (Unicode sentence boundary, 512-token / 64-overlap defaults, configurable per collection), `FileIngester` + `RecordIngester` (extract plain text), embed each chunk via an injected `EmbeddingBackend` seam (thin wrapper over `contract::InferenceBackend::embed`; deterministic fake in tests — the generator-optional precedent). Transactional insert of chunk + fts + vec rows through a store port implemented by `store-local` (so `cronus-domain` never depends on `cronus-store-local` — the `WikiCache` port precedent). KB-3: delete-before-insert on re-index. Chunk-id generation via a process-static `AtomicU64` suffix (the `MemoryId` idiom), never per-instance state.

### [T-21B02] URL adapter + source-type completeness + status lifecycle

- **Spec:** l2-knowledge-store.md §4.2, §5.3 (web scraping); KB-5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — a URL source ingests to `ready` against a hermetic local HTTP fixture (the Phase 17 `TcpListener` mock precedent); HTML→text extraction produces chunks carrying `source_ref`; document status transitions `pending → indexing → ready|error`. Disclosed: live web scraping not exercised in CI.
- **Handoff:** Completes KB-5 (file/URL/record). Independent of C once B01 lands the pipeline.
- **Notes:** `UrlIngester` (HTTP fetch + HTML→text). HTTP fetch lives at the adapter/facade tier (not `cronus-domain`, which must stay I/O-free) — reuse the `model-local` transport path or a minimal std fetch, decided at execution and disclosed. Respect `robots.txt` + rate-limit (§5.3). Correlation-id-via-`status` polling (§5 note 4), no separate job-tracking table.

### [T-21C01] Hybrid retrieval: ANN + FTS5 + RRF fusion

- **Spec:** l2-knowledge-store.md §4.3 (Retrieval); KB-1, KB-6, KB-7
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain knowledge::retrieval` — a query returns only chunks from the requested `collection_ids` (KB-1: never another collection's chunk), RRF fuses vector + keyword hits, every `RetrievedChunk` carries `source_ref` (KB-6) and the API asserts no correctness (KB-7). Against a fake embedder with deterministic vectors.
- **Handoff:** C02 layers query-prep + curation filter on this; T01 sweeps it end-to-end.
- **Notes:** Domain `knowledge::retrieval` — embed the query via the `EmbeddingBackend` seam, ANN over `knowledge_chunk_vec` scoped to ready documents in the target `collection_ids` (top_k*2), FTS5 over `knowledge_chunk_fts` (top_k*2), RRF fusion (k=60), dedup by `chunk_id`, trim to `top_k`, apply `min_score`. **KB-1 collection isolation is the acceptance spine.** If the sqlite-vec extension is deferred (T-21A01 flag), retrieval ships FTS-first with the vector half a documented seam — the query still returns cited keyword hits, never an empty/broken result.

### [T-21C02] Query preparation (KB-11) + min_curation filter (KB-10 read side)

- **Spec:** l2-knowledge-store.md §4.5 (Query Preparation), §4.3 (min_curation); KB-11, KB-10
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — an empty/failed preparation degrades to the raw query (never an empty result set); a compound query's sub-queries are retrieved independently and RRF-merged; `min_curation` excludes `draft` chunks while human-origin/NULL-curation rows stay eligible; preparation never widens the `collection_ids` set nor alters `source_ref`.
- **Handoff:** Completes the retrieval surface; feeds T01.
- **Notes:** `QueryPreparer` seam with a no-op default (opt-in; unwired = raw query embedded directly). A wired preparer does keyword extraction/expansion + compound decomposition; prepared + raw both recorded in the retrieval trace (transparency). The `min_curation` trust floor drops chunks below the requested curation level; human sources always eligible.

### [T-21D01] Authorship zones + curation write-gates + soft-delete + GC

- **Spec:** l2-knowledge-store.md §4.4 (Authorship Zones & Curation), §4.6 (Soft Delete & GC); KB-8, KB-9, KB-10
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — a direct store write to an `origin='human'` row without an override is refused (`ReadOnlyZone`); `set_curation(Reviewed, None)` is refused while `(Reviewed, Some(auth))` succeeds; a soft-deleted document's chunks never appear in retrieval; the GC removes all chunk/fts/vec rows + the document row past the retention window.
- **Handoff:** Guards the B-track write path; T01 sweeps KB-8/9/10.
- **Notes:** **KB-9 is the load-bearing security property** — the `origin='human'` read-only zone is enforced at the single store write seam (`store.rs`/`db.rs`) via a typed `WriteOverride::HumanDirected { audit_ref }`, never by caller convention (the BA-4 / OA-3 structural-enforcement lineage). `origin` is assigned from the ingestion source at creation, never chosen by a later agent write. KB-10: `set_curation` gates `reviewed`/`stable` transitions on human authorization; the agent owns `draft`. KB-8: soft-delete (`status='deleted'`, excluded via `JOIN … WHERE status != 'deleted'`) + a startup+periodic GC.

### [T-21D02] Access gate + facade service + `cronus knowledge` CLI

- **Spec:** l2-knowledge-store.md §3 (KB-4), §4.7 (crate layout); l1-resource-sharing (RS-1…RS-8)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` + 1 real CLI smoke — a denied read reports absent with no cross-collection leak; an owner reads their own collection (owner-wins via `is_owner`); `cronus knowledge query` returns cited chunks end-to-end through the real facade + store.
- **Handoff:** Completes the shipped surface; T01 validates the whole stack.
- **Notes:** Add `ResourceKind::Knowledge` to the uniform grant model + a `GatedKnowledge` wrapper running `has_access(Knowledge, collection_id, Read)` before every retrieval/read — a denied read never reaches the store (the Phase 16 `GatedWiki` precedent). Facade (`crates/core`) re-exports a `KnowledgeStore` service wiring the real store + embedder (via `InferenceBackend`). `cronus knowledge collection create|list`, `knowledge doc add|remove`, `knowledge query` — verb-first per `l2-cli`, INV-9 shipped-surface honesty (no unbound verbs).

### [T-21T01] Validation sweep: KB-1…KB-11 acceptance

- **Goal:** Verify the assembled knowledge store against `l2-knowledge-store` — every KB invariant covered by a named test through the real facade export chain + real SQLite store (deterministic fake embedder; no live model in CI).
- **Method:** New `crates/core/tests/knowledge_invariants.rs` — one named test per invariant: KB-1 collection isolation, KB-3 incremental re-index, KB-4 access gate, KB-5 three source types, KB-6 attribution, KB-7 non-authoritative surface, KB-8 soft-delete, KB-9 authorship-zone write refusal, KB-10 curation gating + `min_curation`, KB-11 query-prep fallback + scope-preservation. KB-2 (directory tree, retrieval-independent) covered structurally. Final gate: `cargo test -p cronus-core knowledge_invariants` green + `cargo test --workspace` exit 0 ×3 + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check`.
- **Status:** Todo

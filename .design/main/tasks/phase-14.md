---
phase: 14
name: "Memory Intelligence & Consolidation (L2)"
status: Todo
subsystem: "crates/store-local (SQLite memory adapter: schema, ranking, maintenance) · crates/domain (consolidation/query policy) · crates/contract (UserDataStore/MemorySearch seam extensions): the active memory tier over l2-memory-store"
requires: [4, 13]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 14 Tasks — Memory Intelligence & Consolidation (L2)

**Phase:** 14
**Status:** Todo
**Strategic Goal:** Realize the two Stable memory L2s — `l2-memory-consolidation` (MC-1…MC-10) and `l2-memory-intelligence` (MI-1…MI-13) — over the existing SQLite substrate (`l2-memory-store`, in `cronus-store-local`), behind the `UserDataStore` seam. Turn the passive store into an active memory agent: an off-hot-path maintenance/consolidation tier and a caller-facing query surface, both generator-optional and backend-swappable.

> **Domain-logic-first (Phases 9–12 precedent).** The community-detection algorithm (MC-7) and every LLM call are **seams with stubs** — this phase proves the policy, the confidence gates, the fact-vs-derived boundary, and the degradation paths, not ML quality. Acceptance = each MC/MI invariant covered by a test + cold-start (empty derived layer) works + no hot-path model call or graph walk introduced.

## Atomic Checklist

- [x] [T-14A01] `memory_signal` fact-vs-derived table + version-guarded neutral-default degradation (MC-5)
- [x] [T-14A02] `depth` + `lifecycle_state` columns with never-rewrite-raw (MC-1) and prune-protective (MI-9) guards
- [x] [T-14B01] Multiplicative offline-precomputed ranking wrapping §4.2 base relevance (MC-8)
- [x] [T-14B02] Corpus-maintenance pass — recency decay, archive, split-flag, merge (MC-6 minus MC-7; summarize moves to T-14B03 — needs the MC-3 edge graph)
- [ ] [T-14B03] Consolidation write: additive edges, action algebra, incremental watermark, co-edit safety, emergent summaries (MC-2/3/4/7/9/10)
- [ ] [T-14C01] `answer` projection + temporal modes + structured predicate + recall-visibility (MI-1/2/3/8)
- [ ] [T-14C02] Conflict routing, digest, capture policy/directives/normalization/raw + distillation (MI-4/5/6/7/10/11/12)
- [ ] [T-14C03] Gated experience reuse with retained authority gate (MI-13)
- [ ] [T-14T01] Validation: MC/MI invariant-compliance sweep + cold-start + generator-degradation

## Detailed Tracking

### [T-14A01] `memory_signal` fact-vs-derived table (MC-5)

- **Spec:** l2-memory-consolidation.md §4.2, MC-5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` covers: a signal write/read round-trip; a version mismatch or absent row yields the neutral multiplier `1.0` (not an error); recall over an empty `memory_signal` table returns base-text-relevance ranking (cold-start). A grep confirms no signal value is written into the authored `notes/*.md` fact layer.
- **Handoff:** T-14B01 (ranking) and T-14B02 (maintenance writes signals) both depend on this table; T-14C* recall reads it.
- **Notes:** The load-bearing new table (`item_id`, `signal_kind`, `value`, `version`, `computed_at`). Disposable and rebuildable — dropping it must cost only ranking quality, never data.
- **Changes:** New `crates/store-local/src/memory/signal.rs`, mirroring the existing `chain.rs`/`trust.rs` shape — pure `&Connection`-taking functions, not `MemoryStore` methods (the field `conn` is private to `store.rs`'s module). `SignalKind` enum (`Centrality`/`Cluster`/`Recency`, closed set — matching only the factors this phase's MC-8 wrap actually needs, not the spec's full illustrative list) with a per-kind `current_version()`; `migrate()`/`write()`/`factor()`/`clear()` free functions; `NEUTRAL_FACTOR = 1.0` public constant. `factor()` degrades silently (no log) on absence or version mismatch — deliberately, since absence is the expected cold-start state (not an anomaly worth logging) and this sits on the recall hot path (MEM-2): one indexed point lookup, no warning call. Wired `signal::migrate` into `store.rs`'s existing `setup()`; added thin `MemoryStore::{set_signal, signal_factor, clear_signals}` methods delegating to it, matching the established `chain`/`trust` delegation pattern. `memory::mod.rs` re-exports `SignalKind`.
  Verify: 6 new unit tests in `signal.rs` (absent→neutral, write/read round-trip, overwrite-not-duplicate, independent kinds, version-mismatch→neutral, clear scoped to one item) + 1 new integration test in `tests/memory_store.rs` exercising the `MemoryStore` methods end-to-end. Grep-confirmed: no `memory_signal` write path touches `title`/`body` (the authored fact columns) — the boundary holds by construction (separate table, separate module).

### [T-14A02] `depth` + `lifecycle_state` columns (MC-1, MI-9)

- **Spec:** l2-memory-consolidation.md §4.2/MC-1; l2-memory-intelligence.md §4.6/MI-9
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local`: a `depth` enum column (`raw`/`working`/`consolidated`) exists and a write asserting a raw-tier item is immutable (never rewritten by a consolidation write); a `lifecycle_state` column (`active`/`paused`/`archived`/`deleted`) defaults to `active`, transitions append to an audit (actor/instant/old→new), and a **prune-protective** test proves MEM-5 decay does not delete a `paused`/`archived` item (it may lower ranking only). Recall defaults to `active`-only.
- **Handoff:** MC-6 `archive` (T-14B02) sets `lifecycle_state = archived`; the prune guard must exist before that action ships.
- **Notes:** Two schema columns + write-path assertions; independent of the passes.
- **Changes:** Added `MemoryDepth` (`Raw`/`Working`/`Consolidated`) and `LifecycleState` (`Active`/`Paused`/`Archived` — `Deleted` is **not** a stored variant; per MI-9's own table it is realized by the existing hard `delete()`, not a state value) to `cronus-contract` alongside `MemoryEntry`, which gained `depth`/`lifecycle_state` fields. `MemoryEntry::new()` defaults `depth: Consolidated` (every pre-existing call site — auth, `cronus memory store`, session capture — already writes a discrete finished fact, not raw evidence; defaulting to `Raw` would misclassify 100% of existing behavior) and `lifecycle_state: Active` (MI-9's own default). Added a `with_depth()` builder for future raw/working ingestion. Grep-confirmed only one struct-literal construction site existed (`store.rs`'s `map_row`) — updated it; every other call site used `MemoryEntry::new()`, so this is source-compatible.
  `store.rs`: `memories` table gained `depth`/`lifecycle_state` columns (both `NOT NULL DEFAULT`, so existing rows on schema upgrade — and every pre-existing test — read back as `Consolidated`/`Active` with no migration step needed); `add()`/`get()`/`export_all()` updated; `search_fts()` gained `AND lifecycle_state = 'Active'` to its WHERE clause (MI-9: "recall defaults to active only" — behavior-neutral today since nothing is ever anything but `Active` yet, becomes load-bearing once T-14B02 ships `archive`). New `lifecycle_audit` table (`item_id`, `actor`, `instant`, `old_state`, `new_state`) and `MemoryStore::{lifecycle_state, set_lifecycle_state}` methods — the latter is a read-then-write-then-audit sequence returning the prior state (`None` if the item doesn't exist — no transition, no audit row, no error).
  **Decision Record — prune guard scope, disclosed:** MEM-5 (decay/prune) has **no existing implementation anywhere in this codebase** — grepped for `decay`/`prune` and found nothing memory-related (only unrelated `autonomy`/`checkpoint`/`doctor` pruning). `l2-memory-store.md`'s MEM-5 is itself unrealized; it is not `Implements:`-scoped to this phase (Phase 14 realizes MC-1…10/MI-1…13, not MEM-5). The prune-protective invariant is therefore satisfied **vacuously** today (nothing can delete via decay, because decay doesn't exist) rather than actively enforced against a real decay path. What *is* real and tested: `lifecycle_state` transitions, the audit trail, and default-recall exclusion of non-`Active` items — the concrete precondition MC-6's `archive` action (T-14B02) needs. Flagged for whoever eventually realizes MEM-5's L2: it must check `lifecycle_state ∈ {Paused, Archived}` before any automatic delete, per this note.
  **Also corrects a scope error in this phase's own Planning Audit** (see Phase Notes below): the "Crate placement" guidance drafted at task-generation time said consolidation/maintenance policy belongs in `cronus-domain`. Building against the real schema showed this is wrong — MC-1…10's logic reads/writes `memories`/`memory_signal`/`lifecycle_audit` directly and has no consumer outside the store adapter, the exact shape of the T-13B01 chain/trust precedent ("when a domain half has zero consumers besides the infra half being extracted, move both together into the adapter"). Corrected inline; no domain-tier module was started under the old guidance, so nothing needed to move.
  Verify: `cargo build --workspace --all-targets` clean on first attempt (contract's new required fields caught by the compiler at the one literal-construction site, `map_row`, and nowhere else). 8 new integration tests in `tests/memory_store.rs` (depth default + explicit-raw round-trip; lifecycle default + transition + unknown-id no-op + audited-transition + default-recall-excludes-paused/archived) + the T-14A01 integration test, all passing. Full workspace suite: 1,266 passed / 0 failed (was 1,252 after Phase 13 — +14 new: +6 `signal.rs` unit tests (T-14A01) + 8 integration tests in `tests/memory_store.rs` split 1 signal / 7 depth-lifecycle across T-14A01/A02). `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all` clean.

### [T-14B01] Multiplicative ranking wrap (MC-8)

- **Spec:** l2-memory-consolidation.md §4.4, MC-8
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` proves `final = base_text_relevance × centrality × cluster × recency`, each derived factor read from `memory_signal` (T-14A01) and defaulting to 1 when absent; a near-zero factor **vetoes** a hit (multiplicative, not averaged); the recall path issues **no model call and no graph walk** (factors precomputed) — assert via a recall that touches no generator seam. `l2-memory-store` §4.2 is unmodified (base relevance unchanged); a `cargo test -p cronus` bench/test confirms existing recall behavior is preserved when the derived layer is empty.
- **Handoff:** T-14C01 recall/`answer` rank through this; T-14T01 asserts the no-hot-path-walk property.
- **Notes:** Composes §4.2's additive fuse as the text factor — does **not** edit it (§4.4). Depends on T-14A01.
- **Changes:** New `MemoryStore::search_ranked(query, limit) -> Vec<(MemoryEntry, f64)>`, added alongside (not replacing) the existing `search_fts`/`MemorySearch::search_fts` — that trait method and its sole consumer (`ContextRouter`) are untouched, zero regression risk. `base_text_relevance` is computed from SQLite FTS5's own `bm25(memories_fts)` (previously unused by this codebase — the existing `search_fts` never extracted a score, only a match/no-match list), transformed `strength = -bm25; relevance = strength / (1 + strength)` into a bounded, monotonic (0, 1) score (FTS5's bm25 is negative-is-better; this flips and bounds it). Final score = `base_text_relevance × signal_factor(Centrality) × signal_factor(Cluster) × signal_factor(Recency)`, each factor from T-14A01's `memory_signal` table via the existing `MemoryStore::signal_factor`. Results filtered by the same trust/superseded/active-lifecycle predicate as `search_fts` (post-fetch, via `get()` — same under-fill characteristic as the existing method when a FTS5-matched hit is later filtered out, not a new limitation) and sorted descending by fused score.
  **Scope correction made before shipping:** the first draft added a LIKE-substring fallback for FTS5 MATCH misses (mirroring `l2-memory-store.md`'s §4.2.1 "multi-script lexical robustness"). Caught in self-review: `search_fts` **itself has no such fallback today** (§4.2.1 is unrealized in this codebase, another instance of the T-14A02 finding — the store spec describes more than is built) and MC-8 is about signal fusion, not lexical robustness — an unrelated, already-Stable-but-unimplemented spec section outside this phase's `Implements:` scope. Removed the fallback branch and its test before it shipped; `search_ranked` now inherits exactly what `memories_fts` finds, nothing more.
  Verify: 4 new integration tests — cold-start (empty `memory_signal`) ranks by text relevance alone, matching plain-relevance order; a centrality factor crushed to `0.001` **vetoes** an otherwise-dominant text match (proves multiplicative, not additive/averaged, fusion); trust/superseded/paused/archived exclusion holds under the ranked path too; a non-matching query returns an empty `Vec`, not an error. Full workspace suite: 1,270 passed / 0 failed (1,266 + 4). Clippy `-D warnings` clean; fmt clean.

### [T-14B02] Corpus-maintenance pass (MC-6 minus MC-7)

- **Spec:** l2-memory-consolidation.md §4.3/§4.6, MC-6 (MC-7 moved to T-14B03)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` / `-p cronus-domain`: the pass runs actions in blast-radius order (archive → split → summarize → merge); `merge` requires the elevated multi-sample gate and, when unmet, surfaces to adjudication rather than acting; `archive`/`summarize` auto-apply; an `anti_cycle` cooldown blocks a split→merge→split oscillation; every action appends to `maintenance_audit`. MC-7: a stable/large/diverse/hub-less cluster (over the MC-3 edge graph, via the community-detection **seam** with a union-find stub) gains a size-bounded summary node grounded in its members. A no-generator run completes as a successful no-op.
- **Handoff:** Sets the MI-9 `archived` state (needs T-14A02); consumes MC-3 edges (T-14B03).
- **Notes:** Community detection is a seam + stub (domain-logic-first). Depends on T-14A01, T-14A02.
- **Changes:** New `crates/store-local/src/memory/maintenance.rs`, same shape as `signal.rs` (pure `&Connection` functions delegated to by thin `MemoryStore` methods). Two new tables: `maintenance_audit` (item_id/action/target_id/actor/instant) and `maintenance_cooldown` (item_id/action/until, 1-hour `ANTI_CYCLE_COOLDOWN_SECS`). Delivered three of MC-6's four actions plus the recency-signal computation step:
  - **`recompute_recency`** — exponential decay (`exp(-age·ln2/halflife)`, 30-day half-life stub) writing the `Recency` signal (T-14A01's table) for every active item. This is "maintenance step 1: recompute derived signals," recency-only — centrality/cluster need the MC-3 edge graph.
  - **`sweep_archive`** — auto-applies (reversible, no gate): archives an item whose recency has decayed past `ARCHIVE_RECENCY_THRESHOLD`, the threshold divided by `(1 + centrality)` so a hub is harder to archive, exactly the "cushioned by centrality" design (today centrality reads the T-14A01 neutral default of 1.0 for every item, since nothing computes it yet — the formula is correct and already tested against a non-default value once T-14B03 starts writing centrality). Respects cooldown; sets `lifecycle_state = Archived` via the T-14A02 machinery (audited).
  - **`touch`** — auto-thaw on touch (MC-6): flips an `Archived` item back to `Active`, audited; no-op otherwise. Not yet wired into any implicit "touch" point (recall/update) — those land with Track C; this ships the primitive.
  - **`flag_split_candidates`** — a length-threshold heuristic (4,000 chars) identifies overload candidates. Deliberately does **not** split them: real splitting is topic segmentation, a generator-dependent operation with no safe non-model heuristic (unlike archive's clean decay formula) — flagging-only is the honest no-generator no-op, extending MC-2's own contract to MC-6.
  - **`find_merge_candidates` / `merge_pair`** — redundancy detection via case/whitespace-normalized exact body match (the domain-logic-first stand-in for MC-6's "multi-sample agreement" elevated gate — unambiguous by construction, not a similarity heuristic that could misfire). `merge_pair` re-points `memory_chains` edges from the discarded item onto the kept one, clears the discard's signals, hard-deletes it, all inside one SQLite transaction (commit-or-rollback, MC-9).
  **Decision Record — MC-7 deferred to T-14B03, disclosed:** MC-7 (emergent topic-cluster summaries) needs community detection over the MC-3 edge graph, which does not exist until T-14B03 builds it. Implementing MC-7 here would mean operating on a graph that is always empty. Moved MC-7 into T-14B03's scope (it lands the edges; summarizing over them belongs with them) — not a scope cut, a sequencing fix caught before code was written against a nonexistent dependency. `l2-memory-consolidation`'s own Verify line and this phase's Planning Audit did not flag this edge dependency at plan time; corrected here rather than silently building the wrong order.
  Verify: 7 new unit tests in `maintenance.rs` (fresh-item recency ≈ neutral; recency decays past threshold with age; archive shelves a stale item, leaves a fresh one; touch thaws Archived only; split-flag finds only long bodies; merge-candidate pairing picks the normalized duplicate and keeps the newer; merge_pair re-points chains and hard-deletes) + 4 new integration tests in `tests/memory_store.rs` proving the `MemoryStore`-level delegation wiring. Full workspace suite: 1,281 passed / 0 failed (1,270 + 11: 7 unit + 4 integration). Clippy `-D warnings` clean; fmt clean. String literals for lifecycle states parameterized through `LifecycleState::as_str()` throughout (not hardcoded SQL string literals), so a future rename of the enum's string form can't silently desync from the SQL.

### [T-14B03] Consolidation write path (MC-2/3/4/9/10)

- **Spec:** l2-memory-consolidation.md §4.1/§4.3/§4.5, MC-2, MC-3, MC-4, MC-9, MC-10
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: write-time additive-only edges — a prose edit may add but never drop an edge (conservation check refuses the drop) and every consolidated item carries a `derived-from` provenance edge (MC-3); the closed `create/corroborate/refine/correct` action is recorded per write (MC-4); an incremental watermark reprocesses a failed-not-checkpointed input on the next pass and no-ops on no change (MC-2); optimistic-concurrency + conservation-check refuses a stale/invalid write and a multi-item merge/split is transactional whole-or-rollback (MC-9); the pass emits a bounded, diversity-windowed, read-only interest-topic set (MC-10).
- **Handoff:** Feeds T-14B02 (edges to cluster) and the MI-4 adjudication (T-14C02, upstream MC-4).
- **Notes:** Extends the §4.12 two-phase pipeline; the domain tier holds the action/gate policy, the adapter the SQLite writes. Depends on T-14A01/A02.

### [T-14C01] `answer` + temporal + predicate + visibility (MI-1/2/3/8)

- **Spec:** l2-memory-intelligence.md §4.1/§4.2/§4.5, MI-1, MI-2, MI-3, MI-8
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: `answer` retrieves a grounding set, synthesizes grounded only in it, attaches KB-6 citations, and passes the CV-3/CV-4 gate — an under-grounded query returns an honest `insufficient` outcome, never a model-prior assertion; with no generator it returns top attributed items verbatim (extractive). Temporal modes `as-of`/`changed-since`/`recent` resolve over the §4.14 record and compose with type/scope/tag filters; `changed-since(checkpoint)` yields the session delta. The structured predicate (eq/ne/ordering/membership/containment + AND/OR/NOT) compiles to a parameterized SQL `WHERE`, falling back to post-fetch evaluation for an unsupported nesting (never dropping the constraint). A freshly-written item is recall-visible immediately (MI-3), enrichment deferred.
- **Handoff:** T-14C02 digest reuses `answer` over a time window; T-14C03 reuse recall composes MI-2/MI-8.
- **Notes:** `answer` is a projection composing existing engines, not a new engine. Depends on T-14B01 (ranking).

### [T-14C02] Conflict routing, digest, capture (MI-4/5/6/7/10/11/12)

- **Spec:** l2-memory-intelligence.md §4.3/§4.4/§4.7–§4.11, MI-4/5/6/7/10/11/12
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: conflict routing classifies (contradiction/update/duplicate/conflict) and applies the **pinned threshold** (§4.3: duplicate → auto de-dupe; recency-dominant higher-confidence → auto-supersede; comparable-confidence-and-trust with no recency winner → surface a report with the closed recommendation set) (MI-4); the digest is a scheduler-fired read-only narrative + analytics + conflict report on the **pinned cadence** (§4.4: per-session-close + daily floor, opt-in per office) that mutates nothing (MI-5); capture is salience-gated/confidence-honest/de-duplicated with actor/expiry/subject metadata (MI-6); an end-of-run `distill_run` writes one procedure memory grounded in the trace (MI-7); relative dates normalize to absolute against the observation instant, verbatim with no generator (MI-10); include/exclude/custom directives steer emphasis without lowering the honesty floor (MI-11); `raw` mode stores verbatim with no model bound, `inferred` is the default (MI-12).
- **Handoff:** Conflict adjudication consumes the MC-4 action output (T-14B03).
- **Notes:** Largest breadth task; several are policy over existing §4.15 fields. Depends on T-14C01.

### [T-14C03] Gated experience reuse (MI-13)

- **Spec:** l2-memory-intelligence.md §4.12, MI-13
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: a high-quality prior `success` is reused directly **only** when the gate passes (similarity ≥ σ AND score ≥ τ AND fresh AND not safety-sensitive); a `failure` injects as an avoid-signal and is never reused as a solution; an `insight` injects as guidance; a reused result is attributed `reused_from` (never passed as fresh, MI-1 honesty); a reused plan still passes the action's SEC-9/SEC-10 authority gate (reuse buys speed, never a bypass); read and write are independently controllable; every outcome is captured back typed+scored.
- **Handoff:** Closes the phase's net-new query surface; validated end-to-end in T-14T01.
- **Notes:** Composes MI-1/2/6/7/8 + the security gate — build last, on their landed realizations.

### [T-14T01] Validation: invariant-compliance sweep + degradation

- **Goal:** Prove every MC-1…MC-10 and MI-1…MI-13 invariant holds on the built tier, cold-start works, and the local-first / hot-path constraints are respected.
- **Method:** One test per MC/MI invariant (exercising the layers together — consolidation write → maintenance → recall/answer); a cold-start test (empty `memory_signal`) yields base-relevance ranking; a no-generator run degrades every model-dependent op (consolidation, summary, `answer`, normalization, inferred capture) rather than failing; an assertion that the recall/`answer` hot path issues no model call and no graph walk (MC-8/MEM-2); the fact-vs-derived boundary holds (no signal in `notes/*.md`).
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test --workspace` green (memory tests added, pre-existing count preserved) + `cargo clippy --workspace --all-targets -- -D warnings` clean + `cargo fmt --all -- --check` clean.

## Phase Notes (Planning Audit)

- **Foundation-then-parallel.** Track A (T-14A01 signal table, T-14A02 depth/lifecycle columns) is a hard prerequisite for B and C. After A, B and C parallelize on file independence. Two cross-edges the orchestrator must respect: MC-8 ranking (B01) is consumed by intelligence recall (C01), and MC-6 `archive` (B02) sets the MI-9 `lifecycle_state` (A02) — so A02 gates B02, and A01 gates B01 + C01's ranking.
- **Generator-optional is a build requirement, not a deferral.** Every model-dependent step ships its no-generator degrade path in the same task (local-first, §2 of both specs). A task is not Done if its degrade path is only asserted, not built.
- **Domain-logic-first.** Community detection (MC-7) and the LLM calls are seams with stubs; the phase proves policy/gates/degradation, not ML quality — mirroring Phases 9–12.
- **No hot-path regression.** MC-8/MEM-2: ranking reads precomputed signals only; T-14B01 and T-14T01 both assert no model call / graph walk on recall.
- **Crate placement — corrected during T-14A02 (Decision Record).** Originally drafted as "SQLite/ranking/maintenance → store-local; policy → domain." Building against the real schema showed the policy has no consumer outside the store adapter (same shape as T-13B01's chain/trust precedent), so **all of MC-1…10's consolidation/maintenance/ranking logic lives in `cronus-store-local`** — no domain-tier split. Shared value types (`MemoryDepth`, `LifecycleState`, `SignalKind` if ever needed cross-crate) → `cronus-contract`. The MI-1…13 intelligence surface (Track C) composes `MemorySearch`/`UserDataStore` through the seam and may legitimately need a thin `cronus-domain` orchestration layer for `answer`'s KB-6/CV-3/4 composition (those engines are domain-tier) — re-evaluate per-task, don't assume.

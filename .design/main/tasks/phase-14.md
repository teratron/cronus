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

- [ ] [T-14A01] `memory_signal` fact-vs-derived table + version-guarded neutral-default degradation (MC-5)
- [ ] [T-14A02] `depth` + `lifecycle_state` columns with never-rewrite-raw (MC-1) and prune-protective (MI-9) guards
- [ ] [T-14B01] Multiplicative offline-precomputed ranking wrapping §4.2 base relevance (MC-8)
- [ ] [T-14B02] Corpus-maintenance pass — archive/split/summarize/merge + emergent summaries (MC-6, MC-7)
- [ ] [T-14B03] Consolidation write: additive edges, action algebra, incremental watermark, co-edit safety (MC-2/3/4/9/10)
- [ ] [T-14C01] `answer` projection + temporal modes + structured predicate + recall-visibility (MI-1/2/3/8)
- [ ] [T-14C02] Conflict routing, digest, capture policy/directives/normalization/raw + distillation (MI-4/5/6/7/10/11/12)
- [ ] [T-14C03] Gated experience reuse with retained authority gate (MI-13)
- [ ] [T-14T01] Validation: MC/MI invariant-compliance sweep + cold-start + generator-degradation

## Detailed Tracking

### [T-14A01] `memory_signal` fact-vs-derived table (MC-5)

- **Spec:** l2-memory-consolidation.md §4.2, MC-5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` covers: a signal write/read round-trip; a version mismatch or absent row yields the neutral multiplier `1.0` (not an error); recall over an empty `memory_signal` table returns base-text-relevance ranking (cold-start). A grep confirms no signal value is written into the authored `notes/*.md` fact layer.
- **Handoff:** T-14B01 (ranking) and T-14B02 (maintenance writes signals) both depend on this table; T-14C* recall reads it.
- **Notes:** The load-bearing new table (`item_id`, `signal_kind`, `value`, `version`, `computed_at`). Disposable and rebuildable — dropping it must cost only ranking quality, never data.

### [T-14A02] `depth` + `lifecycle_state` columns (MC-1, MI-9)

- **Spec:** l2-memory-consolidation.md §4.2/MC-1; l2-memory-intelligence.md §4.6/MI-9
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local`: a `depth` enum column (`raw`/`working`/`consolidated`) exists and a write asserting a raw-tier item is immutable (never rewritten by a consolidation write); a `lifecycle_state` column (`active`/`paused`/`archived`/`deleted`) defaults to `active`, transitions append to an audit (actor/instant/old→new), and a **prune-protective** test proves MEM-5 decay does not delete a `paused`/`archived` item (it may lower ranking only). Recall defaults to `active`-only.
- **Handoff:** MC-6 `archive` (T-14B02) sets `lifecycle_state = archived`; the prune guard must exist before that action ships.
- **Notes:** Two schema columns + write-path assertions; independent of the passes.

### [T-14B01] Multiplicative ranking wrap (MC-8)

- **Spec:** l2-memory-consolidation.md §4.4, MC-8
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` proves `final = base_text_relevance × centrality × cluster × recency`, each derived factor read from `memory_signal` (T-14A01) and defaulting to 1 when absent; a near-zero factor **vetoes** a hit (multiplicative, not averaged); the recall path issues **no model call and no graph walk** (factors precomputed) — assert via a recall that touches no generator seam. `l2-memory-store` §4.2 is unmodified (base relevance unchanged); a `cargo test -p cronus` bench/test confirms existing recall behavior is preserved when the derived layer is empty.
- **Handoff:** T-14C01 recall/`answer` rank through this; T-14T01 asserts the no-hot-path-walk property.
- **Notes:** Composes §4.2's additive fuse as the text factor — does **not** edit it (§4.4). Depends on T-14A01.

### [T-14B02] Corpus-maintenance pass (MC-6, MC-7)

- **Spec:** l2-memory-consolidation.md §4.3/§4.6, MC-6, MC-7
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` / `-p cronus-domain`: the pass runs actions in blast-radius order (archive → split → summarize → merge); `merge` requires the elevated multi-sample gate and, when unmet, surfaces to adjudication rather than acting; `archive`/`summarize` auto-apply; an `anti_cycle` cooldown blocks a split→merge→split oscillation; every action appends to `maintenance_audit`. MC-7: a stable/large/diverse/hub-less cluster (over the MC-3 edge graph, via the community-detection **seam** with a union-find stub) gains a size-bounded summary node grounded in its members. A no-generator run completes as a successful no-op.
- **Handoff:** Sets the MI-9 `archived` state (needs T-14A02); consumes MC-3 edges (T-14B03).
- **Notes:** Community detection is a seam + stub (domain-logic-first). Depends on T-14A01, T-14A02.

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
- **Crate placement.** SQLite schema/SQL/ranking/maintenance → `cronus-store-local`; action/gate/routing policy → `cronus-domain`; any seam extension → `cronus-contract`. All reached through `UserDataStore` — the tier stays backend-swappable.

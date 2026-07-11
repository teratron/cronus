---
phase: 15
name: "Memory Capture Policy & Metadata (L2)"
status: Todo
subsystem: "crates/contract (MemoryEntry capture-metadata fields) · crates/store-local (SQLite schema + capture write path) · crates/domain (capture policy, directives, normalization over the MemorySearch/UserDataStore seam): the write-side completion of l2-memory-intelligence, sibling to Phase 14's query surface"
requires: [4, 14]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 15 Tasks — Memory Capture Policy & Metadata (L2)

**Phase:** 15
**Status:** Todo
**Strategic Goal:** Complete `l2-memory-intelligence` at **capture scope** — the four write-path invariants (MI-6/10/11/12) deferred at Phase 14's delivered scope. Phase 14 built how a memory is *queried*; this phase builds how a memory is *written*: the salience-gated capture policy with net-new actor/expiry/subject/cross-ref metadata (MI-6), capture-time relative→absolute date normalization (MI-10), caller capture directives (MI-11), and the raw-vs-inferred write mode (MI-12). One coherent unit — all four share the capture entry point and the generator-optional degrade contract. Behind the `UserDataStore` seam, over the existing SQLite substrate. Not a reopen of the immutable Phase 14 — a new phase for the same Stable spec's remaining scope (source-layout 1.1.0/1.2.0 precedent).

> **Domain-logic-first (Phases 9–14 precedent).** The `inferred`-mode model extraction (MI-12) is a **seam with a stub** — this phase proves the capture policy, the confidence/salience gate, the metadata degrade-to-baseline, the directive honesty floor, and the no-generator paths, not extraction quality. **Generator-optional is the load-bearing acceptance property, not an afterthought**: MI-10 must store verbatim when no generator is bound (never fabricate a date), and MI-12's `raw` mode IS the no-generator escape hatch. Acceptance = each of MI-6/10/11/12 covered by a test + every optional metadata field degrades to baseline when absent + every generator-dependent step has a tested no-generator degrade.

## Atomic Checklist

- [x] [T-15A01] Capture-metadata schema: `actor`/`expiry`/`subject` fields on `MemoryEntry` + columns, absent-by-default (cross-ref reuses the existing MC-3 `add_edge`, no new field)
- [ ] [T-15B01] Capture policy: salience/confidence gate + semantic dedup + metadata attribution + expiry-void + cross-ref forward edges (MI-6)
- [ ] [T-15B02] Capture-time temporal normalization (MI-10) + raw/inferred write mode (MI-12) — the two generator-optional write-time content transforms
- [ ] [T-15B03] Caller capture directives: `include`/`exclude`/`custom-instruction` recorded as provenance, never lowering the honesty floor (MI-11)
- [ ] [T-15T01] Validation: MI-6/10/11/12 invariant sweep + absent-field-degrades-to-baseline + no-generator degrade

## Detailed Tracking

### [T-15A01] Capture-metadata schema (MI-6 net-new fields)

- **Spec:** l2-memory-intelligence.md §3 (MI-6 Invariant Compliance row); l2-memory-store.md §4.15 (existing MemoryEntry field reference the new fields extend)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` + `-p cronus-contract`: `MemoryEntry` gains `actor: Option<String>` (who-said-it, distinct from scope ownership), `expiry: Option<u64>` (a hard void-after instant, complementing MEM-5 decay — not a decay knob), and `subject: Option<MemorySubject>` (a new closed 2-variant enum: `User` | `AgentSelf`). Each maps to a nullable column so the **entire pre-existing corpus reads back at baseline with no migration step** (the additive-column pattern proven 4× in Phase 14 — assert an old-shape row round-trips with all three absent). A round-trip write/read test proves each field persists and `map_row` decodes it; a builder (`with_actor`/`with_expiry`/`with_subject`, mirroring the existing `with_workspace`/`with_depth`) sets them without disturbing existing call sites. **Cross-reference (the fourth MI-6 metadata item) is deliberately NOT a new field** — grepped first and found `MemoryStore::add_edge(source, target, predicate)` already public (T-14B03); MI-6's own text says cross-ref is "cheap forward MC-3 edges," so it is realized by a `capture()`-time loop calling the existing `add_edge` with a new `CROSS_REF_PREDICATE` constant, no schema change.
- **Handoff:** T-15B01 attributes/sets `actor`/`expiry`/`subject` through the capture policy and writes cross-ref edges via the existing `add_edge`; T-15B03 records directive provenance alongside them.
- **Notes:** Pure store-tier (contract + store-local). **Grep every `map_row` call site before considering this done** — the Phase 14 lesson: adding a column to `map_row`'s positional read silently breaks any independent `SELECT` that lists fewer columns (`consolidate.rs::run_incremental_pass` was caught this way). The count is now higher; enumerate all of them.
- **Changes:** Added `cronus_contract::MemorySubject` (`User`/`AgentSelf`, `as_str`/`from_db_str`, mirroring `ExperienceOutcome`'s shape) and three `MemoryEntry` fields — `actor: Option<String>`, `expiry: Option<u64>`, `subject: Option<MemorySubject>` — plus `with_actor`/`with_expiry`/`with_subject` builders. Confirmed (per T-14A02's own note) only two struct-literal construction sites exist for `MemoryEntry` (`MemoryEntry::new` and `store.rs::map_row`); both updated, every other call site already goes through `::new()` and is source-compatible.
  `store.rs`: `memories` table gained `actor TEXT`, `expiry INTEGER`, `subject TEXT` (all nullable, no `DEFAULT` — `None` is the correct value for the overwhelming majority of memories, unlike `depth`/`lifecycle_state`'s non-null defaults). Enumerated every `map_row` call site before touching anything (6 total: `get`/`search_fts`/`recall_temporal`/`recall_structured`/`export_all` in `store.rs`, plus `consolidate.rs::run_incremental_pass`'s independent `SELECT`) — all 6 updated consistently this time, avoiding the exact miss T-14C03 self-caught. `insert()` and `map_row` extended positionally (indices 15/16/17).
  **Cross-reference confirmed as a non-issue, not deferred:** grepped for `add_edge` before writing anything and found `MemoryStore::add_edge(source, target, predicate) -> Result<()>` already `pub fn` (T-14B03) — MI-6's cross-ref metadata needs no schema at all, only a `CROSS_REF_PREDICATE` constant and a capture-time loop, both left to T-15B01 where the capture path actually exists to call from.
  `cronus_store_local::memory::mod.rs`'s contract re-export list extended with `MemorySubject` (matching the convenience-re-export precedent already applied to `ExperienceOutcome` et al.).
  Verify: 4 new integration tests in `tests/memory_store.rs` (all three fields default to `None`; `actor` round-trips; `expiry` round-trips; `subject` round-trips both variants). Full workspace suite: 1,337 passed / 0 failed (1,333 + 4). Clippy `-D warnings` clean for both touched crates; fmt clean.

### [T-15B01] Capture policy (MI-6)

- **Spec:** l2-memory-intelligence.md §3 (MI-6); composes l2-memory-store.md §4.3 (dedup) / §4.6 (trust) / §4.15 (fields)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: a `capture()` entry point applies the policy — **selective + confidence-honest** (an item whose write-time `confidence` is below the floor is either not stored or stored provisionally, never silently promoted; assert the below-floor path), **de-duplicated** (reuses the normalized-body match already built for MC-4 corroborate / MI-4 duplicate — assert a near-identical capture corroborates rather than duplicates), **actor-attributed** and **subject-attributed** (the T-15A01 fields set from the call), **optionally expirable** (an item past its `expiry` instant is excluded from default recall — assert an expired item does not surface while a non-expired one does), and **cross-referenced** (a capture naming related ids writes forward edges via the existing `add_edge` with a new `CROSS_REF_PREDICATE` — assert the edges land and are readable via `edges_from`). Every optional field **degrades to baseline when absent** (a capture with no actor/expiry/subject/cross-refs behaves exactly as a plain `add`, asserted).
- **Handoff:** T-15B02 (normalization/mode) and T-15B03 (directives) feed content into this same path; T-15T01 sweeps the composed behavior.
- **Notes:** Crate placement decided at execution per the Phase 14 correction — the policy is domain-tier (`cronus-domain`, over the `MemorySearch`/`UserDataStore` seam) **unless** a grep shows it has zero consumers outside the adapter, in which case it stays store-tier (the T-13B01/T-14B03 precedent). Reuse the dedup and forward-edge machinery from Phase 14; do not reinvent. Shares the capture entry point with B02/B03 — the orchestrator serializes edits to it (shared-surface constraint).

### [T-15B02] Capture-time normalization (MI-10) + raw/inferred mode (MI-12)

- **Spec:** l2-memory-intelligence.md §3 (MI-10, MI-12); §5 Implementation Note 6 (generator-availability check); l1-memory-intelligence.md §4.11 (raw vs inferred)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: **MI-10** — at capture, relative temporal expressions in the content ("last week", "yesterday") are rewritten to absolute dates against the **observation instant** (defaulting to write time), normalizing the MEM-4 source text (distinct from and complementary to the §4.14 bi-temporal metadata). **No generator bound → store verbatim, never fabricate a date** (assert: with no generator, content is unchanged, not guessed). **MI-12** — a per-write mode flag: `inferred` (model extracts salient facts; the **default**) or `raw` (verbatim, no model). `raw` MUST function with **no model bound** (assert a `raw` capture with no generator produces an ordinary, immediately-recallable item). Both modes produce ordinary items — uniform recall/lifecycle/trust/decay (assert a `raw` item and an `inferred` item are indistinguishable to the recall path). The `inferred` extraction itself is a **seam+stub** (a bound generator is not required to exist this phase).
- **Handoff:** Feeds the T-15B01 capture path; T-15T01 asserts the no-generator degrade across both.
- **Notes:** MI-10 and MI-12 are combined because both are write-time content transforms governed by the **same generator-optional contract** — MI-12's `raw` mode is precisely "skip the model step MI-10's normalization and inferred extraction would use." Keeping them one task keeps that shared degrade path in one place. The `inferred` model call is the only seam; everything else (mode dispatch, verbatim path, recall uniformity) is real.

### [T-15B03] Caller capture directives (MI-11)

- **Spec:** l2-memory-intelligence.md §3 (MI-11)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test`: optional caller-scoped `include` / `exclude` / `custom-instruction` inputs steer extraction emphasis and are **recorded as capture provenance** (assert the directives persist with the item). Two **negative invariants**, each asserted explicitly: a directive **never lowers the MI-6 honesty floor** (an `exclude` cannot force a below-confidence item to be stored as if confident) and **never suppresses a safety-relevant fact** (an `exclude` targeting a safety-relevant fact does not drop it — assert the fact survives the directive). **Absent directives → baseline MI-6** (assert a capture with no directives is byte-identical to the same capture through T-15B01's plain path).
- **Handoff:** T-15T01 sweeps the honesty-floor and safety-suppression negative invariants.
- **Notes:** Smallest of the B tasks — directive inputs + provenance recording over the T-15B01 path. The two negative invariants are the substance; the happy path is thin. Shares the capture entry point (serialize with B01/B02).

### [T-15T01] Validation: MI-6/10/11/12 invariant sweep + degradation

- **Goal:** Prove each of MI-6/10/11/12 holds on the built capture path, every optional metadata field degrades to baseline when absent, and every generator-dependent step has a tested no-generator degrade.
- **Method:** One cross-layer test per invariant through the real facade + SQLite adapter (mirroring Phase 14's `crates/core/tests/memory_validation.rs` approach, extending that file rather than forking a new one where natural): MI-6 capture policy with all four metadata fields set and separately all absent (baseline); MI-10 relative-date normalization with a generator stub and with none (verbatim); MI-12 `raw` capture with no generator bound yielding an immediately-recallable ordinary item; MI-11 directives recorded as provenance plus the two negative invariants (honesty floor not lowered, safety-relevant fact not suppressed). A no-generator run degrades MI-10 (verbatim) and MI-12 (`raw`) rather than failing.
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test --workspace` green (capture-path tests added, pre-existing count preserved) + `cargo clippy --workspace --all-targets -- -D warnings` clean + `cargo fmt --all -- --check` clean.

## Phase Notes (Planning Audit)

- **Foundation-then-parallel, same shape as Phase 14.** Track A (T-15A01 capture-metadata schema) is a hard prerequisite for every Track B task — the policy sets `actor`/`subject` and the void check reads `expiry` (cross-ref itself needs no new field, only the already-public `add_edge`). After A lands, B01/B02/B03 are logically independent but **share the `capture()` entry point**, so the orchestrator serializes edits to it (shared-surface constraint, exactly the Phase 13 shared-`Cargo.toml` serialization pattern). Track T runs last.
- **Cascade risk concentrated on T-15A01** — the same foundation-gates-everything shape as Phase 14's Track A. Mitigated by the additive-column pattern being proven 4× already; the only new risk is the `map_row` call-site enumeration (Phase 14 caught a missed `SELECT` this way — T-15A01's Notes make grepping every call site an explicit gate).
- **Optimism-bias check (`@role:planner`):** T-15B01 is the heaviest task (four metadata fields + salience gate + dedup + forward edges), but dedup (normalized-body match) and forward edges (`add_edge`) already exist from Phase 14 — B01 is mostly wiring + the confidence gate, not net-new machinery. T-15B02 combines two invariants deliberately (shared generator-optional contract), which is a consolidation, not an under-estimate. Sizing judged realistic.
- **Generator-optional is a build requirement, not deferred.** Both MI-10 and MI-12 have a no-generator path that MUST be built and tested this phase; the `inferred` extraction is the only stubbed seam. This is the Phase 9–14 domain-logic-first discipline applied to the capture side.
- **Crate placement is a run-time decision, not pinned here** — the Phase 14 correction (a policy with zero consumers outside the adapter stays store-tier despite task-generation-time guidance suggesting domain-tier). Schema is unambiguously store-tier; the policy/directives/normalization lean domain-tier over the seam, verified by grep at execution.
- **Task Instruction Review (`@role:prompt-engineer`, PQ-6): PASS.** Every task carries a concrete `Verify` with named assertions; the two MI-11 negative invariants (honesty floor, safety-suppression) are stated as explicit asserted checks in both T-15B03 and T-15T01 rather than left implicit — the one dimension most likely to be under-tested in a "directives" task, called out deliberately.
- **Not a reopen.** Phase 14 stays `Done`/archived at its delivered scope (MI-1/2/3/4/5/7/8/9/13 + MC-1…10). This phase is the deferred MI-6/10/11/12 delta as a new line item — the source-layout 1.1.0/1.2.0 and KAN-8 precedents (Done phases immutable; the spec-vs-code gap tracked as a forward delta, never a silent absorption or a reopen).

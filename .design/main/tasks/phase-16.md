---
phase: 16
name: "Project Wiki Store (L2)"
status: Todo
subsystem: "crates/contract (WikiPage/WikiCitation types + read query surface) · crates/store-local (wiki.db schema: wiki_page tree + wiki_changelog + wiki_page_fts, persistence, read-only query, rebuild) · crates/domain (office-owned regeneration pipeline: event→page mapping, ground-truth gather, PW-4 citation guard, PW-8 internal-detail filter, source_fingerprint + PW-5 stale detection over the store seam) · crates/auth-local (access-grants Read gate for PW-7): the client-facing projection cache realizing l2-project-wiki — a rebuildable cache of upstream truth, distinct from the memory learned-tier where the store IS truth"
requires: [4, 5, 11, 13]
provides: []
key_files: []
patterns_established: []
duration_minutes: ~
---

# Stage 16 Tasks — Project Wiki Store (L2)

**Phase:** 16
**Status:** Todo
**Strategic Goal:** Realize `l2-project-wiki` — the client-facing living documentation materialized as a **per-office SQLite projection cache** (`<ws>/wiki/wiki.db`), not loose Markdown. Build the store (schema + read surface + rebuild), the office-owned regeneration pipeline (event-driven, incremental, transactional), and the grounding/freshness/access guards. The load-bearing property is **projection-not-source (PW-3)**: `rebuild()` reconstructs an equivalent wiki from ground truth (board / graph decisions / operational ledger / deliverables), so nothing authoritative ever lives only in `wiki.db`. This contrasts deliberately with the memory learned-tier, where the store *is* the source of truth — here the store is only a rebuildable cache of upstream truth.

> **Domain-logic-first (Phases 9–15 precedent).** Client-language page generation is model-based; this phase proves the pipeline structure, the citation/freshness/access guards, and the no-generator degrade — not prose quality. **Generator-optional is the load-bearing acceptance property:** with no generator bound, a page stores a grounded stub composed from its citations and **never fabricates** content (mirrors MI-10/MI-12). Read-only-to-client (PW-2) is enforced structurally — the client surface exposes no mutating method — not by convention. Acceptance = each of PW-2/3/4/5/6/7/8 covered by a test + `rebuild` reconstructs an equivalent wiki + dropping `wiki.db` loses nothing durable + every generator-dependent step has a tested no-generator degrade.

## Atomic Checklist

- [x] [T-16A01] Wiki schema & types: `wiki_page` (parent/ord tree, `citations` JSON, `source_fingerprint`, `stale` flag), `wiki_changelog`, `wiki_page_fts` (FTS5) + indices in `crates/store-local`; `WikiPage`/`WikiCitation` contract types, absent-by-default. **Verify:** `cargo test -p cronus-store-local wiki::schema` — tables + indices created; a page round-trips with its citations and fingerprint.
- [ ] [T-16B01] Regeneration pipeline core (PW-2/PW-3): event→affected-page mapping + ground-truth gather (board / graph decisions / ledger / deliverables) + transactional per-page upsert + `wiki_changelog` append; office-owned, no client write path. **Verify:** test — a simulated office event regenerates only the affected pages transactionally; a forced mid-regen failure rolls back, leaving prior rows intact and correctly marked.
- [ ] [T-16B02] Grounding & honesty guards (PW-4/PW-8): citation guard — every substantive claim in a stored section resolves to a cited source or is dropped, never persisted uncited; internal-detail content filter strips engineering / SDD detail; no-generator degrade stores a grounded stub from citations. **Verify:** test — an uncited claim is rejected; an internal-detail input never reaches a row; no-generator mode stores a stub and never fabricates.
- [ ] [T-16B03] Freshness (PW-5): `source_fingerprint` compute + drift check → set `stale=1` when a page's sources moved without regeneration; `wiki_changelog` surfaced newest-first. **Verify:** test — mutating a source without regen flips the page `stale` flag; the changelog records the change newest-first.
- [ ] [T-16C01] Read-only client surface (PW-2/PW-6): navigation-tree query (`parent_id`/`ord`, overview→area→detail) + `wiki_page_fts` search; no write/update endpoint exposed to the client. **Verify:** test — tree + FTS queries return the expected pages; the client-facing surface exposes no mutating method (API/compile-time check).
- [ ] [T-16C02] Access gate + rebuild (PW-3/PW-7): `access-grants` `has_access(Wiki, office_id, Read)` on every client read; `rebuild(office)` drops and re-derives all rows from ground truth. **Verify:** test — a read without the Read grant is denied; `rebuild` reconstructs an equivalent wiki (same structure + citations) and dropping `wiki.db` loses nothing durable.
- [ ] [T-16T01] Validation sweep: PW-2/3/4/5/7/8 invariant tests + drop-loses-nothing + every generator-dependent step has a tested no-generator degrade. **Verify:** `cargo test --workspace` green (new wiki tests) + clippy `-D warnings` + `cargo fmt --all --check` clean.

## Tracks

- **A — Schema (foundation):** T-16A01. Gates B and C.
- **B — Regeneration (office-owned write path):** T-16B01 → T-16B02 → T-16B03 (share the regeneration entry point → serialized).
- **C — Read & access surface:** T-16C01, T-16C02 (depend on A).
- **T — Validation:** T-16T01 (last; sweeps every PW invariant).

Foundation-then-parallel: Track A gates B/C; B is serialized on the shared regeneration entry point; C can proceed once the schema lands; T closes the phase.

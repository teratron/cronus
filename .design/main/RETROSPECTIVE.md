# SDD Retrospective

**Last Full Run:** 2026-09-11
**Full Sessions:** 1
**Snapshots:** 4

## Snapshots

Auto-collected after each phase completion. Lightweight metrics only — no analysis.

| Date | Phase | Specs (D/R/S) | Tasks (Done/Blocked/Cancelled) | Rules | Signal |
| --- | --- | --- | --- | --- | --- |
| 2026-07-11 | Phase 13 | 1/9/184 | 8/0/0 | 25 | 🟢 |
| 2026-07-11 | Phase 14 | 1/7/188 | 9/0/0 | 25 | 🟢 |
| 2026-07-11 | Phase 15 | 1/7/188 | 5/0/0 | 25 | 🟢 |
| 2026-09-11 | Phase 30 | 0/0/294 | 8/0/0 | 27 | 🟢 |

## Session 1 — 2026-09-11

**Scope:** Full system analysis — both workspaces (`main`, `nodus`), triggered manually after Phase 30 closed and the plan read "complete."
**Specs in registry:** 316 total — `main`: 294 (294 Stable, 0 RFC, 0 Draft); `nodus`: 22 (21 Stable, 1 RFC, 0 Draft)
**Tasks total:** 587 (Done: 587, Blocked: 0, Cancelled: 0) — `main`: 305 across 31 archived phases (1–30, plus the 10a/10b split); `nodus`: 282
**RULES.md §7 entries:** 27 (C1–C30, with C5/C18/C19 unused — gap noted, not investigated further this pass)

### 🚀 DORA Metrics (L2 Implementation)

| Metric | Value | Source | Details |
| --- | --- | --- | --- |
| **Deployment Frequency** | N/A | — | No deployment pipeline or environment exists yet for this project; nothing to measure. Not fabricated. |
| **Change Failure Rate** | N/A | — | Same — no deploys recorded. |

### 📊 Observations

| # | Severity | Area | Observation | Evidence |
| ---: | --- | --- | --- | --- |
| 1 | 🟡 Medium | Spec integrity (main) | Three Stable L2 specs — `l2-file-store.md`, `l2-knowledge-store.md`, `l2-notes.md` — still describe file/knowledge/notes storage as living in its own dedicated crate (`crates/file-store/`, `crates/knowledge-store/`, `crates/notes/`), each citing `l2-source-layout.md` for that placement. `l2-source-layout.md` is now v1.3.0 and lists none of the three; the actual code places this logic as modules inside `crates/domain/src/` (`file_store.rs`, `notes.rs`) and, for knowledge, split across `crates/core/src/knowledge_bootstrap.rs` + `crates/domain/src/knowledge_{access,ingest,retrieval}.rs` + `crates/store-local/src/knowledge.rs`. All three specs' domain-logic content may still be accurate; specifically the crate-placement claim, and the citation that supports it, is dead. | `grep -n "crates/file-store\|crates/knowledge-store\|crates/notes" .design/main/specifications/l2-source-layout.md` (0 hits) vs. each spec's own `crates/{file-store,knowledge-store,notes}/` reference; `find crates -iname "*knowledge*"`, `find crates -iname "*notes*"` |
| 2 | 🟡 Medium, but net-positive | Spec integrity (main) | `l2-crate-topology.md`'s Canonical Reference table names `crates/core/src/context_router.rs` as "the migration pivot" (the one inverted domain→infrastructure edge the spec's migration was built to remove). That path does not exist. The module now lives at `crates/domain/src/context_router.rs`, and the edge it names appears genuinely resolved: `crates/domain/Cargo.toml` has no dependency on `cronus-store-local`, and `crates/core/src/lib.rs` imports `context_router` *from* `domain` (the correct inward direction), not the reverse. The architecture improved; the spec's own reference to the thing it fixed was never updated to point at where it landed. | `find` for `context_router.rs` (found under `crates/domain/src/`, not `crates/core/src/`); `crates/domain/src/lib.rs:26` (`pub mod context_router;`); `crates/core/src/lib.rs:14`; `crates/domain/Cargo.toml` (no store-local dep) |
| 3 | 🟢 Low (cosmetic) | Spec integrity (main) | `l2-tool-receipts.md`'s Document History references `crates/domain/src/tool_receipts.rs` as a flat file; the module is organized as a directory (`crates/domain/src/tool_receipts/`, containing at least `receipted.rs`) — a behavior-neutral Rust reorganization the prose never followed. | doctest output naming `crates\domain\src\tool_receipts\receipted.rs`; `find crates/domain/src -iname "*tool_receipts*"` |
| 4 | 🟡 Medium | Workspace drift (nodus) | `nodus`'s own registry carries real, currently-open drift `main`'s does not: one RFC spec (`l1-nodus-graph.md`, held at RFC deliberately pending a second review or an `l2-nodus-graph` authoring pass); one orphaned spec (`l1-nodus-authoring.md`, authored this session, never absorbed into `PLAN.md`); a stale `PLAN.md` (based on registry v1.0.102, registry now at v1.0.103); and 4 open Backlog items, each explicitly blocked on a design decision rather than implementation capacity, per the Backlog's own re-grounding history. None of this is neglect — the Backlog's v1.38.0–v1.51.0 notes show it being actively re-verified against source and retracted/corrected multiple times — but it is open. | `check-prerequisites --json --workspace=nodus`: `SPEC_STATUS`, `ORPHANED_SPEC`, `SYNC_GAP`, `DESIGN_DEBT_PENDING` |
| 5 | ✨ Positive | Execution health (both workspaces) | Across all 587 completed tasks in 31 archived `main` phases + `nodus`'s own history, exactly one historical `Blocked [!]` incident exists in the entire corpus (Phase 1, a missing JS/Tauri toolchain, superseded and resolved by Phase 8, honestly recorded in a closure note) and zero task-level Cancellations. Every other string match on "Cancelled" across the corpus is domain vocabulary (`Error(Cancelled)`, `ResearchStatus::Cancelled`, `CancelledByUser`), not an abandoned task. | `grep -rn "Blocked \[!\]"` / `grep -rn "Cancelled"` across `.design/main/archives/tasks/*.md` |
| 6 | ✨ Positive | Header integrity (main) | A full `--verify-headers` sweep across all 294 `main` specs plus the role registry cross-check found zero `STATUS_DRIFT`/`VERSION_DRIFT` mismatches — `INDEX.md` accurately reflects every spec file's own header, project-wide. | `check-prerequisites --json --verify-headers --workspace=main`: `warnings: []` |
| 7 | 🟢 Low (informational) | Spec evolution | Exactly 1 of 294 Stable `main` specs (`l1-office-visualization.md`) has ever undergone a breaking major-version redesign (→ v2.0.0); the other 293 have stayed within major version 1. 177/294 (60%) reached their current state with no minor revision at all after authoring; 117/294 (40%) picked up at least one substantive minor extension. Both figures read as healthy for a fast-moving, continuously-revised registry — reported as the actual computed ratio rather than an impression. | `grep -oE` version-column extraction over `.design/main/INDEX.md` |
| 8 | 🟡 Medium (process, carried from this session's Retro L1) | Engine/process | `RETROSPECTIVE.md`'s own Snapshots table had no row between Phase 15 (2026-07-11) and Phase 30 (2026-09-11) — 14 phases (16–29) closed with no L1 snapshot appended, though each apparently completed normally via `/magic.run`. Already recorded via `record-diagnostic` (`RETRO_SNAPSHOTS_STALE`, severity warning) during this session's L1 pass; restated here for L2 continuity rather than re-logged. | `.design/main/RETROSPECTIVE.md` Snapshots table (Phase 15 → Phase 30 gap) |

### 💡 Recommendations

| # | Refs Observation | Recommendation | Target File |
| --- | --- | --- | --- |
| R1 | #1 | Fold into the next `/magic.spec main` pass touching any of the three: correct each spec's crate-placement claim to its real location and drop the dead `l2-source-layout.md` citation (replace with whatever *does* place it, if anything does). | `l2-file-store.md`, `l2-knowledge-store.md`, `l2-notes.md` |
| R2 | #2 | Same pass or a smaller dedicated one: update the Canonical Reference to `crates/domain/src/context_router.rs`, and explicitly confirm (not infer) that the INV-8 edge is fully closed before the migration narrative is treated as settled. | `l2-crate-topology.md` |
| R3 | #3 | Low priority — fold in opportunistically with R1/R2 rather than a dedicated pass. | `l2-tool-receipts.md` |
| R4 | #4 | Re-affirm rather than second-guess nodus's own stated policy: `/magic.task nodus` to absorb the orphan and re-sync `PLAN.md`; `/magic.spec nodus` for the 4 Backlog items only once a real feature need motivates one of them, per the Backlog's own standing note. | `.design/nodus/PLAN.md`, `.design/nodus/INDEX.md` |
| R5 | #8 | Audit whether `magic.run`'s Phase Completion step unconditionally invokes Retro L1 on every phase close, or silently no-ops under some condition — the 14-phase gap suggests the latter happened at least once. | `.magic/run.md`, `.magic/retrospective.md` |

### 📈 Trends (from Snapshots)

| Metric | Previous (Phase 15, 2026-07-11) | Current (Phase 30, 2026-09-11) | Δ |
| --- | --- | --- | --- |
| Specs in registry (main, cumulative D+R+S) | 196 (1/7/188) | 294 (0/0/294) | +98 |
| Blocked task rate | 0% | 0% | flat |
| Signal (L1, mechanical) | 🟢 | 🟢 | flat |
| Signal (this L2 pass, deep-audit) | — (no prior L2) | 🟡 | first measurement |

**Why the L2 signal reads 🟡 while every L1 snapshot reads 🟢:** L1's Green/Yellow/Red is computed from `check-prerequisites`' mechanical gates (orphans, phantoms, header parity) alone — all clean in `main`. This pass additionally read spec prose against actual source for a sample of Canonical References and found four stale-reference items (#1–#3) that no mechanical gate catches, plus `nodus`'s own open drift (#4). Per the Score & Signal table, 4 non-critical drift items pushes this pass to 🟡 rather than 🟢 — which is the intended division of labor: L1 catches what a script can see; L2 exists specifically to catch what it cannot.

# Project Context

**Generated:** 2026-07-11

## Active Technologies

- Node.js
- Rust

## Core Project Structure

```plaintext
.
├── .agents/
│   ├── rules/
│   ├── skills/
│   └── workflows/
├── .claude/
│   ├── commands/
│   ├── rules/
│   ├── scheduled_tasks.lock
│   └── skills/
├── .codex/
│   ├── prompts/
│   ├── rules/
│   └── skills/
├── .design/
│   ├── .cache/
│   ├── .graph-cache/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── graph-before.json
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── desktop.drawio.svg
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── release.drawio.svg
│   ├── technology-stack-research.md
│   └── ui-ux.md
├── .env
├── .env.example
├── .github/
│   ├── dependabot.yml
│   └── workflows/
├── .gitignore
├── .markdownlint.json
├── .release/
│   ├── program/
│   └── state/
├── AGENTS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── apps/
│   └── desktop/
├── biome.json
├── crates/
│   ├── auth-local/
│   ├── cli/
│   ├── codegraph/
│   ├── contract/
│   ├── core/
│   ├── domain/
│   ├── nodus/
│   ├── store-local/
│   └── tui/
├── docs/
│   └── README.md
├── firebase-debug.log
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── rust-toolchain.toml
└── scripts/
    └── check-domain-boundary.mjs
```

## Recent Changes

- T-13T01: final validation — full boundary sweep via `cargo tree` confirms the tier diagram exactly (domain carries zero infra deps; neither adapter depends on domain); the §6.4 INV-2 violation (a frontend opening a DB connection) is gone
- Verify: `cargo test --workspace` green, 1,252 passed / 0 failed (the original 314 core-lib tests redistributed as 2 + 265 + 29 + 18 = 314, exactly conserved); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean

## Phase 14 — Memory Intelligence & Consolidation (L2) (2026-07-11)

- T-14A01: `memory_signal` fact-vs-derived table — a closed three-kind signal vocabulary (Centrality/Cluster/Recency), version-guarded neutral-default degradation on absence or mismatch, disposable and rebuildable independently of the authored fact layer
- T-14A02: `depth`/`lifecycle_state` columns — never-rewrite-raw guard (MC-1); reversible Active/Paused/Archived lifecycle with an append-only transition audit and a prune-protective guard so decay can rank down but never delete a paused/archived item (MI-9)
- T-14B01: multiplicative offline-precomputed ranking (MC-8) — FTS5 BM25 mapped to a bounded (0,1] base relevance, fused multiplicatively with precomputed derived signals; no hot-path model call or graph walk
- T-14B02: corpus-maintenance pass — recency decay, prune-protected archive, split-flagging, merge-candidate detection and a transactional merge (MC-6 minus MC-7, moved to T-14B03 once its edge-graph dependency was caught at plan time)
- T-14B03: consolidation write — an additive-only `memory_edge` graph, the closed create/corroborate/refine/correct action algebra, an incremental watermark pass, a real (not tautological) optimistic-concurrency check caught and fixed during review, transactional correct, emergent topic summaries via a locally reimplemented union-find (MC-7), bounded interest topics (MC-10)
- T-14C01: `answer` projection (MI-1) with KB-6 citations and an honest CV-3/4-gated insufficient outcome; temporal recall modes (MI-2) and a closed structured-predicate compiler (MI-8) over the bi-temporal record; immediate recall-visibility proven (MI-3)
- T-14C02: conflict routing (MI-4) with a pinned confidence/trust-gap ambiguity threshold, a read-only intelligence digest (MI-5), and grounded run distillation (MI-7); MI-6/10/11/12 deliberately deferred as their own follow-up rather than rushed through a fourth schema round in one task
- T-14C03: gated experience reuse (MI-13) — a new `ExperienceOutcome` typing (Success/Failure/Insight) pulled forward as the one field the deferral could not do without, a deterministic similarity/score/freshness reuse gate, and a structural retained-authority-gate composing the existing SEC-9/SEC-10 realization — a reused plan needing approval is surfaced, never auto-applied
- T-14T01: cross-layer validation — 5 new integration tests through the real facade and SQLite adapter (capture→consolidate→answer, cold start, a no-graph-rewalk regression proof, the fact/derived boundary, and a real-adapter experience round-trip); MI-6/10/11/12 explicitly out of this sweep's scope — nothing built yet to exercise
- Verify: `cargo test --workspace` green, 1,333 passed / 0 failed (1,252 Phase-13 baseline + 81 new across the phase); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean


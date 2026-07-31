# Project Context

**Generated:** 2026-07-31

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
├── .cargo/
│   └── config.toml
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
│   ├── UX-UI - разбор 3 кейсов.md
│   ├── UX-дизайн - 6 психологических принципов.md
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
│   ├── activation-os/
│   ├── auth-local/
│   ├── cli/
│   ├── codegraph/
│   ├── contract/
│   ├── core/
│   ├── domain/
│   ├── model-local/
│   ├── nodus/
│   ├── store-local/
│   └── tui/
├── docs/
│   └── README.md
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

- T-14B03: consolidation write — an additive-only `memory_edge` graph, the closed create/corroborate/refine/correct action algebra, an incremental watermark pass, a real (not tautological) optimistic-concurrency check caught and fixed during review, transactional correct, emergent topic summaries via a locally reimplemented union-find (MC-7), bounded interest topics (MC-10)
- T-14C01: `answer` projection (MI-1) with KB-6 citations and an honest CV-3/4-gated insufficient outcome; temporal recall modes (MI-2) and a closed structured-predicate compiler (MI-8) over the bi-temporal record; immediate recall-visibility proven (MI-3)
- T-14C02: conflict routing (MI-4) with a pinned confidence/trust-gap ambiguity threshold, a read-only intelligence digest (MI-5), and grounded run distillation (MI-7); MI-6/10/11/12 deliberately deferred as their own follow-up rather than rushed through a fourth schema round in one task
- T-14C03: gated experience reuse (MI-13) — a new `ExperienceOutcome` typing (Success/Failure/Insight) pulled forward as the one field the deferral could not do without, a deterministic similarity/score/freshness reuse gate, and a structural retained-authority-gate composing the existing SEC-9/SEC-10 realization — a reused plan needing approval is surfaced, never auto-applied
- T-14T01: cross-layer validation — 5 new integration tests through the real facade and SQLite adapter (capture→consolidate→answer, cold start, a no-graph-rewalk regression proof, the fact/derived boundary, and a real-adapter experience round-trip); MI-6/10/11/12 explicitly out of this sweep's scope — nothing built yet to exercise
- Verify: `cargo test --workspace` green, 1,333 passed / 0 failed (1,252 Phase-13 baseline + 81 new across the phase); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean

## Phase 15 — Memory Capture Policy & Metadata (L2) (2026-07-11)

- T-15A01: capture-metadata schema — three new nullable `MemoryEntry` fields (`actor`/`expiry`/`subject`, the last a new `MemorySubject` enum), all absent-by-default so the entire pre-existing corpus reads back unchanged; cross-reference deliberately realized with no new field at all, reusing the existing MC-3 `add_edge`
- T-15B01: the salience-gated capture policy (MI-6) — a confidence-honest gate in front of the existing MC-4 create/corroborate decision (reused wholesale, not reimplemented), MI-6 cross-reference edges via a new `CROSS_REF_PREDICATE`, and the previously-inert `expiry` field wired into every recall path's `WHERE` clause so a voided item is actually excluded, not just stored
- T-15B02: capture-time temporal normalization (MI-10) and the raw/inferred write mode (MI-12) — one generator seam serving both, `raw` mode structurally never consults the generator at all rather than asking and discarding the answer; every no-generator path degrades to verbatim, never fabricating
- T-15B03: caller capture directives (MI-11) — `include`/`exclude`/`custom-instruction` steering with the safety-suppression guard enforced structurally (an excluded safety-relevant sentence is retained regardless) and the honesty-floor invariant holding by construction (the function has no confidence parameter at all)
- T-15T01: cross-layer validation — 4 new integration tests through the real facade and SQLite adapter proving MI-6's metadata and cross-ref edges, the confidence gate's real unrecallability, MI-10/12's degrade reaching an actually-recallable row, and MI-11's safety guard reaching real storage
- Verify: `cargo test --workspace` green, 1,360 passed / 0 failed (1,333 Phase-14 baseline + 27 new across the phase); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean


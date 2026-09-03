# Project Context

**Generated:** 2026-09-03

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
├── .codex/
├── .design/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── graph-snapshot.json
│   ├── main/
│   ├── nodus/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── UX-UI - разбор 3 кейсов.md
│   ├── UX-дизайн - 6 психологических принципов.md
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── reverse-derivation-mechanism.md
│   ├── technology-stack-research.md
│   └── ui-ux.md
├── .env.example
├── .github/
│   ├── dependabot.yml
│   └── workflows/
├── .gitignore
├── .magic/
├── .markdownlint.json
├── .release/
│   ├── program/
│   ├── project/
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

- T-15B03: caller capture directives (MI-11) — `include`/`exclude`/`custom-instruction` steering with the safety-suppression guard enforced structurally (an excluded safety-relevant sentence is retained regardless) and the honesty-floor invariant holding by construction (the function has no confidence parameter at all)
- T-15T01: cross-layer validation — 4 new integration tests through the real facade and SQLite adapter proving MI-6's metadata and cross-ref edges, the confidence gate's real unrecallability, MI-10/12's degrade reaching an actually-recallable row, and MI-11's safety guard reaching real storage
- Verify: `cargo test --workspace` green, 1,360 passed / 0 failed (1,333 Phase-14 baseline + 27 new across the phase); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean

## Phase 23 — Tool Receipts (L2) (2026-08-12)

- T-23A01: `ReceiptKey` + canonical length-prefixed `ActionBinding` encoder — every field length-prefixed so the MAC input is injective (a naive concatenation lets `kind="ab",inputs="c"` collide with `kind="a",inputs="bc"`); key is a zeroed-on-drop opaque newtype with a hand-written redacting `Debug`
- T-23A02: `mint`/`verify` — `blake3::keyed_hash` over the canonical binding, a fold-based constant-time tag comparison, and the `cronus-rcpt-<ts>-<tag16>` token format; a fixed-key/pinned-clock replay test proves the per-session `action_id` is what makes two otherwise-identical calls unforgeable against each other
- T-23B01: `Receipted<T>` — no public constructor, no `From<T>`; the sole path to one is `mint_receipted(key, binding, value)`, so a value cannot exist without its receipt. Proven as a `compile_fail` doctest, not a runtime assertion
- T-23B02: `ReceiptLedger` — `status()` defaults every undispatched `action_id` to `Unreceipted`; `CoverageReport{receipted,pending}` reports both counts separately so a caller cannot round outstanding work down to "all verified"
- T-23C01: `ReceiptSession` (`crates/core/src/receipts_bootstrap.rs`) — fresh OS-CSPRNG key per process (the sole non-deterministic act in the subsystem; `getrandom` added to `cronus-core` only, domain allowlist untouched)
- T-23C02: `ReceiptedDispatch::invoke`/`verify` — gate-first-unchanged, execute-only-when-permitted, bind the observed outcome, mint, ledger, audit; `dev_office_workspace::run_elevated_action` rewired onto it, the phase's one disclosed production call site. Real finding fixed mid-task: the shipped `append_audit_entry` silently drops `finding_id`, so the token rides `category` instead — caught by a failing test
- T-23D01: deferred-action lifecycle — `defer`/`resolve_deferred`; a detached action registers `Pending{action_id}` with no tag at dispatch, and mints only on correlated completion with the real observed result
- T-23T01: `crates/core/tests/tool_receipts_invariants.rs` — 11 tests, one per TR-1…TR-9 through the real facade export chain, plus 2 leak-path tests (redacted `Debug`, receipt token survives `redact::redact`)
- Verify: `cargo test --workspace` green across 3 consecutive full runs (75 `test result: ok` blocks each, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean


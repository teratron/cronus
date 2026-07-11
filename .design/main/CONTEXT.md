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

- T-12D01: `cronus ext skill import|create|status` — nested under the existing `Ext` group (no new top-level command group); `import` reads one file and runs it through `convert()`; `create` runs an instruction-only `AuthoredSkill` through `synthesize()`; `status` reports store origin/degradation/pending-review via a separately-testable `compute_status()`; extended `SkillEntry` with `degraded`/`pending_review` fields through a backward-compatible `with_status()` builder
- T-12T01: invariant-compliance sweep — 10 integration tests in `crates/core/tests/skill_system.rs`, one per Invariant Compliance row (EXT-1/2/3/4+6/5+STO-1/7/8/9/11, STO-3), exercising convert→store, convert→exec, and synthesize→store together; CLI/TUI/library parity confirmed structurally (TUI's `/ext` slash command already covers the nested `skill` group at its existing granularity — no TUI-crate change needed)
- Verify: `cargo test --workspace` green across all crates (core 314 lib + 10 integration, cli 37 unit + 28 smoke, tui 198, nodus 34, codegraph, all others); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all` clean

## Phase 13 — Core Decomposition (Crate Topology) — 2026-07-11

- T-13A01: minted `cronus-contract` (zero-dep types + seam traits) — moved `MemoryEntry`/`StateStore`/`ModelProvider`/`CheckpointWriter`/`Compactor`/`BusSender` and their supporting types out of the single core crate; declared the three new DN-2 seam traits (`UserDataStore`, `AuthProvider`, `IdentityProvider`) plus `MemorySearch`; `ArchiveSink` deliberately stayed with its domain-owned `MigrationItem` rather than contaminating the ports crate
- T-13A02: inverted the one domain→infra edge — `ContextRouter` now holds `&dyn MemorySearch` instead of the concrete `MemoryStore`; `MemoryStore` implements the new trait in place. The migration's sole type-signature change and its critical-path pivot
- T-13B01: extracted `cronus-store-local` (SQLite/encryption/keychain adapter) — `memory::{store,chain,trust,encryption}`, the SQLite half of `inbox`, and `workspace` moved wholesale (chain/trust travel with their sole caller, `store.rs`, since the tier model forbids an adapter→domain edge); implements `UserDataStore` + `MemorySearch`; fixed a real `Send + Sync` conflict from `rusqlite::Connection`'s interior `RefCell`
- T-13B02: extracted `cronus-auth-local` (password/TOTP/identity adapter) — `auth.rs` moved whole (its "domain half" — privilege maps, reserved names — had zero consumers outside the file); implements `AuthProvider` + a new minimal `SingleUserIdentity` (`IdentityProvider`)
- T-13C01: renamed the remainder to `cronus-domain`; `crates/core` becomes the `cronus` facade — 71 files moved via `git mv`; `cronus-domain` links only `blake3`/`chrono`/`cron`/`cronus-contract` (deliberately omits `nodus`: zero real usage found, a seam not a dependency); facade re-exports every module under its historical path, so no downstream call site changed
- T-13C02: repointed the TUI at `cronus-domain`; fixed `codegraph`'s public surface (§6.4) — moved `Capabilities`/`Engine` into `cronus-domain` (both are pure, no I/O) so the TUI could drop the facade entirely per the topology spec's frontend table; `codegraph` now exposes a `CodeIndex` wrapper instead of a public `rusqlite::Connection`; the CLI dropped its direct `rusqlite` dependency
- T-13D01: non-optional CI boundary guard — `scripts/check-domain-boundary.mjs` resolves `cargo metadata` and fails naming any `cronus-domain` normal dependency outside the five-crate allowlist; wired into `.github/workflows/deps-gate.yml`; proved both the pass and fail paths locally (temporarily added `rusqlite`, confirmed the named failure, reverted cleanly)
- T-13T01: final validation — full boundary sweep via `cargo tree` confirms the tier diagram exactly (domain carries zero infra deps; neither adapter depends on domain); the §6.4 INV-2 violation (a frontend opening a DB connection) is gone
- Verify: `cargo test --workspace` green, 1,252 passed / 0 failed (the original 314 core-lib tests redistributed as 2 + 265 + 29 + 18 = 314, exactly conserved); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean


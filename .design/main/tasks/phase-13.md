---
phase: 13
name: "Core Decomposition (Crate Topology)"
status: Todo
subsystem: "crates/ workspace repartition: contract · domain · store-local · auth-local · facade; frontends (tui, cli) + codegraph public surface"
requires: [1, 4, 9, 12]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 13 Tasks — Core Decomposition (Crate Topology)

**Phase:** 13
**Status:** Todo
**Strategic Goal:** Repartition the single `crates/core` into five crates on the dependency/seam axis — `cronus-contract` (zero-dep types + seam traits), `cronus-domain` (the no-I/O 82%), `cronus-store-local` + `cronus-auth-local` (the DN-2 adapters), and `cronus` (the facade + composition root). Behavior is preserved exactly (public module paths kept via `pub use` re-exports); the win is compiler-enforced INV-8 boundaries and realized DN-2/DN-3 provider seams.

> **This phase moves working code; it does not add domain logic.** Every step must leave the workspace compiling and every test green before the next begins — that is why the tasks mirror the spec's ordered §5 migration one-to-one. Execution is effectively sequential (see Phase Notes).

## Atomic Checklist

- [ ] [T-13A01] Mint `cronus-contract` (shared types + seam traits, zero external deps)
- [ ] [T-13A02] Invert the `context_router → MemoryStore` edge (the migration pivot)
- [ ] [T-13B01] Extract `cronus-store-local` (SQLite / encryption / keychain adapter)
- [ ] [T-13B02] Extract `cronus-auth-local` (password / TOTP / identity adapter)
- [ ] [T-13C01] Rename remainder to `cronus-domain`; keep `crates/core` as the `cronus` facade
- [ ] [T-13C02] Repoint TUI at `cronus-domain`; fix `codegraph` public surface so CLI drops `rusqlite`
- [ ] [T-13D01] CI boundary guard on the domain dependency allowlist (non-optional)
- [ ] [T-13T01] Validation: behavior equivalence + boundary sweep

## Detailed Tracking

### [T-13A01] Mint `cronus-contract`

- **Spec:** l2-crate-topology.md §4.2, §5 step 1
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo build --workspace` green; `cargo tree -p cronus-contract --edges normal` lists **no** dependency outside the workspace (zero external deps); a downstream call site using a moved type (e.g. `cronus::memory::MemoryEntry`) still compiles unchanged (facade re-export intact).
- **Handoff:** Every later task depends on this crate existing; the seam traits (`UserDataStore`, `AuthProvider`, `IdentityProvider`) it declares are implemented in T-13B01/T-13B02.
- **Notes:** Move shared types (`MemoryEntry`, typed prefixed IDs, `ThinkingLevel`, error taxonomy) and the existing traits (`StateStore`, `ModelProvider`, `CheckpointWriter`, `Compactor`, `BusSender`, `ArchiveSink`) plus the three new DN-2 seam traits. No behavior change; `crates/core` re-exports everything so no call site moves. The workspace still has one real logic crate after this step.

### [T-13A02] Invert the `context_router → MemoryStore` edge

- **Spec:** l2-crate-topology.md §4.6, §5 step 2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus context_router` (and its existing tests) green; a grep of `context_router.rs` shows the field type is now `&dyn MemorySearch` (or a generic bound), **not** the concrete `MemoryStore`; `MemorySearch` lives in `cronus-contract` and `MemoryStore` implements it.
- **Handoff:** This is the pivot — once the one inverted edge is reversed, the domain/infrastructure cut (T-13B01/B02) is a pure file move. Nothing downstream can start until this lands.
- **Notes:** The **only** task that changes a type signature. Introduce `MemorySearch` in the contract, implement it for `MemoryStore` in place, retarget `ContextRouter`. Cascade risk is concentrated here: steps 3–7 are all blocked on it.

### [T-13B01] Extract `cronus-store-local`

- **Spec:** l2-crate-topology.md §4.2, §4.6 (split table), §5 step 3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-store-local` green; the crate implements `UserDataStore` (and `MemorySearch`); `rusqlite`, `aes-gcm`, `argon2`, `keyring` appear in **its** manifest; `cargo test --workspace` still green.
- **Handoff:** `cronus-auth-local` (T-13B02) follows; both are wired by the facade in T-13C01.
- **Notes:** Move `memory::store`, `memory::encryption`, and the persistence halves of `inbox` / `workspace`. The domain halves (`chain`, `trust`, `consolidation`, lifecycle rules) stay behind for T-13C01. Serialize against T-13B02: both edit `crates/core/Cargo.toml` and the facade wiring.

### [T-13B02] Extract `cronus-auth-local`

- **Spec:** l2-crate-topology.md §4.2, §4.6 (split table), §5 step 4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-auth-local` green; the crate implements `AuthProvider` and the trivial single-principal `IdentityProvider`; `bcrypt`, `hmac`, `sha1`, `getrandom` appear in **its** manifest; `cargo test --workspace` still green.
- **Handoff:** Facade wiring in T-13C01 selects this as the default `AuthProvider`/`IdentityProvider`.
- **Notes:** Move `auth.rs`'s password hashing, TOTP, and token issuance. The domain halves (privilege maps, reserved-name policy) stay behind. Serialize against T-13B01 (shared manifest/facade).

### [T-13C01] Rename remainder to `cronus-domain`; keep `crates/core` as the `cronus` facade

- **Spec:** l2-crate-topology.md §4.1, §4.2, §5 step 5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test --workspace` green; `cargo tree -p cronus-domain --edges normal` lists only the §4.3 allowlist (`blake3`, `chrono`, `cron`) plus `cronus-contract` and `nodus` — nothing else; every pre-existing public path (`cronus::memory::…`, `cronus::auth::…`, `cronus::skills::…`, …) still resolves from a downstream compile (facade `pub use` re-exports preserved).
- **Handoff:** Frontends repoint in T-13C02; the guard in T-13D01 locks the allowlist proven here.
- **Notes:** The 47 pure-`std` domain modules (plus Phase 12's pure-`std` `skills` module and the domain halves left by B01/B02) become `cronus-domain`. `crates/core` keeps its path but becomes the `cronus` facade: `Engine`, `Capabilities`, C-ABI/FFI, default-provider wiring, and `pub use` only — no logic of its own.

### [T-13C02] Repoint TUI at `cronus-domain`; fix `codegraph` public surface

- **Spec:** l2-crate-topology.md §4.8, §6.4, §5 step 6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo tree -p cronus-tui` no longer includes `rusqlite`, `keyring`, `bcrypt`, or `argon2`; `crates/cli/Cargo.toml` no longer lists `rusqlite`; a grep of `codegraph`'s public API shows no exported `rusqlite::Connection`; `cargo test --workspace` green.
- **Handoff:** Resolves the §6.4 INV-2 violation (a frontend performing persistence); verified end-to-end in T-13T01.
- **Notes:** The TUI uses one module (`redact`) today, so it links `cronus-domain` directly. `codegraph` must hide its storage engine behind its own API; the CLI then drops its direct `rusqlite` dependency and reaches persistence only through the core contract.

### [T-13D01] CI boundary guard on the domain dependency allowlist

- **Spec:** l2-crate-topology.md §5 step 7 (explicitly non-optional), §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** With a forbidden dependency (e.g. `rusqlite`) **temporarily** added to `cronus-domain`'s manifest, the guard command exits non-zero and names the offending crate; with the clean tree it exits zero. (Prove both directions, then revert the temporary edit.)
- **Handoff:** Closes the phase's structural contract — without it the split silently decays.
- **Notes:** A `cargo`-metadata check (or `fallow`-style boundary rule) asserting `cronus-domain`'s normal dependencies are a subset of {`cronus-contract`, `nodus`, `blake3`, `chrono`, `cron`}. Wire it where the project's other structural gates run. The spec is explicit: the current layout drifted because nothing failed a build when it drifted — a guard that is never exercised is worthless, so the Verify tests the failure path, not just the happy path.

### [T-13T01] Validation: behavior equivalence + boundary sweep

- **Goal:** Prove the refactor changed structure, not behavior, and that every §3 Invariant Compliance claim holds on the new tree (INV-1 embeddable facade, INV-2 no frontend persistence, INV-3 contract-on-facade parity, INV-7 secret confinement, INV-8 acyclic crate graph).
- **Method:** Full `cargo test --workspace` green (every pre-existing test passes unchanged — behavior equivalence); a downstream smoke that the public contract resolves from the facade exactly as before (INV-1/INV-3); confirm the §6.4 INV-2 violation is gone (no frontend opens a DB `Connection`); confirm the crate graph is acyclic and the domain tier links no infra (cross-check with T-13D01's guard). Structural gates: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`.
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test --workspace` green + `cargo clippy --workspace --all-targets -- -D warnings` clean + `cargo fmt --all -- --check` clean, with the pre-migration test count preserved (no test deleted to make the move pass).

## Phase Notes (Planning Audit)

- **Sequential by construction.** The §5 migration is a strict chain: A01 (contract) → A02 (pivot edge) → {B01, B02} → C01 (facade) → C02 (frontends) → D01 (guard) → T01. Parallel mode (C3) is nominal here; the orchestrator serializes B01/B02 (both edit `crates/core/Cargo.toml` + the facade) and everything else is a hard dependency chain.
- **Cascade concentrated on T-13A02.** The single inverted edge is the pivot; if it is done wrong every later task is blocked or built on a bad cut. Treat it as the phase's critical path even though it is the smallest diff.
- **Behavior preservation is the acceptance bar, not feature completeness.** No new capability ships. Success = identical public contract + all existing tests green + the domain tier provably infra-free. A test that had to change to make the move compile is a red flag, not progress.
- **Measurement drift (harmless).** The spec's module counts (47 pure-`std` / 53 total) predate Phase 12's `skills` module, which is pure-`std` domain — it joins `cronus-domain`, leaving the partition *shape* unchanged. The minting rule is measurement-independent: a module earns a crate by holding an infra dependency, not by count.
- **Async-runtime gap is out of scope (§6.5).** The tree has no `tokio` today; whenever a runtime lands it belongs to the adapter/facade tiers, never the domain. This phase only preserves that constraint structurally; it does not introduce a runtime.
- **Follow-up, not a gate:** refreshing `l2-source-layout` §4.1/§4.2's pre-migration tree diagram to the post-migration crate set is a `/magic.spec amend` after this phase lands.

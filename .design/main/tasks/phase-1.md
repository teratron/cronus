---
phase: 1
name: "Seed I — Foundation"
status: In Progress
subsystem: "monorepo (crates/, apps/, packages/) + crates/core"
requires: []
provides:
  - "Cargo workspace (crates/core, nodus, cli, tui) — builds clean"
  - "core::Capabilities contract + Engine (zero frontend deps)"
  - "core::paths OS-native resolver (program/state/cache/logs + portable)"
key_files:
  created:
    - Cargo.toml
    - rust-toolchain.toml
    - crates/core/src/lib.rs
    - crates/core/src/paths.rs
    - crates/cli/src/main.rs
    - crates/tui/src/main.rs
    - crates/nodus/src/lib.rs
  modified: []
patterns_established:
  - "Cargo workspace; edition 2024; std-only foundation (no external deps yet)"
  - "Dependency direction inward: cli/tui -> core; core depends on nothing"
duration_minutes: ~
---

# Stage 1 Tasks — Seed I: Foundation

**Phase:** 1
**Status:** In Progress
**Strategic Goal:** The soil and seed coat — a buildable polyglot monorepo with the engine-skeleton (`crates/core`) exposing its public contract, an OS-native path model, and the security baseline. The Rust foundation is built and verified; JS/Tauri scaffolding is blocked on missing toolchain in this environment.

> **Toolchain note:** this environment has `rustc`/`cargo` 1.96 and `node` 24, but **no `pnpm`, Tauri CLI, or polyglot runner** — so the JS/Tauri tasks (T-1A02, T-1A03) are `Blocked [!]` until those are installed. The Rust tracks are fully built and verified.

## Atomic Checklist

Track A — Scaffold (l2-source-layout, l2-technology-stack)
- [x] [T-1A01] Cargo workspace with members `core`, `nodus` (skeleton), `cli`, `tui`
- [!] [T-1A02] pnpm workspace (`packages/ui`) and `apps/desktop` Tauri v2 scaffold
- [!] [T-1A03] Polyglot runner (moon/Nx) sequencing JS + Tauri builds

Track B — Filesystem (l2-filesystem-layout)
- [x] [T-1B01] OS-native path resolver (program/state/cache/logs + portable override)
- [ ] [T-1B02] Idempotent state-tier bootstrap from templates (`init`)

Track C — Core (l2-core-library)
- [x] [T-1C01] `crates/core` skeleton + public capability contract
- [ ] [T-1C02] Durable state persistence interface + restartable load

Track E — Security (l2-security)
- [ ] [T-1E01] Secure-default `.gitignore` + secret store read path (`.env`/keychain)
- [ ] [T-1E02] Output/log redaction + default-deny outbound egress gate

Track T — Validation
- [x] [T-1T01] Validate scaffold & dependency direction (Track A — Rust)
- [ ] [T-1T02] Validate path resolution & state bootstrap (Track B)
- [ ] [T-1T03] Validate core contract & restartable persistence (Track C)
- [ ] [T-1T04] Validate secret isolation & sandbox defaults (Track E)

## Detailed Tracking

### [T-1A01] Cargo workspace

- **Spec:** l2-source-layout.md §4.1
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo metadata --no-deps` lists `cronus-core`, `nodus`, `cronus-cli`, `cronus-tui`; `cargo build` finished (4 crates, exit 0).
- **Changes:** Root Cargo workspace (edition 2024, pinned 1.96 via `rust-toolchain.toml`); 4 member crates with minimal skeletons. `nodus` is an empty lib (port in Phase 2).
- **Handoff:** unblocks T-1C01 and Phase 2.

### [T-1A02] JS workspace + Tauri scaffold

- **Spec:** l2-source-layout.md §4.1, l2-technology-stack.md
- **Status:** Blocked [!]
- **Assignment:** Agent
- **Verify:** (deferred) `pnpm install`, `cargo tauri build`.
- **Notes:** `[!]` Blocked — `pnpm` and the Tauri CLI are not installed in this environment. Unblock by installing pnpm + `cargo install tauri-cli` (and the mobile SDKs for the iOS/Android smoke).

### [T-1A03] Polyglot runner

- **Spec:** l2-source-layout.md §4.3
- **Status:** Blocked [!]
- **Assignment:** Agent
- **Verify:** (deferred) runner task graph sequences `ui` + `tauri` builds.
- **Notes:** `[!]` Blocked — depends on T-1A02 toolchain (pnpm) + a runner (moon/Nx) not installed.

### [T-1B01] Path resolver

- **Spec:** l2-filesystem-layout.md §4.1
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-core` — 2 `paths` tests pass (portable groups all roots; OS-native roots non-empty & distinct).
- **Changes:** `core::paths` — `Paths::os_native()` / `Paths::portable()` resolving `Root::{Program,State,Cache,Logs}` per Windows/macOS/Linux(XDG); std-only (no `dirs` dep).
- **Handoff:** required by T-1B02 and T-1C02.

### [T-1B02] State-tier bootstrap

- **Spec:** l2-filesystem-layout.md §4.3, l1-storage-model.md (STO-1/3)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `init` on an empty state dir produces the §4.3 tree; second run idempotent.
- **Notes:** next increment; std `fs`, no network needed.

### [T-1C01] Core skeleton + contract

- **Spec:** l2-core-library.md, l1-architecture.md (INV-1/2/3)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo build -p cronus-core` (exit 0); `cargo tree -p cronus-core` → no dependencies (INV-1/2); `cargo test` core engine test passes; `cronus-cli` invokes `Engine::status()` over the contract.
- **Changes:** `core::Capabilities` trait (`version`, `status`) + `Engine`; cli/tui are thin callers. Workflow wiring to `crates/nodus` deferred to Phase 2.
- **Handoff:** unblocks Phases 3–8 (critical path).

### [T-1C02] Durable state + restartable load

- **Spec:** l2-core-library.md, l1-storage-model.md (STO-2), l1-architecture.md (INV-5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** open SQLite at the resolved state path, persist, drop/reopen, reload without loss.
- **Notes:** next increment; needs a SQLite crate (`rusqlite`) — first task to require a crates.io dependency.

### [T-1E01] Secret store + gitignore

- **Spec:** l2-security.md (SEC-1/2)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `git check-ignore .env` ignored (`.env.example` not); secret read never logged.
- **Notes:** next increment; the repo `.gitignore` already carries secure defaults — confirm + add the secret-store read module.

### [T-1E02] Redaction + egress gate

- **Spec:** l2-security.md (SEC-3/5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** secret redacted in rendered output (test); unauthorized outbound send denied + audited.
- **Notes:** next increment; std-only.

### [T-1T01] Validation — scaffold & dependency direction

- **Goal:** Verify Track A (Rust) against l2-source-layout §4.1/4.2.
- **Method:** `cargo build` builds all members; `cargo tree -p cronus-core` shows no `cli`/`tui` dependency (inward direction). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
- **Status:** Done

### [T-1T02] Validation — paths & bootstrap

- **Goal:** Verify Track B against l2-filesystem-layout §4.1/4.3.
- **Method:** per-OS path tests (T-1B01 ✓); bootstrap tree + no-write-outside-state assertion (pending T-1B02).
- **Status:** Todo

### [T-1T03] Validation — core contract & persistence

- **Goal:** Verify Track C against l1-architecture INV-1/2 and STO-2.
- **Method:** `cargo tree -p cronus-core` no frontend deps (✓); restart-persistence test (pending T-1C02).
- **Status:** Todo

### [T-1T04] Validation — security baseline

- **Goal:** Verify Track E against l1-security SEC-1/3/5/6.
- **Method:** backup excludes `.env`; redaction test; sandbox denies network by default (pending T-1E01/E02).
- **Status:** Todo

---
phase: 1
name: "Seed I — Foundation"
status: Done
subsystem: "monorepo (crates/, apps/, packages/) + crates/core"
requires: []
provides:
  - "Cargo workspace (crates/core, nodus, cli, tui) — builds clean"
  - "core::Capabilities contract + Engine (zero frontend deps)"
  - "core::paths OS-native resolver (program/state/cache/logs + portable)"
  - "core::state bootstrap (idempotent state-tier creation; .env.example only)"
  - "core::store durable StateStore seam (file-backed; SQLite backend in Phase 4)"
  - "core::secrets / core::redact / core::egress security baseline"
key_files:
  created:
    - Cargo.toml
    - rust-toolchain.toml
    - crates/core/src/lib.rs
    - crates/core/src/paths.rs
    - crates/core/src/state.rs
    - crates/core/src/store.rs
    - crates/core/src/secrets.rs
    - crates/core/src/redact.rs
    - crates/core/src/egress.rs
    - crates/cli/src/main.rs
    - crates/tui/src/main.rs
    - crates/nodus/src/lib.rs
  modified:
    - .gitignore
patterns_established:
  - "Cargo workspace; edition 2024; std-only foundation (no external deps yet)"
  - "Dependency direction inward: cli/tui -> core; core depends on nothing"
  - "Persistence as a StateStore seam in core; concrete SQLite backend deferred to the memory phase"
  - "Security baseline: secret store + redaction + default-deny egress gate (std-only)"
duration_minutes: ~
---

# Stage 1 Tasks — Seed I: Foundation

**Phase:** 1
**Status:** Done
**Strategic Goal:** The soil and seed coat — a buildable polyglot monorepo with the engine-skeleton (`crates/core`) exposing its public contract, an OS-native path model, durable state, and the security baseline. The full Rust foundation is built and verified; the JS/Tauri scaffolding it originally waited on has since been provisioned by Phase 8.

> **Closure note (2026-06-30):** Phase 1 is closed on its actual scope — the Rust seed (Tracks A-Rust, B, C, E, T all Done). The JS/Tauri scaffold tasks (T-1A02, T-1A03) were originally `Blocked [!]` on a missing toolchain; they have since been **superseded by Phase 8 `T-8A01`**, which provisioned pnpm + Tauri v2 and scaffolded `apps/desktop` + `packages/ui` (full toolchain green via PowerShell). The two security-hardening specs listed against this phase in earlier PLAN revisions (`l2-sandbox-policy`, `l2-multi-user-auth`) never gated a downstream phase and have been **relocated to Phase 9 (Operational Hardening)**; they are not part of Phase 1's closed scope.

## Atomic Checklist

Track A — Scaffold (l2-source-layout, l2-technology-stack)

- [x] [T-1A01] Cargo workspace with members `core`, `nodus` (skeleton), `cli`, `tui`
- [x] [T-1A02] pnpm workspace (`packages/ui`) and `apps/desktop` Tauri v2 scaffold — superseded by Phase 8 T-8A01
- [x] [T-1A03] Polyglot runner sequencing JS + Tauri builds — superseded by Phase 8 T-8A01

Track B — Filesystem (l2-filesystem-layout)

- [x] [T-1B01] OS-native path resolver (program/state/cache/logs + portable override)
- [x] [T-1B02] Idempotent state-tier bootstrap

Track C — Core (l2-core-library)

- [x] [T-1C01] `crates/core` skeleton + public capability contract
- [x] [T-1C02] Durable state persistence interface + restartable load

Track E — Security (l2-security)

- [x] [T-1E01] Secure-default `.gitignore` + secret store read path (`.env`/keychain)
- [x] [T-1E02] Output/log redaction + default-deny outbound egress gate

Track T — Validation

- [x] [T-1T01] Validate scaffold & dependency direction (Track A — Rust)
- [x] [T-1T02] Validate path resolution & state bootstrap (Track B)
- [x] [T-1T03] Validate core contract & restartable persistence (Track C)
- [x] [T-1T04] Validate secret isolation & redaction/egress defaults (Track E)

## Detailed Tracking

### [T-1A01] Cargo workspace

- **Spec:** l2-source-layout.md §4.1
- **Status:** Done
- **Verify:** `cargo metadata --no-deps` lists `cronus`, `nodus`, `cronus-cli`, `cronus-tui`; `cargo build` exit 0 (4 crates).
- **Changes:** Root Cargo workspace (edition 2024, pinned 1.97); 4 member crates. `nodus` empty lib (port in Phase 2).

### [T-1A02] JS workspace + Tauri scaffold

- **Spec:** l2-source-layout.md §4.1, l2-technology-stack.md
- **Status:** Done (superseded by Phase 8 T-8A01)
- **Notes:** originally blocked on a missing toolchain here. Phase 8 T-8A01 provisioned pnpm + Tauri v2 and scaffolded `packages/ui` + `apps/desktop` (full toolchain green via PowerShell) — the JS/Tauri workspace this task called for now exists.

### [T-1A03] Polyglot runner

- **Spec:** l2-source-layout.md §4.3
- **Status:** Done (superseded by Phase 8 T-8A01)
- **Notes:** JS + Tauri build sequencing is carried by the Phase 8 scaffold's pnpm/Vite/Tauri pipeline established in T-8A01.

### [T-1B01] Path resolver

- **Spec:** l2-filesystem-layout.md §4.1
- **Status:** Done
- **Verify:** `cargo test -p cronus` — `paths` tests pass (portable groups all roots; OS-native roots non-empty & distinct).
- **Changes:** `core::paths` resolving `Root::{Program,State,Cache,Logs}` per Windows/macOS/Linux(XDG) + portable; std-only.

### [T-1B02] State-tier bootstrap

- **Spec:** l2-filesystem-layout.md §4.3, l1-storage-model.md (STO-1/3/6)
- **Status:** Done
- **Verify:** `cargo test` — bootstrap creates the §4.3 tree; second run idempotent (existing files preserved); never seeds a real `.env`.
- **Changes:** `core::state::bootstrap_at` creates `memory/notes`, `skills`, `employees`, `workspaces`, `backups` + seeds `app.json`/`config.json`/`.env.example`/`AGENTS.md`; writes only the state tier.

### [T-1C01] Core skeleton + contract

- **Spec:** l2-core-library.md, l1-architecture.md (INV-1/2/3)
- **Status:** Done
- **Verify:** `cargo build -p cronus`; `cargo tree -p cronus` → no dependencies (INV-1/2); cli/tui call `Engine::status()`.
- **Changes:** `core::Capabilities` trait + `Engine`; thin cli/tui callers. Workflow wiring to `crates/nodus` deferred to Phase 2.

### [T-1C02] Durable state + restartable load

- **Spec:** l2-core-library.md, l1-storage-model.md (STO-2), l1-architecture.md (INV-5)
- **Status:** Done
- **Verify:** `cargo test` — `FileStore` put → drop → reopen → values preserved (restart); open-missing starts empty.
- **Changes:** `core::store::StateStore` trait + std-only `FileStore` (persistence seam). SQLite + sqlite-vec backend deferred to Phase 4 (memory store) — core owns the interface, the memory phase owns the engine.

### [T-1E01] Secret store + gitignore

- **Spec:** l2-security.md (SEC-1/2)
- **Status:** Done
- **Verify:** `git check-ignore .env` ignored (`.env.example` not); `cargo test` — env-var precedence + `.env` file read; secret value never logged by `get`.
- **Changes:** `core::secrets::get` (env → state `.env`); `.gitignore` extended for the new tech (secrets, `target/`, `node_modules/`, etc.).

### [T-1E02] Redaction + egress gate

- **Spec:** l2-security.md (SEC-3/5/7)
- **Status:** Done
- **Verify:** `cargo test` — `redact` masks known secrets and preserves non-secret text; `EgressGate` denies by default + audits, allows authorized.
- **Changes:** `core::redact::redact` (mask known secrets) and `core::egress::EgressGate` (default-deny + audit log).

### [T-1T01] Validation — scaffold & dependency direction

- **Method:** `cargo build` all members; `cargo tree -p cronus` no frontend deps; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
- **Status:** Done

### [T-1T02] Validation — paths & bootstrap

- **Method:** per-OS path tests + bootstrap-tree/idempotency/no-real-`.env` tests pass.
- **Status:** Done

### [T-1T03] Validation — core contract & persistence

- **Method:** `cargo tree -p cronus` no frontend deps (INV-1/2); `FileStore` restart-persistence test passes (STO-2/INV-5).
- **Status:** Done

### [T-1T04] Validation — security baseline

- **Method:** `.env` gitignored / `.env.example` tracked; redaction + default-deny egress tests pass; secret read never logs the value.
- **Status:** Done

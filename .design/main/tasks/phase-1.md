---
phase: 1
name: "Foundation & Scaffolding"
status: Todo
subsystem: "monorepo (crates/, apps/, packages/) + crates/core"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 1 Tasks — Foundation & Scaffolding

**Phase:** 1
**Status:** Todo
**Strategic Goal:** A buildable polyglot monorepo with an engine-skeleton (`crates/core`) that exposes its public contract, links the workflow runtime, bootstraps the OS-native state tier, and enforces the security baseline. Unblocks all later phases (core is the critical path).

## Atomic Checklist

Track A — Scaffold (l2-source-layout, l2-technology-stack)
- [ ] [T-1A01] Initialize Cargo workspace (`crates/{core,cli,tui}` + root manifest, pinned toolchain)
- [ ] [T-1A02] Initialize pnpm workspace (`packages/ui`: Vite + React 19 + TS) and `apps/desktop` Tauri v2 scaffold
- [ ] [T-1A03] Configure the polyglot runner (moon/Nx) to sequence JS + Tauri builds; exclude Rust `target/` from runner cache

Track B — Filesystem (l2-filesystem-layout)
- [ ] [T-1B01] Implement the OS-native path resolver (program/state/cache/logs + portable override)
- [ ] [T-1B02] Implement idempotent state-tier bootstrap from templates (`init`)

Track C — Core (l2-core-library)
- [ ] [T-1C01] Create `crates/core` skeleton + public capability contract (the surface frontends call)
- [ ] [T-1C02] Implement durable state persistence interface + restartable load

Track D — Workflow Runtime (l2-workflow-runtime)
- [ ] [T-1D01] Integrate the external workflow-runtime crate (or an in-core stub if not yet available); load schema + validate a sample workflow
- [ ] [T-1D02] Wire the step-handler binding interface (command-category → subsystem dispatch table)

Track E — Security (l2-security)
- [ ] [T-1E01] Ship secure-default `.gitignore` + secret store read path (`.env`/keychain)
- [ ] [T-1E02] Implement output/log redaction + a default-deny outbound egress gate

Track T — Validation
- [ ] [T-1T01] Validate scaffold & dependency direction (Track A)
- [ ] [T-1T02] Validate path resolution & state bootstrap (Track B)
- [ ] [T-1T03] Validate core contract & no-frontend-deps (Track C)
- [ ] [T-1T04] Validate workflow run/transpile round-trip (Track D)
- [ ] [T-1T05] Validate secret isolation & sandbox defaults (Track E)

## Detailed Tracking

### [T-1A01] Cargo workspace
- **Spec:** l2-source-layout.md §4.1
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo metadata --no-deps` lists members `core`, `cli`, `tui`; `cargo build` succeeds on the empty workspace.
- **Handoff:** enables T-1C01 (core crate lives here).
- **Notes:** root Cargo workspace; pin Rust toolchain (`rust-toolchain.toml`).

### [T-1A02] JS workspace + Tauri scaffold
- **Spec:** l2-source-layout.md §4.1, l2-technology-stack.md §frontend/shell
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `pnpm install` succeeds; `pnpm -C packages/ui build` emits assets; `apps/desktop` resolves `cargo tauri` and `cargo tauri build` produces a desktop bundle on the host OS.
- **Handoff:** shell ready for Phase 5 UI.
- **Notes:** React 19 + Vite + TS + Tailwind v4 floor per stack §5; do a mobile (iOS/Android) build smoke EARLY (risk).

### [T-1A03] Polyglot runner
- **Spec:** l2-source-layout.md §4.3, l2-technology-stack.md §monorepo
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** runner task graph runs `ui` build then `tauri build` in order; runner cache config contains no `target/**` (Cargo/sccache owns Rust caching).
- **Handoff:** CI wiring for T-1T01.
- **Notes:** moon or Nx (+@monodon/rust) per stack verdict.

### [T-1B01] Path resolver
- **Spec:** l2-filesystem-layout.md §4.1
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** unit tests resolve program/state/cache/logs roots to the documented paths for Windows, macOS, and Linux (XDG); portable override maps all roots under one directory.
- **Handoff:** required by T-1B02 and T-1C02.
- **Notes:** single resolver in core.

### [T-1B02] State-tier bootstrap
- **Spec:** l2-filesystem-layout.md §4.3, l1-storage-model.md (STO-1/3)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** running `init` on an empty state dir produces the §4.3 tree (config json, `.env.example`, `memory/`, `employees/`, `workspaces/`); a second run is idempotent (no changes).
- **Handoff:** state ready for memory/workspace phases.
- **Notes:** copy from program-tier templates; never write the program tier.

### [T-1C01] Core skeleton + contract
- **Spec:** l2-core-library.md, l1-architecture.md (INV-1/2/3)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo build -p core` succeeds; the public capability surface compiles; `crates/cli` invokes one no-op core capability over the contract.
- **Handoff:** unblocks Phases 2–5 (critical path).
- **Notes:** library crate, zero frontend deps; capability traits frontends bind to.

### [T-1C02] Durable state + restartable load
- **Spec:** l2-core-library.md, l1-storage-model.md (STO-2), l1-architecture.md (INV-5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** a test opens SQLite at the resolved state path, persists a value, drops/reopens the handle, and reloads it without loss (simulated restart).
- **Handoff:** memory store (Phase 2) builds on this.
- **Notes:** depends on T-1B01 (path) — sequence after it.

### [T-1D01] Workflow runtime integration
- **Spec:** l2-workflow-runtime.md §4.1, l1-workflow-language.md (WFL-2/5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo build` resolves the workflow-runtime dependency; a test loads the schema and validates a sample workflow (lint pass). If the Rust runtime crate is unavailable, a minimal in-core stub validates a fixed sample and the task is marked `Blocked [!]` on the external crate with that reason.
- **Handoff:** orchestration/scheduler (Phases 3–4) execute workflows.
- **Notes:** external crate (own repo); honest dependency — see PLAN risks.

### [T-1D02] Step-binding interface
- **Spec:** l2-workflow-runtime.md §4.2, l1-workflow-language.md (WFL-7)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** a sample workflow with a `log` and a `generate` step dispatches to core stub handlers; an unbound command category errors explicitly (no silent no-op).
- **Handoff:** subsystems plug their handlers in later phases.
- **Notes:** dispatch table command-category → subsystem.

### [T-1E01] Secret store + gitignore
- **Spec:** l2-security.md (SEC-1/2)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `git check-ignore .env` reports ignored (and `.env.example` is NOT ignored); reading a secret via the store never writes the value to any log sink (asserted by test).
- **Handoff:** all outbound/secret paths use this.
- **Notes:** `.env`/OS keychain; ship the secure-default `.gitignore`.

### [T-1E02] Redaction + egress gate
- **Spec:** l2-security.md (SEC-3/5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** a test asserts a known secret token is redacted in rendered output/logs; an outbound send on a non-authorized path is denied and written to the audit log.
- **Handoff:** telemetry/error-reporting/model-cloud routing pass this gate.
- **Notes:** default-deny egress; single gate.

### [T-1T01] Validation — scaffold & dependency direction
- **Goal:** Verify Track A against l2-source-layout §4.1/4.2.
- **Method:** CI builds all workspace members; `cargo tree -p core` shows no `cli`/`tui`/`tauri`/UI dependency (dependency points inward).
- **Status:** Todo

### [T-1T02] Validation — paths & bootstrap
- **Goal:** Verify Track B against l2-filesystem-layout §4.1/4.3.
- **Method:** run path-resolver tests per-OS; assert bootstrap tree matches §4.3 and that no path outside the state tier is written during normal operation (STO-1).
- **Status:** Todo

### [T-1T03] Validation — core contract
- **Goal:** Verify Track C against l1-architecture INV-1/2.
- **Method:** `cargo tree -p core` (no frontend deps); restart-persistence test from T-1C02 passes.
- **Status:** Todo

### [T-1T04] Validation — workflow round-trip
- **Goal:** Verify Track D against l1-workflow-language WFL-1/5/8.
- **Method:** validate → run a sample workflow returns a structured result; `transpile --to=human` then back to compact is lossless (AST-equal).
- **Status:** Todo

### [T-1T05] Validation — security baseline
- **Goal:** Verify Track E against l1-security SEC-1/3/5/6.
- **Method:** backup excludes `.env` (test); redaction test passes; sandbox wrapper denies network by default; egress audit records a denied send.
- **Status:** Todo

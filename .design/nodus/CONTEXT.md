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

- T-23A01: Added `InMemoryStorageProvider` to `portability.rs` — `Mutex<Vec<(String, Value)>>`, insert-or-overwrite `store`, clone-on-`load`, poisoned-lock recovery via `unwrap_or_else(|poisoned| poisoned.into_inner())` — replacing `NoopStorageProvider`, whose discarding `store`/always-`None` `load` satisfied neither L1 §4.1's in-memory built-in mandate nor LP-2's in-process-sufficiency requirement
- T-23A02: Updated `lib.rs`'s re-export to `InMemoryStorageProvider`; scoped `portability.rs`'s module doc's "pending LP-3 graduation" note to wiring only, since the built-in itself is now conformant; removed a stale `NoopStorageProvider` citation from `DefaultConfigProvider`'s doc comment
- T-23C01: `l2-nodus-portability` 1.3.0 → 1.3.1 — Overview, §4.2 registry row, §4.3 heading/body, §3.1's LP-15 row, and §5 item 3 (plus its closing sentence) reconciled to the as-built fix; self-review caught two live references the first pass missed (§3's LP-2 row, §4.5's module map), both corrected in the same bump; `INDEX.md` row + top-level version synced
- T-23C02: **Cancelled (superseded)** — its target (correcting §5 item 2's LP-11 framing) was already reached by the prior spec pass, from the opposite direction (opening the gate rather than demoting the claim); nothing left for this task to do
- T-23T01: Split the Phase-5 contract test's storage half out of `noop_storage_and_policy_compile` (renamed `noop_policy_and_schema_compile`, storage assertions removed) into 4 dedicated integration tests — round-trip, overwrite, absent-key, instance-isolation — plus 3 equivalent in-crate unit tests in `portability.rs`
- T-23T02: `cargo test -p nodus` — 435 passed (was 429; +6), 0 failed (every suite `ok`, verified against a precise `test result: FAILED` pattern rather than a naive `passed`/`failed` substring sum, which several `NODUS:*_FAILED` error-code strings would false-positive); clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]` (the one new `lock()` call recovers a poisoned guard); `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 24 — Per-Effect Authorization Call Site (l1-nodus-portability §4.7 · l2-nodus-portability §4.9 · LP-11) (2026-07-31)

- T-24A01: Added `EffectClass{ModelCall,Deferred}` + `effect_class_of` (reusing `MODEL_COMMANDS`/`DIALOG_COMMANDS`) + `EffectClass::as_gate_str` to `portability.rs`; `NODUS:POLICY_DENIED` to `vocab.rs` (`error_code` module, `error_meta` match as `(Error, Runtime)`, `error_registry_lockstep` test 28 → 29)
- T-24A02: Added `Executor.policy: Box<dyn PolicyProvider>` fourth field (default `NoopPolicyProvider` across all six existing constructors) + new `with_policy`/`with_policy_and_audit` constructors
- T-24A03: Added the gate in `execute_command` — after `check_rules`, before the `ASK`/`CONFIRM` dispatch, covering both effect classes with one check; denial emits `StepError`+`NODUS:POLICY_DENIED` (never `ConstraintHit`) and returns bare `None` (non-halting, mirroring `DialogOutcome::Timeout`/`::Rejected`). Added `run_with_policy`/`run_with_policy_and_audit` to `workflows.rs` copying `run_with_dialog`'s shape; re-exported from `lib.rs`. Corrected two stale doc comments found in the shipped crate: `portability.rs`'s module doc and `PolicyProvider`'s own trait doc still said "executor integration is deferred until LP-3 is satisfied"
- T-24C01: `l2-nodus-portability` 1.4.0 → 1.5.0 — Overview, §4.2 registry row, §4.4 heading/framing, §4.9 header + three `[REFERENCE]` blocks (corrected `as_str` → `as_gate_str`, the fuller denial `reason` string, "stays unset" → "stays at its seeded default"), §3.1's LP-11 row, and §5 item 2 reconciled to Implemented; `INDEX.md` row + top-level version synced
- T-24T01: Added 4 unit tests in `portability.rs` (`effect_class_of` classification + gate strings) and 5 integration tests in `tests/portability.rs` (permit/deny on both `ModelCall` and `Deferred`, plus a `NoopPolicyProvider` regression). Testing caught a wrong assumption empirically: a denied step's pipeline target is not *absent* from `RunResult.vars` but stays at its seeded default (`Value::Null` for `out`), since reserved variables are always pre-populated at context construction — fixed the assertions accordingly, which also strengthened the previously-vacuous permitted-effect checks
- T-24T02: `cargo test -p nodus` — 444 passed (was 435; +9), 0 failed; clippy `-D warnings` clean; fmt found one line-wrap violation in the new tests (no logic change), fixed; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)


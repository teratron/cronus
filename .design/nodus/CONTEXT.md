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


## Phase 24 — Per-Effect Authorization Call Site (l1-nodus-portability §4.7 · l2-nodus-portability §4.9 · LP-11) (2026-07-31)

- T-24A01: Added `EffectClass{ModelCall,Deferred}` + `effect_class_of` (reusing `MODEL_COMMANDS`/`DIALOG_COMMANDS`) + `EffectClass::as_gate_str` to `portability.rs`; `NODUS:POLICY_DENIED` to `vocab.rs` (`error_code` module, `error_meta` match as `(Error, Runtime)`, `error_registry_lockstep` test 28 → 29)
- T-24A02: Added `Executor.policy: Box<dyn PolicyProvider>` fourth field (default `NoopPolicyProvider` across all six existing constructors) + new `with_policy`/`with_policy_and_audit` constructors
- T-24A03: Added the gate in `execute_command` — after `check_rules`, before the `ASK`/`CONFIRM` dispatch, covering both effect classes with one check; denial emits `StepError`+`NODUS:POLICY_DENIED` (never `ConstraintHit`) and returns bare `None` (non-halting, mirroring `DialogOutcome::Timeout`/`::Rejected`). Added `run_with_policy`/`run_with_policy_and_audit` to `workflows.rs` copying `run_with_dialog`'s shape; re-exported from `lib.rs`. Corrected two stale doc comments found in the shipped crate: `portability.rs`'s module doc and `PolicyProvider`'s own trait doc still said "executor integration is deferred until LP-3 is satisfied"
- T-24C01: `l2-nodus-portability` 1.4.0 → 1.5.0 — Overview, §4.2 registry row, §4.4 heading/framing, §4.9 header + three `[REFERENCE]` blocks (corrected `as_str` → `as_gate_str`, the fuller denial `reason` string, "stays unset" → "stays at its seeded default"), §3.1's LP-11 row, and §5 item 2 reconciled to Implemented; `INDEX.md` row + top-level version synced
- T-24T01: Added 4 unit tests in `portability.rs` (`effect_class_of` classification + gate strings) and 5 integration tests in `tests/portability.rs` (permit/deny on both `ModelCall` and `Deferred`, plus a `NoopPolicyProvider` regression). Testing caught a wrong assumption empirically: a denied step's pipeline target is not *absent* from `RunResult.vars` but stays at its seeded default (`Value::Null` for `out`), since reserved variables are always pre-populated at context construction — fixed the assertions accordingly, which also strengthened the previously-vacuous permitted-effect checks
- T-24T02: `cargo test -p nodus` — 444 passed (was 435; +9), 0 failed; clippy `-D warnings` clean; fmt found one line-wrap violation in the new tests (no logic change), fixed; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 25 — Effect Risk-Class Declaration (l1-nodus-portability §4.12 · l2-nodus-portability §4.10 · LP-16) (2026-07-31)

- T-25A01: Extended `execute_command`'s existing LP-11 `context` literal with a small loop reading `+reversible`/`+external`/`+value` off `cmd.modifiers`, pushing each stripped-prefix pair only when present — descriptors are omitted, never defaulted, when a step carries no matching modifier. No new DSL grammar, AST field, `Value` kind, or `PolicyProvider` signature change; the only production-code change in this phase
- T-25C01: `l2-nodus-portability` 1.6.0 → 1.6.1 — §4.10 header + §4.10.2's `[REFERENCE]` pseudocode (corrected to show the real inline literal, not the illustrative separate `build_context` function), §3.1's LP-16 row, its Leverage paragraph's invariant count, and §5 item 7 reconciled to Implemented/Done; `INDEX.md` row + top-level version synced
- T-25T01: Added 3 integration tests in `tests/portability.rs` — a decorated `GEN` step's descriptors reach a capturing `PolicyProvider`'s `context` verbatim (`risk_descriptors_reach_context_when_declared`); an undecorated step's `context` omits all three keys entirely (`undeclared_risk_descriptors_are_absent_from_context_not_defaulted`); a `NoopPolicyProvider`/plain-`run` regression proving the modifiers are inert without a consulting host (`risk_descriptors_are_inert_without_a_policy_provider`). All three passed on first run — no empirical surprises, unlike Phase 24's reserved-variable-seeding catch. `cargo test -p nodus` — 447 passed (was 444; +3), 0 failed; clippy `-D warnings` clean; fmt found one line-wrap violation in the new tests (no logic change), fixed; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]` in `executor.rs`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)


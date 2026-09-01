# Project Context

**Generated:** 2026-09-01

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
├── .qwen/
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

## Phase 29 — Declared Budget Measure (l1-nodus-environment §3/NE-14 · l2-nodus-environment §4.4.1) (2026-08-01)

- T-29A01: `EnvironmentProfile` and `CandidateResult` (`environment.rs`) each gained `token_measure: Option<String>`; `EnvironmentProfile::empty()` and `EnvRunResult::candidate()` updated (`candidate()` gained a fourth parameter, an LP-6 pre-1.0 signature change); fixed 5 call sites across `environment.rs`'s own tests and `tests/environment.rs`
- T-29C01: `vocab.rs` gained `error_code::ENV_MEASURE_UNKNOWN` + `(Error, Control)` metadata beside `CAPABILITY_UNMET` (lockstep test 30 → 31). `workflows.rs` gained `env_measure_rejection` (sibling to `capability_rejection`) + the NE-14 check itself, wired into the single shared `run_with_environment_impl` immediately after `env.profile()` — the plan-time-corrected mechanism (`Ok(EnvRunResult)`/`RuntimeError`, not `Diagnostic`; after `env.profile()`, not before `env.open`) proved exactly right, no further adjustment needed
- T-29T01: Added `MeasureEnv` test double + 4 integration tests in `tests/environment.rs` (rejection proves `env.open`/`reset` ran but no workflow step or `evaluate` did; measure-present regression; no-budget regression; `CandidateResult` carries the measure through). Reconciled `l2-nodus-environment.md` 1.1.0 → 1.1.1 — §4.4.1's `Diagnostic`/"before `env.open`" claims corrected to the real mechanism; §3's NE-14 row → Implemented. `INDEX.md` row + top-level version synced. `cargo test -p nodus` — 471 passed (was 467; +4), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

**All three phases planned in the `/magic.task nodus` cycle (27, 28, 29) are now Done.**

## Phase 30 — DG-11 Authoring Advisories (l1-nodus-dialog §3/DG-11 · l2-nodus-dialog §4.8) (2026-09-01)

- T-30A01: `MODEL_COMMANDS`/`DIALOG_COMMANDS` widened `const` → `pub(crate) const` in `portability.rs`; `validator.rs` gained `use crate::portability::{DIALOG_COMMANDS, MODEL_COMMANDS};` and `w016_dialog_placement`, which flattens `wf.steps` into one root scope and calls `w016_scan_scope`. **Grounding correction**: `CommandCall.modifiers` keys carry the surface `+` (matched `"+reversible"`/`"+external"` verbatim, per the LP-11/LP-16 call site at `executor.rs:1608-1613`), not the bare name a derived-`context`-map test had suggested.
- T-30A02: `w016_scan_scope` (pairing within one flat sequence: a dialog `D`'s last qualifying `S`, stopping at the first node reading `D`'s target) + `w016_recurse_stmt`/`w016_recurse_conditional` (independent recursion into `Conditional`/`ForLoop`/`UntilLoop` bodies; `Parallel` branches walked for nesting but never paired, since they run concurrently). **Correction**: `?SWITCH` arms/`~MAP`'s command are a single `CommandCall`, not a sequence — no recursion target, unlike the plan's assumption.
- T-30B01: `w017_dialog_payload_inlining` + four free functions (`w017_collect_producers`/`_conditional`, `w017_scan_stmt`/`_conditional`) — a **dedicated** `target-root → producing command name` map (not a reuse of `collect_vars_stmt`, which tracks declared/used sets only), firing on a bare `$var` `ASK`/`CONFIRM` argument whose producer is in `MODEL_COMMANDS`.
- T-30C01: `l2-nodus-dialog.md` 1.2.0 → 1.2.1 — §4.8.3 corrected to the whole-arg reference model (nodus has no interpolation scanner) and the dedicated-walker mechanism; §4.8.4's scope-list bullet corrected to name exactly `?IF`/`?ELIF`/`?ELSE`/`~FOR`/`~UNTIL` plus the `~PARALLEL`/`?SWITCH`/`~MAP` exclusions found at T-30A02. `INDEX.md` row + top-level version synced (1.0.92 → 1.0.93).
- T-30T01/T-30T02: Added 11 unit tests in `validator.rs` (7 `W016` + 4 `W017`, built via direct `WorkflowFile`/`Step`/`Stmt` struct literals for precise nested-block control) plus two small helpers (`cmd_step`, `wf_with_steps`). Ran the full `tests/fixtures/` corpus (14 files) through a scratch example, deleted after use: zero fixtures newly emit either code — checked by hand why (an `@in`-sourced argument with no producer; a branch-action `ASK` that is never a candidate by construction). `cargo test -p nodus` — 482 passed (was 471; +11), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

# Project Context

**Generated:** 2026-08-12

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
│   ├── desktop.drawio.svg
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── release.drawio.svg
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

- T-27T01: Added `SETTLE_WF` fixture, `CapturingSettlementRail`, and 6 integration tests in `tests/portability.rs` (settle-and-bind, gate-denial short-circuit before the rail, rail-returns-`None` unaccounted, both denial paths reaching NL-9 dispatch, positional `context.args` verbatim, byte-for-byte regression for `SETTLE`-free workflows) plus a manifest-derivation unit test in `portability.rs`. **Found and fixed one wrong assumption empirically**: the first fixture draft used a non-reserved `@out: $receipt` pipeline target and asserted `Some(Value::Null)` on denial/unaccounted; only reserved variables (`out`/`error`/`meta`) are pre-seeded, so a custom name starts absent from `vars` — fixed by using the reserved `$out` binding. Reconciled `l2-nodus-settlement.md` 1.0.0 → 1.0.1 and `l2-nodus-portability.md` 1.7.0 → 1.7.1 (§3.1 LP-17 row → Implemented, three of twelve LP-9…LP-20 items now done); `INDEX.md` rows + top-level version synced. `cargo test -p nodus` — 462 passed (was 452; +10), 0 failed; clippy `-D warnings` clean; fmt clean after one auto-fix (import line wrap); no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 28 — Memoizable & Promotable Approval (l1-nodus-dialog §3/DG-9,DG-10 · l2-nodus-dialog §4.7) (2026-08-01)

- T-28A01: `DialogOutcome` gained `Remembered(Value)` in `executor.rs`; `observability.rs` gained `pub enum DialogProvenance { Answered, Remembered }` beside `EventAnnotations`, plus a fifth field `dialog_provenance: Option<DialogProvenance>` — the struct's doc comment updated to note this is the first field the crate's own dispatch logic populates directly, not a host-supplied one; re-exported from `lib.rs`
- T-28C01: `handle_dialog`'s `match outcome { .. }` now returns `(signal, dialog_provenance)` — `Remembered(value)` gained its own arm identical to `Answer`'s (`ctx.log_step`/`ctx.set_var`), both now also return their respective provenance, `Pause`/`Timeout`/`Rejected` return `None`; the single `StepEnd` emit's `annotations` changed from `EventAnnotations::default()` to `EventAnnotations { dialog_provenance, ..Default::default() }` — one emit call, unchanged
- T-28T01: Added `DialogRemembers`, `RecordingAudit`, `step_end_provenance` helper, and 4 integration tests in `tests/portability.rs` plus a unit test in `observability.rs`. **Found and fixed one wrong assumption empirically**: asserting no-provenance-on-rejection against `DEFERRED_WF` (declares `@err: ESCALATE(human)`) initially expected one `StepEnd`; `DIALOG_REJECTED` is `Signal`-free so NL-9 dispatch fires automatically, landing two `StepEnd` events (the rejected `ASK`, then the dispatched `ESCALATE`), both correctly provenance-free — the test was corrected. Reconciled `l2-nodus-dialog.md` 1.1.0 → 1.1.1 (§3 DG-9/DG-10 rows → Implemented; §4.7 gains a confirmation note that `dialog_provenance` is the first crate-populated `EventAnnotations` field). `INDEX.md` row + top-level version synced. `cargo test -p nodus` — 467 passed (was 462; +5), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 29 — Declared Budget Measure (l1-nodus-environment §3/NE-14 · l2-nodus-environment §4.4.1) (2026-08-01)

- T-29A01: `EnvironmentProfile` and `CandidateResult` (`environment.rs`) each gained `token_measure: Option<String>`; `EnvironmentProfile::empty()` and `EnvRunResult::candidate()` updated (`candidate()` gained a fourth parameter, an LP-6 pre-1.0 signature change); fixed 5 call sites across `environment.rs`'s own tests and `tests/environment.rs`
- T-29C01: `vocab.rs` gained `error_code::ENV_MEASURE_UNKNOWN` + `(Error, Control)` metadata beside `CAPABILITY_UNMET` (lockstep test 30 → 31). `workflows.rs` gained `env_measure_rejection` (sibling to `capability_rejection`) + the NE-14 check itself, wired into the single shared `run_with_environment_impl` immediately after `env.profile()` — the plan-time-corrected mechanism (`Ok(EnvRunResult)`/`RuntimeError`, not `Diagnostic`; after `env.profile()`, not before `env.open`) proved exactly right, no further adjustment needed
- T-29T01: Added `MeasureEnv` test double + 4 integration tests in `tests/environment.rs` (rejection proves `env.open`/`reset` ran but no workflow step or `evaluate` did; measure-present regression; no-budget regression; `CandidateResult` carries the measure through). Reconciled `l2-nodus-environment.md` 1.1.0 → 1.1.1 — §4.4.1's `Diagnostic`/"before `env.open`" claims corrected to the real mechanism; §3's NE-14 row → Implemented. `INDEX.md` row + top-level version synced. `cargo test -p nodus` — 471 passed (was 467; +4), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

**All three phases planned in the `/magic.task nodus` cycle (27, 28, 29) are now Done.**


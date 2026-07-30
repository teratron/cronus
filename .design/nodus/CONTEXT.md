# Project Context

**Generated:** 2026-07-30

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

<!-- NOTE: Phases 11-19 have no entry in this internal phase journal — their
Changelog-L1 append step appears to have been skipped in the sessions that ran
them. Permanent record for that span exists in RETROSPECTIVE.md (snapshot
metrics per phase) and root CHANGELOG.md (user-facing, phase-content redacted
per §6). Not backfilled here — out of scope for Phase 20; flagged for the
Backlog rather than silently left unexplained. -->

## Phase 20 — Branch-Action Pipeline Targets & `?SWITCH` Binding Conformance (l2-nodus-control-flow §3/§4.5) (2026-07-30)

- T-20A01: `try_parse_command_from_string` (parser.rs) now splits a trailing `→ $target` off via `rsplit_once('→')` *before* the `(` split, so the no-paren path cannot swallow it into the command name; fixes all 3 call sites (`?SWITCH` arms/default, `?IF` branch actions, `@err:` handlers) with one change. 4 new unit tests. Executor untouched (already honored `pipeline_target`)
- T-20B01: No code change — `collect_vars_stmt`'s `Stmt::Switch` arm already walked `sw.arms`/`sw.default` via `collect_vars_cmd` (declares targets, marks uses); the plan-time "second coupled defect" was a `sed` line-range grounding artifact, corrected rather than left standing. 2 regression tests added confirming the existing behavior
- T-20C01: Reconciled `l2-nodus-restart.md` to as-built (1.0.0 → 1.0.1): no `Signal::Restart` variant exists (executor reads `$restart` post-hoc off `RunResult.vars`); nested-restart rejection is bare validator code `E019`, not a `vocab.rs` `NODUS:RESTART_SCOPE` constant; §5/INDEX no longer misgroup `$restart` into `RUNTIME_OWNED_VARIABLES`
- T-20C02: Reconciled `l2-nodus-compensation.md` to as-built (1.0.0 → 1.0.1): ledger is `CompletedEffect { step_number, compensation }` only (no `step_identity`/`CompensationOutcome`/`uncompensable`); canonical example `UNPUBLISH` → `NOTIFY` (confirmed in `KNOWN_COMMANDS`); removed a never-built "explicit compensate request" arming trigger; fixed three dangling `§7` cross-references (document has no §7) to `§6`
- T-20T01: `tests/control_flow.rs` — 2 new tests (`switch_arm_bound_target_reachable_through_run`, `switch_arm_targets_bind_independently_per_arm`) proving `?SWITCH` arm targets are reachable through `workflows::run` and bind per-arm, not to one shared target; 3 pre-existing zero-target `?SWITCH` tests pass unmodified (non-regression). Retired the Phase-18 `restart.rs` AST workaround — `restart_count_progresses_and_context_is_fresh_each_attempt` now runs a real parsed fixture through `run_with_audit`
- T-20T02: `cargo test -p nodus` — 405 passed (was 397; +8), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)


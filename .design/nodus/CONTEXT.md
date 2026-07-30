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

- T-20B01: No code change — `collect_vars_stmt`'s `Stmt::Switch` arm already walked `sw.arms`/`sw.default` via `collect_vars_cmd` (declares targets, marks uses); the plan-time "second coupled defect" was a `sed` line-range grounding artifact, corrected rather than left standing. 2 regression tests added confirming the existing behavior
- T-20C01: Reconciled `l2-nodus-restart.md` to as-built (1.0.0 → 1.0.1): no `Signal::Restart` variant exists (executor reads `$restart` post-hoc off `RunResult.vars`); nested-restart rejection is bare validator code `E019`, not a `vocab.rs` `NODUS:RESTART_SCOPE` constant; §5/INDEX no longer misgroup `$restart` into `RUNTIME_OWNED_VARIABLES`
- T-20C02: Reconciled `l2-nodus-compensation.md` to as-built (1.0.0 → 1.0.1): ledger is `CompletedEffect { step_number, compensation }` only (no `step_identity`/`CompensationOutcome`/`uncompensable`); canonical example `UNPUBLISH` → `NOTIFY` (confirmed in `KNOWN_COMMANDS`); removed a never-built "explicit compensate request" arming trigger; fixed three dangling `§7` cross-references (document has no §7) to `§6`
- T-20T01: `tests/control_flow.rs` — 2 new tests (`switch_arm_bound_target_reachable_through_run`, `switch_arm_targets_bind_independently_per_arm`) proving `?SWITCH` arm targets are reachable through `workflows::run` and bind per-arm, not to one shared target; 3 pre-existing zero-target `?SWITCH` tests pass unmodified (non-regression). Retired the Phase-18 `restart.rs` AST workaround — `restart_count_progresses_and_context_is_fresh_each_attempt` now runs a real parsed fixture through `run_with_audit`
- T-20T02: `cargo test -p nodus` — 405 passed (was 397; +8), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 21 — Compact-Form Round-Trip Fidelity for Control Flow (NL-6) (2026-07-30)

- T-21A01: Added `nodus_stmt` (exhaustive dispatcher, no `_` arm), `nodus_conditional_chain`/`nodus_conditional_branch` (`?IF`/`?ELIF`/`?ELSE` + flags + action), `nodus_switch` (arms + `*` default + `~END`) to `transpiler.rs`. `Conditional.body` confirmed empirically dead on the parse path — deliberately not rendered
- T-21A02: Added `nodus_for`, `nodus_until` (handles `MAX:n` present or absent), `nodus_map` (re-attaches the block-level target to a cloned inner command), and the shared `push_indented_body` helper
- T-21A03: Added `sub_steps` emission (found NOT dead on the parse path, contrary to the plan-time assumption — `ticket_triage.nodus` already exercises it) and `Step.retry` (`~RETRY:n`, completely unemitted, a second finding beyond the planned scope) to `nodus_step`. Added `nodus_command_or_assignment` detecting the `$var = expr` shorthand's synthetic `Stmt::Command{name:"ASSIGN",..}` shape (unrenderable via generic call syntax since "ASSIGN" isn't a lexable command name) and emitting the shorthand back. Also fixed, found via the same harness: `WorkflowFile.comments` was never emitted (added right after the header), and `nodus_command`'s modifier values were never re-quoted when containing whitespace (both were blocking every corpus fixture, not just one)
- T-21B01: Added `full_corpus_ast_equal_after_compact_round_trip` to `tests/parity.rs` (not `workflows.rs`, where the spec incorrectly claimed it lived — spec patched), scoped to `.steps` equality after a whole-struct pass surfaced an out-of-scope, pre-existing `@test:` block re-emission gap owned by `l2-nodus-testing.md`
- T-21C01: Added `switch_dispatch.nodus`, `retry_bounded.nodus`, `halt_pause.nodus` fixtures, completing the v0.7 normative corpus Phase 17 started with `map_transform.nodus`
- T-21T01: 11 new construct-specific round-trip unit tests in `transpiler.rs` (one per `Stmt` variant + `~RETRY:n` + a nested case) plus the corpus-wide harness; all pre-existing round-trip tests pass unmodified
- T-21T02: `cargo test -p nodus` — 420 passed (was 405; +15), 0 failed; clippy `-D warnings` clean (1 `collapsible_if` fixed); fmt clean (3 findings fixed); no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)


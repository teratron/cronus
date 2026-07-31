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

- T-21A02: Added `nodus_for`, `nodus_until` (handles `MAX:n` present or absent), `nodus_map` (re-attaches the block-level target to a cloned inner command), and the shared `push_indented_body` helper
- T-21A03: Added `sub_steps` emission (found NOT dead on the parse path, contrary to the plan-time assumption — `ticket_triage.nodus` already exercises it) and `Step.retry` (`~RETRY:n`, completely unemitted, a second finding beyond the planned scope) to `nodus_step`. Added `nodus_command_or_assignment` detecting the `$var = expr` shorthand's synthetic `Stmt::Command{name:"ASSIGN",..}` shape (unrenderable via generic call syntax since "ASSIGN" isn't a lexable command name) and emitting the shorthand back. Also fixed, found via the same harness: `WorkflowFile.comments` was never emitted (added right after the header), and `nodus_command`'s modifier values were never re-quoted when containing whitespace (both were blocking every corpus fixture, not just one)
- T-21B01: Added `full_corpus_ast_equal_after_compact_round_trip` to `tests/parity.rs` (not `workflows.rs`, where the spec incorrectly claimed it lived — spec patched), scoped to `.steps` equality after a whole-struct pass surfaced an out-of-scope, pre-existing `@test:` block re-emission gap owned by `l2-nodus-testing.md`
- T-21C01: Added `switch_dispatch.nodus`, `retry_bounded.nodus`, `halt_pause.nodus` fixtures, completing the v0.7 normative corpus Phase 17 started with `map_transform.nodus`
- T-21T01: 11 new construct-specific round-trip unit tests in `transpiler.rs` (one per `Stmt` variant + `~RETRY:n` + a nested case) plus the corpus-wide harness; all pre-existing round-trip tests pass unmodified
- T-21T02: `cargo test -p nodus` — 420 passed (was 405; +15), 0 failed; clippy `-D warnings` clean (1 `collapsible_if` fixed); fmt clean (3 findings fixed); no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 22 — Whole-File NL-6 Round-Trip Closure (l2-nodus-testing §10 / l1-nodus-language NL-6) (2026-07-30)

- T-22A01: Added `Transpiler::nodus_braced_raw_body` (re-quotes every `raw_lines` token except literal `{`/`}`, which must stay unquoted since `collect_braced_raw_lines`'s depth counter matches token *type*, not value). Inverted the `@test:` emission branch in `to_nodus` to prefer `raw_lines` when non-empty; structured-field emission is now reached only for a programmatically-constructed `TestBlock`
- T-22A02: Added `Validator::w015_test_pair_separator`, mirroring `parse_test_body`'s own section-tracking control flow so a legitimately-consumed `key : value` triple's value token is never re-examined as a would-be key. Warning-severity; co-fires with `W009` by design when all of a block's `expected:` pairs are non-conforming
- T-22B01: Added `@macro:` emission, reusing `nodus_braced_raw_body` directly. Found the real corpus fixture (`macro_expand.nodus`) uses the non-braced `@macro: name` form whose body `parse_macro_block` never captures (macro body expansion is a separately deferred feature) — `MacroBlock.raw_lines` is empty in the corpus today, so the braced-body path is implemented but untested by any fixture (recorded in PLAN.md Backlog)
- T-22B02: Added `human_mode` emission, deliberately placed **last** in `to_nodus` — `collect_comment_block` greedily consumes every following Comment token, so emitting it earlier would silently absorb any free-standing comment into it on re-parse
- T-22T01: Widened `full_corpus_ast_equal_after_compact_round_trip` (`tests/parity.rs`) from `ast1.steps == ast2.steps` to `ast1 == ast2` — green across the whole 11-fixture normative corpus. Per-task sensitivity confirmed empirically: temporarily disabling `@macro:` emission failed the widened harness with the exact predicted `macros` diff, then restored
- T-22T02: `cargo test -p nodus` — 429 passed (was 420; +9), 0 failed; clippy `-D warnings` clean; fmt clean (1 finding fixed); no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)


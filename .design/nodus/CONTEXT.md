# Project Context

**Generated:** 2026-06-27

## Active Technologies

- Rust

## Core Project Structure

```plaintext
.
├── .agents/
│   ├── rules/
│   ├── skills/
│   └── workflows/
├── .claude/
│   ├── commands/
│   ├── rules/
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
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── desktop.drawio.svg
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── release.drawio.svg
│   └── technology-stack-research.md
├── .env
├── .env.example
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
├── crates/
│   ├── cli/
│   ├── codegraph/
│   ├── core/
│   ├── nodus/
│   └── tui/
├── docs/
├── installer/
├── packages/
│   └── ui/
└── rust-toolchain.toml
```

## Recent Changes

- T-7C01: Implemented `CapabilityManifest::from_workflow` — walks the AST (conditionals/loops/parallel), mapping model commands (GEN/ANALYZE) → Model role and non-builtin commands → Vocabulary role + required command name; `@needs` DSL declaration deferred to parity backlog
- T-7T01: Integration test `manifest_lp3_two_host_substitution` — host with Storage runs to completion; host without it is rejected fail-fast naming Storage
- T-7T02: Integration test `manifest_rejects_before_side_effects` — counting audit sink records zero events on a rejected run; control run proves the sink counts
- T-7T03: `cargo test -p nodus` — 217 passed (was 204; +13), 0 failed; `cargo clippy --all-targets -- -D warnings` — zero lints; `cargo fmt --check` — clean; `cargo doc --no-deps` — only the pre-existing `test`-fn baseline warning
- T-7D01: Authored l2-nodus-portability.md §4.7 (capability manifest Rust design); §3 LP-8 row → Implemented; bumped v1.0.0 → 1.1.0, RFC → Stable; synced INDEX.md (v1.0.13)

## Phase 8 — Error Taxonomy (l2-nodus-errors) (2026-06-27)

- T-8A01: Added `ErrorSeverity` (Error/Warn/Info) and `ErrorCategory` (Parse/Runtime/Validation/Routing/Memory/Test/Control/Dialog) enums to `vocab.rs`
- T-8A02: Added the 14 new `error_code` constants (UNDEFINED_CMD, UNDEFINED_MACRO, VALIDATION_FAILED, ESCALATION_FAILED, CONFIDENCE_LOW, KB_UNAVAILABLE, MEMORY_FAILED, TEST_FAILED, SWITCH_NO_MATCH, PAUSED, COUNTER_OVERFLOW, GIT_UNAVAILABLE, DIALOG_TIMEOUT, DIALOG_REJECTED)
- T-8A03: Added the `error_meta(code) -> Option<(ErrorSeverity, ErrorCategory)>` static registry mapping each canonical code to its severity×category
- T-8B01: Marked `EXECUTION_FAILED` `#[deprecated]` and excluded it from the canonical registry (`error_meta` returns `None`); supersede the catch-all
- T-8B02: Confirmed no live catch-all emission sites existed (EXECUTION_FAILED was defined-only); validation-category codes defined-ahead pending the validator↔runtime code bridge — no production reassignment needed
- T-8T01: `error_registry_lockstep` test — every canonical code (24 language codes + CAPABILITY_UNMET; EXECUTION_FAILED excluded) carries metadata
- T-8T02: `cargo test -p nodus` — 222 passed (was 217; +5), 0 failed; clippy `-D warnings` clean; fmt clean; doc only the pre-existing `test`-fn baseline warning; SDD §6 reference-containment clean (no spec refs leaked into product code)


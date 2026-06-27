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


- T-8A01: Added `ErrorSeverity` (Error/Warn/Info) and `ErrorCategory` (Parse/Runtime/Validation/Routing/Memory/Test/Control/Dialog) enums to `vocab.rs`
- T-8A02: Added the 14 new `error_code` constants (UNDEFINED_CMD, UNDEFINED_MACRO, VALIDATION_FAILED, ESCALATION_FAILED, CONFIDENCE_LOW, KB_UNAVAILABLE, MEMORY_FAILED, TEST_FAILED, SWITCH_NO_MATCH, PAUSED, COUNTER_OVERFLOW, GIT_UNAVAILABLE, DIALOG_TIMEOUT, DIALOG_REJECTED)
- T-8A03: Added the `error_meta(code) -> Option<(ErrorSeverity, ErrorCategory)>` static registry mapping each canonical code to its severity×category
- T-8B01: Marked `EXECUTION_FAILED` `#[deprecated]` and excluded it from the canonical registry (`error_meta` returns `None`); supersede the catch-all
- T-8B02: Confirmed no live catch-all emission sites existed (EXECUTION_FAILED was defined-only); validation-category codes defined-ahead pending the validator↔runtime code bridge — no production reassignment needed
- T-8T01: `error_registry_lockstep` test — every canonical code (24 language codes + CAPABILITY_UNMET; EXECUTION_FAILED excluded) carries metadata
- T-8T02: `cargo test -p nodus` — 222 passed (was 217; +5), 0 failed; clippy `-D warnings` clean; fmt clean; doc only the pre-existing `test`-fn baseline warning; SDD §6 reference-containment clean (no spec refs leaked into product code)

## Phase 9 — Closed Vocabulary Registries (l2-nodus-registries) (2026-06-27)

- T-9A01: Added `KNOWN_FLAGS` (12 analysis extractors), `KNOWN_VALIDATORS` (12 validator names), and `PRIMITIVE_TYPES` (10 field types) closed registries to `vocab.rs`
- T-9A02: Added `Schema::is_known_flag` / `is_known_validator` (matches the pre-colon name, so `len:32` resolves to `len`) / `is_known_type` query methods
- T-9B01: Added advisory validator diagnostics `W011` (unknown `~flag`), `W012` (unknown `^validator`), `W013` (unknown `@in` field type); warnings never set `ValidationReport::has_errors`, so unknown host vocabulary degrades gracefully (NL-1/NL-7/NL-9 strengthening)
- T-9T01: `cargo test -p nodus` — 228 passed (was 222; +6), 0 failed; clippy `-D warnings` clean; fmt clean; doc only the pre-existing `test`-fn baseline; SDD §6 clean; no fixture regressed to an error (registry checks are advisory)


# Project Context

**Generated:** 2026-06-26

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

- T-4T01: `tests/observability.rs` — 3 integration tests: `observer_neutrality` (HO-5), `run_with_audit_api`, `run_with_provider_and_audit_api`; 163 total tests

## Phase 2 — Library Hardening (2026-06-24)

- T-2B03: Added `"RUN"` to `KNOWN_COMMANDS`; bumped `BUILTIN_SCHEMA_VERSION` from `"0.4.5"` to `"0.4.6"`; added `RUNTIME_OWNED_VARIABLES` constant (9 read-only runtime variables); added `Schema::is_runtime_owned()` method
- T-2B01: Implemented E013 (NL-8) validator check — rejects pipeline target that is a runtime-owned variable; uses `RUNTIME_OWNED_VARIABLES` subset rather than full `RESERVED_VARIABLES` to preserve writable reserved vars ($out, $draft, etc.)
- T-2B02: Implemented E014 (NL-10) validator check — rejects forward references; per-step ordered traversal with own-step self-reference allowance; pre-seeds available set from `@in` fields and `RESERVED_VARIABLES`
- T-2A01: Added `crates/nodus/tests/fixtures/conditional.nodus` — ?IF/?ELIF/?ELSE branching with ESCALATE/NOTIFY handlers; confirmed `StubProvider.analyze()` returns `intent` + `sentiment` (not `level`)
- T-2A02: Added `crates/nodus/tests/fixtures/for_loop.nodus` — ~FOR $item IN $in.items with LOG inside body
- T-2A03: Added `crates/nodus/tests/fixtures/parallel_join.nodus` — ~PARALLEL/~JOIN with two concurrent branches (GEN + ANALYZE)
- T-2A04: Added `crates/nodus/tests/fixtures/macro_expand.nodus` — @macro:greet declaration + RUN(@greet) invocation; confirmed `@something` lexes as Identifier (valid RUN argument)
- T-2C01: Cargo.toml audit — 4 workspace-delegated fields (version, edition, license, repository); zero external dependencies; extraction requirements documented
- T-2C02: Intra-workspace import scan — zero matches for `use (crate_core|cronus|codegraph|cli|tui)::` in `crates/nodus/src/`; no blockers for Phase 3
- T-2T01: `cargo test -p nodus` — 142 passed, 0 failed (91 unit + 17 invariant + 34 parity); 16 new tests added this phase
- T-2T02: `cargo clippy -p nodus -- -D warnings` — zero lints


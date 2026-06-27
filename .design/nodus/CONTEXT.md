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

- T-2C02: Intra-workspace import scan — zero matches for `use (crate_core|cronus|codegraph|cli|tui)::` in `crates/nodus/src/`; no blockers for Phase 3
- T-2T01: `cargo test -p nodus` — 142 passed, 0 failed (91 unit + 17 invariant + 34 parity); 16 new tests added this phase
- T-2T02: `cargo clippy -p nodus -- -D warnings` — zero lints

## Phase 7 — Capability Manifest (LP-8) (2026-06-27)

- T-7A01: Added `ExtensionRole` enum (Model/Audit/Storage/Policy/Vocabulary) and `CapabilityManifest` (BTreeSet roles/commands/capabilities, ordered for deterministic diagnostics) to `portability.rs`; empty manifest is satisfied by any host
- T-7A02: Added `HostCapabilities` resolution surface (`provides`/`has_command`/`satisfies`) with `builtin()` = Model + Audit + Vocabulary; constructed explicitly so the same type serves the built-in host and LP-3 substitution tests
- T-7B01: Implemented pure `validate_manifest(manifest, host) -> Vec<Missing>` resolver (no I/O, order-stable) with typed `Missing` (Role/Command/Capability)
- T-7B02: Added `run_with_manifest` + `run_with_manifest_and_audit` gate to `workflows.rs` — runs after lint validation, before executor boot; non-empty missing set → fail-fast `RunResult` (Status::Failed, zero steps, `NODUS:CAPABILITY_UNMET` naming the missing set), executor never invoked so audited variant emits no events; added `NODUS:CAPABILITY_UNMET` to `vocab.rs`; `lib.rs` re-exports
- T-7C01: Implemented `CapabilityManifest::from_workflow` — walks the AST (conditionals/loops/parallel), mapping model commands (GEN/ANALYZE) → Model role and non-builtin commands → Vocabulary role + required command name; `@needs` DSL declaration deferred to parity backlog
- T-7T01: Integration test `manifest_lp3_two_host_substitution` — host with Storage runs to completion; host without it is rejected fail-fast naming Storage
- T-7T02: Integration test `manifest_rejects_before_side_effects` — counting audit sink records zero events on a rejected run; control run proves the sink counts
- T-7T03: `cargo test -p nodus` — 217 passed (was 204; +13), 0 failed; `cargo clippy --all-targets -- -D warnings` — zero lints; `cargo fmt --check` — clean; `cargo doc --no-deps` — only the pre-existing `test`-fn baseline warning
- T-7D01: Authored l2-nodus-portability.md §4.7 (capability manifest Rust design); §3 LP-8 row → Implemented; bumped v1.0.0 → 1.1.0, RFC → Stable; synced INDEX.md (v1.0.13)


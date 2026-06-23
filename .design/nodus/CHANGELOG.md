# Nodus Workspace Changelog

Internal phase journal. Each entry corresponds to a completed phase.

## Phase 1 — Spec Completeness & Vocabulary Alignment (2026-06-23)

- T-1A01: Defined `~PARALLEL` fail-fast error propagation semantics in l1-nodus-language.md §4.4 — first branch error bypasses `~JOIN`, forwards to `@err:`, `NODUS:RULE_VIOLATION` bypasses `@err:` per NL-2
- T-1A02: Defined `RUN(@macro_name)` macro invocation syntax in l1-nodus-language.md §4.3 — meta-command outside schema vocabulary, recognized before schema validation pass
- T-1B01: Cross-checked l2-nodus-runtime.md §4.6 against `vocab.rs::KNOWN_COMMANDS` — 50 commands, exact match; `BUILTIN_SCHEMA_VERSION = "0.4.5"` confirmed; documented `RUN` vocabulary gap (in `TRANSPILER_VERB_MAP` but not in `KNOWN_COMMANDS`)
- T-1B02: Verified all Canonical References in both specs resolve to existing source paths
- T-1C01: Added Document History table to l1-nodus-language.md; bumped spec to v1.0.1
- T-1C02: Added Document History table to l2-nodus-runtime.md; bumped spec to v1.0.1
- T-1T01: `cargo test -p nodus` — 126 passed, 0 failed (83 unit + 17 invariant + 26 parity); mapped all 10 NL invariants to tests; gaps filed: NL-8 (no validator test for reserved variable shadow) and NL-10 (no validator test for forward reference) → Phase 2 fixture corpus

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

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

## Phase 3 — Standalone Extraction (2026-06-24)

- T-3A01: Synced l2-nodus-runtime.md to v1.0.2 — BUILTIN_SCHEMA_VERSION v0.4.6, 51 commands (RUN meta-command added), RUNTIME_OWNED_VARIABLES documented, NL-8→E013 and NL-10→E014 enforcement rows updated; INDEX.md bumped to v1.0.3
- T-3B01: Replaced workspace-delegated fields in `crates/nodus/Cargo.toml` with explicit values (version 0.1.0, edition 2024, license MIT, repository)
- T-3B02: Added crates.io publication metadata (description, homepage, documentation, keywords, categories, readme, `[package.metadata.docs.rs]`); rewrote `README.md` for standalone audience with lifecycle table
- T-3C01: Rewrote `crates/nodus/src/lib.rs` `//!` doc — standalone quick-start doctest, lifecycle table, design note; removed Cronus-internal references
- T-3C02: Fixed broken intra-doc link in `workflows.rs`; removed SDD task-ID references (T-2F01, T-2T01) and internal invariant labels (WFL-8, WFL-9) from `executor.rs` and `workflows.rs`; `cargo doc --no-deps -p nodus` → 0 warnings
- T-3D01: Written `crates/nodus/.github/workflows/ci.yml` for standalone repo (check, test, clippy, fmt, doc steps)
- T-3D02: Written `crates/nodus/EXTRACTION.md` — 7-step human extraction procedure (create repo, copy, commit, tag, publish, update Cronus, archive)
- T-3T01: `cargo test -p nodus` — 143 passed (91 unit + 17 invariant + 34 parity + 1 doctest); 0 failed; 0 regressions
- T-3T02: `cargo doc --no-deps -p nodus` — 0 warnings; `cargo clippy -p nodus -- -D warnings` — 0 lints

## Phase 6 — Testing Implementation (2026-06-24)

- T-6A01: Extended `TestBlock` AST node in `ast.rs` — `input: Vec<(String, String)>`, `expected: Vec<(String, String)>`, `tags: Vec<String>` typed fields; `raw_lines` retained as deprecated backward-compat companion
- T-6A02: Updated `parse_test_block()` in `parser.rs` — key-value parsing into typed fields; E015 emitted on duplicate `@test:` name in same file (NT-9)
- T-6A03: Updated `transpiler.rs` — `to_nodus()` emits `input:`/`expected:`/`tags:` from typed fields; `to_human()` emits readable assertion prose; parity tests pass
- T-6B01: Implemented `evaluate_test_block()` in `workflows.rs` — per-`expected:` entry lookup against `RunResult.vars`; type mismatch = assertion failure; returns `(bool, String)`
- T-6B02: Rewrote `test()` in `workflows.rs` — full NT-1 (block isolation), NT-2 (input override), NT-3 (assertion bind), NT-4 (failure semantics), NT-5 (provider neutrality); empty `expected:` passes trivially on `Status::Ok`
- T-6C01: Extended `Validator` in `validator.rs` — W001 (route uncovered: `ROUTE(wf:x)` with no covering `@test:`) and W002 (no assertions: `@test:` block with empty `expected:`) diagnostics; E015 not double-reported
- T-6C02: Added tag filtering — `test_with_tags()` / `TestOptions { tags }` in `workflows.rs`; NT-6 (skip non-matching blocks); `lib.rs` re-exports updated
- T-6T01: `tests/testing.rs` — 7 integration tests: `block_isolation` (NT-1), `input_override` (NT-2), `expected_assertion_pass` (NT-3), `expected_assertion_fail` (NT-4), `tag_filter_skips_unmatched` (NT-6), `ordered_report` (NT-7), `tag_filter_empty_runs_all`
- T-6T02: Authored `l2-nodus-testing.md` v1.0.0 — NT-1…NT-10 compliance table; TestBlock AST, test()/test_with_tags() signatures, TestOptions/TestReport/TestResult types; E015/W001/W002 diagnostics; registered in INDEX.md
- T-6T03: `cargo test -p nodus` — 204 passed (138 unit + 17 invariant + 4 observability + 34 parity + 3 portability + 7 testing + 1 doctest); `cargo clippy` zero lints; `cargo fmt` clean

## Phase 5 — Portability Implementation (2026-06-24)

- T-5A01: Created `portability.rs` — `SchemaProvider` trait + `BuiltinSchemaProvider` (wraps KNOWN_COMMANDS); `StorageProvider` + `NoopStorageProvider`; `PolicyProvider` + `NoopPolicyProvider` (LP-3 interface stubs)
- T-5A02: `vocab.rs` delta — `Schema::with_provider()` constructor (merges BuiltinSchemaProvider vocabulary with host extensions); `is_host_command()` predicate; `host_commands`/`host_reserved` fields
- T-5B01: `lexer.rs` delta — `new_with_schema()`, `tokenize_str_with_schema()`, `extra_commands` field; schema-aware lexing recognizes host-registered commands
- T-5B02: `parser.rs` delta — `parse_with_schema()` uses extended lexer; host commands parsed identically to built-in commands
- T-5C01: `workflows.rs` delta — `run_with_schema()` + `run_with_schema_and_audit()` public functions; `lib.rs` re-exports for all portability types
- T-5C02: `l2-nodus-runtime.md` synced v1.0.3 → v1.0.4 — portability.rs module added to §4.1; SchemaProvider/StorageProvider/PolicyProvider documented in §4.4; run_with_schema/run_with_schema_and_audit added to §4.5
- T-5T01: `tests/portability.rs` — 3 integration tests: `host_schema_extends_builtin`, `host_schema_unknown_command_not_dispatched`, `noop_storage_and_policy_compile`; 166 total tests (107 unit + 17 invariant + 4 observability + 34 parity + 3 portability + 1 doctest)

## Phase 4 — Observability & Extension Framework (2026-06-24)

- T-4A01: Created `observability.rs` — `AuditProvider` trait (7 methods); `ExecutionEvent` 10-variant enum (StepStart/StepEnd/StepError/ConstraintHit/BranchTaken/LoopIteration/MacroEnter/MacroExit/ModelCall/ModelResponse); `NoopAuditProvider` no-op impl; `RunManifest` + `FieldDescriptor` metadata types
- T-4B01: `executor.rs` hook points — all 10 ExecutionEvent variants emitted at correct lifecycle points; HO-1…HO-6 compliant; no raw user-text in traces
- T-4B02: `run_with_audit()` + `run_with_provider_and_audit()` added to `workflows.rs`; re-exported from `lib.rs`; AuditProvider composable with ModelProvider
- T-4C01: `l2-nodus-runtime.md` synced v1.0.2 → v1.0.3 — §4.1 observability.rs module added; §4.5 run_with_audit/run_with_provider_and_audit documented
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

## Phase 8 — Error Taxonomy (l2-nodus-errors) (2026-06-27)

- T-8A01: Added `ErrorSeverity` (Error/Warn/Info) and `ErrorCategory` (Parse/Runtime/Validation/Routing/Memory/Test/Control/Dialog) enums to `vocab.rs`
- T-8A02: Added the 14 new `error_code` constants (UNDEFINED_CMD, UNDEFINED_MACRO, VALIDATION_FAILED, ESCALATION_FAILED, CONFIDENCE_LOW, KB_UNAVAILABLE, MEMORY_FAILED, TEST_FAILED, SWITCH_NO_MATCH, PAUSED, COUNTER_OVERFLOW, GIT_UNAVAILABLE, DIALOG_TIMEOUT, DIALOG_REJECTED)
- T-8A03: Added the `error_meta(code) -> Option<(ErrorSeverity, ErrorCategory)>` static registry mapping each canonical code to its severity×category
- T-8B01: Marked `EXECUTION_FAILED` `#[deprecated]` and excluded it from the canonical registry (`error_meta` returns `None`); supersede the catch-all
- T-8B02: Confirmed no live catch-all emission sites existed (EXECUTION_FAILED was defined-only); validation-category codes defined-ahead pending the validator↔runtime code bridge — no production reassignment needed
- T-8T01: `error_registry_lockstep` test — every canonical code (24 language codes + CAPABILITY_UNMET; EXECUTION_FAILED excluded) carries metadata
- T-8T02: `cargo test -p nodus` — 222 passed (was 217; +5), 0 failed; clippy `-D warnings` clean; fmt clean; doc only the pre-existing `test`-fn baseline warning; SDD §6 reference-containment clean (no spec refs leaked into product code)

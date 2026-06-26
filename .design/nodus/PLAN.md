# Implementation Plan

**Version:** 1.9.0
**Generated:** 2026-06-24
**Based on:** .design/nodus/INDEX.md v1.0.12
**Status:** Active

## Overview

Strategic plan for maturing nodus from an in-tree vendored crate to an independently extractable, production-ready workflow-language library. The arc runs: spec completeness → library hardening → extraction readiness → observability & extension framework → portability → testing → capability manifest (LP-8). Each new contract spec is implemented, then its L2 spec is synced to the realized Rust shape.

Execution mode: **Sequential** (spec correctness must precede hardening; hardening must precede extraction).

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic language contracts. Must be Stable before Phase 1 begins.*

- [x] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1] — Stable
- [x] **Nodus Portability Contract** ([l1-nodus-portability.md](specifications/l1-nodus-portability.md)) [L1] — host neutrality + extension interface contract; LP-1…LP-7; ModelProvider + AuditProvider + future StorageProvider/PolicyProvider taxonomy; feedback distillation protocol; vocabulary layering model
- [x] **Nodus Observability Contract** ([l1-nodus-observability.md](specifications/l1-nodus-observability.md)) [L1] — execution observability protocol; HO-1…HO-6; AuditProvider role; 10-type event taxonomy (step_start/step_end/step_error/constraint_hit/branch_taken/loop_iteration/macro_enter/macro_exit/model_call/model_response); run manifest; data-safety boundary (no raw user text in traces)

## Phase 1 — Spec Completeness & Vocabulary Alignment

*Close open items in both specs; ensure the vocabulary table and Canonical References are authoritative.*

- [x] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1]
  - ✅ `~PARALLEL` fail-fast error propagation documented (§4.4)
  - ✅ `RUN(@macro_name)` macro invocation syntax documented (§4.3)
  - ✅ Document History added (v1.0.1)
- [x] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - ✅ §4.6 verified: 50 commands match `vocab.rs::KNOWN_COMMANDS`; `BUILTIN_SCHEMA_VERSION` = "0.4.5"
  - ✅ `RUN` meta-command vocabulary gap documented
  - ✅ All Canonical References resolve; Document History added (v1.0.1)

## Phase 2 — Library Hardening ✓

*Build confidence required for safe extraction: golden test corpus, NL-invariant coverage map, public API stability baseline.*

- [x] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - ✅ Normative fixture corpus: `conditional.nodus`, `for_loop.nodus`, `parallel_join.nodus`, `macro_expand.nodus`
  - ✅ E013 (NL-8): validator rejects runtime-owned variable as pipeline target; `RUNTIME_OWNED_VARIABLES` constant added
  - ✅ E014 (NL-10): validator rejects forward references; per-step ordered tracking implemented
  - ✅ `RUN` added to `KNOWN_COMMANDS`; `BUILTIN_SCHEMA_VERSION` bumped to `"0.4.6"`
  - ✅ 142 tests pass (91 unit + 17 invariant + 34 parity); clippy clean
  - ✅ Extraction audit: zero external deps, zero intra-workspace imports; `Cargo.toml` workspace fields documented

## Phase 3 — Standalone Extraction ✓

*Prepare `crates/nodus` for publication as an independent library: sync the spec with Phase 2 implementation, harden the Cargo manifest for crates.io, document the public API, and produce the extraction artifacts (CI workflow, extraction procedure).*

- [x] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - ✅ l2-nodus-runtime.md synced to v1.0.2: BUILTIN_SCHEMA_VERSION v0.4.6, 51 commands, E013/E014, RUNTIME_OWNED_VARIABLES
  - ✅ `Cargo.toml` workspace-delegated fields replaced; crates.io metadata added (description, keywords, categories, readme, homepage, documentation, docs.rs config)
  - ✅ `lib.rs` `//!` doc rewritten standalone; broken intra-doc links and SDD reference leaks removed; 0 `cargo doc` warnings
  - ✅ `crates/nodus/.github/workflows/ci.yml` written (check + test + clippy + fmt + doc)
  - ✅ `crates/nodus/EXTRACTION.md` written (7-step human extraction procedure)
  - ✅ 143 tests pass (91 unit + 17 invariant + 34 parity + 1 doctest); clippy clean

## Phase 4 — Observability & Extension Framework ✓

*Implement the full AuditProvider event taxonomy from `l1-nodus-observability.md` and complete the extension interface framework from `l1-nodus-portability.md`. Raises nodus from "extraction-ready" to "production-observable".*

- [x] **L2 Nodus Observability** ([l2-nodus-observability.md](specifications/l2-nodus-observability.md)) [L2] — full AuditProvider implementation: AuditProvider trait + ExecutionEvent 10-variant enum + NoopAuditProvider + RunManifest + FieldDescriptor; executor hook-point map (all 10 events); run_with_audit + run_with_provider_and_audit public API; 13-test plan; `Implements: l1-nodus-observability.md`
  - ✅ `observability.rs` module created; AuditProvider trait + 10-variant ExecutionEvent + NoopAuditProvider
  - ✅ `executor.rs` wired: StepStart/StepEnd/StepError/ConstraintHit/BranchTaken/LoopIteration/MacroEnter/MacroExit/ModelCall/ModelResponse
  - ✅ `run_with_audit` + `run_with_provider_and_audit` added to `workflows.rs` + re-exported from `lib.rs`
  - ✅ `tests/observability.rs`: observer_neutrality, run_with_audit_api, run_with_provider_and_audit_api
  - ✅ All tests pass; clippy clean; docs zero-warning

- [x] **Nodus Runtime delta** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2] — §4.1 updated (observability.rs added), §4.5 updated (run_with_audit/run_with_provider_and_audit), version bumped 1.0.2 → 1.0.3

## Phase 5 — Portability Implementation

*Implement the `SchemaProvider` vocabulary-extension seam and define the pending `StorageProvider`/`PolicyProvider` trait interfaces in `crates/nodus`. Raises nodus from "observability-capable" to "fully-portable" per the LP-invariants.*

> **Status:** Complete — all tracks A/B/C/T delivered. Atomic tasks in `archives/tasks/phase-5.md`.

- [x] **L2 Nodus Portability** ([l2-nodus-portability.md](specifications/l2-nodus-portability.md)) [L2]
  - ✅ `portability.rs` module: SchemaProvider + BuiltinSchemaProvider, StorageProvider + NoopStorageProvider, PolicyProvider + NoopPolicyProvider
  - ✅ `vocab.rs` delta: `Schema::with_provider()` constructor, `is_host_command()` helper, `host_commands`/`host_reserved` fields
  - ✅ `lexer.rs` delta: schema-aware lexing (`new_with_schema`, `tokenize_str_with_schema`, `extra_commands` field)
  - ✅ `parser.rs` delta: `parse_with_schema()` using extended lexer
  - ✅ `workflows.rs` delta: `run_with_schema` + `run_with_schema_and_audit` public functions
  - ✅ `lib.rs` re-exports for all portability types and new workflow functions
  - ✅ `l2-nodus-runtime.md` spec sync v1.0.3 → v1.0.4
  - ✅ 166 tests pass (107 unit + 17 invariant + 4 observability + 34 parity + 3 portability + 1 doctest); clippy clean; fmt clean; docs zero warnings

## Phase 6 — Testing Implementation ✓

*Implement the full `@test:` block contract from `l1-nodus-testing.md`. Raises nodus from a
stub-level runner to an assertion-evaluating test facility (NT-1…NT-10).*

> **Status:** Complete — all tracks A/B/C/T delivered. Atomic tasks in `archives/tasks/phase-6.md`.

- [x] **L2 Nodus Testing** ([l2-nodus-testing.md](specifications/l2-nodus-testing.md)) [L2]
  - ✅ `ast.rs`: structured `TestBlock` (input/expected/tags/raw_lines typed fields)
  - ✅ `parser.rs`: `parse_test_block()` populates structured fields; E015 on duplicate test name
  - ✅ `transpiler.rs`: round-trip fidelity for structured test blocks (to_nodus + to_human)
  - ✅ `workflows.rs`: `evaluate_test_block()` assertion evaluator; `test()` rewritten for per-block NT-1/NT-2/NT-3/NT-4/NT-5; `test_with_tags()` for NT-6
  - ✅ `validator.rs`: W001 (route uncovered), W002 (no expected:) diagnostics; E015 not double-reported
  - ✅ `tests/testing.rs`: 7 integration tests covering NT-1…NT-7
  - ✅ `l2-nodus-testing.md` v1.0.0 authored; NT-1…NT-10 compliance table; registered in INDEX.md
  - ✅ Quality gates: 204 tests pass (target 175+ met); clippy clean; fmt clean; docs zero new warnings

## Phase 7 — Capability Manifest (LP-8)

*Implement the LP-8 capability manifest + pre-run satisfiability validation (fail-fast) from `l1-nodus-portability.md` §4.6 in `crates/nodus`. A workflow declares the extension-point roles / host commands / named capabilities it needs; the runtime validates that declaration against the active host before the first step runs and rejects fail-fast with the missing-capability set, never starting a partially-capable run. The same manifest is the machine-checkable two-host portability contract (LP-3). Completing this phase restabilizes `l2-nodus-portability` (RFC → Stable, C12.1 Stabilization Exception). Atomic tasks in `tasks/phase-7.md`.*

- [ ] **L2 Nodus Portability** ([l2-nodus-portability.md](specifications/l2-nodus-portability.md)) [L2] — LP-8 implementation: `CapabilityManifest` + `ExtensionRole` + `HostCapabilities` in `portability.rs`; `validate_manifest()` fail-fast gate wired into executor boot before step 1; `NODUS:*` capability-rejection diagnostic; AST-derived required-role manifest; `run_with_manifest` API; LP-3 two-host substitution + pre-run purity tests; §4.7 spec authored and §3 LP-8 row → Implemented; spec v1.0.0 → 1.1.0, RFC → Stable

## Backlog

<!-- Upstream parity gap v0.4.6 → v0.7 (l1-nodus-language §4.6 / l2-nodus-runtime §4.7): control constructs (?SWITCH/~MAP/~RETRY/!HALT/!PAUSE), operators/expressions (MATCHES/?./??/WHERE/FIRST/LAST/string-interpolation), HITL dialog (ASK/CONFIRM), @needs selective schema loading, error taxonomy 11 → 24, @ON(priority=N), closed flag/validator/type registries. Each cluster needs focused L2 implementation-design authoring before it can be decomposed into atomic tasks — not plannable as-is. -->
<!-- StorageProvider/PolicyProvider executor integration deferred pending LP-3 satisfied (interfaces present in portability.rs; hook points + run_with_storage/run_with_policy variants pending the second documented host context). -->
<!-- Future: l2-nodus-transpiler.md — dedicated transpiler L2 spec (currently covered by l2-nodus-runtime.md §4). -->

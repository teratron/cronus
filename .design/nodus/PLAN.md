# Implementation Plan

**Version:** 1.3.0
**Generated:** 2026-06-24
**Based on:** .design/nodus/INDEX.md v1.0.6
**Status:** Active

## Overview

Strategic plan for maturing nodus from an in-tree vendored crate to an independently extractable, production-ready workflow-language library. The plan follows three phases: spec completeness → library hardening → extraction readiness.

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

## Phase 4 — Observability & Extension Framework

*Implement the full AuditProvider event taxonomy from `l1-nodus-observability.md` and complete the extension interface framework from `l1-nodus-portability.md`. Raises nodus from "extraction-ready" to "production-observable".*

> **Status:** Active — `l2-nodus-observability.md` authored (Stable v1.0.0). Atomic tasks decomposed in `tasks/phase-4.md`.

- [x] **L2 Nodus Observability** ([l2-nodus-observability.md](specifications/l2-nodus-observability.md)) [L2] — full AuditProvider implementation: AuditProvider trait + ExecutionEvent 10-variant enum + NoopAuditProvider + RunManifest + FieldDescriptor; executor hook-point map (all 10 events); run_with_audit + run_with_provider_and_audit public API; 13-test plan; `Implements: l1-nodus-observability.md`

  Atomic tasks in [tasks/phase-4.md](tasks/phase-4.md):
  - Track A (observability.rs): T-4A01 (types), T-4A02 (unit tests)
  - Track B (executor.rs delta): T-4B01 (Executor field + lib.rs), T-4B02 (step events), T-4B03 (branch/loop/macro events), T-4B04 (model events + run_complete)
  - Track C (workflows.rs delta): T-4C01 (run_with_audit API)
  - Track T (integration tests + spec sync): T-4T01 (observer_neutrality), T-4T02 (API integration), T-4T03 (runtime spec delta + cargo clean)

- [ ] **Nodus Runtime delta** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2] — update §4.1 module structure (add observability.rs), update §4.5 public API table (add run_with_audit variants), version bump 1.0.2 → 1.0.3; task T-4T03 covers this

## Backlog

<!-- All 4 registered specs are scheduled across Phases 0–4. -->

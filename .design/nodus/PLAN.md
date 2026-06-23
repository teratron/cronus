# Implementation Plan

**Version:** 1.1.0
**Generated:** 2026-06-24
**Based on:** .design/nodus/INDEX.md v1.0.2
**Status:** Active

## Overview

Strategic plan for maturing nodus from an in-tree vendored crate to an independently extractable, production-ready workflow-language library. The plan follows three phases: spec completeness → library hardening → extraction readiness.

Execution mode: **Sequential** (spec correctness must precede hardening; hardening must precede extraction).

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic language contracts. Must be Stable before Phase 1 begins.*

- [x] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1] — Stable

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

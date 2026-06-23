# Implementation Plan

**Version:** 1.0.0
**Generated:** 2026-06-23
**Based on:** .design/nodus/INDEX.md v1.0.1
**Status:** Active

## Overview

Strategic plan for maturing nodus from an in-tree vendored crate to an independently extractable, production-ready workflow-language library. The plan follows three phases: spec completeness → library hardening → extraction readiness.

Execution mode: **Sequential** (spec correctness must precede hardening; hardening must precede extraction).

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic language contracts. Must be Stable before Phase 1 begins.*

- [x] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1] — Stable

## Phase 1 — Spec Completeness & Vocabulary Alignment

*Close open items in both specs; ensure the vocabulary table and Canonical References are authoritative.*

- [ ] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1]
  - Fill TBD: `~PARALLEL` branch error propagation semantics (§4.4)
  - Fill TBD: `@macro` invocation syntax in `@steps` (§4.3 / §4.4)
  - Add Document History table
- [ ] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - Cross-check §4.6 vocabulary table against `crates/nodus/src/vocab.rs::KNOWN_COMMANDS`
  - Verify all Canonical References resolve to existing source paths
  - Add Document History table

## Phase 2 — Library Hardening

*Build confidence required for safe extraction: golden test corpus, NL-invariant coverage map, public API stability baseline.*

- [ ] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - Normative fixture corpus: one `.nodus` file per control-flow construct in `crates/nodus/tests/fixtures/`
  - Coverage map: all NL-1..NL-10 invariants exercised by at least one named test
  - Extraction checklist: `Cargo.toml` standalone readiness, zero internal Cronus imports, declared semver baseline

## Backlog

- Phase 3 — Standalone Extraction (move `crates/nodus` to an independent repository, update Cronus to depend on the published crate)

---
phase: 12
name: "Skill System (Two-Tier Stores & Canonical Stack)"
status: Todo
subsystem: "crates/core (skills module); read-side seam to the workflow-runtime vocabulary"
requires: [1, 2, 5]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 12 Tasks — Skill System (Two-Tier Stores & Canonical Stack)

**Phase:** 12
**Status:** Todo
**Strategic Goal:** The skill extension kind gets its concrete realization: a read-only preset store in the program tier and a mutable store in the state tier, canonical packages with no interpreted scripts, a closed built-in command surface bridged into the workflow runtime, an atomic conversion pipeline for imported packages, prompt synthesis, and the `cronus skill` command group.

## Atomic Checklist

- [ ] [T-12A01] Two-tier skill stores + shadowing precedence
- [ ] [T-12A02] Canonical package model + manifest validation
- [ ] [T-12B01] Built-in command surface (registry + grant checks)
- [ ] [T-12B02] Execution model wiring (instructions / workflow / degraded guard)
- [ ] [T-12C01] Conversion pipeline (verify → classify → retain → transpile → degrade → report)
- [ ] [T-12C02] Prompt synthesis path
- [ ] [T-12D01] `cronus skill` command group (import / create / status)
- [ ] [T-12T01] Validation: invariant compliance + parity sweep

## Detailed Tracking

### [T-12A01] Two-tier skill stores + shadowing precedence

- **Spec:** l2-skill-system.md §4.1
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::store` — precedence tests: workspace > state > preset resolution; program-tier write attempts rejected; override-identical-to-preset yields warning, not error.
- **Handoff:** Package model (T-12A02) reads from resolved store paths.
- **Notes:** Program tier is read-only at runtime; nothing writes under the program root. `.release/program/skills/` and `.release/state/skills/` visualization stubs are repository fixtures, not build artifacts.

### [T-12A02] Canonical package model + manifest validation

- **Spec:** l2-skill-system.md §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::package` — parse/validate tests: SKILL.md frontmatter + extension.json (kind "skill", source, permissions) + optional workflow pair; a package containing a scripts/ directory or unknown executable material fails canonical validation; origin/ contents never classified as executable.
- **Handoff:** Conversion (T-12C01) and synthesis (T-12C02) emit this package shape; execution (T-12B02) consumes it.
- **Notes:** No scripts/ directory exists in canonical form — that absence is a validated property, not a convention.

### [T-12B01] Built-in command surface (registry + grant checks)

- **Spec:** l2-skill-system.md §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::commands` — CommandSpec registry tests: id/category/input_schema/required_grants/surface_version fields present; input validated against schema before dispatch; a call whose manifest lacks a required grant is rejected; surface_version constant only changes with core releases (lockstep test).
- **Handoff:** T-12B02 dispatches through this surface; T-12C01 transpiles against it.
- **Notes:** Commands register into the workflow-runtime vocabulary via its existing schema contract (vocabulary-as-data). No changes to the runtime crate's source are expected; if a genuine runtime gap surfaces, it routes through that crate's own workspace pipeline, not this phase.

### [T-12B02] Execution model wiring (instructions / workflow / degraded guard)

- **Spec:** l2-skill-system.md §4.5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::exec` — activation loads instruction body only; a package with workflow.nd routes through validate → bounded execute → structured result; a `degraded: instruction-only` package never reaches the runtime (guard test); per-call grant check invoked on every operation step.
- **Handoff:** T-12D01 exposes this via verbs; T-12T01 sweeps the full path.
- **Notes:** Outbound network passes the existing egress gate; no new egress path is introduced by skills.

### [T-12C01] Conversion pipeline (verify → classify → retain → transpile → degrade → report)

- **Spec:** l2-skill-system.md §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::convert` — stage tests: missing/failed witness stops the pipeline before any read (default-deny); classification partitions instructions/procedures/scripts/assets; unmapped script degrades to instruction-only with original preserved under origin/; failed validation lands nothing (atomicity test); conversion report persisted with the package.
- **Handoff:** Landed packages carry `source: custom`, `status: discovered` — activation stays behind the standard grant gate.
- **Notes:** Deterministic mapping to the command surface is in scope now; the LLM-assisted transpile assist is a seam (defer model wiring), consistent with the domain-logic-first pattern of Phases 10–11. This is the largest task of the phase — split with `.N` suffixes if it exceeds one work unit.

### [T-12C02] Prompt synthesis path

- **Spec:** l2-skill-system.md §4.4 (Prompt synthesis)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::synthesize` — synthesized package validates against the loaded schema + lints before landing; lands with `source: generated`, `status: discovered`; the open TBD (auto-activate after validation vs explicit review) is resolved conservatively as explicit-review until the spec amends.
- **Handoff:** Review gate shared with curator-distilled skills.
- **Notes:** The authoring model call is a seam; synthesis logic is prompt-construction + validation, testable without a live model.

### [T-12D01] `cronus skill` command group (import / create / status)

- **Spec:** l2-skill-system.md §4.6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-cli skill` — verb parsing + binding tests for `skill import <path>`, `skill create --prompt`, `skill status [<id>]`; library methods are the source of truth; status output includes store origin, degradation flag, pending-review state.
- **Handoff:** TUI mirrors the group as slash commands per the standing parity discipline (existing TUI catalog task pattern).
- **Notes:** Extends the existing `skill` group of the extension registry — no new top-level command group (CLI grammar rule).

### [T-12T01] Validation: invariant compliance + parity sweep

- **Goal:** Verify implementation against every Invariant Compliance row of l2-skill-system.md §3 (EXT-1…EXT-11, STO-1, STO-3).
- **Method:** Integration test in `crates/core/tests/` exercising: preset immutability, override-by-copy, discovered-not-activated ingestion, degraded-never-executed, per-call grant checks, witness-before-read, atomic landing; plus CLI/TUI/library parity matrix for the three verbs. Structural gates: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`, full `cargo test -p cronus`.
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus --test skill_system` green + clippy/fmt clean workspace-wide.

## Phase Notes (Planning Audit)

- **Cascade:** Track A (stores + package) gates B, C, D; T closes. B and C are mutually independent once A lands; D needs B's surface for `status` semantics.
- **Optimism flag:** T-12C01 is the heavyweight (six pipeline stages + atomicity); budget it as the phase's critical path and split before letting it sprawl.
- **Hidden dependency:** the command surface registers into the workflow-runtime vocabulary — read-side contract only; any change to the runtime crate itself is out of phase scope and routes through its own workspace.
- **Security posture:** no interpreted scripts anywhere on the execution path; default-deny on failed witness; grants checked per call, not per process.

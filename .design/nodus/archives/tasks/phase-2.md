---
phase: 2
name: "Library Hardening"
status: Done
subsystem: "crates/nodus"
requires:
  - "Phase 1 — Spec Completeness & Vocabulary Alignment"
provides: []
key_files:
  created:
    - "crates/nodus/tests/fixtures/conditional.nodus"
    - "crates/nodus/tests/fixtures/for_loop.nodus"
    - "crates/nodus/tests/fixtures/parallel_join.nodus"
    - "crates/nodus/tests/fixtures/macro_expand.nodus"
  modified:
    - "crates/nodus/src/vocab.rs"
    - "crates/nodus/src/validator.rs"
    - "crates/nodus/tests/parity.rs"
patterns_established: []
duration_minutes: ~
---

# Phase 2 Tasks — Library Hardening

**Phase:** 2
**Status:** Done
**Strategic Goal:** Build the confidence required for safe extraction: a normative control-flow fixture corpus, closure of NL-8 and NL-10 validator gaps, RUN meta-command vocabulary wiring, and an extraction readiness audit.

## Atomic Checklist

### Track A — Fixture Corpus

- [x] [T-2A01] Add `conditional.nodus` — covers ?IF/?ELIF/?ELSE + !BREAK/!SKIP/!OVERRIDE branch flags
- [x] [T-2A02] Add `for_loop.nodus` — covers ~FOR $var IN $collection
- [x] [T-2A03] Add `parallel_join.nodus` — covers ~PARALLEL blocks and ~JOIN result collection
- [x] [T-2A04] Add `macro_expand.nodus` — covers @macro declaration + RUN(@macro_name) invocation *(depends on T-2B03)*

### Track B — Invariant & Vocabulary Fixes

- [x] [T-2B01] NL-8: validator rejects `→ $reserved_var` (runtime-owned variable as pipeline target) with `Severity::Error` (E013)
- [x] [T-2B02] NL-10: validator rejects forward references — step uses `$x` before any prior step declares `→ $x` (E014)
- [x] [T-2B03] Add `RUN` to `KNOWN_COMMANDS`; bumped `BUILTIN_SCHEMA_VERSION` to `"0.4.6"`; added `RUNTIME_OWNED_VARIABLES`

### Track C — Extraction Readiness Audit

- [x] [T-2C01] Audit `crates/nodus/Cargo.toml` — workspace-delegated fields documented; zero external `[dependencies]` confirmed; extraction requirements recorded
- [x] [T-2C02] Scan `crates/nodus/src/` for intra-workspace imports — zero matches; no blockers for Phase 3

### Track T — Validation

- [x] [T-2T01] Run `cargo test -p nodus` — 142 tests pass (91 unit + 17 invariant + 34 parity); 0 failures
- [x] [T-2T02] Run `cargo clippy -p nodus -- -D warnings` — zero lints

## Detailed Tracking

### [T-2A01] Add `conditional.nodus` fixture

**Track:** A — Fixture Corpus
**File:** `crates/nodus/tests/fixtures/conditional.nodus`

Create a normative `.nodus` workflow that exercises:

- `?IF cond → action` with a `!SKIP` branch flag
- `?ELIF cond → action` (second branch)
- `?ELSE → action` (fallback branch)
- `!OVERRIDE` on one branch to suppress a `!PREF` rule locally
- `!BREAK` inside a loop-adjacent conditional

The workflow should be self-contained (no external schema) and parse + validate without errors.

**Verify:** `cargo test -p nodus -- parity` — new test exercising `conditional.nodus` passes; `validate` reports zero `Error`-severity diagnostics.

### [T-2A02] Add `for-loop.nodus` fixture

**Track:** A — Fixture Corpus
**File:** `crates/nodus/tests/fixtures/for-loop.nodus`

Create a normative `.nodus` workflow that exercises:

- `~FOR $item IN $collection … ~END` over a list declared in `@in`
- At least one step inside the body that uses the loop variable (`$item`)
- A `!SKIP` branch inside the loop body

**Verify:** `cargo test -p nodus -- parity` — new test exercising `for-loop.nodus` passes; execution completes with `Status::Ok`.

### [T-2A03] Add `parallel-join.nodus` fixture

**Track:** A — Fixture Corpus
**File:** `crates/nodus/tests/fixtures/parallel-join.nodus`

Create a normative `.nodus` workflow that exercises:

- `~PARALLEL … ~JOIN → $target` with at least two branches
- Each branch performs a distinct operation and contributes a result to `$target`
- A `@err:` handler to demonstrate fail-fast propagation (see NL-2 / l1 §4.4)

**Verify:** `cargo test -p nodus -- parity` — new test exercising `parallel-join.nodus` passes; `$target` after `~JOIN` contains a non-null value.

### [T-2A04] Add `macro-expand.nodus` fixture *(depends on T-2B03)*

**Track:** A — Fixture Corpus
**File:** `crates/nodus/tests/fixtures/macro-expand.nodus`

Create a normative `.nodus` workflow that exercises:

- `@macro: greet` section declaring a reusable step sequence
- `RUN(@greet)` in `@steps:` to expand the macro
- Standard `+modifier` and `→ $target` decorators on the `RUN` step

Requires `RUN` to be in `KNOWN_COMMANDS` (T-2B03 must be Done before this task runs).

**Verify:** `cargo test -p nodus -- parity` — new test exercising `macro-expand.nodus` passes; `validate` emits no `Error`-severity diagnostics.

### [T-2B01] NL-8 validator enforcement

**Track:** B — Invariant Fixes
**File:** `crates/nodus/src/validator.rs`

Add a validation check: when a step's pipeline target (`→ $name`) matches a name in `vocab::RESERVED_VARIABLES`, emit `Severity::Error` with a new error code (suggest `E013` if available, or extend the existing code series). The check applies to all step types including loop bodies and parallel branches.

**Acceptance criteria:**

- New test `validator::tests::e013_fires_when_pipeline_target_is_reserved` passes.
- `no_e013_when_target_is_user_defined` test passes.
- `cargo clippy -p nodus -- -D warnings` clean.

**Verify:** `cargo test -p nodus -- validator::tests::e013` exits 0 with 1 passed.

### [T-2B02] NL-10 validator enforcement

**Track:** B — Invariant Fixes
**File:** `crates/nodus/src/validator.rs`

Add a forward-reference check: when the validator encounters a variable reference `$x` (as a command argument or step input), verify that `$x` is either declared in `@in` / `@ctx` OR has already been assigned by a `→ $x` on a preceding step (in declaration order, per `Vec<Step>`). A reference before assignment emits `Severity::Error` with a new code (suggest `E014`).

Scope boundary: track pipeline bindings (`→ $x`) only; `@ctx` keys and `@in` fields are pre-loaded before step 1 (per NL-9 / executor boot step 4).

**Acceptance criteria:**

- New test `validator::tests::e014_fires_when_variable_used_before_assignment` passes.
- `no_e014_when_variable_assigned_in_prior_step` passes.
- Existing `pipeline_threads_output_between_steps` executor test continues to pass.

**Verify:** `cargo test -p nodus -- validator::tests::e014` exits 0 with 1 passed.

### [T-2B03] Add `RUN` to `KNOWN_COMMANDS`

**Track:** B — Vocabulary
**Files:** `crates/nodus/src/vocab.rs`, `crates/nodus/src/validator.rs`

1. Add `"RUN"` to `KNOWN_COMMANDS` in `vocab.rs`.
2. In the validator, mark `RUN` as a meta-command: skip the domain-command argument/modifier rules that apply to regular vocabulary commands. `RUN` accepts exactly one argument (the `@macro:` name including the `@` sigil); other decorator rules (`+modifier`, `→ $target`) still apply.
3. Update `TRANSPILER_VERB_MAP` entry `("RUN", "Run macro")` — already present, no change needed.
4. Bump `BUILTIN_SCHEMA_VERSION` from `"0.4.5"` to `"0.4.6"` to signal the vocabulary extension.

**Verify:** `vocab::tests::builtin_schema_lists_known_commands` passes; new test `vocab::tests::run_is_known_command` passes; `BUILTIN_SCHEMA_VERSION` = `"0.4.6"`.

### [T-2C01] Audit `crates/nodus/Cargo.toml`

**Track:** C — Extraction Readiness Audit
**File:** `crates/nodus/Cargo.toml` (read-only audit; document findings in phase-2.md Results)

Read and record:

- Which fields use `*.workspace = true` inheritance (version, edition, license, repository).
- Current workspace version (from root `Cargo.toml`).
- `[dependencies]` — confirm empty.
- `[dev-dependencies]` — if any, note whether they reference workspace-internal crates.
- Any `[features]` that expose or hide functionality relevant to the public API.

Document the standalone extraction requirements in the Results section below.

**Verify:** Findings recorded in `## Results → T-2C01` below; no undocumented external dependencies found.

### [T-2C02] Scan for intra-workspace imports

**Track:** C — Extraction Readiness Audit
**Directory:** `crates/nodus/src/` (read-only)

Search all `.rs` files for:

- `use crate_core::`, `use cronus::`, `use codegraph::`, or any crate name from the workspace `members` list (other than `nodus`).
- `path = "../` or `path = "../../` style Cargo.toml path deps (already checked in T-2C01, but verify in source too).

Expected result: zero intra-workspace references. If any are found, document them and mark as blockers for Phase 3 (Standalone Extraction).

**Verify:** Grep exits with zero matches for `use (crate_core|cronus|codegraph|cli|tui)::` in `crates/nodus/src/**/*.rs`.

### [T-2T01] Run full test suite

**Track:** T — Validation
**Command:** `cargo test -p nodus`

Run after all Track A and Track B tasks are Done. Verify:

1. Test count ≥ 126 (baseline) — new tests from T-2B01, T-2B02, T-2B03, T-2A01–T-2A04 add to the count.
2. 0 failures.
3. New parity tests for each fixture file added in Track A: `parity::execution::conditional_executes_ok`, `parity::execution::for_loop_executes_ok`, `parity::execution::parallel_join_executes_ok`, `parity::execution::macro_expand_executes_ok`.

**Verify:** `cargo test -p nodus` exits 0; output line `N passed; 0 failed` where N ≥ 130.

### [T-2T02] Run clippy

**Track:** T — Validation
**Command:** `cargo clippy -p nodus -- -D warnings`

Run after Track B tasks are Done. Zero lints required.

**Verify:** `cargo clippy -p nodus -- -D warnings` exits 0 with no `warning:` or `error:` output lines attributable to the nodus crate.

## Results

### T-2C01 Cargo.toml extraction audit

**Workspace-delegated fields:** `version`, `edition`, `license`, `repository` — all use `.workspace = true`.

**Current workspace version:** `0.1.0` (from root `Cargo.toml`).

**`[dependencies]`:** Empty. Zero external crate dependencies; the runtime is entirely std-only.

**`[dev-dependencies]`:** Absent from `crates/nodus/Cargo.toml`. Test helpers (`StubProvider`, `Parser`, `Executor`) live in the same crate's `src/` — no cross-crate test dependencies.

**`[features]`:** None declared.

**Standalone extraction requirements:**

1. Provide explicit `version`, `edition`, `license`, `repository` fields (currently delegated to workspace).
2. No `[dependencies]` to carry over — the crate is dependency-free.
3. Move dev tooling (if any shared with workspace root) into a dedicated dev-dependency block.

### T-2C02 Intra-workspace import scan

**Command:** `grep -rn "use crate_core|use cronus|use codegraph|use cli|use tui" crates/nodus/src/`

**Result:** Zero matches. No intra-workspace imports found.

**Verdict:** `crates/nodus` is fully self-contained. All source files under `crates/nodus/src/` reference only the Rust standard library and other modules within `crates/nodus/src/` itself (via `crate::` paths). No blockers for Phase 3 (Standalone Extraction) from the import perspective.

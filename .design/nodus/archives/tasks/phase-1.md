---
phase: 1
name: "Spec Completeness & Vocabulary Alignment"
status: Done
subsystem: ".design/nodus/specifications"
requires: []
provides:
  - "l1-nodus-language.md v1.0.1 — macro invocation syntax (RUN(@macro_name)), ~PARALLEL fail-fast error propagation, Document History"
  - "l2-nodus-runtime.md v1.0.1 — vocabulary alignment (50 commands verified), RUN meta-command gap documented, Document History"
key_files:
  created: []
  modified:
    - ".design/nodus/specifications/l1-nodus-language.md"
    - ".design/nodus/specifications/l2-nodus-runtime.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "Macro invocation via RUN(@macro_name) — meta-command outside schema vocabulary"
  - "~PARALLEL fail-fast: first branch error bypasses ~JOIN and forwards to @err:"
duration_minutes: 30
---

# Phase 1 Tasks — Spec Completeness & Vocabulary Alignment

**Phase:** 1
**Status:** Done
**Strategic Goal:** Fill every TBD marker in the nodus L1 spec, align the L2 vocabulary table with the authoritative source in `vocab.rs`, and verify all Canonical References resolve to existing paths.

## Atomic Checklist

- [x] [T-1A01] Define `~PARALLEL` branch error propagation semantics in l1-nodus-language.md §4.4
- [x] [T-1A02] Define `@macro` invocation syntax in l1-nodus-language.md §4.3/§4.4
- [x] [T-1B01] Cross-check §4.6 command table in l2-nodus-runtime.md against `vocab.rs::KNOWN_COMMANDS`
- [x] [T-1B02] Verify all Canonical References in both specs resolve to existing source files
- [x] [T-1C01] Add Document History table to l1-nodus-language.md
- [x] [T-1C02] Add Document History table to l2-nodus-runtime.md
- [x] [T-1T01] Run `cargo test -p nodus`; map passing tests to NL-1..NL-10 invariants; file gaps

## Detailed Tracking

### [T-1A01] Define `~PARALLEL` branch error propagation semantics

**Track:** A — Spec Authoring
**File:** `.design/nodus/specifications/l1-nodus-language.md`
**Section:** §4.4 Control flow

Open question: when one branch inside `~PARALLEL` fails, does the block fail immediately (fail-fast), continue remaining branches (collect-errors), or collect all results into `~JOIN` regardless? The answer must be stated as a new invariant or a note in §4.4. Consider:

- Fail-fast aligns with `!!` absoluteness (NL-2 style).
- Collect-errors aligns with `@err:` handler semantics (NL-9 style).
- Document which policy is mandatory for conforming runtimes.

**Acceptance:** §4.4 contains a prose statement describing error propagation policy for `~PARALLEL` blocks, and the L2 spec (l2-nodus-runtime.md) confirms it in the invariant compliance table.

### [T-1A02] Define `@macro` invocation syntax

**Track:** A — Spec Authoring
**File:** `.design/nodus/specifications/l1-nodus-language.md`
**Section:** §4.3 Step syntax (or §4.4 as a new subsection)

`@macro:` declarations are listed in §4.2 section declarations, but the syntax for invoking a macro from within `@steps:` is not specified. Options to evaluate:

- `N. CALL(@macro_name args)` — uses a built-in CALL command.
- `N. @macro_name(args)` — sigil-prefixed step syntax.
- `N. EXPAND(@macro_name, args)` — explicit expansion command.

The chosen form must fit the step grammar in §4.3 without ambiguity.

**Acceptance:** §4.3 (or a new §4.4 subsection) shows macro invocation syntax in a `[REFERENCE]` code block; NL-10 sequential pipeline constraint is explicitly evaluated for macro-expanded steps.

### [T-1B01] Cross-check vocabulary table against `vocab.rs`

**Track:** B — Cross-Validation
**File:** `.design/nodus/specifications/l2-nodus-runtime.md`
**Section:** §4.6 Vocabulary schema

Open `crates/nodus/src/vocab.rs` and read `KNOWN_COMMANDS` (or equivalent constant). Compare:

- Command count: spec §4.6 lists 47 commands across 8 categories — verify count matches source.
- Category groupings: verify each command is in the correct category.
- Missing entries: commands in `vocab.rs` not in spec → add to spec.
- Extra entries: commands in spec not in `vocab.rs` → remove from spec.
- `BUILTIN_SCHEMA_VERSION`: confirm the string in spec matches the constant in `vocab.rs`.

**Acceptance:** §4.6 table and `vocab.rs::KNOWN_COMMANDS` are in exact agreement; any delta is corrected in the spec.

### [T-1B02] Verify Canonical References resolve

**Track:** B — Cross-Validation
**Files:** both nodus specs

For each `[ALIAS] | path | purpose` row in both Canonical References tables, confirm the `path` exists in the repository. Use `cargo metadata` or direct filesystem check. Flag any stale paths and either fix them or add a note.

**Acceptance:** Every Canonical Reference path resolves; no stale entries remain.

### [T-1C01] Add Document History table to l1-nodus-language.md

**Track:** C — Documentation
**File:** `.design/nodus/specifications/l1-nodus-language.md`

Add a `## Document History` section at the end of the file with the initial entry:

```markdown
| Version | Date | Change |
| --- | --- | --- |
| 1.0.0 | 2026-06-23 | Initial spec — language invariants, file types, section grammar, step syntax, control flow, error taxonomy |
```

**Acceptance:** Document History section present with correct initial entry; file version remains 1.0.0.

### [T-1C02] Add Document History table to l2-nodus-runtime.md

**Track:** C — Documentation
**File:** `.design/nodus/specifications/l2-nodus-runtime.md`

Add a `## Document History` section with the initial entry:

```markdown
| Version | Date | Change |
| --- | --- | --- |
| 1.0.0 | 2026-06-23 | Initial spec — module structure, AST nodes, Value type, executor boot sequence, public API, vocabulary schema v0.4.5 |
```

**Acceptance:** Document History section present; file version remains 1.0.0.

### [T-1T01] Run cargo test and map to NL invariants

**Track:** T — Validation
**Command:** `cargo test -p nodus`

Run the nodus test suite and:

1. List total passing / failing tests.
2. For each NL-1..NL-10 invariant, identify the test(s) that exercise it.
3. Report any invariant with no covering test as a gap.

Gap report feeds directly into Phase 2 (Library Hardening) test-corpus tasks — any uncovered invariant becomes a fixture-corpus entry.

**Acceptance:** A coverage table mapping each NL-1..NL-10 invariant to one or more test names is produced; gaps are filed as notes in the phase-1.md results section below.

## Results

### T-1B01 Vocabulary delta

`vocab.rs::KNOWN_COMMANDS` = **50 commands**; spec §4.6 = **50 commands** across 8 categories. ✅ No delta.
`BUILTIN_SCHEMA_VERSION` = `"0.4.5"` in both `vocab.rs` and spec. ✅

**Finding:** `TRANSPILER_VERB_MAP` contains `("RUN", "Run macro")` but `RUN` is absent from `KNOWN_COMMANDS`. This is the macro-invocation meta-command: the transpiler was built ahead of the spec. Documented in l2-nodus-runtime.md §4.6 and l1-nodus-language.md §4.3.

### T-1T01 NL invariant coverage

`cargo test -p nodus`: **126 passed, 0 failed** (83 unit + 17 invariant + 26 parity).

| Invariant | Tests |
| --- | --- |
| NL-1 Schema-first | `wfl_2_builtin_schema_is_loaded_and_queryable`, `wfl_2_validator_uses_schema_to_catch_unknown_commands`, `validator::e001_fires_when_runtime_absent`, `vocab::builtin_schema_lists_known_commands` |
| NL-2 Hard constraints absolute | `wfl_3_never_rule_halts_execution_with_failed_status`, `wfl_4_hard_rule_wins_over_preference`, `executor::slice::never_rule_blocks_publish_without_validate` |
| NL-3 Soft preferences advisory | `wfl_4_preference_does_not_halt_execution`, `wfl_4_hard_rule_wins_over_preference` |
| NL-4 Validate-before-run | `wfl_5_block_class_error_prevents_execution`, `wfl_5_valid_workflow_passes_gate_and_executes`, `workflows::run_fails_fast_on_block_class_errors` |
| NL-5 Bounded loops | `wfl_6_bounded_loop_executes_within_limit`, `wfl_6_until_loop_sets_max_reached_flag`, `wfl_6_until_without_max_is_lint_error`, `validator::e010_fires_for_until_missing_max` |
| NL-6 Dual representation | `wfl_1_compact_round_trip_preserves_ast`, `wfl_1_human_form_is_distinct_prose`, `workflows::transpile_compact_roundtrips_ast`, `transpiler::round_trip_ast_equality` |
| NL-7 Closed value types | Compile-time guarantee (closed Rust enum); exercised implicitly by `executor::control_flow::merge_command_combines_maps` |
| NL-8 Reserved namespace | `vocab::builtin_schema_knows_reserved_vars_and_tones` (schema presence only) |
| NL-9 Typed I/O contract | `executor::slice::input_data_seeds_in_variable`, `executor::slice::field_defaults_applied_when_no_input`, `wfl_8_failure_result_has_required_fields`, `wfl_8_success_result_has_required_fields` |
| NL-10 Sequential pipeline | `executor::slice::pipeline_threads_output_between_steps`, `executor::control_flow::break_from_conditional_halts_steps` |

### Gaps (→ Phase 2 fixture corpus)

- **NL-8 gap**: no validator test asserting that `→ $in` (or any reserved variable as pipeline target) emits `Severity::Error`. Schema presence is verified; runtime enforcement is not.
- **NL-10 gap**: no validator test asserting that a forward reference (`$x` used before `→ $x` is assigned) emits `Severity::Error`.
- **NL-7 note**: closed enum is a compile-time invariant in Rust; a runtime test is not strictly possible, but a doc-test demonstrating the exhaustive match would make the guarantee explicit.

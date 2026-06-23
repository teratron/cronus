---
phase: 1
name: "Spec Completeness & Vocabulary Alignment"
status: Todo
subsystem: ".design/nodus/specifications"
requires: []
provides: ["l1-nodus-language.md (complete)", "l2-nodus-runtime.md (aligned)"]
key_files:
  created: []
  modified:
    - ".design/nodus/specifications/l1-nodus-language.md"
    - ".design/nodus/specifications/l2-nodus-runtime.md"
patterns_established: []
duration_minutes: ~60
---

# Phase 1 Tasks — Spec Completeness & Vocabulary Alignment

**Phase:** 1
**Status:** Todo
**Strategic Goal:** Fill every TBD marker in the nodus L1 spec, align the L2 vocabulary table with the authoritative source in `vocab.rs`, and verify all Canonical References resolve to existing paths.

## Atomic Checklist

- [ ] [T-1A01] Define `~PARALLEL` branch error propagation semantics in l1-nodus-language.md §4.4
- [ ] [T-1A02] Define `@macro` invocation syntax in l1-nodus-language.md §4.3/§4.4
- [ ] [T-1B01] Cross-check §4.6 command table in l2-nodus-runtime.md against `vocab.rs::KNOWN_COMMANDS`
- [ ] [T-1B02] Verify all Canonical References in both specs resolve to existing source files
- [ ] [T-1C01] Add Document History table to l1-nodus-language.md
- [ ] [T-1C02] Add Document History table to l2-nodus-runtime.md
- [ ] [T-1T01] Run `cargo test -p nodus`; map passing tests to NL-1..NL-10 invariants; file gaps

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

## Results (fill during execution)

<!-- Agent fills this during /magic.run -->

### T-1B01 Vocabulary delta

<!-- discovered mismatches go here -->

### T-1T01 NL invariant coverage

| Invariant | Tests |
| --- | --- |
| NL-1 Schema-first | |
| NL-2 Hard constraints absolute | |
| NL-3 Soft preferences advisory | |
| NL-4 Validate-before-run | |
| NL-5 Bounded loops | |
| NL-6 Dual representation | |
| NL-7 Closed value types | |
| NL-8 Reserved namespace | |
| NL-9 Typed I/O contract | |
| NL-10 Sequential pipeline | |

### Gaps

<!-- invariants with no covering test -->

---
phase: 33
name: "Exclusive-Binding Duplicate Detection (NL-27, buildable slice)"
status: Todo
subsystem: "crates/nodus/src/validator.rs, crates/nodus/src/ast.rs (read-only), crates/nodus/tests"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes:
---

# Stage 33 Tasks — Exclusive-Binding Duplicate Detection (NL-27, buildable slice)

**Phase:** 33
**Status:** Todo
**Strategic Goal:** NL-27 says a name admitting exactly one holder must reject a second
declaration rather than silently absorb it. Against source, three of its six name classes are
already violated in shipped code. Two of those three are closable with no shape change, because
the AST *retains* both declarations and a validation pass can therefore see the collision — and
one of the two is not a tidiness issue but a confidentiality-flow defect. This phase closes those
two. The third live class and NL-27's stated-displacement half are explicitly out of scope and
stay in the Backlog with their reasons; see the Scope note.

## Scope note (read before starting)

Four facts, all checked against source at planning time.

**1. The `§config` duplicate is a secret-leak path, not an untidiness.** `check_config_values`
(`validator.rs`) iterates `for field in &decl.fields` and every branch ends in
`accepted.push((field.name.clone(), …, field.secret))`. Nothing tests `decl.fields` for unique
names, so a `§config:` file declaring one name twice — once `secret`, once not — produces **two
entries in `accepted` under that name with different secret flags**. The three accessors on
`AcceptedConfig` then disagree with each other: `get` returns the **first** match, `is_secret`
answers `.any()` (true if *either* entry is secret), and `non_secret_fields` filters `!secret` and
therefore **yields the non-secret entry** — which `run_with_config` merges into `$in.config`. The
value the author marked `secret` on one declaration is exposed through the other, while
`is_secret` simultaneously reports the field as secret. No malice is required; ordinary
user-authored input reaches it.

**2. Both target classes are detectable with no shape change, because the AST keeps both
declarations.** `macros: Vec<MacroBlock>` (`ast.rs`) with `MacroBlock.name: String`, and
`ConfigDecl.fields` with `ConfigField.name: String`. This is the property that makes them
buildable and the out-of-scope classes not.

**3. Each has a shipped precedent to copy, so no new mechanism is invented.**
`e015_no_duplicate_test_names` (`validator.rs`) is the exact shape for the macro-name pass —
`Severity::Error`, a workflow-level uniqueness scan over a `Vec` of named blocks. For config,
`check_config_values` already returns `Result<AcceptedConfig, Vec<ConfigViolation>>` with a typed
`ConfigReason` enum, so a new `DuplicateField` variant extends an existing typed channel; the
return type does not change and a violation already blocks by construction.

**4. Severity is settled by the invariant, not by preference.** NL-27 states that a second
declaration of an exclusive name **is a validation error**, not a warning and not a silent
replacement. Both checks are therefore blocking. The exact `E0xx` code for the macro pass is
assigned at implementation time against the validator's existing code set (E001–E019 are in use),
following the convention `l2-nodus-registries.md` §4.3 already states for its own diagnostics.

**What must not happen.** No change to `Parser::parse`'s signature or to any AST type — this
phase adds validation passes over data that already exists. No change to
`check_config_values`'s return type; `DuplicateField` is a new `ConfigReason` variant, nothing
more. No attempt at NL-27's *stated-displacement* half (a declaration naming the holder it
removes) — that needs grammar the language does not have. No touching `Schema::with_provider`
or `error_decl`; both are out of scope for the reasons the Backlog records. LP-1's zero-dependency
constraint is preserved — no new crate.

**Out of scope, and why (the Backlog keeps these).** *A duplicate `@err:` block*: `error_decl` is
an `Option<ErrorDecl>` and the parser assigns `wf.error_decl = Some(…)`, so the first declaration
is **already gone** before any validator runs, and `Parser::parse` returns
`Result<WorkflowFile>` with no advisory channel to report from. Closing it needs either an AST
shape change that retains the second occurrence or a decision to make it a hard parse error —
a design decision, not a mechanical addition. *Host vocabulary collisions*:
`Schema::with_provider` silently filters colliding host names and returns a bare `Schema` with
nowhere to record what it discarded. *NL-28 entirely*: it needs a return-type change on both
validation surfaces, tracked separately.

## Atomic Checklist

- [ ] [T-33A01] `§config` duplicate-field detection — close the secret-leak path before the accept loop
- [ ] [T-33B01] Duplicate macro-name detection — a blocking validator pass modelled on E015
- [ ] [T-33C01] Spec reconciliation — NL-27 verdicts updated from Pending to partially realized in both carriers
- [ ] [T-33T01] Validation — both checks fire, and the secret-leak path is pinned by a regression test

## Detailed Tracking

### [T-33A01] `§config` duplicate-field detection — close the secret-leak path before the accept loop

- **Spec:** l2-nodus-runtime.md §3.1 (NL-27 verdict, the `§config` clause) · l1-nodus-language.md NL-27, NL-20
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. `ConfigReason` carries a new `DuplicateField` variant. `check_config_values` scans
  `decl.fields` for repeated names **before** the accept loop runs and pushes one
  `ConfigViolation { field, reason: DuplicateField }` per repeated name, so no duplicate ever
  reaches `accepted`. A `§config:` declaring one name twice returns `Err`, and the returned
  violation names that field. Return type unchanged
  (`Result<AcceptedConfig, Vec<ConfigViolation>>`).
- **Handoff:** T-33T01 is the acceptance evidence; T-33C01 reconciles the specs once this and
  T-33B01 land.
- **Notes:**
- **Changes:**

### [T-33B01] Duplicate macro-name detection — a blocking validator pass modelled on E015

- **Spec:** l2-nodus-runtime.md §3.1 (NL-27 verdict, the macro clause) · l1-nodus-language.md NL-27
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. A new `e0NN_no_duplicate_macro_names` is registered in `Validator::validate`'s error
  section and emits `Severity::Error` with a code not already present in the validator's code set
  (E001–E019 in use at planning time; confirm before assigning). A workflow declaring two
  `@macro` blocks of one name produces that diagnostic and `has_errors`, so `run()` fast-fails
  before execution per NL-4. A workflow with unique macro names produces none.
- **Handoff:** T-33T01 is the acceptance evidence.
- **Notes:**
- **Changes:**

### [T-33C01] Spec reconciliation — NL-27 verdicts updated from Pending to partially realized in both carriers

- **Spec:** l2-nodus-runtime.md §3.1 · l2-nodus-registries.md §3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `l2-nodus-runtime.md` §3.1's NL-27 row moves from **Pending** to **Partially
  realized**, naming exactly which two of the three live classes are closed and which one is
  not, with the out-of-scope reasons preserved verbatim rather than dropped (the `Option`-loses-
  the-first argument for `@err:`, the nowhere-to-report argument for host vocabulary, and the
  missing grammar for stated displacement). The `l2-nodus-registries.md` NL-27 row is re-read and
  left **Vacuous** if still accurate — this phase does not make its registries host-extensible.
  Both carriers' file headers and their `INDEX.md` rows are updated atomically, and
  `check-prerequisites --verify-headers --workspace=nodus` reports no `VERSION_DRIFT` or
  `STATUS_DRIFT`.
- **Handoff:** Final task before the phase closes; the Backlog entry is re-decided at the next
  `/magic.task nodus`.
- **Notes:**
- **Changes:**

### [T-33T01] Validation — both checks fire, and the secret-leak path is pinned by a regression test

- **Spec:** l2-nodus-runtime.md §3.1 (NL-27 verdict) · l1-nodus-language.md NL-27
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a higher count than the phase's starting
  baseline (record the baseline before writing tests), `cargo fmt --all` clean. Named tests
  exist for each of: (a) the duplicate `§config` field returns `Err` carrying
  `ConfigReason::DuplicateField`; (b) a unique-field `§config` still returns `Ok` unchanged —
  the behaviour-neutrality proof; (c) two `@macro` blocks of one name produce the new
  error-severity diagnostic; (d) unique macro names produce none. Plus the regression test that
  is the point of Track A: a `§config:` declaring one name twice, once `secret` and once not,
  and an assertion that the run is **rejected** — pinning that no `AcceptedConfig` is ever built
  in which `non_secret_fields()` could emit a value another declaration marked secret. Assert on
  the rejection, not on the old two-entry shape, so the test survives any later refactor of the
  accept loop.
- **Handoff:** Acceptance evidence for T-33A01 and T-33B01.
- **Notes:**
- **Changes:**

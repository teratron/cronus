---
phase: 33
name: "Exclusive-Binding Duplicate Detection (NL-27, buildable slice)"
status: Done
subsystem: "crates/nodus/src/validator.rs, crates/nodus/src/ast.rs (read-only), crates/nodus/tests"
requires: []
provides:
  - "ConfigReason::DuplicateField + a pre-accept-loop duplicate-name scan in check_config_values (validator.rs) — closes a real confidentiality path where a §config field declared twice, once secret once not, could leak through non_secret_fields while is_secret reported it secret"
  - "e020_no_duplicate_macro_names validator rule (validator.rs), modelled on e015_no_duplicate_test_names — rejects two @macro blocks of one name"
  - "l2-nodus-runtime.md NL-27 verdict: Pending -> Partially realized (1.5.0 -> 1.5.1), with the three still-open classes' reasons preserved verbatim"
  - "l2-nodus-registries.md NL-27 verdict re-confirmed Vacuous, unaffected by this phase (1.1.0 -> 1.1.1)"
  - "4 new tests (2 unit in validator.rs, 2 integration in tests/config.rs): 490 passing (was 486)"
key_files:
  created: []
  modified:
    - "crates/nodus/src/validator.rs"
    - "crates/nodus/tests/config.rs"
    - ".design/nodus/specifications/l2-nodus-runtime.md"
    - ".design/nodus/specifications/l2-nodus-registries.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "A duplicate-name validation check is added as a pre-accept-loop scan over data the AST already retains (macros: Vec<MacroBlock>, ConfigDecl.fields), not a shape change — the deciding property for whether an NL-27 name class is buildable in one phase is whether the collision is still visible to a validator by the time it runs, not how serious the class is; the classes where the first declaration is already overwritten or discarded before validation (@err:, host vocabulary) need a shape decision instead and were left in the Backlog"
duration_minutes: 35
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

- [x] [T-33A01] `§config` duplicate-field detection — close the secret-leak path before the accept loop
- [x] [T-33B01] Duplicate macro-name detection — a blocking validator pass modelled on E015
- [x] [T-33C01] Spec reconciliation — NL-27 verdicts updated from Pending to partially realized in both carriers
- [x] [T-33T01] Validation — both checks fire, and the secret-leak path is pinned by a regression test

## Detailed Tracking

### [T-33A01] `§config` duplicate-field detection — close the secret-leak path before the accept loop

- **Spec:** l2-nodus-runtime.md §3.1 (NL-27 verdict, the `§config` clause) · l1-nodus-language.md NL-27, NL-20
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. `ConfigReason` carries a new `DuplicateField` variant. `check_config_values` scans
  `decl.fields` for repeated names **before** the accept loop runs and pushes one
  `ConfigViolation { field, reason: DuplicateField }` per repeated name, so no duplicate ever
  reaches `accepted`. A `§config:` declaring one name twice returns `Err`, and the returned
  violation names that field. Return type unchanged
  (`Result<AcceptedConfig, Vec<ConfigViolation>>`).
  **Satisfied:** both commands clean; `shape_check_duplicate_field` (unit) and
  `duplicate_secret_field_rejects_rather_than_leaking` (integration, `tests/config.rs`) both pass.
- **Handoff:** T-33T01 is the acceptance evidence; T-33C01 reconciles the specs once this and
  T-33B01 land.
- **Notes:** Used two `HashSet<&str>` (`seen_names`, `reported_duplicates`) rather than the
  initial `Vec<&str>` draft — `insert` gives O(1) membership and lets the "exactly one violation
  per repeated name, however many times it repeats" rule read as one line:
  `!seen_names.insert(name) && reported_duplicates.insert(name)`. Simplified during the diff
  review pass, before QA, per RULES §2 Clean Code.
- **Changes:** `validator.rs`: `ConfigReason` gained `DuplicateField`;
  `check_config_values` gained a pre-accept-loop duplicate-name scan (2 `HashSet`s, one violation
  per repeated name). `tests/config.rs`: `shape_check_duplicate_field` (unit, direct
  `check_config_values` call) + `duplicate_secret_field_rejects_rather_than_leaking`
  (integration, parses a real `§config:` text and asserts the `run_with_config` outcome is
  `Status::Failed` / `NODUS:CONFIG_INVALID`, zero steps executed).

### [T-33B01] Duplicate macro-name detection — a blocking validator pass modelled on E015

- **Spec:** l2-nodus-runtime.md §3.1 (NL-27 verdict, the macro clause) · l1-nodus-language.md NL-27
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. A new `e0NN_no_duplicate_macro_names` is registered in `Validator::validate`'s error
  section and emits `Severity::Error` with a code not already present in the validator's code set
  (E001–E019 in use at planning time; confirm before assigning). A workflow declaring two
  `@macro` blocks of one name produces that diagnostic and `has_errors`, so `run()` fast-fails
  before execution per NL-4. A workflow with unique macro names produces none.
  **Satisfied:** both commands clean; `e020_fires_on_duplicate_macro_names` and
  `e020_absent_with_unique_macro_names` both pass.
- **Handoff:** T-33T01 is the acceptance evidence.
- **Notes:** `E020` was the next available code (E001–E019 in use). Modelled directly on
  `e015_no_duplicate_test_names` — same `HashSet::insert` shape, same per-extra-occurrence firing
  (2 occurrences → 1 diagnostic, 3 → 2), applied to `wf.macros` instead of `wf.tests`. This differs
  deliberately from T-33A01's "exactly one violation per name" rule: E015 is the named precedent
  for this class and its firing shape was not itself in question. Corrected the module doc
  comment's stale rule count and error-code range (`33` → `34` rules, `E001–E017` → `E001–E020`)
  in the same edit — it already undercounted before this task (E018/E019 existed), and is the
  header for the exact function registration list this task extends.
- **Changes:** `validator.rs`: new `e020_no_duplicate_macro_names` (registered in
  `Validator::validate`), module doc comment corrected. `tests/config.rs`: n/a (macro tests live in
  `validator.rs`'s own `#[cfg(test)] mod tests`, alongside `e015`, per that module's existing
  pattern) — `e020_fires_on_duplicate_macro_names` + `e020_absent_with_unique_macro_names`.

### [T-33C01] Spec reconciliation — NL-27 verdicts updated from Pending to partially realized in both carriers

- **Spec:** l2-nodus-runtime.md §3.1 · l2-nodus-registries.md §3
- **Status:** Done
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
  **Satisfied:** `l2-nodus-runtime.md` NL-27 row now **Partially realized**, naming the two closed
  classes and preserving verbatim why `@err:`, host vocabulary, both vacuous classes, and
  stated-displacement stay open. `l2-nodus-registries.md` re-read and confirmed **Vacuous**
  unchanged — none of its three registries is host-extensible, so Phase 33's closes (elsewhere in
  the crate) do not touch it; added a one-line re-confirmation rather than leaving the prior
  verdict looking unexamined. Both file headers + `INDEX.md` rows updated atomically
  (`l2-nodus-runtime` 1.5.0→1.5.1, `l2-nodus-registries` 1.1.0→1.1.1, `INDEX.md` v1.0.99→v1.0.100).
  `check-prerequisites --verify-headers --workspace=nodus` → `ok: true`, no `VERSION_DRIFT`/
  `STATUS_DRIFT` (sole warning is the expected `SYNC_GAP`, resolved by plan sync at phase close).
- **Handoff:** Final task before the phase closes; the Backlog entry is re-decided at the next
  `/magic.task nodus`.
- **Notes:** `l2-nodus-registries.md` is CRLF-encoded (`l2-nodus-runtime.md` is LF) — edited via a
  script that detects the file's own line ending and writes back with `newline=""` to avoid churn;
  confirmed 122→123 lines, 0 stray LF introduced. `l2-nodus-runtime.md`'s Document History table is
  **append-only at the bottom** (not version-descending — 1.4.x rows sit after 1.0.2, and 1.5.0 was
  already the last row before this edit), a real convention divergence from the main-workspace
  specs' newest-first tables; the new 1.5.1 row was appended after 1.5.0, not prepended, after an
  initial insertion at the top was caught and corrected before this Done mark.
- **Changes:** `l2-nodus-runtime.md` 1.5.0 → 1.5.1: NL-27 row rewritten (Pending → Partially
  realized) + Document History row appended. `l2-nodus-registries.md` 1.1.0 → 1.1.1: NL-27 row
  re-confirmed Vacuous with a cross-reference update + Document History row appended.
  `INDEX.md` v1.0.99 → v1.0.100: both rows synced, Last Updated note added.

### [T-33T01] Validation — both checks fire, and the secret-leak path is pinned by a regression test

- **Spec:** l2-nodus-runtime.md §3.1 (NL-27 verdict) · l1-nodus-language.md NL-27
- **Status:** Done
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
  **Satisfied:** baseline recorded as 486 passing (`cargo test -p nodus`, matching Phase 32's own
  closing count) before any edit. After: 490 passing, 0 failed (+4: 2 lib + 2 integration).
  `cargo fmt -p nodus -- --check` clean. (a)/(c)/(d) are new named tests; (b) is the
  pre-existing `shape_check_accepts_defaults_when_nothing_proposed_for_optional_fields`, unchanged
  and still passing — the behaviour-neutrality proof did not need a new test because nothing about
  the unique-name path changed.
- **Handoff:** Acceptance evidence for T-33A01 and T-33B01.
- **Notes:** Tests were written alongside the A/B implementation rather than after, so the
  Verify commands above ran once covering all four tasks; recorded here as the task whose own
  Verify criterion (the test suite) they satisfy.
- **Changes:** No production code — test-only task. Counted above under T-33A01/T-33B01's own
  `Changes` lines to avoid listing the same four tests twice.
- **Notes:**
- **Changes:**

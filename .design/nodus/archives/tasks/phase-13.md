---
phase: 13
name: "Declarative Configuration Surface"
status: Done
subsystem: "crates/nodus"
requires: [7, 8]
provides:
  - "ConfigDecl/ConfigField/FieldConstraint AST + Parser::parse_config (replaces the §config parser stub)"
  - "Transpiler::config_to_nodus round-trip"
  - "check_config_values pure shape check + AcceptedConfig value model"
  - "ConfigProvider/ConfigOutcome/DefaultConfigProvider + ExtensionRole::Config (LP-8)"
  - "run_with_config / run_with_config_and_audit public API"
  - "NODUS:CONFIG_INVALID error code (validation category)"
key_files:
  created:
    - "crates/nodus/tests/config.rs"
  modified:
    - "crates/nodus/src/ast.rs"
    - "crates/nodus/src/parser.rs"
    - "crates/nodus/src/transpiler.rs"
    - "crates/nodus/src/vocab.rs"
    - "crates/nodus/src/validator.rs"
    - "crates/nodus/src/portability.rs"
    - "crates/nodus/src/workflows.rs"
    - "crates/nodus/src/lib.rs"
patterns_established:
  - "Structural (clause-keyword) field-boundary parsing without indentation tracking, for lexers that emit no indent/dedent tokens"
  - "Secret write-only via merge-omission (never placed in the merged variable surface) rather than a post-hoc redaction filter"
  - "decl-as-parameter shape for run_with_X entry points (caller parses once via Parser::parse_X, mirrors run_with_manifest taking a pre-built CapabilityManifest)"
duration_minutes: ~
---

# Stage 13 Tasks — Declarative Configuration Surface

**Phase:** 13
**Status:** Done
**Strategic Goal:** Realize the `§config` declarative-configuration surface (`l2-nodus-config.md`, NL-20) in `crates/nodus` — replace the `§config` parser stub with a field-declaration AST + parser, a pure pre-run shape check, a provenance- and secret-aware accepted-value model, a host-acceptance provider seam, and the `run_with_config` entry points. All-additive, zero new dependency (LP-1). Sequential tracks A→B→C→T.

## Atomic Checklist

- [x] [T-13A01] Config AST — `ConfigDecl` / `ConfigField` / `FieldConstraint`
- [x] [T-13A02] `Parser::parse_config` + rewire the `§config` deferral branch
- [x] [T-13A03] Transpiler `§config` round-trip
- [x] [T-13B01] `CONFIG_INVALID` error code + registry lockstep
- [x] [T-13B02] Pure `check_config_values` shape check
- [x] [T-13B03] `AcceptedConfig` value model — provenance + write-only secrets
- [x] [T-13C01] `ConfigProvider` + `ExtensionRole::Config` + `builtin()`
- [x] [T-13C02] `run_with_config` / `run_with_config_and_audit` + lib re-exports
- [x] [T-13T01] Validation suite — shape check, secret neutrality, LP-8, zero-dep

## Detailed Tracking

### [T-13A01] Config AST types

- **Spec:** l2-nodus-config.md §4.2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib ast::` → 8 passed (3 new: `config_field_defaults_to_any_and_unrestricted`, `config_decl_preserves_field_order`, `config_field_one_of_constraint`).
- **Handoff:** AST is referenced by every later track — land first.
- **Notes:** Added `ConfigDecl { header, fields }`, `ConfigField { name, type_name, default, constraint, required, secret, describe }`, `FieldConstraint { Range { lo, hi }, OneOf(Vec<String>) }` to `ast.rs`. Kept `default`/`constraint` values as raw `String` (not `executor::Value`) — mirrors `InputField::default`'s existing raw-string precedent and keeps `ast.rs` free of a dependency on `executor.rs`; coercion to a typed `Value` happens at shape-check time (T-13B02).

### [T-13A02] `parse_config` + rewire deferral branch

- **Spec:** l2-nodus-config.md §4.3 (patched to v1.0.1 — see Notes)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib parser::` → 24 passed (8 new config tests, incl. `parse_rejects_config_header_with_redirect` and `parse_schema_header_still_deferred`).
- **Handoff:** feeds T-13A03 (round-trip) and T-13B02 (shape check consumes the decl).
- **Notes:** **[DR] Decision Review (run.md §3.3):** `Parser::parse`/`parse_with_schema` return `Result<WorkflowFile>`, so they cannot literally return a `ConfigDecl` as the spec's §4.3 draft said ("delegate to parse_config"). Two options: (A) add an `Option<ConfigDecl>` field to `WorkflowFile`; (B) keep `parse`/`parse_with_schema` strictly workflow-typed and give the `§config` arm a precise redirect error naming `parse_config`. Chose **(B)** — preserves the one-file-type-per-entry-point symmetry the crate already uses (`run_with_schema` vs `run_with_config` as siblings); (A) would pollute the workflow AST with a foreign concern. Patched `l2-nodus-config.md` §4.3 to match (1.0.0 → 1.0.1, patch, no logic/status change) and bumped its `INDEX.md` registry version to keep header parity. Implemented `Parser::parse_config(source) -> Result<ConfigDecl>` reusing `parse_header(FileType::Config)`. Field-clause grammar: `-` is not a valid identifier character in this lexer, so the enumeration clause lexes as `one_of` (underscore), not the spec prose's `one-of`; field boundaries are disambiguated structurally (a closed clause-keyword set {default, range, one_of, describe} plus bare `required`/`secret` continue the current field; anything else starts a new one) rather than by indentation, since the lexer emits no indent/dedent tokens.

### [T-13A03] Transpiler `§config` round-trip

- **Spec:** l2-nodus-config.md §4.3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib transpiler::` → 15 passed (2 new: `config_round_trips_through_nodus` asserts `parse_config(config_to_nodus(&decl)) == decl` for a 3-field declaration covering `secret`, `required`, `range`, `one_of`, and `describe:`; `config_to_nodus_emits_header`).
- **Handoff:** enables a host editor to serialize an amended declaration back to source.
- **Notes:** Added `Transpiler::config_to_nodus(&ConfigDecl) -> String`. `describe:` values are re-quoted on emit so they re-lex as `StringLit`.

### [T-13B01] `CONFIG_INVALID` error code + lockstep

- **Spec:** l2-nodus-config.md §4.8
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib vocab::` → 19 passed (`error_registry_lockstep` now covers 26 codes; `error_meta_maps_known_codes` asserts `CONFIG_INVALID` → `(Error, Validation)`).
- **Handoff:** the code T-13B02 emits and T-13C02 maps a host `Rejected` to.
- **Notes:** Added the `CONFIG_INVALID` `error_code` constant + `error_meta` row (`Error`, `Validation`) in `vocab.rs`; extended the lockstep test to 26 (24 language codes + `CAPABILITY_UNMET` + `CONFIG_INVALID`).

### [T-13B02] Pure `check_config_values` shape check

- **Spec:** l2-nodus-config.md §4.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` → 41 passed (10 new: one per `ConfigReason` variant + `..._reports_all_violations_applies_none` + `..._is_pure`).
- **Handoff:** produces the `AcceptedConfig` T-13B03 models and T-13C02 boots with.
- **Notes:** `check_config_values(decl, proposed: &[(String, Value)]) -> Result<AcceptedConfig, Vec<ConfigViolation>>` in `validator.rs`. `proposed` values arrive as real typed `Value`s from the caller (no coercion needed); a field's own `default`/`range`/`one_of` bounds are raw `String`s from the AST, coerced to `Value` by declared type via `coerce_config_literal`. Reports ALL violations in one pass, applies NONE. Purity verified directly (same inputs → same `AcceptedConfig` twice).

### [T-13B03] `AcceptedConfig` value model — provenance + write-only secrets

- **Spec:** l2-nodus-config.md §4.5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::accepted_config_non_secret_fields_excludes_secret` → the accepted set's non-secret projection omits `api_key`; `is_secret("api_key")` is `true`.
- **Handoff:** the readable-configuration surface the executor binds in T-13C02.
- **Notes:** **Scope note (documented, not a `[DR]` — no viable alternative within this phase):** the crate's `Value` enum carries no provenance tag anywhere today — NL-11 provenance tracking is a system-wide pending Invariant-Compliance obligation across *every* prior phase (dialog, portability, environment all reference it conceptually without a `Value`-level tag existing), not something introduced or closable by this config feature alone. Implementing real per-value provenance here would mean redefining `Value` crate-wide — far beyond "declarative configuration surface." The achievable, verifiable DC-9 boundary: `AcceptedConfig::get`/`is_secret` make a secret value *usable* (Rust-level accessor), while `AcceptedConfig::non_secret_fields()` is the only projection `run_with_config` merges into the workflow's `$in.config` surface (T-13C02) — so an ordinary `GEN`/`REFINE` step has no path to a secret at all (an omission, not a redaction filter, mirroring NL-11's "remove the capability" discipline at the scope this phase can actually deliver).

### [T-13C01] `ConfigProvider` + `ExtensionRole::Config` + `builtin()`

- **Spec:** l2-nodus-config.md §4.6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib portability::` → 19 passed (4 new: `default_config_provider_accepts_candidate`, `builtin_host_provides_config`, `manifest_requiring_config_is_satisfied_by_builtin`, `manifest_requiring_config_rejected_by_stripped_host`).
- **Handoff:** the acceptance seam T-13C02's entry points call.
- **Notes:** Added `ConfigProvider` trait + `ConfigOutcome { Accepted, Rejected }` + `DefaultConfigProvider` to `portability.rs`, plus `ExtensionRole::Config`; `HostCapabilities::builtin()` now provides it (like `Environment`'s stub, unlike `Dialog`). The `Config` requirement is entry-point-driven — asserted when a run goes through `run_with_config`/`run_with_config_and_audit` (T-13C02) — not derived from AST walking in `from_workflow`, since a `§config` reference has no in-workflow syntax to detect (this mirrors NE-10: calling the config-aware entry point IS the declaration).

### [T-13C02] `run_with_config[_and_audit]` + lib re-exports

- **Spec:** l2-nodus-config.md §4.7
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib workflows::` → 38 passed (3 new: `..._rejects_before_boot_on_missing_required` asserts `Status::Failed` + `CONFIG_INVALID` + zero steps logged; `..._happy_path_merges_non_secret_into_in_config`; `..._excludes_secret_from_merged_input`).
- **Handoff:** completes the public API; feeds the T-13T01 integration suite.
- **Notes:** **[DR]** Signature takes `decl: &ConfigDecl` (parsed once by the caller via `Parser::parse_config`), not a raw `§config` source string — mirrors `run_with_manifest` taking an already-built `CapabilityManifest` rather than re-deriving one from source on every call; avoids juggling two independent parse-error shapes (workflow vs. config) inside one function. Accepted non-secret values merge into `$in.config` under a stable key (mirroring `merge_observation`'s `"observation"` key); a shape-check failure or host `Rejected` outcome returns `Status::Failed` + `NODUS:CONFIG_INVALID` with an empty `log` (zero steps ran) before the executor ever boots. Re-exported `ConfigOutcome`/`ConfigProvider`/`DefaultConfigProvider` (portability), `AcceptedConfig`/`ConfigReason`/`ConfigViolation`/`check_config_values` (validator), `run_with_config`/`run_with_config_and_audit` (workflows) from `lib.rs`.

### [T-13T01] Validation Task — config contract suite

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-config.md` (NL-20 + the secret/provenance boundary + LP-8), and confirm LP-1 zero-dep is preserved.
- **Method:** `tests/config.rs` integration suite (13 tests) covering: NL-20 shape-check coverage (each `ConfigReason`); the secret-neutrality gate; the LP-8 fail-fast path; the full declaration → proposed → acceptance → run happy path; and rejection-emits-no-audit-events (mirrors `tests/portability.rs`'s `CountingAudit` pattern).
- **Status:** Done
- **Verify:** `cargo test -p nodus --test config` → 13 passed. Full-crate `cargo test -p nodus` → **335 passed** (was 292; +43 across ast/parser/transpiler/vocab/validator/portability/workflows/config), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean. `cargo fmt -p nodus -- --check` → clean (after one `cargo fmt` pass to apply canonical style — no logic change). `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; `[dependencies]` section still reads "zero runtime dependencies beyond std" (LP-1 preserved).

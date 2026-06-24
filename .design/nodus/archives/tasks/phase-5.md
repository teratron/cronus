---
phase: 5
name: "Portability Implementation"
status: Done
subsystem: "crates/nodus"
requires:
  - phase-4
provides:
  - SchemaProvider trait (portability.rs)
  - BuiltinSchemaProvider
  - StorageProvider trait + NoopStorageProvider (interface-only)
  - PolicyProvider trait + NoopPolicyProvider (interface-only)
  - Schema::with_provider() constructor
  - run_with_schema / run_with_schema_and_audit public API
  - l2-nodus-runtime.md v1.0.4 (spec sync)
key_files: []
patterns_established: []
duration_minutes: 0
---

# Phase 5 — Portability Implementation

**Status:** Done
**Execution Mode:** Sequential (A → B → C → T)
**Specs:** [l2-nodus-portability.md](../specifications/l2-nodus-portability.md) · [l2-nodus-runtime.md](../specifications/l2-nodus-runtime.md)

## Overview

Wire the portability contract from `l2-nodus-portability.md` into `crates/nodus`.
Track A adds a new `portability.rs` module with all five traits and no-op built-ins.
Track B extends `Schema` with a `with_provider` constructor so hosts can supply custom
vocabularies. Track C adds the `run_with_schema` public API variants. Track T validates
the full suite and syncs `l2-nodus-runtime.md` to v1.0.4.

> **Architecture note**: `StorageProvider` and `PolicyProvider` are specified as trait
> interfaces only. Executor hook points for `STORE`/`LOAD` commands and policy evaluation
> are deferred to a future phase pending LP-3 graduation.

## Track A — `portability.rs` (New module)

### T-5A01: Create `portability.rs` — traits, built-ins, unit tests

- [x] Create `crates/nodus/src/portability.rs`
- [x] Define `SchemaProvider` trait: `fn host_commands(&self) -> &[&str]` + `fn host_reserved_variables(&self) -> &[&str]`
- [x] Implement `BuiltinSchemaProvider`: both methods return `&[]`
- [x] Define `StorageProvider` trait: `fn store(&self, key: &str, value: &crate::executor::Value)` + `fn load(&self, key: &str) -> Option<crate::executor::Value>`
- [x] Implement `NoopStorageProvider`: `store` is a no-op; `load` returns `None`
- [x] Define `PolicyProvider` trait: `fn evaluate(&self, gate: &str, context: &crate::executor::Value) -> bool`
- [x] Implement `NoopPolicyProvider`: `evaluate` returns `true`
- [x] Add `#[cfg(test)] mod tests` block in `portability.rs`
- [x] `builtin_schema_provider_empty_commands` — `BuiltinSchemaProvider::host_commands()` returns `&[]`
- [x] `builtin_schema_provider_empty_reserved` — `BuiltinSchemaProvider::host_reserved_variables()` returns `&[]`
- [x] `noop_storage_load_returns_none` — `NoopStorageProvider::load("any")` returns `None`
- [x] `noop_storage_store_no_panic` — `NoopStorageProvider::store("k", &Value::Null)` does not panic
- [x] `noop_policy_permits_all` — `NoopPolicyProvider::evaluate("any", &Value::Null)` returns `true`

**Verify:** `cargo check -p nodus` passes; `portability.rs` compiles with zero warnings; `cargo clippy -p nodus -- -D warnings` clean.

## Track B — `vocab.rs` delta

### T-5B01: Add `Schema::with_provider()` constructor

- [x] Add `use crate::portability::SchemaProvider;` import to `vocab.rs`
- [x] Add `pub fn with_provider(provider: &dyn SchemaProvider) -> Self` method to `Schema`:
  - Start from `Schema::builtin()` as the base
  - Merge `provider.host_commands()` entries: skip any that already appear in `KNOWN_COMMANDS` (silent dedup)
  - Merge `provider.host_reserved_variables()` entries: skip duplicates from `RESERVED_VARIABLES`
  - Return the extended `Schema` value
- [x] Add `is_host_command(&self, name: &str) -> bool` helper: returns true if `name` is in the host-added command list (used in tests to distinguish builtin from host commands)
- [x] Add unit test `with_provider_extends_schema`:
  - Define an inline struct `TestProvider` implementing `SchemaProvider` with `host_commands = &["MY_CMD"]` and `host_reserved_variables = &["$myvar"]`
  - Assert `schema.is_command("MY_CMD")` is true
  - Assert `schema.is_host_command("MY_CMD")` is true
  - Assert `schema.is_command("GEN")` remains true (builtin not shadowed)
- [x] Add unit test `with_provider_deduplicates_builtins`:
  - `TestProvider` returns `&["GEN"]` (collision with builtin)
  - Assert `Schema::with_provider(&TestProvider)` does not panic; `is_command("GEN")` true; `is_host_command("GEN")` false

**Verify:** `cargo test -p nodus` — all existing tests still pass; new vocab tests pass; `cargo check -p nodus` clean.

## Track C — `workflows.rs` delta

### T-5C01: Add `run_with_schema` + `run_with_schema_and_audit` public functions

- [x] Add `use crate::portability::SchemaProvider;` import to `workflows.rs`
- [x] Add `run_with_schema(source, filename, input?, schema_provider) -> Result<RunResult, Vec<Diagnostic>>`:
  - Build `Schema::with_provider(schema_provider)` before validation
  - Pass the extended schema to `Validator::validate` and `Executor::new`
  - Fast-fail on validation errors (NL-4)
- [x] Add `run_with_schema_and_audit(source, filename, input?, schema_provider, audit, run_id, started_at) -> Result<RunResult, Vec<Diagnostic>>`:
  - Same as above but constructs `Executor::with_audit(StubProvider, audit)`
  - Passes extended schema + audit provider
- [x] Re-export both functions and `SchemaProvider` from `lib.rs` public surface:
  - `pub use portability::{SchemaProvider, BuiltinSchemaProvider, StorageProvider, NoopStorageProvider, PolicyProvider, NoopPolicyProvider};`
  - `pub use workflows::{..., run_with_schema, run_with_schema_and_audit};`

**Verify:** `cargo doc -p nodus --no-deps` — zero warnings; `run_with_schema` and `run_with_schema_and_audit` appear in generated docs with correct signatures; `lib.rs` re-exports compile cleanly.

## Track T — Integration Tests + Spec Sync

### T-5T01: Schema-extension integration test (LP-4 gate)

- [x] Create `crates/nodus/tests/portability.rs`
- [x] `host_schema_extends_builtin`: define a `TestSchemaProvider` with `host_commands = &["CUSTOM_CMD"]`; run a minimal workflow containing `CUSTOM_CMD(...)` via `run_with_schema`; assert `RunResult.status == Status::Ok`
- [x] `host_schema_unknown_command_rejected`: run the same workflow without the schema extension (using plain `run`); assert the result is `Err(diagnostics)` containing an `E002` (unknown command) diagnostic

**Verify:** `cargo test -p nodus --test portability host_schema_extends_builtin` passes; `cargo test -p nodus --test portability host_schema_unknown_command_rejected` passes.

### T-5T02: Noop-provider compilation test

- [x] `noop_storage_and_policy_compile`: confirm `NoopStorageProvider` and `NoopPolicyProvider` implement their respective traits without executor wiring; assert both can be constructed and methods called without panic

**Verify:** `cargo test -p nodus --test portability` — all tests in the file pass.

### T-5T03: Full cargo suite + spec sync

- [x] `cargo test -p nodus` — all tests pass (adds ≥7 new tests on top of Phase 4 baseline)
- [x] `cargo clippy -p nodus --all-targets -- -D warnings` — zero lints
- [x] `cargo fmt -p nodus` — no formatting changes
- [x] `cargo doc -p nodus --no-deps` — zero warnings
- [x] Update [l2-nodus-runtime.md](../specifications/l2-nodus-runtime.md):
  - §4.1 Module structure: add `portability.rs` entry after `observability.rs`
  - §4.5 Public API table: add `run_with_schema` and `run_with_schema_and_audit` rows
  - §4.5 note: update ModelProvider/AuditProvider extension-point paragraph to reference `SchemaProvider`
  - Version: 1.0.3 → 1.0.4
  - Document History: add row for v1.0.4
- [x] Update nodus INDEX.md: l2-nodus-runtime.md version 1.0.3 → 1.0.4

**Verify:** `cargo test -p nodus` passes with the updated runtime; `cargo check -p nodus` clean; INDEX.md version matches file header.

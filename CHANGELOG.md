# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [nodus-0.2.0] — 2026-06-24

### Added (crates/nodus)

- **Observability framework** (`observability.rs`): `AuditProvider` trait (7-method lifecycle), `ExecutionEvent` 10-variant enum (step_start/step_end/step_error/constraint_hit/branch_taken/loop_iteration/macro_enter/macro_exit/model_call/model_response), `NoopAuditProvider`, `RunManifest`, `FieldDescriptor`; executor hook-points wired for all 10 events; HO-1…HO-6 compliant
- **Observability public API**: `run_with_audit()`, `run_with_provider_and_audit()` — composable with `ModelProvider`
- **Portability framework** (`portability.rs`): `SchemaProvider` trait + `BuiltinSchemaProvider`; `StorageProvider` + `NoopStorageProvider`; `PolicyProvider` + `NoopPolicyProvider`; LP-1…LP-7 compliant
- **Portability public API**: `run_with_schema()`, `run_with_schema_and_audit()`; `Schema::with_provider()`, `is_host_command()`; schema-aware lexer (`new_with_schema`) and parser (`parse_with_schema`)
- **Testing framework** (`workflows.rs`): `evaluate_test_block()` assertion evaluator; `test_with_tags()` with `TestOptions { tags }`; NT-1 block isolation, NT-2 input override, NT-3–NT-4 assertion pass/fail, NT-5 provider neutrality, NT-6 tag filtering, NT-7 ordered reporting
- **Validator diagnostics**: W001 (route with no covering `@test:` block), W002 (`@test:` block with no `expected:` entries), E015 (duplicate test name in same file)
- **Testing spec**: `l2-nodus-testing.md` v1.0.0 (Stable) — NT-1…NT-10 compliance table, `TestBlock` AST documentation, `TestReport`/`TestResult` types, diagnostic codes

### Changed (crates/nodus)

- `TestBlock` AST node: `input: Vec<(String, String)>`, `expected: Vec<(String, String)>`, `tags: Vec<String>` typed fields added alongside `raw_lines` backward-compat companion
- `parse_test_block()`: now populates typed fields from `input:`/`expected:`/`tags:` key-value lines
- `test()` function: replaced stub pass-on-Ok with full per-block assertion evaluation
- `l2-nodus-runtime.md` bumped v1.0.2 → v1.0.3 (observability) → v1.0.4 (portability)
- Test count: 126 (Phase 0) → 142 (Phase 2) → 143 (Phase 3) → 163 (Phase 4) → 166 (Phase 5) → **204 (Phase 6)**
- All 8 nodus workspace specs promoted to `Stable`

## [Unreleased]

### Changed

- Added 6 specifications (main)
- Added 7 specifications (main)
- Added 2 specifications (main)
- Updated 3 specifications (main)
- Updated 5 specifications (main)
- Updated 7 specifications (main)
- Updated 9 specifications (main)
- Added 4 specifications (main)
- Added 9 specifications (main)
- Added 17 specifications (main)
- Updated 2 specifications (main)
- Updated task plan and task index (main)
- Updated specification `quality-pipeline` (main)
- Completed task `phase-1` (main)
- Updated 6 specifications (main)
- Updated 10 specifications (main)
- Updated 4 specifications (main)
- Updated specification `memory-store` (main)
- Completed task `phase-2` (main)
- Updated specification `workflow-runtime` (main)
- Completed task `phase-3` (main)
- Updated task index (main)
- Updated global project rules (main)
- Completed task `phase-4` (main)
- Completed 3 tasks (main)
- Completed task `phase-5` (main)
- Completed 5 tasks (main)
- Updated task execution state (main)
- Added specification `harness-engineering` (main)
- Updated 2 specifications (nodus)
- Added 11 specifications (main)
- Added 3 specifications (main)
- Added specification `agent-framework-skeleton` (main)
- Added specification `dynamic-harness` (main)
- Added specification `task-graph-model` (main)
- Added specification `application-shell` (main)
- Added specification `spec-driven-governance` (main)
- Added specification `practice-analytics` (main)
- Updated specification `deep-research` (main)
- Added specification `browser-control` (main)


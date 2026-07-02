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
- **Testing framework spec**: NT-1…NT-10 compliance table, `TestBlock` AST documentation, `TestReport`/`TestResult` types, diagnostic codes

### Changed (crates/nodus)

- `TestBlock` AST node: `input: Vec<(String, String)>`, `expected: Vec<(String, String)>`, `tags: Vec<String>` typed fields added alongside `raw_lines` backward-compat companion
- `parse_test_block()`: now populates typed fields from `input:`/`expected:`/`tags:` key-value lines
- `test()` function: replaced stub pass-on-Ok with full per-block assertion evaluation
- Runtime specification revised for observability, then portability
- Test count: 126 → 142 → 143 → 163 → 166 → **204** over the release cycle
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
- Completed milestone 1 (main)
- Updated 6 specifications (main)
- Updated 10 specifications (main)
- Updated 4 specifications (main)
- Updated specification `memory-store` (main)
- Completed milestone 2 (main)
- Updated specification `workflow-runtime` (main)
- Completed milestone 3 (main)
- Updated task index (main)
- Updated global project rules (main)
- Completed milestone 4 (main)
- Completed 3 tasks (main)
- Completed milestone 5 (main)
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
- Updated specification `voice-input` (main)
- Updated specification `automation-pipeline` (main)
- Updated specification `model-runtime` (main)
- Updated specification `tool-security` (main)
- Added specification `evaluation-suites` (main)
- Added specification `facilitation` (main)
- Updated specification `evaluation-suites` (main)
- Added specification `change-merge` (main)
- Added specification `requirement-checklists` (main)
- Added specification `policy-governance` (main)
- Added specification `execution-sandbox` (main)
- Added specification `context-compression` (main)
- Added specification `intent-resolution` (main)
- Added specification `generation-budget` (main)
- Updated specification `navigation-model` (main)
- Updated task plan and task index (nodus)
- Completed milestone 7 (nodus)
- Added specification `nodus-dialog` (nodus)
- Added specification `nodus-errors` (nodus)
- Completed milestone 8 (nodus)
- Added specification `nodus-registries` (nodus)
- Completed milestone 9 (nodus)
- Completed milestone 10 (nodus)
- Added specification `nodus-control-flow` (nodus)
- Completed milestone 11 (nodus)
- Updated specification `architecture` (main)
- Completed milestone 7 (main)
- Completed milestone 8 (main)
- Added specification `perspective-model` (main)
- Added specification `event-mesh` (main)
- Added specification `claim-verification` (main)
- Added specification `memory-intelligence` (main)
- Updated specification `memory-intelligence` (main)
- Updated specification `nodus-environment` (nodus)
- Added specification `context-provenance` (main)
- Updated specification `nodus-language` (nodus)
- Updated specification `nodus-portability` (nodus)
- Added specification `harness-optimization` (main)
- Updated specification `execution-graph` (main)
- Updated specification `security` (main)
- Added specification `tool-receipts` (main)
- Updated specification `context-compression` (main)
- Updated specification `acp` (main)
- Updated specification `loop-governance` (main)
- Updated specification `orchestration` (main)
- Added specification `work-convergence` (main)
- Added specification `process-monitor` (main)
- Added specification `issue-reporting` (main)
- Added specification `context-attachment` (main)
- Added specification `model-benchmarking` (main)
- Added specification `crash-recovery` (main)

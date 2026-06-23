# Nodus Runtime (Rust)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-language.md

## Overview

Concrete Rust implementation of the Nodus DSL: a self-contained crate (`crates/nodus`) comprising a lexer, parser (→ AST), validator (lint), executor, and transpiler. Vendored in the Cronus monorepo; the crate boundary keeps future extraction to a standalone repository low-cost. The current built-in schema version is **v0.4.5** (`BUILTIN_SCHEMA_VERSION`).

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — Language invariants this crate implements
- [../../main/specifications/l2-workflow-runtime.md](../../main/specifications/l2-workflow-runtime.md) — Cronus integration layer (subsystem dispatch, step binding, platform constraints)

## 1. Motivation

A Rust implementation links in-process with any host (desktop, mobile, server) without a separate language process. Rust's type system enforces the closed value type system (NL-7) at compile time, and `std`-only dependencies keep the crate embeddable on constrained targets. The `ModelProvider` trait decouples the language runtime from any specific AI provider.

## 2. Constraints & Assumptions

- No runtime dependencies beyond `std`; model integration is injected via the `ModelProvider` trait.
- `StubProvider` ships with the crate for tests and early development; real providers are wired at the host layer.
- Schema and grammar are data, not code: `BUILTIN_SCHEMA_VERSION = "0.4.5"` is the baseline; adding a command requires updating `vocab.rs` and bumping the version constant.
- `Map` values use `Vec<(String, Value)>` (not `HashMap`) to preserve declaration order across transpiler round-trips (NL-6).

## 3. Invariant Compliance

| L1 Invariant | Implementation |
| --- | --- |
| NL-1 Schema-first | `Validator::validate` loads `vocab::Schema::builtin()` first; unknown commands emit `Severity::Error` code `E002`; execution is blocked by `run()` fast-fail |
| NL-2 Hard constraints absolute | `Executor` internalizes `!!` rules in boot step 2; violations return `Status::RuleViolation`, bypassing `@err:` entirely |
| NL-3 Soft preferences advisory | `!PREF` rules loaded in boot step 3 as advisory context; stored separately from `!!` rules; `!OVERRIDE` flag suppresses them per-branch |
| NL-4 Validate-before-run | `run()` and `run_with_provider()` call `Validator::validate` first; any `has_errors = true` returns `Err(diagnostics)` before `Executor::execute` is reached |
| NL-5 Bounded loops | Validator checks every `~UNTIL` AST node for a `MAX:n` bound; absence emits `Severity::Error` code `E006` |
| NL-6 Dual representation | `Transpiler::to_nodus` (compact, lossless) and `Transpiler::to_human` (one-way prose); round-trip test in `workflows.rs` verifies AST equality after compact re-parse |
| NL-7 Closed value types | `executor::Value` is a closed Rust enum: `Null / Bool / Int / Float / Text / List / Map` — extension requires a source change and a breaking version bump |
| NL-8 Reserved namespace | `vocab::RESERVED_VARIABLES` slice; validator emits `Severity::Error` if a pipeline target shadows a reserved name |
| NL-9 Typed I/O contract | Required `@in` fields (no `?` suffix, no default) verified by executor before step 1; missing input returns `Status::Error` |
| NL-10 Sequential pipeline | AST `steps` field is `Vec<Step>` in declaration order; validator detects forward-reference targets and emits `Severity::Error` |

## 4. Detailed Design

### 4.1 Module structure

```text
[REFERENCE]
crates/nodus/src/
├── lib.rs           — public re-exports and crate-level doc
├── error.rs         — Error enum, Result alias, Span (line:col)
├── lexer.rs         — tokenizer → Vec<Token>
├── parser.rs        — recursive-descent parser → WorkflowFile AST
├── ast.rs           — all AST node types (FileType, FileHeader, WorkflowFile, Stmt, …)
├── vocab.rs         — Schema, KNOWN_COMMANDS, RESERVED_VARIABLES, VALID_TONES, error_code
├── validator.rs     — Diagnostic, Severity, Validator
├── executor.rs      — Value, Executor, ModelProvider trait, StubProvider, RunResult, Status
├── transpiler.rs    — Transpiler (to_nodus / to_human)
└── workflows.rs     — public library API (scaffold / validate / run / transpile / test / run_with_provider)
```

### 4.2 AST node summary

| Node | Key fields |
| --- | --- |
| `WorkflowFile` | `header`, `runtime`, `triggers`, `rules`, `prefs`, `inputs`, `outputs`, `ctx`, `err`, `steps`, `tests`, `macros` |
| `FileHeader` | `file_type` (Workflow/Schema/Config), `name`, `version` |
| `CommandCall` | `name`, `args`, `modifiers`, `validators`, `flags`, `pipeline_target` |
| `Conditional` | `condition`, `action`, `elif_branches`, `else_branch`, `break_flag`, `skip_flag`, `override_flag` |
| `ForLoop` | `variable`, `collection`, `body` |
| `UntilLoop` | `condition`, `max_iterations`, `body` |
| `ParallelBlock` | `body`, `join_target` |
| `AbsoluteRule` | `kind` (Never/Always), `text` |
| `Preference` | `preferred`, `over`, `condition` |

### 4.3 Value type system

```text
[REFERENCE]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),  // ordered Vec, not HashMap — preserves declaration order
}
```

### 4.4 Executor boot sequence

1. Load schema from `§runtime.core` (validates against `BUILTIN_SCHEMA_VERSION`).
2. Internalize `!!` absolute rules — violations halt immediately with `Status::RuleViolation`.
3. Internalize `!PREF` soft preferences — stored as advisory context.
4. Register `@in` / `@ctx` inputs into the value environment; fail on missing required inputs.
5. Match `@ON` triggers against current input.
6. Execute `@steps` sequentially; thread `→` pipeline targets between steps.

### 4.5 Public library API

| Function | Signature summary | Contract |
| --- | --- | --- |
| `scaffold` | `(name: &str) -> WorkflowFile` | Returns a minimal valid AST (one `GEN` step); always succeeds |
| `validate` | `(source, filename) -> Result<ValidationReport>` | All diagnostics regardless of severity; `has_errors` true on any `Error`-severity finding |
| `run` | `(source, filename, input?) -> Result<RunResult, Vec<Diagnostic>>` | Fast-fail on validation errors (NL-4); uses `StubProvider` |
| `transpile` | `(source, mode: TranspileMode) -> Result<String>` | `Compact` = lossless round-trip; `Human` = one-way prose |
| `test` | `(source, filename) -> Result<TestReport>` | Executes all `@test:` blocks; aggregates `passed`/`failed` counts |
| `run_with_provider` | `(source, filename, input?, provider) -> Result<RunResult, Vec<Diagnostic>>` | Like `run` but with injected `ModelProvider` |

The `ModelProvider` trait is the sole extension point for real model integration. Any host (Cronus core, CLI, test harness) may supply a concrete provider; the crate itself is provider-agnostic.

### 4.6 Vocabulary schema (v0.4.5)

| Category | Commands |
| --- | --- |
| Memory & knowledge | `REMEMBER`, `RECALL`, `FORGET`, `QUERY_KB` |
| Generation | `GEN`, `REFINE`, `TRANSLATE`, `SUMMARIZE`, `FILL`, `GENERATE_DOC` |
| Analysis | `ANALYZE`, `SCORE`, `COMPARE`, `EXTRACT`, `FILTER`, `PARSE`, `PARSE_MD_HEADER`, `PARSE_INDEX` |
| I/O & messaging | `FETCH`, `STORE`, `LOAD`, `APPEND`, `MERGE`, `PUBLISH`, `NOTIFY`, `LOG` |
| Routing & control | `VALIDATE`, `ROUTE`, `ESCALATE`, `WAIT`, `TONE`, `DEBUG` |
| Execution | `EXECUTE`, `SIMULATE`, `EXECUTE_TEST`, `TRANSPILE` |
| Filesystem | `WRITE`, `READ_FILE`, `MKDIR`, `FILE_EXISTS`, `SCAN_DIR`, `MOVE`, `COPY` |
| System & meta | `ENV`, `DATE`, `COUNTER`, `GIT`, `QUERY_GIT`, `HASH`, `VERSION_BUMP` |

Tone values (`+tone=` modifier): `warm` · `neutral` · `formal` · `casual` · `urgent` · `empathetic` · `brand`

Reserved variables: `$in` `$out` `$error` `$meta` `$raw` `$draft` `$ctx` `$user` `$session` `$log` `$flags` `$quality` `$sentiment` `$confidence` `$memory` `$kb_results`

## 5. Drawbacks & Alternatives

- **`StubProvider` gap**: the stub returns placeholder values; real AI generation requires a concrete `ModelProvider` wired at the host layer. Tests that rely on stub output cannot verify generation quality.
- **Schema evolution coupling**: bumping `BUILTIN_SCHEMA_VERSION` requires all callers that hard-code a schema name to update their `§runtime.core` declaration — schema updates are breaking by design to prevent silent vocabulary drift.
- **Alternative — Lua/Wren embed**: rejected; an embedded scripting interpreter adds a non-`std` runtime dependency and cannot enforce `!!` constraints at the language level.
- **Alternative — proc-macro DSL**: rejected; a macro-based DSL would couple the workflow language to the Rust compiler and break the portability goal (NL across host languages).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CRATE]` | `crates/nodus/` | Source of truth for all module-level details |
| `[VOCAB]` | `crates/nodus/src/vocab.rs` | Authoritative command list, `BUILTIN_SCHEMA_VERSION`, reserved variables |
| `[EXECUTOR]` | `crates/nodus/src/executor.rs` | `Value` type definition, executor boot sequence |
| `[WORKFLOWS]` | `crates/nodus/src/workflows.rs` | Public API function signatures and contracts |
| `[AST]` | `crates/nodus/src/ast.rs` | Full AST node type definitions |
| `[FIXTURES]` | `crates/nodus/tests/fixtures/` | Canonical sample workflows (normative test corpus) |

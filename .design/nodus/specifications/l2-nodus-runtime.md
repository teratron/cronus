# Nodus Runtime (Rust)

**Version:** 1.2.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-language.md

## Overview

Concrete Rust implementation of the Nodus DSL: a self-contained crate (`crates/nodus`) comprising a lexer, parser (→ AST), validator (lint), executor, and transpiler. Vendored in the Cronus monorepo; the crate boundary keeps future extraction to a standalone repository low-cost. The current built-in schema version is **v0.4.6** (`BUILTIN_SCHEMA_VERSION`).

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — Language invariants this crate implements
- [../../main/specifications/l2-workflow-runtime.md](../../main/specifications/l2-workflow-runtime.md) — Cronus integration layer (subsystem dispatch, step binding, platform constraints)

## 1. Motivation

A Rust implementation links in-process with any host (desktop, mobile, server) without a separate language process. Rust's type system enforces the closed value type system (NL-7) at compile time, and `std`-only dependencies keep the crate embeddable on constrained targets. The `ModelProvider` trait decouples the language runtime from any specific AI provider.

## 2. Constraints & Assumptions

- No runtime dependencies beyond `std`; model integration is injected via the `ModelProvider` trait.
- `StubProvider` ships with the crate for tests and early development; real providers are wired at the host layer.
- Schema and grammar are data, not code: `BUILTIN_SCHEMA_VERSION = "0.4.6"` is the baseline; adding a command requires updating `vocab.rs` and bumping the version constant.
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
| NL-8 Reserved namespace | `vocab::RUNTIME_OWNED_VARIABLES` (9-element subset of `RESERVED_VARIABLES`); validator emits `Severity::Error` code `E013` if a pipeline target (`→ $name`) names a runtime-owned variable |
| NL-9 Typed I/O contract | Required `@in` fields (no `?` suffix, no default) verified by executor before step 1; missing input returns `Status::Error` |
| NL-10 Sequential pipeline | AST `steps` field is `Vec<Step>` in declaration order; validator emits `Severity::Error` code `E014` when a variable is referenced before any prior step declares it via `→ $name` |

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
├── vocab.rs         — Schema, KNOWN_COMMANDS, RESERVED_VARIABLES, RUNTIME_OWNED_VARIABLES, VALID_TONES, error_code
├── validator.rs     — Diagnostic, Severity, Validator
├── executor.rs      — Value, Executor, ModelProvider trait, StubProvider, RunResult, Status
├── observability.rs — AuditProvider trait, ExecutionEvent (10 variants), NoopAuditProvider, RunManifest, FieldDescriptor, RunStatus, LoopType
├── portability.rs   — SchemaProvider trait, BuiltinSchemaProvider; StorageProvider + NoopStorageProvider (pending LP-3); PolicyProvider + NoopPolicyProvider (pending LP-3)
├── transpiler.rs    — Transpiler (to_nodus / to_human)
└── workflows.rs     — public library API (scaffold / validate / run / transpile / test / run_with_provider / run_with_audit / run_with_provider_and_audit / run_with_schema / run_with_schema_and_audit)
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

<!-- [ADDED] v1.2.0 -->
#### `~PARALLEL` branch execution

The language defines `~PARALLEL` as a scheduling hint; this crate upgrades it to real concurrency when the injected providers allow it, staying `std`-only:

- **Eligibility**: branches run concurrently only when every injected provider used inside the block is thread-shareable (`Send + Sync`). Otherwise the executor falls back to sequential branch execution — permitted by the language contract and behaviorally identical under the fail-fast rule.
- **Mechanism**: scoped OS threads from the standard library (`std::thread::scope`) — no new runtime dependency, no unbounded detach. Width is bounded by `max_parallel_branches` (default: branch count capped at 4); excess branches queue.
- **Isolation & join**: each branch executes over an isolated snapshot of the value environment; writes merge at `~JOIN` in **declared branch order**, so `$target` is deterministic regardless of completion order.
- **Fail-fast**: the first branch error puts the block in fail-fast state per the language semantics — `~JOIN` is bypassed and the error routes to `@err:`. Already-running sibling branches are not force-killed (no mid-step cancellation); they run to completion and their results are discarded. Not-yet-started branches never start.
- **Audit ordering**: events emitted from concurrent branches carry the `(correlation_id, seq)` ordering fields; per-branch event sequences remain internally ordered, and interleaving across branches is resolved by the observability contract, not by wall-clock arrival.

### 4.5 Public library API

| Function | Signature summary | Contract |
| --- | --- | --- |
| `scaffold` | `(name: &str) -> WorkflowFile` | Returns a minimal valid AST (one `GEN` step); always succeeds |
| `validate` | `(source, filename) -> Result<ValidationReport>` | All diagnostics regardless of severity; `has_errors` true on any `Error`-severity finding |
| `run` | `(source, filename, input?) -> Result<RunResult, Vec<Diagnostic>>` | Fast-fail on validation errors (NL-4); uses `StubProvider` |
| `transpile` | `(source, mode: TranspileMode) -> Result<String>` | `Compact` = lossless round-trip; `Human` = one-way prose |
| `test` | `(source, filename) -> Result<TestReport>` | Executes all `@test:` blocks; aggregates `passed`/`failed` counts |
| `run_with_provider` | `(source, filename, input?, provider) -> Result<RunResult, Vec<Diagnostic>>` | Like `run` but with injected `ModelProvider` |
| `run_with_audit` | `(source, filename, input?, audit, run_id, started_at) -> Result<RunResult, Vec<Diagnostic>>` | Like `run` but with injected `AuditProvider`; forwards run metadata to the manifest |
| `run_with_provider_and_audit` | `(source, filename, input?, provider, audit, run_id, started_at) -> Result<RunResult, Vec<Diagnostic>>` | Full injection: custom `ModelProvider` + `AuditProvider` |
| `run_with_schema` | `(source, filename, input?, schema_provider) -> Result<RunResult, Vec<Diagnostic>>` | Extends the builtin vocabulary from `schema_provider` before parse; host-declared commands lex and execute without error |
| `run_with_schema_and_audit` | `(source, filename, input?, schema_provider, audit, run_id, started_at) -> Result<RunResult, Vec<Diagnostic>>` | Schema extension + `AuditProvider`; run metadata forwarded to the manifest |

The `ModelProvider`, `AuditProvider`, and `SchemaProvider` traits are the three active extension points. Any host may supply concrete implementations; the crate is provider-agnostic. `StubProvider`, `NoopAuditProvider`, and `BuiltinSchemaProvider` are the zero-cost built-ins. `StorageProvider` and `PolicyProvider` are specified in `portability.rs` but executor integration is deferred pending LP-3 graduation.

### 4.6 Vocabulary schema (v0.4.6)

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
| Meta-command | `RUN` |

Tone values (`+tone=` modifier): `warm` · `neutral` · `formal` · `casual` · `urgent` · `empathetic` · `brand`

Reserved variables: `$in` `$out` `$error` `$meta` `$raw` `$draft` `$ctx` `$user` `$session` `$log` `$flags` `$quality` `$sentiment` `$confidence` `$memory` `$kb_results`

**Runtime-owned variables** (`RUNTIME_OWNED_VARIABLES` — read-only subset of reserved): `$in` `$error` `$meta` `$ctx` `$user` `$session` `$flags` `$memory` `$kb_results` — pipeline targets (`→ $name`) must not name these variables; violation emits E013.

**Vocabulary alignment (v0.4.6):** `KNOWN_COMMANDS` contains **51** commands across the 9 categories above. `BUILTIN_SCHEMA_VERSION = "0.4.6"` matches the spec. `RUN` is a meta-command — it bypasses domain-command argument and modifier checks; only decorator rules (`+modifier`, `→ $target`) apply.

### 4.7 Upstream parity gaps (v0.4.6 → v0.7)

<!-- [ADDED] v1.1.0 -->

`BUILTIN_SCHEMA_VERSION` is **v0.4.6**; the upstream schema is **v0.7**, so the crate implements the v0.4 generation. The language-design view of these gaps is in `l1-nodus-language.md` §4.6; this section records the concrete implementation work — each item is absent from today's `lexer` / `parser` / `ast` / `vocab` / `executor` / `validator`.

#### (a) Vocabulary — commands

`KNOWN_COMMANDS` (51) omits the v0.6 human-interaction commands `ASK` and `CONFIRM`. Add both with their dialog modifiers (`+type`, `+options`, `+hint`, `+default`, `+validate`, `+timeout`, `+strict`, `+actions`), bound to the host HITL surface at the integration layer.

#### (b) Control constructs (`lexer` + `ast` + `executor`)

`?SWITCH` (+ `*` wildcard, `SWITCH_NO_MATCH`), `~MAP` (implicit `$it`), `~RETRY:n` (step modifier with `+backoff` / `+retry_on`), `!HALT` (→ `Status::Failed`), `!PAUSE` (→ new `Status::Paused`). None are tokenized or parsed today — `~MAP` / `~RETRY` currently mis-lex as a `Flag`. Semantics: `l1-nodus-language.md` §4.6.

#### (c) Operators & expressions (`lexer`)

`MATCHES` regex operator, `?.` optional chaining, `??` null-coalescing, `WHERE` / `FIRST` / `LAST` collection expressions, and string interpolation in literals (with `\$` escape) are not recognized as distinct tokens.

#### (d) `@needs:` selective schema loading

Add a `needs` field to the runtime-block AST plus load-time section filtering against `extends:` schemas.

#### (e) Error-code registry (`vocab::error_code`: 11 → 24)

Add `UNDEFINED_CMD`, `UNDEFINED_MACRO`, `VALIDATION_FAILED`, `ESCALATION_FAILED`, `CONFIDENCE_LOW`, `KB_UNAVAILABLE`, `MEMORY_FAILED`, `TEST_FAILED`, `SWITCH_NO_MATCH`, `PAUSED`, `COUNTER_OVERFLOW`, `GIT_UNAVAILABLE`, `DIALOG_TIMEOUT`, `DIALOG_REJECTED`, each with a severity and category. `NODUS:EXECUTION_FAILED` is non-canonical and is superseded.

#### (f) Closed vocabulary registries (new `vocab` data + validation)

The crate treats `~flag` and `^validator` tokens as free-form. Add closed registries and validate `~flag` / `^validator` / `@in` field types against them:

- **Analysis flags**: `sentiment`, `intent`, `entities`, `topics`, `lang`, `toxicity`, `urgency`, `formality`, `clarity`, `relevance`, `pii`, `keywords`.
- **Validators**: `len:n`, `min_len:n`, `no_pii`, `no_toxic`, `lang:code`, `format:type`, `required:keys`, `sentiment:op:n`, `confidence:n`, `no_links`, `brand_voice`, `approved`.
- **Primitive types**: `str`, `int`, `float`, `bool`, `list`, `obj`, `url`, `ts`, `null`, `any`.

#### (g) Macro execution

`@macro:` bodies are parsed (`MacroBlock`) and audited (`MacroEnter` / `MacroExit`) but **not executed**. `RUN(@macro:name)` must expand and run the body — caller params bind as `$in`, the last assigned variable is the implicit return.

#### (h) `Status` enum

Add `Paused` to the `Ok / Partial / Failed / Aborted` set (for `!PAUSE`).

#### (i) Minor lexer parity

Single-character `;` inline comments (only `;;` is recognized) and the `\$` interpolation escape.

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

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.2.0 | 2026-07-04 | §4.4: real `~PARALLEL` branch execution via `std::thread::scope` when providers are `Send + Sync` (sequential fallback otherwise); bounded width (`max_parallel_branches`, default cap 4); isolated branch environments with deterministic declared-order `~JOIN`; fail-fast without mid-step cancellation; audit interleaving resolved by `(correlation_id, seq)` |
| 1.1.0 | 2026-06-25 | Added §4.7 upstream parity gaps (v0.4.6 → v0.7): missing `ASK`/`CONFIRM` commands, control constructs (`?SWITCH`/`~MAP`/`~RETRY`/`!HALT`/`!PAUSE`), operators/expressions, `@needs:`, `error_code` 11 → 24, closed flag/validator/type registries, macro execution, `Status::Paused`, lexer parity items |
| 1.0.4 | 2026-06-24 | §4.1: add `portability.rs` module; §4.5: add `run_with_schema` and `run_with_schema_and_audit` rows; update extension-point note to reference `SchemaProvider` |
| 1.0.3 | 2026-06-24 | §4.1: add `observability.rs` module; §4.5: add `run_with_audit` and `run_with_provider_and_audit` rows; note `AuditProvider` as second extension point |
| 1.0.2 | 2026-06-24 | §4.6: BUILTIN_SCHEMA_VERSION v0.4.6, 51 commands (RUN added as meta-command), RUNTIME_OWNED_VARIABLES constant documented; §3: NL-8→E013, NL-10→E014 enforced |
| 1.0.1 | 2026-06-23 | §4.6: documented vocabulary alignment (50 commands verified against `vocab.rs`), added `RUN` meta-command gap note |
| 1.0.0 | 2026-06-23 | Initial spec — module structure, AST nodes, Value type, executor boot sequence, public API, vocabulary schema v0.4.5 |

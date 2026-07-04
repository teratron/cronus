# Nodus DSL Testing — Rust Implementation

**Version:** 1.1.0
**Status:** Stable
**Layer:** implementation
**Implements:** [l1-nodus-testing.md](l1-nodus-testing.md)

## Overview

This document specifies the Rust implementation of the nodus `@test:` block testing contract
defined in `l1-nodus-testing.md`. It covers the concrete types in `crates/nodus`, the
execution model wired in `workflows.rs`, the assertion evaluator, and the validator
diagnostics for test-related issues.

## Related Specifications

- [l1-nodus-testing.md](l1-nodus-testing.md) — parent contract; defines NT-1…NT-10 invariants
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust runtime module structure and `Executor`
- [l1-nodus-language.md](l1-nodus-language.md) — `@test:` block as a language-level declaration

## 1. Module Map

| Module | Role |
| --- | --- |
| `crates/nodus/src/ast.rs` | `TestBlock` AST node — structured fields after parsing |
| `crates/nodus/src/parser.rs` | `parse_test_block()` + `parse_test_body()` |
| `crates/nodus/src/workflows.rs` | `test()`, `test_with_tags()`, assertion evaluator, `TestReport`/`TestResult` |
| `crates/nodus/src/validator.rs` | `E015` duplicate-name check; `W009` no-expected advisory |
| `crates/nodus/src/executor.rs` | `RunResult.vars` — final variable environment exposed for assertions |

## 2. TestBlock AST Node

```rust
pub struct TestBlock {
    pub name: String,
    pub input: Vec<(String, String)>,    // field_name → raw_value_string
    pub expected: Vec<(String, String)>, // variable_name → raw_expected_string
    pub tags: Vec<String>,               // tag identifiers
    pub raw_lines: Vec<String>,          // backward-compat for transpiler
}
```

`raw_lines` is retained for the transpiler's round-trip path. For old-style inline bodies
(`input: { key: "val" }`), the structured fields are empty and `raw_lines` holds the tokens.
New-style bodies use the structured fields.

## 3. RunResult Variable Environment

`executor::RunResult` exposes the full post-execution variable environment:

```rust
pub struct RunResult {
    pub workflow: String,
    pub status: Status,
    pub out: Value,
    pub log: Vec<LogEntry>,
    pub errors: Vec<RuntimeError>,
    pub flags: Vec<String>,
    pub vars: HashMap<String, Value>, // key without '$': "out", "confidence", etc.
}
```

`vars` is populated from `ExecutionContext.variables` after the last step executes.
It is the authoritative source for `expected:` assertions (NT-3).

## 4. Public API

```rust
// Run all @test: blocks in declaration order. Returns an empty report for
// workflows with no test blocks (not an error).
pub fn test(source: &str, filename: &str) -> Result<TestReport, Error>

// Like test(), but only runs blocks whose tags list intersects tag_filter.
// If tag_filter is empty, all blocks run (NT-6).
pub fn test_with_tags(source: &str, tag_filter: &[&str]) -> Result<TestReport, Error>
```

Both functions are re-exported from the crate root.

### TestReport / TestResult

```rust
pub struct TestReport {
    pub results: Vec<TestResult>, // declaration order (NT-7)
    pub passed: usize,
    pub failed: usize,
}

pub struct TestResult {
    pub name: String,  // @test: block name
    pub passed: bool,
    pub message: String, // "ok" or first-failing-assertion description
}
```

`TestReport.passed + TestReport.failed == TestReport.results.len()` is a structural
invariant enforced by `TestReport::from_results`.

## 5. Execution Protocol (NT-1…NT-5)

`test_with_tags` iterates over parsed `WorkflowFile.tests` in declaration order. Per block:

1. **Build input (NT-2)**: `build_test_input(ast, &tb.input)` seeds the `@in:` declared defaults,
   then overlays the block's `input:` key-value pairs. Keys absent from `@in:` are silently
   ignored. Returns `Value::Map(...)` passed to `Executor::with_stub().execute()`.

2. **Fresh executor (NT-1 / NT-5)**: `Executor::with_stub()` creates a new executor with a
   `StubProvider` instance — no state from prior blocks, no real I/O or network access.

3. **Execute**: `executor.execute(ast, Some(input))` runs the full `@steps:` body and returns
   `RunResult` including `vars` (NT-8: same schema as production runs).

4. **Evaluate (NT-3 / NT-4)**: `evaluate_test_block(&run_result.vars, &run_result.status, &tb.expected)`.

<!-- [ADDED] v1.1.0 -->
**Parallel-safe stub (NT-5 extension).** `StubProvider` is stateless and input-keyed (`Send + Sync`), which makes it the parallel-safe stub the language contract permits hosts to provide: a `@test:` block whose body contains `~PARALLEL` exercises the executor's real concurrent branch scheduling (see `l2-nodus-runtime.md §4.4`) instead of the sequential fallback. Assertions stay deterministic — stub responses depend only on inputs, and `~JOIN` merges in declared branch order, so `expected:` values are independent of interleaving. Block-level isolation is unchanged: one fresh executor per block, blocks themselves run in declaration order.

## 6. Assertion Evaluator

```rust
fn evaluate_test_block(
    vars: &HashMap<String, Value>,
    status: &Status,
    expected: &[(String, String)],
) -> (bool, String)
```

Semantics:

- Empty `expected`: passes iff `status == Status::Ok`; fails with `"execution failed with status …"` otherwise.
- Non-empty `expected`: first checks `Status` is `Ok` or `Partial`; then for each `(var, val)` pair:
  - Strip leading `$` from `var` to get the key in `vars`.
  - If key absent from `vars` → fail with `"… is not in the execution context"` (NT-3).
  - `parse_expected_value(val)` parses the raw string into `Value`.
  - Compare with `PartialEq` on `Value` (structural recursive equality per NL-7 / §4.3).
  - If mismatch → fail with `"assertion failed: $var expected … got …"`.
  - Continue to next assertion only if current passes.
- Returns `(true, "ok")` when all assertions pass.

### Value Parsing

`parse_expected_value(s: &str) -> Value`:

| Input | Parsed as |
| --- | --- |
| `null` | `Value::Null` |
| `true` / `false` | `Value::Bool` |
| Valid `i64` | `Value::Int` |
| Valid `f64` | `Value::Float` |
| `"quoted"` or bare text | `Value::Text` (quotes stripped) |

The lexer already strips quotes from `StringLit` tokens, so the raw string arriving in
`TestBlock.expected` is unquoted. Bare words that are not numbers or booleans become `Text`.

## 7. Validator Diagnostics

| Code | Severity | Trigger | Rust location |
| --- | --- | --- | --- |
| `E015` | Error | Two `@test:` blocks share the same name within a file | `Validator::e015_no_duplicate_test_names` |
| `W006` | Warning | `ROUTE(wf:x)` step with no `@test:` block covering it (NT-10) | `Validator::w006_route_test_coverage` (pre-existing) |
| `W009` | Warning | `@test:` block with no `expected:` section (passes trivially on Status::Ok) | `Validator::w009_test_no_expected` |

`E015` is a block-class error — workflows with duplicate test names fail the validate-before-run
gate and cannot execute.

## 8. NT-1…NT-10 Compliance Table

| Invariant | Status | Implementation |
| --- | --- | --- |
| NT-1 Block isolation | **Implemented** | Fresh `Executor::with_stub()` per block in `run_test_block` |
| NT-2 Input override | **Implemented** | `build_test_input` overlays block `input:` over `@in:` defaults |
| NT-3 Expected assertion binding | **Implemented** | `evaluate_test_block` checks `vars` by key; absent variable = fail |
| NT-4 Assertion failure semantics | **Implemented** | Blocks continue regardless; first-failing-assertion message |
| NT-5 Provider neutrality | **Implemented** | `StubProvider` per block; no real I/O |
| NT-6 Tag metadata | **Implemented** | `test_with_tags` filters by tag intersection; skipped blocks absent from report |
| NT-7 Ordered reporting | **Implemented** | Iterator preserves `WorkflowFile.tests` declaration order |
| NT-8 Schema inheritance | **Implemented** | `Executor::with_stub().execute(ast, ...)` uses the same `ast` (same `§runtime`) |
| NT-9 Parse-time validation | **Partial** | `E015` duplicate-name enforced; forward-reference variable checks on `@steps:` (E014) cover declared variables; a full `@test:`-specific forward-reference check is deferred |
| NT-10 Route coverage advisory | **Implemented** | `W006` emitted by pre-existing `Validator::w006_route_test_coverage` |

## 9. Test Coverage

| Test type | Location | Count |
| --- | --- | --- |
| `parse_test_body_*` unit tests | `parser.rs #[cfg(test)]` | 3 |
| `parse_expected_value_*` unit tests | `workflows.rs #[cfg(test)]` | 9 |
| `evaluate_test_block_*` unit tests | `workflows.rs #[cfg(test)]` | 5 |
| `test_*` integration unit tests | `workflows.rs #[cfg(test)]` | 10 |
| E015 / W009 validator unit tests | `validator.rs #[cfg(test)]` | 4 |
| NT-1…NT-7 integration tests | `tests/testing.rs` | 7 |

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.1.0 | 2026-07-04 | Parallel-safe stub (§5): `StubProvider` documented as `Send + Sync`, so `@test:` blocks containing `~PARALLEL` exercise real concurrent branch scheduling with deterministic assertions (input-keyed stub + declared-order `~JOIN`); realizes the NT-5 host extension the language contract permits |
| 1.0.0 | 2026-06-24 | Initial spec — Rust implementation of NT-1…NT-10; types, API, evaluator, validator diagnostics |

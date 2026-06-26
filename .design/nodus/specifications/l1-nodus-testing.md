# Nodus DSL Testing Contract

**Version:** 1.0.1
**Status:** Stable
**Layer:** concept

## Overview

Nodus provides an inline testing facility through `@test:` blocks — named test cases embedded directly in a workflow file. Each block declares an independent invocation of the parent workflow with a custom `input:` override and one or more `expected:` assertions against pipeline outputs. This specification governs the structure, execution semantics, isolation guarantees, and reporting contract for all conforming test runners.

The testing facility is language-level: it is syntax that any conforming implementation must recognise and honour, not a host-specific extension. A workflow's `@test:` blocks are a first-class part of its specification, not a secondary tooling concern.

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — parent language spec; defines `@test:` block as a section declaration (§4.2)
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust runtime; `Executor`, `RunResult`
- [l2-nodus-testing.md](l2-nodus-testing.md) — Rust implementation of this spec; `TestBlock`, `test()`/`test_with_tags()`, assertion evaluator, NT-1…NT-10 compliance table
- [l1-nodus-observability.md](l1-nodus-observability.md) — observability contract; test runs emit execution events under the same protocol

## 1. Motivation

Workflow correctness is difficult to verify without executable test cases collocated with the workflow definition. Separate test harnesses require maintaining two files in sync, duplicating the `@in:` schema declaration, and relying on conventions that vary across hosts. Inline `@test:` blocks solve these problems:

- Tests are parsed alongside the workflow, so they always reflect the current schema and input contract.
- Assertions are checked against the same runtime that executes the workflow in production.
- Test isolation (NT-1) prevents accidental state sharing while keeping tests co-located.
- Tag-based filtering (NT-6) allows CI and manual runs to select subsets without modifying the source.

## 2. Constraints & Assumptions

- `@test:` blocks are optional; a workflow with no test blocks is valid per NL-4.
- A test block does not alter the workflow's runtime semantics — no step executes differently because a test block exists.
- `expected:` assertions are against workflow output variables, not intermediate step values (unless those values are explicitly bound to named variables via `→ $name`).
- Test blocks may not define new `!!NEVER` / `!!ALWAYS` rules or override the parent `§runtime` schema.
- A conforming test runner must execute each block independently; the execution order of blocks within a file is unspecified but must be deterministic per invocation.
- The testing facility applies only to `§wf:` files; `§schema:` and `§config:` files contain no executable steps and therefore no testable behaviour.

## 3. Core Invariants

Rules that Layer 2 implementations MUST NOT violate:

- **NT-1 Block isolation**: each `@test:` block executes in a fresh context; variable bindings, flags, and log entries from one block must not carry over into another block's execution environment.
- **NT-2 Input override**: the `input:` section within a test block fully overrides the parent workflow's `@in:` values for that test run; fields not specified in `input:` use the field's declared default, or `Null` for optional fields; the original `@in:` declaration remains the type contract.
- **NT-3 Expected assertion binding**: each entry in the `expected:` section names a workflow output variable (an `@out:` binding or any `→ $name` pipeline target) and the value it must hold after the test run; if the named variable is absent from the execution context at assertion time, the assertion fails.
- **NT-4 Assertion failure semantics**: if any `expected:` assertion evaluates to false, the test block fails; other test blocks in the same file continue executing regardless — a failure is local to its block; the overall `TestReport` accumulates pass/fail counts across all blocks.
- **NT-5 Provider neutrality**: model-calling commands (`GEN`, `REFINE`, `TRANSLATE`, etc.) inside a test run receive stub/mock responses from a test-context provider; test blocks must not trigger real external I/O, network calls, or storage writes; any provider injected at test time must satisfy the no-side-effect requirement.
- **NT-6 Tag metadata**: `tags:` is declarative metadata only — it has no effect on the execution or assertion logic of a block; it exists solely to support selection and filtering by a test runner; an absent `tags:` field is equivalent to an empty tag list.
- **NT-7 Ordered reporting**: a `TestReport` contains one `TestResult` entry per `@test:` block, in their declaration order within the file; each `TestResult` carries the block name, a pass/fail flag, and a human-readable diagnostic message.
- **NT-8 Schema inheritance**: test blocks run under the same vocabulary schema that governs the parent workflow (declared in `§runtime: { core: … }`); a test block cannot extend or replace the schema for its own run.
- **NT-9 Parse-time validation**: a `@test:` block that references a variable that cannot exist in the parent workflow (e.g., a name not declared in `@in:`, `@out:`, or any `→ $name` step) must produce a validation error, not a silent assertion-miss at run time; the block fails before execution.
- **NT-10 Route coverage advisory**: a workflow that declares one or more `ROUTE(wf:name)` steps SHOULD have at least one `@test:` block that exercises each routed target; absence of coverage emits a warning-severity diagnostic code `W001`; this is advisory and does not block execution.

## 4. Detailed Design

### 4.1 Block structure

```text
[REFERENCE]
@test: {name}
  input:
    {field}: {literal-value}
  expected:
    {variable}: {literal-value}
  tags: [{tag}, …]
```

| Field | Required | Description |
| --- | --- | --- |
| `{name}` | Mandatory | Unique test identifier within the file; used in `TestResult.name` |
| `input:` | Optional | Key-value pairs overriding `@in:` fields for this test run |
| `expected:` | Optional | Assertions: variable name → expected value; absence of `expected:` means the test passes if `Status::Ok` |
| `tags:` | Optional | List of string labels for filtering; no semantic effect on pass/fail |

Multiple `@test:` blocks with the same `name` are a validation error (code `E015`).

### 4.2 Execution protocol

1. **Parse**: the workflow file is parsed once; all `@test:` blocks are collected from `WorkflowFile.tests`.
2. **For each block** (declaration order):
   1. Build a fresh execution context: empty variable environment, no carry-over from prior blocks.
   2. Apply `input:` overrides: merge block `input:` values over the workflow's declared `@in:` defaults; `input:` keys that are not declared in `@in:` are ignored silently (they have no corresponding schema slot to bind to).
   3. Execute the full `@steps:` body against the overridden inputs using a side-effect-free provider (NT-5).
   4. Evaluate each `expected:` assertion against the final execution context (NT-3).
   5. If all assertions pass and `Status::Ok`, mark the block passed. If any assertion fails or `Status` ≠ `Ok`, mark the block failed with a diagnostic message naming the first failing assertion.
3. **Aggregate**: collect all `TestResult` entries into a `TestReport` (NT-7). Return the report regardless of individual failures — all blocks run (NT-4).

### 4.3 Assertion semantics

An `expected:` entry asserts strict equality between the variable's value and the declared literal. The equality check uses the same semantics as the `Value` type system (NL-7): `Text`, `Int`, `Float`, `Bool`, `Null`, `List`, and `Map` equality are structurally recursive. Type mismatch is an assertion failure, not an error.

```text
[REFERENCE]
@test: smoke
  input:
    query: "hello"
  expected:
    $out: "hello"         — passes if Value::Text("hello") equals run output $out
    $confidence: 0.9      — passes if Value::Float(0.9) equals $confidence
  tags: [smoke, unit]
```

### 4.4 TestReport contract

```text
[REFERENCE]
TestReport {
    results: Vec<TestResult>,  — one entry per @test: block, declaration order
    passed:  usize,            — count of TestResult entries where passed = true
    failed:  usize,            — count of TestResult entries where passed = false
}

TestResult {
    name:    String,           — @test: block name
    passed:  bool,             — true iff all assertions passed and Status::Ok
    message: String,           — human-readable outcome: first failing assertion, or "ok"
}
```

`TestReport.passed + TestReport.failed == TestReport.results.len()` is a structural invariant.

An empty `results` vector (no `@test:` blocks in the file) is a valid report with `passed = 0`, `failed = 0`.

### 4.5 Tag-based filtering

A test runner MAY accept a tag predicate at call time (e.g., `tags: ["smoke"]`) and skip blocks whose `tags:` list does not satisfy the predicate. When a block is skipped by tag filtering, it must NOT appear in the `TestReport.results` list — the report reflects only executed blocks. An empty tag list on a block matches a "run all" predicate.

### 4.6 Validation diagnostics

| Code | Severity | Trigger |
| --- | --- | --- |
| `E015` | Error | Duplicate `@test:` name within the same file |
| `W006` | Warning | `ROUTE(wf:x)` step with no `@test:` block exercising it (NT-10); code pre-exists and is re-used here |
| `W009` | Warning | `@test:` block with no `expected:` section (block will pass trivially on `Status::Ok`) |

## 5. Drawbacks & Alternatives

- **Inline vs. external test files**: external test files (e.g., `*.test.nodus`) allow larger test suites without growing the workflow source. Rejected for the base contract because co-location eliminates synchronisation drift and external files would need to import the workflow schema — reintroducing the coupling problem inline tests avoid. External suites are a host-level extension, not a language invariant.
- **Assertion language expressiveness**: `expected:` supports only strict equality. Range checks, regex matches, and partial-map assertions require a richer DSL. Deferred: the base contract covers the 80 % case (smoke tests, contract tests); richer assertions are an amendment to this spec.
- **Parallel block execution in tests**: test blocks that contain `~PARALLEL` steps run their parallel body sequentially in the stub provider context (NT-5). Parallel scheduling behaviour is not testable within the base contract; host implementations that support real parallelism may extend NT-5 with a parallel-safe stub.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[LANG]` | `crates/nodus/src/ast.rs` | `TestBlock` AST node definition |
| `[RUNNER]` | `crates/nodus/src/workflows.rs` | `test()` public API, `TestReport`, `TestResult` |
| `[PARSER]` | `crates/nodus/src/parser.rs` | `parse_test_block()` — how `@test:` body is tokenised |
| `[FIXTURES]` | `crates/nodus/tests/fixtures/` | Canonical sample workflows (normative test corpus) |

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.0.1 | 2026-06-24 | §4.6: corrected diagnostic codes — W006 (pre-existing ROUTE coverage) replaces spec-proposed W001; W009 (new no-expected advisory) replaces spec-proposed W002 to avoid conflict with existing validator codes |
| 1.0.0 | 2026-06-24 | Initial spec — NT-1…NT-10 invariants, block structure, execution protocol, assertion semantics, TestReport contract, tag filtering, validation diagnostics |

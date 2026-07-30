# Nodus DSL Testing — Rust Implementation

**Version:** 1.2.0
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
- [l1-nodus-language.md](l1-nodus-language.md) — `@test:` block as a language-level declaration; **NL-6** dual-representation (compact → human → compact must be AST-equal), the invariant §10.4's round-trip rule realizes for `TestBlock`
- [l2-nodus-control-flow.md](l2-nodus-control-flow.md) — its §3 NL-6 row governs the same round-trip guard for control-flow statements; §10.5 records why `@test:` bodies were initially outside that guard's assertion scope

## 1. Module Map

| Module | Role |
| --- | --- |
| `crates/nodus/src/ast.rs` | `TestBlock` AST node — structured fields after parsing |
| `crates/nodus/src/parser.rs` | `parse_test_block()` + `parse_test_body()` |
| `crates/nodus/src/workflows.rs` | `test()`, `test_with_tags()`, assertion evaluator, `TestReport`/`TestResult` |
| `crates/nodus/src/validator.rs` | `E015` duplicate-name check; `W009` no-expected advisory; `W015` non-conforming pair separator (§10.3) |
| `crates/nodus/src/executor.rs` | `RunResult.vars` — final variable environment exposed for assertions |
| `crates/nodus/src/transpiler.rs` | `@test:` block compact-form emission — governed by the NL-6 round-trip rule (§10.4) |

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

<!-- [MODIFIED] v1.2.0 — the previous description of this field was factually wrong. -->
`raw_lines` is the **lexed token stream of the block body**, and the structured
`input`/`expected`/`tags` fields are a **derived view** of it: `parse_test_block` calls
`collect_braced_raw_lines()` and then hands that same vector to `parse_test_body()`. They
are therefore **not alternative representations**, and both are populated for every parsed
block regardless of how the body was written. A structured field is empty only when nothing
in the token stream matched that section's pair grammar (§10.2/§10.3) — not because the body
used one style rather than another.

> Prior to v1.2.0 this section stated that "for old-style inline bodies the structured
> fields are empty and `raw_lines` holds the tokens". No parse produces that state. The
> transpiler's emission branch was written against that non-existent state and consequently
> never reached its own `raw_lines` path (§10.4).

`raw_lines` is the authoritative round-trip source (§10.4) and additionally has two live
readers in the validator — `w006_route_test_coverage` (NT-10) and the smoke-tag heuristic
both scan it textually — so it is not a transpiler-only artifact and cannot be dropped.

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
| `W015` | Warning | A token run inside `input:`/`expected:` that resembles a key-value pair but uses a separator other than `:` — the pair is skipped by `parse_test_body`, so the assertion never reaches the evaluator (§10.3) | `Validator::w015_test_pair_separator` [ADDED v1.2.0] |

`E015` is a block-class error — workflows with duplicate test names fail the validate-before-run
gate and cannot execute.

`W015` is deliberately warning-severity, not an error: its purpose is to surface assertions
that are being silently ignored in the existing corpus, which an error would instead convert
into a hard validate-before-run failure on files that parse today. Note the interaction with
`W009` — a block whose only `expected:` pairs are all non-conforming has an `expected:` section
in source but an empty one in the AST, so it emits **both** `W015` (the pairs were dropped) and
`W009` (nothing is asserted). That pairing is the intended signal, not a duplicate report.

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
| NT-9 Parse-time validation | **Partial** | `E015` duplicate-name enforced; forward-reference variable checks on `@steps:` (E014) cover declared variables; `W015` covers the silent-drop case NT-9's "not a silent assertion-miss" clause targets (§10.3); a full `@test:`-specific forward-reference check remains deferred |
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

## 10. Body Grammar Conformance & NL-6 Round-Trip [ADDED v1.2.0]

### 10.1 Canonical grammar

`l1-nodus-testing.md` §4.1 defines the body as one `{key}: {value}` pair per line beneath a
section header, and §4.3's worked example uses the same shape. That is the only form the
parent contract defines, and the only form this implementation treats as canonical:

```text
[REFERENCE]
@test: smoke {
  input:
    query: "hello"
  expected:
    $out: "hello"
  tags: [smoke]
}
```

### 10.2 The tolerated inline-brace form

`parse_test_body` scans a flat token vector for `{key} : {value}` triples and skips any token
it does not recognize. A body written inline — `input: { query: "hello", tone: "warm" }` —
therefore also parses: the `{`, `,`, and `}` tokens are simply skipped between triples.

This form is **accepted but not canonical**: it appears nowhere in `l1-nodus-testing.md` §4.1.
It is retained because corpus files use it, and it MUST NOT be extended — no nesting, no
alternative separators, no reliance on the brace structure carrying meaning. It survives a
round-trip only because `raw_lines` preserves it verbatim (§10.4), not because the grammar
models it.

### 10.3 Non-conforming pairs are diagnosed, never silently dropped

Only `:` separates a key from its value. A pair written with any other separator — in the
current corpus, `expected: { status = SUCCESS }` — matches no triple, so its tokens are
skipped and **the assertion never reaches the evaluator**. An `expected:` section reduced to
empty this way then takes the evaluator's empty-`expected` path (§6), which passes on
`Status::Ok`. The net effect is that a declared assertion silently becomes no assertion:
the block reports `passed` without ever having checked what it claims to check.

NT-9 requires that a defective test block surface "as a validation error, not a silent
assertion-miss at run time". A dropped pair is that same failure class, so it is diagnosed as
`W015` (§7).

This resolves the question of the `=` form's legality: it is **not** legal — L1 §4.1 admits
only `:` — and the defect was never that the parser rejected it, but that it rejected it
without saying so.

### 10.4 Round-trip rule (NL-6)

`l1-nodus-language.md` NL-6 requires `parse → to_nodus → parse` to be AST-equal, and
`TestBlock` — including its `raw_lines` field — is part of that AST.

Two facts constrain the emission. First, the lexer strips quotes from `StringLit` tokens
(§6), so every value held in `raw_lines` and in the structured fields is **unquoted**, and a
value emitted bare does not always re-lex to the single token it came from. Both failure
shapes are observed in the current corpus: a value containing whitespace
(`When is my invoice due?`) splits on its spaces, and a value containing a token-splitting
character (`T-001`) splits at that character — in each case the re-parse keeps only the
first fragment (`When`, `T`) as the pair's value. Second, emitting the
canonical §10.1 form for a body that was written in the §10.2 inline form produces a
**different `raw_lines`** on re-parse (the `{`/`,`/`}` tokens are gone), which breaks
AST-equality even if every value were quoted correctly.

The rule therefore has two parts:

> **(a) Source selection.** When `raw_lines` is non-empty it is the emission source, because
> it is the only representation that reproduces the body in the form the author wrote it.
> The structured fields are the fallback, used for `TestBlock` values constructed
> programmatically rather than parsed.
>
> **(b) Value re-quoting.** A value MUST be emitted so that re-lexing it yields exactly one
> token whose value equals the value emitted. Where the bare form would not, the value is
> re-quoted. (Tags and section keywords are identifiers and need no quoting.)

Part (a) is a correction to the *branch order*, not a new mechanism: the emitter already has
a `raw_lines` path, but selects it only when the structured fields are all empty — a state
no parse produces (§2), so the path is unreachable for every parsed block. Inverting the
condition makes §2's long-standing claim that "`raw_lines` is retained for the transpiler's
round-trip path" true for the first time.

Part (b) is what makes AST-equality achievable **without** storing quoting information in the
AST. NL-6 requires equality of the *parsed AST*, not of the source text; quoting is invisible
to the AST because the lexer strips it on the way back in. A conservative always-quote
emission satisfies the rule, as does quoting only those values whose bare form would not
re-lex to a single equal token.

### 10.5 Scope note

`@test:` bodies were outside the corpus-wide NL-6 round-trip guard when that guard was first
built: it asserts equality of `WorkflowFile.steps` rather than of the whole file, precisely
because the gap described here was open. Closing §10.4 is what allows that assertion to be
widened to the full `WorkflowFile`, which is the observable acceptance signal for this
section.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[AST]` | `crates/nodus/src/ast.rs` | `TestBlock` — the node whose `raw_lines`/structured-field relationship §2 corrects |
| `[PARSER]` | `crates/nodus/src/parser.rs` | `parse_test_block` / `collect_braced_raw_lines` / `parse_test_body` — the triple-scan that defines which pair shapes are recognized (§10.2/§10.3) |
| `[TRANSPILER]` | `crates/nodus/src/transpiler.rs` | `@test:` emission in `to_nodus` — the branch whose ordering §10.4(a) corrects and the site the §10.4(b) quoting rule applies to |
| `[WORKFLOWS]` | `crates/nodus/src/workflows.rs` | `test` / `test_with_tags` / `evaluate_test_block` / `parse_expected_value` — the execution and assertion path (§4–§6) |
| `[VALIDATOR]` | `crates/nodus/src/validator.rs` | `E015` / `W009` / the new `W015` (§7); `w006_route_test_coverage` and the smoke-tag heuristic, the two non-transpiler `raw_lines` readers named in §2 |
| `[CORPUS]` | `crates/nodus/tests/parity.rs` | the normative fixture corpus and its NL-6 round-trip harness — §10.5's acceptance signal is widening its assertion from `.steps` to the whole `WorkflowFile` |

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.2.0 | 2026-07-30 | Added §10 (body grammar conformance & NL-6 round-trip) and `W015`. Corrects §2, which described a `raw_lines`/structured-fields split that no parse produces — `raw_lines` is the lexed token stream and the structured fields are a derived view, both always populated; the transpiler's emission branch was written against the non-existent state and so never reached its own `raw_lines` path. §10.1/§10.2 fix the canonical body grammar to L1 §4.1's line-per-pair colon form and record the inline-brace form as tolerated-but-not-canonical. §10.3 resolves the `=`-separator question — not legal (L1 admits only `:`), the defect being that non-conforming pairs were dropped *silently*, letting a declared assertion pass without ever being checked; now `W015`, warning-severity so existing corpus files still parse. §10.4 states the two-part round-trip rule (emit from `raw_lines` when present; re-quote values that would not re-lex to a single equal token), which achieves NL-6 AST-equality without storing quoting in the AST. §1 module map gains `transpiler.rs`. |
| 1.1.0 | 2026-07-04 | Parallel-safe stub (§5): `StubProvider` documented as `Send + Sync`, so `@test:` blocks containing `~PARALLEL` exercise real concurrent branch scheduling with deterministic assertions (input-keyed stub + declared-order `~JOIN`); realizes the NT-5 host extension the language contract permits |
| 1.0.0 | 2026-06-24 | Initial spec — Rust implementation of NT-1…NT-10; types, API, evaluator, validator diagnostics |

# Nodus DSL Language

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

Nodus is a declarative, schema-driven workflow language for AI-augmented automation pipelines. A workflow describes inputs, outputs, reactive triggers, hard constraints, soft preferences, and a bounded sequential step body. The language has two syntactic representations — **compact** (machine-canonical `.nodus` files) and **human** (prose rendering) — that are semantically equivalent and convertible in either direction.

Nodus is runtime-agnostic at L1: any conforming implementation must honour the invariants below regardless of host language or platform.

## Related Specifications

- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust implementation of this language
- [l1-nodus-testing.md](l1-nodus-testing.md) — Full testing contract for `@test:` blocks (NT-1…NT-10 invariants, execution protocol, TestReport contract)
- [../../main/specifications/l2-workflow-runtime.md](../../main/specifications/l2-workflow-runtime.md) — Cronus integration layer (step binding, subsystem dispatch, platform constraints)

## 1. Motivation

Workflow automation for AI agents requires a language that:

- Enforces safety rules at the language level — hard constraints cannot be bypassed by any step handler.
- Keeps business logic auditable — human-readable rendering; declared I/O at file boundaries.
- Stays portable across host languages and platforms — schema-driven vocabulary, no built-in I/O primitives.
- Bounds execution to prevent runaway loops — mandatory `MAX:n` on unbounded iterations.
- Provides a typed error taxonomy — every failure mode has a named code that tooling can pattern-match.

## 2. Constraints & Assumptions

- Workflows are stateless between invocations; state is passed via `@in` / `@ctx` or returned via `@out`.
- The schema is the authoritative vocabulary; adding a command requires a schema update, not a runtime patch.
- Workflows are not Turing-complete by design: loops must be bounded; recursion is not supported.
- Parallel blocks (`~PARALLEL`) are a scheduling hint to the runtime, not a strict concurrency guarantee.
- A schema file (`§schema:`) is a configuration artifact, not a workflow; it has no `@steps:` body.

## 3. Core Invariants

Rules that Layer 2 implementations MUST NOT violate:

- **NL-1 Schema-first**: every workflow loads a vocabulary schema before execution; unknown commands or undefined variables fail at validation, not at run time.
- **NL-2 Hard constraints are absolute**: `!!NEVER` / `!!ALWAYS` rules halt execution immediately on violation; no command handler, conditional branch, or caller may suppress or catch them.
- **NL-3 Soft preferences are advisory**: `!PREF` rules are inputs to the execution environment; they never override `!!` rules; a step may suppress a preference locally using `!OVERRIDE`.
- **NL-4 Validate-before-run**: a workflow with any block-class validation error must not execute; invoking execution on an invalid workflow is an implementation error.
- **NL-5 Bounded loops**: `~UNTIL` requires `MAX:n` declared at the loop site; a `~UNTIL` without `MAX:n` is a validation error. `~FOR` over a finite collection is intrinsically bounded.
- **NL-6 Dual representation**: compact form and human form are semantically equivalent; compact → human → compact must produce an AST-equal result; compact is the canonical form.
- **NL-7 Closed value type system**: the runtime value space is finite: Null, Bool, Int, Float, Text, List, Map. No dynamic type extension is permitted.
- **NL-8 Reserved variable namespace**: a fixed set of variable names is owned by the runtime; user-defined pipeline targets must not shadow them.
- **NL-9 Typed I/O contract**: `@in`, `@out`, `@ctx`, and `@err` are the public contract of a workflow; an implementation must enforce input presence and type at run time.
- **NL-10 Sequential pipeline**: steps execute in declared order; a pipeline target (`→ $x`) is available from the step immediately following its definition; forward references are a validation error.

## 4. Detailed Design

### 4.1 File types

A `.nodus` source file begins with a typed sigil header:

```text
[REFERENCE]
§wf:name vX.Y      — Workflow (executable; requires @steps:)
§schema:name vX.Y  — Vocabulary + rule schema (no @steps:)
§config:name vX.Y  — Runtime configuration overlay
```

File type determines which section declarations are permitted. A schema file contains only rule and vocabulary definitions; a config file contains only runtime-binding overrides.

### 4.2 Section declarations

| Section | Required | Purpose |
| --- | --- | --- |
| `§runtime: { core: … }` | Mandatory (`§wf`) | Loads the named schema; `extends:` for overlays; `agents:` for model bindings |
| `@ON: cond → action` | Optional (≥0) | Reactive trigger — workflow activates when condition holds |
| `!!NEVER: …` / `!!ALWAYS: …` | Optional (≥0) | Absolute hard constraint (NL-2) |
| `!PREF: X OVER Y IF cond` | Optional (≥0) | Soft preference (NL-3) |
| `@in: { field: type }` | Recommended | Input contract; `?` suffix = optional; `= default` = fallback |
| `@out: $var` | Recommended | Output binding |
| `@ctx: [key, …]` | Optional | Required context keys |
| `@err: HANDLER` | Optional | Uncaught-error handler |
| `@steps:` | Mandatory (`§wf`) | Sequential numbered step body |
| `@test: name { … }` | Optional (≥0) | Inline test case (`input:` / `expected:` / `tags:`) |
| `@macro: name` | Optional (≥0) | Reusable step-sequence macro |

### 4.3 Step syntax

```text
[REFERENCE]
N. COMMAND(args) +modifier=value ^validator ~flag → $target
     │            │               │           │       │
     │            │               │           │       └── pipeline binding (optional)
     │            │               │           └────────── flag extractor (optional, repeatable)
     │            │               └────────────────────── output validator rule (optional, repeatable)
     │            └────────────────────────────────────── named modifier (optional, repeatable)
     └─────────────────────────────────────────────────── command from the schema vocabulary (NL-1)
```

Step numbers are positive integers; numbers need not be consecutive but must be monotonically increasing within `@steps:`.

Macro invocation — expands a `@macro:` body in place of a step:

```text
[REFERENCE]
N. RUN(@macro_name) +modifier=value → $target
```

`RUN` is a built-in meta-command reserved for macro expansion. It accepts the
`@macro:` name (sigil included) as its sole argument. Standard `+modifier` and
`→ $target` decorators apply; `^validator` and `~flag` decorators are not
permitted on macro steps. The transpiler renders macro invocations as
`Run macro: macro_name` in human form.

`RUN` is not a domain command and does not appear in the schema vocabulary table.
Conforming implementations must recognize `RUN(@…)` before the schema validation
pass so that macro steps are not rejected as unknown commands.

### 4.4 Control flow

```text
[REFERENCE]
?IF cond → action         — conditional branch
?ELIF cond → action       — additional branch
?ELSE → action            — default branch

~FOR $var IN $collection  — iterator loop over a finite collection
  …
~END

~UNTIL cond | MAX:n       — bounded convergence loop (NL-5: MAX:n mandatory)
  …
~END

~PARALLEL                 — parallel block (scheduling hint)
  …
~JOIN → $target           — collect results into $target
```

Branch-level flags:

- `!BREAK` — exit the enclosing loop immediately
- `!SKIP` — continue to the next loop iteration
- `!OVERRIDE` — suppress a `!PREF` rule for this branch only (NL-3)

**Parallel error propagation**: if any branch in a `~PARALLEL` block raises an
error, the block enters fail-fast state — `~JOIN` is bypassed and the error is
forwarded to the `@err:` handler (or terminates with `NODUS:UNHANDLED_ERROR`
if no handler is declared). `NODUS:RULE_VIOLATION` bypasses `@err:` entirely
per NL-2. Runtimes that execute `~PARALLEL` branches sequentially (permitted by
the "scheduling hint" clause) apply the same fail-fast semantics.

> **v0.7 target additions (see §4.6):** `?SWITCH` multi-branch dispatch, `~MAP` collection transform, `~RETRY:n` step retry, `!HALT` (fatal), `!PAUSE` (suspend).

### 4.5 Error model

A runtime error carries a typed `NODUS:*` code. The `@err:` handler is invoked for uncaught step errors. If no handler is declared, an unhandled error terminates the workflow with `NODUS:UNHANDLED_ERROR`. Hard-constraint violations (`!!`) bypass the error handler entirely and terminate with `NODUS:RULE_VIOLATION`.

| Code | Trigger |
| --- | --- |
| `NODUS:RULE_VIOLATION` | `!!` constraint violated (bypasses `@err:`) |
| `NODUS:PARSE_ERROR` | Source failed to parse |
| `NODUS:MAX_REACHED` | `~UNTIL` exhausted its `MAX:n` bound |
| `NODUS:EXECUTION_FAILED` | Step failed at run time |
| `NODUS:UNDEFINED_VAR` | Variable referenced before assignment |
| `NODUS:ROUTE_NOT_FOUND` | `ROUTE(wf:name)` target does not exist |
| `NODUS:RULE_CONFLICT` | Two `!!` rules contradict each other |
| `NODUS:SCHEMA_MISMATCH` | Schema version incompatible with workflow header |
| `NODUS:NO_SCHEMA` | Workflow executed without a loaded schema |
| `NODUS:NO_TRIGGER` | No `@ON:` matched the current input |
| `NODUS:UNHANDLED_ERROR` | Step error reached no `@err:` handler |

### 4.6 Upstream parity gaps (schema v0.4.6 → v0.7)

<!-- [ADDED] v1.1.0 -->

§4.2–§4.5 specify the **v0.4** generation of the language. The upstream schema has advanced to **v0.7**; the constructs below are defined upstream but not yet here. They are the design target for this spec and its implementation; the crate-side gap is itemized in `l2-nodus-runtime.md` §4.7.

#### Control constructs (extends §4.4)

| Construct | Semantics |
| --- | --- |
| `?SWITCH $v:` with `value → action` arms and an optional `* → default` | multi-branch dispatch on a scalar; first match wins, no fallthrough; no match and no `*` → `NODUS:SWITCH_NO_MATCH` (warn, continue) |
| `~MAP $coll: CMD($it) → $out` | single-line collection transform; implicit `$it`; empty collection → `[]`, never errors |
| `~RETRY:n` (+`backoff`, +`retry_on`) | step-level retry up to `n` (mandatory, max 10); on exhaustion routes to `@err:` |
| `!HALT` | fatal stop; status `FAILED`; requires `ESCALATE()` in the same step; no auto-resume |
| `!PAUSE` | suspend; status `PAUSED`; resumes only on explicit human re-trigger |

#### Operators & expressions

| Element | Semantics |
| --- | --- |
| `MATCHES` | deterministic regex operator in conditions (PCRE; `(?i)` prefix for case-insensitive) |
| `?.` | optional chaining — null-safe path access; short-circuits to `null` without raising `NODUS:UNDEFINED_VAR` |
| `??` | null-coalescing fallback (`$a?.b ?? default`) |
| `WHERE` / `FIRST` / `LAST` | inline collection filter/access with implicit `$it`; no match → `[]` (WHERE) or `null` (FIRST/LAST) |
| String interpolation | `$var` / `$obj.field` expand inside string literals before the step runs (runtime-resolved, not by the model); `\$` suppresses |

#### Human-in-the-loop dialog commands (new command class)

| Command | Semantics |
| --- | --- |
| `ASK(prompt)` | blocking typed question that auto-resumes on answer; `+type` = str/bool/confirm/choice/multi_choice, plus `+options`/`+hint`/`+default`/`+validate`/`+timeout` (→ `NODUS:DIALOG_TIMEOUT`) |
| `CONFIRM(content)` | approval decision; `+msg`/`+actions`/`+default`/`+strict` (reject under `+strict` → `NODUS:DIALOG_REJECTED`) |

#### Selective schema loading

`@needs:` inside `§runtime:` loads only named sections of an `extends:` schema (flat `[§a, §b]` or keyed `{ "x.nodus": [§a] }` form); a schema's `§meta` and `!!` rules always load regardless. Reduces per-execution schema context.

#### Error taxonomy (extends §4.5: 11 → 24)

Adds `UNDEFINED_CMD`, `UNDEFINED_MACRO`, `VALIDATION_FAILED`, `ESCALATION_FAILED`, `CONFIDENCE_LOW`, `KB_UNAVAILABLE`, `MEMORY_FAILED`, `TEST_FAILED`, `SWITCH_NO_MATCH`, `PAUSED`, `COUNTER_OVERFLOW`, `GIT_UNAVAILABLE`, `DIALOG_TIMEOUT`, `DIALOG_REJECTED` — each carrying a severity (error/warn/info) and category (parse/runtime/validation/routing/memory/test/control/dialog). The current `NODUS:EXECUTION_FAILED` (§4.5) is a non-canonical catch-all superseded by these specific codes.

#### Trigger priority

`@ON(priority=N):` — optional trigger priority; lower `N` = higher priority; default is declaration order.

#### Closed vocabulary registries

Beyond commands, the schema declares closed registries for **analysis flags** (`~`), **validators** (`^`), and **primitive types** that a conforming implementation should validate against (strengthening NL-1). The concrete registry contents are enumerated in `l2-nodus-runtime.md` §4.7(f).

## 5. Drawbacks & Alternatives

- **Not Turing-complete by design**: the `MAX:n` constraint prevents infinite computation; workflows requiring convergence over large iterations must increase `MAX:n` or redesign step logic. This is intentional — safety over expressiveness.
- **Alternative — general scripting language**: rejected; a general scripting language cannot enforce schema-first, bounded-execution, and hard-constraint invariants at the language level.
- **Alternative — YAML-based workflow DSL**: rejected; YAML lacks the typed section grammar, inline validators (`^`), and dual-representation contract needed for auditable AI pipelines.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[RUNTIME]` | `crates/nodus/` | Reference implementation of this spec |
| `[FIXTURES]` | `crates/nodus/tests/fixtures/` | Canonical sample workflows (normative test corpus) |

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.1.0 | 2026-06-25 | Added §4.6 upstream parity gaps (schema v0.4.6 → v0.7): control constructs (`?SWITCH`/`~MAP`/`~RETRY`/`!HALT`/`!PAUSE`), operators/expressions (`MATCHES`, `?.`, `??`, `WHERE`/`FIRST`/`LAST`, string interpolation), HITL dialog command class (`ASK`/`CONFIRM`), `@needs:` selective schema loading, error taxonomy 11 → 24, `@ON(priority=N)`, closed flag/validator/type registries; §4.4 target-additions note |
| 1.0.1 | 2026-06-23 | Added macro invocation syntax (`RUN(@macro_name)`) to §4.3; added `~PARALLEL` fail-fast error propagation semantics to §4.4 |
| 1.0.0 | 2026-06-23 | Initial spec — language invariants NL-1..NL-10, file types, section grammar, step syntax, control flow, error taxonomy |

# Nodus Error Taxonomy Implementation (Rust)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-language.md

## Overview

Concrete Rust realization of the nodus error model. `l1-nodus-language.md` §4.5
defines the original eleven `NODUS:*` codes and the `@err:` routing contract;
§4.6 expands the canonical taxonomy to twenty-four codes, each carrying a
**severity** (error / warn / info) and a **category** (parse / runtime /
validation / routing / memory / test / control / dialog), and demotes the
catch-all `NODUS:EXECUTION_FAILED` to a non-canonical legacy code superseded by
specific ones.

This spec specifies how `crates/nodus` realizes that taxonomy: the `error_code`
constant set, the severity/category metadata lookup, the supersede rule for the
catch-all code, and the emission points across the validator and executor. It
does not restate each code's trigger — those semantics live in
`l1-nodus-language.md` §4.5/§4.6; this spec maps each code to its runtime
metadata and enforcement site.

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — the error model this spec implements: §4.5 original codes + `@err:` routing, §4.6 the 24-code taxonomy with severity/category
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — the runtime crate this spec extends; `vocab::error_code`, `Error`, `RuntimeError`, the validator/executor emission sites (§4.7 itemizes the 11 → 24 gap)
- [l1-nodus-dialog.md](l1-nodus-dialog.md) — owns the `DIALOG_TIMEOUT` / `DIALOG_REJECTED` / `PAUSED` subset
- [l2-nodus-portability.md](l2-nodus-portability.md) — `NODUS:CAPABILITY_UNMET` (LP-8) is an implemented runtime code beyond the §4.6 language set

## 1. Motivation

The crate currently ships the original eleven codes (plus `CAPABILITY_UNMET`
from LP-8) as flat string constants with no severity or category metadata. The
catch-all `EXECUTION_FAILED` swallows distinct failure modes — a model
low-confidence result, a knowledge-base outage, and a counter overflow all
collapse into one opaque code, so callers cannot route or report them
differently. Closing the §4.6 gap requires three things the current code lacks:

- the thirteen additional codes, so each distinct failure is nameable;
- per-code severity and category, so a host can decide what halts, what warns, and what is informational, and can group failures for reporting;
- a supersede rule that retires `EXECUTION_FAILED` from the canonical set without breaking existing matches.

## 2. Constraints & Assumptions

- No new external crate dependency (LP-1): severity/category are in-tree enums and a static lookup table.
- Codes remain stable `NODUS:*` string constants — the wire/report contract; metadata is attached *beside* a code, never by changing its string.
- `EXECUTION_FAILED` is retained as a deprecated constant for backward-compatibility but is excluded from the canonical registry; new emission sites must use a specific code.
- Hard-constraint handling is unchanged: `NODUS:RULE_VIOLATION` still bypasses `@err:` and terminates the run (NL-2); this spec does not alter that path.
- Severity does not by itself decide control flow — the executor's existing status logic does — but it classifies a code for reporting and for the `@err:` vs. continue decision a host applies.

## 3. Invariant Compliance

| L1 Invariant | Rust Enforcement |
| --- | --- |
| NL-1 Schema-first | Validation-category codes (`UNDEFINED_CMD`, `UNDEFINED_VAR`, `NO_SCHEMA`, `VALIDATION_FAILED`) are emitted by the validator before execution, never first surfaced at run time. |
| NL-2 Hard constraints absolute | `RULE_VIOLATION` retains its dedicated path: emitted by the executor's rule check, bypasses `@err:`, forces `Status::Failed`. Severity `error`, category `runtime`; metadata never reclassifies it as catchable. |
| NL-4 Validate-before-run | Any `error`-severity, `validation`-category code in the validation report blocks execution (the existing `has_errors` gate); execution-stage codes are unreachable on an invalid workflow. |
| NL-9 Typed I/O / `@err:` contract | Every runtime error carries a canonical `NODUS:*` code routed to `@err:` (except the NL-2 bypass); `UNHANDLED_ERROR` is emitted when no handler is declared. The taxonomy is the typed error surface of the `@err:` contract. |

## 4. Detailed Design

### 4.1 Severity and category types

```text
[REFERENCE]
pub enum ErrorSeverity { Error, Warn, Info }

pub enum ErrorCategory {
    Parse, Runtime, Validation, Routing, Memory, Test, Control, Dialog,
}
```

### 4.2 Canonical code registry (severity × category)

Each canonical code maps to one severity and one category. Triggers are defined
in `l1-nodus-language.md` §4.5/§4.6 and are not restated here.

| Code | Severity | Category |
| --- | --- | --- |
| `RULE_VIOLATION` | error | runtime |
| `PARSE_ERROR` | error | parse |
| `MAX_REACHED` | warn | control |
| `UNDEFINED_VAR` | error | runtime |
| `ROUTE_NOT_FOUND` | error | routing |
| `RULE_CONFLICT` | error | validation |
| `SCHEMA_MISMATCH` | error | validation |
| `NO_SCHEMA` | error | validation |
| `NO_TRIGGER` | warn | routing |
| `UNHANDLED_ERROR` | error | runtime |
| `UNDEFINED_CMD` | error | validation |
| `UNDEFINED_MACRO` | error | validation |
| `VALIDATION_FAILED` | error | validation |
| `ESCALATION_FAILED` | error | routing |
| `CONFIDENCE_LOW` | warn | runtime |
| `KB_UNAVAILABLE` | error | memory |
| `MEMORY_FAILED` | error | memory |
| `TEST_FAILED` | error | test |
| `SWITCH_NO_MATCH` | warn | control |
| `PAUSED` | info | control |
| `COUNTER_OVERFLOW` | error | runtime |
| `GIT_UNAVAILABLE` | error | runtime |
| `DIALOG_TIMEOUT` | error | dialog |
| `DIALOG_REJECTED` | error | dialog |

Twenty-four canonical codes. `CAPABILITY_UNMET` (category `control`, severity
`error`) is an additional implemented runtime code introduced by LP-8
(`l2-nodus-portability.md`); it sits beside this set as a portability-layer code.

### 4.3 Metadata lookup

A single static table maps each canonical code string to its metadata; a lookup
returns `None` for a non-canonical code (including the legacy `EXECUTION_FAILED`),
which lets callers detect non-canonical usage.

```text
[REFERENCE]
pub fn error_meta(code: &str) -> Option<(ErrorSeverity, ErrorCategory)>;
```

The table is the single source of truth; `error_code` string constants and this
table are kept in lockstep (a test asserts every canonical constant has metadata
and every metadata row names an existing constant).

### 4.4 Superseding the catch-all

`EXECUTION_FAILED` is retained as a `#[deprecated]` constant so existing string
matches still compile, but it is excluded from the canonical registry (§4.2) and
returns `None` from `error_meta`. Each former `EXECUTION_FAILED` emission site is
reassigned to a specific code:

| Former site | Replacement |
| --- | --- |
| Unknown command dispatched | `UNDEFINED_CMD` |
| `RUN(@x)` with undefined macro | `UNDEFINED_MACRO` |
| `^validator` failed | `VALIDATION_FAILED` |
| `ESCALATE` target unreachable | `ESCALATION_FAILED` |
| Model confidence below threshold | `CONFIDENCE_LOW` |
| `QUERY_KB` backend unavailable | `KB_UNAVAILABLE` |
| `REMEMBER`/`RECALL` failure | `MEMORY_FAILED` |

A site with no more specific code keeps a generic runtime failure but must select
the closest category rather than the retired catch-all.

### 4.5 Emission points

| Stage | Codes |
| --- | --- |
| Parser | `PARSE_ERROR` |
| Validator | `NO_SCHEMA`, `SCHEMA_MISMATCH`, `RULE_CONFLICT`, `UNDEFINED_CMD`, `UNDEFINED_VAR`, `VALIDATION_FAILED` |
| Executor (control) | `RULE_VIOLATION`, `MAX_REACHED`, `SWITCH_NO_MATCH`, `PAUSED`, `COUNTER_OVERFLOW` |
| Executor (runtime/routing/memory/test/dialog) | `ROUTE_NOT_FOUND`, `NO_TRIGGER`, `ESCALATION_FAILED`, `CONFIDENCE_LOW`, `KB_UNAVAILABLE`, `MEMORY_FAILED`, `TEST_FAILED`, `GIT_UNAVAILABLE`, `DIALOG_TIMEOUT`, `DIALOG_REJECTED`, `UNHANDLED_ERROR` |

`UNDEFINED_VAR` is validator-emitted under NL-1; the optional-chaining operator
`?.` (a separate parity cluster) will later short-circuit it at run time rather
than raise it.

## 5. Drawbacks & Alternatives

- **Encode severity/category in the code string** (e.g., `NODUS:WARN:SWITCH_NO_MATCH`): rejected — it breaks the stable string contract and forces every matcher to parse structure out of an identifier.
- **One error enum variant per code**: rejected — 24+ variants bloat the `Error` type and couple every match site to the full set; a code string plus a metadata lookup keeps the surface flat and the metadata data-driven (consistent with the "vocabulary as data" property).
- **Delete `EXECUTION_FAILED` outright**: rejected — it is a published constant; deprecation plus registry exclusion retires it without a breaking removal.

## 6. Implementation Notes

- Severity/category are reporting metadata; the executor's `Status` derivation (Ok/Partial/Failed/Aborted) is unchanged. A host consults `error_meta` to decide presentation and whether a `warn`/`info` code should continue past `@err:`.
- The lockstep test (§4.3) is the guard that keeps the registry honest as future clusters (control-flow, dialog) add their codes.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CODES]` | `crates/nodus/src/vocab.rs` | `error_code` constants + the metadata table |
| `[ERR]` | `crates/nodus/src/error.rs` | `Error` enum carrying the `NODUS:*` code |
| `[RTERR]` | `crates/nodus/src/executor.rs` | `RuntimeError` emission sites |
| `[VALIDATOR]` | `crates/nodus/src/validator.rs` | validation-category emission sites |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-27 | Core Team | Initial spec — Rust realization of the 24-code taxonomy (`l1-nodus-language.md` §4.6): `ErrorSeverity`/`ErrorCategory` types, per-code severity×category registry, `error_meta` lookup, `EXECUTION_FAILED` supersede rule + site reassignment, validator/executor emission map; `CAPABILITY_UNMET` noted as the LP-8 portability-layer code. |

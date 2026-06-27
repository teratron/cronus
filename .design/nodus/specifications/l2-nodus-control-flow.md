# Nodus Control-Flow Constructs Implementation (Rust)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-language.md

## Overview

Rust realization of the v0.7 control constructs that extend the v0.4 control flow
(`?IF`/`~FOR`/`~UNTIL`/`~PARALLEL`). `l1-nodus-language.md` §4.6 defines five new
constructs; this spec records how the lexer, AST, parser, and executor realize
them: `?SWITCH` multi-branch dispatch, `~MAP` collection transform, `~RETRY:n`
step-level retry, `!HALT` fatal stop, and `!PAUSE` suspension. The supporting
machinery already exists — `Status::Failed`/`Status::Paused` and `Signal::Pause`
(Phase 10), and the `SWITCH_NO_MATCH`/`PAUSED` error codes (Phase 8) — so this
spec mostly adds parsing and dispatch.

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — the language contract: §4.4 base control flow, §4.6 the constructs added here; NL-5 bounded loops
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — runtime crate extended here: `lexer`, `ast`, `parser`, `executor`, `transpiler`, `validator`; §4.7(b) itemizes the gap
- [l2-nodus-errors.md](l2-nodus-errors.md) — owns `SWITCH_NO_MATCH` and `PAUSED`
- [l2-nodus-dialog.md](l2-nodus-dialog.md) — established `Status::Paused` + `Signal::Pause`, which `!PAUSE` reuses

## 1. Motivation

The crate parses only the v0.4 control set; `?SWITCH`/`~MAP`/`~RETRY` are not
tokenized (`~MAP`/`~RETRY` currently mis-lex as a `Flag`), and `!HALT`/`!PAUSE`
have no token or AST flag. Workflows needing multi-way dispatch, a one-line
collection transform, bounded retry, a hard stop, or a human pause cannot be
expressed. This spec closes that, reusing the pause/halt machinery already in the
executor.

## 2. Constraints & Assumptions

- No new external dependency (LP-1).
- NL-5 (bounded loops) extends to `~RETRY`: `n` is mandatory and capped (max 10); a missing or over-cap bound is a validation error.
- `?SWITCH` is first-match-wins with no fallthrough; an unmatched scrutinee with no `* →` default emits `NODUS:SWITCH_NO_MATCH` (warning) and continues.
- `~MAP` over an empty or non-list collection yields an empty list and never errors.
- `!HALT` requires an `ESCALATE()` in the same step (validation-checked); it sets `Status::Failed` and stops the run.
- `!PAUSE` reuses the dialog suspension path: `Signal::Pause` → `Status::Paused` + `ResumeDescriptor`.

## 3. Invariant Compliance

| L1 Invariant | Rust Enforcement |
| --- | --- |
| NL-5 Bounded loops | `~RETRY:n` requires a declared `n` ≤ 10; the validator rejects a missing/over-cap bound, mirroring the `~UNTIL MAX:n` rule (E010-style). |
| NL-6 Dual representation | The transpiler round-trips each new construct (compact ⇄ human) so `compact → human → compact` stays AST-equal. |
| NL-7 Closed value system | `~MAP` produces a `Value::List`; `?SWITCH` compares scalars within the closed value set; no new value kinds. |
| NL-10 Sequential pipeline | `~MAP`'s `→ $out` target follows the existing pipeline rule; `?SWITCH` arm actions bind their targets in declaration order. |

## 4. Detailed Design

### 4.1 Lexer tokens

```text
[REFERENCE]
// lexer.rs — new TokenType variants
QSwitch,     // ?SWITCH
TildeMap,    // ~MAP
TildeRetry,  // ~RETRY (carries :n like Max)
BangHalt,    // !HALT
BangPause,   // !PAUSE
```

`~MAP`/`~RETRY` must be matched as keywords *before* the generic `~identifier`
`Flag` rule, fixing the current mis-lex.

### 4.2 AST nodes

```text
[REFERENCE]
// ast.rs — new statements + action flags
pub struct SwitchBlock {
    pub scrutinee: String,                       // $var
    pub arms: Vec<(String, CommandCall)>,        // value → action
    pub default: Option<CommandCall>,            // * → action
}

pub struct MapBlock {
    pub collection: String,                      // $coll
    pub command: CommandCall,                    // body using implicit $it
    pub target: Option<String>,                  // → $out
}

pub enum Stmt { /* … */ Switch(SwitchBlock), Map(MapBlock) }

// Conditional gains action flags alongside break_flag/skip_flag:
//   halt_flag: bool, pause_flag: bool
// Step gains: retry: Option<u32>   (from ~RETRY:n)
```

### 4.3 Parser

- `?SWITCH $v:` followed by `value → action` arms and an optional `* → action`, terminated like other blocks (`~END` / dedent per the existing block rules).
- `~MAP $coll: CMD($it) → $out` as a single-line construct; `$it` is the implicit per-element binding.
- `~RETRY:n` as a step modifier attached to the step it precedes (parsed like `MAX:n`).
- `!HALT` / `!PAUSE` as action flags on a conditional/step action, parsed like `!BREAK`.

### 4.4 Executor

```text
[REFERENCE]
Stmt::Switch → evaluate scrutinee; run the first arm whose value compares equal
              (reuse the condition comparator); else run `*` default; else push
              NODUS:SWITCH_NO_MATCH (warn) and continue.
Stmt::Map    → for each element of the Value::List collection, bind $it, run the
              command, collect results into a Value::List bound to the target;
              empty/non-list ⇒ empty list.
~RETRY:n     → run the step; on a runtime error, re-run up to n times; on
              exhaustion route the error to @err (status Partial/Failed).
!HALT        → push the step's error, set Signal::Break with a halt marker ⇒
              Status::Failed.
!PAUSE       → return Signal::Pause ⇒ Status::Paused + ResumeDescriptor (reuses
              the dialog suspension path).
```

`!HALT` status precedence sits with the existing rule-violation/abort logic
(fatal); `!PAUSE` reuses the paused branch added for dialog.

### 4.5 Validator

- `~RETRY` without `:n`, or `n > 10`, → error (NL-5 bounded-loop extension).
- `!HALT` without an `ESCALATE()` in the same step → error.
- `?SWITCH` with no arms → warning.

### 4.6 Transpiler

Each new node gains compact and human emitters so NL-6 round-trip holds; the
human form reads as "Switch on …", "Map … over …", "Retry up to n", "Halt", "Pause".

## 5. Drawbacks & Alternatives

- **`?SWITCH` as desugared `?IF/?ELIF` chains**: rejected — loses the single-scrutinee intent and the precise `SWITCH_NO_MATCH` diagnostic; a first-class node round-trips cleanly (NL-6).
- **`~MAP` as a `~FOR` with an accumulator**: rejected — `~MAP` is a one-line total transform with implicit `$it`; expressing it as a loop reintroduces the boilerplate it removes.
- **`~RETRY` as an unbounded loop**: rejected — violates NL-5; the mandatory capped `n` keeps it bounded.

## 6. Implementation Notes

Phased order (each a complete vertical slice): (1) `!HALT` / `!PAUSE` action
flags — smallest, reuse `Signal`/`Status`; (2) `?SWITCH` — new node + dispatch;
(3) `~MAP` — new node + dispatch; (4) `~RETRY:n` — step modifier + retry loop.
Each slice carries lexer + AST + parser + executor + transpiler + validator +
tests. `SWITCH_NO_MATCH`/`PAUSED` codes and `Status::Paused`/`Signal::Pause`
already exist; this cluster wires them to syntax.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[LEXER]` | `crates/nodus/src/lexer.rs` | new control tokens |
| `[AST]` | `crates/nodus/src/ast.rs` | `SwitchBlock`/`MapBlock`, action flags, retry |
| `[PARSER]` | `crates/nodus/src/parser.rs` | construct parsing |
| `[EXEC]` | `crates/nodus/src/executor.rs` | dispatch + halt/pause/retry |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-27 | Core Team | Initial spec — Rust realization of the v0.7 control constructs (`?SWITCH`/`~MAP`/`~RETRY`/`!HALT`/`!PAUSE`): lexer tokens, `SwitchBlock`/`MapBlock` AST + action flags + retry, parser/executor/validator/transpiler wiring; reuses `Status::Paused`/`Signal::Pause` and `SWITCH_NO_MATCH`/`PAUSED`. Phased implementation recommended. |

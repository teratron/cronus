---
phase: 19
name: "Compensation Seam"
status: Done
subsystem: "crates/nodus"
requires: [11, 14]
provides: ["~COMPENSATE declaration surface", "Step.compensation", "completed-effect ledger", "NODUS:COMPENSATION_FAILED"]
key_files:
  created:
    - crates/nodus/tests/compensation.rs
  modified:
    - crates/nodus/src/lexer.rs
    - crates/nodus/src/ast.rs
    - crates/nodus/src/parser.rs
    - crates/nodus/src/transpiler.rs
    - crates/nodus/src/validator.rs
    - crates/nodus/src/vocab.rs
    - crates/nodus/src/executor.rs
patterns_established:
  - "A trailing same-line clause parsed via a recursive parse_command_call call must suppress the OUTER call's own unconditional skip_to_newline() — the recursive call already consumed through its own newline, so the outer one, no longer sitting at a newline, silently swallows the entire next line/step otherwise."
  - "Don't add a parallel tracking structure (or an outcome enum) for an invariant's 'honesty' property when the property already holds by construction (the ledger only ever contains what was explicitly declared) and nothing external can read the structure anyway — matches the project's no-fields-nobody-reads discipline."
duration_minutes: ~
---

# Stage 19 Tasks — Compensation Seam

**Phase:** 19
**Status:** Todo
**Strategic Goal:** Realize `l2-nodus-compensation.md` (Stable v1.0.0) in `crates/nodus` — NL-22: an effectful step may declare a host-supplied compensating action, and when the scope fails, is cancelled, or is explicitly compensated, the compensations of its **successfully-completed** steps run in reverse order of completion (LIFO). Purely additive: a workflow declaring no compensation, or a run that never fails, behaves exactly as today. Sequential tracks A (declaration surface) → B (ledger + unwind) + validation.

> **Scope note — three compositions are vacuous in core today; do not build them.** The spec records that NL-22's stated composition with the LP-11 `decide → effect → observe` gate, with NL-12 crash-resume, and with a declared sub-region scope all name machinery that **does not exist in this crate** (verified at spec time). Do **not** invent an LP-11 gate, an NL-12 resume path, or a `~SCOPE ... ~END` construct to satisfy them. Compensations route through the one existing `execute_command` effect path (so they inherit the LP-11 seam for free when it lands); at-least-once driving is per process run with the ledger as the host's replay artifact; the run *is* the compensation scope this phase.

> **Decision already made — a failed compensation continues the unwind.** NL-22 fixes the LIFO order and requires each failure surfaced, but does not say whether a failure aborts the rest. The spec resolves this (§4.5, alternative weighed in §6): **continue** to the next entry. Do not "improve" this into abort-on-first-failure — nodus cannot verify whether the failure invalidated earlier undos (host domain), and abandoning leaves more effects live with fewer attempts recorded.

## Atomic Checklist

- [x] [T-19A01] `~COMPENSATE` declaration surface — token, AST field, parser clause, transpiler
- [x] [T-19A02] `NODUS:COMPENSATION_FAILED` error code
- [x] [T-19B01] Completed-effect ledger (un-compensable record dropped, see notes — nothing reads it)
- [x] [T-19B02] Arming condition + LIFO drain + failure recording
- [x] [T-19T01] Validation suite — order, completed-only, fallible, armed, honesty, zero-dep

## Detailed Tracking

### [T-19A01] `~COMPENSATE` declaration surface

- **Spec:** l2-nodus-compensation.md §4.1 (lexer) + §4.2 (AST) + §4.3 (parser) + §4.7 (transpiler)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib lexer::` — unaffected suite passes; `~COMPENSATE` lexes as `TildeCompensate` (verified via the parser tests below, which fail loudly if it mis-lexed as `Flag`). `cargo test -p nodus --lib parser::` → `parses_compensate_clause_trailing_pipeline_target`, `no_compensation_when_clause_absent`, `compensation_inside_for_loop_body_does_not_leak_to_outer_step` (29 total, 0 failed). `cargo test -p nodus --lib transpiler::` → `compensation_clause_survives_compact_round_trip` (16 total, 0 failed).
- **Handoff:** the declaration surface every later task reads; landed first.
- **Notes:** `Step.compensation: Option<CommandCall>` mirrors `Step.retry`. `~COMPENSATE` keyword-ordering matched `~MAP`/`~RETRY`'s existing precedent automatically (added to the same `tilde_keyword` lookup). **Two real parser bugs found and fixed along the way, both pre-dating this phase's design and neither anticipated by the spec:** (1) the canonical example command `UNPUBLISH` used in the spec and initial fixtures is **not a real nodus command** (absent from `KNOWN_COMMANDS`, so it lexes as a plain `Identifier`, not `CommandName`) — switched all fixtures to real commands (`NOTIFY`, `LOG`); flagged the spec's own example as inaccurate. (2) A genuine, self-inflicted parsing bug: the trailing `~COMPENSATE: CMD(args)` clause is parsed via a **recursive** call to `parse_command_call`, which already consumes through its own newline via `skip_to_newline()`; the **outer** call's own unconditional `skip_to_newline()` then ran a second time, found itself sitting at the *start* of the next line (not at a newline), and silently swallowed that entire next step. Fixed with a `compensation_consumed_line` flag suppressing the outer call's redundant skip. Caught immediately by `compensation_inside_for_loop_body_does_not_leak_to_outer_step` and a temporary debug print showing step 2 vanishing from a 2-step fixture. Also added a `nested_body_depth` counter (incremented around `collect_body_until_end` and `parse_parallel`'s branch loop, plus the step's own sub-steps loop) so a `~COMPENSATE` clause only ever attaches to a step's own top-level action, never leaking out of a nested `~FOR`/`~UNTIL`/`~PARALLEL` body or a sub-step line into a later step's `pending_compensation.take()`.

### [T-19A02] `NODUS:COMPENSATION_FAILED` error code

- **Spec:** l2-nodus-compensation.md §4.8
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib vocab::` → `error_registry_lockstep` passes with 28 canonical codes (was 27); `error_meta(COMPENSATION_FAILED) == Some((Error, Runtime))`.
- **Handoff:** fed T-19B02, which emits it.
- **Notes:** Phase-13 `CONFIG_INVALID` precedent followed exactly (constant + `error_meta` row + lockstep-array entry, all three together).

### [T-19B01] Completed-effect ledger

- **Spec:** l2-nodus-compensation.md §4.4 + §3 (NL-22(a) row)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test compensation` → `a_clean_run_never_triggers_compensation` (a clean run's compensable steps never fire their compensation commands, confirming the ledger holds entries but nothing drains it on `Status::Ok`); `a_step_whose_own_action_fails_is_never_compensated` (a step whose own action violates a rule never reaches the ledger-append point).
- **Handoff:** the record T-19B02 drains; landed first.
- **Notes:** **`[DR]` Dropped the planned parallel `uncompensable: Vec<(u32, String)>` record and the `CompensationOutcome` enum — neither has a reader anywhere (no public accessor, no test can observe internal `ExecutionContext` state, no internal branch consults a persisted outcome).** Keeping either would be a field nobody reads, which the project's own anti-overengineering discipline forbids. NL-22(a)'s honesty property is preserved by construction instead: the ledger (`ctx.compensations: Vec<CompletedEffect>`, `CompletedEffect { step_number, compensation }`) only ever contains entries a step's `Step.compensation` explicitly declared — nothing fabricates an entry for an undeclared step, so there is nothing to falsely claim was undone. Populated at the exact point `run_step_with_retry` already detects a clean success (`ctx.errors.len() == errors_at_attempt`), reusing that existing check rather than adding a new one; a step whose action returns `Some(Signal)` (`!HALT`/`!PAUSE`/`!BREAK`/`!SKIP`) returns before that point and is correctly never recorded either (documented on the `CompletedEffect` type).

### [T-19B02] Arming condition + LIFO drain + failure recording

- **Spec:** l2-nodus-compensation.md §4.5 + §3 (NL-22(b)/(c)/(d) + CO-4 rows)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test compensation` → `compensations_run_in_reverse_completion_order_on_failure` (3 completed effects, 2 compensable; LOG (step 2's) observed before NOTIFY (step 1's) in the audit stream — LIFO, not declaration order; the non-compensable step's command never repeats); `a_failed_compensation_is_surfaced_and_does_not_abort_the_unwind` (rigged a `!!NEVER` rule against the compensation command itself — `NODUS:COMPENSATION_FAILED` present, and the *next* compensation in the ledger still runs).
- **Handoff:** completes NL-22; fed T-19T01.
- **Notes:** Armed on `Status::Failed`/`Status::Aborted` only, inserted right after `status` is computed in `execute_inner` and before the final `RunResult` is built (so anything the unwind adds to `ctx.errors` is included). Drains via plain `Vec::pop()` — LIFO falls out of that for free, no explicit reverse-iteration needed. Each compensation call's own errors-before/after check (mirroring `run_step_with_retry`'s existing success-detection idiom) decides `COMPENSATION_FAILED`; the loop never inspects or reacts to `execute_command`'s returned `Signal`, which is what lets a failed compensation's own rule violation stop *that* compensation without stopping the drain of the rest. No new `ExecutionEvent` variant — compensations route through the existing `execute_command` path, verified structurally by `compensation_emits_only_existing_event_variants`'s exhaustive match (would fail to compile if a new variant existed). **Explicit-compensate-request arming (the third condition in NL-22(d)) was not built** — no existing signal or command lets a workflow request compensation without also failing, so this is folded into the phase's existing vacuous-in-core list rather than inventing a new mechanism.

### [T-19T01] Validation Task — order, completed-only, fallible, armed, honesty

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-compensation.md` (NL-22(a)–(d) + CO-4) and confirm NL-2/NL-7/HO-6 and LP-1 still hold.
- **Method:** New `crates/nodus/tests/compensation.rs` (6 integration tests) + 3 new parser unit tests + 1 transpiler unit test + a proactive E004 coverage fix (see below).
- **Status:** Done
- **Verify:** `cargo test -p nodus` → **397 passed** (was 387 after Phase 18; +10: 3 parser + 1 transpiler + 6 compensation integration tests), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean. `cargo fmt -p nodus -- --check` → clean (after one `cargo fmt` pass). No `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` introduced outside `#[cfg(test)]`. `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; LP-1 zero-dep preserved.
- **Notes:** While building fixtures, discovered `collect_vars_step` (E004's variable-declaration walker) never visited `step.compensation` at all — a `~COMPENSATE` clause's own args/target were invisible to the "used but never assigned" check. Fixed proactively (one line: `collect_vars_cmd(comp, declared, used)` alongside the existing body/sub_steps calls) since it was directly in scope for this phase's own declaration surface, not a pre-existing unrelated defect like the two parser bugs above.

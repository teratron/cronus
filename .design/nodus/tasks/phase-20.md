---
phase: 20
name: "Branch-Action Pipeline Targets & ?SWITCH Binding Conformance"
status: Todo
subsystem: "crates/nodus"
requires: [11, 17]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 20 Tasks — Branch-Action Pipeline Targets & ?SWITCH Binding Conformance

**Phase:** 20
**Status:** Todo
**Strategic Goal:** Restore conformance to `l2-nodus-control-flow.md` §3 (Stable v1.0.1), whose NL-10 Invariant Compliance row states that **`?SWITCH` arm actions bind their targets in declaration order**. The realized code cannot honor that: `try_parse_command_from_string` constructs every branch action with `..Default::default()`, so `pipeline_target` is structurally always `None`, and `collect_vars_stmt`'s `Stmt::Switch` arm visits only the scrutinee, so nothing an arm binds is ever declared. Two coupled defects, one contract. Sequential tracks A (parser) → B (validator) + T; Track C (spec-to-as-built reconciliation) is file-independent and may land in any order.

## Context: why this is plannable without a new spec

Same class as Phase 17 (`~MAP`/`$it`): the mandate is **already Stable**, and the code diverges from it — a C12.1 fix-to-regain-conformance, not a design-blocked Backlog item. Every other remaining Backlog entry still needs an L2 realization spec first.

Grounded at plan time rather than assumed:

- **The mandate is real and specific.** `l2-nodus-control-flow.md:51` (NL-10 row, Stable v1.0.1): *"`~MAP`'s `→ $out` target follows the existing pipeline rule; `?SWITCH` arm actions bind their targets in declaration order."*
- **The parser defect is structural, not an edge case.** `try_parse_command_from_string` (parser.rs:1758) computes `args` from `r.split_once(')').map_or(r, |(a, _)| a)` — keeping the text *before* `)` and discarding the tail — then returns `CommandCall { name, args, ..Default::default() }`. There is **no** code path in the function that can set `pipeline_target` under any input.
- **The blast radius is 3 call sites, not 1.** The Backlog names only inline `?IF`, but all three callers share the one helper: `parse_error_decl` (`@err:` handler, parser.rs:741), `parse_branch_tail` (`?IF` branch action, parser.rs:1189), and `parse_switch` (arms **and** the `*` default, parser.rs:1421). The helper cannot be fixed for one caller without fixing all three — that is the shape of the code, not scope creep.
- **The executor is already correct.** `execute_switch` (executor.rs:1248) delegates to `execute_command`, which honors `cmd.pipeline_target` (executor.rs:1514/1528/1594/1622). This is a **parser+validator-only** fix; no executor change is in scope.
- **The validator defect is independent and coupled.** `collect_vars_stmt`'s `Stmt::Switch` arm (validator.rs:771) inserts only the scrutinee and never walks `sw.arms` or `sw.default` — while `extract_commands_stmt` (validator.rs:961) right below it *does* visit them, so other rules see arm actions and the variable collector does not.
- **The coverage hole is total.** Every existing `pipeline_target` assertion sits on the main-step path (`parse_command_call`) or the Phase-19 `~COMPENSATE` path. Zero tests cover any of the three `try_parse_command_from_string` sites. `crates/nodus/tests/restart.rs:63` already carries a written witness comment from Phase 18 describing the workaround this phase removes the need for.

## Sequencing rationale

Track A and Track B **must land together**, and the direction of the E004 error flips between them — this is the phase's load-bearing insight:

- Track B alone (declare/use arm actions) with Track A unfixed: harmless, because no arm ever produces a target to declare.
- **Track A alone (parse targets) with Track B unfixed: actively breaks valid workflows.** The target parses, the executor binds it, but `collect_vars_stmt` never adds it to `declared` — so a later step reading that variable is rejected `E004 "used but never assigned"`. That is precisely the `~MAP`/`$it` failure mode Phase 17 cured: parses, executes, unreachable through every validated entry point.

Hence A → B → T, with no intermediate commit boundary that ships A without B.

## Atomic Checklist

- [ ] [T-20A01] Parse a trailing `→ $target` in `try_parse_command_from_string`
- [ ] [T-20B01] Walk `?SWITCH` arm actions in `collect_vars_stmt` (uses + declarations)
- [ ] [T-20C01] Reconcile `l2-nodus-restart.md` §4.2/§4.6/§5 + INDEX row to as-built
- [ ] [T-20C02] Reconcile `l2-nodus-compensation.md` §4.4 + canonical example to as-built
- [ ] [T-20T01] Conformance + non-regression validation suite
- [ ] [T-20T02] Quality gates + LP-1 zero-dependency check

## Detailed Tracking

### [T-20A01] Parse a trailing `→ $target` in `try_parse_command_from_string`

- **Spec:** `l2-nodus-control-flow.md` §3 (NL-10 row) · `l2-nodus-runtime.md` §4 (`CommandCall.pipeline_target`)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** New unit tests in `crates/nodus/src/parser.rs` `mod tests` assert `pipeline_target == Some("$picked")` for all three call sites — a `?SWITCH` arm (`"a" → GEN(x) → $picked`), the `*` default, an inline `?IF cond → GEN(x) → $picked`, and an `@err: NOTIFY(admin) → $picked`. `cargo test -p nodus` green with a strictly higher test count than the 397 baseline and zero failures.
- **Handoff:** T-20B01 — parsed targets must become visible to the variable collector before the pair is coherent.
- **Notes / guardrails:**
  - **Split the arrow off FIRST, before splitting on `(`.** The no-paren path (`raw.split_once('(')` → `None`) sets `name = raw.trim()`, which would otherwise swallow `→ $target` into the command *name*. Splitting on the arrow first makes both the paren and no-paren paths fall out correctly.
  - The arrow is the Unicode token `\u{2192}` (`→`), emitted by lexer.rs:360; `consume_rest_of_line` joins token *values* with single spaces, so the arrow is present in the raw string as a space-delimited `→`.
  - The lowercase-first-char early return (parser.rs:1766) builds `CommandCall { name: raw, .. }` for bare non-command actions. Decide its behavior **deliberately** and state the choice in the code comment — the recommended handling is that the arrow split happens before this branch, so a bare action also gets a clean name plus its target, rather than keeping the arrow inside `name`.
  - **Do not touch the executor.** It already honors `pipeline_target`; adding a second binding path would be a redundant enforcement layer, the same trap Phase 18 declined.
  - `parse_error_decl` keeps its `raw` field alongside `handler` — confirm the transpiler round-trip for `@err:` is unaffected (it re-emits `raw`, not the parsed handler).

### [T-20B01] Walk `?SWITCH` arm actions in `collect_vars_stmt`

- **Spec:** `l2-nodus-control-flow.md` §3 (NL-10 row), §4.5 (validator-rule interactions)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** A validator test asserts that a workflow whose `?SWITCH` arm binds `→ $picked` and whose **later** step reads `$picked` produces **no** `E004`; a second asserts an arm action's own `$var` reference is now marked *used*. `cargo test -p nodus` green, zero failures.
- **Handoff:** T-20T01 — end-to-end reachability through `workflows::run`.
- **Notes / guardrails:**
  - Mirror the `Stmt::Map` arm's shape from Phase 17 (`collect_vars_cmd` over the inner command, plus declaring what the construct binds) — do **not** invent a different traversal idiom for this one construct.
  - Walk **both** `sw.arms` and `sw.default`; the default is a distinct field and is the easier of the two to forget.
  - `extract_commands_stmt` (validator.rs:961) already visits arm actions — read it first and match its traversal so the two walkers do not disagree about what an arm contains.
  - **Expected, correct behavior changes to confirm rather than suppress:** `E013` (runtime-owned variable as pipeline target) will newly fire for an arm binding e.g. `→ $restart_count`, and `E014` forward-reference ordering now includes arm-declared variables. Both are the rules doing their job per NL-8/NL-10 — verify no existing fixture or test regresses, and if one does, establish whether the fixture was relying on the gap before changing any rule.
  - This phase does **not** make `collect_vars_stmt` scope-aware. Flat-set semantics remain (a binding stays visible after its construct), consistent with `~FOR` and `~MAP` today; proper lexical scoping stays a separate Backlog design pass.

### [T-20C01] Reconcile `l2-nodus-restart.md` to as-built

- **Spec:** `l2-nodus-restart.md` §4.2, §4.6, §5 + its `INDEX.md` row
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `grep -n "Signal::Restart\|RESTART_SCOPE" .design/nodus/specifications/l2-nodus-restart.md` returns only text describing them as **not built** (with the validator-only rationale), never as the realization; §5 and the INDEX row no longer place `$restart` in `RUNTIME_OWNED_VARIABLES`; file header + INDEX row both read `1.0.1` and match exactly.
- **Handoff:** T-20C02.
- **Notes:** Two accumulated Phase-18 deviations, both already recorded in the PLAN Backlog. (a) The spec calls for a `Signal::Restart` variant and a `vocab.rs` `NODUS:RESTART_SCOPE` constant; neither was built — enforcement landed as bare validator code `E019`, matching the E010/E013/E014/E016/E017 validator-only precedent, because `Executor::execute` can be called directly with an unvalidated AST and a `Signal`-based re-check would have been this crate's first redundant enforcement layer. Record the rationale, not just the outcome. (b) §5 slice 3 and the INDEX row loosely group `$restart` into `RUNTIME_OWNED_VARIABLES` alongside `$restart_count`, contradicting §3 (authoritative) — and a runtime-owned `$restart` would make the request variable an `E013` error to assign, i.e. the feature unusable. §3 is correct; §5/INDEX are the text to fix. Patch-level bump only — no status change, no design change.

### [T-20C02] Reconcile `l2-nodus-compensation.md` to as-built

- **Spec:** `l2-nodus-compensation.md` §4.4 + its canonical example
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** §4.4 describes the ledger as the as-built `CompletedEffect { step_number, compensation }` with no parallel un-compensable record and no per-entry outcome enum; the illustrative example uses a command present in `vocab.rs::KNOWN_COMMANDS` (verify by grepping the chosen name in `crates/nodus/src/vocab.rs`); file header + INDEX row both read `1.0.1` and match exactly.
- **Handoff:** T-20T02.
- **Notes:** Two Phase-19 findings already in the Backlog. (a) §4.4 specifies a parallel `uncompensable` record plus an implied per-entry outcome; neither was built, because nothing reads either (no accessor exists and no external test can observe `ExecutionContext` internals) and NL-22(a)'s honesty property already holds by construction — the ledger only ever contains what a step explicitly declared. (b) The canonical example uses `UNPUBLISH`, which is not a real nodus command, so a reader copying it gets an `Identifier`, not a `CommandName`. Cosmetic but misleading. Patch-level bump only.

### [T-20T01] Conformance + non-regression validation suite

- **Goal:** Prove the NL-10 mandate now holds end-to-end through a validated public entry point, and that nothing outside `?SWITCH`/`?IF`/`@err:` moved.
- **Method:** Add integration coverage in `crates/nodus/tests/control_flow.rs` (the file that already drives `?SWITCH` end-to-end) asserting a `?SWITCH` workflow whose arm binds `→ $picked` runs through `workflows::run` — not `Executor::execute` — and that `$picked` holds the arm action's output, with a later step reading it successfully. Add the declaration-order assertion the NL-10 row names (two arms binding distinct targets). Add a non-regression test that a `?SWITCH` with no `→` target in any arm behaves byte-identically to today. Retire or update the Phase-18 witness comment at `crates/nodus/tests/restart.rs:63` and re-evaluate whether that test's directly-constructed `WorkflowFile` workaround can now be replaced with a parsed fixture.
- **Status:** Todo
- **Verify:** `cargo test -p nodus` green, zero failures, test count strictly above the 397 baseline; the `?SWITCH`-through-`workflows::run` test fails if T-20A01 or T-20B01 is reverted individually.

### [T-20T02] Quality gates + LP-1 zero-dependency check

- **Goal:** Definition-of-done gates for the phase.
- **Method:** Run the workspace's standing gate set for `crates/nodus`.
- **Status:** Todo
- **Verify:** All four green — `cargo test -p nodus` (zero failures); `cargo clippy -p nodus --all-targets -- -D warnings` (zero lints); `cargo fmt -p nodus -- --check` (clean); and no `.unwrap()` / `panic!()` / `unreachable!()` / `.expect(` introduced outside `#[cfg(test)]`. LP-1: `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` is empty. Spec header ↔ INDEX parity holds for both files touched by Track C.

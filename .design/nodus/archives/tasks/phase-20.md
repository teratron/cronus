---
phase: 20
name: "Branch-Action Pipeline Targets & ?SWITCH Binding Conformance"
status: Done
subsystem: "crates/nodus"
requires: [11, 17]
provides:
  - "trailing pipeline-target parsing for ?SWITCH arms/default, ?IF branch actions, and @err: handlers (try_parse_command_from_string)"
  - "?SWITCH arm-target E004 declare/use conformance (already correct; verified with regression tests, no code change)"
  - "l2-nodus-restart.md v1.0.1 (reconciled to as-built)"
  - "l2-nodus-compensation.md v1.0.1 (reconciled to as-built)"
key_files:
  created: []
  modified:
    - crates/nodus/src/parser.rs
    - crates/nodus/src/validator.rs
    - crates/nodus/tests/control_flow.rs
    - crates/nodus/tests/restart.rs
    - .design/nodus/specifications/l2-nodus-restart.md
    - .design/nodus/specifications/l2-nodus-compensation.md
    - .design/nodus/INDEX.md
patterns_established:
  - "When a re-parsed action string may carry a trailing pipeline arrow (`→ $target`), split it off FIRST — before any other structural split (e.g. on `(`) — or the no-paren path swallows the arrow and target into the command name."
  - "Before treating a plan-time grounding finding as a confirmed defect, re-read the full match arm/block rather than trusting a bounded sed/grep line range — a range that ends mid-block can manufacture a false-positive defect (this phase's planned 'second coupled defect' in collect_vars_stmt was exactly this)."
duration_minutes: ~
---

# Stage 20 Tasks — Branch-Action Pipeline Targets & ?SWITCH Binding Conformance

**Phase:** 20
**Status:** Done
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

- [x] [T-20A01] Parse a trailing `→ $target` in `try_parse_command_from_string`
- [x] [T-20B01] Walk `?SWITCH` arm actions in `collect_vars_stmt` (uses + declarations)
- [x] [T-20C01] Reconcile `l2-nodus-restart.md` §4.2/§4.6/§5 + INDEX row to as-built
- [x] [T-20C02] Reconcile `l2-nodus-compensation.md` §4.4 + canonical example to as-built
- [x] [T-20T01] Conformance + non-regression validation suite
- [x] [T-20T02] Quality gates + LP-1 zero-dependency check

## Detailed Tracking

### [T-20A01] Parse a trailing `→ $target` in `try_parse_command_from_string`

- **Spec:** `l2-nodus-control-flow.md` §3 (NL-10 row) · `l2-nodus-runtime.md` §4 (`CommandCall.pipeline_target`)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** New unit tests in `crates/nodus/src/parser.rs` `mod tests` assert `pipeline_target == Some("$picked")` for all three call sites — a `?SWITCH` arm (`"a" → GEN(x) → $picked`), the `*` default, an inline `?IF cond → GEN(x) → $picked`, and an `@err: NOTIFY(admin) → $picked`. `cargo test -p nodus` green with a strictly higher test count than the 397 baseline and zero failures.
- **Handoff:** T-20B01 — parsed targets must become visible to the variable collector before the pair is coherent.
- **Changes:** `try_parse_command_from_string` now splits off a trailing `→ $target` via `rsplit_once('→')` *before* the `(` split, so the no-paren path cannot swallow it into the command name; empty targets after trim map to `None`. All 3 call sites (`?SWITCH` arms/default, `?IF` branch action, `@err:` handler) fixed by the one change, as predicted. 4 new unit tests added (`switch_arm_action_captures_pipeline_target`, `switch_default_action_captures_pipeline_target`, `inline_if_action_captures_pipeline_target`, `error_handler_captures_pipeline_target`), all passing. Executor untouched, as planned.
- **Notes / guardrails:**
  - **Split the arrow off FIRST, before splitting on `(`.** The no-paren path (`raw.split_once('(')` → `None`) sets `name = raw.trim()`, which would otherwise swallow `→ $target` into the command *name*. Splitting on the arrow first makes both the paren and no-paren paths fall out correctly.
  - The arrow is the Unicode token `\u{2192}` (`→`), emitted by lexer.rs:360; `consume_rest_of_line` joins token *values* with single spaces, so the arrow is present in the raw string as a space-delimited `→`.
  - The lowercase-first-char early return (parser.rs:1766) builds `CommandCall { name: raw, .. }` for bare non-command actions. Decide its behavior **deliberately** and state the choice in the code comment — the recommended handling is that the arrow split happens before this branch, so a bare action also gets a clean name plus its target, rather than keeping the arrow inside `name`.
  - **Do not touch the executor.** It already honors `pipeline_target`; adding a second binding path would be a redundant enforcement layer, the same trap Phase 18 declined.
  - `parse_error_decl` keeps its `raw` field alongside `handler` — confirm the transpiler round-trip for `@err:` is unaffected (it re-emits `raw`, not the parsed handler).

### [T-20B01] Walk `?SWITCH` arm actions in `collect_vars_stmt`

- **Spec:** `l2-nodus-control-flow.md` §3 (NL-10 row), §4.5 (validator-rule interactions)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** A validator test asserts that a workflow whose `?SWITCH` arm binds `→ $picked` and whose **later** step reads `$picked` produces **no** `E004`; a second asserts an arm action's own `$var` reference is now marked *used*. `cargo test -p nodus` green, zero failures.
- **Handoff:** T-20T01 — end-to-end reachability through `workflows::run`.
- **Changes:** **No code change was needed.** Re-reading `collect_vars_stmt`'s `Stmt::Switch` arm in full (validator.rs:771-787) showed it already calls `collect_vars_cmd` over both `sw.arms` and `sw.default`, and `collect_vars_cmd` already declares `pipeline_target` and marks args used (validator.rs:893-906) — matching the `Stmt::Map`/`Stmt::Conditional` precedent exactly. **The plan-time "second, coupled defect" claim was itself a grounding error**: the `sed -n '765,780p'` range used while planning cut off mid-match-arm, one line before the arm-walking loop that was already there, making the block look shorter than it is. Corrected honestly rather than left standing (see PLAN.md's Backlog note on this). Two regression tests added instead of a fix (`e004_does_not_fire_on_switch_arm_bound_target_used_later`, `e004_fires_on_switch_arm_actions_own_undeclared_variable`), confirming the existing behavior holds now that Track A makes it observable end-to-end.
- **Notes / guardrails:**
  - Mirror the `Stmt::Map` arm's shape from Phase 17 (`collect_vars_cmd` over the inner command, plus declaring what the construct binds) — do **not** invent a different traversal idiom for this one construct.
  - Walk **both** `sw.arms` and `sw.default`; the default is a distinct field and is the easier of the two to forget.
  - `extract_commands_stmt` (validator.rs:961) already visits arm actions — read it first and match its traversal so the two walkers do not disagree about what an arm contains.
  - **Expected, correct behavior changes to confirm rather than suppress:** `E013` (runtime-owned variable as pipeline target) will newly fire for an arm binding e.g. `→ $restart_count`, and `E014` forward-reference ordering now includes arm-declared variables. Both are the rules doing their job per NL-8/NL-10 — verify no existing fixture or test regresses, and if one does, establish whether the fixture was relying on the gap before changing any rule.
  - This phase does **not** make `collect_vars_stmt` scope-aware. Flat-set semantics remain (a binding stays visible after its construct), consistent with `~FOR` and `~MAP` today; proper lexical scoping stays a separate Backlog design pass.

### [T-20C01] Reconcile `l2-nodus-restart.md` to as-built

- **Spec:** `l2-nodus-restart.md` §4.2, §4.6, §5 + its `INDEX.md` row
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `grep -n "Signal::Restart\|RESTART_SCOPE" .design/nodus/specifications/l2-nodus-restart.md` returns only text describing them as **not built** (with the validator-only rationale), never as the realization; §5 and the INDEX row no longer place `$restart` in `RUNTIME_OWNED_VARIABLES`; file header + INDEX row both read `1.0.1` and match exactly.
- **Handoff:** T-20C02.
- **Changes:** §4.2/§4.3/§4.4/§4.6 rewritten to the as-built mechanism (post-hoc `$restart` check on `RunResult.vars`, no `Signal` variant; `E019` as a bare validator code, not a `vocab.rs` constant); §3 NL-23(b) row and the `Related Specifications` line corrected likewise (both had drifted along with §4.2's original text); §5 slices 2-4 corrected; §6 gained a Drawbacks entry recording why `Signal::Restart`/`RESTART_SCOPE` were dropped; `[EXEC]` Canonical Reference corrected to name `execute_with_restart` instead of `Signal`. Header + Document History bumped to 1.0.1. `INDEX.md` row corrected to match and bumped to 1.0.1; `INDEX.md`'s own top-level Version bumped 1.0.65 → 1.0.66 (mirrors the Phase-17 Track-C precedent of bumping INDEX during a run phase's spec-reconciliation track; `PLAN.md`'s basis pointer resyncs at the next `/magic.task`, per §5 Post-Task Replan). `grep -n "Signal::Restart\|NODUS:RESTART_SCOPE" .design/nodus/specifications/l2-nodus-restart.md` confirms only §6/Document-History mentions remain, all describing the dropped design, never the realization.
- **Notes:** Two accumulated Phase-18 deviations, both already recorded in the PLAN Backlog. (a) The spec calls for a `Signal::Restart` variant and a `vocab.rs` `NODUS:RESTART_SCOPE` constant; neither was built — enforcement landed as bare validator code `E019`, matching the E010/E013/E014/E016/E017 validator-only precedent, because `Executor::execute` can be called directly with an unvalidated AST and a `Signal`-based re-check would have been this crate's first redundant enforcement layer. Record the rationale, not just the outcome. (b) §5 slice 3 and the INDEX row loosely group `$restart` into `RUNTIME_OWNED_VARIABLES` alongside `$restart_count`, contradicting §3 (authoritative) — and a runtime-owned `$restart` would make the request variable an `E013` error to assign, i.e. the feature unusable. §3 is correct; §5/INDEX are the text to fix. Patch-level bump only — no status change, no design change.

### [T-20C02] Reconcile `l2-nodus-compensation.md` to as-built

- **Spec:** `l2-nodus-compensation.md` §4.4 + its canonical example
- **Status:** Done
- **Assignment:** Agent
- **Verify:** §4.4 describes the ledger as the as-built `CompletedEffect { step_number, compensation }` with no parallel un-compensable record and no per-entry outcome enum; the illustrative example uses a command present in `vocab.rs::KNOWN_COMMANDS` (verify by grepping the chosen name in `crates/nodus/src/vocab.rs`); file header + INDEX row both read `1.0.1` and match exactly.
- **Handoff:** T-20T02.
- **Changes:** §4.4 rewritten to the as-built `CompletedEffect { step_number, compensation }` (no `step_identity`, no `CompensationOutcome`, no `uncompensable` vector); §4.5 pseudocode corrected to the actual `ctx.errors.len()` delta detection (no per-entry outcome field) and the "explicit compensate request" arming trigger removed everywhere it appeared (Overview, §3 NL-22(b)/(c)/(d), §4.5) since no such mechanism exists — only `Status::Failed`/`Aborted` arm the unwind. §4.3's canonical example `UNPUBLISH` (absent from `KNOWN_COMMANDS`) replaced with `NOTIFY` (confirmed present, `crates/nodus/src/vocab.rs:37`), matching the real test fixtures in `tests/compensation.rs`. Also fixed, found during this pass and not previously flagged: three dangling `§7` cross-references (the document has no §7) corrected to `§6`. §6 gained two Drawbacks entries recording why the outcome enum/uncompensable record and the explicit-request trigger were dropped. Header + Document History bumped to 1.0.1; `INDEX.md` row corrected and bumped to 1.0.1.
- **Notes:** Two Phase-19 findings already in the Backlog. (a) §4.4 specifies a parallel `uncompensable` record plus an implied per-entry outcome; neither was built, because nothing reads either (no accessor exists and no external test can observe `ExecutionContext` internals) and NL-22(a)'s honesty property already holds by construction — the ledger only ever contains what a step explicitly declared. (b) The canonical example uses `UNPUBLISH`, which is not a real nodus command, so a reader copying it gets an `Identifier`, not a `CommandName`. Cosmetic but misleading. Patch-level bump only.

### [T-20T01] Conformance + non-regression validation suite

- **Goal:** Prove the NL-10 mandate now holds end-to-end through a validated public entry point, and that nothing outside `?SWITCH`/`?IF`/`@err:` moved.
- **Method:** Add integration coverage in `crates/nodus/tests/control_flow.rs` (the file that already drives `?SWITCH` end-to-end) asserting a `?SWITCH` workflow whose arm binds `→ $picked` runs through `workflows::run` — not `Executor::execute` — and that `$picked` holds the arm action's output, with a later step reading it successfully. Add the declaration-order assertion the NL-10 row names (two arms binding distinct targets). Add a non-regression test that a `?SWITCH` with no `→` target in any arm behaves byte-identically to today. Retire or update the Phase-18 witness comment at `crates/nodus/tests/restart.rs:63` and re-evaluate whether that test's directly-constructed `WorkflowFile` workaround can now be replaced with a parsed fixture.
- **Status:** Done
- **Verify:** `cargo test -p nodus` green, zero failures, test count strictly above the 397 baseline; the `?SWITCH`-through-`workflows::run` test fails if T-20A01 or T-20B01 is reverted individually.
- **Changes:** Added `SWITCH_ARM_TARGETS_WF` + 2 tests to `control_flow.rs` (`switch_arm_bound_target_reachable_through_run`, `switch_arm_targets_bind_independently_per_arm`) — the latter runs the same fixture twice with different `@in.category` overrides and asserts each arm binds only its own target (`$urgent_pick`/`$spam_pick`), proving declaration-order per-arm binding, not one shared target. Non-regression: the 3 pre-existing zero-target `?SWITCH` tests pass unmodified (noted in a doc comment above their fixtures rather than duplicated). **Retired the Phase-18 workaround in `restart.rs`**: `restart_once_ast()`'s directly-constructed `AST` is gone; `restart_count_progresses_and_context_is_fresh_each_attempt` now runs a real parsed fixture (`RESTART_ONCE_WF`, using inline `?IF $restart_count < 1 → GEN(...) → $target`) through `run_with_audit` — the full parse → validate → execute path — proving the Track A fix end-to-end for the very feature that first surfaced the defect. Unused imports (`CommandCall`, `Conditional`, `RuntimeBlock`, `Step`, `Stmt`, `WorkflowFile`, `Executor`, `StubProvider`) removed accordingly.
- **Test count:** 397 → 405 (+8: 4 parser unit tests, 2 validator unit tests, 2 control_flow integration tests; restart.rs stayed at 5 — one test's implementation changed, none added/removed).

### [T-20T02] Quality gates + LP-1 zero-dependency check

- **Goal:** Definition-of-done gates for the phase.
- **Method:** Run the workspace's standing gate set for `crates/nodus`.
- **Status:** Done
- **Verify:** All four green — `cargo test -p nodus` (zero failures); `cargo clippy -p nodus --all-targets -- -D warnings` (zero lints); `cargo fmt -p nodus -- --check` (clean); and no `.unwrap()` / `panic!()` / `unreachable!()` / `.expect(` introduced outside `#[cfg(test)]`. LP-1: `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` is empty. Spec header ↔ INDEX parity holds for both files touched by Track C.
- **Changes:** `cargo test -p nodus` → 405 passed, 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean. `cargo fmt -p nodus -- --check` → 2 findings (parser.rs closure-arg formatting, a control_flow.rs long-line wrap), fixed via `cargo fmt -p nodus`, re-verified clean. Manual scan of `parser.rs`/`validator.rs` production code (outside `#[cfg(test)]`) for `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` → none found. `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty (LP-1 preserved). Header↔INDEX parity for both Track-C files confirmed via `check-prerequisites --verify-headers` → `ok: true`, 18/18 Stable, 0 warnings besides the expected `SYNC_GAP` from bumping `INDEX.md`'s own version during this run phase (self-resolves at the next `/magic.task`, per §5).

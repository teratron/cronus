---
phase: 30
name: "DG-11 Authoring Advisories"
status: Done
subsystem: "crates/nodus/src/validator.rs, crates/nodus/src/portability.rs"
requires: []
provides:
  - "W016 dialog-placement advisory (l2-nodus-dialog.md §4.8.2): w016_dialog_placement + w016_scan_scope/w016_recurse_stmt/w016_recurse_conditional, scoped to ?IF/?ELIF/?ELSE and ~FOR/~UNTIL bodies; ~PARALLEL branches walked for nesting but never paired; ?SWITCH/~MAP have no internal sequence to scope"
  - "W017 dialog-payload advisory (l2-nodus-dialog.md §4.8.3): w017_dialog_payload_inlining + w017_collect_producers/w017_scan_stmt (+ *_conditional pairs), whole-arg $var reference model over a dedicated producer map, no reuse of collect_vars_stmt"
  - "MODEL_COMMANDS/DIALOG_COMMANDS widened to pub(crate) in portability.rs for cross-module reuse"
  - "l2-nodus-dialog.md 1.2.1: §4.8.3/§4.8.4 reconciled to the as-built mechanism (whole-arg model, dedicated producer walker, precise per-construct scope list)"
key_files:
  created: []
  modified:
    - "crates/nodus/src/portability.rs"
    - "crates/nodus/src/validator.rs"
    - ".design/nodus/specifications/l2-nodus-dialog.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "A rule's own doc comment naming the exact spec-section prose it realizes (§4.8.1/§4.8.2/§4.8.3/§4.8.4) at the fn definition, not only in the phase file, so the connection survives past this session"
  - "When a design section reuses an existing shared walker's name in prose but the concrete data need differs (producer identity vs. declared/used sets), build a dedicated walker rather than overloading the shared one — cheaper than widening a function nine other rules depend on, and the spec correction records why"
duration_minutes: 45
---

# Stage 30 Tasks — DG-11 Authoring Advisories

**Phase:** 30
**Status:** Done
**Strategic Goal:** Build the two validator advisories `l2-nodus-dialog.md` §4.8 designed —
`W016` for a dialog positioned earlier than its own dependencies require, and `W017` for a
dialog prompt that carries a produced artifact instead of a reference to it. Advisory only:
no runtime behaviour changes, `Paused`/resume (DG-4) and memoization (DG-9) are untouched,
and a workflow that ignores both warnings still runs exactly as it does today.

## Scope note (read before starting)

`l2-nodus-dialog.md` §4.8 is the design source and is unusually complete for a first
implementation pass — §4.8.2 states `W016`'s predicate as a three-clause rule with both of
its boundary cases spelled out, and §4.8.3 states `W017`'s. Build against it. Three things
were ground against real source during planning and are recorded here so they are not
rediscovered at implementation time; the third is a correction to the spec's own wording.

**1. The emission pattern to copy is uniform and already established.** Every advisory in
this crate is a `Validator` associated function with the same signature, registered in one
place:

```
fn w0NN_short_name(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic>   // validator.rs
d.extend(Self::w0NN_short_name(ast, filename));                            // validator.rs:93-105
diags.push(Diagnostic::new(Severity::Warning, "W0NN", message, filename));
```

`w014_switch_has_arms` (`validator.rs:414`) is the smallest complete example; `w015_test_pair_separator`
(`:600`) is the closest in shape to what `W016` needs. Unit tests live in the same file's
`#[cfg(test)]` module as `w0NN_fires_when_…` / `w0NN_absent_when_…` pairs — the W009/W015
precedent. Follow both conventions rather than inventing a new arrangement.

**2. `W016`/`W017` are free, and the command-class constants are not reachable yet.**
`W001`…`W015` are in use in `validator.rs`; neither `W016` nor `W017` occurs in any file
under `crates/nodus/src/`. But `MODEL_COMMANDS` and `DIALOG_COMMANDS` (`portability.rs:209`
and `:213`) are **module-private** `const`s, and `validator.rs` imports only `ast`,
`executor::Value` and `vocab` — it cannot see them today.

> **Decision — widen to `pub(crate)`, do not relocate.** The alternative is moving both sets
> into `vocab.rs` beside the other closed registries (`KNOWN_COMMANDS`, `KNOWN_FLAGS`,
> `PRIMITIVE_TYPES`). Rejected: these two sets exist to define `EffectClass` and belong beside
> it, the validator only needs to *read* them, and a crate-internal visibility change publishes
> nothing (LP-6 neutral) while relocation edits two modules for no behavioural gain. Add
> `pub(crate)` and a `use crate::portability::{DIALOG_COMMANDS, MODEL_COMMANDS};` to
> `validator.rs`.

**3. Plan-time correction to §4.8.3 — `W017` fires on a whole-arg reference, not on an
interpolation.** §4.8.3 says the advisory fires when a prompt "interpolates a variable whose
value was written by a `GEN`/`ANALYZE` step". Verified against `collect_vars_stmt`
(`validator.rs:820-833`) and `CommandCall` (`ast.rs:149-162`): nodus models a variable
reference as a **whole argument token beginning with `$`** — every reference site in the
validator tests `arg.starts_with('$')` — and there is no string-interpolation scanner in the
validator at all. So the realizable trigger is *an `ASK`/`CONFIRM` argument that is itself a
bare `$var`* whose writer is a `MODEL_COMMANDS` step, which is the same defect §4.8.3 is
about (the artifact travelling in the prompt rather than a reference to it) expressed in the
model the crate actually has. **Build the whole-arg rule; T-30C01 folds the correction into
the spec once it lands** — the Phase 29 precedent of recording a plan-time mechanism
correction in the phase file first and reconciling the spec after, never the reverse.

**What must not happen.** No new AST node, no new grammar, no parser or transpiler change, no
executor change, and no `Severity::Error`. Both advisories are computed at validation time
from the AST alone (NL-4), inspect no runtime value, and must leave the existing diagnostic
count for every fixture in `tests/fixtures/` unchanged except where a fixture genuinely
exhibits the pattern — check that before assuming a test failure is a bug in the walker.

> **Implementation-time correction to item 2's Notes (T-30A02) — found while building, not at
> plan time.** The plan's Notes said to recurse into `Stmt::Switch` (arms + default) and
> `Stmt::Parallel` branches the same way as `Conditional`/`ForLoop`/`UntilLoop`. Neither holds
> up against the AST: `SwitchBlock.arms`/`.default` and `MapBlock.command` are a single
> `CommandCall`, not a `Vec<Stmt>` — there is no internal sequence for a dialog to have
> siblings within, so nothing to recurse into as a *scope*. `ParallelBlock.branches` **is** a
> `Vec<Stmt>`, but its branches run concurrently, not sequentially, so pairing them the way
> `w016_scan_scope` pairs a sequential body's members would advise moving a dialog past a
> sibling that does not run "before" it at all — pairing is skipped for `Parallel`, though each
> branch is still walked for scopes nested further inside it. Landed in `l2-nodus-dialog.md`
> 1.2.1 (T-30C01) alongside the whole-arg correction, rather than left for a later audit.

## Atomic Checklist

- [x] [T-30A01] `pub(crate)` command classes + `w016_dialog_placement` walker
- [x] [T-30A02] Block scoping for `W016` — nested constructs evaluated as their own blocks
- [x] [T-30B01] `w017_dialog_payload_inlining` walker
- [x] [T-30C01] Spec reconciliation — `l2-nodus-dialog` §4.8.3 to as-built
- [x] [T-30T01] `W016` coverage — fires, and the three cases where it must not
- [x] [T-30T02] `W017` coverage — fires, and the two cases where it must not

## Detailed Tracking

### [T-30A01] `pub(crate)` command classes + `w016_dialog_placement` walker

- **Spec:** l2-nodus-dialog.md §4.8.1 (the three facts), §4.8.2 (the predicate)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. A workflow whose steps are `1. ASK("ok?") → $a` then `2. LOG("x") +reversible=true`
  produces exactly one `W016` from `Validator::validate`; the same workflow with step 2's
  modifier removed produces none. Assert on the diagnostic's `code` field, not on message text.
  **Satisfied**: `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  both clean; `w016_fires_when_dialog_precedes_declared_reversible_step` and
  `w016_absent_when_following_step_declares_nothing` (`validator.rs` tests) cover exactly this
  pair, asserting on `d.code == "W016"`.
- **Handoff:** T-30A02 extends the same walker to nested blocks; T-30B01 reuses the per-step
  inventory it builds; T-30T01 is its acceptance evidence.
- **Notes:** Add `pub(crate)` to `MODEL_COMMANDS`/`DIALOG_COMMANDS` per the Scope note's
  decision. Build the walker over `wf.steps` in declaration order: for each dialog step `D`
  (`CommandCall.name` in `DIALOG_COMMANDS`), scan the steps after it for the **last** `S`
  satisfying §4.8.2 (b) — non-dialog, declaring `+reversible=true` in `CommandCall.modifiers`,
  not declaring `+external=true` — stopping the scan at the first step that reads `D`'s
  `pipeline_target` (clause (c)). Emit one diagnostic naming that last `S`. A dialog with
  `pipeline_target: None` has an empty dependency set, so clause (c) never stops the scan —
  that is the intended reading per §4.8.2's boundary paragraph, not a case to skip. Both
  modifier clauses are load-bearing and independent: a step may declare `+reversible=true`
  **and** `+external=true`, and an outward effect stops the dialog moving past it regardless
  of reversibility.
- **Changes:** `portability.rs`: `MODEL_COMMANDS`/`DIALOG_COMMANDS` widened `const` →
  `pub(crate) const`. `validator.rs`: added `use crate::portability::{DIALOG_COMMANDS,
  MODEL_COMMANDS};`; `w016_dialog_placement` (dispatch-registered method) flattens
  `wf.steps` (each step's `body` then `sub_steps`, in order) into one root scope and calls
  `w016_scan_scope`. **Grounding correction found while implementing**: `CommandCall.modifiers`
  keys carry the surface `+` (`"+reversible"`, `"+external"`, matching the LP-11/LP-16 call
  site at `executor.rs:1608-1613` verbatim) — not the bare name the earlier plan assumed from
  reading only the derived `context` map in `tests/portability.rs`; the walker matches on
  `"+reversible"`/`"+external"`.

### [T-30A02] Block scoping for `W016` — nested constructs evaluated as their own blocks

- **Spec:** l2-nodus-dialog.md §4.8.4 (first and second bullets)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** A dialog inside an `?IF` body followed by a `+reversible=true` step **inside the
  same body** produces one `W016`; a dialog inside an `?IF` body followed by a
  `+reversible=true` step **after the `?IF`** produces none. `cargo test -p nodus --lib
  validator::` green.
  **Satisfied**: `w016_scoped_to_same_block_fires_when_reversible_step_inside_same_if_body`
  and `w016_scoped_to_same_block_absent_when_reversible_step_is_after_the_block` cover both
  directions; `cargo test -p nodus --lib validator::` 66 passed.
- **Handoff:** T-30T01 asserts both directions as regression tests.
- **Notes/Correction:** Planned to recurse into `Stmt::Switch` (arms + default) and
  `Stmt::Parallel` branches the same way as `Conditional`/`ForLoop`/`UntilLoop` — **did not
  survive grounding against the AST** (see the phase-file note above `## Atomic Checklist`).
  `w016_recurse_stmt` recurses into `Conditional` (via `w016_recurse_conditional`, covering
  `body`/`elif_branches`/`else_branch`), `ForLoop.body`, `UntilLoop.body`, and walks (but does
  not pair) `Parallel.branches`; `Switch`/`Map`/`Command`/`VarRef`/`Comment` are leaves for
  this rule (no `Vec<Stmt>` to scope). Exhaustive `match` with no `_` arm, per the Phase 21
  precedent.
- **Changes:** `validator.rs`: `w016_scan_scope` (pairing within one flat scope) and
  `w016_recurse_stmt`/`w016_recurse_conditional` (independent recursion into nested scopes)
  added as free functions beside `find_empty_switches_stmt`.

### [T-30B01] `w017_dialog_payload_inlining` walker

- **Spec:** l2-nodus-dialog.md §4.8.3, **as corrected by this phase file's Scope note item 3**
- **Status:** Done
- **Assignment:** Agent
- **Verify:** A workflow `1. GEN("draft") → $d` / `2. ASK("approve?", $d) → $ok` produces
  exactly one `W017` naming `$d` and step 1; replacing step 1 with an `@in` declaration of
  `$d` produces none; replacing `$d` in step 2 with a string literal produces none.
  `cargo clippy -p nodus --all-targets -- -D warnings` clean.
  **Satisfied**: `w017_fires_when_prompt_carries_a_gen_produced_artifact`,
  `w017_absent_when_variable_has_no_producer`, `w017_absent_when_prompt_argument_is_a_string_literal`
  cover exactly these three; clippy clean.
- **Handoff:** T-30C01 folds the whole-arg correction into the spec; T-30T02 is its acceptance
  evidence.
- **Notes/Correction:** Planned to reuse `collect_vars_stmt`'s "declaration-order walk" — did
  not survive grounding: that function tracks *declared*/*used* variable-root `HashSet`s for
  `E004`/`E014`, with no notion of *which command* produced a target. Built a **dedicated**
  producer-tracking pair instead — `w017_collect_producers`/`w017_collect_producers_conditional`
  — recording `target-root → producing command name` (unordered walk; ordering is irrelevant
  since a use-before-producer is already a separate `E014` finding), then
  `w017_scan_stmt`/`w017_scan_conditional` fire on a bare `$var` argument to `ASK`/`CONFIRM`
  whose producer is in `MODEL_COMMANDS`. No size/length heuristic (the value does not exist at
  validation time, NL-4). A variable with no producer entry (an `@in` field, or an undeclared
  reference `E014` already catches) never fires.
- **Changes:** `validator.rs`: `w017_dialog_payload_inlining` (dispatch-registered method) +
  four new free functions (`w017_collect_producers`, `w017_collect_producers_conditional`,
  `w017_scan_stmt`, `w017_scan_conditional`).

### [T-30C01] Spec reconciliation — `l2-nodus-dialog` §4.8.3 to as-built

- **Spec:** l2-nodus-dialog.md §4.8.3 + Document History
- **Status:** Done
- **Assignment:** Agent
- **Verify:** §4.8.3 no longer describes `W017` as firing on an *interpolated* variable and
  instead states the whole-arg `$var` reference model, with one sentence recording why (nodus
  has no string-interpolation scanner; a reference is a whole argument token). `l2-nodus-dialog.md`
  1.2.0 → 1.2.1 with a Document History row; `INDEX.md` version cell updated to match; a
  re-run of `check-prerequisites --verify-headers --workspace=nodus` reports no
  `VERSION_DRIFT`.
  **Satisfied**: §4.8.3 rewritten (whole-arg model + dedicated-walker correction, both
  named); §4.8.4's first bullet also corrected (precise per-construct scope list — see
  T-30A02's note) since implementation diverged from that bullet too, not only §4.8.3.
  `l2-nodus-dialog.md` 1.2.0 → 1.2.1, Document History row added, `INDEX.md` version cell and
  top-level `**Version:**` (1.0.92 → 1.0.93) both updated. `check-prerequisites --json
  --require-tasks --verify-headers --workspace=nodus` reports `ok: true`, no `VERSION_DRIFT`
  (only the expected `SYNC_GAP` — `PLAN.md`'s basis pointer is `/magic.task`'s to update, not
  `/magic.run`'s).
- **Handoff:** Closes the phase's spec-sync obligation; nothing downstream reads it.
- **Notes:** Patch-level, stays `Stable` — the correction narrows *how* the rule is expressed
  without changing what it forbids, matching the `l2-nodus-restart` 1.0.0 → 1.0.1 and
  `l2-nodus-compensation` 1.0.0 → 1.0.1 as-built reconciliations. §4.8.2 left untouched — the
  implementation did not diverge from its predicate, only from §4.8.3's producer-lookup
  description and §4.8.4's scope-list bullet.
- **Changes:** `.design/nodus/specifications/l2-nodus-dialog.md` 1.2.0 → 1.2.1 (§4.8.3
  rewritten, §4.8.4 first bullet rewritten, Document History row added); `INDEX.md` version
  cell 1.2.0 → 1.2.1, top-level version 1.0.92 → 1.0.93, Meta Information entry added.

### [T-30T01] `W016` coverage — fires, and the three cases where it must not

- **Spec:** l2-nodus-dialog.md §4.8.2 (all clauses + both boundary cases)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` green with at least five new assertions, each named for
  the clause it pins: fires on a dialog followed by a declared-reversible step; **absent**
  when that step also declares `+external=true`; **absent** when a step between them reads the
  dialog's `pipeline_target`; **absent** when the following step declares nothing at all
  (the soundness trade of §4.8.2); fires for a dialog with no `pipeline_target` followed by a
  declared-reversible step. Plus T-30A02's two block-scoping directions.
  **Satisfied**: seven tests, all named for the clause pinned —
  `w016_fires_when_dialog_precedes_declared_reversible_step`,
  `w016_absent_when_following_step_declares_external_true`,
  `w016_absent_when_intervening_step_reads_dialog_target`,
  `w016_absent_when_following_step_declares_nothing`,
  `w016_fires_for_dialog_with_no_pipeline_target`,
  `w016_scoped_to_same_block_fires_when_reversible_step_inside_same_if_body`,
  `w016_scoped_to_same_block_absent_when_reversible_step_is_after_the_block`. Built via direct
  `WorkflowFile`/`Step`/`Stmt`/`CommandCall` struct literals (the `no_e019_for_top_level_restart_request`
  precedent), not source-text parsing, for precise nested-block control.
- **Handoff:** Phase acceptance signal for Track A.
- **Notes:** Asserted on `Diagnostic.code`, never on message wording — the W015 tests
  precedent. The "declares nothing" case is commented in-line as the one worth protecting most
  carefully: it pins a deliberate non-firing against a future "fix" that would raise recall by
  firing on undeclared steps.

### [T-30T02] `W017` coverage — fires, and the two cases where it must not

- **Spec:** l2-nodus-dialog.md §4.8.3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` green with three new assertions: fires when an `ASK`
  argument is a `$var` written by `GEN` (and by `ANALYZE`); **absent** when the variable comes
  from `@in`; **absent** when the argument is a string literal. Full-suite count recorded in
  the phase completion note, and `cargo fmt --all` clean.
  **Satisfied**: four tests — `w017_fires_when_prompt_carries_a_gen_produced_artifact`,
  `w017_fires_when_prompt_carries_an_analyze_produced_artifact` (GEN and ANALYZE as separate
  cases per the parenthetical), `w017_absent_when_variable_has_no_producer`,
  `w017_absent_when_prompt_argument_is_a_string_literal`. `cargo test -p nodus`: **482 passed,
  0 failed** (was 471, +11: 7 `W016` + 4 `W017`); `cargo fmt --all -- --check` clean.
- **Handoff:** Phase acceptance signal for Track B.
- **Notes:** Ran the full `tests/fixtures/*.nodus` corpus (14 files) through
  `Parser::parse` → `Validator::validate` via a scratch example, deleted after use (not a
  committed artifact): **zero fixtures newly emit `W016` or `W017`**. Checked by hand why:
  `retry_bounded.nodus`'s `ASK($in.question)` references an `@in` field with no producer;
  `halt_pause.nodus`'s `?IF … → ASK(confirm) !PAUSE` is a branch-*action* `CommandCall`
  (`Conditional.action`), which is architecturally never scanned as a `W016`/`W017` candidate
  — only a plain `Stmt::Command` in a scope's flat sequence is (a branch action has no
  following sibling within its own arm to be advised past or to inline into, so the exclusion
  costs nothing observable). No other fixture combines a dialog with a model-produced variable
  or a declared-reversible neighbour.
- **Changes:** `validator.rs` test module: 11 new tests (`w016_*` ×7, `w017_*` ×4) plus two
  small shared helpers (`cmd_step`, `wf_with_steps`) built on the existing direct-AST-literal
  idiom.

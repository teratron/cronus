---
phase: 26
name: "Uncaught-Error Handler Dispatch"
status: Done
subsystem: "crates/nodus/src/executor.rs"
requires: []
provides:
  - "NL-9 @err: handler dispatch — structural eligibility check + $error population + handler dispatch in execute_inner's main loop"
  - "l2-nodus-error-dispatch.md 1.0.1: NL-9 fully reconciled to as-built"
key_files:
  created: []
  modified:
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/tests/portability.rs"
    - "crates/nodus/tests/control_flow.rs"
    - ".design/nodus/specifications/l2-nodus-error-dispatch.md"
    - ".design/nodus/specifications/l2-nodus-runtime.md"
    - ".design/nodus/specifications/l2-nodus-errors.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "Structural eligibility over enumerated code lists: a mechanism attached to an existing control-flow shape (which errors return no Signal) needs no maintained allowlist and extends to future codes for free"
  - "Activating previously-inert declared behavior can break pre-existing tests outside the immediately-related test file — swept tests/control_flow.rs, not just the LP-11/LP-16 tests named at spec time, and found a real regression there"
duration_minutes: ~
---

# Stage 26 Tasks — Uncaught-Error Handler Dispatch

**Phase:** 26
**Status:** Done
**Strategic Goal:** Invoke a workflow's declared `@err:` handler exactly once when a non-fatal runtime error escapes ordinary step execution uncaught, exactly as `l2-nodus-error-dispatch.md` specifies — no new DSL grammar, no new AST field, no `PolicyProvider`/`AuditProvider` change, no new error code.

## Scope note (read before starting)

`l2-nodus-error-dispatch.md` (v1.0.0) is the authoritative design for this phase — every
task below cites the subsection it realizes. `WorkflowFile.error_decl` is already fully
parsed, validated, and transpiled (`ast.rs`/`parser.rs`/`validator.rs`/`transpiler.rs`) —
nothing in those layers changes. This phase adds exactly one thing: the executor actually
invoking the already-parsed handler.

**Eligibility is structural, not an enumerated list (§4.1).** Any `RuntimeError` a
top-level step returns with no `Signal` reaches the dispatch check — today that is
`POLICY_DENIED` (LP-11) and `DIALOG_TIMEOUT`/`DIALOG_REJECTED` (dialog). `RULE_VIOLATION`
returns `Signal::Break` and is caught by an earlier match arm — it never reaches the
dispatch check, so it needs no explicit exclusion. `COMPENSATION_FAILED` is pushed only
during the post-loop compensation drain (after the main loop has already exited with
`Status::Failed`/`Aborted`) — it cannot occur inside the loop this phase attaches to. Do
**not** add a hardcoded list of "dispatchable codes" anywhere — the control-flow shape is
the whole mechanism.

**Dispatch fires once per top-level step (see PLAN.md's Scope note for this phase).** If
the top-level step is itself a `~FOR`/`~UNTIL`/`~PARALLEL` construct, its own inner
iterations run to completion exactly as today (nested errors do not stop nested iteration);
once the *whole* top-level step returns, the dispatch check fires once using
`ctx.errors.last()`. A step that succeeds via `~RETRY:n` truncates its own transient
attempt-errors back to `errors_before` (`run_step_with_retry`, `executor.rs:1027`) before
returning — so a retry-then-succeed step structurally never reaches dispatch; do not add
special-case logic for this, it already falls out of the existing retry mechanism.

**Explicitly out of scope**, per the spec's own §2/Drawbacks:

- Any new `NODUS:*` error code — dispatch reuses whichever code the triggering error
  already carries. `UNHANDLED_ERROR` stays unemitted.
- A DSL-level named-gate declaration, or any `PolicyProvider`/`AuditProvider` signature
  change.
- Making the handler dispatch re-entrant. If the handler's own execution pushes a new
  error, it surfaces in `ctx.errors` like any other command's — it does **not** re-trigger
  `@err:` dispatch. Do not add a recursion guard beyond simply not calling the dispatch
  check recursively.
- Dispatching at run-end instead of at the point of failure (collecting every error, then
  invoking `@err:` once after all steps ran) — rejected in the spec (§6); dispatch is the
  terminal action for the step that triggered it.

## Guardrails (from the §4.1–§4.5 design)

1. **`$error` populated before the handler dispatches.** `$error` is already `RESERVED` +
   `RUNTIME_OWNED` (`vocab.rs`), seeded to `Value::Map(Vec::new())` at context construction,
   and never written to by anything today. Populate it from the triggering `RuntimeError`
   (`code`/`step`/`reason`, mirroring the struct's own fields) *before* calling
   `execute_command` for the handler, so a handler referencing `$error.reason` via ordinary
   `$var` interpolation sees it.
2. **The handler dispatches through the ordinary `execute_command` path**, reusing the
   triggering step's own `step_num` (the compensation-drain precedent —
   `self.execute_command(&mut ctx, &effect.compensation, effect.step_number)`,
   `executor.rs` around the compensation-drain loop). This means the handler gets its own
   `StepStart`/`StepEnd` events, honors its own `→ $target` pipeline binding if the workflow
   declared one, and passes through the LP-11 gate + LP-16 descriptors like any other
   command — no special-casing.
3. **Main step sequence ends after dispatch.** `break` the main loop once the handler has
   been dispatched — the workflow's remaining declared `@steps:` do not execute.
4. **`Status` computation is untouched.** Do not add a new `Status` variant or change the
   `Partial`/`Failed`/`Aborted`/`Ok` logic. A denial still degrades to `Partial` via the
   existing `!ctx.errors.is_empty()` branch; dispatch is an additional observable side
   effect on top of that, never a reinterpretation of it.
5. **No handler declared, or handler didn't parse (`error_decl.handler == None`).**
   Behavior is byte-for-byte unchanged from before this phase — do not invent a fallback or
   a partial-dispatch path for unparseable handler text.

## Atomic Checklist

- [x] [T-26A01] Structural dispatch check + `$error` population + handler dispatch
- [x] [T-26C01] Reconcile the touched specs to the as-built result
- [x] [T-26T01] New coverage: dispatch fires, `$error` populated, no-op cases, retry-then-succeed
- [x] [T-26T02] Review and update existing Phase 24/25 tests that already trigger a dispatchable error; final gate run

## Detailed Tracking

### [T-26A01] Structural dispatch check + `$error` population + handler dispatch

- **Spec:** l2-nodus-error-dispatch.md §4.1, §4.2, §4.3, §4.4, §4.5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; a temporary manual check (or the Track T tests)
  confirms: (a) a `GEN` step decorated with `@err: ESCALATE(human)` run against a deny-all
  `PolicyProvider` produces a `context`-style `$error` map with `code`/`step`/`reason`
  populated, the handler's own `StepStart`/`StepEnd` events present in the log, and no
  further declared steps executed; (b) a workflow with no `@err:` line, run the same way,
  behaves exactly as it does before this phase (no `$error` write, no extra dispatch).
  **Satisfied**: `err_handler_dispatches_on_policy_denial` and
  `no_err_handler_declared_is_unchanged` (T-26T01) both pass; `cargo check -p nodus` clean.
- **Handoff:** T-26C01 reconciles the spec once this is real; T-26T01/T-26T02 add the full
  coverage matrix.
- **Changes:** `execute_inner`'s main step loop (`executor.rs`) gained
  `let errors_before_this_step = ctx.errors.len();` before the existing
  `match self.run_step_with_retry(...)`, and the `_ => {}` catch-all arm gained a let-chain
  (`if let Some(handler) = ast.error_decl.as_ref().and_then(|d| d.handler.as_ref()) &&
  ctx.errors.len() > errors_before_this_step`) that populates `$error` via a new
  `error_to_value(&RuntimeError) -> Value` helper (added beside `RuntimeError`'s own
  definition), dispatches the handler via `self.execute_command(&mut ctx, handler,
  step.number)`, then `break`s the loop. No new type, no new field, no new error code, no
  `PolicyProvider`/`AuditProvider` change.
- **Notes:** In `execute_inner`'s main step loop, the existing match on
  `self.run_step_with_retry(&mut ctx, step)` has arms for `Signal::Halt`/`Signal::Break`/
  `Signal::Pause` and a catch-all `_ => {}`. Capture `let errors_before_this_step =
  ctx.errors.len();` immediately before the `match`. Inside the `_ => {}` arm:

  ```
  _ => {
      if let Some(handler) = wf.error_decl.as_ref().and_then(|d| d.handler.as_ref()) {
          if ctx.errors.len() > errors_before_this_step {
              ctx.variables.insert(
                  "error".to_string(),
                  error_to_value(ctx.errors.last().expect("just grew")),
              );
              self.execute_command(&mut ctx, handler, step.number);
              break;
          }
      }
  }
  ```

  Add a small private helper (near `RuntimeError`'s definition or beside `execute_command`):

  ```
  fn error_to_value(err: &RuntimeError) -> Value {
      Value::Map(vec![
          ("code".to_string(), Value::Text(err.code.clone())),
          ("step".to_string(), Value::Int(err.step as i64)),
          ("reason".to_string(), Value::Text(err.reason.clone())),
      ])
  }
  ```

  Check `wf`'s exact binding name inside `execute_inner` before writing this (it may be
  `self.workflow`, a parameter, or reached via `ctx` — confirm rather than assume). Check
  `Step.number`'s exact field name/type (used elsewhere in the file, e.g. the compensation
  drain's `effect.step_number` — confirm whether `Step` itself uses `number` or a different
  field name). `ctx.variables.insert("error", ...)` mirrors the exact pattern the boot
  sequence already uses to seed `$error` (`ctx.variables.entry("error".to_string())...`) —
  use `insert`, not `entry(...).or_insert(...)`, since this call must overwrite the seeded
  empty map.

### [T-26C01] Reconcile the touched specs to the as-built result

- **Spec:** l2-nodus-error-dispatch.md (all sections), l2-nodus-runtime.md §3 NL-9 row,
  l2-nodus-errors.md NL-9 row
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `node .magic/scripts/executor.js check-prerequisites --json --require-specs --verify-headers --workspace=nodus`
  reports no `VERSION_DRIFT` and every touched file's header matches its `INDEX.md` row;
  `grep -n "never invokes the declared handler"
  .design/nodus/specifications/l2-nodus-errors.md` no longer matches (the claim becomes
  false once T-26A01 lands).
  **Satisfied**: pre-flight `ok: true`, only the expected post-bump `SYNC_GAP`; grep for
  stale "not yet implemented"/"zero call sites" language across all three touched specs
  found only the historical Overview framing in `l2-nodus-error-dispatch.md` (legitimate —
  it describes the pre-fix problem the spec exists to solve, immediately followed by "this
  spec closes that gap").
- **Handoff:** Track T.
- **Changes:** `l2-nodus-error-dispatch.md` 1.0.0 → 1.0.1 — §3's NL-9 row, §4.3's pseudocode
  (`wf` → `ast`, collapsed let-chain, dropped the unused `err_dispatched` flag), §4.5
  (corrected: `try_parse_command_from_string` returns `None` only for empty input, never
  for unrecognized-but-non-empty text — the "handler text unparseable" framing was never
  really true), Document History. `l2-nodus-errors.md` 1.1.1 → 1.1.2 and
  `l2-nodus-runtime.md` 1.3.1 → 1.3.2 — both NL-9 rows updated from "specified"/"realized
  by" to Implemented. `INDEX.md` rows + top-level version (1.0.75 → 1.0.76) synced for all
  three.
- **Notes:** Patch or minor bump depending on how much the spec's `[REFERENCE]` pseudocode
  diverges from the real, compiled shape (the exact `wf`/`Step.number` binding names found
  during T-26A01 — rename this task's own scope note if they differ from the illustrative
  pseudocode, following the Phase-17/20/23/24/25 Track-C precedent: spec pseudocode is not
  binding on exact identifier names, but should be corrected to match once real code
  exists). Update `l2-nodus-error-dispatch.md`'s own header to Implemented/Done framing;
  `l2-nodus-runtime.md` §3's NL-9 row (already updated at spec-authoring time to describe
  the mechanism — confirm it still matches the as-built shape, correct if not);
  `l2-nodus-errors.md`'s NL-9 row (currently "dispatch specified, not yet implemented" —
  update to reflect the real, wired mechanism). `INDEX.md` rows + top-level version synced
  for every spec file actually touched. Self-review before declaring this task done — check
  all three specs for any remaining "not yet implemented"/"zero call sites" language beyond
  the sections already known to need updating.

### [T-26T01] New coverage: dispatch fires, `$error` populated, no-op cases, retry-then-succeed

- **Spec:** l2-nodus-error-dispatch.md §4.1, §4.2, §4.4, §4.5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase covering: (a) a
  `GEN` step decorated with `@err: ESCALATE(human)` run through `run_with_policy` against a
  deny-all `PolicyProvider` — assert `$error` contains the expected `code`/`step`/`reason`,
  the log contains `ESCALATE`'s own `StepStart`, and (for a multi-step fixture) a step
  declared after the denied one did **not** run; (b) the same shape for a `DIALOG_TIMEOUT`/
  `DIALOG_REJECTED` denial via `run_with_dialog` against a timing-out/rejecting
  `DialogProvider`; (c) a workflow with no `@err:` handler, denied the same way, behaves
  exactly as before this phase (no `$error` write — assert it's still the seeded empty
  map); (d) a workflow whose `@err:` text does not parse into a recognized command (e.g.
  free-text notes), denied the same way, also behaves unchanged; (e) a `~RETRY:n` step that
  fails once then succeeds never reaches dispatch (assert `$error` stays the seeded empty
  map and the handler's command never appears in the log) even though a `@err:` handler is
  declared.
  **Satisfied, with one correction to (d)**: grounded against `try_parse_command_from_string`
  directly rather than assumed — it returns `None` only for genuinely empty `@err:` text, so
  "free-text notes" was the wrong example (that parses into `Some(CommandCall)` and dispatches
  harmlessly as an `UNKNOWN_COMMAND`). Replaced with `empty_err_handler_is_unchanged` (an
  `@err:` line with no text at all — the real `handler: None` case) — §4.5 corrected to match
  in T-26C01. All 5 new tests (`err_handler_dispatches_on_policy_denial`,
  `err_handler_dispatches_on_dialog_denial`, `no_err_handler_declared_is_unchanged`,
  `empty_err_handler_is_unchanged`, `retry_then_succeed_never_dispatches_err_handler`) passed
  on first run.
- **Handoff:** T-26T02 reviews existing tests and runs the final gate.
- **Changes:** Added `ERR_HANDLER_WF`/`EMPTY_ERR_HANDLER_WF`/`NO_ERR_HANDLER_WF`/
  `RETRY_ERR_HANDLER_WF` fixtures, `DialogRejects` (`DialogProvider` returning
  `DialogOutcome::Rejected` unconditionally), `DenyOncePolicy` (`PolicyProvider` denying
  exactly the first `evaluate` call via an `AtomicUsize` counter, permitting every call
  after), and 5 integration tests to `tests/portability.rs`, placed after the existing
  LP-16 test section under a new `// ─── NL-9 uncaught-error handler dispatch ───` marker.
- **Notes:** Place unit-level helper tests (e.g. `error_to_value`) in `executor.rs`'s own
  `#[cfg(test)] mod tests` if a dedicated unit test is warranted; place the integration
  tests in a location matching where LP-11/LP-16 tests already live (`tests/portability.rs`
  has the existing `AllowAllPolicy`/`DenyAllPolicy`/dialog fixtures — reuse or extend rather
  than duplicating fixtures). New fixtures need a **multi-step** workflow with an `@err:`
  handler and a decorated first step, so "remaining steps did not run" is actually
  observable (the existing `MANIFEST_WF`/`DEFERRED_WF` fixtures may or may not already fit
  — check before assuming a new fixture is required).

### [T-26T02] Review and update existing Phase 24/25 tests; final gate run

- **Goal:** Confirm every existing test whose fixture already declares `@err:` and triggers
  a dispatchable error still asserts something true now that dispatch is live, and verify
  the phase against the project's mandatory gates.
- **Method:** Re-read `policy_denies_model_call_effect` and `policy_denies_deferred_effect`
  (`tests/portability.rs`, from Phase 24) — both use fixtures (`MANIFEST_WF`, `DEFERRED_WF`)
  that already declare `@err: ESCALATE(human)`. Confirm each test's existing assertions
  (`status == Partial`, pipeline target stays at seeded default, the `POLICY_DENIED` error
  present) still hold now that `ESCALATE` actually dispatches — if any assertion becomes
  false or vacuous, fix it rather than weakening it or deleting the test. Do the same for
  any Phase 25 test that triggers a denial on a fixture with `@err:` declared. Then:
  `cargo test -p nodus`; `cargo clippy -p nodus --all-targets -- -D warnings`; `cargo fmt -p
  nodus -- --check`; `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock`
  empty (LP-1); manual scan confirming no `unwrap()`/`panic!()`/`expect(` added outside
  `#[cfg(test)]`. Run cargo via PowerShell, not Git Bash.
- **Status:** Done — **found and fixed one real regression, not anticipated at spec time**:
  `tests/control_flow.rs`'s `retry_reruns_failing_step_up_to_bound` asserted "the step after
  an exhausted retry still runs" — true only in the absence of dispatch. `RETRY_TIMEOUT_WF`
  already declares `@err: ESCALATE(human)`, and `l1-nodus-language.md`'s own `~RETRY:n` row
  has always said exhaustion "routes to `@err:`", so the old assertion was pinning exactly
  the gap this phase closes. Corrected to assert the handler now dispatches (`ESCALATE` in
  the log) and step 2 (`LOG`) no longer runs, plus a new assertion on `$error`'s exact
  populated content. Reviewed Phase 24's `policy_denies_model_call_effect`/
  `policy_denies_deferred_effect` (`tests/portability.rs`) — both use fixtures that already
  declare `@err:`; confirmed their existing assertions (status, `$out` at seeded default,
  `POLICY_DENIED` present) still hold and are not vacuous now that dispatch is live
  (`ESCALATE` has no pipeline target so `$out` is untouched by it, and it is a
  zero-dependency stub that never itself pushes an error) — no changes needed there.
  `cargo test -p nodus`: 452 passed (was 447; +5), 0 failed. `cargo clippy -p nodus
  --all-targets -- -D warnings`: one `collapsible_if` finding in the new dispatch code,
  fixed via a let-chain. `cargo fmt -p nodus -- --check`: several line-wrap violations in
  the new tests, fixed (no logic change). `git diff --stat` on `Cargo.toml`/`Cargo.lock`:
  empty (LP-1 preserved). Manual scan: the one new `.expect()` on a production path
  (`ctx.errors.last().expect(...)`) is structurally unreachable-as-`None` — it runs only
  inside the branch that already proved `ctx.errors.len() > errors_before_this_step`, so
  the vec is provably non-empty; every other `unwrap`/`expect`/`panic!` in the touched
  files falls inside `#[cfg(test)]`.

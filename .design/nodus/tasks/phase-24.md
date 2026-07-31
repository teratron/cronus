---
phase: 24
name: "Per-Effect Authorization Call Site"
status: Todo
subsystem: "crates/nodus/src/portability.rs, executor.rs, workflows.rs"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 24 Tasks — Per-Effect Authorization Call Site

**Phase:** 24
**Status:** Todo
**Strategic Goal:** Convert `PolicyProvider` from a re-exported, zero-call-site trait into a real pre-effect gate over model-call and deferred effects, exactly as `l2-nodus-portability` §4.9 specifies.

## Scope note (read before starting)

`l2-nodus-portability` §4.9 (v1.4.0) is the authoritative design for this phase — every
task below cites the subsection it realizes. Nothing here should deviate from that design
without a documented reason; if implementation surfaces a gap the spec pass missed, record
it in Track C rather than silently improvising a different shape.

**Explicitly out of scope**, per §4.9.6:

- `EffectClass::ToolUse` — every tool-shaped builtin command (`FETCH`, `WRITE`, `GIT`,
  `NOTIFY`, …) is a fixed, zero-dependency stub with no host-swappable seam; gating one
  would authorize nothing real. Do not add a `ToolUse` variant or classify any command
  into it.
- `@err:` handler dispatch — `WorkflowFile.error_decl` has zero call sites in
  `executor.rs` today and building that dispatch is a separate, crate-wide NL-9
  undertaking. `POLICY_DENIED` surfaces via `RunResult.errors` only, exactly like every
  other non-fatal typed error in the crate.
- A DSL-level named-gate declaration syntax — `gate` is the effect-class string
  (`"model_call"` / `"deferred"`), not something a step declares.

## Guardrails (from the §4.9 design)

1. **Do not reuse `ConstraintHit`.** That variant's own doc comment
   (`observability.rs`) scopes it to hard `!!NEVER`/`!!ALWAYS` constraints, and
   `l2-nodus-errors.md` states the `RULE_VIOLATION`/`ConstraintHit` path is "unchanged"
   by any later spec. The denial event is the existing `StepError` variant with
   `error_code: NODUS:POLICY_DENIED` — no new `ExecutionEvent` variant.
2. **Do not return `Signal::Skip` or `Signal::Break` on denial.** `Signal::Skip` is the
   `?IF … !SKIP` branch flag; at the top level of `execute_inner`'s step loop it is
   caught by a bare `_ => {}`, identical to `None`, so it has no distinguishing effect
   there and using it would mislabel the mechanism. `Signal::Break` aborts the whole run
   (`Status::Aborted`) — wrong for a per-attempt gate. Return a bare `None`, mirroring
   `DialogOutcome::Timeout`/`::Rejected`'s exact shape (push a `RuntimeError`, return
   `None`, let `RunResult.status` degrade to `Partial` via the existing
   `!ctx.errors.is_empty()` branch).
3. **`context` carries raw, unresolved argument strings.** `CommandCall.args: Vec<String>`
   — do not resolve `$var` references before building the gate's `context`; that is
   `dispatch`'s job, not the gate's. Passing raw args keeps the seam a thin, honest
   pre-effect snapshot.
4. **Exactly two `EffectClass` variants.** Do not add `ToolUse` even as a placeholder
   variant with no classified commands — an unused variant with no test coverage would
   be dead code the moment it's written (`cargo clippy` should catch this if attempted).
5. **`policy` defaults to `NoopPolicyProvider`.** Every existing `Executor` constructor
   (`new`, `with_dialog`, etc.) must continue to produce allow-all behaviour — this is
   the seam's stated purity guarantee (L1 §4.7: "no `PolicyProvider` = built-in allow-all
   = today's behaviour"). A regression test in Track T must prove this explicitly, not
   just implicitly via existing tests still passing.

## Atomic Checklist

- [ ] [T-24A01] `EffectClass` + `effect_class_of` + `NODUS:POLICY_DENIED`
- [ ] [T-24A02] `Executor.policy` field + `with_policy`/`with_policy_and_audit`
- [ ] [T-24A03] Gate in `execute_command` + `run_with_policy`/`run_with_policy_and_audit`
- [ ] [T-24C01] Reconcile `l2-nodus-portability` to the as-built result
- [ ] [T-24T01] Classification, permit/deny, and no-op-regression coverage
- [ ] [T-24T02] Run the full gate set and confirm zero-dep

## Detailed Tracking

### [T-24A01] `EffectClass` + `effect_class_of` + `NODUS:POLICY_DENIED`

- **Spec:** l2-nodus-portability.md §4.9.1, §4.9.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; a new unit test in `portability.rs`'s
  `#[cfg(test)] mod tests` asserts `effect_class_of("GEN") == Some(EffectClass::ModelCall)`,
  `effect_class_of("ASK") == Some(EffectClass::Deferred)`, and
  `effect_class_of("LOG") == None`.
- **Handoff:** T-24A02 adds the `Executor` field this gate will call into; T-24A03 wires
  the actual call site.
- **Notes:** Add to `portability.rs`:
  ```
  pub enum EffectClass { ModelCall, Deferred }
  pub fn effect_class_of(command: &str) -> Option<EffectClass> {
      if MODEL_COMMANDS.contains(&command) { Some(EffectClass::ModelCall) }
      else if DIALOG_COMMANDS.contains(&command) { Some(EffectClass::Deferred) }
      else { None }
  }
  ```
  `MODEL_COMMANDS` and `DIALOG_COMMANDS` already exist in `portability.rs` (used by
  `CapabilityManifest::from_workflow`) — reuse them, do not redefine. `EffectClass` needs
  a way to render as the `gate` string (`"model_call"` / `"deferred"`) for T-24A03 — an
  inherent method or a `Display` impl, whichever fits the file's existing style better (a
  `From<EffectClass> for &'static str` inherent method is a reasonable default; check
  whether `ExtensionRole` nearby already established a convention for enum-to-string and
  match it rather than introduce a second style). Add
  `pub const POLICY_DENIED: &str = "NODUS:POLICY_DENIED";` to `vocab.rs` next to
  `DIALOG_TIMEOUT`/`DIALOG_REJECTED`, and register it in the severity/category table as
  `(Error, Runtime)` — same classification as `RULE_VIOLATION`, per §4.9.4. Add it to the
  crate's lockstep test (the one asserting every canonical constant has metadata) if that
  test iterates a fixed list rather than deriving it — check `vocab.rs`'s existing test
  module before assuming.

### [T-24A02] `Executor.policy` field + `with_policy`/`with_policy_and_audit`

- **Spec:** l2-nodus-portability.md §4.9.5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `Executor::new(...)`,
  `Executor::with_dialog(...)`, and every other existing constructor still compile
  unchanged (proving the new field has a default and doesn't force every call site to
  supply a policy).
- **Handoff:** T-24A03 is the only caller of `self.policy`.
- **Notes:** Add `policy: Box<dyn PolicyProvider>` as a fourth field on `Executor`
  (alongside `provider`, `audit`, `dialog`), defaulting to `NoopPolicyProvider` in every
  existing constructor. Add `with_policy(policy: impl PolicyProvider + 'static) -> Self`
  and `with_policy_and_audit(policy: impl PolicyProvider + 'static, audit: impl
  AuditProvider + 'static) -> Self`, mirroring `with_dialog`/`with_dialog_and_audit`'s
  exact shape (same defaulting pattern for the fields these constructors don't set).

### [T-24A03] Gate in `execute_command` + `run_with_policy`/`run_with_policy_and_audit`

- **Spec:** l2-nodus-portability.md §4.9.2, §4.9.3, §4.9.5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` — a workflow with a `GEN` step run through
  `run_with_policy` against a deny-all `PolicyProvider` produces `Status::Partial`, an
  empty `pipeline_target` binding, and `result.errors` containing one entry with
  `code == "NODUS:POLICY_DENIED"`; the same workflow run through plain `run` (no policy)
  still produces `Status::Ok` (Guardrail 5's regression).
- **Handoff:** T-24C01 reconciles the spec once this is real; T-24T01 adds the full
  coverage matrix.
- **Notes:** In `execute_command`, after the existing `check_rules` (`!!`-rule) block and
  before the `ASK`/`CONFIRM` dialog dispatch, insert:
  ```
  if let Some(class) = effect_class_of(&cmd.name) {
      let gate = class.as_gate_str();               // T-24A01's rendering
      let context = build_policy_context(cmd);        // Value::Map{command, args} — §4.9.2
      if !self.policy.evaluate(gate, &context) {
          self.emit(ctx, |seq, cid| ExecutionEvent::StepError { ... POLICY_DENIED ... });
          ctx.errors.push(RuntimeError { code: POLICY_DENIED, step: step_num, reason: ... });
          return None;
      }
  }
  ```
  Follow the phase file's Guardrails 1–3 exactly for the event shape, the return value,
  and the context's raw-args requirement. `step_identity`/`fault_identity` are already
  imported and used by the neighbouring `RULE_VIOLATION` block — reuse them, do not
  duplicate. For the combinators, copy `run_with_dialog`/`run_with_dialog_and_audit`
  (`workflows.rs`) verbatim and substitute `Executor::with_policy(...)` /
  `Executor::with_policy_and_audit(...)` for the dialog constructors — the parse →
  validate → construct-executor → execute shape is otherwise identical.

### [T-24C01] Reconcile `l2-nodus-portability` to the as-built result

- **Spec:** l2-nodus-portability.md §4.9 (all subsections), §3.1 (LP-11 row), §4.2, §4.4, §5 item 2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `node .magic/scripts/executor.js check-prerequisites --json --require-specs --verify-headers --workspace=nodus`
  reports no `VERSION_DRIFT` and the file header matches its `INDEX.md` row; `grep -n
  "zero call sites" .design/nodus/specifications/l2-nodus-portability.md` no longer
  matches the LP-11 row (the claim becomes false once T-24A03 lands).
- **Handoff:** Track T.
- **Notes:** Patch or minor bump depending on how much §4.9's `[REFERENCE]` pseudocode
  diverges from the real, compiled shape (rename this task's own scope note if
  `effect_class_of`'s actual signature, the `as_gate_str`-equivalent method name, or the
  context-building helper's name differ from what §4.9 sketched — spec pseudocode is not
  binding on exact identifier names, but should be corrected to match once real code
  exists, following the Phase-17/20/23 Track-C precedent). Update: §4.2's registry row
  (`Wired` column Storage/Policy → Policy now **Yes**), §4.4 (drop "Interface Shipped;
  Wiring Specified in §4.9" framing now that wiring exists), §3.1's LP-11 row (from
  "DESIGN COMPLETE, not yet built" to **Implemented**), and §5 item 2 (drop from the
  "Order of implementation" list or mark done, matching how Phase 23 closed out LP-15's
  built-in half in the same section). Self-review before declaring this task done — the
  LP-3/LP-15 reconciliation two cycles ago found two live references the first pass
  missed (§3's LP-2 row, §4.5's module map); check `l2-nodus-portability.md` for any
  remaining "zero call sites" / "not yet built" language beyond the sections listed above
  before finishing.

### [T-24T01] Classification, permit/deny, and no-op-regression coverage

- **Spec:** l2-nodus-portability.md §4.9.1, §4.9.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase covering: (a)
  `effect_class_of` for every `MODEL_COMMANDS`/`DIALOG_COMMANDS` entry plus at least one
  `None` case (`LOG` or similar); (b) a permitted `GEN` effect via `run_with_policy`
  against an allow-all `PolicyProvider` executes normally (`Status::Ok`, `$out` bound);
  (c) a denied `GEN` effect via `run_with_policy` against a deny-all `PolicyProvider`
  does not execute (`Status::Partial`, `POLICY_DENIED` in `result.errors`, target
  unbound); (d) the same permit/deny pair for an `ASK`/`CONFIRM` (`Deferred`) step; (e) a
  plain `run`/`run_with_provider` call with no policy behaves byte-for-byte as before
  this phase (Guardrail 5).
- **Handoff:** T-24T02 runs the full gate set.
- **Notes:** Place unit tests for `effect_class_of` in `portability.rs`'s own
  `#[cfg(test)] mod tests` (matching where `InMemoryStorageProvider`'s unit tests landed
  in Phase 23). Place the integration tests (b)–(e) in `tests/portability.rs`, following
  the file's existing `TestSchemaProvider`-style pattern: define a minimal
  `AllowAllPolicy`/`DenyAllPolicy` test fixture implementing `PolicyProvider`, reusing
  `MANIFEST_WF` or a similarly minimal fixture already in that file rather than
  authoring a new one if an existing `GEN`/`ASK`-bearing fixture already fits.

### [T-24T02] Run the full gate set and confirm zero-dep

- **Goal:** Verify the phase against spec and the project's mandatory gates.
- **Method:** `cargo test -p nodus`; `cargo clippy -p nodus --all-targets -- -D warnings`;
  `cargo fmt -p nodus -- --check`; `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock`
  empty (LP-1); manual scan confirming no `unwrap()`/`panic!()`/`expect(` added outside
  `#[cfg(test)]`. Run cargo via PowerShell, not Git Bash.
- **Status:** Todo

---
phase: 27
name: "Settlement Effect Seam"
status: Done
subsystem: "crates/nodus/src/portability.rs, crates/nodus/src/executor.rs, crates/nodus/src/workflows.rs, crates/nodus/src/vocab.rs"
requires: []
provides:
  - "LP-17 settlement effect seam — EffectClass::Settlement reuses the LP-11 gate unchanged for the decide half; SettlementRail/NoopSettlementRail + ExtensionRole::Settlement for the act half"
  - "handle_settlement — Signal-free on both denial and unaccounted exits, so NL-9 @err: dispatch covers SETTLE automatically"
  - "run_with_settlement / run_with_settlement_and_audit public combinators"
  - "l2-nodus-settlement.md 1.0.1 / l2-nodus-portability.md 1.7.1: LP-17 reconciled to Implemented"
key_files:
  created: []
  modified:
    - "crates/nodus/src/portability.rs"
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/src/workflows.rs"
    - "crates/nodus/src/vocab.rs"
    - "crates/nodus/src/lib.rs"
    - "crates/nodus/tests/portability.rs"
    - ".design/nodus/specifications/l2-nodus-settlement.md"
    - ".design/nodus/specifications/l2-nodus-portability.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "A spec authored directly against the real code in the same session can land with zero plan/implementation scope correction — contrast with Phase 24's first LP-11 build, which found real spec/code divergences"
  - "A non-reserved pipeline target (@out: $name other than $out) is NOT pre-seeded in RunResult.vars — only reserved variables (out/error/meta) are; a denial/unaccounted assertion against a custom target name must expect None, not Some(Value::Null)"
duration_minutes: ~
---

# Stage 27 Tasks — Settlement Effect Seam

**Phase:** 27
**Status:** Done
**Strategic Goal:** Build the LP-17 settlement effect seam exactly as `l2-nodus-settlement.md`
(v1.0.0) specifies: a new `SETTLE(payee, amount, purpose) → $target` command whose decide
half reuses the already-shipped LP-11 gate unchanged, and whose act half runs through a new
`SettlementRail` extension point.

## Scope note (read before starting)

`l2-nodus-settlement.md` is the authoritative design — every task below cites the subsection
it realizes. Ground every `[REFERENCE]` block against the real code before writing to it;
this spec was authored in the same session as this plan and has not yet been implementation-
checked.

**The decide half needs no new gate.** `execute_command`'s LP-11 block (`executor.rs:1558-
1603`) already gates *any* command `effect_class_of` recognizes — adding `Settlement` to that
function's classification is sufficient; the `if let Some(class) = effect_class_of(&cmd.name)`
block itself does not change. `context`'s existing `Value::Map` literal already carries
`"command"` and `"args"` (a `Value::List` of the raw, unresolved argument strings) — for
`SETTLE(payee, amount, purpose)` that list is `[payee, amount, purpose]` in order, so a
conforming host's `PolicyProvider::evaluate("settlement", context)` reads `context`'s `"args"`
list positionally. No `context` builder change beyond `Settlement` reaching the same code path
`ModelCall`/`Deferred` already do.

**The act half is genuinely new.** `PolicyProvider::evaluate` returns only `bool` — there is no
channel for a receipt. `SettlementRail` (§4.3) is the crate's first new extension-point trait
since `DialogProvider`/`EnvironmentProvider`; `handle_settlement` (§4.4) mirrors `handle_dialog`'s
shape (`StepStart`, an effect-call event, dispatch, `StepEnd`), binding `Some(value)` to the
pipeline target on success and pushing `NODUS:SETTLEMENT_UNACCOUNTED` on `None` — **not**
`RULE_VIOLATION`/`ConstraintHit`, and returning a bare `None` (not `Signal::Break`), so both the
gate-denial and unaccounted-settlement exits are `Signal`-free and reach NL-9's existing
structural `@err:` dispatch (`execute_inner`'s catch-all arm) with zero new dispatch code.

**Explicitly out of scope**, per the spec's own §4.6/§6:

- VS-6's payment-required retry handshake (negotiate → check envelope → gate → settle →
  retry) — a workflow-authoring concern expressible with existing `?IF`/`~RETRY:n`, not a new
  nodus control-flow primitive.
- LP-14 peer-payee resolution (LP-14 itself is vacuous-in-core).
- Any currency/amount parsing, validation, or comparison in the core — `amount` stays an
  opaque string end to end (LP-1/LP-2).
- Adding `Settlement` to `HostCapabilities::builtin()` — deliberately absent, matching the
  `Storage`/`Policy`/`Dialog` precedent, so a manifest-declaring workflow with no real rail is
  rejected pre-run rather than silently discovering unaccounted settlements per step.

## Atomic Checklist

- [x] [T-27A01] `EffectClass::Settlement` + `SETTLEMENT_COMMANDS` + `SETTLE` vocabulary + error code
- [x] [T-27B01] `SettlementRail` trait + `NoopSettlementRail` + `ExtensionRole::Settlement`
- [x] [T-27C01] Executor wiring — `Executor.settlement` field + constructors + `handle_settlement`
- [x] [T-27C02] `run_with_settlement` / `run_with_settlement_and_audit` combinators
- [x] [T-27T01] Validation coverage + spec reconciliation

## Detailed Tracking

### [T-27A01] `EffectClass::Settlement` + `SETTLEMENT_COMMANDS` + `SETTLE` vocabulary + error code

- **Spec:** l2-nodus-settlement.md §4.1
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `effect_class_of("SETTLE")` returns
  `Some(EffectClass::Settlement)` and `EffectClass::Settlement.as_gate_str()` returns
  `"settlement"` (unit test, mirroring the existing `effect_class_of`/gate-string tests for
  `ModelCall`/`Deferred`); `SETTLE` present in `KNOWN_COMMANDS`; `BUILTIN_SCHEMA_VERSION`
  bumped `"0.4.6"` → `"0.4.7"`.
  **Satisfied**: `effect_class_of_settlement_command` passes; `cargo check -p nodus` clean.
- **Handoff:** T-27B01/T-27C01 build on this classification and error code.
- **Changes:** `portability.rs`: `SETTLEMENT_COMMANDS: &[&str] = &["SETTLE"]` beside
  `MODEL_COMMANDS`/`DIALOG_COMMANDS`; `EffectClass` gained `Settlement` (`as_gate_str() ==
  "settlement"`); `effect_class_of` checks `SETTLEMENT_COMMANDS`. `vocab.rs`:
  `BUILTIN_SCHEMA_VERSION` "0.4.6" → "0.4.7"; `KNOWN_COMMANDS` gained `"SETTLE"` (53 → 54);
  `error_code::SETTLEMENT_UNACCOUNTED` + `error_meta` entry `(Error, Runtime)` beside
  `POLICY_DENIED`; lockstep test's canonical array/count updated (29 → 30).
- **Notes:** Confirm the exact real shape of `MODEL_COMMANDS`/`DIALOG_COMMANDS` (visibility,
  which module they live in — `portability.rs`, per `effect_class_of`'s own location) before
  adding `SETTLEMENT_COMMANDS: &[&str] = &["SETTLE"]` alongside them; reuse the identical
  `pub(crate)`-or-private visibility so `CapabilityManifest::from_workflow` (T-27B01) can also
  see it. `EffectClass` gains a third variant `Settlement`; `as_gate_str` gains a
  `Settlement => "settlement"` arm. Register `NODUS:SETTLEMENT_UNACCOUNTED` in `vocab.rs`
  beside the frozen 24-code set, `(Error, Runtime)` — same classification as `POLICY_DENIED`
  (confirm the exact registry shape — `ec::POLICY_DENIED => (Error, Runtime)` — and add the
  matching line for the new code, plus its `error_code` constant).

### [T-27B01] `SettlementRail` trait + `NoopSettlementRail` + `ExtensionRole::Settlement`

- **Spec:** l2-nodus-settlement.md §4.3, §4.5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; a unit test constructs `NoopSettlementRail` and
  confirms `.settle(&cmd)` returns `None` for an arbitrary `CommandCall`; a unit test confirms
  `CapabilityManifest::from_workflow` on an AST containing a `SETTLE` step includes
  `ExtensionRole::Settlement` in the derived manifest, and `HostCapabilities::builtin()`'s
  role set does **not** contain `Settlement` (mirroring the existing `Dialog`-absence
  assertion, if one exists — check `portability.rs`'s test module first).
  **Satisfied**: `noop_settlement_rail_never_settles`, `manifest_from_settle_workflow_requires_settlement`,
  `builtin_host_does_not_provide_settlement` all pass.
- **Handoff:** T-27C01 wires this trait into the executor.
- **Changes:** `portability.rs`: `SettlementRail` trait (`settle(&self, cmd: &CommandCall) ->
  Option<Value>`) + `NoopSettlementRail` (always `None`), added beside `PolicyProvider`/
  `NoopPolicyProvider`; `ExtensionRole` gained ninth variant `Settlement`;
  `CapabilityManifest::from_workflow` gained the `SETTLEMENT_COMMANDS` → `Settlement` role
  derivation arm; `HostCapabilities::builtin()`'s doc comment updated to name the deliberate
  absence (code itself unchanged — `builtin()` simply never inserts `Settlement`).
- **Notes:** `SettlementRail::settle(&self, cmd: &CommandCall) -> Option<Value>` — raw
  `&CommandCall`, not a typed wrapper (per §4.3/§6, matching `PolicyProvider::evaluate`'s raw-
  context precedent). `ExtensionRole` gains a ninth variant `Settlement`; confirm the real
  enum's current 8-variant list and ordering before appending. `CapabilityManifest::from_workflow`
  gains an `if SETTLEMENT_COMMANDS.contains(&name) { manifest.roles.insert(ExtensionRole::Settlement); }`
  arm — find the real equivalent arm for `MODEL_COMMANDS`/`DIALOG_COMMANDS` first and mirror
  its exact shape (`insert` vs a builder method — confirm `CapabilityManifest`'s real field/
  method names rather than assuming `.roles.insert`).

### [T-27C01] Executor wiring — `Executor.settlement` field + constructors + `handle_settlement`

- **Spec:** l2-nodus-settlement.md §4.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `cargo clippy -p nodus --all-targets -- -D
  warnings` clean; a temporary manual/integration check (formalized in T-27T01) confirms a
  `SETTLE` step whose `SettlementRail::settle` returns `Some(v)` binds `v` to the pipeline
  target with the ordinary `StepStart`/effect-call/`StepEnd` event triple; a `SETTLE` step
  whose rail returns `None` pushes `NODUS:SETTLEMENT_UNACCOUNTED` to `ctx.errors` and returns
  bare `None` (not `Signal::Break`); every existing `run_with_*` variant's behavior is
  unchanged byte-for-byte (the `NoopSettlementRail` default is inert).
  **Satisfied**: `settlement_permits_and_settles`, `settlement_unaccounted_when_rail_returns_none`,
  `no_settle_step_is_byte_for_byte_unchanged` (T-27T01) all pass; `cargo check`/`clippy` clean.
- **Handoff:** T-27C02 adds the public combinators; T-27T01 covers this + spec reconciliation.
- **Changes:** `Executor` gained a fifth field `settlement: Box<dyn SettlementRail>` (default
  `NoopSettlementRail`) across all 8 constructors, plus `with_settlement`/
  `with_settlement_and_audit`. `execute_command` gained a `cmd.name == "SETTLE"` dispatch
  branch immediately after the `ASK`/`CONFIRM` branch. New `handle_settlement`: `StepStart` →
  an effect-call event (reused `ExecutionEvent::ModelCall`, `input_summary` a length
  descriptor over the summed arg lengths, DG-7/§4.4 data-safety precedent — no new
  `ExecutionEvent` variant) → `self.settlement.settle(cmd)` → `Some(v)` binds via
  `ctx.log_step`/`ctx.set_var` (identical to `handle_dialog`'s `Answer` arm), `None` pushes
  `StepError` + `ctx.errors.push` with `SETTLEMENT_UNACCOUNTED` → `StepEnd` (elapsed timed via
  `Instant`, unlike dialog's `Measurement::Unavailable`, since settlement never suspends) →
  bare `None` return either way.
- **Notes:** Confirm `Executor`'s real constructor list (how many `with_*`/`with_*_and_audit`
  variants exist today, and their exact default-field pattern for `dialog`/`policy`/`environment`)
  before adding a fifth field `settlement: Box<dyn SettlementRail>` (default
  `Box::new(NoopSettlementRail)`) across all of them, plus `with_settlement`/
  `with_settlement_and_audit`. Add the `cmd.name == "SETTLE"` dispatch branch to
  `execute_command` immediately after the LP-11 gate block (`executor.rs:1603`), mirroring the
  `ASK`/`CONFIRM` → `handle_dialog` branch's exact position and shape. `handle_settlement`
  itself: `StepStart` → an effect-call event (confirm whether to reuse `ExecutionEvent::ModelCall`
  as `handle_dialog` does, or whether a settlement-specific reuse of an existing variant reads
  better — do not invent a new `ExecutionEvent` variant, HO-6) → `self.settlement.settle(cmd)`
  → bind-or-error → `StepEnd` → return `None` either way (terminal, non-halting).

### [T-27C02] `run_with_settlement` / `run_with_settlement_and_audit` combinators

- **Spec:** l2-nodus-settlement.md §4.6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; both functions are re-exported from `lib.rs`
  (confirm the real re-export list before adding); a smoke test calls each with a
  `NoopSettlementRail`-equivalent host rail and confirms a `Result<RunResult, Vec<Diagnostic>>`
  return matching the shape of `run_with_dialog`/`run_with_dialog_and_audit`.
  **Satisfied**: `settlement_permits_and_settles` exercises `run_with_settlement`;
  re-exported from `lib.rs` alongside the other `run_with_*` combinators.
- **Handoff:** T-27T01.
- **Changes:** `workflows.rs`: `run_with_settlement`/`run_with_settlement_and_audit` added
  immediately after `run_with_policy_and_audit`, mirroring its parse→validate→construct→
  execute shape exactly. `lib.rs`: both re-exported from `workflows::{..}`;
  `SettlementRail`/`NoopSettlementRail` re-exported from `portability::{..}`.
- **Notes:** Mirror `run_with_dialog`/`run_with_dialog_and_audit`'s exact signature shape in
  `workflows.rs` (parse → validate → construct `Executor::with_settlement[_and_audit]` →
  `execute`). Confirm the real `run_with_dialog_and_audit` parameter order (source, filename,
  input, provider, audit, run_id, started_at) before mirroring it.

### [T-27T01] Validation coverage + spec reconciliation

- **Spec:** l2-nodus-settlement.md (all sections); reconciles to `l2-nodus-portability.md`
  §3.1's LP-17 row, `l2-nodus-errors.md`'s `SETTLEMENT_UNACCOUNTED` row, `INDEX.md`
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase covering: (a) a
  `SETTLE` step permitted by a capturing `PolicyProvider` and settled by a rail returning
  `Some(receipt)` — assert the receipt binds to the pipeline target and the manifest-declared
  gate saw `context.args == [payee, amount, purpose]`; (b) the same step denied by a deny-all
  `PolicyProvider` — assert `NODUS:POLICY_DENIED`, pipeline target stays at its seeded
  default, and (if the fixture declares `@err:`) the handler dispatches, proving the `Signal`-
  free path reaches NL-9 dispatch with zero new code; (c) a permitted `SETTLE` whose rail
  returns `None` — assert `NODUS:SETTLEMENT_UNACCOUNTED` and the same NL-9 reachability; (d)
  `CapabilityManifest::from_workflow` + `run_with_manifest` against a host with no
  `Settlement` role rejects the workflow pre-run (`NODUS:CAPABILITY_UNMET`), never reaching
  `handle_settlement`; (e) a `NoopSettlementRail`/plain-`run` regression proving every
  existing `run_with_*` variant is unchanged byte-for-byte. `cargo clippy -p nodus
  --all-targets -- -D warnings` clean; `cargo fmt -p nodus -- --check` clean; `git diff
  --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` empty (LP-1); manual scan
  confirming no `unwrap()`/`panic!()`/`expect(` added outside `#[cfg(test)]`. Run cargo via
  PowerShell, not Git Bash.
  **Satisfied, with one correction found empirically**: the first fixture draft used
  `@out: $receipt` (non-reserved) and asserted `Some(Value::Null)` on denial/unaccounted; both
  failed — only reserved variables are pre-seeded, so an ordinary declared name starts absent
  from `vars`. Fixed by using the reserved `$out` binding throughout `SETTLE_WF`, matching
  every sibling fixture. All 10 new tests pass (4 unit + 6 integration); 462 tests total (was
  452); `cargo clippy -p nodus --all-targets -- -D warnings` clean; `cargo fmt -p nodus --
  --check` clean after one auto-fix (import line wrap); `git diff --stat` on
  `Cargo.toml`/`Cargo.lock` empty; the one new `.expect()` sits inside `#[cfg(test)]`
  (`manifest_from_settle_workflow_requires_settlement`).
- **Handoff:** Phase closure.
- **Changes:** `tests/portability.rs`: `SETTLE_WF` fixture, `CapturingSettlementRail`, and 6
  integration tests (`settlement_permits_and_settles`,
  `settlement_gate_receives_positional_args`, `settlement_denied_by_policy_never_settles`,
  `settlement_unaccounted_when_rail_returns_none`,
  `run_with_manifest_rejects_settle_without_settlement_role`,
  `no_settle_step_is_byte_for_byte_unchanged`); `portability.rs`'s own `#[cfg(test)]` module
  gained `manifest_from_settle_workflow_requires_settlement`. Reconciled
  `l2-nodus-settlement.md` 1.0.0 → 1.0.1 (§3 LP-17 row → Implemented, Document History) and
  `l2-nodus-portability.md` 1.7.0 → 1.7.1 (§3.1 LP-17 row → Implemented, Leverage paragraph
  now three-of-twelve, §5 item 8 → Done, Document History). `INDEX.md` rows for both specs +
  top-level version (1.0.78 → 1.0.79) + Last Updated synced.
- **Notes:** `[REFERENCE]` pseudocode in `l2-nodus-settlement.md` needed no correction beyond
  the usual illustrative elisions — the spec was authored directly against the real
  `execute_command`/`handle_dialog`/`Executor` code in the same session, so every structural
  claim held (contrast Phase 24's first LP-11 build, which found real divergences like
  `as_str` → `as_gate_str`).

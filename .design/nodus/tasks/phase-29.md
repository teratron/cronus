---
phase: 29
name: "Declared Budget Measure"
status: Todo
subsystem: "crates/nodus/src/environment.rs, crates/nodus/src/workflows.rs, crates/nodus/src/vocab.rs"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 29 Tasks — Declared Budget Measure

**Phase:** 29
**Status:** Todo
**Strategic Goal:** Build the NE-14 seam `l2-nodus-environment.md` §4.4.1 designed:
`EnvironmentProfile.token_measure` plus a fail-fast rejection when a declared token budget
has no identified encoder.

## Scope note (read before starting)

`l2-nodus-environment.md` §4.4.1 is the design source, but its stated mechanism does **not**
match the real code — verified directly against `crates/nodus/src/workflows.rs:777-859`
during planning, not assumed from the spec text. Build against **this** section, not the
spec's own wording; the spec is queued for a patch-level correction at the next
`/magic.spec` touch (T-29C02 below records the exact diff).

**The real rejection shape.** `run_with_environment_impl` already has an
`ExtensionRole::Environment` manifest-rejection precedent at `workflows.rs:806-814`:

```
let missing = validate_manifest(&manifest, host);
if !missing.is_empty() {
    return Ok(EnvRunResult {
        result: capability_rejection(&ast, &missing),
        reward: Reward::no_op(),
        budget_halted: false,
    });
}
```

`capability_rejection` (`workflows.rs:1056-1083`) builds an ordinary `RunResult { status:
Failed, errors: vec![RuntimeError { code: CAPABILITY_UNMET, .. }], .. }` — **never** an
`Err(Vec<Diagnostic>)`. `Diagnostic` is reserved for the parse/validate failures at the top of
the same function (`workflows.rs:789-803`) and is not the right shape for this check. NE-14's
own rejection must mirror `capability_rejection`'s pattern: a `RunResult { status: Failed,
errors: vec![RuntimeError { code: ENV_MEASURE_UNKNOWN, .. }] }`, wrapped in
`Ok(EnvRunResult { result: .., reward: Reward::no_op(), budget_halted: false })`.

**The real insertion point.** `env.profile()` — the only source of `budget`/`token_measure` —
is not called until `workflows.rs:828`, which is **after** `env.open` (`:818`) and `env.reset`
(`:821`) already ran (both are needed for the frozen-boundary reset-observation shape this
function establishes before the workflow itself executes). This phase's check therefore
inserts **immediately after line 828** (`let profile = env.profile();`) and **before line 839**
(`executor.execute_for_environment(..)`, the actual workflow run) — no workflow step has
executed yet, which is what "never mid-run" means here, even though the `Instance` is
technically already open+reset. `guard`'s `Drop` impl still releases it correctly on this
early return (NE-7 is unaffected — `release` is idempotent and unconditional via the guard).

**Explicitly out of scope:**

- A new `ExtensionRole::TokenMeasure` — rejected in the spec's own §5; the condition is
  profile-shaped (does this specific budget have an identified measure), not role-shaped.
- Defaulting `token_measure` to a built-in encoder identity — rejected (the invariant's
  central rule): an unidentifiable measure must fail loudly, never estimate.
- Any change to `Budget`, `max_steps`, or `wall_clock_ms` handling — untouched; a profile
  whose budget has no `max_tokens` carries no measure and is unaffected.

## Atomic Checklist

- [ ] [T-29A01] `EnvironmentProfile.token_measure` + `CandidateResult.token_measure`
- [ ] [T-29C01] `NODUS:ENV_MEASURE_UNKNOWN` + pre-run check wired into `run_with_environment_impl`
- [ ] [T-29T01] Validation coverage + spec reconciliation

## Detailed Tracking

### [T-29A01] `EnvironmentProfile.token_measure` + `CandidateResult.token_measure`

- **Spec:** l2-nodus-environment.md §4.4 (struct shapes), §4.4.1 (rule)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `EnvironmentProfile` and `CandidateResult`
  (`environment.rs:178`, `:317`) each gain a `token_measure: Option<String>` field; existing
  construction sites (`StubEnvironment::profile()`, `EnvRunResult::candidate()` or wherever
  `CandidateResult` is built) compile with `None` for the new field with no other behavior
  change.
- **Handoff:** T-29C01 reads `token_measure` in the new check.
- **Notes:** Read `environment.rs:166-220` and `:317-360` (the real `Budget`/
  `EnvironmentProfile`/`CandidateResult` definitions) before editing — confirm exact field
  order and any `#[derive(..)]` list the new field must stay compatible with (e.g. if
  `EnvironmentProfile` derives `PartialEq`/`Clone`, `Option<String>` is trivially compatible;
  if it derives something stricter, check). `token_measure` is opaque to the crate — no
  encoder/tokenizer type, matching `Budget`'s own primitive-only fields (LP-1/LP-2).

### [T-29C01] `NODUS:ENV_MEASURE_UNKNOWN` + pre-run check wired into `run_with_environment_impl`

- **Spec:** l2-nodus-environment.md §4.4.1 — **use this phase file's Scope note for the real
  mechanism, not the spec's own "Diagnostic"/"before env.open" wording** (T-29C02 corrects
  the spec after this lands)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; `cargo clippy -p nodus --all-targets -- -D
  warnings` clean; a temporary manual/integration check (formalized in T-29T01) confirms: a
  `StubEnvironment`-like test double whose `profile()` returns `budget: Some(Budget{
  max_tokens: Some(_), .. })` and `token_measure: None` causes `run_with_environment`/
  `run_with_environment_and_audit` to return `Ok(EnvRunResult{ result: RunResult{ status:
  Failed, errors: [RuntimeError{ code: "NODUS:ENV_MEASURE_UNKNOWN", .. }], .. }, .. })`, and
  that the test double's `step`/`evaluate` methods are **never called** (proving the
  rejection happens before the workflow executes, per the corrected insertion point).
- **Handoff:** T-29T01.
- **Notes:** Register `NODUS:ENV_MEASURE_UNKNOWN` in `vocab.rs` beside `CAPABILITY_UNMET`,
  `(Error, Control)` — confirm the real registry line for `CAPABILITY_UNMET` first and mirror
  its classification exactly. Insert the check in `run_with_environment_impl`
  (`workflows.rs`) immediately after `let profile = env.profile();` (currently line 828) and
  before the executor construction/`execute_for_environment` call (currently line 835-847):
  `if profile.budget.as_ref().and_then(|b| b.max_tokens).is_some() && profile.token_measure.is_none()`
  → build a `RunResult{status: Failed, errors: vec![RuntimeError{code: ENV_MEASURE_UNKNOWN, step: 0, reason: ..}], ..}`
  (mirror `capability_rejection`'s field-by-field shape, or factor a small sibling helper
  function next to it if that reads cleaner) and `return Ok(EnvRunResult{result: .., reward: Reward::no_op(), budget_halted: false});`.
  This must be duplicated (or factored into a shared helper) for **both**
  `run_with_environment` and `run_with_environment_and_audit` if they do not already share one
  `_impl` function — confirm whether both public functions delegate to the single
  `run_with_environment_impl` this scope note quotes, or whether the audit variant has its
  own separate body.

### [T-29T01] Validation coverage + spec reconciliation

- **Spec:** l2-nodus-environment.md (all sections); reconciles to `l2-nodus-errors.md`'s
  `ENV_MEASURE_UNKNOWN` row, `INDEX.md`
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase covering: (a) a
  profile with `max_tokens: Some(_)` and `token_measure: None` rejects pre-run as specified
  in T-29C01's Verify line; (b) the same profile with `token_measure: Some("gpt-tokenizer-v1")`
  (or similar) runs normally to completion; (c) a profile with `max_tokens: None` and
  `token_measure: None` is unaffected (regression — today's `StubEnvironment`/existing
  environment tests behave byte-for-byte unchanged); (d) `EnvRunResult::candidate()`'s
  `CandidateResult.token_measure` carries the profile's value through when a run completes
  normally. `cargo clippy -p nodus --all-targets -- -D warnings` clean; `cargo fmt -p nodus --
  --check` clean; `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` empty
  (LP-1); manual scan confirming no `unwrap()`/`panic!()`/`expect(` added outside
  `#[cfg(test)]`. Run cargo via PowerShell, not Git Bash.
- **Handoff:** Phase closure.
- **Notes:** **This task also reconciles `l2-nodus-environment.md` §4.4.1 itself** (spec
  bodies are `/magic.spec`'s write domain in general, but per the Phase-17/20/23/24/25/26
  Track-C precedent, correcting a design section to match its own just-landed as-built result
  is the same kind of reconciliation those phases already did — not a new design decision).
  Correct: (1) "the same pre-run `Result<_, Vec<Diagnostic>>` channel LP-8's `validate_manifest`
  already uses" → an `Ok(EnvRunResult{result: RunResult{status: Failed, errors: [RuntimeError{..}]}, ..})`,
  mirroring `capability_rejection`'s exact shape; (2) "before `env.open`" → "after
  `env.profile()`, before the workflow's own steps execute (`env.open`/`env.reset` already ran
  for the frozen-boundary reset-observation shape)". Bump `l2-nodus-environment.md` 1.1.0 →
  1.1.1 (patch — text-only correction to match an already-correct design intent, no rule
  changes). `INDEX.md` row + top-level version synced.

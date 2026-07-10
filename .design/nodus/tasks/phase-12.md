---
phase: 12
name: "Environment & Evaluation (l2-nodus-environment)"
status: Todo
subsystem: "crates/nodus"
requires:
  - phase-4
  - phase-7
  - phase-10
provides: []
key_files: []
patterns_established: []
duration_minutes: 0
---

# Phase 12 — Environment & Evaluation (l2-nodus-environment)

Implement the Environment/Evaluation contract from `l1-nodus-environment.md`
(Stable) in `crates/nodus`, per `l2-nodus-environment.md`. Adds the 5th extension
role — a graded-run substrate the host's evaluation-driven improvement loop
consumes. **All-additive, pure in-tree**: no new external dependency (LP-1 —
the NE-12 content address is a `std` digest, not a crypto crate), no new `Status`
(a `step` reuses the dialog suspend/resume shape from Phase 10), no new command,
no new error category beyond reusing `NODUS:CAPABILITY_UNMET`. The trajectory
rides existing audit events as an **optional side-band descriptor** — HO-6's
closed `ExecutionEvent` taxonomy is preserved (the HO-8/HO-13 additive-field
discipline). Sequential; Track A first as everything references the trait.

**Specs covered**: `l2-nodus-environment.md` (Stable). **Builds on**: `observability.rs`
(AuditProvider/ExecutionEvent/FieldDescriptor/RunManifest, Phase 4), `portability.rs`
(ExtensionRole/CapabilityManifest/HostCapabilities, Phases 5/7), the dialog
suspend/resume shape (Phase 10).

## Track A — Provider trait, StubEnvironment, lifecycle (NE-1/NE-2/NE-7)

- [ ] **T-12A01** — New `environment.rs` module: `EnvironmentProvider` trait (`task_ids`/`profile`/`open`/`reset`/`step`/`evaluate`/`release`), the `TaskId`/`Seed`/`Observation`/`Action`/`Instance` types (Observation/Action opaque over `Value`), and the built-in `StubEnvironment` (task_ids `["__stub__"]`, empty profile, `step` echoes the action, `evaluate` → no-op `Reward { score: None, metadata: {} }`). Mirrors the `DefaultDialogProvider`/`StubProvider` convention.
  - **Verify**: unit tests — `StubEnvironment::task_ids()` = `["__stub__"]`; `step` echoes its action; `evaluate` returns a `score: None` reward (NE-9)
- [ ] **T-12A02** — Lifecycle correctness: deterministic `reset`/`step` per `(task, seed, actions)`; `open` returns a fresh isolated `Instance`; `release` mandatory and **idempotent** (a second `release` is a no-op, never a panic).
  - **Verify**: unit tests — identical `(task, seed, action-sequence)` yields byte-identical observations across two runs (NE-2); a double `release` on one instance is a no-op (NE-7)

## Track B — Trajectory side-band projection (NE-3/NE-5)

- [ ] **T-12B01** — `Reward { score: Option<f64>, metadata }`, `EnvRole { Policy, Environment, Context, Summary }`, `EnvKind { Reset, Step, Evaluate }`, and `EnvInteraction` (seq reusing the HO-7 run sequence; `action`/`observation` carried as `observability::FieldDescriptor` only — never raw `Action`/`Observation`; `reward` on Evaluate; `step_index` back-pointer). Attach `EnvInteraction` as an **optional field on existing events** — `step` → the workflow step's `StepStart`/`StepEnd`, `reset` → the run-open boundary, `evaluate` → the terminal `RunManifest` — adding **no new `ExecutionEvent` variant**.
  - **Verify**: integration test — the `AuditProvider` stream carries the `reset`/`step`/`evaluate` `EnvInteraction` descriptors in run-sequence order; assert the `ExecutionEvent` enum variant count is unchanged (HO-6 preserved); assert no descriptor field holds raw action/observation content (FieldDescriptor only)

## Track C — Capability-manifest role (NE-10)

- [ ] **T-12C01** — `portability.rs`: add `ExtensionRole::Environment`; `HostCapabilities::builtin()` **inserts** `Environment` (the `StubEnvironment` satisfies it — deliberate contrast with `Dialog`, which `builtin()` omits); `CapabilityManifest::from_workflow` adds `Environment` when a workflow declares an environment need; `validate_manifest` rejects fail-fast reusing `NODUS:CAPABILITY_UNMET`.
  - **Verify**: unit tests — `builtin().provides(Environment)` is true (and `provides(Dialog)` stays false); a `HostCapabilities` constructed without `Environment` makes `validate_manifest` return `Missing::Role(Environment)` for an environment-declaring manifest (NE-10)

## Track D — Run combinator, frozen boundary, profile/grading/budget/candidate (NE-4/5/6/11/13/12)

- [ ] **T-12D01** — `workflows.rs`: `run_with_environment` / `run_with_environment_and_audit` combinators sequencing `open → reset → execute → FREEZE → evaluate → release` (release via a drop guard so it always runs); `EnvRunResult { result: RunResult, reward: Reward, budget_halted: bool }` — reward returned **alongside**, never bound into `RunResult.vars`. `evaluate` receives an immutable frozen view and cannot re-enter `execute`.
  - **Verify**: integration tests — `evaluate` observes the run only after it reaches a terminal state; two `evaluate` calls over one frozen instance return the same `Reward` (NE-4); the reward is absent from `RunResult.vars` (NE-5); `release` is invoked even when `execute` errors (drop guard, NE-7)
- [ ] **T-12D02** — `EnvironmentProfile { labels, grading: GradingMode, budget: Option<Budget> }` (NE-6 orthogonal slice labels); `GradingMode { Automated, Judge, Hybrid }` with the **hybrid floor** — the deterministic checker runs first, and on a checker failure the judge cannot rescue; otherwise the result is `min(checker, judge)`. Judge scoring resolves through a function-scoped auxiliary `ModelProvider` binding (NE-8), not the policy model.
  - **Verify**: unit tests — hybrid with a failing checker returns the checker's (failed) reward regardless of the judge (judge cannot lift, NE-11); hybrid with a passing checker and a lower judge returns the judge's lower score; automated mode never calls the judge
- [ ] **T-12D03** — `Budget { wall_clock_ms, max_steps, max_tokens }` (any subset) on the profile; `run_with_environment` halts the run **uniformly** when any declared limit is reached. A budget halt is a **normal outcome** — `Status::Partial` with `budget_halted = true`, **not** an error — and `evaluate` runs over what was achieved. The workflow cannot read, extend, or evade the budget (enforced by the run loop, LP-10 kinship).
  - **Verify**: integration test — a workflow that would run past `max_steps` halts at the budget with `Status::Partial` and `budget_halted == true` (not `Failed`), and `evaluate` still produces a reward over the partial run (NE-13); a profile with no budget is unaffected
- [ ] **T-12D04** — `CandidateResult { workflow_digest, reward, trajectory_ref, budget }` from `EnvRunResult::candidate()`; `workflow_digest` is a deterministic `std`-library digest over the canonical workflow source (zero-dep). Record the profile's `budget` in the tuple (rewards under different budgets are not comparable, NE-13).
  - **Verify**: unit tests — the same workflow source yields the same `workflow_digest`, a changed source yields a different one, and the `budget` is carried on the tuple (NE-12). Note: cross-version/cross-platform digest *stability* for durable archival is a host concern (compute a cryptographic address over the exposed source, LP-2) — documented on the task, not implemented in-core

## Track T — Gates

- [ ] **T-12T01** — NE-1…NE-13 integration suite in `crates/nodus/tests/environment.rs` (deterministic replay, frozen-boundary immutability, idempotent release, manifest fail-fast, hybrid floor, budget halt, candidate digest) + full quality gates.
  - **Verify**: `cargo test -p nodus` full suite green (≥ current 265 + the new environment tests); `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt` clean; `cargo doc --no-deps` no new warnings; SDD §6 clean; **extraction audit still zero external deps** (no `ron`/crypto crate added — Environment is pure in-tree, LP-1)

## Status

**Status:** Todo — 9 atomic tasks across tracks A→D + gates (Sequential). All-additive
over the existing provider/audit/manifest seams; the correctness risk concentrates in
Track D (the frozen-boundary ordering and the always-run `release` drop guard).

## Notes

Track A defines the trait every later track references, so it lands first. Track B's
trajectory projection is the one non-obvious design point: it rides existing events as
an optional descriptor rather than a new `ExecutionEvent` variant, so `reset`/`evaluate`
(which fall outside the step loop) attach to the run-open boundary and the terminal
`RunManifest` respectively. The `ExtensionRole::Environment` `builtin()`-provides-it
choice (Track C) is deliberate and differs from `Dialog` — the stub is a complete
trivial world, so a manifest-declaring workflow stays runnable in-process. No new
dependency, `Status`, command, or error category is introduced.

---
phase: 12
name: "Environment & Evaluation (l2-nodus-environment)"
status: Done
subsystem: "crates/nodus"
requires:
  - phase-4
  - phase-7
  - phase-10
provides:
  - EnvironmentProvider trait (task_ids/profile/open/reset/step/evaluate/release) + StubEnvironment
  - Reward, EnvironmentProfile, GradingMode + grade() hybrid-floor composition
  - Budget (max_steps/wall_clock_ms enforced; max_tokens declared, unenforced)
  - CandidateResult + EnvRunResult::candidate() (std-digest, zero-dep)
  - ExtensionRole::Environment (builtin() provides it via StubEnvironment)
  - run_with_environment / run_with_environment_and_audit public API
  - EnvInteraction/EnvInteractionKind trajectory side-band on RunManifest (NE-3)
key_files:
  created:
    - crates/nodus/src/environment.rs
    - crates/nodus/tests/environment.rs
  modified:
    - crates/nodus/src/observability.rs
    - crates/nodus/src/executor.rs
    - crates/nodus/src/portability.rs
    - crates/nodus/src/workflows.rs
    - crates/nodus/src/lib.rs
patterns_established:
  - "Side-band-on-existing-events over new enum variants: extend a closed taxonomy (HO-6) via an optional struct field on the carrying record, never a new variant — reused from the HO-8/HO-13 precedent, now proven end-to-end in code."
  - "Explicit host-declared pass/fail over inferred numeric thresholds: when nodus core is metric-neutral (NE-9), a composition function takes the verdict as a parameter rather than inferring it from a score it doesn't own the scale of."
  - "InstanceGuard (Option<T> + Drop::take()) for mandatory-and-idempotent release of a by-value-consumed resource, guaranteeing single release even on a panic."
duration_minutes: 45
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

- [x] **T-12A01** — New `environment.rs` module: `EnvironmentProvider` trait (`task_ids`/`profile`/`open`/`reset`/`step`/`evaluate`/`release`), the `TaskId`/`Seed`/`Observation`/`Action`/`Instance` types (Observation/Action opaque over `Value`), and the built-in `StubEnvironment` (task_ids `["__stub__"]`, empty profile, `step` echoes the action, `evaluate` → no-op `Reward { score: None, metadata: {} }`). Mirrors the `DefaultDialogProvider`/`StubProvider` convention.
  - **Verify**: unit tests — `StubEnvironment::task_ids()` = `["__stub__"]`; `step` echoes its action; `evaluate` returns a `score: None` reward (NE-9)
  - **Changes**: `crates/nodus/src/environment.rs` created. `Instance` is a plain `{ task, seed }` struct (not `Box<dyn Any>`) — `StubEnvironment` needs no internal state, so a state-carrying instance is deferred until a real environment needs it. `cargo test -p nodus environment::` → 3/3 pass.
- [x] **T-12A02** — Lifecycle correctness: deterministic `reset`/`step` per `(task, seed, actions)`; `open` returns a fresh isolated `Instance`; `release` mandatory and **idempotent** (a second `release` is a no-op, never a panic).
  - **Verify**: unit tests — identical `(task, seed, action-sequence)` yields byte-identical observations across two runs (NE-2); a double `release` on one instance is a no-op (NE-7)
  - **Changes**: 4 unit tests added (determinism, double-release, isolation). Determinism holds trivially for the stub since `step` echoes `action` alone (a pure function, no history dependency).

## Track B — Trajectory side-band projection (NE-3/NE-5)

- [x] **T-12B01** — `EnvInteraction`/`EnvInteractionKind` added to `observability.rs` (not `environment.rs` — avoids a circular module dependency, since `RunManifest` needed the type and `environment.rs` already depends on `observability::FieldDescriptor`). `EnvInteraction { kind, observation: Option<FieldDescriptor>, action: Option<FieldDescriptor> }` attached via a new `RunManifest.env_trajectory: Vec<EnvInteraction>` field (empty for every non-environment run) — **no new `ExecutionEvent` variant**.
  - **Verify**: integration test — the trajectory carries the `reset` entry; `ExecutionEvent`'s variant count is unchanged; no descriptor holds raw content
  - **Changes**: **Design refinement vs. plan** — `EnvKind`/`EnvInteractionKind` covers `{ Reset, Step }` only, not `Evaluate`. Reason: `evaluate` runs strictly after `execute_inner`'s single `run_complete` call already fired (NE-4's frozen-boundary ordering — evaluate is *after* the run, not part of it), so it structurally cannot ride the same manifest delivery. Its outcome is instead delivered directly as `EnvRunResult.reward` (NE-5's own "returned alongside" requirement) rather than duplicated into a side-channel — a cleaner satisfaction of NE-3/NE-5 together, not a scope cut. `step` entries are similarly not emitted automatically (v1 combinator does not call `env.step()`, see Track D note). `crates/nodus/src/executor.rs` `execute_inner` extended with `env_trajectory: Vec<EnvInteraction>` param, threaded through to `RunManifest`. Two existing `RunManifest` test literals (`observability.rs`) updated for the new field. `cargo test -p nodus --lib` → 198/198 pass.

## Track C — Capability-manifest role (NE-10)

- [x] **T-12C01** — `portability.rs`: `ExtensionRole::Environment` added; `HostCapabilities::builtin()` **inserts** `Environment` (the `StubEnvironment` satisfies it — deliberate contrast with `Dialog`, which `builtin()` omits); `from_workflow`'s doc comment records that `Environment` (like `Storage`/`Policy`) has no AST-derivable command syntax and must be required explicitly — `run_with_environment` does this on every call via `.require_role(Environment)`, since calling it is itself the need declaration.
  - **Verify**: unit tests — `builtin().provides(Environment)` true, `provides(Dialog)` stays false; `validate_manifest` returns `Missing::Role(Environment)` when absent (NE-10)
  - **Changes**: 1 new unit test (`builtin_host_provides_environment_but_not_dialog`) added to `portability.rs`; no existing test needed updating (the prior `builtin_host_provides_model_audit_vocabulary` test only asserts Storage/Policy negatively). `cargo test -p nodus --lib portability::` → all pass.

## Track D — Run combinator, frozen boundary, profile/grading/budget/candidate (NE-4/5/6/11/13/12)

- [x] **T-12D01** — `workflows.rs`: `run_with_environment` / `run_with_environment_and_audit` sequencing `open → reset → execute → FREEZE → evaluate → release`; `InstanceGuard` (Drop-based, `Option<Instance>` + `take()`) guarantees `release` fires exactly once even on a panic inside `evaluate`. `EnvRunResult { result: RunResult, reward: Reward, budget_halted: bool }`.
  - **Verify**: integration tests — evaluate strictly after execute; two evaluates over one frozen instance agree; reward absent from `RunResult.vars`; release always runs
  - **Changes**: **v1 scope decision, executed as designed** — the combinator does not call `env.step()` mid-run (no nodus DSL syntax binds a workflow step to an environment action yet; the trait method is complete and independently unit-tested). The reset `Observation` seeds `$in.observation` via a new private `merge_observation` helper. `Executor::with_boxed_audit` added (takes an already-boxed `AuditProvider`) so the plain/audited entry points share one `run_with_environment_impl`. `crates/nodus/tests/environment.rs` created — 10 integration tests, all pass.
- [x] **T-12D02** — `EnvironmentProfile { labels, grading: GradingMode, budget: Option<Budget> }`; `GradingMode { Automated, Judge, Hybrid }`; public `grade(mode, checker, checker_passed, judge) -> Reward` composition function with the hybrid floor (checker-first, judge lowers-never-rescues).
  - **Verify**: unit tests — hybrid-fail-not-rescued, hybrid-lower-wins, automated-ignores-judge, judge-ignores-checker
  - **Changes**: **Design refinement vs. plan** — `grade()` takes an explicit `checker_passed: bool` parameter rather than inferring pass/fail from `checker.score` (e.g. `score <= 0.0`). Reason: NE-9 metric neutrality — nodus owns no scoring semantics, so it cannot invent a numeric pass/fail threshold a host's checker might use a completely different scale for. This was a genuine correction caught during implementation, not present in the original spec pseudocode's `a.failed` shorthand. 7 unit tests added; 1 integration test (`hybrid_floor_end_to_end`) exercises the same path.
- [x] **T-12D03** — `Budget { wall_clock_ms, max_steps, max_tokens }`; `execute_inner`'s step loop checks `max_steps`/`wall_clock_ms` each iteration and halts with `budget_halted = true` → `Status::Partial` (reusing the existing status, not a new one).
  - **Verify**: integration test — a 4-step workflow with `max_steps: 2` halts at exactly 2 logged steps, `Partial`, `budget_halted == true`, and still produces a reward; no-budget behaves as today
  - **Changes**: `max_tokens` is accepted on `Budget` and carried on `CandidateResult` (profile-identity completeness) but **not enforced** — no token-accounting seam exists on `ModelProvider` (documented gap, same "interface declared, host integration pending" precedent as `StorageProvider`/`PolicyProvider`). `execute`/`execute_with_params` pass `None, None` for the two new params — zero behavior change for every existing caller (confirmed by all 198 pre-existing lib tests + 61 integration tests staying green unmodified). 2 new integration tests.
- [x] **T-12D04** — `CandidateResult { workflow_digest, reward, trajectory_ref, budget }` from `EnvRunResult::candidate(source, run_id, budget)`; `workflow_digest` via `std::hash::DefaultHasher` over the raw source (zero-dep, SipHash).
  - **Verify**: unit + integration tests — same source → same digest, changed source → different digest, budget/run_id carried through
  - **Changes**: 3 unit tests + 1 integration test. Doc comment states the caveat directly: not guaranteed stable cross-Rust-version/platform for durable archival — a host needing that computes its own cryptographic digest over the exposed source (LP-2), matching the LP-9 attestation precedent.

## Track T — Gates

- [x] **T-12T01** — `crates/nodus/tests/environment.rs`: 10 integration tests (deterministic replay, frozen-boundary ordering via an instrumented provider, NE-10 fail-fast with zero `env.open` calls on rejection, hybrid floor, budget halt + no-budget control, candidate digest, guaranteed single release) + full quality gates.
  - **Verify**: `cargo test -p nodus` full suite green; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean; `cargo doc --no-deps` no new warnings; zero new dependency; downstream `cronus-cli` (the only other in-workspace nodus consumer) still compiles
  - **Changes**: Evidence — `cargo test -p nodus`: 9 binaries, `198+7+7+10+17+4+34+7+7+1 = 292` passed, 0 failed (was 265 pre-phase). `cargo clippy -p nodus --all-targets -- -D warnings`: exit 0, 0 warnings (2 lints caught and fixed during implementation: `unnecessary_literal_unwrap` in a test, `arc_with_non_send_sync` — swapped `Arc<RefCell<_>>` for `Rc<RefCell<_>>` in a single-threaded test). `cargo fmt -p nodus -- --check`: exit 0. `cargo doc -p nodus --no-deps`: exit 0, 0 warnings. `crates/nodus/Cargo.toml [dependencies]` unchanged (still empty — LP-1 zero-dep preserved). `cargo check -p cronus-cli`: exit 0 (workspace-wide, confirms no downstream breakage).

## Status

**Status:** Done — all 9 atomic tasks across tracks A→D + gates landed (Sequential).
All-additive over the existing provider/audit/manifest seams; the frozen-boundary
ordering and the always-run `release` drop guard (Track D) both verified directly by
integration tests. Two honest design refinements surfaced during implementation (see
Track B and T-12D02 Changes) — both strengthen spec compliance rather than cut scope.

## Notes

Track A defines the trait every later track references, so it lands first. Track B's
trajectory projection is the one non-obvious design point: it rides existing events as
an optional descriptor rather than a new `ExecutionEvent` variant, so `reset` (which
falls outside the step loop) attaches to the run-open boundary; `evaluate` is delivered
directly as `EnvRunResult.reward` instead of duplicated into the trajectory, since it
structurally happens after the audit contract's one `run_complete` call already fired.
The `ExtensionRole::Environment` `builtin()`-provides-it choice (Track C) is deliberate
and differs from `Dialog` — the stub is a complete trivial world, so a
manifest-declaring workflow stays runnable in-process. No new dependency, `Status`,
command, or error category was introduced; `crates/nodus/Cargo.toml` still declares
zero dependencies.

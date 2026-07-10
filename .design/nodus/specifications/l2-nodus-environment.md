# Nodus Environment and Evaluation Implementation (Rust)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-environment.md

## Overview

Concrete Rust realization of the Environment/Evaluation contract in `crates/nodus`. It maps each `NE`-invariant to its enforcing mechanism: an `EnvironmentProvider` trait following the established `ModelProvider`/`DialogProvider` pattern, a built-in deterministic `StubEnvironment`, the closed `open/reset/step/evaluate/release` lifecycle, a typed `Reward` and a read-side `Trajectory` projected onto the existing `AuditProvider` stream (no new `ExecutionEvent` variant, HO-6 preserved), the `ExtensionRole::Environment` capability-manifest binding, a `run_with_environment` frozen-boundary combinator, the closed grading-mode set, the NE-13 profile budget, and the NE-12 archivable candidate tuple. It adds one extension role, no new command, no new `Status`, and no new error category beyond reusing the existing capability-unmet code.

## Related Specifications

- [l1-nodus-environment.md](l1-nodus-environment.md) — the contract this implements (NE-1…NE-13).
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — runtime crate extended here: `executor` (provider seam), the `run_with_*` family, `RunResult`.
- [l2-nodus-portability.md](l2-nodus-portability.md) — the LP-8 `ExtensionRole`/`CapabilityManifest`/`HostCapabilities` taxonomy this adds `Environment` to.
- [l2-nodus-observability.md](l2-nodus-observability.md) — the `AuditProvider`/`ExecutionEvent`/`FieldDescriptor`/`RunManifest` the trajectory projects onto (NE-3).
- [l2-nodus-dialog.md](l2-nodus-dialog.md) — the sibling extension role whose `step`-suspend shape a `step` reuses (a machine turn vs. a human turn).

## 1. Motivation

`l1-nodus-environment.md` defines a host-neutral way to run a workflow against a gradable world, but the crate has no environment seam, no reward type, and no graded-run combinator. This spec records the minimal additions that realize the contract while leaving the executor's run-to-completion model intact: a provider trait with a built-in no-I/O stub, a reward carried alongside (never inside) the run, a trajectory reconstructed from the audit stream rather than a second store, and a frozen-boundary run combinator that guarantees `evaluate` observes only a completed run.

## 2. Constraints & Assumptions

- No new external dependency (LP-1): the environment backend is an in-tree trait with a built-in deterministic resolver; the NE-12 content address uses a std-library stable digest, and any cryptographic content-hash is host-supplied (mirroring the LP-9 no-crypto-in-core rule).
- Additive to the LP-8 enum: `ExtensionRole::Environment` joins the existing `Model/Audit/Storage/Policy/Vocabulary/Dialog` set; `validate_manifest` already handles new roles generically.
- No new `Status` and no new command vocabulary: a `step` reuses the dialog suspend/resume machinery; `reset`/`step`/`evaluate` are provider calls the run combinator sequences, not workflow commands.
- The trajectory is **not a new store**: entries ride as optional side-band descriptors on existing `ExecutionEvent`s and run-boundary structures, so a host recording events already records trajectories and a no-op audit sink simply keeps none (NE-3, HO-6 preserved).
- `Observation` and `Action` are opaque to the crate: their shape is the environment's `profile()` concern, validated host-side exactly as host-schema commands are (LP-4).

## 3. Invariant Compliance

| NE Invariant | Rust Enforcement |
| --- | --- |
| NE-1 Environment as extension role | `EnvironmentProvider` trait (like `ModelProvider`/`DialogProvider`); built-in deterministic `StubEnvironment` ships in-crate; concrete worlds live outside (LP-2). |
| NE-2 Closed lifecycle | The trait exposes exactly `task_ids`/`profile`/`open`/`reset`/`step`/`evaluate`/`release`; `run_with_environment` is the only sequencer. `StubEnvironment` is deterministic per `(task, seed, actions)`; `release` is idempotent (double-release is a no-op). |
| NE-3 Trajectory is an observability projection | Each interaction attaches an optional `EnvInteraction` side-band descriptor to existing events — a `step` to the workflow step's `StepStart`/`StepEnd`, `reset` to the run-open boundary, `evaluate` to the terminal `RunManifest` — **no new `ExecutionEvent` variant** (HO-6). `Trajectory` is a read-side projection assembled by filtering the stream; it inherits HO-2 attribution, the §4.4 data-safety boundary (`FieldDescriptor`, no raw `Observation`/`Action` text), and HO-3 append-only immutability. |
| NE-4 Frozen evaluation | `run_with_environment` executes the workflow to a terminal state, marks the run frozen, then calls `evaluate` — which receives an immutable view and cannot re-enter `execute`. Two `evaluate` calls over one frozen instance return the same `Reward` (enforced by construction: `evaluate` reads the frozen trajectory only). |
| NE-5 Reward is typed data | `Reward` is returned in `EnvRunResult` **alongside** `RunResult`, never bound to a workflow variable or branched on mid-run; it has no effect on `Status` (a low reward is not `Failed`, a high reward is not `Ok`). |
| NE-6 Addressable catalog + profile | `task_ids() -> Vec<TaskId>` and `profile() -> EnvironmentProfile` (observation/action shape descriptors, orthogonal task labels, optional `Budget`, declared grading mode) published before any run; selection is by stable `TaskId`. |
| NE-7 Instance isolation | `open` returns a fresh `Instance` per run; instances share no mutable state. `release` is mandatory (called in a `finally`/drop guard) and idempotent; a non-released instance surfaces a liveness fault to the host, never silently reclaimed mid-run. |
| NE-8 Function-scoped auxiliary roles | `EnvRole { Policy, Environment, Context, Summary }` tags every trajectory entry; auxiliary roles (context/summary/judge scoring) resolve through function-scoped `ModelProvider` bindings, economical by default — never the policy route (reaffirms RTG-9). |
| NE-9 Host-supplied metric neutrality | `evaluate` delegates scoring to the host `EnvironmentProvider`; `StubEnvironment` (and any host supplying no scorer) returns `Reward { score: None, metadata: {} }` and the run stays valid. |
| NE-10 Capability-declared, fail-fast | `CapabilityManifest::from_workflow` adds `ExtensionRole::Environment` when a workflow declares an environment need; `validate_manifest` rejects fail-fast (reusing `NODUS:CAPABILITY_UNMET`) if the active host omits it. `HostCapabilities::builtin()` **provides** `Environment` via `StubEnvironment` — a deliberate contrast with `Dialog` (whose default resolver is incomplete): the stub is a complete trivial world, so a manifest-declaring workflow stays runnable in-process; a host constructed without `Environment` triggers the fail-fast. |
| NE-11 Declared grading mode | `GradingMode { Automated, Judge, Hybrid }` is declared on the task/profile (not per run). `evaluate` applies the mode: `Automated` runs a deterministic checker; `Judge` scores via a function-scoped auxiliary model over a published rubric; `Hybrid` runs the checker as a floor first and takes `min(checker, judge)` so a judge may lower but never rescue a checker-failed run. |
| NE-12 Archivable candidate result | `EnvRunResult::candidate()` yields `CandidateResult { workflow_digest, reward, trajectory, budget }` — a content-addressable tuple. `workflow_digest` is a std-library stable digest over the canonical workflow source (deterministic, zero-dep); a host wanting a cryptographic content-address computes it over the exposed source (LP-2). The candidate space/mutation/search/frontier stay host-side; the crate holds no optimizer. |
| NE-13 Budget-normalized graded runs | `EnvironmentProfile::budget: Option<Budget>` (`wall_clock_ms`/`max_steps`/`max_tokens`, any subset). `run_with_environment` halts the run uniformly when any declared limit is reached; a budget halt is a **normal outcome** (`Status::Partial`, not an error) with `budget_halted: true` recorded, and `evaluate` runs over what was achieved. The budget is part of the profile identity and copied into `CandidateResult.budget`; the workflow cannot read, extend, or evade it (enforced by the run loop, LP-10 kinship). Steps/tokens are deterministic; wall-clock is host-measured and recorded. A profile with no budget behaves as today. |

## 4. Detailed Design

### 4.1 EnvironmentProvider trait and StubEnvironment

```text
[REFERENCE]
pub type TaskId = String;
pub type Seed   = u64;

pub trait EnvironmentProvider {
    fn task_ids(&self) -> Vec<TaskId>;
    fn profile(&self) -> EnvironmentProfile;
    fn open(&self, task: &TaskId, seed: Seed) -> Instance;      // NE-2/NE-7
    fn reset(&self, inst: &mut Instance) -> Observation;         // NE-2 deterministic
    fn step(&self, inst: &mut Instance, action: Action) -> Observation;  // NE-2
    fn evaluate(&self, inst: &Instance) -> Reward;              // NE-4 read-only
    fn release(&self, inst: Instance);                          // NE-2/NE-7 idempotent
}

/// Built-in, deterministic, no I/O (mirrors StubProvider / DefaultDialogProvider).
pub struct StubEnvironment;   // task_ids = ["__stub__"], empty profile,
                              // step echoes the action, evaluate = no-op Reward (NE-9)
```

`Observation` and `Action` are opaque wrappers over `Value`; the crate never inspects their domain semantics (LP-4).

### 4.2 Reward, Trajectory and the side-band projection (NE-3)

```text
[REFERENCE]
pub struct Reward { pub score: Option<f64>, pub metadata: BTreeMap<String, Value> }

pub enum EnvRole { Policy, Environment, Context, Summary }        // NE-8
pub enum EnvKind { Reset, Step, Evaluate }

pub struct EnvInteraction {          // OPTIONAL side-band field on existing events
    pub seq: u64,                    // reuses the HO-7 monotonic run sequence
    pub kind: EnvKind,
    pub role: EnvRole,
    pub action: Option<FieldDescriptor>,       // descriptor only — never raw Action (NE-3/HO)
    pub observation: Option<FieldDescriptor>,  // descriptor only — never raw Observation
    pub reward: Option<Reward>,                // present on Evaluate
    pub step_index: Option<u32>,               // HO-2 back-pointer
}
```

An `EnvInteraction` rides an existing event, never a new `ExecutionEvent` variant (the HO-8/HO-13 additive-field discipline, so HO-6's closed taxonomy holds): a `step` on the workflow step's `StepStart`/`StepEnd`, `reset` on the run-open boundary, `evaluate` on the terminal `RunManifest`. `Trajectory` is a **read-side projection** a host assembles by filtering the audit stream for these descriptors — not a second store. A no-op audit sink keeps no trajectory; the run is otherwise unaffected.

### 4.3 Frozen-boundary run combinator (NE-4/NE-5)

```text
[REFERENCE]
pub struct EnvRunResult {
    pub result: RunResult,      // the workflow's own run (Status, vars, ...)
    pub reward: Reward,         // NE-5 alongside, never inside
    pub budget_halted: bool,    // NE-13
}

fn run_with_environment(source, filename, input, env, task, seed) -> Result<EnvRunResult, _> {
    let mut inst = env.open(task, seed);                 // NE-7
    let out = (|| {
        let obs = env.reset(&mut inst);                  // NE-2, reset side-band
        let result = execute(workflow, obs, env, &mut inst, budget);  // steps emit interactions (NE-3);
                                                         // budget halt = Status::Partial (NE-13)
        // FROZEN — no further mutation
        let reward = env.evaluate(&inst);                // NE-4 read-only over frozen run
        EnvRunResult { result, reward, budget_halted }
    })();
    env.release(inst);                                   // NE-7 mandatory + idempotent (drop guard)
    out
}
```

`execute` may `step` the environment as the workflow acts, reusing the dialog suspend/resume shape (a machine turn); `evaluate` runs strictly after the frozen point and cannot re-enter `execute`.

### 4.4 Manifest, grading, budget, candidate

```text
[REFERENCE]
// portability.rs — additive variant
pub enum ExtensionRole { Model, Audit, Storage, Policy, Vocabulary, Dialog, Environment }
// HostCapabilities::builtin() inserts Environment (StubEnvironment satisfies it) — unlike Dialog.

pub enum GradingMode { Automated, Judge, Hybrid }                 // NE-11, declared on the task
pub struct Budget { pub wall_clock_ms: Option<u64>,              // NE-13, any subset
                    pub max_steps: Option<u32>,
                    pub max_tokens: Option<u64> }
pub struct EnvironmentProfile { pub labels: BTreeMap<String,String>,  // NE-6 orthogonal slice labels
                                pub grading: GradingMode,
                                pub budget: Option<Budget> }

pub struct CandidateResult { pub workflow_digest: String,        // NE-12 content address (std digest)
                             pub reward: Reward,
                             pub trajectory_ref: RunId,
                             pub budget: Option<Budget> }         // rewards under different budgets not comparable
```

`grade` composes the modes with the hybrid floor: `Hybrid` runs the deterministic checker first and returns it on failure (a judge cannot rescue), else takes `min(checker, judge)`.

### 4.5 Public API

```text
[REFERENCE]
pub fn run_with_environment(
    source, filename, input, env: &dyn EnvironmentProvider, task: &TaskId, seed: Seed,
) -> Result<EnvRunResult, Vec<Diagnostic>>;

pub fn run_with_environment_and_audit(
    source, filename, input, env, task, seed, audit, run_id, started_at,
) -> Result<EnvRunResult, Vec<Diagnostic>>;
```

Consistent with the orthogonal `run_with_*` combinator family (LP-5); the audit variant is what actually emits the NE-3 trajectory side-bands.

## 5. Drawbacks & Alternatives

- **A dedicated trajectory store** — rejected (violates NE-3): forks observability, duplicates the data-safety boundary, and creates two sources of truth. The side-band-on-existing-events projection keeps one stream.
- **Reward bound to a workflow variable** — rejected (violates NE-4/NE-5): entangles grading with control flow and makes runs non-comparable. Reward lives in `EnvRunResult`, outside the `vars` map.
- **Cryptographic workflow hash in-core** — rejected (LP-1 zero-dep): the crate exposes a std-digest content address and the canonical source; a host needing a cryptographic address computes it (LP-2), exactly as LP-9 attestation is host-supplied.
- **`builtin()` omits `Environment` (mirror Dialog)** — rejected: the L1 (§4.4) makes the stub a complete world, so a manifest-declaring workflow must stay runnable in-process; the fail-fast is exercised by a host that omits the role.

## 6. Implementation Notes

Order that minimizes rework (follows the L1 §5 sequence): (1) `EnvironmentProvider` trait + `StubEnvironment` + lifecycle (NE-1/NE-2); (2) `EnvInteraction` side-band wiring onto existing audit hook points (NE-3, no store); (3) `ExtensionRole::Environment` + `builtin()` + `from_workflow` (NE-10, near-zero cost); (4) `run_with_environment` frozen ordering + `release` drop guard (NE-4/NE-5/NE-7, the hardest correctness property); (5) `EnvironmentProfile`/`Budget`/`GradingMode` (NE-6/NE-11/NE-13); (6) `CandidateResult` digest (NE-12). Each lands with unit tests plus an NE-invariant integration suite in `crates/nodus/tests/environment.rs` (deterministic replay, frozen-boundary immutability, idempotent release, manifest fail-fast, hybrid floor, budget halt).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[EXEC]` | `crates/nodus/src/executor.rs` | Provider trait pattern; the `EnvironmentProvider` seam and step dispatch |
| `[PORT]` | `crates/nodus/src/portability.rs` | `ExtensionRole::Environment`, `HostCapabilities::builtin`, `validate_manifest` |
| `[AUDIT]` | `crates/nodus/src/observability.rs` | `AuditProvider`/`ExecutionEvent`/`FieldDescriptor`/`RunManifest` the trajectory rides |
| `[API]` | `crates/nodus/src/workflows.rs` | `run_with_environment` combinators alongside the `run_with_*` family |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-10 | Core Team | Initial spec — Rust realization of the Environment/Evaluation contract: `EnvironmentProvider` trait + built-in deterministic `StubEnvironment`, closed `open/reset/step/evaluate/release` lifecycle, typed `Reward` carried in `EnvRunResult` (never in `vars`), `Trajectory` as a read-side projection via optional `EnvInteraction` side-band descriptors on existing events (no new `ExecutionEvent` variant, HO-6 preserved), `ExtensionRole::Environment` manifest binding with `builtin()` providing it via the stub (contrast with `Dialog`), `run_with_environment` frozen-boundary combinator, closed `GradingMode` (automated/judge/hybrid floor), `EnvironmentProfile` budget (NE-13, budget-halt = normal `Status::Partial`), and `CandidateResult` content-addressable tuple (NE-12, std-digest, crypto host-supplied). NE-1…NE-13 compliance table. Adds one extension role, no new command/status/error category. |

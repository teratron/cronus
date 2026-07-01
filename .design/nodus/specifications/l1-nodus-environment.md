# Nodus Environment and Evaluation Contract

**Version:** 1.0.0
**Status:** Draft
**Layer:** concept

## Overview

Nodus workflows are authored, validated, and executed against a *model provider*
and observed through an *audit provider*. What the runtime lacks is a first-class
way to run a workflow **against a task environment that produces a graded outcome**:
a reproducible world the workflow acts upon, a reward that scores the resulting
run, and a trajectory record that pairs actions with observations for later study.

This spec defines that missing seam: the **Environment/Evaluation contract**. It
introduces an `EnvironmentProvider` extension role (alongside the existing
`ModelProvider`, `AuditProvider`, `SchemaProvider`, `DialogProvider`), a typed
`Reward`, and a `Trajectory` projection carried through the existing observability
contract — never a parallel channel. The environment contract is the executable
substrate an evaluation-driven improvement loop scores a workflow against; it is
the runtime complement to the harness-evolution pattern, which decides *how* to
improve but presumes an executable, graded run already exists.

The contract is deliberately host-neutral: nodus defines the *shape* of an
environment and a reward, never the *metric* or the *world*. Both live in the host.

## Related Specifications

- [l1-nodus-portability.md](l1-nodus-portability.md) — extension-point taxonomy (LP-2) and capability manifest (LP-8); this spec adds one role to both.
- [l1-nodus-observability.md](l1-nodus-observability.md) — a trajectory is an observability projection; NE-3 binds to HO-1…HO-6.
- [l1-nodus-language.md](l1-nodus-language.md) — the executed workflow whose run is graded.
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust realization surface (`run_with_*` combinators) an environment run extends.
- [l1-nodus-dialog.md](l1-nodus-dialog.md) — shares the suspend/resume shape (a `step` is the machine analogue of an `ASK` turn).

## 1. Motivation

Three recurring needs are unmet by the current provider set:

1. **Graded execution.** A workflow may be *correct by validation* yet *poor by
   outcome*. Distinguishing the two requires running it against a task world and
   scoring the result. There is today no interface that yields a score.
2. **Reproducible task selection.** To compare two workflow revisions fairly they
   must run against the *same* task instances. That requires an addressable
   catalog of tasks with a stable profile, resettable to a known seed.
3. **Trajectory capture.** Improvement work needs to see *what happened* — the
   ordered interplay of the workflow's actions and the world's observations, with
   the reward attached — not just the final output. This is richer than a plain
   event log and must not fork a second observability path.

Without a contract, each host reinvents an ad-hoc scoring harness, couplings leak
into the library (violating host neutrality), and cross-host comparison becomes
impossible. This spec makes the environment a *portable seam*: the same workflow
runs against interchangeable environments that satisfy the same task profile.

## 2. Constraints & Assumptions

- The environment is **host-implemented**. The library ships exactly one built-in
  deterministic environment sufficient for in-process testing without external
  I/O (mirrors the `StubProvider`/`NoopAuditProvider` convention).
- Evaluation is a **read-only projection over a completed run**. It must not be
  able to alter the run it grades (reaffirms the frozen-evaluation boundary).
- Reward is **data, not control**. No workflow control-flow construct branches on
  a reward value during the run that produced it. Reward-driven adaptation, if a
  host wants it, is an *offline* activity across runs, outside this contract.
- The contract adds **one** extension role. It does not introduce a new command
  vocabulary, new error categories beyond a single capability code reuse, or a new
  status. A `step` reuses the existing suspend/resume machinery.
- Portability invariants (LP-1…LP-8) and language invariants (NL-1…NL-10) are
  upstream; NE-invariants add constraints, never relax them.

## 3. Core Invariants

Rules every implementation of this spec (and its host projects) MUST NOT violate:

- **NE-1 Environment as extension role**: a task environment is expressed as a
  named abstract interface (`EnvironmentProvider`) resolved from the host, never
  a concrete world baked into nodus core (reaffirms LP-1/LP-2). The library ships
  exactly one built-in deterministic environment for in-process testing.

- **NE-2 Closed interaction lifecycle**: an environment exposes a fixed lifecycle —
  `reset(task, seed) → observation`, `step(action) → observation`, and
  `evaluate() → reward`, bracketed by `open(task) → instance` and a mandatory,
  idempotent `release(instance)`. No other verbs mutate environment state. Given
  the same task, seed, and action sequence, `reset`/`step` are deterministic.

- **NE-3 Trajectory is an observability projection**: every environment
  interaction (`reset`, each `step`, `evaluate`) is recorded as an ordered entry
  in a **trajectory** carried through the existing audit contract — not a parallel
  channel. A trajectory entry references the same run and step attribution the
  observability contract already defines (HO-1/HO-2) and inherits its data-safety
  boundary (no raw user text in the clear) and append-only immutability (HO-3).

- **NE-4 Frozen evaluation**: `evaluate` is a pure read-only projection over a
  completed (frozen) run. It observes the trajectory and emits a reward; it cannot
  mutate the run, replay it with side effects, or feed back into the run's control
  flow. Two evaluations of the same frozen run yield the same reward.

- **NE-5 Reward is typed data**: a reward is a value with a numeric score and
  open metadata. It never appears as a control signal to the workflow during the
  run. Reward has no effect on `Status`; a low reward is not an error and a high
  reward is not a success — outcome grading and run success are orthogonal axes.

- **NE-6 Addressable task catalog with profile**: an environment publishes, before
  any run, a stable set of task identifiers and a **profile** describing the shape
  of its observations and admissible actions. Selection by identifier is
  reproducible; the profile is the portability contract two environments must
  share to be interchangeable for the same workflow.

- **NE-7 Instance isolation**: each run occupies an isolated environment instance;
  instances never share mutable state. Concurrent runs against one environment are
  independent. `release` is mandatory and idempotent; a leaked instance is a
  liveness fault surfaced to the host, never silently reclaimed mid-run.

- **NE-8 Function-scoped auxiliary roles**: where an environment run distinguishes
  a *policy* model (the workflow's own reasoning) from *auxiliary* cognitive roles
  (context-shaping, summarization, scoring assistance), those auxiliary roles
  resolve through function-scoped model bindings, economical by default — never
  the premium policy route by default. This reaffirms the smart-router
  function-scoped model-role rule at the environment boundary.

- **NE-9 Host-supplied metric neutrality**: the reward/scoring function is
  host-supplied and deterministic with respect to the frozen trajectory. Nodus
  defines the reward *shape* and the evaluation *timing*, never the metric. A host
  that provides no scoring yields a no-op reward (score absent, metadata empty),
  and the run remains valid.

- **NE-10 Capability-declared, fail-fast**: a workflow that requires an environment
  declares the `Environment` extension role in its capability manifest. Pre-run
  satisfiability validation rejects the run fail-fast if the active host provides
  no environment, before the first step executes (reaffirms LP-8). A workflow with
  no environment requirement runs unchanged against the built-in stub.

> An L2 spec realizing this contract cannot reach RFC until every NE-invariant is
> addressed in its Invariant Compliance section.

## 4. Detailed Design

### 4.1 EnvironmentProvider Interface

The environment is a single extension role added to the §4.1 portability taxonomy.
Its interface is expressed only in library and primitive types (LP-2):

```text
[REFERENCE]
role Environment:
  task_ids()                  -> list<TaskId>          // NE-6 addressable catalog
  profile()                   -> EnvironmentProfile     // NE-6 observation/action shape
  open(task: TaskId,
       seed: Seed)            -> Instance               // NE-2/NE-7 isolated instance
  reset(inst: Instance)       -> Observation            // NE-2 deterministic
  step(inst: Instance,
       action: Action)        -> Observation            // NE-2 deterministic
  evaluate(inst: Instance)    -> Reward                  // NE-4 frozen, read-only
  release(inst: Instance)     -> ()                      // NE-2/NE-7 mandatory, idempotent

builtin StubEnvironment:      // NE-1: deterministic, no external I/O
  task_ids() -> ["__stub__"]
  profile()  -> empty-profile
  step(_, _) -> echo(action)  // deterministic identity world
  evaluate(_)-> Reward{ score: none, metadata: {} }   // NE-9 no-op reward
```

`Observation` and `Action` are opaque to nodus core — their shape is declared by
the environment's `profile()` and validated by the host, exactly as host schema
commands are (vocabulary isolation, LP-4). Nodus never inspects their domain
semantics.

### 4.2 Trajectory and Reward Model

A trajectory is the ordered, append-only record of one run's interaction with its
environment, projected onto the observability event stream (NE-3):

```text
[REFERENCE]
Reward := {
  score    : number?          // absent = ungraded (NE-9 no-op)
  metadata : map<string, any> // host-defined breakdown; data-safety bounded (HO)
}

TrajectoryEntry := {
  seq       : int             // monotonic within the run (see l1-nodus-observability)
  kind      : reset | step | evaluate
  role      : policy | environment | context | summary   // NE-8 role attribution
  action?   : Action          // present for step
  observation? : Observation  // present for reset | step
  reward?   : Reward           // present for evaluate
  step_ref  : StepRef          // HO-2 per-step attribution back-pointer
}

Trajectory := ordered list<TrajectoryEntry>   // append-only (HO-3)
```

The trajectory is not a new store: each `TrajectoryEntry` is emitted as an audit
event through the existing `AuditProvider`, so a host that records events already
records trajectories. A host that discards events (the no-op audit sink) still
runs; it simply keeps no trajectory.

### 4.3 Frozen Evaluation Boundary

Evaluation happens after the run reaches a terminal state. The boundary is a hard
line (NE-4):

```text
[REFERENCE]
run_with_environment(workflow, env, task, seed):
    inst = env.open(task, seed)
    try:
        obs = env.reset(inst)                 // NE-2
        result = execute(workflow, obs, env, inst)   // steps emit trajectory entries (NE-3)
        // run is now FROZEN — no further mutation permitted
        reward = env.evaluate(inst)           // NE-4 read-only projection over frozen run
        return { result, reward, trajectory } // NE-5 reward alongside, not inside
    finally:
        env.release(inst)                     // NE-7 mandatory, idempotent
```

`execute` may `step` the environment as the workflow acts; each `step` reuses the
suspend/resume shape the dialog contract already defines (a machine turn instead of
a human turn), so no new status is introduced. `evaluate` runs strictly after the
frozen point and cannot re-enter `execute`.

### 4.4 Capability Manifest Integration

The environment adds one role to the manifest taxonomy (LP-8). No new resolver
logic is required — the existing pre-run validation already iterates roles:

```text
[REFERENCE]
manifest.roles ⊇ { Environment }   // declared when a workflow needs a graded world
validate(manifest, host):
    ... existing role loop ...
    if Environment in manifest.roles and not host.provides(Environment):
        reject_fail_fast("workflow requires Environment; host provides none")  // NE-10
```

A built-in host satisfies `{ Environment }` via `StubEnvironment`, so a
manifest-declaring workflow remains runnable in the in-process test configuration.

### 4.5 Auxiliary Model Roles

An environment run may involve more model traffic than the workflow's own
reasoning: shaping the observation into context, summarizing a long trajectory,
or assisting the host metric. NE-8 requires these to be *function-scoped* bindings
distinct from the policy model, resolved economically by default. This mirrors the
host-side smart-router discipline: internal cognitive chores never default to the
premium user-facing route. The taxonomy is advisory at L1 — a host may collapse
all roles onto one binding — but the *separation of concern* is the invariant, so
cost and behavior remain attributable per role in the trajectory (`role` field).

### 4.6 Relationship to Evaluation-Driven Improvement

This contract is the **executable substrate**, not the improvement policy. A
harness-evolution loop (evaluate → analyse → improve) supplies the *decision*
about what to change between runs; this spec supplies the *graded run* that loop
consumes. The division of labour:

| Concern | Owner |
| --- | --- |
| Reproducible graded run (reset/step/evaluate/reward) | this spec (NE-2, NE-4, NE-5) |
| Trajectory capture | observability contract, projected here (NE-3) |
| What to change after a run, transfer validity, budget | harness-evolution pattern (host) |
| The metric / the world | host environment (NE-9) |

Keeping these separate means an environment is reusable under any improvement
policy, and any improvement policy composes with any conforming environment.

## 5. Implementation Notes

Evaluation order that minimises rework:

1. NE-1/NE-2 (interface + lifecycle) — define the trait and lifecycle first; every
   other invariant references it.
2. NE-3 (trajectory projection) — wire entries onto the existing audit hook points;
   do not add a store.
3. NE-10 (manifest) — one role added to the existing resolver; near-zero cost.
4. NE-4/NE-5 (frozen boundary + reward-is-data) — enforce the ordering in the run
   combinator; the hardest correctness property to get right.
5. NE-6/NE-7 (catalog/profile + isolation) — required before any cross-revision
   comparison is meaningful.
6. NE-8/NE-9 (role split + metric neutrality) — advisory shaping; safe to land last.

## 6. Drawbacks & Alternatives

**Alternative: fold environment into ModelProvider.** Treat the world as "another
model call." Rejected: conflates a *stochastic reasoning* seam with a *stateful,
resettable, gradable world* seam; breaks NE-6 reproducibility and NE-4 frozen
evaluation, and pollutes model-role accounting.

**Alternative: reward as a control signal.** Let workflows branch on reward mid-run
for reinforcement-style shaping. Rejected: violates NE-4/NE-5, entangles grading
with execution, and makes runs non-comparable. Reward-driven adaptation belongs to
an offline loop across frozen runs.

**Alternative: a dedicated trajectory store.** A parallel persistence path for
trajectories. Rejected: forks observability (violates NE-3), duplicates the
data-safety boundary, and creates two sources of truth for "what happened."

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[EXT-POINT]` | `crates/nodus/src/executor.rs` | Provider trait pattern (`ModelProvider`) the `EnvironmentProvider` interface mirrors |
| `[MANIFEST]` | `crates/nodus/src/portability.rs` | `ExtensionRole` enum + `validate_manifest` the `Environment` role extends |
| `[AUDIT]` | `crates/nodus/src/observability.rs` | `AuditProvider` + `ExecutionEvent` the trajectory projects onto |
| `[PUBLIC-API]` | `crates/nodus/src/lib.rs` | `run_with_*` combinator surface a `run_with_environment` combinator joins |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-01 | Core Team | Initial spec — Environment/Evaluation contract: `EnvironmentProvider` extension role, closed reset/step/evaluate lifecycle, typed `Reward`, `Trajectory` as observability projection, frozen-evaluation boundary, capability-manifest role, function-scoped auxiliary model roles; NE-1…NE-10. Adds one role to the portability taxonomy (LP-2/LP-8); the executable substrate for evaluation-driven harness improvement. |

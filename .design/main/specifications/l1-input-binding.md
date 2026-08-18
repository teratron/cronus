# Input Binding

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

`l1-output-contracts` governs what a work unit **produces**: a declared schema, validators,
retry-with-verdict. Nothing yet governs the symmetric and earlier question — how a unit's
**inputs are obtained** from the invocation it was handed, and what happens when they
cannot be.

Today each invocable surface answers that ad hoc. A tool digs into the raw call payload and
parses fields itself; a node reaches into the run context; a command re-parses its own
arguments. Each one re-implements the same parsing, invents its own failure shape, and
discovers the problem *after* it has already started doing work. The consequences are
concrete: the caller cannot tell whether the unit ran and failed or never ran at all;
"missing field", "malformed payload", and "wrong type" collapse into one useless *invalid
input*; an optional field silently swallows a malformed value; and the schema advertised to
the caller drifts away from what the body actually reads.

**Input binding** is the missing discipline: a unit **declares** an ordered list of typed
**binders** naming what it needs; the runtime supplies them **before the body runs**; and a
binder that cannot supply its value produces a **typed rejection** — a first-class, named
outcome on the normal result channel, never an unstructured fault. Two supply phases are
distinguished: what is fixed when the unit is mounted (identity, configuration, capability
handles) is **sealed at composition time**, so an unmet dependency is a construction error
rather than a first-call surprise; what varies per call is bound per call.

The payoff for an agent-facing surface is direct. A rejection that names *which* input,
*which* failure mode, and *where* is a rejection the caller can act on unassisted — and the
guarantee that a rejected call **did not run** is exactly what the completion and receipt
disciplines need to stay honest.

## Related Specifications

- [l1-output-contracts.md](l1-output-contracts.md) — the **twin on the other side** of the unit: OC governs the produced value (schema + validators + retry-with-verdict); this governs the consumed values (binders + typed rejections + bind-before-invoke). Neither re-specifies the other; a unit may carry both, either, or neither.
- [l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) — ATE-2 (recoverable conditions return actionable guidance, not errors) is the ergonomic law a typed rejection obeys; ATE-12's output-side overflow guard is the twin of IB-10's input-side default ceiling; ATE-13 distinguishes *unresolved address* from *resolved-but-wrong-invocation*, the failure class that sits immediately upstream of binding.
- [l1-tool-call-transport.md](l1-tool-call-transport.md) — TCT-4 (one schema source, transport-idiomatic advertisement) is what IB-1 supplies the source *for*; TCT-8 (malformed-call containment) is the transport-layer sibling of a binder rejection, one layer below.
- [l1-tool-composition.md](l1-tool-composition.md) — TC-2 derives a dispatcher's callable interface from the union of member schemas; those member schemas are exactly the IB-1 declarations, so derivation and enforcement share one source.
- [l1-interception-model.md](l1-interception-model.md) — **deliberately distinct**: an interceptor *guards an effect* (observe / decide / transform) and may veto it; a binder *supplies a value* and may only reject. A binder is not a permission gate and never becomes one; where a binder needs an authorization answer it consults the guard, it does not replace it. INT-10 fixes which side of resolution each attaches to.
- [l1-execution-graph.md](l1-execution-graph.md) — EG-2 (channel-only state transfer) and EG-11 (immutable invocation context) are the graph-level expression of the same rule: a unit reads only what was declared and handed to it, never ambient state.
- [l1-declarative-configuration.md](l1-declarative-configuration.md) — the composition-phase supply (IB-6) is where a validated configuration surface lands; configuration is bound once and sealed, not re-parsed per call.
- [l1-security.md](l1-security.md) — the binder is the natural place for trust-boundary input validation, and IB-10's default ceiling is a resource-exhaustion control; frugality never cuts either (FR-3).
- [l1-completion-verification.md](l1-completion-verification.md) — IB-2's guarantee makes *did not run* a reportable state distinct from *ran and failed*, which CMP-1/CMP-5 need in order to describe an outcome honestly.
- [l1-practice-analytics.md](l1-practice-analytics.md) — the IB-12 rejection channel is a detector metric: a rising rejection rate indicts the advertisement, not the implementation.
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.7): the typed workflow I/O contract is already a binder declaration; the delta is *when* it is enforced and *how finely* its failure is typed.

## 1. Motivation

**A failure discovered inside the body is a failure discovered too late.** When a unit parses
its own arguments, the parse happens after the unit has been entered — often after it has
opened a resource, taken a lease, or written a log line claiming it started. The caller then
receives a failure it cannot classify: was the work attempted? Is a retry safe? Binding
before invocation makes "the body never ran" a *structural* guarantee rather than a hope,
and idempotent retry a safe default for the whole rejection class.

**Collapsed failure modes destroy the corrective signal.** *Absent*, *unreadable*,
*malformed*, and *well-formed-but-wrong-shape* have four different fixes: supply it, fix the
channel, fix the syntax, fix the type. A surface that answers all four with "invalid input"
forces the caller — human or model — to guess among them, and a model guessing among four
fixes burns a call per guess. Naming the mode and the location converts a retry loop into a
single corrected call.

**Optional handling is where quiet corruption enters.** The natural implementation of "this
input is optional" swallows *any* failure and proceeds with a default. That silently turns a
malformed value into a missing one, and the unit does the wrong work with full confidence.
The distinction is small to specify and impossible to retrofit once callers depend on the
lenient behavior.

**Unsatisfied dependencies should not be discovered by traffic.** A unit that needs a
configuration value, a store handle, or an identity has that need permanently, not per call.
If the need is resolved from an ambient bag at call time, the first call is the test — in
production, on the user's work. Sealing composition-phase needs when the surface is
assembled moves that failure to assembly, where it is cheap and total.

**A schema nobody enforces is a lie with a version number.** When the advertised input shape
and the code that reads the input are two artifacts, they drift, and the caller is trained on
the stale one. One declaration serving both is the only arrangement in which advertisement
and enforcement cannot disagree.

## 2. Constraints & Assumptions

- The unit of binding is an **invocable**: a tool, a command, a node, a handler, a workflow
  entry point. This spec names no transport, payload dialect, or type system.
- Binding is **supply**, not **authorization**. A binder answers *can this value be
  produced?*; whether the caller may invoke at all is the permission layer's question and is
  settled independently (a binder that succeeds does not imply an effect is permitted).
- Address resolution happens **before** binding: this spec presumes the invocation already
  resolved to exactly one unit. Failure to resolve is a different, earlier outcome class.
- A rejection is a **normal outcome**, not an exception. Everything here presumes a result
  channel that can carry a structured negative answer.
- The discipline is **opt-in per unit but total within a unit**: a unit that declares no
  binders is unaffected; a unit that declares any binds all of them under these rules.
- Determinism is assumed for the *classification* of a rejection, not for the value: given
  the same invocation, the same binder rejects with the same mode and location.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **IB-1 Declared need, runtime supply, one source**: an invocable declares an **ordered list
  of typed binders** naming what it needs; the runtime obtains those values and hands them
  in. The body MUST NOT reach into the raw invocation envelope for anything it did not
  declare. The declaration is machine-readable and is the **single source** from which the
  unit's advertised input schema is derived, so what the caller is told and what the runtime
  enforces cannot drift.

- **IB-2 Bind-before-invoke**: every declared binder completes successfully **before** the
  body begins. A binding failure means the body **did not run**, and that fact is part of the
  reported outcome — never "started and failed". No effect, no lease, no partial write may
  precede a completed binding. This is what makes the entire rejection class safely
  re-invocable.

- **IB-3 A rejection is a typed first-class outcome, never a fault**: each binder declares a
  **closed, enumerable rejection set**, and a rejection travels the **normal result channel**
  as a structured value. A binding problem MUST NOT escape as an unstructured fault, an
  unhandled exception, or a terminated channel. The unit's fault channel for binding is
  effectively uninhabited: failure is representable only as an outcome, so it cannot be
  dropped on the floor by an intermediary that forgot to handle it.

- **IB-4 Four distinguishable modes, each located**: a rejection names its mode from a closed
  set — **absent** (nothing was supplied), **unreadable** (the source could not be obtained
  at all), **malformed** (present but not parseable), **ill-shaped** (parseable but not
  conforming to the declared type/constraint) — plus the **location** within the input (field
  path, index, position). Collapsing the four into one "invalid input" is a defect: each mode
  implies a different corrective action, and the location is what makes the correction a
  single edit rather than a search.

- **IB-5 Optional means absent, never invalid**: a binder declared optional yields its
  declared *not-provided* value **only** when the input is genuinely absent. A value that is
  present and fails to bind still **rejects**, with its mode and location intact. Mapping a
  malformed value onto "not provided" is forbidden — it is the quiet failure that makes a
  unit do confidently wrong work.

- **IB-6 Two supply phases; composition-phase needs are sealed before mounting**: every need
  is supplied at exactly one of two phases — **composition** (fixed for the unit's lifetime:
  identity, validated configuration, store/capability handles) or **invocation** (per-call
  data). A unit with an **unsatisfied composition-phase need is not mountable**: the unmet
  dependency is surfaced when the surface is assembled, never deferred to the first call. A
  mounted surface is by construction one whose fixed needs are all supplied.

- **IB-7 The dynamic side-channel is a bounded fallback, never the default**: where a value
  genuinely cannot be declared (it is contributed by a guard at run time, or its type is not
  known to the unit), it may travel an untyped context channel whose absence is an ordinary
  IB-4 rejection. Any need expressible as a declared composition- or invocation-phase binder
  **MUST** be declared as one. Routing ordinary state through the untyped channel trades a
  composition-time error for a run-time one and is a defect, not a style choice.

- **IB-8 Once-only inputs bind once, last, by declaration**: at most **one** binder may
  consume an input that can be consumed only once (a stream, an exclusive lease, a one-time
  token), and it is ordered **last** among the unit's binders. The constraint is enforced
  when the unit is **declared**, not detected at run time: two declared consumers of the same
  once-only input is a declaration error, surfaced at assembly. Non-consuming binders may
  appear in any order before it.

- **IB-9 Ordered, short-circuit, stable**: binders run in **declared order**; the **first**
  rejection stops the remainder and is the reported outcome. The same invocation therefore
  always yields the same first rejection — a prerequisite for a caller to learn the fix, and
  for a rejection to be a reproducible test fixture. Reporting an arbitrary or aggregated
  rejection instead is forbidden; a surface MAY additionally report the remaining
  *independent* findings, but never in place of the deterministic first.

- **IB-10 Every unbounded source binds under a default ceiling**: a binder that reads an
  input of caller-controlled size applies a **default limit** which must be **explicitly
  raised** where a larger input is genuinely expected. Unbounded-by-default is forbidden.
  This is the input-side twin of the output overflow guard: the caller controls the size, so
  the ceiling belongs to the host, and an exceeded ceiling is an ordinary typed rejection
  naming the limit — not a truncation, and not a crash.

- **IB-11 Binders compose and wrap**: a binder MAY be defined in terms of other binders, and
  a **wrapping** binder MAY add measurement, caching, normalization, or policy consultation
  around an inner binder without the inner one knowing. Composition is the reuse mechanism
  that keeps a shared input shape parsed in exactly one place; re-implementing the same parse
  per unit is the duplication this invariant exists to prevent.

- **IB-12 Rejections are observable on their own channel**: binding rejections are recorded
  **distinctly from execution failures** and are attributable to (unit, binder, mode). This
  separation is the point: a rising rejection rate indicts the **advertisement** — the
  declared schema, its description, the examples the caller was trained on — not the
  implementation. A surface whose rejections are indistinguishable from its errors cannot
  tell "my callers are confused" from "my code is broken".

- **IB-13 Deferred satisfaction is legitimate until settlement, and a defect after it**: `[ADDED
  v1.1.0]` where a surface is assembled from parts that mount **concurrently**, a unit whose
  composition-phase need is not yet supplied **waits** rather than failing — a provider not yet
  mounted is not a missing provider, and IB-6's "not mountable" would otherwise reject every
  legitimate mount-ordering race. That deferral is bounded by a declared **settlement point**:
  once assembly settles, every unit that is **enabled and still unsatisfied** is a startup
  failure, and the failure **enumerates, per unit, the exact needs that were never supplied**.
  Waiting is a state, never an outcome. Without the settlement assertion IB-6's guarantee
  inverts into its worst form — the unit neither runs nor fails, emits no output and no error,
  and the only symptom is a capability that is silently absent; the operator's first evidence is
  a feature that does nothing, with nothing anywhere naming what it was waiting for. Two
  corollaries: an intentionally-disabled unit is exempt (it was never expected to activate), and
  the settlement report distinguishes **failed to activate** (it ran and threw — report its
  original fault) from **never activated** (it is still waiting — report its unresolved needs),
  because the two have different first actions and collapsing them loses the one piece of
  information that locates the problem.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The binding pipeline

```text
[REFERENCE]
invoke(unit, invocation):
    // composition-phase needs were already sealed at mount (IB-6) — nothing to do here
    values := []
    for binder in unit.binders:                       // IB-9 declared order
        r := binder.bind(invocation, unit.sealed)
        if r is Rejection:
            return Outcome::NotRun(r)                 // IB-2: the body never ran
        values += r.value
    return unit.body(values...)                       // binding complete, effects may begin
```

Two properties carry the design. The loop is **before** the body, so `NotRun` is a
structurally distinct outcome rather than a claim the body makes about itself. And the loop
short-circuits, so the reported rejection is the *first* one in declared order — stable
across runs, and therefore learnable.

### 4.2 The rejection taxonomy (IB-4)

| Mode | Means | Corrective action it implies |
| --- | --- | --- |
| **absent** | nothing was supplied for this need | supply the value |
| **unreadable** | the source itself could not be obtained (channel failure, truncated transfer, exceeded ceiling) | retry or fix the channel; not a caller-data problem |
| **malformed** | present but not parseable in its own encoding | fix the syntax at the named location |
| **ill-shaped** | parses, but violates the declared type or constraint | fix the value's type or range at the named location |

The value of the split is that each row routes to a different actor: *absent* and *ill-shaped*
are the caller's to fix, *malformed* is usually the caller's encoder, and *unreadable* is the
transport's. A surface answering "invalid input" makes every one of these look like the
caller's fault, which is both wrong and unactionable.

The **location** is not decoration. "The value at `items[3].deadline` is not a date" is one
edit; "invalid input" is a search through the whole payload.

### 4.3 Two supply phases and sealing (IB-6)

```text
[REFERENCE]
mount(unit, surface):
    unmet := unit.composition_needs - surface.supplied
    if unmet ≠ ∅:  return CompositionError(unmet)     // assembly fails, not the first call
    return surface.with(unit.sealed_with(surface.supplied))
```

A mounted surface is one whose fixed needs are all satisfied — that is the property mounting
*means*. The alternative arrangement, where a unit resolves its dependencies from an ambient
registry on each call, has the same failure available at every call forever, and its first
occurrence is by definition in production.

A sub-surface may be sealed with its **own** fixed supplies and then mounted inside a larger
one that has different supplies; sealing is what makes the sub-surface a self-contained,
composable unit rather than something that leaks its requirements upward.

### 4.4 Once-only inputs (IB-8)

A once-only input is not a normal value: reading it destroys it. Two binders both trying to
read one is not a rare race but a **guaranteed** bug in every invocation, which makes it a
property of the *declaration*, not the run.

Placing the consuming binder **last** is what makes the rule checkable without executing
anything: every binder before it is, by declaration, non-consuming. The check is a
declaration-time scan, and its failure is a construction error with a name — the same class
as an unmet composition need, and diagnosable in the same place.

### 4.5 Boundaries

- **Against output contracts.** OC validates a value the unit *produced* and may retry the
  unit with accumulated verdicts. IB validates values the unit *consumes* and never retries —
  a rejection is returned, because re-running a binder against the same unchanged invocation
  yields the same rejection by IB-9. Retry-with-feedback belongs to the producer side; the
  consumer side's honest move is to reject clearly and let the caller correct.
- **Against interception.** A guard decides whether an effect may happen; a binder decides
  whether a value can be produced. They are adjacent and must not merge: a binder that starts
  vetoing effects becomes an undeclared permission surface outside the guard taxonomy, and a
  guard that starts supplying values becomes an undeclared dependency. Where a binder needs an
  authorization answer, it consults the guard.
- **Against transport.** The transport decodes the wire form into a logical invocation and
  contains what is unparseable at that level; binding starts from a well-formed logical
  invocation. A malformed *wire* payload is a transport containment case; a malformed *field*
  is an IB-4 rejection.

### 4.6 Why a rejection is not an error

The load-bearing structural choice is that a binding failure is **representable only as an
outcome**. When failure can also travel a separate fault channel, every intermediary between
the unit and the caller must remember to handle that channel, and the one that forgets turns
a precise, actionable rejection into a dropped call or a closed connection — the caller
learns nothing, and the surface looks broken rather than misused.

Making the fault channel uninhabited for this class removes the choice: there is nowhere for
a binding failure to go except into the answer. Every rejection therefore reaches the caller
in the shape the caller can act on, which is the entire point of typing them.

### 4.7 nodus projection

The workflow language already carries the front half of this model: its typed I/O contract
declares a workflow's inputs, context, and error surface, and its declarative configuration
surface is validated **before** any value becomes visible to a workflow — precisely the
composition-phase sealing IB-6 describes. Three concrete deltas follow, none requiring a new
language construct:

1. **Move input enforcement ahead of the first step.** Input presence and type are enforced
   at run time today, which means a violation can surface *after* earlier steps have run and
   produced effects. The binding discipline makes it a **validation-class** failure emitted
   before execution — the same posture the language already takes for undefined commands,
   undefined variables, and schema mismatches. This is a candidate amendment to the language's
   typed-I/O invariant, recorded here and deliberately **not** applied by this spec.
2. **Refine the input error taxonomy to the four modes.** The runtime's error vocabulary
   distinguishes validation from run-time origin but not *absent* from *malformed* from
   *ill-shaped* on a workflow input. Splitting them (with the offending input named) is a
   closed-vocabulary extension of the existing taxonomy, not a new mechanism.
3. **Optional inputs bind to the null value, never to a swallowed failure.** The language's
   closed value space already has a null; IB-5 fixes its meaning for an optional input —
   absent binds to null, present-and-invalid rejects.

The ceiling on unbounded inputs (IB-10) and the wrapping-binder pattern (IB-11) stay
host-side, consistent with how every other resource-and-policy concern maps onto the host
provider surface: the language contributes the declaration and the typed failure, the host
contributes the limit and the judgement.

## 5. Implementation Notes

1. Derive the advertised schema **from** the binder declarations (IB-1) rather than
   maintaining it beside them; a generated advertisement cannot drift from what is enforced.
2. The once-only check (IB-8) and the unmet-need check (IB-6) belong in the same
   assembly-time pass — both are construction errors, and reporting them together makes
   assembly a single diagnosable step.
3. Keep the rejection vocabulary closed and versioned with the surface contract: callers
   (and models trained on them) match on the mode, and an open-ended vocabulary is one that
   silently grows a case nobody handles.
4. Instrument the rejection channel (IB-12) from the start; retrofitting it means losing the
   baseline against which "my advertisement is confusing" becomes visible.

## 6. Drawbacks & Alternatives

- **Declaring binders is more ceremony than reading the payload inline.** Real for a
  one-argument unit. The ceremony buys the advertised-schema derivation, the bind-before-run
  guarantee, and the located rejection — none of which the inline version can offer at any
  price. IB is opt-in per unit (§2) precisely so a trivial unit need not pay it.
- **A closed rejection vocabulary will eventually lack a case.** Accepted: the closure is
  what makes callers able to match exhaustively. Extension is a versioned change to the
  contract, which is the visible, reviewable path — an open vocabulary makes the same
  extension invisible.
- **Sealing at composition time makes late-bound configuration awkward.** Intended. A value
  that genuinely varies per call is an invocation-phase need, not a composition-phase one;
  the awkwardness is the model correctly refusing to let a per-call value masquerade as a
  fixed one.
- **Alternative — validate inputs inside the body, first thing.** Rejected by IB-2: "first
  thing in the body" is still inside the body, so *did not run* stops being provable, and
  every unit re-implements the check with its own failure shape.
- **Alternative — one generic "invalid input" rejection.** Rejected by IB-4: it is the
  status quo, and it is what forces a caller to guess among four different fixes.
- **Alternative — fold binding into the interception model as a "supply" interceptor kind.**
  Rejected: it would add a sixth kind whose composition semantics (ordered, short-circuit,
  value-producing, non-vetoing) match none of the existing five, and it would let a supplier
  quietly acquire veto power over effects (§4.5).
- **Alternative — fold into output contracts as a symmetric "input contract" section.**
  Rejected: OC's core mechanic is *retry the producer with verdicts*, which is exactly what
  the consumer side must not do (§4.5). Same word, opposite machinery.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[OUTPUT]` | `.design/main/specifications/l1-output-contracts.md` | The producer-side twin; the boundary §4.5 draws. |
| `[ERGONOMICS]` | `.design/main/specifications/l1-agent-tool-ergonomics.md` | ATE-2 rejection ergonomics, ATE-12 output ceiling, ATE-13 the upstream failure class. |
| `[TRANSPORT]` | `.design/main/specifications/l1-tool-call-transport.md` | TCT-4 single schema source; TCT-8 the layer below binding. |
| `[INTERCEPT]` | `.design/main/specifications/l1-interception-model.md` | The guard taxonomy a binder must not become (§4.5); INT-10 phase placement. |
| `[CONFIG]` | `.design/main/specifications/l1-declarative-configuration.md` | Where the composition-phase supply is declared and validated. |
| `[COMPLETION]` | `.design/main/specifications/l1-completion-verification.md` | Consumes the *did not run* outcome IB-2 creates. |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the discipline projects onto (§4.7). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.1.0 | 2026-08-19 | Core Team | IB-13 added — **deferred satisfaction is legitimate until settlement, and a defect after it**, reconciling IB-6 with concurrent assembly. IB-6 says an unsatisfied composition-phase need makes a unit unmountable, which is right at the end of assembly and wrong during it: where parts mount concurrently, a provider not yet mounted is not a missing provider, and rejecting every mount-ordering race would forbid the ordinary case. The reconciliation is a declared **settlement point** — waiting is permitted before it and is a startup failure after it, with the failure enumerating **per unit** the exact needs never supplied. Without the assertion IB-6's guarantee inverts into its worst form: the unit neither runs nor fails, emits no output and no error, and the only symptom is a silently absent capability whose first evidence is a feature that does nothing with nothing naming what it awaited. Two corollaries: a deliberately-disabled unit is exempt, and the report separates **failed to activate** (ran and threw — carry its original fault) from **never activated** (still waiting — carry its unresolved needs), since the two have different first actions and collapsing them discards the one fact that locates the problem. Composes the new `l1-composition-layering` (LAY-3 disabled entries are the exempt class) and `l1-composition-binding` (a composition that cannot load never reaches settlement — CBD-8). Distilled from an adoption pass over an external plugin-framework-based agent-harness reference. Additive; no existing invariant weakened. |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — input binding as the consumer-side twin of output contracts, closing the asymmetry where nothing governed how a unit obtains its inputs or what happens when it cannot: declared typed binders as the single source for both runtime supply and the advertised schema, with the body forbidden to reach into the raw envelope (IB-1); bind-before-invoke making *did not run* a structural outcome rather than a self-report, and the whole rejection class safely re-invocable (IB-2); rejection as a typed first-class outcome on the normal channel with an effectively uninhabited fault channel, so a binding failure cannot be dropped by a forgetful intermediary (IB-3); four distinguishable located modes — absent / unreadable / malformed / ill-shaped — because each implies a different corrective action and a different responsible actor (IB-4); optional means absent and never invalid, closing the quiet-corruption path where a malformed value becomes a default (IB-5); two supply phases with composition-phase needs sealed before mounting, so an unmet dependency fails at assembly rather than in production traffic (IB-6); the untyped dynamic channel as a bounded fallback that never substitutes for a declarable need (IB-7); once-only inputs bound once and last, enforced at declaration rather than detected at run time (IB-8); ordered short-circuit binding whose first rejection is stable and therefore learnable (IB-9); a default ceiling on every caller-sized input, the input-side twin of the output overflow guard (IB-10); composing and wrapping binders as the single-parse reuse mechanism (IB-11); and a separate rejection channel, since a rising rejection rate indicts the advertisement rather than the implementation (IB-12). Nodus projection needs no new construct: move input enforcement ahead of the first step (recorded as a candidate amendment to the typed-I/O invariant, deliberately not applied here), split the input error taxonomy into the four modes, and fix optional-input semantics onto the existing null value. Concept-only. |

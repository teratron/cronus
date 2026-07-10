# Nodus DSL Language

**Version:** 1.10.0
**Status:** Stable
**Layer:** concept

## Overview

Nodus is a declarative, schema-driven workflow language for AI-augmented automation pipelines. A workflow describes inputs, outputs, reactive triggers, hard constraints, soft preferences, and a bounded sequential step body. The language has two syntactic representations — **compact** (machine-canonical `.nodus` files) and **human** (prose rendering) — that are semantically equivalent and convertible in either direction.

Nodus is runtime-agnostic at L1: any conforming implementation must honour the invariants below regardless of host language or platform.

## Related Specifications

- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust implementation of this language
- [l1-nodus-testing.md](l1-nodus-testing.md) — Full testing contract for `@test:` blocks (NT-1…NT-10 invariants, execution protocol, TestReport contract)
- [../../main/specifications/l2-workflow-runtime.md](../../main/specifications/l2-workflow-runtime.md) — Cronus integration layer (step binding, subsystem dispatch, platform constraints)
- [../../main/specifications/l1-tokenization-boundary.md](../../main/specifications/l1-tokenization-boundary.md) — the contract NL-19 realizes: the control sub-alphabet is unreachable from content (TB-4/TB-5), and encoding is not composable so cuts land only on declared stable seams (TB-2/TB-3)

## 1. Motivation

Workflow automation for AI agents requires a language that:

- Enforces safety rules at the language level — hard constraints cannot be bypassed by any step handler.
- Keeps business logic auditable — human-readable rendering; declared I/O at file boundaries.
- Stays portable across host languages and platforms — schema-driven vocabulary, no built-in I/O primitives.
- Bounds execution to prevent runaway loops — mandatory `MAX:n` on unbounded iterations.
- Provides a typed error taxonomy — every failure mode has a named code that tooling can pattern-match.

## 2. Constraints & Assumptions

- Workflows are stateless between invocations; state is passed via `@in` / `@ctx` or returned via `@out`.
- The schema is the authoritative vocabulary; adding a command requires a schema update, not a runtime patch.
- Workflows are not Turing-complete by design: loops must be bounded; recursion is not supported.
- Parallel blocks (`~PARALLEL`) are a scheduling hint to the runtime, not a strict concurrency guarantee.
- A schema file (`§schema:`) is a configuration artifact, not a workflow; it has no `@steps:` body.

## 3. Core Invariants

Rules that Layer 2 implementations MUST NOT violate:

- **NL-1 Schema-first**: every workflow loads a vocabulary schema before execution; unknown commands or undefined variables fail at validation, not at run time.
- **NL-2 Hard constraints are absolute**: `!!NEVER` / `!!ALWAYS` rules halt execution immediately on violation; no command handler, conditional branch, or caller may suppress or catch them.
- **NL-3 Soft preferences are advisory**: `!PREF` rules are inputs to the execution environment; they never override `!!` rules; a step may suppress a preference locally using `!OVERRIDE`.
- **NL-4 Validate-before-run**: a workflow with any block-class validation error must not execute; invoking execution on an invalid workflow is an implementation error.
- **NL-5 Bounded loops**: `~UNTIL` requires `MAX:n` declared at the loop site; a `~UNTIL` without `MAX:n` is a validation error. `~FOR` over a finite collection is intrinsically bounded.
- **NL-6 Dual representation**: compact form and human form are semantically equivalent; compact → human → compact must produce an AST-equal result; compact is the canonical form.
- **NL-7 Closed value type system**: the runtime value space is finite: Null, Bool, Int, Float, Text, List, Map. No dynamic type extension is permitted.
- **NL-8 Reserved variable namespace**: a fixed set of variable names is owned by the runtime; user-defined pipeline targets must not shadow them.
- **NL-9 Typed I/O contract**: `@in`, `@out`, `@ctx`, and `@err` are the public contract of a workflow; an implementation must enforce input presence and type at run time.
- **NL-10 Sequential pipeline**: steps execute in declared order; a pipeline target (`→ $x`) is available from the step immediately following its definition; forward references are a validation error.
- **NL-11 Provenance-safe interpolation** [ADDED v1.2.0]: a value interpolated into a **model-facing** string (a prompt built for a model-calling command such as `GEN`/`REFINE`) carries a provenance — *trusted* (a workflow literal or a value the author declared trusted) or *untrusted* (anything derived from a model output, an `ASK` answer, an external fetch, or a memory read; unknown provenance is untrusted). An untrusted value is **neutralized-as-data by default** at render — inserted so it cannot be read as instructions or prompt structure — and inserting it raw requires an **explicit modifier** at the interpolation site, never omission. Provenance is sticky: a value derived from an untrusted value is untrusted unless explicitly elevated. Nodus defines the provenance rule and the default-neutralize boundary; the concrete neutralization mechanism (encode/escape/delimit) is **host-supplied** (LP-2 style), so the core stays host-neutral. Interpolation into non-model-facing text is unaffected.
- **NL-12 Deferred / external step execution** [ADDED v1.3.0]: a step MAY **suspend the run pending an external completion** — a long-running or externally-performed operation whose result arrives later — instead of completing synchronously. It reuses the existing suspend/resume lifecycle (`Status::Paused` + a resume descriptor, the same machinery `ASK`/`CONFIRM`/`!PAUSE` use, `l1-nodus-dialog` DG-4), but **generalizes the resume trigger from a human answer to any host-supplied completion correlated to the suspended step** — a backgrounded tool result, a delegated/external execution, or a human-performed action. The run is never busy-blocked: a suspended step yields `Status::Paused` with a resume descriptor carrying a **correlation token**; the host performs or awaits the operation and resumes the run by supplying the correlated completion; resumption is deterministic (same completion → same continuation), and a completion for an unknown or already-consumed token is rejected, never applied. Nodus defines only the suspension + correlation + determinism contract; *whether* the operation runs in the background, is delegated, or is persisted across invocations is **host-supplied** (LP-1/LP-2) — the language names no scheduler, thread, or transport, and a deferred step declares its need as a host capability (LP-8) so a host that cannot satisfy it is rejected fail-fast, not mid-run. A step that is neither synchronous nor backed by a declared external-completion capability fails fast with a `NODUS:*` runtime error, never hangs. This is the nodus realization of the main `l1-execution-graph` EG-12 deferred-execution contract; a human dialog (`l1-nodus-dialog`) is the special case where the external actor is a person.

- **NL-13 Competitive parallel selection** [ADDED v1.4.0]: a `~PARALLEL` block MAY declare a **selection** discipline that binds **exactly one winning branch result** instead of joining all branches. Default `~PARALLEL … ~JOIN → $t` collects **every** branch result into `$t` and is all-must-succeed fail-fast (§4.4). A **competitive** `~PARALLEL` runs its N branches over the same input and binds into `$t` the single result chosen by a **declared selector**: `first-success` (the first branch to complete successfully wins; the rest are cancelled) or `best` (a **host-supplied** scorer/judge ranks the completed branch results and the highest wins). Non-winning branches are **cancelled and surfaced** in the trace as *superseded* — never silently dropped and **never merged** into `$t` (composing one whole winner, not a blend). The ranking policy is host-supplied (LP-2 style, host-neutral — the language names no metric or judge); a `best` selector that needs an external scorer/judge declares it as a host capability (LP-8) and is rejected fail-fast when unsatisfied. Determinism is preserved: because `~PARALLEL` is a scheduling hint that a runtime MAY execute sequentially (§2), `first-success` resolves to **declaration order** under sequential execution, and `best` is deterministic whenever the host selector is deterministic (NL-6). Fail-semantics: `first-success` fails only when **all** branches fail; `best` requires **≥1** successful branch — an all-branches-fail competitive block routes to `@err:` with a typed `NODUS:*` code exactly like the fail-fast `~JOIN` path, and `!!` violations still bypass `@err:` per NL-2. This is a selection discipline layered on `~PARALLEL`, not a new command. It is the nodus realization of the main `l1-competitive-execution` best-of-N contract (select one whole winner, discard the losers, CE-2/CE-4/CE-5); `~JOIN`'s integrate-all remains the analog of parallel-staffing.

- **NL-14 Grader-gated iterative refinement** [ADDED v1.5.0]: a `~UNTIL … MAX:n` loop MAY declare a **`+grade` discipline** whose continuation is decided by a **host-supplied grader verdict over the artifact the loop body produces** (the latest `GEN`/`REFINE` output bound in the loop), instead of a bare boolean expression. On each iteration the grader — reading only the artifact and a **fixed rubric**, never the producer's working state — returns a verdict {*pass* | *continue*}; a *continue* verdict's **structured feedback is bound into a reserved loop variable** the next iteration's `REFINE` reads (feedback threading), so every cycle targets the unmet criteria. The rubric/criteria are **immutable for the loop's lifetime**: the workflow may revise the artifact but MUST NOT rewrite, weaken, or reinterpret the criteria that judge it — a step may *request* an outcome but never self-author its own pass contract (LP-10), so there is no goalpost drift. The loop stays bounded by `MAX:n` (NL-5): it terminates on *pass* or on `MAX:n` exhaustion, and an exhausted loop binds the **best-so-far** artifact and routes the **named unmet criteria** to `@err:` with a typed `NODUS:*` code (`NODUS:MAX_REACHED` or a refinement code) — never open-ended and never a silent give-up. The grader is a **function-scoped auxiliary role, host-supplied** (LP-2 — the core names no rubric vocabulary and no metric), declared as a host capability (LP-8) when it needs an external judge and rejected fail-fast when unsatisfied; determinism is preserved whenever the grader is deterministic (NL-6). A plain `~UNTIL cond | MAX:n` (boolean predicate) is unchanged and `+grade` is opt-in (additive). This is a discipline layered on `~UNTIL`, not a new command (§4.4). It is the nodus realization of the main `l1-iterative-refinement` generate–evaluate–refine contract — one artifact improved in place (IR-1), producer/grader independence (IR-2), rubric-actionable feedback (IR-3), bounded honest convergence (IR-5), and frozen criteria (IR-6); the **sequential-depth** complement to NL-13's **competitive width** (`~UNTIL +grade` deepens one artifact; `~PARALLEL +select` selects among many).

- **NL-15 Cache-stable prompt composition** [ADDED v1.6.0]: nodus renders a model-facing prompt (a `GEN`/`REFINE` input) so that its **reusable segments form a byte-stable leading prefix** across repeated invocations, with per-invocation volatile content confined to the **suffix**. The durable segments — the system preamble, the loaded `§schema`/vocabulary, macro definitions, and a loop-invariant `+grade` rubric — are emitted **first and in a deterministic, stable byte order**, so the same preamble + schema + macros always produce the same prefix bytes; the volatile content — the current `@in`, prior-iteration feedback (NL-14), loop state, freshly interpolated values — is appended **after** the prefix. Because the prefix stays byte-identical across the iterations of a `~UNTIL +grade` / `~RETRY` loop and across successive steps that share it, a **host prefix-addressed cache** (a durable local cache or a provider prompt cache) can reuse the already-ingested prefix state instead of recomputing it. Nodus defines only the **ordering discipline and the prefix/volatile boundary**; whether and how the prefix is cached is entirely **host-supplied** (LP-2 — the core names no cache, tier, or block), so a host with no cache is unaffected (**additive**) and a host with one hits. The discipline composes NL-6 (deterministic rendering is what makes the prefix byte-stable) and NL-11 (an untrusted value neutralized-as-data belongs in the volatile suffix so it never destabilizes the shared prefix; a value promoted into the frozen prefix must itself be stable), and it makes the resulting cache economy observable through the existing `l1-nodus-observability` HO-8 `cache_read`/`cache_creation` token classes. It is the nodus realization of the main `l1-inference-cache` prefix-addressed reuse contract (IC-1/IC-9) and the workflow-side realization of `l1-cache-stable-context`'s frozen-prefix discipline — the authoring guarantee that lets recompute become lookup.

- **NL-16 Selective vocabulary disclosure** [ADDED v1.7.0]: a workflow declares, via `@needs` in its `§runtime:`, the specific schema/vocabulary units (named sections of an `extends:` host schema) it actually uses, and the runtime loads **only the declared units** into the validation and execution context — the rest of the host vocabulary stays **deferred** (not loaded). A workflow that draws on a small slice of a large host schema therefore pays, in resident context, only for that slice, not the whole vocabulary (§4.6). A schema's `§meta` and `!!` rules **always load** (they bound safety and cannot be opted out of), but every optional command group is disclosed **on demand**. Using an **undeclared** unit is a **fail-fast validation error** — you must declare what you disclose, so the loaded set is explicit and auditable (LP-8 kinship: a needed vocabulary unit is a declared capability, resolved before the run, not discovered mid-run). The disclosed set is **deterministic** (same `@needs` → same loaded vocabulary, NL-6) and **cache-stable** (the loaded vocabulary is part of the byte-stable prefix, NL-15), so selective loading never destabilizes the prompt cache. Nodus defines only the *declare-and-load-the-slice* discipline; which units a host schema offers and how they are stored is host-supplied (LP-4/LP-2). This **elevates** the `@needs` selective-schema-loading feature (previously only a §4.6 note) to a first-class contract, and it is the nodus realization of the main `l1-progressive-disclosure` minimal-load contract — the always-resident descriptor is the schema's mandatory `§meta` + `!!` core (PD-1), an optional command group is a deferred body expanded only when `@needs` declares it (PD-2), and declaring only the needed slice is selective loading that bounds context rot (PD-5).

- **NL-17 Origin-taint provenance** [ADDED v1.8.0]: a value's provenance (NL-11) MAY additionally carry a host-supplied **origin taint** — a label classifying the value's source-trust (e.g. *internal-authored* vs *external-synced*), attached when the value enters the run (a memory read, an external fetch, a synced import). The taint is **sticky and propagates** exactly like NL-11 provenance — a value derived from a tainted value carries the taint, and combining values takes the **most-untrusted** taint — so it rides every derivation without a workflow tracking it by hand. Crucially, the taint **refines but NEVER relaxes** the NL-11 boundary: an external/unknown-origin value stays **untrusted and neutralized-as-data by default** regardless of taint detail — the taint only records *which* external origin (for host policy, audit, and the observability trace), it never licenses raw insertion — and a workflow can **never upgrade its own taint** (untrusted → trusted): an upgrade is a host-supplied, human-rooted decision (LP-10 — the workflow declares and reads authority, it never self-grants). The origin taxonomy, the store of taints, and any upgrade authority are entirely **host-supplied** (LP-2 — the nodus core names no origin class); a host supplying no taint leaves NL-11 behaviour exactly as today (**additive**). This is the nodus realization of the main `l1-provenance-taint` at-rest origin-trust contract — its propagation-through-derivation (PT-4/PT-5, the sticky most-untrusted-wins rule at the value level) and its taint-sets-runtime-provenance-on-recall (PT-6, a tainted memory read enters neutralized) — the workflow-side carrier of the origin taint the host tracks at rest, with nodus contributing propagation plus the preserved safe default, never a trust-authority.

- **NL-18 Bounded recursive decomposition** [ADDED v1.9.0]: a macro or sub-workflow invocation (`RUN`) MAY **recurse** — invoke itself, or another workflow, over a **piece** of its input — and any recursion MUST declare a **mandatory depth bound** (`MAX_DEPTH:n`, the recursive analog of NL-5's `MAX:n` for loops): a recursion with no declared depth bound is a **validation error** (as an unbounded `~UNTIL` is), and exceeding the bound at run time is a **fail-fast** `NODUS:*` error (`NODUS:MAX_DEPTH_REACHED`), never an unbounded blow-up — keeping nodus bounded / non-Turing-complete by design (§6). The recursive shape is a **map-then-reduce over bounded pieces**: `~MAP` slices the input and applies the recursive call per piece, `~JOIN` reduces the child results, and a reduce over too many results is itself bounded. Each recursive child is **correlated to its parent** — a child run carries a parent/root correlation (extending `l1-nodus-observability` HO-7) so the recursion tree's events and cost **roll up** (HO-8 per-subtree cost) and the tree is observable; per-piece derivation rides the reduction (HO-13). Determinism holds: same input + same slicing → same piece boundaries → same tree (NL-6). Nodus contributes only the **bounded-recursion control-flow, the mandatory depth bound, and the child correlation**; *whether* an input is large enough to warrant decomposition and *how* the model navigates it as an environment are host concerns (LP-1/LP-2 — the core names no window size and no context metric), and a workflow that never recurses is unchanged (**additive**). This is the nodus realization of the main `l1-recursive-decomposition` contract — bounded recursive processing over pieces (RD-2/RD-4), map-reduce combination (RD-3), an observable cost-rolled-up tree (RD-5), and deterministic slicing (RD-7).

- **NL-19 Unforgeable frame markers & seam-anchored segmentation** [ADDED v1.10.0]: where the host's model-facing input distinguishes a **control alphabet** — frame, turn, role, channel, and stop markers — nodus treats that alphabet as **host-owned and workflow-unreachable**. No interpolated value, trusted or untrusted, can **emit** a control marker; a value whose rendered text merely *resembles* one is **refused by default** at the render boundary, and thereafter either rendered **inertly** as ordinary content or **minted** as the marker, per the host's declared per-surface disposition — never silently promoted, never silently inerted. This is the **structural completion of NL-11**: NL-11 neutralizes an untrusted value *as data*, and NL-19 makes that neutralization unforgeable by removing the capability rather than escaping the text. It is LP-10 restated at the alphabet grain — a workflow may **request** a frame (a step, a dialog, a deferred completion) but can never **mint** one, so a workflow assembled under untrusted influence cannot fabricate a turn boundary and escalate past its pre-consented manifest.

  Because control markers are unforgeable, **they are the only cut points no continuation can move** — the two properties are one fact seen from two sides. NL-15's prefix/volatile boundary is therefore **anchored at such a seam**, not at an arbitrary byte offset: the byte-stable prefix NL-15 guarantees is **necessary but not sufficient** for host cache reuse, because the encoder MAY merge the prefix's trailing content with the suffix's leading content and re-encode symbols the prefix already emitted — every byte preserved, the symbol sequence changed. A prefix cut at a seam the host does not declare stable **does not license cache reuse** and MUST NOT be presented as reusable. Nodus contributes only the **seam-anchoring discipline** and the **workflow-unreachability of the control alphabet**; the alphabet itself, its markers, the encoder, the seam-stability assertion, and the resemblance disposition are entirely **host-supplied** (LP-2 — the core names no tokenizer, no control symbol, and no encoding), so a host whose prompt surface is flat (no control alphabet) is unaffected and a workflow that renders no prompt is unchanged (**additive**). Determinism (NL-6) is untouched: seam anchoring constrains *where* a cut may fall, never *what* is rendered. This is the nodus realization of the main `l1-tokenization-boundary` contract — non-composable encoding (TB-2), cut-only-at-declared-stable-seams (TB-3), the disjoint unreachable control sub-alphabet (TB-4), and minting-as-authority with refusal as the default disposition (TB-5) — and the token-level completion of NL-15's byte-stable prefix, mirroring main `l1-inference-cache` IC-10 and `l1-cache-stable-context` CSC-12.

## 4. Detailed Design

### 4.1 File types

A `.nodus` source file begins with a typed sigil header:

```text
[REFERENCE]
§wf:name vX.Y      — Workflow (executable; requires @steps:)
§schema:name vX.Y  — Vocabulary + rule schema (no @steps:)
§config:name vX.Y  — Runtime configuration overlay
```

File type determines which section declarations are permitted. A schema file contains only rule and vocabulary definitions; a config file contains only runtime-binding overrides.

### 4.2 Section declarations

| Section | Required | Purpose |
| --- | --- | --- |
| `§runtime: { core: … }` | Mandatory (`§wf`) | Loads the named schema; `extends:` for overlays; `agents:` for model bindings |
| `@ON: cond → action` | Optional (≥0) | Reactive trigger — workflow activates when condition holds |
| `!!NEVER: …` / `!!ALWAYS: …` | Optional (≥0) | Absolute hard constraint (NL-2) |
| `!PREF: X OVER Y IF cond` | Optional (≥0) | Soft preference (NL-3) |
| `@in: { field: type }` | Recommended | Input contract; `?` suffix = optional; `= default` = fallback |
| `@out: $var` | Recommended | Output binding |
| `@ctx: [key, …]` | Optional | Required context keys |
| `@err: HANDLER` | Optional | Uncaught-error handler |
| `@steps:` | Mandatory (`§wf`) | Sequential numbered step body |
| `@test: name { … }` | Optional (≥0) | Inline test case (`input:` / `expected:` / `tags:`) |
| `@macro: name` | Optional (≥0) | Reusable step-sequence macro |

### 4.3 Step syntax

```text
[REFERENCE]
N. COMMAND(args) +modifier=value ^validator ~flag → $target
     │            │               │           │       │
     │            │               │           │       └── pipeline binding (optional)
     │            │               │           └────────── flag extractor (optional, repeatable)
     │            │               └────────────────────── output validator rule (optional, repeatable)
     │            └────────────────────────────────────── named modifier (optional, repeatable)
     └─────────────────────────────────────────────────── command from the schema vocabulary (NL-1)
```

Step numbers are positive integers; numbers need not be consecutive but must be monotonically increasing within `@steps:`.

Macro invocation — expands a `@macro:` body in place of a step:

```text
[REFERENCE]
N. RUN(@macro_name) +modifier=value → $target
```

`RUN` is a built-in meta-command reserved for macro expansion. It accepts the
`@macro:` name (sigil included) as its sole argument. Standard `+modifier` and
`→ $target` decorators apply; `^validator` and `~flag` decorators are not
permitted on macro steps. The transpiler renders macro invocations as
`Run macro: macro_name` in human form.

`RUN` is not a domain command and does not appear in the schema vocabulary table.
Conforming implementations must recognize `RUN(@…)` before the schema validation
pass so that macro steps are not rejected as unknown commands.

### 4.4 Control flow

```text
[REFERENCE]
?IF cond → action         — conditional branch
?ELIF cond → action       — additional branch
?ELSE → action            — default branch

~FOR $var IN $collection  — iterator loop over a finite collection
  …
~END

~UNTIL cond | MAX:n       — bounded convergence loop (NL-5: MAX:n mandatory)
  …
~END

~PARALLEL                 — parallel block (scheduling hint)
  …
~JOIN → $target           — collect ALL branch results into $target (all-must-succeed)

~PARALLEL +select=first-success | best   — competitive: bind ONE winning branch (NL-13)
  …
~PICK → $target           — bind the selected winner; losers cancelled + surfaced, never merged
```

`~JOIN` integrates every branch (the parallel-staffing analog); `~PICK` under a
`+select` selector keeps exactly one whole winner and discards the rest (NL-13, the
competitive-execution analog). `best` uses a host-supplied scorer/judge (LP-2); a
selector needing one declares it as an LP-8 capability.

```text
~UNTIL +grade | MAX:n     — iterative refinement: continue on a host-supplied grader
  …                          verdict over the produced artifact (NL-14); a *continue*
~END                         verdict threads feedback into the next REFINE; exhausting
                             MAX:n binds best-so-far and routes unmet criteria to @err:
```

`~UNTIL +grade` deepens **one** artifact under a frozen rubric (the sequential
iterative-refinement analog, NL-14) — the depth complement to `~PARALLEL +select`'s
width. The grader and its rubric are host-supplied (LP-2); a grader needing an
external judge declares it as an LP-8 capability. A plain boolean `~UNTIL cond | MAX:n`
is unchanged — `+grade` is opt-in.

Branch-level flags:

- `!BREAK` — exit the enclosing loop immediately
- `!SKIP` — continue to the next loop iteration
- `!OVERRIDE` — suppress a `!PREF` rule for this branch only (NL-3)

**Parallel error propagation**: if any branch in a `~PARALLEL` block raises an
error, the block enters fail-fast state — `~JOIN` is bypassed and the error is
forwarded to the `@err:` handler (or terminates with `NODUS:UNHANDLED_ERROR`
if no handler is declared). `NODUS:RULE_VIOLATION` bypasses `@err:` entirely
per NL-2. Runtimes that execute `~PARALLEL` branches sequentially (permitted by
the "scheduling hint" clause) apply the same fail-fast semantics.

> **v0.7 target additions (see §4.6):** `?SWITCH` multi-branch dispatch, `~MAP` collection transform, `~RETRY:n` step retry, `!HALT` (fatal), `!PAUSE` (suspend).

### 4.5 Error model

A runtime error carries a typed `NODUS:*` code. The `@err:` handler is invoked for uncaught step errors. If no handler is declared, an unhandled error terminates the workflow with `NODUS:UNHANDLED_ERROR`. Hard-constraint violations (`!!`) bypass the error handler entirely and terminate with `NODUS:RULE_VIOLATION`.

| Code | Trigger |
| --- | --- |
| `NODUS:RULE_VIOLATION` | `!!` constraint violated (bypasses `@err:`) |
| `NODUS:PARSE_ERROR` | Source failed to parse |
| `NODUS:MAX_REACHED` | `~UNTIL` exhausted its `MAX:n` bound |
| `NODUS:EXECUTION_FAILED` | Step failed at run time |
| `NODUS:UNDEFINED_VAR` | Variable referenced before assignment |
| `NODUS:ROUTE_NOT_FOUND` | `ROUTE(wf:name)` target does not exist |
| `NODUS:RULE_CONFLICT` | Two `!!` rules contradict each other |
| `NODUS:SCHEMA_MISMATCH` | Schema version incompatible with workflow header |
| `NODUS:NO_SCHEMA` | Workflow executed without a loaded schema |
| `NODUS:NO_TRIGGER` | No `@ON:` matched the current input |
| `NODUS:UNHANDLED_ERROR` | Step error reached no `@err:` handler |

### 4.6 Upstream parity gaps (schema v0.4.6 → v0.7)

<!-- [ADDED] v1.1.0 -->

§4.2–§4.5 specify the **v0.4** generation of the language. The upstream schema has advanced to **v0.7**; the constructs below are defined upstream but not yet here. They are the design target for this spec and its implementation; the crate-side gap is itemized in `l2-nodus-runtime.md` §4.7.

#### Control constructs (extends §4.4)

| Construct | Semantics |
| --- | --- |
| `?SWITCH $v:` with `value → action` arms and an optional `* → default` | multi-branch dispatch on a scalar; first match wins, no fallthrough; no match and no `*` → `NODUS:SWITCH_NO_MATCH` (warn, continue) |
| `~MAP $coll: CMD($it) → $out` | single-line collection transform; implicit `$it`; empty collection → `[]`, never errors |
| `~RETRY:n` (+`backoff`, +`retry_on`) | step-level retry up to `n` (mandatory, max 10); on exhaustion routes to `@err:` |
| `!HALT` | fatal stop; status `FAILED`; requires `ESCALATE()` in the same step; no auto-resume |
| `!PAUSE` | suspend; status `PAUSED`; resumes only on explicit human re-trigger |

#### Operators & expressions

| Element | Semantics |
| --- | --- |
| `MATCHES` | deterministic regex operator in conditions (PCRE; `(?i)` prefix for case-insensitive) |
| `?.` | optional chaining — null-safe path access; short-circuits to `null` without raising `NODUS:UNDEFINED_VAR` |
| `??` | null-coalescing fallback (`$a?.b ?? default`) |
| `WHERE` / `FIRST` / `LAST` | inline collection filter/access with implicit `$it`; no match → `[]` (WHERE) or `null` (FIRST/LAST) |
| String interpolation | `$var` / `$obj.field` expand inside string literals before the step runs (runtime-resolved, not by the model); `\$` suppresses; interpolation into a model-facing prompt is provenance-safe (NL-11) — untrusted values neutralized-as-data by default, raw insertion needs an explicit modifier |

#### Human-in-the-loop dialog commands (new command class)

| Command | Semantics |
| --- | --- |
| `ASK(prompt)` | blocking typed question that auto-resumes on answer; `+type` = str/bool/confirm/choice/multi_choice, plus `+options`/`+hint`/`+default`/`+validate`/`+timeout` (→ `NODUS:DIALOG_TIMEOUT`) |
| `CONFIRM(content)` | approval decision; `+msg`/`+actions`/`+default`/`+strict` (reject under `+strict` → `NODUS:DIALOG_REJECTED`) |

#### Selective schema loading

`@needs:` inside `§runtime:` loads only named sections of an `extends:` schema (flat `[§a, §b]` or keyed `{ "x.nodus": [§a] }` form); a schema's `§meta` and `!!` rules always load regardless. Reduces per-execution schema context.

#### Error taxonomy (extends §4.5: 11 → 24)

Adds `UNDEFINED_CMD`, `UNDEFINED_MACRO`, `VALIDATION_FAILED`, `ESCALATION_FAILED`, `CONFIDENCE_LOW`, `KB_UNAVAILABLE`, `MEMORY_FAILED`, `TEST_FAILED`, `SWITCH_NO_MATCH`, `PAUSED`, `COUNTER_OVERFLOW`, `GIT_UNAVAILABLE`, `DIALOG_TIMEOUT`, `DIALOG_REJECTED` — each carrying a severity (error/warn/info) and category (parse/runtime/validation/routing/memory/test/control/dialog). The current `NODUS:EXECUTION_FAILED` (§4.5) is a non-canonical catch-all superseded by these specific codes.

#### Trigger priority

`@ON(priority=N):` — optional trigger priority; lower `N` = higher priority; default is declaration order.

#### Closed vocabulary registries

Beyond commands, the schema declares closed registries for **analysis flags** (`~`), **validators** (`^`), and **primitive types** that a conforming implementation should validate against (strengthening NL-1). The concrete registry contents are enumerated in `l2-nodus-runtime.md` §4.7(f).

## 5. Drawbacks & Alternatives

- **Not Turing-complete by design**: the `MAX:n` constraint prevents infinite computation; workflows requiring convergence over large iterations must increase `MAX:n` or redesign step logic. This is intentional — safety over expressiveness.
- **Alternative — general scripting language**: rejected; a general scripting language cannot enforce schema-first, bounded-execution, and hard-constraint invariants at the language level.
- **Alternative — YAML-based workflow DSL**: rejected; YAML lacks the typed section grammar, inline validators (`^`), and dual-representation contract needed for auditable AI pipelines.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[RUNTIME]` | `crates/nodus/` | Reference implementation of this spec |
| `[FIXTURES]` | `crates/nodus/tests/fixtures/` | Canonical sample workflows (normative test corpus) |
| `[TOKEN-BOUNDARY]` | `.design/main/specifications/l1-tokenization-boundary.md` | The main contract NL-19 realizes (TB-2…TB-5) |

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.10.0 | 2026-07-10 | Added NL-19 unforgeable frame markers & seam-anchored segmentation — where the host's model-facing input distinguishes a control alphabet (frame/turn/role/channel/stop markers), nodus treats it as host-owned and workflow-unreachable: no interpolated value can emit a control marker, and a value whose rendered text merely resembles one is refused by default and thereafter rendered inertly or minted per the host's declared per-surface disposition, never silently promoted nor silently inerted; the structural completion of NL-11 (neutralization by removing the capability, not by escaping the text) and LP-10 at the alphabet grain (a workflow may request a frame but never mint one, so a workflow assembled under untrusted influence cannot fabricate a turn boundary and escalate past its pre-consented manifest). Because control markers are unforgeable they are the only cut points no continuation can move — one fact seen from two sides — so NL-15's prefix/volatile boundary is anchored at such a seam and not at an arbitrary byte offset: the byte-stable prefix NL-15 guarantees is necessary but NOT sufficient for host cache reuse, since the encoder MAY merge the prefix's trailing content with the suffix's leading content and re-encode symbols the prefix already emitted (every byte preserved, the symbol sequence changed); a prefix cut at a seam the host does not declare stable does not license cache reuse and MUST NOT be presented as reusable. Nodus contributes only the seam-anchoring discipline + the workflow-unreachability of the control alphabet; the alphabet, its markers, the encoder, the seam-stability assertion, and the resemblance disposition are entirely host-supplied (LP-2, no tokenizer/control symbol/encoding in core), so a flat-prompt-surface host is unaffected and a workflow rendering no prompt is unchanged (additive); determinism (NL-6) untouched — seam anchoring constrains where a cut may fall, never what is rendered. The nodus realization of the new main l1-tokenization-boundary contract (TB-2 non-composable encoding / TB-3 cut-only-at-declared-stable-seams / TB-4 disjoint unreachable control sub-alphabet / TB-5 minting-as-authority with refusal as default), and the token-level completion of NL-15's byte-stable prefix mirroring main IC-10 and CSC-12. L1 stays Stable (C9); l2-nodus-runtime carries NL-19 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11…NL-18 precedent). |
| 1.9.0 | 2026-07-09 | Added NL-18 bounded recursive decomposition — a macro/sub-workflow invocation (`RUN`) MAY recurse (invoke itself or another workflow over a piece of its input), and any recursion MUST declare a mandatory depth bound (`MAX_DEPTH:n`, the recursive analog of NL-5's `MAX:n` for loops): a recursion with no depth bound is a validation error, exceeding it at run time is a fail-fast `NODUS:MAX_DEPTH_REACHED`, never an unbounded blow-up — keeping nodus bounded/non-Turing-complete (§6); the recursive shape is a map-then-reduce over bounded pieces (`~MAP` slices + applies the recursive call per piece, `~JOIN` reduces, a too-large reduce is itself bounded); each recursive child carries a parent/root correlation (extending HO-7) so the tree's events + cost roll up (HO-8 per-subtree) and the tree is observable, per-piece derivation riding the reduction (HO-13); determinism holds (same input + same slicing → same tree, NL-6); nodus contributes only the bounded-recursion control-flow + mandatory depth bound + child correlation, whether an input warrants decomposition and how the model navigates it as an environment being host concerns (LP-1/LP-2, no window size or context metric in core), a workflow that never recurses unchanged (additive). The nodus realization of the new main l1-recursive-decomposition contract (RD-2/RD-4 bounded recursive processing over pieces / RD-3 map-reduce combination / RD-5 observable cost-rolled-up tree / RD-7 deterministic slicing). L1 stays Stable (C9); l2-nodus-runtime carries NL-18 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11…NL-17 precedent). |
| 1.8.0 | 2026-07-09 | Added NL-17 origin-taint provenance — a value's provenance (NL-11) MAY additionally carry a host-supplied origin taint classifying source-trust (e.g. internal-authored vs external-synced), attached when the value enters the run (memory read, external fetch, synced import); the taint is sticky and propagates exactly like NL-11 provenance (a value derived from a tainted value carries it; combining takes the most-untrusted taint), so it rides every derivation; it refines but NEVER relaxes the NL-11 boundary — an external/unknown-origin value stays untrusted + neutralized-as-data by default regardless of taint detail (the taint only records which external origin, for host policy/audit/trace, never licensing raw insertion), and a workflow can never upgrade its own taint untrusted→trusted (upgrade is a host-supplied human-rooted decision, LP-10); origin taxonomy + taint store + upgrade authority entirely host-supplied (LP-2, no origin class in core), a host supplying no taint leaving NL-11 exactly as today (additive). The nodus realization of the new main l1-provenance-taint at-rest origin-trust contract (PT-4/PT-5 propagation + most-untrusted-wins at the value level, PT-6 taint-sets-runtime-provenance-on-recall) — the workflow-side carrier of the origin taint the host tracks at rest, nodus contributing propagation + the preserved safe default, never a trust-authority. L1 stays Stable (C9); l2-nodus-runtime carries NL-17 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11…NL-16 precedent). |
| 1.7.0 | 2026-07-09 | Added NL-16 selective vocabulary disclosure — a workflow declares via `@needs` (in `§runtime:`) the specific schema/vocabulary units (named sections of an `extends:` host schema) it uses, and the runtime loads only the declared units into the validation/execution context, the rest of the host vocabulary staying deferred; a schema's `§meta` + `!!` rules always load (safety floor), every optional command group is disclosed on demand; using an undeclared unit is a fail-fast validation error (declare what you disclose, LP-8 kinship: a needed vocabulary unit is a declared capability resolved before the run); the disclosed set is deterministic (same `@needs` → same vocabulary, NL-6) and cache-stable (loaded vocabulary is part of the byte-stable prefix, NL-15); nodus defines only the declare-and-load-the-slice discipline, the host schema's units + storage host-supplied (LP-4/LP-2); elevates the `@needs` selective-schema-loading feature from a §4.6 note to a first-class contract. The nodus realization of the new main l1-progressive-disclosure minimal-load contract (PD-1 the mandatory `§meta`+`!!` core as always-resident descriptor / PD-2 an optional command group as a deferred body expanded only when `@needs` declares it / PD-5 selective loading bounds context rot). L1 stays Stable (C9); l2-nodus-runtime carries NL-16 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11…NL-15 precedent). |
| 1.6.0 | 2026-07-09 | Added NL-15 cache-stable prompt composition — nodus renders a model-facing `GEN`/`REFINE` prompt so its reusable segments (system preamble, loaded `§schema`/vocabulary, macro definitions, a loop-invariant `+grade` rubric) form a byte-stable leading prefix emitted first in deterministic stable order, with per-invocation volatile content (current `@in`, prior-iteration feedback, loop state, freshly interpolated values) confined to the suffix; because the prefix stays byte-identical across `~UNTIL +grade`/`~RETRY` iterations and across steps that share it, a host prefix-addressed cache (durable local or provider prompt cache) reuses the ingested prefix state instead of recomputing it; nodus defines only the ordering discipline + prefix/volatile boundary, the cache being entirely host-supplied (LP-2, no cache/tier/block in core) so a host with no cache is unaffected (additive); composes NL-6 (deterministic rendering makes the prefix byte-stable) and NL-11 (untrusted neutralized-as-data belongs in the volatile suffix so it never destabilizes the shared prefix), and surfaces the cache economy through the existing HO-8 `cache_read`/`cache_creation` classes. The nodus realization of the new main l1-inference-cache prefix-addressed reuse contract (IC-1/IC-9) and the workflow-side realization of l1-cache-stable-context's frozen-prefix discipline — the authoring guarantee that lets recompute become lookup. L1 stays Stable (C9); l2-nodus-runtime carries NL-15 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11/NL-12/NL-13/NL-14 precedent). |
| 1.5.0 | 2026-07-09 | Added NL-14 grader-gated iterative refinement — a `~UNTIL … MAX:n` loop MAY declare a `+grade` discipline whose continuation is decided by a host-supplied grader verdict over the produced artifact (latest `GEN`/`REFINE` output) instead of a bare boolean; a *continue* verdict threads structured feedback into a reserved loop variable the next `REFINE` reads (feedback threading), so each cycle targets the unmet criteria; the rubric/criteria are immutable for the loop's lifetime — the workflow revises the artifact but never rewrites the criteria that judge it (LP-10, no goalpost drift); bounded by `MAX:n` (NL-5), terminating on *pass* or exhaustion (exhaustion binds best-so-far and routes named unmet criteria to `@err:` typed, `NODUS:MAX_REACHED`/refinement code, never open-ended); grader is a function-scoped host-supplied auxiliary role (LP-2, no rubric vocabulary or metric in core), declared as an LP-8 capability when it needs an external judge and rejected fail-fast when unsatisfied; determinism preserved when the grader is (NL-6); a plain boolean `~UNTIL` is unchanged and `+grade` is opt-in (additive). A discipline layered on `~UNTIL`, not a new command; §4.4 `~UNTIL +grade` note added. The nodus realization of the new main l1-iterative-refinement generate–evaluate–refine contract (IR-1/IR-2/IR-3/IR-5/IR-6); the sequential-depth complement to NL-13's competitive width (`~UNTIL +grade` deepens one artifact; `~PARALLEL +select` selects among many). L1 stays Stable (C9); l2-nodus-runtime carries NL-14 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11/NL-12/NL-13 precedent). |
| 1.4.0 | 2026-07-04 | Added NL-13 competitive parallel selection — a `~PARALLEL` block MAY declare a `+select` discipline (`first-success` / `best`) that binds exactly one winning branch result via `~PICK` instead of `~JOIN`'s integrate-all; non-winning branches cancelled + surfaced as superseded, never merged (one whole winner, not a blend); ranking policy host-supplied (LP-2, `best` needs an LP-8 capability); determinism preserved (`first-success` = declaration order under sequential execution §2, `best` deterministic when the selector is); fail-semantics (`first-success` fails only if all fail, `best` needs ≥1 success, all-fail routes to `@err:` typed, `!!` still bypasses per NL-2). A selection discipline on `~PARALLEL`, not a new command; §4.4 `~PICK` note added. The nodus realization of the new main l1-competitive-execution best-of-N contract (CE-2/CE-4/CE-5); `~JOIN` integrate-all remains the parallel-staffing analog. L1 stays Stable (C9); l2-nodus-runtime carries NL-13 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11/NL-12 precedent). |
| 1.3.0 | 2026-07-02 | Added NL-12 deferred / external step execution — a step MAY suspend the run pending an external completion (backgrounded tool result, delegated/external execution, human-performed action), reusing the `Status::Paused` + resume-descriptor lifecycle (DG-4) but generalizing the resume trigger from a human answer to any host-supplied completion correlated to the suspended step by a token; deterministic resume, stray/consumed-token completion rejected, host owns background/delegation/persistence (LP-1/LP-2) and declares capability via LP-8, fail-fast (never hang) when neither synchronous nor capability-backed. The nodus realization of the new main l1-execution-graph EG-12 deferred-execution contract; a human dialog is the special case where the external actor is a person. L1 stays Stable (C9); l2-nodus-runtime carries NL-12 as a pending Invariant-Compliance obligation reconciled at magic.task (NL-11 precedent). |
| 1.2.0 | 2026-07-02 | Added NL-11 provenance-safe interpolation — a value interpolated into a model-facing prompt (`GEN`/`REFINE`) carries trusted/untrusted provenance (unknown=untrusted; model-output/`ASK`-answer/external-fetch/memory-read = untrusted); untrusted is neutralized-as-data by default at render, raw insertion needs an explicit modifier, provenance is sticky; neutralization mechanism host-supplied (LP-2 style, host-neutral). The nodus realization of the main l1-context-provenance contract (CP-2/CP-3/CP-4). §4.6 interpolation row annotated. L1 stays Stable (C9); l2-nodus-runtime carries NL-11 as a pending Invariant-Compliance obligation reconciled at magic.task (RTG-9 precedent). |
| 1.1.0 | 2026-06-25 | Added §4.6 upstream parity gaps (schema v0.4.6 → v0.7): control constructs (`?SWITCH`/`~MAP`/`~RETRY`/`!HALT`/`!PAUSE`), operators/expressions (`MATCHES`, `?.`, `??`, `WHERE`/`FIRST`/`LAST`, string interpolation), HITL dialog command class (`ASK`/`CONFIRM`), `@needs:` selective schema loading, error taxonomy 11 → 24, `@ON(priority=N)`, closed flag/validator/type registries; §4.4 target-additions note |
| 1.0.1 | 2026-06-23 | Added macro invocation syntax (`RUN(@macro_name)`) to §4.3; added `~PARALLEL` fail-fast error propagation semantics to §4.4 |
| 1.0.0 | 2026-06-23 | Initial spec — language invariants NL-1..NL-10, file types, section grammar, step syntax, control flow, error taxonomy |

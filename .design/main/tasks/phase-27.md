---
phase: 27
name: "Invocable Registry & CLI Derivation"
status: Todo
subsystem: "crates/contract · crates/domain · crates/core · crates/cli"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 27 Tasks — Invocable Registry & CLI Derivation

**Phase:** 27
**Status:** Todo
**Strategic Goal:** Mint one runtime invocable registry in the core, dispatch every action through it once, and move the command line onto it as a **projection** — so command parity stops being a property that is asserted and becomes one that is structural, and a verb contributed by an extension becomes reachable at all.

## Phase Notes

**Behaviour-preservation is the acceptance property, not a nice-to-have.** Every task in tracks A–D must leave observable command output unchanged. Where a task uncovers behaviour that is wrong, it is recorded as a **residual at the owning invocable** and left working as-is; the corrections ship in Phase 29 as separate disclosed changes (SP-10). A diff that both unifies and corrects presents a regression to users as a refactor to reviewers, and neither audience can read it.

**Four residuals are already known and must be preserved through this phase:** a store failure reported as an empty listing with a success exit code; the output-format flag discarded in nine sites; structured output assembled by unescaped string formatting in five; and a redaction path constructed with an empty secret list on every surface. Preserve, record, do not fix here.

**Build environment.** Anything that triggers C compilation (`rusqlite` `bundled`) or a Tauri/`windres` step must run in **PowerShell**, not Git Bash — MSYS2 makes `cc1.exe` fail to load there, so a clean check from Bash that suddenly fails at a C step is an environment artifact, not a code defect.

**The specification moved after tracks A and B shipped — read this before opening any remaining task.** T-27A01…T-27B04 were built against `l2-invocable-registry` 1.0.0 and are **correct at that scope**; their `[x]` stands and is not reopened. The spec is now 1.1.0, and the delta is scheduled as three explicit retrofit tasks (T-27A04, T-27B05, T-27B06) rather than folded silently into C or D. What changed, concretely: an `Installation` locus exists; identity construction is fallible; a descriptor is bounds-checked and detached at registration; `register` returns the effect that reverses it; the bare-form collision rule is **shadowing, not displacement**, and is reported; an unknown invocable is a **resolution** result rather than an `Outcome`; the registry announces its own mutations; a resolved dispatch is journaled as a pair.

**Track independence.** A → B → **retrofit (A04, B05, B06)** → (C ∥ D) → T, with **one crossing edge**: T-27D03 registers the command line against the corpus and therefore needs T-27C01. So C and D are file-independent and may run in parallel once the retrofit lands, but D cannot *finish* before C01 does. Treat C01 as the earliest task in track C rather than assuming the tracks are fully independent.

**The retrofit is on the critical path and is not optional sequencing.** The corpus asserts on the resolution/outcome split, and the generated parser needs the `Installation` locus to know which half of the verb set it owns. Opening C or D against the 1.0.0 shape means writing code twice.

**The retrofit reaches backwards into files tracks A and B already delivered — this is a known crossing edge, not a surprise.** Three concrete consequences, each already identified so no executor has to rediscover it:

1. **`InvocableId`'s `From<&str>`/`From<String>` impls cannot survive.** A `From` conversion cannot fail, and T-27A04 makes identity construction fallible. They give way to a fallible constructor (`TryFrom`, or a named function returning a result). Every current call site — the facade bootstrap, the dispatcher, and both integration tests — changes.
2. **`register`'s signature changes**, so the two bootstrap call sites and the registration assertions in the facade integration test change with it.
3. **`Dispatcher::dispatch`'s return changes** when resolution splits out, which the facade integration test asserts on directly.

None of these are behaviour changes and none of them touch a *product* behaviour a user observes — but they do mean the retrofit's diff includes edits to tests written by tracks A and B. Say so in the task's `Changes` note rather than letting a reviewer read a modified test as a weakened one.

**Phase-size tripwire (planner audit).** This phase is now **18 tasks**, up from 14, and the largest track has not started. The retrofit is re-sequenced scope rather than new scope, and T-27D01.1 is a split of T-27D01 rather than an addition — so the growth is honest. But the prior audit's warning stands and now has a number attached: **if track D's `.N` splits push this phase past ~24 tasks, split track D into its own phase rather than continuing to grow this one.** The reason to hold them together until then is that T-27T02 proves behaviour preservation across the primitive *and* its first migration; splitting earlier splits that proof.

**Critical path and cascade risk.** Track B is the single point every later task and both later phases depend on. If the registry or dispatch shape is wrong, tracks C and D and all of Phases 28–29 rework. Land B behind its own tests before opening C or D, and prefer discovering a shape problem in B's unit tests over discovering it in the command line's migration.

**Sizing warning (planner audit).** Track D is the largest and most optimistically sized work here: the command line is ~4000 lines across 29 groups, and T-27D01/T-27D02 each touch all of them. Expect to split them by group with `.N` sub-task IDs rather than attempting one sweeping edit — the behaviour-preservation gate (`cli_smoke.rs` unmodified) is far easier to keep green in small increments, and a failed sweep is expensive to bisect.

## Atomic Checklist

- [x] [T-27A01] Invocable descriptor and catalog types in the ports tier
- [x] [T-27A02] Invocation / Outcome / Rejection envelope with four located rejection modes
- [x] [T-27A03] Optional serialization feature on the ports tier
- [x] [T-27B01] Registry: one published registration door, qualified identity, declared collision rule
- [x] [T-27B02] Dispatch: bind-before-invoke over declared binders, returning a structured outcome
- [x] [T-27B03] Contribution safety: failure policy, grant-gated reach, attribution marker
- [x] [T-27B04] Core invocables registered from the facade through the public door; redaction at the boundary
- [ ] [T-27A04] Retrofit: `Installation` locus, validated identity grammar, descriptor bounds
- [ ] [T-27B05] Retrofit: registration returns its own reversal; descriptor normalized and refused, never repaired
- [ ] [T-27B06] Retrofit: resolution split from outcome; change announcement; paired dispatch journal
- [ ] [T-27C01] Conformance fixture library and harness with three assertion families
- [ ] [T-27C02] Finding inventory and the two one-way ledgers, seeded with F-1…F-8
- [ ] [T-27D01] Command-line parser generated from `Semantic` descriptors (semantic half only)
- [ ] [T-27D01.1] The installation half: one closed launcher grammar feeding both the parser and the catalog
- [ ] [T-27D02] One renderer over the structured outcome; uniform output-format handling
- [ ] [T-27D03] Shipped-surface honesty: the five unbound groups leave the default surface; corpus registration
- [ ] [T-27T01] Corpus first run — convert failures into findings before fixing anything
- [ ] [T-27T02] Behaviour-preservation proof and full quality gates

## Detailed Tracking

### [T-27A01] Invocable descriptor and catalog types in the ports tier

- **Spec:** l2-invocable-registry.md §4.1, §4.2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-contract` green; `cargo tree -p cronus-contract --depth 1` lists no external dependency in the default build.
- **Handoff:** T-27A02 (envelope types name these).
- **Notes:** Descriptor carries id, name, summary, group, `locus`, ordered binders, `stability`. The descriptor is the **whole** advertised contract — a surface must be able to render help, completion, and grouping from it alone, because any fact a surface must hold privately is the first step of a fork. `locus` is `Semantic | ClientLocal | HostOnly`; `stability` is `Shipped | Retired { superseded_by }`. Types live in the ports tier because all three surfaces and the domain must name them without depending on each other.
- **Changes:** Added `InvocableId`/`Locus`/`Stability`/`BinderKind`/`Binder`/`Invocable` to `crates/contract/src/lib.rs`, following the file's existing per-seam banner + `#[cfg(test)] mod {seam}_tests` convention (mirrors the `MemoryId`/`ActivationMode` precedent already in the file — no new module file). `Binder`'s declared shape (name/kind/optional) is included here since `Invocable.binders` needs it to compile; binder *matching* behavior stays out of scope for T-27B02. Considered and dropped a `new()` constructor on `InvocableId` as redundant with `From<&str>`/`From<String>` (minimalism). 5 new tests in `invocable_tests`, all 10 pre-existing tests in the crate unmodified and passing. `cargo test -p cronus-contract` 15/15 green; `cargo tree -p cronus-contract --depth 1` shows no dependency; `cargo clippy -p cronus-contract --all-targets -- -D warnings` clean; `cargo fmt -p cronus-contract -- --check` clean.

### [T-27A02] Invocation / Outcome / Rejection envelope with four located rejection modes

- **Spec:** l2-invocable-registry.md §4.5, §4.6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-contract` green, including a test asserting that an empty result and an unavailable source are **distinct** `Outcome` values and cannot be constructed from one another.
- **Handoff:** T-27B02 (dispatch produces these).
- **Notes:** `Outcome` is `Value | Stream | Rejected` and carries **structured data, never rendered text** — that is what lets one dispatch serve a text renderer, a structured renderer, terminal widgets, and IPC without any of them re-deriving the others' content. A `Rejection` names one of absent / unreadable / malformed / ill-shaped plus its location; collapsing the four into one "invalid input" is a defect, because each implies a different corrective action.
- **Decision (recorded, not asked — pure technical realization):** found a real tension between two already-Stable specs on where "resource unavailable" lives: one reads as a distinct `Outcome` variant, the other as "a rejection carries its mode". Resolved via the L1 parent (input-binding's own definition of rejection *location* as a position within a bound *argument*): a backend/store failure binds to no declared binder and has no such location, so forcing it into `Rejection` would either fabricate a location or make location optional and dilute the one guarantee IB-4 exists to give. Added `Outcome::Unavailable { reason }` as a fourth variant instead, keeping `Rejection` strictly binder-scoped with an always-real location. The imprecise spec wording ("a rejection carries its mode") needs a one-line patch-level correction in a future `/magic.spec` pass — flagged, not fixed here (out of this workflow's write scope).
- **Changes:** Added `Surface`/`ArgValue`/`ArgValues`/`Invocation`/`StreamHandle`/`OutcomeValue`/`RejectionMode`/`Rejection`/`Outcome` to `crates/contract/src/lib.rs`, plus a `dispatch_tests` module (5 tests) following the file's existing per-seam convention. `OutcomeValue` is a small bounded tree (Empty/Text/Integer/Boolean/List/Record) — general enough for text/structured/widget/IPC rendering per §4.5's own reasoning, deliberately no Float (no current call site needs one) and no open-ended/serde-style catch-all. `cargo test -p cronus-contract` 20/20 green (10 pre-existing unmodified); `cargo clippy -p cronus-contract --all-targets -- -D warnings` clean; `cargo fmt -p cronus-contract -- --check` clean.

### [T-27A03] Optional serialization feature on the ports tier

- **Spec:** l2-invocable-registry.md §4.1
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p cronus-contract` (default, no serde in the *production* dependency graph — `cargo tree -p cronus-contract --depth 1 -e=no-dev` empty) and `cargo check -p cronus-contract --features serde` both succeed; `cargo test -p cronus-contract --features serde` exercises `Serialize` on a descriptor and every `Outcome` variant. *(Revised in-flight from a round-trip test — see Decision below.)*
- **Handoff:** Phase 29 (the desktop shell enables it; `Invocation`/`ArgValues` gain `Deserialize` there, when the generic-dispatch IPC command actually needs the JS→Rust half).
- **Notes:** The ports tier is dependency-free **by construction** and its manifest carries an empty dependency section — so serialization is a feature, off by default, never an unconditional dependency. Landing it now rather than when the desktop needs it is SP-9: extraction before the second consumer is the only cheap point on the ladder.
- **Decision (recorded, not asked — corrects my own over-specification):** the original Verify line asked for a full round-trip, which turned out to be unachievable *and* unneeded. `Invocable.name/summary/group`, `Binder.name`, `Locus::HostOnly.reason`, and `Rejection.binder` are all `&'static str` — compile-time literals that cannot be honestly reconstructed from arbitrary wire input without leaking memory per deserialized instance, so `Deserialize` fails to compile on this type shape by construction. Checked against the real IPC direction (`l2-application-shell` §4.3): `catalog() -> Promise<Invocable[]>` and the return half of `invoke() -> Promise<Outcome>` both cross **Rust → JS only** — only `Invocation` (the argument JS supplies) crosses JS → Rust, and it was already out of this task's scope. So `Invocable`/`Outcome` need `Serialize` alone; asking for `Deserialize` on them was planning-time over-specification, not a real requirement. Corrected the Verify line to match the direction that actually exists, rather than forcing a leak-based workaround to satisfy an invented symmetry.
- **Changes:** `serde` added to root `Cargo.toml` workspace.dependencies (1.0.228, `derive` feature) and to `crates/contract/Cargo.toml` as `optional = true` behind a `serde` feature (`dep:serde`); `serde_json` added there as a test-only dev-dependency (never affects the production graph or `cargo tree -e=no-dev`). `#[cfg_attr(feature = "serde", derive(serde::Serialize))]` on the 11 descriptor/outcome types from T-27A01/T-27A02. New `serde_tests` module (2 tests, gated `#[cfg(all(test, feature = "serde"))]`) verifying JSON shape and per-variant distinctness. `cargo test -p cronus-contract` 20/20 and `--features serde` 22/22, both green; `cargo clippy -p cronus-contract --all-targets [--features serde] -- -D warnings` clean both ways; `cargo fmt -p cronus-contract -- --check` clean; `cargo check --workspace` unaffected by the root manifest edit.

### [T-27B01] Registry: one published registration door, qualified identity, declared collision rule

- **Spec:** l2-invocable-registry.md §4.3, §4.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain` green, including tests that (a) a contribution cannot claim the reserved core identity, (b) a bare verb colliding with a core verb resolves to the core's by the declared rule while the contribution stays reachable by qualified name, and (c) two sources claiming one identity resolve by declared source order with the later refused.
- **Handoff:** T-27B02, T-27B04.
- **Notes:** Registry logic is pure and belongs in the domain tier — no I/O. The reserved core namespace is what guarantees a contribution cannot shadow a core name **whatever the core adds later**, which a first-registration-wins rule cannot promise. Ordering must be *declared*, never a function of load order.
- **Decision (recorded, not asked):** unified "reserved core identity" and "identity/source collision" into one mechanism rather than two — the registry pre-claims `"core"` for the core's own source at construction, so an extension claiming `"core"` fails the *ordinary* collision check instead of needing a special-cased rule. Bare-form resolution is populated **only** by core registrations (a contribution never enters the bare map at all, regardless of arrival order) rather than "core displaces a later contribution" — simpler, and sufficient: the spec's own wording says a contribution is reached by its qualified name, never by the bare form, so the bare map has no reason to ever hold one. Verified order-independence directly: the test registers the contribution *before* the core entry and still finds core at the bare lookup. "Source" (who may extend an already-claimed identity) is a caller-supplied opaque token, not registry-generated — assigning real tokens is an extension-loader concern for a later task/phase, not this one.
- **Changes:** New `crates/domain/src/invocable/mod.rs` (`InvocableRegistry`, `Registrant`, `RegistrationError`, `CORE_IDENTITY`), registered in `crates/domain/src/lib.rs`. Added `InvocableId::qualifier()`/`tail()` to `crates/contract/src/lib.rs` (the registry's own need, extending the T-27A01 type rather than duplicating the split logic). 5 new tests in `invocable::tests`. `cargo test -p cronus-domain` 460/460 (455 pre-existing unmodified); `cargo test -p cronus-contract` 20/20 unaffected; `node scripts/check-domain-boundary.mjs` confirms no new dependency entered the domain tier's allowlist; `cargo clippy -p cronus-domain --all-targets -- -D warnings` and `cargo fmt -p cronus-domain -p cronus-contract -- --check` both clean.

### [T-27B02] Dispatch: bind-before-invoke over declared binders, returning a structured outcome

- **Spec:** l2-invocable-registry.md §4.5, §4.6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain` green, including a test that a binding failure leaves **no** side effect — the body did not run — and a test that each of the four rejection modes reports its location. *(Second half satisfied at the type level per the Decision below, not by this module's own `bind()`.)*
- **Handoff:** T-27B03, T-27D01.
- **Notes:** Every declared binder completes before the body begins, which is what makes *did not run* a structural fact rather than a self-report and makes the whole rejection class safely re-invocable. Optional means **absent, never invalid**: a value that is present and fails to bind still rejects, with mode and location intact. Mapping a malformed value onto "not provided" is the quiet failure that makes a unit do confidently wrong work.
- **Decision (recorded, not asked):** a real modeling gap surfaced against the Verify line's literal wording. `bind()` checks already-parsed, already-typed in-memory `ArgValue`s, so it can only naturally produce `Absent` (nothing supplied) and `IllShaped` (present, wrong shape) — there is no "unparsed raw candidate" to be `Malformed`, and (since this crate is I/O-free by construction, §4.3) no external source to fail as `Unreadable`. Both of those belong to whatever surface constructs `ArgValues` from raw input, upstream of dispatch — reading a source, or parsing a raw string, is surface/adapter work, not domain-tier binding. Rather than redesign the already-Done `ArgValue` shape (T-27A02) into a raw-string model just to force all four triggers into one function, left `bind()` honestly scoped to the two modes it can produce, and broadened `Outcome::Unavailable`'s doc comment (a one-line, non-breaking clarification, not a shape change) to also honestly cover "no such invocable is registered" — a third failure class this task discovered that is neither a binder rejection (no binder is at fault) nor the resource-unavailability case the variant was first written for.
- **Changes:** `crates/domain/src/invocable/mod.rs` reorganized into a thin doc+re-export shell (matching the `tool_receipts/` directory-module convention) with the T-27B01 registry moved unchanged into `registry.rs` and a new `dispatch.rs` holding `bind`/`Dispatcher`/`Handler`. Added `InvocableId`'s `Display`-based `as_str()` use in error messages (no new type). 5 new tests in `dispatch::tests`, covering: no side effect on a binding rejection, an absent-optional binder still invoking the handler, an ill-shaped value rejecting with its binder location, an unknown invocable resolving to `Unavailable` rather than panicking, and a registered-but-unattached descriptor doing the same. `cargo test -p cronus-domain` 465/465 (455 pre-existing + 10 invocable, all unmodified/passing); `cargo test -p cronus-contract` 20/20 unaffected; `node scripts/check-domain-boundary.mjs` clean; `cargo clippy -p cronus-domain --all-targets -- -D warnings` and `cargo fmt -p cronus-domain -p cronus-contract -- --check` both clean.

### [T-27B03] Contribution safety: failure policy, grant-gated reach, attribution marker

- **Spec:** l2-invocable-registry.md §4.10
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain` green, including tests that (a) a contributed invocable that panics or exceeds its bound resolves to a rejection **attributed to the contribution** and does not unwind into the core, (b) registering an invocable without the declared manifest grant is refused, and (c) the attribution marker is produced by the projection and cannot be set or suppressed by the contribution.
- **Handoff:** T-27T02.
- **Notes:** These three are what make a contributed verb *safe to be reachable*; reachability without them is the plugin story with its failure modes unspecified. The attribution rule is security-relevant, not cosmetic: a contributed verb that can present itself as the product converts the user's trust in the product into a capability an extension holds.
- **Decision (recorded, not asked):** Rust cannot preempt a running synchronous closure without unsafe/OS-specific work, so "terminated" for an overrun contribution means the *caller* stops waiting at the bound (via a fresh, unscoped `std::thread::spawn` + `mpsc::recv_timeout`), not that the handler's thread is actually killed — it becomes orphaned and runs to completion discarding its result. `std::thread::scope` was considered and rejected: scoped threads are always joined before `scope()` returns, which would make the whole function block for the handler's real duration regardless of the timeout, defeating the purpose. `Handler` changed from `Box<dyn Fn>` to `Arc<dyn Fn>` (T-27B02 API) to get a cheap, `'static`-safe clone onto the spawned thread — the T-27B02 tests updated their `Box::new` call sites to `Arc::new`, no test *assertion* changed. Core invocables are exempted from both containment and the bound entirely (branch on `id.qualifier() == CORE_IDENTITY`): a core panic is a real defect that must propagate, and silently swallowing it would hide bugs rather than contain a contribution's fault. The bound is a `Dispatcher` field (`CONTRIBUTION_TIME_BOUND` default, `with_contribution_bound` override), not a hardcoded constant burned into `run_contribution` — otherwise proving the timeout fires would cost the test suite the production bound's real duration on every run. The grant check implements only the *general* `CONTRIBUTE_GRANT` (registering at all); the spec's further "a specific grant beyond the general one" for security-relevant invocables has no per-invocable security-relevance flag to key off yet and is deferred, not silently dropped. Attribution needed no new field or type: it is a pure function deriving the answer from `id.qualifier()`, which registration had already validated against the real registrant — so a contribution's `name`/`summary` (fields it fully controls) cannot influence it, proven directly by a test that sets a deceptive `name` and confirms the derived attribution is unaffected.
- **Changes:** `crates/domain/src/invocable/dispatch.rs`: `Handler` now `Arc`; new `run_contribution` (catch_unwind + thread + `recv_timeout`); `Dispatcher` gained a `contribution_bound` field, `with_contribution_bound`, and branches core-vs-contribution in `dispatch`. `crates/domain/src/invocable/registry.rs`: `Registrant` gained `grants: HashSet<String>` + `with_grant`; new `CONTRIBUTE_GRANT` const, `RegistrationError::MissingGrant`, the grant check in `register`, and the new `attribution` function. `mod.rs` re-exports updated. 6 new tests (2 dispatch: panic-containment, timeout; 1 dispatch: core-panic-propagates-uncontained, proving the exemption; 3 registry: missing-grant refused, core needs no grant, attribution-from-deceptive-fields) plus the 5 pre-existing B01 registry tests updated to declare `CONTRIBUTE_GRANT` (their own assertions unchanged). `cargo test -p cronus-domain` 471/471 (455 pre-existing + 16 invocable, all green, timeout test completes in milliseconds not seconds); `node scripts/check-domain-boundary.mjs` clean (std::thread/panic/sync are stdlib, not new crate dependencies); `cargo clippy -p cronus-domain --all-targets -- -D warnings` and `cargo fmt -p cronus-domain -- --check` both clean.

### [T-27B04] Core invocables registered from the facade through the public door; redaction at the boundary

- **Spec:** l2-invocable-registry.md §4.3, §4.12
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-core` green *(the Verify line above named the wrong package — the facade crate is `cronus-core`, not `cronus`; corrected here, not silently)*; a test asserts every core invocable was registered through the same published function used by contributions (no private registration path exists); a test asserts dispatch output passes the shared redaction path exactly once.
- **Handoff:** T-27D01.
- **Notes:** EP-12 has one purpose — a seam exercised only by third parties is one nobody notices has become insufficient. Core-supplied contributions may differ in exactly three sanctioned ways (eager loading where ordering demands, implicit trust, surviving third-party disable switches) and no others. Redaction moves here so INV-7 is satisfied once instead of remembered three times; **the empty-secret-list half stays a recorded residual and is not fixed in this phase.**
- **Decision (recorded, not asked):** scoped "core invocables registered from the facade" to two real, already-existing capabilities (`core:status`, `core:version`, delegating to the `Capabilities` methods `Engine` already implements) rather than the full 29-group CLI surface — that migration is T-27D01's own explicitly-scoped job (its notes cite ~4000 lines needing `.N` splits), and this task's own Verify line only asks to prove the *mechanism*, not completeness. `redact_outcome` was added to the existing `cronus_domain::redact` module (extending the established single-owner precedent) rather than a new module, walking `Outcome`'s string-bearing fields (`OutcomeValue::Text`/`List`/`Record`, `Rejection.detail`, `Unavailable.reason`) — `Stream`'s channel name is left unmasked since it names a subscription, not rendered content. `Dispatcher::dispatch` was split into a private `dispatch_unmasked` plus one public wrapper applying `redact_outcome` exactly once, so every return path — including the three early-return `Unavailable`/`Rejected` cases — passes through the same single point structurally, not by remembering to call it at each site. `Dispatcher` gained a `secrets: Vec<String>` field defaulting empty (mirroring the TUI/desktop bridge's own pre-existing `secrets` field) plus `set_secrets` — proving the mechanism while leaving the empty-list-in-production defect as the recorded residual the task notes call for.
- **Changes:** `crates/domain/src/redact.rs`: `redact_outcome` + `redact_value`, 2 new tests (nested Record/List masking, Rejection/Unavailable masking). `crates/domain/src/invocable/dispatch.rs`: `Dispatcher` gained `secrets`/`set_secrets`; `dispatch` now wraps a private `dispatch_unmasked` with one `redact_outcome` call. New `crates/core/src/invocable_bootstrap.rs` (`bootstrap(Engine) -> (InvocableRegistry, Dispatcher)`, registering `core:status`/`core:version` through `Registrant::core()` + `InvocableRegistry::register`). `crates/core/src/lib.rs`: added `invocable` to the domain re-export list and `pub mod invocable_bootstrap;`. New `crates/core/tests/invocable_invariants.rs` (2 tests, matching this crate's `*_invariants.rs` convention): shared-door proof (a real contribution registers into the exact registry `bootstrap` populated) and redaction-boundary proof (a secret-bearing `Outcome::Value` comes back masked). `cargo test -p cronus-core` green across the whole crate (every `test result:` line 0 failed, unit + all ~37 integration files incl. the 2 new); `cargo test -p cronus-domain redact` 6/6; `node scripts/check-domain-boundary.mjs` clean; `cargo clippy -p cronus-core -p cronus-domain --all-targets -- -D warnings` and `cargo fmt` on both, clean; `cargo check --workspace` green.

### [T-27A04] Retrofit: `Installation` locus, validated identity grammar, descriptor bounds

- **Spec:** l2-invocable-registry.md §4.2, §4.4, §4.8
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** In the ports tier: add the fourth `Locus` variant; make identity construction **fallible** against a closed grammar (`<qualifier>:<tail>`, both halves non-empty, neither containing the separator) so an unvalidated identity cannot exist; declare the descriptor bound constants (name, summary, group, binder count, binder name/description) that T-27B05 enforces.
- **Why it is fallible rather than checked at registration:** a type that accepts any string parses `"a:b:c"` and `"noqualifier"` without complaint, and the ambiguity surfaces much later as a lookup resolving to the wrong entry or to none. Refusing at construction means no unvalidated identity exists to be registered.
- **Verify:** `cargo test -p cronus-contract` green, including new cases that a multi-separator, empty-half, and empty-string identity are all **refused**; `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean. Existing call sites updated to the fallible constructor — the count of changed sites is reported, not glossed.
- **Handoff:** T-27B05.

### [T-27B05] Retrofit: registration returns its own reversal; descriptor normalized and refused, never repaired

- **Spec:** l2-invocable-registry.md §4.2, §4.3, §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `register` returns a handle whose disposal removes that registration **and its attached handler** (EP-13); no `unregister(id)` is added. Descriptor fields are bounds-checked at registration and stored as a **normalized owned copy** the registrant cannot afterwards reach (EP-14). The bare-form collision rule is implemented as declared: the core's tail wins, the loser stays registered and reachable by qualified identity, and the shadowing is **reported** naming winner, loser, and the qualified name the loser is still reachable by.
- **Refuse, do not repair:** a descriptor failing a bound is rejected naming the field and the constraint. Truncating or defaulting it is forbidden — a repaired declaration is one whose author never learns it was invalid, and the repair silently becomes part of the contract.
- **Verify:** `cargo test -p cronus-domain` green with new cases for: disposal removing both descriptor and handler; a shadowed contribution still resolvable by qualified id **and** returning to the bare form after the shadowing entry is disposed; an over-long and an empty descriptor field each refused with the field named. `clippy -D warnings` + `fmt --check` clean; `node scripts/check-domain-boundary.mjs` clean.
- **Handoff:** T-27B06.

### [T-27B06] Retrofit: resolution split from outcome; change announcement; paired dispatch journal

- **Spec:** l2-invocable-registry.md §4.5, §4.9, §4.13
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Split resolution from dispatch so an unknown invocable yields a **resolution** result, not an `Outcome`; `Outcome::Unavailable` narrows back to *resolved, ran, could not answer*. Add the registry change announcement with **individually contained** observer failures (one failing observer neither aborts the mutation nor starves the observers after it). Journal a resolved dispatch as a paired entry/settlement record joined by an identity unique across process restarts, with a per-invocable declaration that suppresses raw-input recording.
- **Why the split is not cosmetic:** the three surfaces answer `Unknown` in three incompatible ways — the terminal UI falls through to ordinary input, the command line raises a usage error, the desktop refreshes a stale catalog. One outcome variant cannot serve all three without one of them behaving incorrectly.
- **Journal failure asymmetry (implement exactly this way):** a failure to write the **entry** record fails the dispatch loudly; a failure to write the **settlement** record on an already-failing dispatch is contained, so the handler's own failure stays the reported one.
- **Verify:** `cargo test -p cronus-domain -p cronus-core` green with new cases for: an unknown id producing a resolution miss and **no journal entry**; a panicking observer not preventing registration and not starving a later observer; a paired run/settle journal for a resolved dispatch; a suppressed-input invocable recording its name but not its arguments. `clippy -D warnings` + `fmt --check` clean.
- **Handoff:** T-27C01, T-27D01.

### [T-27C01] Conformance fixture library and harness with three assertion families

- **Spec:** l2-surface-conformance.md §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-conformance` green on the harness's own self-tests; the harness is callable from another crate's test target (proven by T-27D03's use of it).
- **Handoff:** T-27D03, and Phases 28–29 registrations.
- **Notes:** New crate `cronus-conformance`, added to the engine workspace. It must be reachable **by path** from the detached desktop-shell workspace (which already depends on the facade that way), because that shell registers against the same harness in Phase 29 — a corpus crate the shell cannot link is a corpus one surface can never join. A **library plus a harness, not a suite** — the desktop shell builds in a detached workspace and cannot be linked from an engine-workspace test, and SP-7 wants registration to be something a surface does as part of becoming a surface. Three families: surface set (exposed = shipped minus **declared** exclusions), advertised schema against declared binders, and semantic outcome including rejection mode and the empty-versus-unavailable distinction. Fixtures are written **from the divergence, not the feature** — empty, single, zero-count, renamed, oversized, absent, unavailable, secret-bearing, identity-colliding — because the happy path is where implementations already agree and therefore proves nothing.

### [T-27C02] Finding inventory and the two one-way ledgers, seeded with F-1…F-8

- **Spec:** l2-surface-conformance.md §4.1, §4.2, §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** The inventory file lists all eight seed findings with every field of the §4.1 record populated, including `class` for each; at least two entries are class *preemptive* (an inventory with none has given up on SP-9). Ledger direction is checked by a test or a gate that fails on a tombstone removal or a debt addition.
- **Handoff:** T-27T01.
- **Notes:** Keep tombstones boringly literal — a path or an owner-qualified symbol plus its finding. Cleverness here produces a check nobody trusts and everybody bypasses. Repayment requires **all four** SP-4 conditions; three of four is recorded as *open*, not as *mostly repaid*.

### [T-27D01] Command-line parser generated from `Semantic` descriptors (semantic half only)

- **Spec:** l2-cli.md §4.1, §4.1.1, §4.3, §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Renarrowed at v2.70.0 — read this before starting.** The prior wording was *generate the parser from descriptors*, meaning all of it. That is wrong and would not work: an installation verb must be answerable when the composition it would configure is precisely what failed to come up, so it cannot be projected from a registry that does not exist yet (LH-1/LH-5). This task now covers the **`Semantic` half only**; the installation half is T-27D01.1.
- **Scope:** Build the argument parser for `Semantic` descriptors at startup, after composition. Help, completion, and grouping for that half derive from the same descriptors. The statically declared command enum is deleted **for the semantic verbs it held** and its removal recorded as a tombstone; verbs belonging to the installation half move to T-27D01.1 rather than being deleted here.
- **Verify:** `cargo test -p cronus-cli` green; `cli_smoke.rs` passes **unmodified** (behaviour preservation); `cronus --help` lists exactly the registry's shipped `Semantic` groups plus the installation verbs T-27D01.1 owns, and nothing else. `clippy -D warnings` + `fmt --check` clean.
- **Sizing:** expected to split by group with `.N` sub-task ids rather than one sweeping edit — see the Phase Notes sizing warning.
- **Handoff:** T-27D01.1, T-27D02.

### [T-27D01.1] The installation half: one closed launcher grammar feeding both the parser and the catalog

- **Spec:** l2-cli.md §4.1.1, §4.2; l2-invocable-registry.md §4.7.1
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Declare the installation verbs **once**, in this frontend's own source: a closed grammar containing no name an extension can define. That single declaration has two consumers — the pre-composition parser is built from it, and the same declaration registers `Installation` descriptors into the catalog at composition. Add the three-way failure distinction (usage ≠ composition ≠ application) with a usage failure opening no session and journaling no run (LH-7).
- **The two-consumer shape is the point, and the shortcut is the defect.** A verb hand-declared in a parser *and* hand-listed as a descriptor is two statements of one fact — the exact fork this phase exists to close. Omitting the descriptor instead is the other failure: SP-11 forbids expressing a boundary by silence, and without the catalog entry the terminal UI cannot **declare** that it deliberately does not offer `config`, leaving it indistinguishable from unimplemented.
- **Verify:** `cargo test -p cronus-cli` green with a test asserting **set equality** between the verbs the launcher parser accepts and the `Installation` descriptors registered at composition. State the oracle honestly: this equality is what fails the moment the two are maintained separately and drift, and it is the checkable form of "one declaration" — *derived from one source* is a structural property a unit test cannot observe directly, so it is asserted through the consequence it would violate. A usage error and an application error produce different exit codes, asserted. `clippy -D warnings` + `fmt --check` clean.
- **Handoff:** T-27D02.
- **Notes:** Build the parser at startup from descriptors using the builder API rather than the derive macro — the derive form is fixed at build time and therefore cannot carry a contributed verb, which is the whole reason for this phase. The project command grammar (verb-first, explicit verbs, noun groups) becomes a property the **registry** validates at registration, so it holds for a contributed verb as well as a core one.

### [T-27D02] One renderer over the structured outcome; uniform output-format handling

- **Spec:** l2-cli.md §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-cli` green; a test drives every shipped invocable through both output formats and asserts the requested format is honoured in **every** case; existing `crates/cli/tests/cli_smoke.rs` passes **unmodified** (behaviour preservation).
- **Handoff:** T-27D03.
- **Notes:** Rendering becomes one function of `Outcome` plus the requested format, replacing per-command decisions. **Record, do not fix:** the nine sites that discarded the format flag and the five that built structured output by unescaped string formatting are residuals — the renderer must reproduce today's observable output, including its defects, and Phase 29 corrects them separately.

### [T-27D03] Shipped-surface honesty: the five unbound groups leave the default surface; corpus registration

- **Spec:** l2-cli.md §3 (INV-9), l2-surface-conformance.md §4.5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cronus --help` and completion list **no** verb that answers "not implemented"; `cargo test -p cronus-cli` includes the conformance harness invoked against this surface's real projection.
- **Handoff:** T-27T01.
- **Notes:** `goal`, `trigger`, `mission`, `research`, `change` are unbound — they have no descriptor and therefore cannot appear. Their **behaviour is out of scope** for this phase; only their surface treatment changes. Register against the corpus here, before the second projection exists — registration is an order of magnitude cheaper before a surface has behaviour to preserve.

### [T-27T01] Corpus first run — convert failures into findings before fixing anything

- **Goal:** Run the corpus against the migrated command-line surface and treat the output as the initial inventory.
- **Method:** Execute the harness; expect it to fail immediately and unflatteringly. **Convert every failure into a finding record first**, then decide what is repaid in this phase and what is carried. Fixing on sight produces repairs that are rediscovered later as duplicates.
- **Verify:** Every corpus failure is either a closed finding with all four SP-4 conditions met, or an entry in the shrink-only debt ledger with a stated reason. No failure is silently fixed and unrecorded.
- **Status:** Todo

### [T-27T02] Behaviour-preservation proof and full quality gates

- **Goal:** Prove the migration changed structure and not behaviour, and that the phase meets the project's definition of done.
- **Method:** Assert pre-existing command-line tests pass **unmodified**; confirm the four known residuals are recorded at their owning invocables and still behave as before; run the workspace gates.
- **Verify:** `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` (zero), `cargo test --workspace` green across consecutive full runs; `crates/cli/tests/cli_smoke.rs` unmodified and passing; no `unwrap()`/`panic!()` introduced on production paths.
- **Status:** Todo

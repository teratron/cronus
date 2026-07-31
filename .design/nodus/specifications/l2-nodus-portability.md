# Nodus Portability Implementation (Rust)

**Version:** 1.5.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-portability.md

## Overview

Concrete Rust implementation of the nodus portability and extension contract.
Maps each LP-invariant from `l1-nodus-portability.md` to its enforcing mechanism in
`crates/nodus`. Specifies the `SchemaProvider` vocabulary-extension seam, the
`InMemoryStorageProvider` built-in (conformant per LP-2/LP-15, though its executor wiring
stays unconsulted — §3.1) and the `PolicyProvider` per-effect authorization gate, wired
into `Executor::execute_command` for `ModelCall`/`Deferred` effects (§4.9, LP-11), and
documents the extraction artifacts that deliver LP-6 compliance.

§3 maps LP-1…LP-8, the invariants that existed when this spec was written. §3.1
records the realization status of LP-9…LP-20, added to the L1 by later additive
refinement, against a four-way axis that distinguishes a seam with nothing to
attach to from one that ships but is never consulted.

## Related Specifications

- [l1-nodus-portability.md](l1-nodus-portability.md) — portability contract this spec implements
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — runtime module structure this spec extends
- [l2-nodus-observability.md](l2-nodus-observability.md) — AuditProvider extension point (LP-2 example); HO-10 completeness anchors LP-20's reporting half
- [l1-nodus-language.md](l1-nodus-language.md) — NL-invariants that LP-invariants complement
- [l2-nodus-config.md](l2-nodus-config.md) — [ADDED v1.2.0] `ConfigProvider` role; already carries LP-2/LP-15 and LP-10 rows, and is the seam LP-19 should extend rather than parallel
- [l2-nodus-environment.md](l2-nodus-environment.md) — [ADDED v1.2.0] `EnvironmentProvider` role; NE-3/NE-7/NE-10 discharge the realized part of LP-18
- [l2-nodus-dialog.md](l2-nodus-dialog.md) — [ADDED v1.2.0] `DialogProvider` role; its `Status::Paused` + `ResumeDescriptor` lifecycle is the NL-12 substrate LP-14 waits on
- [l2-nodus-compensation.md](l2-nodus-compensation.md) — [ADDED v1.2.0] establishes the "vacuous in core" recording vocabulary §3.1 uses, and records the LP-11 gate as vacuous by composition
- [../../main/specifications/l1-interception-model.md](../../main/specifications/l1-interception-model.md) — [ADDED v1.3.0] the host-side decide-class contract LP-11 realizes; its Document History records the plugin-hook ↔ tool-security fail-behaviour divergence that §4.8.1 cites as LP-3 independence evidence
- [l2-nodus-errors.md](l2-nodus-errors.md) — [ADDED v1.4.0] owns the `NODUS:*` severity/category registry `POLICY_DENIED` (§4.9.4) registers into, beside the frozen 24-code set, matching the `CAPABILITY_UNMET` (LP-8) precedent; also documents that `@err:` dispatch is unrealized (NL-9), the finding §4.9.3 surfaced

## 1. Motivation

`l1-nodus-portability.md` defines the invariants that govern how nodus stays decoupled
from any specific host — seven when this spec was written, twenty today (LP-1…LP-20) —
but the current `l2-nodus-runtime.md` only documents the runtime from an execution
perspective. No spec records:

- exactly which Rust constructs enforce each LP-invariant,
- the planned vocabulary-extension seam (`SchemaProvider`) documented in `vocab.rs` as
  a future refinement,
- the lifecycle of `StorageProvider` and `PolicyProvider` before they satisfy LP-3,
- or the extraction artifacts (CI workflow, `EXTRACTION.md`) as first-class portability
  deliverables.

This spec closes those gaps without duplicating the runtime implementation detail already
in `l2-nodus-runtime.md`.

## 2. Constraints & Assumptions

- All new traits introduced here follow LP-2: one built-in no-op implementation ships with
  the crate; concrete host implementations live outside the library.
- `SchemaProvider` may NOT modify `KNOWN_COMMANDS` or `RESERVED_VARIABLES` constants —
  it builds an extended `Schema` value on top of the builtin baseline.
- `StorageProvider` and `PolicyProvider` are specified here as interface contracts only;
  executor integration (hook points, run-parameter variants) is deferred to a future phase
  once LP-3 is satisfied.
- No new external crate dependencies may be introduced by any trait in this spec
  (LP-1 + LP-5 constraint).
- Elapsed-time measurement, run IDs, and timestamps follow the same conventions established
  in `l2-nodus-observability.md` (caller-supplied `run_id`; `std::time::Instant` for
  elapsed; wall-clock string for manifests).

## 3. Invariant Compliance

Explicit mapping of LP-1…LP-8 to its Rust enforcement mechanism. LP-9…LP-20 were added
to the L1 after this table was written; their realization status is §3.1.

| LP Invariant | Rust Enforcement |
| --- | --- |
| LP-1 Host neutrality | `crates/nodus/Cargo.toml` lists zero `[dependencies]` beyond `std`. The workspace `Cargo.toml` uses `workspace = true` for metadata fields only; no workspace crate is imported. Verified by extraction audit (Phase 3) and by CI's `cargo check --no-default-features`. |
| LP-2 Extension via abstract interfaces | Every integration point is a named Rust `trait`. Built-in implementations (`StubProvider`, `NoopAuditProvider`, `InMemoryStorageProvider`, `NoopPolicyProvider`) satisfy the interface with no I/O. Host implementations live outside the crate boundary. |
| LP-3 Two-host generalisation rule | Enforced by the spec amendment process: a new trait or vocabulary command reaches the library only after a `/magic.spec nodus` amendment documents two independent host-usage contexts, recorded as an **admission record** per L1 §4.14 (§4.8 here). Dispositions are per seam: **`PolicyProvider` is satisfied** (§4.8.1 — the runtime tool guard and the plugin-hook interception, two divergent host decision shapes) and may be wired; **`StorageProvider` is not** (§4.8.2 — one context only), so its executor hook points stay held. |
| LP-4 Vocabulary isolation | `KNOWN_COMMANDS` and `RESERVED_VARIABLES` are compile-time constants in `vocab.rs`. `SchemaProvider` builds an extended `Schema` value at runtime without mutating these constants. Host-specific commands are schema artifacts, never constants. |
| LP-5 Composable extension | All extension points are independently composable: `run_with_provider`, `run_with_audit`, `run_with_provider_and_audit` accept each provider in orthogonal parameters. No global mutable state; no inheritance. Combinators are pure functions in `workflows.rs`. |
| LP-6 Semantic versioning contract | Published via `crates/nodus/Cargo.toml` with `version = "{semver}"`. Breaking changes follow the notice protocol in `l1-nodus-portability.md §4.5`. CI enforces `cargo semver-checks` (planned; currently manual). `EXTRACTION.md` documents the release checklist. |
| LP-7 Feedback loop lifecycle | Operationalised by the `/magic.spec nodus` → `/magic.task nodus` → `/magic.run nodus` pipeline. Discovery and distillation steps happen in `l1-nodus-portability.md §4.2`; spec amendment is the Proposal step; the run pipeline is Implementation; `CHANGELOG.md` + version bump is Release. |
| LP-8 Capability manifest | **Implemented (§4.7).** `portability.rs` defines `CapabilityManifest` (roles / host commands / named capabilities), `HostCapabilities`, and the pure `validate_manifest` resolver. `workflows.rs` runs the gate in `run_with_manifest` / `run_with_manifest_and_audit` after lint validation and before the executor boots, rejecting fail-fast with a `NODUS:CAPABILITY_UNMET` error that names the missing-capability set and emitting no audit events (the executor is never invoked). `CapabilityManifest::from_workflow` derives the manifest from invoked commands. The same resolver is the machine-checkable LP-3 host-substitution test. |

### 3.1 Invariants added after v1.0.0 — realization status [ADDED v1.2.0]

`l1-nodus-portability.md` has grown LP-9 … LP-20 through additive refinement since the
§3 table was written. Every one of those invariants' Document History entries closes with
the same sentence — *"l2-nodus-portability carries LP-{N} as a pending Invariant-Compliance
obligation reconciled at magic.task"* — so they are listed here explicitly: an obligation
tracked only in a plan document is not tracked in the place its own contract points to.

These twelve are host-supplied seams, so the binary *Realized / Pending* axis used by
`l2-nodus-runtime.md` §3.1 would misreport them. A four-way axis is used instead:

- **Satisfied structurally** — enforced by what the crate contains and, more often, by what
  it deliberately lacks. Verifiable today; the standing check is a negative one.
- **Vacuous in core** — the construct the invariant gates does not exist in the crate, so the
  seam has no call site. Nothing is owed until that construct lands, and the obligation
  attaches to whoever builds it. (Vocabulary established by `l2-nodus-compensation.md` §2.)
- **Seam declared, wiring pending** — the trait ships and is re-exported from `lib.rs`, but no
  code path consults it: a host implementing it today observes **no behaviour change**. This
  is published API that silently does nothing, not deferred work. (The crate's own phrasing
  for this state is "interface-declared, wiring pending".)
- **Partially realized** — some of the invariant's rules are discharged, named individually.

| L1 Invariant | Realization status |
| --- | --- |
| LP-9 Extraction attestation | **Vacuous in core.** No bundle-load path exists (zero occurrences of `bundle` / `witness` / `attest`-as-verification), so the verify-before-load hook has no call site. The `EXTRACTION.md` + `ci.yml` artifacts are LP-6 deliverables (§4.6), not an LP-9 witness. Nodus never signs — verification is the host's (LP-2) — so the core's obligation begins only when an import path does. |
| LP-10 Host-granted authority, never self-authored | **Satisfied structurally.** Discharged by absence of vocabulary: `KNOWN_COMMANDS` holds no policy-writing command; `CapabilityManifest` exposes only `require_*` builders (declare) and getters (read) with no grant/relax/widen operation; acceptance authority sits behind `ConfigProvider` (`l2-nodus-config.md` LP-10 row); `Budget` is enforced by the run loop and unreadable by the workflow (`l2-nodus-environment.md` NE-13). Load-bearing in four sibling specs (cited by NL-14 / NL-17 / NL-20 DC-10 / NL-21) while having had no row here — the reason it needs one. |
| LP-11 Per-effect authorization seam | **Implemented [v1.5.0].** `Executor::execute_command` gates every `ModelCall`/`Deferred` effect (`EffectClass{ModelCall,Deferred}`, reusing `MODEL_COMMANDS`/`DIALOG_COMMANDS`; `ToolUse` deliberately unrealized — every tool-shaped builtin command is a fixed stub with no host-swappable seam to gate) through `self.policy.evaluate(gate, context)` before the effect happens — `gate` the effect-class string, `context` a raw pre-resolution `Value::Map` snapshot (§4.9.2). On denial: `ExecutionEvent::StepError` with `NODUS:POLICY_DENIED` (not `ConstraintHit`, which stays scoped to hard `!!`-constraints per §4.4), a `RuntimeError` pushed to `ctx.errors`, and a bare `None` return — non-halting, mirroring `DialogOutcome::Timeout`/`::Rejected`; `RunResult.status` degrades to `Partial`, the pipeline target stays at its seeded default. `Executor` carries a fourth `policy: Box<dyn PolicyProvider>` field (default `NoopPolicyProvider`) across every constructor; `run_with_policy`/`run_with_policy_and_audit` mirror `run_with_dialog`'s exact shape. 444 tests pass (was 435; +9: 4 `effect_class_of`/gate-string unit tests, 5 `run_with_policy` integration tests covering permit/deny on both classes plus a `NoopPolicyProvider` regression). `@err:` dispatch remains unbuilt (NL-9, out of scope per §4.9.6) — `POLICY_DENIED` surfaces via `RunResult.errors` exactly like every other non-fatal typed error. |
| LP-12 Imported-bundle admission vetting | **Vacuous in core.** Same absent load path as LP-9, and explicitly sequenced after it (attest → vet → run, §4.8 of the L1). No classifier call site can exist before a bundle can be loaded. |
| LP-13 Addressable versioned import resolution | **Vacuous in core.** No resolver, catalog, pin record, or reference form. `@runtime: { core: <schema-file> }` names a schema file but performs no resolution or pinning. Precedes LP-9/LP-12 in the load order (resolve → attest → vet → run), so the three are one deliverable, not three. |
| LP-14 Verified-peer delegation seam | **Vacuous in core**, with a named upstream blocker: it specializes NL-12 generalized deferred execution, which `l2-nodus-runtime.md` §3.1 records as **Pending** — the `Status::Paused` + `ResumeDescriptor` lifecycle exists but is human-answer-shaped. A peer seam cannot precede the generalized deferred step it is a special case of. |
| LP-15 Host-supplied durable-state seam | **Seam declared, wiring pending — the built-in divergence is RESOLVED.** `StorageProvider` ships `InMemoryStorageProvider`, which round-trips within one invocation and shares no state across instances, matching L1 §4.1's *in-memory* built-in mandate ("non-durable; state discarded between invocations") and LP-2's "sufficient for in-process testing without I/O" — neither of which the prior `NoopStorageProvider` satisfied (`store` discarded, `load` always returned `None`). The **wiring** half remains open: `store`/`load` still have zero call sites, held by the LP-3 record (§4.8.2 — one host context, not two). One naming divergence from the L1 persists, deliberately unaddressed here: L1 §4.11 names `put`/`get`/`delete`; the crate keeps `store`/`load` and no `delete` — changing the trait signature would be a second breaking change with no conformance gain, since the interface was already LP-1-clean (no host type named). |
| LP-16 Effect risk-class declaration | **Vacuous in core.** No effect-class type exists (zero occurrences of `EffectClass`), and the consequence descriptors are carried *to* the LP-11 gate — which has no call site. Blocked strictly behind LP-11. |
| LP-17 Settlement effect seam | **Vacuous in core.** Zero settlement vocabulary. Two levels of unrealized dependency: it specializes the LP-11 gate (unwired) and reuses LP-14's verified peer (vacuous) for the peer-payee case. |
| LP-18 Environment-liveness seam | **Partially realized.** Rule (c) is discharged — `EnvInteraction` carries environment attribution on existing events (`l2-nodus-environment.md` NE-3), `release` is mandatory + idempotent behind an `InstanceGuard` drop guard (NE-7), and the capability half is complete (NE-10: `ExtensionRole::Environment` + `validate_manifest` fail-fast, provided by `HostCapabilities::builtin()`). Rules (a) and (b) are **not**: `EnvironmentProvider::open` returns `Instance`, not a `Result`, so a vanished or un-openable environment has no typed failure path; no `NODUS:*` code is environment-liveness-shaped; and `ResumeDescriptor` is dialog-shaped, carrying no environment identity, so a resume cannot detect that its environment is gone. Closing (a)/(b) changes a published trait signature — LP-6 **major**, not additive. |
| LP-19 Host-supplied exposure switch seam | **Vacuous in core.** Zero `rollout` / `holdout` vocabulary; the `switch` occurrences are the `?SWITCH` control-flow construct, unrelated. Its nearest neighbour is `§config` (NL-20, `l2-nodus-config.md`), which already shares the resolve-once / host-accepts / workflow-read-only shape but lacks the two rules that make LP-19 what it is: a **declared safe value** and the unresolvable-switch fallback path. When LP-19 lands it should extend the `ConfigProvider` acceptance shape rather than introduce a parallel seam. |
| LP-20 Obligation-gated effect seam | **Vacuous in core.** Zero `obligation` / `discharge` vocabulary. Its gating half reuses the LP-11 ordering and is blocked with it. Its *reporting* half is nearer: "an obligation open at run end is surfaced in the terminal `RunManifest`" has a real anchor, since `RunManifest` exists and `l2-nodus-observability.md` HO-10 already carries completeness honesty (`TraceCompleteness` / `classify_trace`). |

**Leverage.** Of the twelve, none is a plain "needs a phase": eight are vacuous, two are
seam-declared-wiring-pending, one is satisfied structurally, one is partially realized. The
single highest-value observation is that **LP-11's absent call site blocks LP-16, LP-17, and
LP-20** — four invariants reduce to one authorization hook in `execute_command`. The two
nearest-to-plannable items are independent of it: the LP-15 built-in divergence (small and
self-contained) and LP-18 (a)/(b) (which needs an L2 design decision first, because a
`Result`-returning environment lifecycle is a breaking change).

**Import triad.** LP-13 → LP-9 → LP-12 are the ordered stages of one absent mechanism.
Recording them as three independent gaps overstates the backlog; they land together or
not at all.

## 4. Detailed Design

### 4.1 SchemaProvider — Vocabulary Extension Seam

`Schema`'s public query surface (`is_command`, `is_reserved`, `is_valid_tone`) is the
seam for runtime vocabulary extension noted in `vocab.rs`. `SchemaProvider` formalises
the interface:

```text
[REFERENCE]
pub trait SchemaProvider {
    /// Return the host-declared command names that extend the builtin baseline.
    /// Return an empty slice to use the builtin vocabulary unchanged.
    fn host_commands(&self) -> &[&str];

    /// Return additional reserved variable names beyond RESERVED_VARIABLES.
    /// Return an empty slice to use the builtin set unchanged.
    fn host_reserved_variables(&self) -> &[&str];
}

/// Built-in provider: no host extensions; pure builtin schema.
pub struct BuiltinSchemaProvider;

impl SchemaProvider for BuiltinSchemaProvider {
    fn host_commands(&self) -> &[&str] { &[] }
    fn host_reserved_variables(&self) -> &[&str] { &[] }
}
```

`Schema` gains a companion constructor:

```text
[REFERENCE]
impl Schema {
    /// Build an extended schema merging the builtin baseline with host additions.
    /// Host commands that collide with KNOWN_COMMANDS are silently deduplicated.
    pub fn with_provider(provider: &dyn SchemaProvider) -> Self { ... }
}
```

The validator and executor receive the merged `Schema` value; they do not call the
provider directly. This preserves their existing interface and keeps the seam at the
boundary of schema construction.

### 4.2 Extension Point Registry (Rust)

Full registry of LP-2 extension points and their implementation status in `crates/nodus`.
The rows correspond one-to-one with the `ExtensionRole` variants (§4.7); *Wired* records
whether any code path actually consults the trait, which is the distinction §3.1 turns on:

| Role | Trait | Built-in | Status | Wired |
| --- | --- | --- | --- | --- |
| Model | `ModelProvider` | `StubProvider` | Implemented (`executor.rs`) | Yes |
| Audit | `AuditProvider` | `NoopAuditProvider` | Implemented (`observability.rs`) | Yes |
| Vocabulary | `SchemaProvider` | `BuiltinSchemaProvider` | Implemented (`portability.rs`; `Schema::with_provider`, `run_with_schema`) | Yes |
| Dialog | `DialogProvider` | `DefaultDialogProvider` | Implemented (`executor.rs`; `l2-nodus-dialog.md`) | Yes |
| Environment | `EnvironmentProvider` | `StubEnvironment` | Implemented (`environment.rs`; `l2-nodus-environment.md`) | Yes |
| Config | `ConfigProvider` | `DefaultConfigProvider` | Implemented (`portability.rs`; `l2-nodus-config.md`) | Yes |
| Storage | `StorageProvider` | `InMemoryStorageProvider` | Implemented (`portability.rs`; round-trips within an invocation, LP-2/LP-15 conformant — §4.3) | **No** — zero call sites (LP-15, §3.1) |
| Policy | `PolicyProvider` | `NoopPolicyProvider` | Implemented (`executor.rs`; gates `ModelCall`/`Deferred` effects — §4.9) | **Yes** |

> `Dialog`, `Environment`, and `Config` are roles the implementation added after the L1
> §4.1 taxonomy table was last amended; that table still lists four. L1 §4.1 declares itself
> "the authoritative extension point registry", so the divergence is recorded here and
> reconciled L1-side rather than resolved unilaterally by this L2.

### 4.3 StorageProvider — Built-In Conformant, Wiring Pending LP-3

The trait and its built-in satisfy LP-2/LP-15 today; only executor integration remains
gated:

```text
[REFERENCE]
pub trait StorageProvider {
    /// Persist a named value. `key` is host-defined; nodus treats it as opaque.
    fn store(&self, key: &str, value: &crate::executor::Value);

    /// Retrieve a named value. Returns `None` if the key is absent.
    fn load(&self, key: &str) -> Option<crate::executor::Value>;
}

/// In-memory built-in: round-trips within one process; instances share no
/// state. Mutex-guarded; a poisoned lock degrades to empty rather than panics.
pub struct InMemoryStorageProvider { /* Mutex<Vec<(String, Value)>> */ }

impl StorageProvider for InMemoryStorageProvider {
    fn store(&self, key: &str, value: &crate::executor::Value) { /* insert-or-overwrite */ }
    fn load(&self, key: &str) -> Option<crate::executor::Value> { /* clone if present */ }
}
```

`InMemoryStorageProvider` replaced the prior no-op built-in (`NoopStorageProvider`,
`store` discarded, `load` always returned `None`) because L1 §4.1's Storage row specifies
an **in-memory** default and LP-2 requires every built-in be "sufficient for in-process
testing without I/O" — a discarding store cannot round-trip and so satisfied neither. A
discarding **audit** built-in stays fine, since audit is write-only; the store/audit
asymmetry is why this fix does not generalize to `NoopPolicyProvider` or
`NoopAuditProvider`. This is a public-API rename, acceptable at the crate's pre-1.0
version (LP-6 §5's `cargo semver-checks` gate lands at 1.0.0).

Executor integration (hook points for `STORE`/`LOAD`/`RECALL`/`REMEMBER` commands) remains
deferred — LP-3 not satisfied for this seam (§4.8.2); those commands currently operate
against the in-memory variable environment. LP-3 graduation also triggers the addition of
`run_with_storage` and `run_with_storage_and_audit` API variants to `workflows.rs`.

### 4.4 PolicyProvider — Implemented (LP-11) [MODIFIED v1.5.0]

Wired: `Executor::execute_command` gates every `ModelCall`/`Deferred` effect through
`evaluate` before the effect happens. The trait signature is unchanged from what shipped —
only its callers are new (§4.9):

```text
[REFERENCE]
pub trait PolicyProvider {
    /// Evaluate a named policy gate. Returns `true` if the action is permitted.
    ///
    /// `gate` is the effect-class string (`"model_call"` / `"deferred"`,
    /// `EffectClass::as_gate_str`). `context` is a pre-effect `Value::Map`
    /// snapshot of the command and its unresolved argument strings — §4.9.2.
    fn evaluate(&self, gate: &str, context: &crate::executor::Value) -> bool;
}

pub struct NoopPolicyProvider;

impl PolicyProvider for NoopPolicyProvider {
    fn evaluate(&self, _gate: &str, _context: &crate::executor::Value) -> bool { true }
}
```

The `evaluate` contract is deliberately narrow: boolean permit/deny, no mutation. Spend
tracking, approval workflows, and tool-access lists are host-side concerns; the interface
only passes context to the host's decision function.

**[CORRECTED v1.4.0]** This doc comment previously claimed "the executor emits
`ConstraintHit { halt: false }` and skips the step if `false` is returned". That was
wrong on inspection of `observability.rs`: `ConstraintHit`'s own doc comment scopes it to
"a hard (`!!NEVER`/`!!ALWAYS`) constraint" — a language-level absolute rule, not a
host-side policy decision — and `l2-nodus-errors.md` states the `RULE_VIOLATION`/
`ConstraintHit` path is "unchanged" by any later spec. Reusing that event for a policy
denial would silently conflate two different invariants (NL-2 vs LP-11) under one event
type and violate a standing guarantee. §4.9 specifies the real, distinct mechanism.

### 4.5 Module Additions

Delivered (`SchemaProvider` per §4.1, the manifest per §4.7):

```text
[REFERENCE]
crates/nodus/src/
├── ...
├── vocab.rs          — Schema::with_provider constructor
└── portability.rs    — SchemaProvider  + BuiltinSchemaProvider   (wired)
                        ConfigProvider  + DefaultConfigProvider   (wired, l2-nodus-config)
                        StorageProvider + InMemoryStorageProvider (LP-2/LP-15 conformant; executor call sites still pending — §3.1 LP-15)
                        PolicyProvider  + NoopPolicyProvider      (interface only — §3.1 LP-11)
                        ExtensionRole / CapabilityManifest / HostCapabilities / validate_manifest
```

`lib.rs` re-exports from `portability` follow the same pattern as `observability`.

### 4.6 Extraction Artifacts as LP-6 Deliverables

LP-6 compliance requires two publishable artifacts already delivered in Phase 3:

| Artifact | Path | LP-6 role |
| --- | --- | --- |
| CI workflow | `crates/nodus/.github/workflows/ci.yml` | Validates `check + test + clippy + fmt + doc` on every push; acts as the regression gate for semantic versioning |
| Extraction procedure | `crates/nodus/EXTRACTION.md` | Seven-step checklist for human extraction to a standalone repository; includes `cargo semver-checks` step |
| Cargo manifest | `crates/nodus/Cargo.toml` | `version`, `description`, `keywords`, `categories`, `homepage`, `documentation`, `docs.rs` config — all required for crates.io publication |

### 4.7 Capability Manifest & Pre-Run Validation (LP-8)

Realizes `l1-nodus-portability.md` §4.6. A workflow's manifest and the host's
provisions are two values; a pure resolver compares them before the executor
boots, so an unsatisfiable run never starts.

The extension-point taxonomy (§4.2) is a closed enum, and the manifest is three
ordered sets — ordered so diagnostics are deterministic. An empty manifest is
satisfied by any host:

```text
[REFERENCE]
pub enum ExtensionRole {
    Model, Audit, Storage, Policy, Vocabulary,   // original five
    Dialog, Environment, Config,                 // added by later L2 work (§4.2)
}

pub struct CapabilityManifest {
    roles:        BTreeSet<ExtensionRole>,
    commands:     BTreeSet<String>,   // host-schema (non-builtin) commands
    capabilities: BTreeSet<String>,   // named finer-grained features of a role
}
```

A manifest may be authored explicitly (`require_role` / `require_command` /
`require_capability`) or derived from a workflow's AST:

```text
[REFERENCE]
impl CapabilityManifest {
    pub fn from_workflow(ast: &WorkflowFile) -> Self;
}
```

`from_workflow` walks every command (descending into conditionals, loops, and
parallel branches): a model-backed command (`GEN`/`ANALYZE`) requires `Model`; a
dialog command (`ASK`/`CONFIRM`) without a `+default` requires `Dialog`; a
command outside the builtin vocabulary requires `Vocabulary` and is recorded as a
required command name; a builtin non-model command requires nothing. `Storage`,
`Policy`, and `Environment` have no corresponding command syntax and are never
derived — a caller requires them explicitly via `require_role` (`run_with_environment`
does so on every call, since calling it is itself the declaration). Explicit DSL
declaration (an `@needs` section) is deferred to the upstream-parity backlog.

`HostCapabilities` is what the active host provides. It is constructed
explicitly, so the same type serves both the built-in in-process configuration
and host-substitution tests (LP-3):

```text
[REFERENCE]
pub struct HostCapabilities { /* roles, commands, capabilities */ }

impl HostCapabilities {
    pub fn builtin() -> Self;                              // Model + Audit + Vocabulary
    pub fn provides(&self, role: ExtensionRole) -> bool;
    pub fn has_command(&self, command: &str) -> bool;
    pub fn satisfies(&self, capability: &str) -> bool;
}
```

`HostCapabilities::builtin()` provides five roles: `Model` (the `StubProvider`),
`Audit` (a sink is always wired), `Vocabulary` (the builtin schema),
`Environment` (`StubEnvironment`), and `Config` (`DefaultConfigProvider`) — the
last two because each built-in is a *complete trivial* implementation, so a
workflow declaring the need stays runnable in-process. It provides neither
`Storage` nor `Policy` (interface-only, §4.3/§4.4), and deliberately not
`Dialog`, whose default resolver is incomplete: a workflow that asks a question
no host can answer must fail its manifest rather than silently proceed. A
model-only or manifest-free workflow therefore runs against the built-in host
with no wiring.

The resolver is pure — no I/O, no events — and returns every unmet item by name:

```text
[REFERENCE]
pub enum Missing { Role(ExtensionRole), Command(String), Capability(String) }

pub fn validate_manifest(
    manifest: &CapabilityManifest,
    host: &HostCapabilities,
) -> Vec<Missing>;   // empty ⇒ satisfiable
```

The gate lives in `workflows.rs` as two combinators, consistent with the existing
orthogonal `run_with_*` family (LP-5):

```text
[REFERENCE]
pub fn run_with_manifest(
    source, filename, input,
    manifest: &CapabilityManifest, host: &HostCapabilities,
) -> Result<RunResult, Vec<Diagnostic>>;

pub fn run_with_manifest_and_audit(
    source, filename, input,
    manifest, host, audit, run_id, started_at,
) -> Result<RunResult, Vec<Diagnostic>>;
```

Both run the resolver after lint validation and before the executor boots. A
non-empty `missing` set yields a fail-fast `RunResult`: `Status::Failed`, zero
steps logged, and one `NODUS:CAPABILITY_UNMET` error naming each missing
capability. On the audited variant the audit sink receives nothing — no events,
no run-complete callback — because the executor is never invoked (observer
neutrality for runs that never start, HO-3/HO-5). A satisfiable manifest delegates
to the stub executor. This is the machine-checkable LP-3 contract: a workflow is
portable to a host exactly when that host satisfies its manifest.

### 4.8 LP-3 Admission Records [ADDED v1.3.0]

Admission records per `l1-nodus-portability.md` §4.14, which makes the LP-3 gate checkable.
Dispositions are per seam; the two open seams reach **different** outcomes, which is the
rule working rather than a half-finished pass.

#### 4.8.1 `PolicyProvider` (LP-11) — **LP-3 SATISFIED**

| Field | Record |
| --- | --- |
| **Context A** | **Runtime tool guard** — `crates/domain/src/tool_security.rs`. Decides: may this tool call execute? Evaluates *every* tool call before execution against named severity-ranked signature rules ("Layer 2 (guard) evaluates every tool call before execution"). Fails by severity classification. |
| **Context B** | **Plugin-hook interception** — `crates/domain/src/hooks.rs`. Decides: does any registered hook block this tool event? Nine `HookEvent` kinds; hooks run **in parallel** and the aggregated decision is deny if *any* returns `Block`. Fails by aggregation. |
| **Independence** | Documented divergence, and not by our reading: the host's own Stable `l1-interception-model.md` exists to unify "the **currently-divergent fail-behaviours of the plugin-hook and tool-security realizations**" (its Document History, INT-3). Two mechanisms that already disagree on fail direction — severity classification vs parallel deny-aggregation — are the §4.14 strongest signal. Neither derives its behaviour from the other. |
| **Host types the seam must not name** | `HookEvent`, the hook rule/matcher types, `tool_security::Severity`, the scanner's signature-rule types, and `contract::*`. Verified against the interface as it stands: `evaluate(&self, gate: &str, context: &Value) -> bool` names only `&str` and nodus's own `Value`, so LP-3's second half is **already satisfied by construction** — the interface needs no change to pass. |
| **Disposition** | **Satisfied.** The seam spans two independent, divergent host decision shapes without naming a host type. `PolicyProvider` may be wired. |

The host-side need is specified rather than hypothetical: `l1-interception-model.md` §165–167
records that nodus "already carries the *observe* seam … the mechanic it lacks — and adopts
under this pass — is the **decide** seam", and assigns the realization to this workspace.

#### 4.8.2 `StorageProvider` (LP-15) — **LP-3 NOT SATISFIED**

| Field | Record |
| --- | --- |
| **Context A** | **Durable local store** — `crates/store-local`. Decides nothing; persists and retrieves. A genuine backend, but a *provider* rather than a second decision shape. |
| **Context B** | **None identified.** No second integration consumes durable cross-invocation workflow state today. |
| **Independence** | Not assessable — one context. |
| **Host types the seam must not name** | Verified clean regardless: `store(&self, key: &str, value: &Value)` / `load(&self, key: &str) -> Option<Value>` name only `&str` and `Value`. LP-3's second half is satisfied; the first is not. |
| **Disposition** | **Not satisfied.** Missing evidence that would settle it: a second integration that would persist workflow state *with a different durability or locality contract* than `store-local` — e.g. a remote-backed store whose egress decision differs (the `l1-deployment-neutrality` DN-3 local-vs-remote axis LP-15 already cites). Until then `StorageProvider` stays interface-only, and its executor hook points (`STORE`/`LOAD`) remain unplannable. |

Note the asymmetry this produces, and that it is correct: the LP-15 **built-in conformance**
fix (an in-memory built-in per L1 §4.1 and LP-2) is *not* gated by this record, because it
touches neither hook points nor run-parameter variants — the scope §2 defers. Only the wiring
is held.

### 4.9 Per-Effect Authorization Call-Site (LP-11) [ADDED v1.4.0, IMPLEMENTED v1.5.0]

LP-3 is satisfied for `PolicyProvider` (§4.8.1); this section specifies the Rust shape and
**is now implemented** in `crates/nodus` exactly as designed — every reference below is the
real, as-built code, not a projection.

#### 4.9.1 EffectClass — realized narrower than the L1 sketch

L1 §4.7 names three classes: `model_call`, `tool_use`, `deferred`. Grounded against what
the crate actually has:

```text
[REFERENCE]
pub enum EffectClass { ModelCall, Deferred }

/// `None` for a command with no effect class — LOG, COUNTER, DATE, and every other
/// purely in-memory builtin are not gated at all; calling PolicyProvider::evaluate
/// for them would authorize nothing (they touch no host-swappable seam).
pub fn effect_class_of(command: &str) -> Option<EffectClass> {
    if MODEL_COMMANDS.contains(&command) { Some(EffectClass::ModelCall) }
    else if DIALOG_COMMANDS.contains(&command) { Some(EffectClass::Deferred) }
    else { None }
}
```

`ModelCall` = `MODEL_COMMANDS` (`GEN`, `ANALYZE`) and `Deferred` = `DIALOG_COMMANDS`
(`ASK`, `CONFIRM`) — the **same two constants** `CapabilityManifest::from_workflow` already
uses (§4.7), reused rather than re-derived so the two seams stay in lockstep.

**`ToolUse` is deliberately not realized.** `tool_access` even appears as `evaluate`'s own
example gate name (§4.4) — but every `tool`-shaped builtin command (`FETCH`, `WRITE`,
`MKDIR`, `GIT`, `NOTIFY`, `PUBLISH`, …) is, on inspection of `executor.rs`, a fixed,
zero-dependency **stub**: `handle_fetch`, for one, returns `Value::Map([("_stub", true),
("source", <arg>)])` unconditionally — there is no host-swappable seam behind it the way
`ModelProvider` backs `GEN` or `DialogProvider` backs `ASK`. Gating a stub would ask a host
to authorize an effect that cannot vary by host and does not reach outside the crate. This
is the same shape as the §3.1 *vacuous in core* entries (LP-9/12/13/14/16/17/19/20): the
construct the invariant gates does not yet exist, so nothing is owed until a generic
host-tool extension point is introduced (a role this spec does not yet define). Host-schema
(non-builtin) commands are also not gated here — they already pass through LP-8's
pre-run manifest check, and a host implementing `SchemaProvider` controls their dispatch
entirely on its own side.

#### 4.9.2 `gate` and `context` at the call site

`PolicyProvider::evaluate`'s shipped signature (§4.4) is `(gate: &str, context: &Value) ->
bool` — narrower than L1 §4.7's illustrative `authorize(effect_class, args)`. The DSL has
no syntax for a step to declare an arbitrary named gate (that would be the unrealized
`@needs`-adjacent declaration surface the LP-16/17/20 Backlog entries also wait on), so
`gate` is the effect-class name as a literal string — the only vocabulary that exists
today and is host-matchable without a parser change:

```text
[REFERENCE]
gate    := "model_call" | "deferred"        // EffectClass, stringified
context := Value::Map([
    ("command", Value::Text(cmd.name.clone())),
    ("args",    Value::List(cmd.args.iter().map(|a| Value::Text(a.clone())).collect())),
])
```

`context` carries **raw, unresolved** argument strings (`CommandCall.args: Vec<String>`) —
the same pre-resolution form the executor holds before `dispatch` looks up `$var`
references. Passing resolved values would require the gate to duplicate dispatch's own
variable-lookup logic; passing raw args keeps the seam a thin, honest snapshot of what is
about to run, consistent with "decide, pre-effect" (L1 §4.7).

#### 4.9.3 Call-site integration in `execute_command`

The gate brackets the same function that already brackets the `!!`-rule check — mirroring
its structural position, not its consequence:

```text
[REFERENCE]
fn execute_command(&self, ctx, cmd, step_num) -> Option<Signal> {
    if let Some(violation) = ctx.check_rules(&cmd.name, &cmd.args) { /* unchanged, NL-2 */ }

    if let Some(class) = effect_class_of(&cmd.name) {
        let gate = class.as_gate_str();                    // "model_call" | "deferred"
        let context = build_context(cmd);                  // §4.9.2
        if !self.policy.evaluate(gate, &context) {          // decide, pre-effect
            let error_detail = format!("policy denied gate '{gate}' for '{}'", cmd.name);
            self.emit(ctx, |seq, cid| ExecutionEvent::StepError {
                step_index: step_num,
                step_command: cmd.name.clone(),
                error_code: vocab::error_code::POLICY_DENIED.to_string(),
                error_detail: error_detail.clone(),
                step_identity: step_identity(step_num, &cmd.name),
                fault_identity: FaultIdentity {
                    step_identity: step_identity(step_num, &cmd.name),
                    code: vocab::error_code::POLICY_DENIED.to_string(),
                    discriminator: None,
                },
                seq, correlation_id: cid, annotations: EventAnnotations::default(),
            });
            ctx.errors.push(RuntimeError {
                code: vocab::error_code::POLICY_DENIED.to_string(),
                step: step_num,
                reason: error_detail,
            });
            return None;    // the effect never runs; pipeline_target stays at its
        }                   // seeded default (e.g. $out : Value::Null); execution
    }                       // continues to the next step (non-halting)

    /* … existing dispatch, StepStart/StepEnd, pipeline_target binding … */
}
```

Three choices here are load-bearing, each grounded against an existing pattern rather than
invented:

- **`ConstraintHit` is not used.** §4.4 already corrects the doc comment that claimed it;
  the shape above emits `StepError` instead — the same event `RULE_VIOLATION` emits
  alongside its own `ConstraintHit`, so a denial is observable through the ordinary
  per-step channel (HO-2) without touching an event scoped to a different invariant.
- **Return `None`, not `Signal::Skip` or `Signal::Break`.** `Signal::Skip` is the `?IF …
  !SKIP` branch flag — at the top level of `execute_inner`'s step loop it is caught by a
  bare `_ => {}` arm, identical to `None`, so using it here would be a
  same-effect-different-name substitution with no real benefit and a misleading label.
  `Signal::Break` aborts the whole run (`Status::Aborted`), which would make every denial
  fatal — wrong for a per-effect, per-attempt gate whose own L1 invariant (§3, LP-11) draws
  it as the *dynamic* complement to LP-8's *static* strip, not a second hard-constraint
  path. The precedent this shape actually follows is `DialogOutcome::Timeout` /
  `::Rejected` (`executor.rs`): push a typed `RuntimeError`, return `None`, let the run
  continue with the target unset. `RunResult.status` degrades to `Partial` via the existing
  `!ctx.errors.is_empty()` branch — the same path every other non-fatal typed error takes.
- **No `@err:` dispatch.** L1 §4.7 (corrected, v1.14.1) no longer claims `route_to(@err)`;
  grounding here confirmed why — `WorkflowFile.error_decl` has zero call sites anywhere in
  `executor.rs`. It is parsed (`parser.rs`), validated (`validator.rs`), and transpiled
  (`transpiler.rs`), but the declared `@err:` handler is never invoked at runtime. `NL-9`'s
  "the `@err:` handler is invoked for uncaught step errors" is therefore an **unrealized**
  part of `l1-nodus-language.md`, predating LP-11 and out of this pass's scope — building it
  is a separate, crate-wide undertaking (it would touch every error-emitting site, not just
  this one), not something a single seam's call site should silently take on. `POLICY_DENIED`
  surfacing in `RunResult.errors` matches the **actual**, as-built NL-9 pattern every other
  code already follows (`l2-nodus-errors.md` §Overview: "the `@err:` vs. continue decision a
  host applies").

#### 4.9.4 `POLICY_DENIED` error code

New portability-layer code, registered the same way `CAPABILITY_UNMET` (LP-8) was — beside
the frozen 24-code canonical registry, not inside it (`l2-nodus-errors.md` §4.2):

```text
[REFERENCE]
pub const POLICY_DENIED: &str = "NODUS:POLICY_DENIED";
// severity: Error, category: Runtime — same classification as RULE_VIOLATION,
// since a denied effect is a runtime-stage failure, not a validation-stage one.
```

#### 4.9.5 `run_with_policy` / `run_with_policy_and_audit`

Orthogonal combinators (LP-5), matching `run_with_dialog` / `run_with_dialog_and_audit`'s
exact shape — `PolicyProvider` becomes a fourth `Executor` field (alongside `provider`,
`audit`, `dialog`), defaulting to `NoopPolicyProvider`:

```text
[REFERENCE]
impl Executor {
    pub fn with_policy(policy: impl PolicyProvider + 'static) -> Self;
    pub fn with_policy_and_audit(
        policy: impl PolicyProvider + 'static,
        audit: impl AuditProvider + 'static,
    ) -> Self;
}

pub fn run_with_policy(
    source: &str, filename: &str, input: Option<Value>,
    policy: impl PolicyProvider + 'static,
) -> Result<RunResult, Vec<Diagnostic>>;

pub fn run_with_policy_and_audit(
    source: &str, filename: &str, input: Option<Value>,
    policy: impl PolicyProvider + 'static, audit: impl AuditProvider + 'static,
    run_id: &str, started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>>;
```

Additive: a caller using any existing `run_with_*` variant continues to get
`NoopPolicyProvider`'s allow-all, byte-for-byte today's behaviour (L1 §4.7's own purity
guarantee).

#### 4.9.6 What this section does not settle

`ToolUse` realization (§4.9.1) waits on a host-tool extension seam this spec does not
define. `gate` values beyond the two effect-class strings — a workflow declaring its own
named gate — wait on DSL surface this spec does not add (the LP-16/17/20 kinship). Building
`@err:` dispatch itself (§4.9.3) is `l1-nodus-language.md` NL-9's obligation, not LP-11's;
flagging it here is this pass's contribution, not its remedy.

## 5. Implementation Notes

Order of implementation across future phases, revised against the §3.1 findings:

1. **SchemaProvider + capability manifest (LP-8)** — delivered. `Schema::with_provider`, the `run_with_schema` variant, and the `CapabilityManifest` / `validate_manifest` gate (§4.7) are purely additive — zero risk to the existing API.
2. ~~**PolicyProvider call site (LP-11)**~~ — **DONE [v1.5.0].** `Executor::execute_command` gates every `ModelCall`/`Deferred` effect through `self.policy.evaluate`; `NODUS:POLICY_DENIED` (not `ConstraintHit`) is non-halting and degrades `RunResult.status` to `Partial`; `run_with_policy`/`run_with_policy_and_audit` ship. This item passed through all four states the LP-11 arc named: v1.2.0 called it task-authorable while omitting the LP-3 gate (error) → v1.3.0 opened the gate but left the shape undesigned (permitted-not-designed) → v1.4.0 designed it (permitted-and-designed) → v1.5.0 built it. LP-16/LP-17/LP-20 remain blocked behind the same `decide → effect → observe` ordering until their own designs land — unblocking *this* seam does not, by itself, realize theirs.
3. **StorageProvider built-in + call site (LP-15)** — two separable pieces, correctly separated in practice. **[MODIFIED v1.3.1] The built-in half is DONE**: `InMemoryStorageProvider` replaced the no-op, round-trips within an invocation, and was self-contained exactly as predicted — it did not wait on LP-3. Executor hook points for `STORE`/`LOAD` remain the larger piece, still held: `StorageProvider`'s own LP-3 record (§4.8.2) is **not satisfied** — one host context, not two — unlike `PolicyProvider`'s.
4. **Environment liveness (LP-18 a/b)** — requires an L2 design decision before it can be planned: giving `EnvironmentProvider::open` a `Result` return and teaching `ResumeDescriptor` an environment identity are breaking changes to published items, so LP-6 classes them **major**. Amend this spec (or `l2-nodus-environment.md`) before authoring a task.
5. **Import triad (LP-13 → LP-9 → LP-12)** — one deliverable, not three; nothing is owed until a bundle-load path exists. Not plannable today.
6. **`cargo semver-checks` gate** — add to `ci.yml` once the crate reaches `1.0.0`; this is the LP-6 mechanical enforcement.

**[MODIFIED v1.5.0]** Item 2 is **done**; item 3's built-in half has been done since v1.3.1
(its wiring half stays held by its own unsatisfied LP-3 record, §4.8.2). Item 4 still needs
an L2 decision. Everything in §3.1 marked *Vacuous in core* is deliberately absent from this
list: an item with no call site yields no verifiable task.

## 6. Drawbacks & Alternatives

**SchemaProvider as a generic parameter (not trait object)**: a `Schema::with_provider<P: SchemaProvider>(p: &P)` signature avoids one virtual dispatch. Rejected in favour of `&dyn SchemaProvider` to keep `run_with_schema` callable with any boxed host implementation without requiring the caller to specify a type parameter.

**Merging StorageProvider into ModelProvider**: pairing state persistence with model invocation. Rejected: violates LP-5 (composability); storage and model are independent concerns; combining them prevents hosts that need only one from avoiding the other.

**Runtime feature flags for pending extension points**: expose `StorageProvider`/`PolicyProvider` behind `#[cfg(feature = ...)]`. Rejected: violates LP-5 (no feature-flag extension mechanism); the no-op built-in achieves zero-cost absence without compile-time gating.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[VOCAB]` | `crates/nodus/src/vocab.rs` | `KNOWN_COMMANDS`, `RESERVED_VARIABLES`, `Schema` — LP-4 seam |
| `[EXT-POINTS]` | `crates/nodus/src/executor.rs` | `ModelProvider`, `StubProvider` — LP-2 canonical example |
| `[AUDIT-EXT]` | `crates/nodus/src/observability.rs` | `AuditProvider`, `NoopAuditProvider` — LP-2 second example |
| `[PORTABILITY]` | `crates/nodus/src/portability.rs` | `ExtensionRole`, `CapabilityManifest`, `HostCapabilities`, `validate_manifest`; the Schema/Storage/Policy/Config traits — the §3.1 evidence base |
| `[ENV-EXT]` | `crates/nodus/src/environment.rs` | `EnvironmentProvider`, `StubEnvironment`, `Instance` — the LP-18 realized surface |
| `[RUN-VARIANTS]` | `crates/nodus/src/workflows.rs` | The orthogonal `run_with_*` family — LP-5 composability evidence |
| `[GUARD-CTX-A]` | `crates/domain/src/tool_security.rs` | LP-3 host context A (§4.8.1) — runtime tool guard, pre-execution severity-rule evaluation |
| `[GUARD-CTX-B]` | `crates/domain/src/hooks.rs` | LP-3 host context B (§4.8.1) — plugin-hook interception, parallel deny-aggregation |
| `[HOST-EMBED]` | `crates/core/src/model_bridge.rs` | The canonical LP-2 host adaptor: `contract::InferenceBackend` → `nodus::ModelProvider`, kept outside the crate so LP-1 holds |
| `[API-SURFACE]` | `crates/nodus/src/lib.rs` | Public re-export surface — LP-6 versioning scope |
| `[CI]` | `crates/nodus/.github/workflows/ci.yml` | LP-6 regression gate |
| `[EXTRACTION]` | `crates/nodus/EXTRACTION.md` | LP-6 extraction checklist |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.5.0 | 2026-07-31 | Core Team | **Implemented the LP-11 call site designed in v1.4.0 — Phase 24.** `Executor::execute_command` now gates every `ModelCall`/`Deferred` effect through `self.policy.evaluate(gate, context)` before it runs, exactly per §4.9's design; `Executor` gained a fourth `policy: Box<dyn PolicyProvider>` field (default `NoopPolicyProvider`) across all six constructors, plus `with_policy`/`with_policy_and_audit` and the `run_with_policy`/`run_with_policy_and_audit` combinators mirroring `run_with_dialog`'s exact shape. `NODUS:POLICY_DENIED` registered in `vocab.rs` beside the frozen 24-code registry (`Error`, `Runtime` — same as `RULE_VIOLATION`), surfaced via the existing `StepError` event and `ctx.errors`, never `ConstraintHit`. 444 tests pass (was 435; +9): `effect_class_of` classification + gate-string unit tests in `portability.rs`, and five integration tests in `tests/portability.rs` — permit/deny on both `ModelCall` and `Deferred`, plus a `NoopPolicyProvider` regression proving every existing `run_with_*` variant stays byte-for-byte unchanged. **Caught two real assertion errors empirically, not assumed correct**: the test's first draft asserted a denied step's pipeline target is *absent* from `RunResult.vars`; it is not — `$out` (and `$error`/`$meta`/etc.) are seeded to defaults (`Value::Null` for `out`) at context construction regardless of whether any step runs, so "unbound" means "still `Value::Null`", not "key absent" — fixed by asserting against the seeded default instead, which also strengthened the *permitted*-effect assertions (previously vacuously true, since the key is always present). **Self-review before closing the task caught two stale doc comments in the shipped crate itself**, not just the spec: `portability.rs`'s module doc and `PolicyProvider`'s own trait doc still said "executor integration is deferred until LP-3 is satisfied" — both corrected to describe the real, wired gate. §4.9's own three `[REFERENCE]` pseudocode blocks were reconciled to the exact as-built code during this pass: `class.as_str()` corrected to the real method name `as_gate_str()`, and the denial `reason` string corrected to the fuller `"policy denied gate '{gate}' for '{cmd}'"` form the real code emits (the pseudocode had shown a shorter placeholder). §4.2's registry row, §4.4's heading/framing, §3.1's LP-11 row, §5 item 2, and the Overview all updated to Implemented; `Related Specifications`/`Canonical References` needed no new entries (existing `[EXT-POINTS]`/`[PORTABILITY]` aliases already cover the touched files generically). `cargo test -p nodus`: 444 passed, 0 failed; `cargo clippy -p nodus --all-targets -- -D warnings`: clean; `cargo fmt -p nodus -- --check`: one violation (line-wrap in the new integration tests), fixed by `cargo fmt`; `Cargo.toml`/`Cargo.lock` diff empty (LP-1 preserved); every `unwrap`/`expect`/`panic!` hit in the touched files falls inside `#[cfg(test)] mod tests`, none on a production path. |
| 1.4.0 | 2026-07-31 | Core Team | **Designed the LP-11 call site — new §4.9, first non-vacuous design closing the highest-leverage item in §5.** `EffectClass{ModelCall,Deferred}` realized narrower than L1 §4.7's three-way sketch: `ModelCall`/`Deferred` reuse the existing `MODEL_COMMANDS`/`DIALOG_COMMANDS` constants (already load-bearing in `CapabilityManifest::from_workflow`, §4.7); `ToolUse` is deliberately left **vacuous** — every `tool`-shaped builtin command (`FETCH`, `WRITE`, `GIT`, `NOTIFY`, …) is, on inspection, a fixed zero-dependency stub with no host-swappable seam behind it, so gating one would authorize nothing real, the same shape as the §3.1 *vacuous in core* entries. `gate`/`context`: since the DSL has no syntax for a step to declare a named gate, `gate` is the effect-class string itself and `context` a raw, pre-resolution `Value::Map` snapshot of the command and its unresolved args. Call site: `execute_command`, mirroring the `!!`-rule check's structural position but not its consequence. **Corrected a real defect in §4.4's own doc comment**, found on inspection of `observability.rs` rather than assumed: it claimed the denial reuses `ConstraintHit { halt: false }`, but that variant's own doc comment scopes it to hard `!!NEVER`/`!!ALWAYS` constraints and `l2-nodus-errors.md` states the `RULE_VIOLATION`/`ConstraintHit` path is "unchanged" by any later spec — reusing it would have silently conflated NL-2 and LP-11 under one event type. The real mechanism: a new `NODUS:POLICY_DENIED` code (registered beside the frozen 24-code registry exactly as `CAPABILITY_UNMET` was for LP-8), emitted via the existing `StepError` event (no new `ExecutionEvent` variant — purely additive), `ctx.errors.push`, and a bare `None` return — mirroring `DialogOutcome::Timeout`/`::Rejected`'s existing non-halting precedent, not `Signal::Skip` (a different, branch-scoped primitive that top-level code treats identically to `None` anyway) or `Signal::Break` (which would make every denial fatal, wrong for a per-attempt gate). **A genuine, previously-unflagged finding surfaced while grounding the denial signal**: `WorkflowFile.error_decl` — the parsed `@err:` handler — has **zero call sites in `executor.rs`**. It is parsed, validated, and transpiled but never dispatched, meaning `@err:` routing is not a mechanism the runtime has today; L1 §4.7's `route_to(@err)` pseudocode described something unbuilt. Corrected there (v1.14.1) rather than silently built here — a full `@err:` dispatch mechanism is `l1-nodus-language.md` NL-9's own obligation, touching every error-emitting site, not a thing this one seam's call site should absorb; flagged as a new, real gap rather than a rediscovery of a known one (checked the Backlog first — it wasn't there). `run_with_policy`/`run_with_policy_and_audit` combinators specified mirroring `run_with_dialog`'s exact shape (`PolicyProvider` becomes a fourth `Executor` field). §3.1's LP-11 row, §4.4 (doc-comment correction), and §5 item 2 (closed out through its third and final state: error → permitted-not-designed → permitted-and-designed) all updated; `l2-nodus-errors.md` gains a one-line cross-reference for the new code, matching its own `CAPABILITY_UNMET` precedent. Design only — no line has landed in `crates/nodus`; `magic.task`/`magic.run` build it next. |
| 1.3.1 | 2026-07-31 | Core Team | Reconciled to the as-built LP-15 fix: `NoopStorageProvider` replaced by `InMemoryStorageProvider` (`Mutex`-guarded, round-trips within an invocation, shares no state across instances), satisfying L1 §4.1's in-memory built-in mandate and LP-2's in-process-sufficiency requirement that the prior no-op met for neither. Updated §4.2's registry row and Overview to Implemented; §4.3 heading and body to record the built-in as conformant while its executor wiring stays held by the unsatisfied LP-15 admission record (§4.8.2); §3.1's LP-15 row split into a resolved built-in half and a still-open wiring half; §5 item 3 marked complete for the built-in and its closing sentence updated to two states (was three, since item 3 no longer shares item 2's "not yet designed" status). Public-API rename (`NoopStorageProvider` → `InMemoryStorageProvider`), acceptable pre-1.0 per LP-6 §5. No new dependency (LP-1); `cargo test -p nodus` 435 passing (was 429), `clippy`/`fmt` clean. |
| 1.3.0 | 2026-07-31 | Core Team | Added §4.8 — the first **LP-3 admission records** under the L1 §4.14 rule authored in the same pass, reaching **different dispositions for the two open seams**, which is the gate doing work rather than a half-finished pass. **`PolicyProvider` (LP-11) — SATISFIED.** Context A: the runtime tool guard (`crates/domain/src/tool_security.rs`), deciding whether a tool call may execute, evaluating every call pre-execution against severity-ranked signature rules. Context B: plugin-hook interception (`crates/domain/src/hooks.rs`), nine `HookEvent` kinds evaluated **in parallel** with deny-if-any-`Block` aggregation. Independence is established by **documented divergence and not by our own reading**: the host's Stable `l1-interception-model.md` exists precisely to unify "the currently-divergent fail-behaviours of the plugin-hook and tool-security realizations" (INT-3), and two mechanisms that already disagree on fail direction — severity classification vs parallel deny-aggregation — are §4.14's strongest signal. LP-3's second half was already satisfied by construction: `evaluate(&self, gate: &str, context: &Value) -> bool` names only `&str` and nodus's own `Value`, so the interface needs no change; the host types it must never name (`HookEvent`, hook matcher/rule types, `tool_security::Severity`, scanner signature-rule types, `contract::*`) are all absent. The host need is specified rather than hypothetical — `l1-interception-model` §165–167 records that nodus "already carries the *observe* seam … the mechanic it lacks … is the **decide** seam" and assigns the realization here. **`StorageProvider` (LP-15) — NOT SATISFIED**: only one context (`crates/store-local`, a provider rather than a second *decision shape*); the missing evidence is named precisely — a second integration persisting workflow state under a different durability/locality contract, the `l1-deployment-neutrality` DN-3 local-vs-remote axis LP-15 already cites — so its `STORE`/`LOAD` hook points stay unplannable while the separate LP-15 **built-in conformance** fix remains ungated, since it touches neither hook points nor run-parameter variants. Updated accordingly: §3's LP-3 row (per-seam dispositions replacing the blanket "both held pending"), §3.1's LP-11 row (**unblocked**, with the four items still needing design named — effect-class notion, `gate` derivation, denial signal, `run_with_policy` combinators), and §5 item 2, whose v1.2.0 text called the call site task-authorable while omitting the gate this same document asserts in §3/§2/§4.4 — corrected from the other side, since the gate is now genuinely open but the seam is *permitted, not yet designed*: the difference between "not allowed" and "not specified". §5's closing sentence rewritten to separate the three states it had collapsed. Related Specifications gains the host-side `l1-interception-model`; Canonical References gain the two divergence-evidence files and the LP-2 host adaptor. |
| 1.2.0 | 2026-07-30 | Core Team | Reconciled the twelve deferred Invariant-Compliance obligations LP-9…LP-20, each of whose L1 Document History entries names *this* spec as their carrier — new §3.1. Replaces the binary Realized/Pending axis of `l2-nodus-runtime` §3.1 with a **four-way** one, because these are host-supplied seams and the binary form misreports them: **Satisfied structurally** (LP-10 — discharged by absence of vocabulary, and load-bearing in four sibling specs while having had no row here), **Vacuous in core** (LP-9/12/13/14/16/17/19/20 — the gated construct is absent, so the seam has no call site and nothing is owed until it lands; vocabulary from `l2-nodus-compensation` §2), **Seam declared, wiring pending** (LP-11 and LP-15 — traits ship and are re-exported but `evaluate` / `store` / `load` have **zero call sites**, so a host implementing them observes no behaviour change: published API that silently does nothing, not deferred work), and **Partially realized** (LP-18 — rule (c) discharged via NE-3/NE-7/NE-10, rules (a)/(b) not, and closing them is an LP-6 *major* change since `EnvironmentProvider::open` returns `Instance` not `Result`). Records two structural findings: **LP-11's absent call site alone blocks LP-16, LP-17, and LP-20**, so four invariants reduce to one hook in `execute_command`; and LP-13 → LP-9 → LP-12 are ordered stages of one absent import mechanism, not three independent gaps. Names an L1 divergence: L1 §4.1 specifies an *in-memory* built-in store while the crate ships `NoopStorageProvider`, which never round-trips within an invocation (also `put`/`get`/`delete` vs `store`/`load`, no `delete`). Corrected three stale surfaces against `portability.rs`: §4.2 registry (5 roles → 8, adding `Dialog`/`Environment`/`Config`, plus a *Wired* column, and un-marking `SchemaProvider` as pending when §5 already recorded it delivered), §4.7 `ExtensionRole` reference enum (5 → 8 variants) and its built-in-host prose (3 → 5 roles provided, with `Dialog`'s deliberate exclusion explained), and `from_workflow`'s derivation rules (adds the `ASK`/`CONFIRM` → `Dialog` rule and the never-derived roles). §1 seven-invariants → twenty; §4.5 module map marked delivered; §5 re-ordered so the only two task-authorable items (LP-11 call site, LP-15 built-in) lead. Flags for L1 reconciliation that §4.1 — self-declared "the authoritative extension point registry" — still lists four roles against the implementation's eight. |
| 1.1.0 | 2026-06-27 | Core Team | LP-8 implemented: §4.7 capability manifest + pre-run satisfiability gate (`CapabilityManifest`, `ExtensionRole`, `HostCapabilities`, `validate_manifest`, `run_with_manifest` / `run_with_manifest_and_audit`, `NODUS:CAPABILITY_UNMET`); §3 LP-8 row → Implemented; status RFC → Stable |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — LP-1…LP-7 compliance table; SchemaProvider seam; StorageProvider + PolicyProvider pending LP-3 interfaces; extraction artifacts |

# Nodus Portability Implementation (Rust)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-nodus-portability.md

## Overview

Concrete Rust implementation of the nodus portability and extension contract.
Maps each LP-invariant from `l1-nodus-portability.md` to its enforcing mechanism in
`crates/nodus`. Specifies the `SchemaProvider` vocabulary-extension seam, defines
the pending `StorageProvider` and `PolicyProvider` trait interfaces (pending LP-3
graduation), and documents the extraction artifacts that deliver LP-6 compliance.

## Related Specifications

- [l1-nodus-portability.md](l1-nodus-portability.md) — portability contract this spec implements
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — runtime module structure this spec extends
- [l2-nodus-observability.md](l2-nodus-observability.md) — AuditProvider extension point (LP-2 example)
- [l1-nodus-language.md](l1-nodus-language.md) — NL-invariants that LP-invariants complement

## 1. Motivation

`l1-nodus-portability.md` defines seven invariants (LP-1…LP-7) that govern how nodus
stays decoupled from any specific host, but the current `l2-nodus-runtime.md` only
documents the runtime from an execution perspective. No spec records:

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

Explicit mapping of every LP-invariant to its Rust enforcement mechanism:

| LP Invariant | Rust Enforcement |
| --- | --- |
| LP-1 Host neutrality | `crates/nodus/Cargo.toml` lists zero `[dependencies]` beyond `std`. The workspace `Cargo.toml` uses `workspace = true` for metadata fields only; no workspace crate is imported. Verified by extraction audit (Phase 3) and by CI's `cargo check --no-default-features`. |
| LP-2 Extension via abstract interfaces | Every integration point is a named Rust `trait`. Built-in implementations (`StubProvider`, `NoopAuditProvider`, `NoopStorageProvider`, `NoopPolicyProvider`) satisfy the interface with no I/O. Host implementations live outside the crate boundary. |
| LP-3 Two-host generalisation rule | Enforced by the spec amendment process: a new trait or vocabulary command reaches the library only after a `/magic.spec nodus` amendment documents two independent host-usage contexts. `StorageProvider` and `PolicyProvider` are held here pending that gate. |
| LP-4 Vocabulary isolation | `KNOWN_COMMANDS` and `RESERVED_VARIABLES` are compile-time constants in `vocab.rs`. `SchemaProvider` builds an extended `Schema` value at runtime without mutating these constants. Host-specific commands are schema artifacts, never constants. |
| LP-5 Composable extension | All extension points are independently composable: `run_with_provider`, `run_with_audit`, `run_with_provider_and_audit` accept each provider in orthogonal parameters. No global mutable state; no inheritance. Combinators are pure functions in `workflows.rs`. |
| LP-6 Semantic versioning contract | Published via `crates/nodus/Cargo.toml` with `version = "{semver}"`. Breaking changes follow the notice protocol in `l1-nodus-portability.md §4.5`. CI enforces `cargo semver-checks` (planned; currently manual). `EXTRACTION.md` documents the release checklist. |
| LP-7 Feedback loop lifecycle | Operationalised by the `/magic.spec nodus` → `/magic.task nodus` → `/magic.run nodus` pipeline. Discovery and distillation steps happen in `l1-nodus-portability.md §4.2`; spec amendment is the Proposal step; the run pipeline is Implementation; `CHANGELOG.md` + version bump is Release. |

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

Full registry of LP-2 extension points and their implementation status in `crates/nodus`:

| Role | Trait | Built-in | Status |
| --- | --- | --- | --- |
| Model | `ModelProvider` | `StubProvider` | Implemented (`executor.rs`) |
| Audit | `AuditProvider` | `NoopAuditProvider` | Implemented (`observability.rs`) |
| Vocabulary | `SchemaProvider` | `BuiltinSchemaProvider` | Specified here; pending implementation |
| Storage | `StorageProvider` | `NoopStorageProvider` | Pending LP-3 (interface below) |
| Policy | `PolicyProvider` | `NoopPolicyProvider` | Pending LP-3 (interface below) |

### 4.3 StorageProvider — Pending LP-3

Interface contract recorded for when LP-3 is satisfied (two independent hosts require
durable cross-invocation state):

```text
[REFERENCE]
pub trait StorageProvider {
    /// Persist a named value. `key` is host-defined; nodus treats it as opaque.
    fn store(&self, key: &str, value: &crate::executor::Value);

    /// Retrieve a named value. Returns `None` if the key is absent.
    fn load(&self, key: &str) -> Option<crate::executor::Value>;
}

pub struct NoopStorageProvider;

impl StorageProvider for NoopStorageProvider {
    fn store(&self, _key: &str, _value: &crate::executor::Value) {}
    fn load(&self, _key: &str) -> Option<crate::executor::Value> { None }
}
```

Executor integration (hook points for `STORE`/`LOAD`/`RECALL`/`REMEMBER` commands) is
deferred; those commands currently operate against the in-memory variable environment.
LP-3 graduation also triggers the addition of `run_with_storage` and
`run_with_storage_and_audit` API variants to `workflows.rs`.

### 4.4 PolicyProvider — Pending LP-3

Interface contract recorded for when LP-3 is satisfied (two independent hosts require
runtime policy evaluation beyond `!!`-rules):

```text
[REFERENCE]
pub trait PolicyProvider {
    /// Evaluate a named policy gate. Returns `true` if the action is permitted.
    ///
    /// `gate` is the host-defined policy identifier (e.g., "spend_cap", "tool_access").
    /// `context` is the current variable environment snapshot as a `Value::Map`.
    /// The executor emits `ConstraintHit { halt: false }` and skips the step if `false`
    /// is returned; `halt: true` is reserved for hard `!!`-rule violations.
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

### 4.5 Module Additions

When `SchemaProvider` is implemented (§4.1):

```text
[REFERENCE]
crates/nodus/src/
├── ...
├── vocab.rs          — Schema::with_provider constructor added
└── portability.rs    — SchemaProvider + BuiltinSchemaProvider
                        StorageProvider + NoopStorageProvider  (pending LP-3)
                        PolicyProvider + NoopPolicyProvider    (pending LP-3)
```

`lib.rs` re-exports from `portability` follow the same pattern as `observability`.

### 4.6 Extraction Artifacts as LP-6 Deliverables

LP-6 compliance requires two publishable artifacts already delivered in Phase 3:

| Artifact | Path | LP-6 role |
| --- | --- | --- |
| CI workflow | `crates/nodus/.github/workflows/ci.yml` | Validates `check + test + clippy + fmt + doc` on every push; acts as the regression gate for semantic versioning |
| Extraction procedure | `crates/nodus/EXTRACTION.md` | Seven-step checklist for human extraction to a standalone repository; includes `cargo semver-checks` step |
| Cargo manifest | `crates/nodus/Cargo.toml` | `version`, `description`, `keywords`, `categories`, `homepage`, `documentation`, `docs.rs` config — all required for crates.io publication |

## 5. Implementation Notes

Order of implementation across future phases:

1. **SchemaProvider** (next phase) — purely additive; `Schema::with_provider` + `portability.rs` module + `run_with_schema` variant. Zero risk to existing API.
2. **StorageProvider** (after LP-3) — requires executor hook points for `STORE`/`LOAD` commands. Plan as a separate phase once the second host usage context is documented.
3. **PolicyProvider** (after LP-3) — requires a new pre-step evaluation hook in `execute_command`. Plan alongside or after StorageProvider.
4. **`cargo semver-checks` gate** — add to `ci.yml` once the crate reaches `1.0.0`; this is the LP-6 mechanical enforcement.

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
| `[API-SURFACE]` | `crates/nodus/src/lib.rs` | Public re-export surface — LP-6 versioning scope |
| `[CI]` | `crates/nodus/.github/workflows/ci.yml` | LP-6 regression gate |
| `[EXTRACTION]` | `crates/nodus/EXTRACTION.md` | LP-6 extraction checklist |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — LP-1…LP-7 compliance table; SchemaProvider seam; StorageProvider + PolicyProvider pending LP-3 interfaces; extraction artifacts |

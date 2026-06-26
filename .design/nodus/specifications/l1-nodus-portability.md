# Nodus Portability and Extension Contract

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

Nodus is designed as a portable, host-agnostic workflow library. It is initially developed inside a host project but must remain useful across diverse, independent host contexts. This spec defines the portability contract: how host-project feedback flows back into nodus as universal improvements, what constitutes a valid extension point, and the invariants that keep nodus decoupled from any single host.

The guiding principle is *feedback distillation*: a host project's concrete need becomes a nodus feature only when the abstraction generalises beyond that host.

## Related Specifications

- [l1-nodus-language.md](l1-nodus-language.md) — language design; portability contract governs its extension
- [l2-nodus-runtime.md](l2-nodus-runtime.md) — Rust implementation; must satisfy LP-2, LP-6

## 1. Motivation

A workflow library that begins inside one project risks accumulating host-specific assumptions — custom command vocabularies, internal type references, or hardcoded integration paths — that block adoption by a second host. These assumptions are invisible at first (the original host "just works") but surface as breaking friction when extraction is attempted.

The portability contract prevents this by:

- Defining the feedback loop so host-project observations become universally scoped improvements
- Enumerating extension points that any host can implement without modifying the library
- Setting the stability contract (versioning + backward compatibility) that extraction-ready libraries must honour
- Establishing a composability principle so features snap together rather than accumulate as global options

## 2. Constraints & Assumptions

- nodus core must compile and pass all tests with zero host-project dependencies: no conditional imports, no `#[cfg(feature = "…")]` gates that pull in host types
- Extension points are the only coupling surface; they are always expressed as abstract interfaces (traits, protocols, or equivalent), never as concrete host types
- The built-in schema vocabulary is a default baseline; host-specific command extensions are schema artifacts loaded at runtime, not constants in the library
- Language invariants (NL-1…NL-10 from `l1-nodus-language.md`) are upstream of portability invariants; LP-invariants add constraints, never relax NL-invariants
- Portability does not imply minimal featureset: universal features (those satisfying LP-3) are encouraged

## 3. Core Invariants

Rules that every implementation of this spec (and its host projects) MUST NOT violate:

- **LP-1 Host neutrality**: the library's source tree must not import, reference, or conditionally compile any type, identifier, or path belonging to a specific host project. All host-side logic flows inward through the public extension-point API.

- **LP-2 Extension via abstract interfaces**: every integration point with the host environment is expressed as a named abstract interface (trait, protocol, or equivalent). Concrete implementations of that interface live in the host project or in separate adaptor crates — never in the library core. The library ships exactly one built-in implementation per interface, sufficient for in-process testing without I/O.

- **LP-3 Two-host generalisation rule**: a pattern or feature observed in one host project is admitted into nodus only when its abstraction is demonstrably useful to at least two independent host contexts and requires no host-specific type in its interface. Single-host optimisations remain in the host project's adaptor layer.

- **LP-4 Vocabulary isolation**: the built-in schema vocabulary (command names, reserved variable names) is a fixed, versioned baseline. Host-specific commands are declared in schema files (§schema:) loaded at runtime; they do not modify library constants. A library release never adds a host-specific command to the shared vocabulary.

- **LP-5 Composable extension**: universal patterns are delivered as independently composable units — abstract interfaces, concrete value types, combinators, or pure functions. Feature flags, global mutable state, and inheritance hierarchies are forbidden as primary extension mechanisms.

- **LP-6 Semantic versioning contract**: the published public API surface follows strict semantic versioning:
  - *Patch*: documentation-only or internal changes with no observable API difference
  - *Minor*: additive changes — new interfaces, new methods with default implementations, new public types
  - *Major*: any removal, rename, or incompatible signature change in a previously published item; requires advance notice

- **LP-7 Feedback loop lifecycle**: host-observed needs travel through a defined lifecycle before entering the library:
  1. *Discovery* — host developer identifies a recurring gap or pattern in production use
  2. *Distillation* — host-specific framing is rephrased as a host-agnostic requirement; LP-3 assessed
  3. *Proposal* — a spec amendment is authored capturing the generalised invariant or extension point
  4. *Implementation* — the spec-to-execution pipeline delivers and validates the feature
  5. *Release* — library version is bumped per LP-6; host updates its dependency

- **LP-8 Capability manifest & pre-run satisfiability**: a workflow declares, in its definition, the set of extension-point roles and named capabilities it requires to execute — for example a model provider, durable storage, a specific audit sink, or named commands drawn from a host schema layer (§4.4). Before execution begins, the runtime validates this manifest against the capabilities the active host actually provides. If any required capability is unsatisfiable, the workflow is rejected fail-fast with a diagnostic naming the missing capability; a partially-capable run never starts. The same manifest is the contract checked when a workflow moves between hosts (LP-3): a workflow is portable to a second host only if that host satisfies its manifest. The manifest references roles from the §4.1 taxonomy and never names a concrete host type (LP-1).

## 4. Detailed Design

### 4.1 Extension Point Taxonomy

Each extension point is identified by role. An implementation must provide exactly one built-in (stub or no-op) implementation that satisfies the interface without external I/O.

| Role | Interface name | Responsibility | Built-in |
| --- | --- | --- | --- |
| Model | `ModelProvider` | Invoke AI model; return structured output | `StubProvider` (deterministic) |
| Storage | `StorageProvider` | Persist and retrieve workflow state across invocations | <!-- TBD: define when first host requires durable state --> |
| Audit | `AuditProvider` | Record execution events for observability and replay | `NoopAuditProvider` (discards all events; see `l1-nodus-observability.md`) |
| Policy | `PolicyProvider` | Evaluate runtime policy decisions beyond hard schema rules: spend caps, approval gates, tool-access restrictions | <!-- TBD: define when first host requires dynamic policy beyond !!NEVER/!PREF --> |

> New roles are added only when LP-3 is satisfied. The table above is the authoritative extension point registry; an implementation that introduces a role not listed here must first amend this spec.

### 4.2 Feedback Distillation Protocol

The feedback loop is the mechanism by which a host project improves the library without coupling it to the host.

**Distillation criteria (all must hold)**:

1. The feature is expressible using only types already in the library's public API or primitive language types
2. The feature does not require the library to import a new external dependency
3. At least two independent host-project usage contexts benefit (LP-3)
4. The feature strengthens one of the language invariants (NL-1…NL-10) or adds a new, independently useful extension point role

**Distillation anti-patterns** (auto-reject signals):

- Feature name or type includes a host-project identifier (e.g., a product name, internal subsystem label)
- Feature requires modifying the base schema vocabulary with commands that only make sense in one domain
- Feature disables or weakens an existing NL-invariant for the host's convenience

### 4.3 Universal Pattern Criteria

A reusable pattern from the host project qualifies for inclusion in the library when:

- It is expressed as a pure function, a value type, or an abstract interface (LP-2, LP-5)
- It composes with existing patterns without requiring changes to their signatures
- It passes LP-3 (two-host generalisation rule)
- It has at least one passing automated test exercising it without any host-project dependency

Patterns that do not yet satisfy LP-3 may be documented in the host project's adaptor layer as candidates; they graduate to nodus via a spec amendment and the feedback lifecycle (LP-7).

### 4.4 Vocabulary Extension Model

The schema vocabulary is a layered system:

```text
[REFERENCE]
Built-in baseline (shipped by nodus)
  └── §schema: <built-in>  — KNOWN_COMMANDS + RESERVED_VARIABLES
Host extension layer (owned by host project)
  └── §schema: <host-file.nodus>  — additional commands, domain-specific rules
Workflow layer (authored by users)
  └── §wf: <workflow.nodus>  — @runtime: { core: <schema-file> }
```

A workflow declares which schema it targets. The library validates against that schema. Host commands never propagate into the built-in baseline unless they satisfy LP-3 and pass the spec amendment process.

### 4.5 Versioning and Backward Compatibility

Breaking changes (major version) MUST be:

- Announced in the library's CHANGELOG at least one minor release before they take effect
- Accompanied by a migration note describing the exact substitution
- Validated against all known host projects (at minimum the originating host) before release

Non-breaking additions (minor version) that introduce new extension-point roles MUST:

- Provide a default no-op built-in implementation so existing hosts compile without changes
- Be listed in the extension point taxonomy (§4.1)

### 4.6 Capability Manifest & Pre-Run Validation

A workflow's manifest is a declarative list of what it needs from its host, expressed only in terms of the extension-point taxonomy (§4.1) and named schema capabilities (§4.4) — never concrete host types:

```text
[REFERENCE]
manifest := {
  roles        : set of ExtensionRole      // e.g. { Model, Storage }
  commands     : set of CommandName        // host-schema commands the workflow calls
  capabilities : set of NamedCapability    // optional finer-grained features of a role
}
```

The runtime resolves the manifest against the active host before the first step executes:

```text
[REFERENCE]
validate(manifest, host):
    missing := []
    for role in manifest.roles:        if not host.provides(role):        missing.append(role)
    for cmd  in manifest.commands:     if not host.schema.has(cmd):        missing.append(cmd)
    for cap  in manifest.capabilities: if not host.satisfies(cap):         missing.append(cap)
    if missing not empty:
        reject_fail_fast("unsatisfiable capability manifest", missing)   // LP-8: never start
```

Two consequences follow. First, failures surface as a precise pre-run diagnostic ("workflow requires Storage; host provides none") instead of an opaque mid-run error after side effects have begun. Second, the manifest is the machine-checkable portability contract: the two-host generalisation rule (LP-3) reduces, for a given workflow, to "does host B satisfy the same manifest host A satisfied?". A built-in host (the in-process test configuration) satisfies a minimal manifest, so manifest-free or model-only workflows remain runnable without any host wiring.

## 5. Implementation Notes

The LP-invariants are evaluated in the order that minimises rework:

1. LP-1 (host neutrality) — check first; a violation here invalidates everything downstream
2. LP-3 (two-host rule) — evaluate before writing any spec amendment
3. LP-2, LP-5 (interface + composability shape) — define the interface before implementing
4. LP-4 (vocabulary isolation) — check schema constants after any vocabulary change
5. LP-6 (versioning) — apply at release time; LP-2/LP-5 shape determines major vs. minor

## 6. Drawbacks & Alternatives

**Alternative: feature flags per host** — host-specific behaviour toggled at compile time. Rejected: violates LP-1 (host types in library) and LP-5 (no feature-flag extension mechanism).

**Alternative: monorepo-internal coupling** — rely on workspace-level access to host types. Rejected: defeats extraction; LP-1 is the hard boundary.

**Alternative: single-host-driven evolution** — evolve nodus exclusively from one host's needs. Rejected: leads to host-specific accumulation over time; LP-3 is the safeguard.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[L1-LANG]` | `crates/nodus/src/vocab.rs` | KNOWN_COMMANDS and RESERVED_VARIABLES — the vocabulary baseline LP-4 protects |
| `[EXT-POINT]` | `crates/nodus/src/executor.rs` | ModelProvider trait — canonical example of LP-2 extension interface |
| `[PUBLIC-API]` | `crates/nodus/src/lib.rs` | Re-exported public surface — LP-6 versioning scope |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.1.0 | 2026-06-26 | Core Team | Added LP-8 (capability manifest + pre-run satisfiability validation, fail-fast); new §4.6; manifest reframed as the machine-checkable LP-3 portability contract. |
| 1.0.1 | 2026-06-24 | Core Team | AuditProvider row filled — references l1-nodus-observability.md; PolicyProvider TBD refined |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — portability contract, LP-1…LP-7, extension taxonomy, feedback lifecycle |

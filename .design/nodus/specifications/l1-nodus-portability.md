# Nodus Portability and Extension Contract

**Version:** 1.7.0
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

- **LP-9 Extraction attestation**: [ADDED v1.2.0] the extraction deliverables (§4.5) — the portable library bundle and its accompanying capability manifest — carry an **independently-verifiable witness** binding the bundle's exact content set and its version. A host verifies the witness **before loading** an imported workflow bundle, and refuses it fail-closed when the witness is missing where required or fails verification (tampered content, unknown signer, superseded). The signing and verification mechanism is **host-supplied** (LP-2), so the library core carries no cryptographic dependency and stays host-neutral; the witness, like the manifest, never names a concrete host type (LP-1). This is the nodus realization of the host attestation contract: a workflow library that moves between hosts (LP-3) proves its integrity and authorship on arrival, rather than being trusted because of how it was fetched.

- **LP-10 Host-granted authority, never self-authored**: [ADDED v1.3.0] a workflow has **no ambient authority**. It **declares** the capabilities it needs (the LP-8 manifest) and **reads** host-supplied policy decisions (the PolicyProvider extension role), but it MUST NOT grant itself a capability, relax a policy, widen its reach, or otherwise author the authority under which it runs — the authority plane is **host-supplied and workflow-read-only**. The language gives a workflow no vocabulary to *write* policy: a step may *request* an outcome (a dialog or approval, NL-12) but never self-authorize one, and the request is data, not a grant. Because a workflow may be assembled under untrusted influence — a definition built after reading model output or external content is itself untrusted (NL-11) — its declared-capability manifest is the **single consent point the host resolves before the run**, not a per-step self-grant; an injected instruction inside a workflow therefore cannot escalate what the run may do beyond its pre-consented manifest. The library defines only *declare-and-read*; *granting* stays entirely host-side (LP-1/LP-2). This is the nodus realization of the main `l1-security` SEC-10 authority-self-containment contract: a portable workflow, like the agent that runs it, can ask for authority but never mint it.

- **LP-11 Per-effect authorization seam**: [ADDED v1.4.0] an **effectful step** — one that calls a model (GEN/REFINE), invokes a host tool/operation, or suspends for a deferred/external completion (NL-12) — MAY be gated by a host-supplied **authorization decision** evaluated **before** the effect occurs. The runtime guarantees the ordering **decide → effect → observe**: the authorization runs before the effect is performed, and the audit observer (`l1-nodus-observability` HO-*) runs after, against the real result. The gate is **fail-closed**: a *deny*, or an authorization that errors or times out, aborts the effect with a typed `NODUS:*` code — it never silently permits and never hangs (fail-fast). The decision is **host-supplied** through the `PolicyProvider` role (LP-2): the nodus core names no policy vocabulary and no metric; a step may **declare its effect class** (model-call / tool-use / deferred) so the host gate can match, but consistent with LP-10 it can neither author nor relax the policy. This is the *dynamic per-effect* complement to the *static whole-run* LP-8 manifest: LP-8 rejects, before the run, an effect the host **cannot** satisfy (the effect is stripped/unavailable); LP-11 evaluates, per attempt, an effect the host **can** satisfy but may refuse on its arguments (the effect is attempted and may be denied). A step with no matching policy proceeds unchanged, and a host supplying no `PolicyProvider` runs exactly as today (additive). This is the nodus realization of the main `l1-interception-model` decide-class seam (INT-1), its check-before-use ordering (INT-2), its fail-closed direction for guards (INT-3), and its strip-vs-deny axis (INT-6, LP-8 = strip / LP-11 = deny).

- **LP-12 Imported-bundle admission vetting seam**: [ADDED v1.5.0] before a host **loads** an imported workflow bundle or a host schema/vocabulary extension (§4.4), the runtime MAY submit the incoming artifact to a host-supplied **vetting classifier** that returns typed findings over the artifact's content — its embedded natural language (step prompts, macro bodies, schema command descriptions), its declared capability manifest (LP-8), and its declared effect classes (LP-11) — and the runtime enforces the host's admission disposition: **fail-closed refusal to load** on a finding the host marks admission-blocking (a typed `NODUS:*` code, never a partial load), pass-through with the verdict **recorded as load-time provenance** otherwise. This is **content-safety, distinct from and sequenced after LP-9 authenticity**: LP-9 verifies the bundle is what its author shipped, LP-12 asks whether *what the author shipped is safe to admit* — an authentic bundle can still carry an injected instruction (NL-11), and because a bundle may be assembled under untrusted influence its content is untrusted until vetted. The classifier, its heuristics, its severity scale, and its finding taxonomy are **entirely host-supplied** (LP-2): the nodus core names no threat pattern, no severity, no scanner, so a host supplying no vetting classifier loads exactly as today (**additive**). The verdict crosses the observability data-safety boundary as counts/descriptors, never raw secret content (`l1-nodus-observability` §4.4). Nodus's own static substrate — NL-11 value provenance and LP-11 effect classes — is the material a host classifier reads to compute a **cross-effect toxic-flow** finding (an untrusted-provenance value reaching an external-egress effect) over the bundle's graph, but the flow policy (which effect classes count as egress, which composition is dangerous) stays host-supplied (LP-1/LP-2). This is the nodus realization of the main `l1-component-scanning` admission-vetting contract — its vetting-gate placement (CS-2), its content-≠-integrity separation (CS-3), its cross-component flow analysis (CS-6), its advisory-vs-critical disposition (CS-8), and its channel data-safety (CS-9) — the **pre-load static** complement to the **per-effect runtime** gate LP-11 (LP-9 attest → LP-12 vet → run).

- **LP-13 Addressable versioned import resolution**: [ADDED v1.6.0] an imported workflow bundle or host schema/vocabulary extension MAY be referenced by a **stable name + version constraint** rather than a concrete artifact; **before the load gate**, a host-supplied **resolver** resolves the reference to a **concrete pinned revision** (an exact content revision) drawn from a host-supplied source or catalog. Resolution is **deterministic given a recorded pin** (a lockfile-like binding), so the same reference reproduces the same bundle, and the resolved pin is the identity that **LP-9 attests and LP-12 vets** — the full load order is **resolve → attest → vet → run**. The catalog, the source addressing, the transport, and the version-selection policy are **entirely host-supplied** (LP-2): the nodus core names no registry protocol, no URL scheme, and no version-range algebra beyond letting a workflow *declare* the constraint, so a host that supplies no resolver runs exactly as today with only concrete artifacts (**additive**). An **unresolvable, ambiguous, or unpinned** reference is rejected **fail-fast pre-run** — a missing bundle dependency is a missing capability, resolved before the first step or not at all (LP-8 kinship), never mid-run. A resolved bundle's **identity slug is stable**: a rename is a host-supplied migration recorded in the catalog, not a silent re-point, so a reference can never be transparently swapped to a *different* bundle behind a user who pinned it. This extends the LP-8 manifest (roles + commands + capabilities) with **named external bundle dependencies** and their resolution, and it is the nodus realization of the main `l1-extension-marketplace` distribution contract — its addressable-sourced entry (XM-1), reproducible version-pinning (XM-3), stable identity + governed rename (XM-2), and resolution-feeds-the-load-gate flow (XM-8) — the **pre-load resolution** stage that precedes the LP-9 / LP-12 load gate.

- **LP-14 Verified-peer delegation seam**: [ADDED v1.7.0] a deferred/external step (NL-12) MAY target a **named external peer agent** rather than an anonymous host-supplied completion — the peer's **verifiable identity** and the **offered capability** it is invoked for are resolved by a host-supplied **peer directory / identity provider** before the delegation. The runtime guarantees the peer is **resolved and authorized before the effect** — the LP-11 per-effect gate applies to a peer call exactly as to any effect (**decide → delegate → observe**); the peer's result enters the run as **untrusted provenance** (NL-11, a peer is an external actor) and MAY carry a host-supplied **execution receipt** (the HO-9 authenticity witness) so a fabricated peer result is distinguishable from a real one; an **unresolvable or unauthorized** peer is rejected **fail-fast pre-delegation** (never mid-run, LP-8 kinship — a required peer/capability is a declared capability). The peer identity, the directory, the transport, the channel encryption, and any settlement are **entirely host-supplied** (LP-1/LP-2): the nodus core names no network, no key, no ledger, and no chain — it contributes only the *declare-a-peer-delegation-and-let-the-host-resolve-and-gate-it* seam, **generalizing NL-12's "external actor" from an anonymous completion to a verifiable named peer** (a human dialog remains the special case where the actor is a person; a peer agent is the case where it is another sovereign agent). A workflow that delegates to no peer is unaffected (**additive**). This is the nodus realization of the main `l1-agent-federation` peer-interaction contract — the sovereign verifiable identity the host resolves (AF-1), the default-deny peer trust enforced by the LP-11 gate plus the NL-11 untrusted result (AF-6), and the accountable attribution carried by the optional receipt (AF-7/AF-8) — while identity issuance, discovery, channels, and exchange stay host-side.

### 4.1 Extension Point Taxonomy

Each extension point is identified by role. An implementation must provide exactly one built-in (stub or no-op) implementation that satisfies the interface without external I/O.

| Role | Interface name | Responsibility | Built-in |
| --- | --- | --- | --- |
| Model | `ModelProvider` | Invoke AI model; return structured output | `StubProvider` (deterministic) |
| Storage | `StorageProvider` | Persist and retrieve workflow state across invocations | <!-- TBD: define when first host requires durable state --> |
| Audit | `AuditProvider` | Record execution events for observability and replay | `NoopAuditProvider` (discards all events; see `l1-nodus-observability.md`) |
| Policy | `PolicyProvider` | Evaluate a per-effect authorization decision before an effectful step (model call / tool use / deferred completion): allow / deny / require-approval, fail-closed (LP-11, §4.7) | Built-in *allow-all* (no gate; existing behaviour) |

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

### 4.7 Per-Effect Authorization Seam (LP-11)

The `PolicyProvider` role is a *decide-class* seam over effectful steps. It is distinct from the LP-8 manifest along two axes: **when** (LP-8 pre-run, once; LP-11 per effect, at each attempt) and **what** (LP-8 checks a capability *exists*; LP-11 checks this *use* of it is permitted). The executor brackets every effectful step so the authorization is settled before the effect and the observer runs after it:

```text
[REFERENCE]
run_effect(step, host):
    if host.has(PolicyProvider):
        decision := host.policy.authorize(step.effect_class, step.args)   // decide, pre-effect
        // fail-closed: any of {Deny, error, timeout} -> abort
        if decision != Allow:
            emit(NODUS:CAPABILITY_UNMET | NODUS:POLICY_DENIED)            // typed, never hang
            route_to(@err) or bypass per NL-2                            // !! violations bypass @err
            return
    result := perform(step)                    // the effect happens exactly once, here
    host.audit.record(step, result)            // observe, post-effect (HO-*, after the real result)
    return result

effect_class(step) := one of { model_call, tool_use, deferred }   // declared, host-matchable (LP-10: read-only)
```

Two guarantees follow. First, the ordering is authoritative: a `PolicyProvider` never sees a post-effect state and an `AuditProvider` never observes a not-yet-happened effect — mirroring the main interception model's check-before-use rule. Second, the seam is purely additive: with no `PolicyProvider` the built-in *allow-all* applies and execution is byte-for-byte today's behaviour, so LP-11 imposes no cost on hosts that do not opt in.

### 4.8 Imported-Bundle Vetting Seam (LP-12)

The vetting seam is a host-supplied *content classifier* consulted at **load time**, shaped exactly like the LP-9 witness verifier (a verify-before-load hook, not a new extension-point role): the core defines only the call site and the fail-closed disposition, never a heuristic. It runs after LP-9 authenticity and before executor boot — the load-gate order is **attest → vet → run**:

```text
[REFERENCE]
load_bundle(bundle, host):
    if host.has(WitnessVerifier) and not host.verify_witness(bundle):   // LP-9 authenticity
        refuse_fail_closed("attestation failed")                        // is it what the author shipped?
    if host.has(VettingClassifier):
        findings := host.vet(bundle.content, bundle.manifest, bundle.effect_classes)   // LP-12 content-safety
        // findings are host-typed data (code, severity, evidence); the core reads only the disposition
        if any f in findings where host.blocks_admission(f):
            emit(NODUS:ADMISSION_REFUSED)                               // typed, never a partial load
            refuse_fail_closed(findings)
        record_load_provenance(bundle, findings)                        // vetted-with-verdict, data-safe (counts/descriptors)
    boot_executor(bundle)                                               // only a vetted, authentic bundle boots
```

Two properties mirror the rest of the contract. First, **separation of concerns**: LP-9 answers *authentic?* and LP-12 answers *safe to admit?* — a bundle must pass both, because an authentic artifact can carry an injected instruction (NL-11) and a benign one can arrive tampered. Second, **host-neutrality is preserved**: the core names no threat pattern, severity, or metric (LP-2), so it cannot itself judge a bundle malicious — it only enforces the host's disposition and records the verdict; a host with no `VettingClassifier` loads exactly as today (additive). The material a host classifier reasons over is already in the artifact: the NL-11 provenance of interpolated values and the LP-11 effect class of each effectful step let a classifier detect a **cross-effect toxic flow** (untrusted-provenance value → external-egress effect) statically over the bundle graph, while the flow policy itself stays host-supplied.

### 4.9 Addressable Import Resolution Seam (LP-13)

The resolver is a host-supplied hook consulted **before** the LP-9/LP-12 load gate — the same verify-before-load shape as the witness verifier and the vetting classifier, adding no §4.1 role. The core defines only the reference form (a name + constraint), the pinned-binding record, and the fail-fast disposition; it names no registry or transport:

```text
[REFERENCE]
import_ref := { name: Slug, constraint: VersionConstraint }   // declared in the workflow / manifest

acquire(import_ref, host):
    if not host.has(Resolver):
        reject_fail_fast("unresolved import; no resolver", import_ref)   // LP-8 kinship: missing dependency
    pin := host.resolve(import_ref.name, import_ref.constraint)          // host-supplied catalog/source/policy
    if pin is None or ambiguous(pin):
        reject_fail_fast("unresolvable or ambiguous import", import_ref)  // pre-run, never mid-run
    record_lock(import_ref, pin)             // deterministic: same ref + lock -> same bundle
    bundle := host.fetch(pin)                // host-supplied transport
    return load_bundle(bundle, host)         // -> LP-9 attest -> LP-12 vet -> boot  (§4.8)
```

Two properties complete the chain. First, **ordering**: resolution produces the concrete pin that authenticity (LP-9) and content-safety (LP-12) are then scoped to — *resolve → attest → vet → run* — so a floating reference never reaches the executor unpinned. Second, **host-neutrality and additivity**: a workflow that names only concrete bundles needs no resolver and behaves exactly as today; the core carries no registry, version algebra, or transport (LP-1/LP-2). Slug stability (a rename is a recorded catalog migration, not a silent re-point) means a pinned reference cannot be swapped for a different bundle, mirroring the marketplace's immutable-identity contract on the host side.

### 4.10 Verified-Peer Delegation Seam (LP-14)

Peer delegation reuses the NL-12 suspend/correlate/resume machinery unchanged; LP-14 adds only *who the external actor is* — a named, host-verified peer rather than an anonymous completion — and reuses the existing gate and provenance seams so no new trust primitive enters the core:

```text
[REFERENCE]
delegate_to_peer(step, host):
    if not host.has(PeerDirectory):
        reject_fail_fast("peer delegation unsupported", step.peer_ref)   // LP-8 kinship
    peer := host.resolve_peer(step.peer_ref, step.capability)            // identity + offered capability
    if peer is None or not verify_identity(peer):
        reject_fail_fast("unresolvable or unverified peer", step.peer_ref) // pre-delegation, never mid-run
    if host.has(PolicyProvider):
        if host.policy.authorize(effect=peer_call, args={peer, step.capability}) != Allow:
            emit(NODUS:POLICY_DENIED); route_to(@err); return             // LP-11 gate, fail-closed, default-deny
    token  := suspend_with_correlation(step)                             // NL-12: yield Status::Paused
    result := host.await_peer(peer, step.capability, token)              // host owns channel/transport/settlement
    mark(result, provenance = Untrusted)                                 // NL-11: a peer is an external actor
    host.audit.record(step, result, receipt = result.receipt?)          // HO-9 witness optional
    resume(step, result)                                                 // deterministic, correlated (NL-12)
```

Everything nodus-side already exists: NL-12 suspends and correlates, LP-11 gates the effect before it happens, NL-11 marks the result untrusted, HO-9 optionally witnesses authenticity. LP-14 is the thin declaration that the deferred actor is a *verified sovereign peer* the host resolves and gates — the identity system, directory, encryption, and settlement stay entirely host-supplied (LP-1/LP-2), so a host with no `PeerDirectory` runs exactly as today.

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
| 1.7.0 | 2026-07-09 | Core Team | Added LP-14 (verified-peer delegation seam) — a deferred/external step (NL-12) MAY target a named external peer agent rather than an anonymous host-supplied completion; the peer's verifiable identity + offered capability are resolved by a host-supplied peer directory/identity provider before the delegation, the runtime guaranteeing resolve+authorize before the effect (the LP-11 per-effect gate applies: decide → delegate → observe), the peer's result entering as untrusted provenance (NL-11) and optionally carrying a host-supplied execution receipt (HO-9 witness) so a fabricated peer result is detectable; an unresolvable or unauthorized peer is rejected fail-fast pre-delegation (LP-8 kinship, never mid-run); the peer identity, directory, transport, channel encryption, and settlement are entirely host-supplied (LP-1/LP-2, no network/key/ledger/chain in core), so a workflow delegating to no peer is unaffected (additive); generalizes NL-12's "external actor" from an anonymous completion to a verifiable named peer (a human dialog is the special case where the actor is a person); new §4.10, reusing the NL-12 suspend/correlate/resume + LP-11 gate + NL-11 provenance + HO-9 receipt seams with no new trust primitive in the core. The nodus realization of the new main l1-agent-federation peer-interaction contract (AF-1 sovereign verifiable identity the host resolves / AF-6 default-deny peer trust via the LP-11 gate + NL-11 untrusted result / AF-7/AF-8 accountable attribution via the optional receipt) while identity issuance, discovery, channels, and exchange stay host-side. L1 stays Stable (C9); l2-nodus-portability carries LP-14 as a pending Invariant-Compliance obligation reconciled at magic.task (LP-8…LP-13 precedent). |
| 1.6.0 | 2026-07-09 | Core Team | Added LP-13 (addressable versioned import resolution) — an imported workflow bundle or host schema/vocabulary extension MAY be referenced by a stable name + version constraint that a host-supplied resolver resolves to a concrete pinned revision, from a host-supplied source/catalog, before the load gate; resolution is deterministic given a recorded pin (lockfile-like), the resolved pin being the identity LP-9 attests and LP-12 vets — full load order resolve → attest → vet → run; catalog/source-addressing/transport/version-selection entirely host-supplied (LP-2, no registry protocol/URL scheme/version algebra in core) so a host with no resolver runs as today with only concrete artifacts (additive); an unresolvable/ambiguous/unpinned reference is rejected fail-fast pre-run (LP-8 kinship — a missing bundle dependency is a missing capability, never mid-run); a resolved bundle's identity slug is stable (a rename is a host-supplied recorded migration, not a silent re-point), so a pinned reference cannot be swapped for a different bundle; extends the LP-8 manifest with named external bundle dependencies + their resolution; new §4.9. The nodus realization of the new main l1-extension-marketplace distribution contract (XM-1 addressable-sourced entry / XM-2 stable identity + governed rename / XM-3 reproducible pinning / XM-8 resolution-feeds-the-load-gate) — the pre-load resolution stage that precedes the LP-9/LP-12 load gate. L1 stays Stable (C9); l2-nodus-portability carries LP-13 as a pending Invariant-Compliance obligation reconciled at magic.task (LP-8/LP-9/LP-10/LP-11/LP-12 precedent). |
| 1.5.0 | 2026-07-09 | Core Team | Added LP-12 (imported-bundle admission vetting seam) — before loading an imported workflow bundle or a host schema/vocabulary extension, the runtime MAY submit the artifact to a host-supplied vetting classifier returning typed content findings (over embedded NL, the LP-8 manifest, and LP-11 effect classes) and enforces the host's admission disposition: fail-closed refusal on an admission-blocking finding (typed `NODUS:*`, never a partial load), pass-through with the verdict recorded as load-time provenance otherwise; content-safety distinct from and sequenced after LP-9 authenticity (an authentic bundle can still carry an injected instruction, NL-11), the load-gate order attest → vet → run; classifier/heuristics/severity/taxonomy entirely host-supplied (LP-2, no threat pattern or metric in core) so a host with no classifier loads exactly as today (additive); verdict crosses the observability data-safety boundary as counts/descriptors (§4.4); NL-11 provenance + LP-11 effect classes are the static substrate a host reads to compute a cross-effect toxic-flow finding over the bundle graph, the flow policy staying host-supplied; new §4.8. The nodus realization of the new main l1-component-scanning admission-vetting contract (CS-2 gate / CS-3 content-≠-integrity / CS-6 cross-component flow / CS-8 advisory-vs-critical / CS-9 channel data-safety), the pre-load static complement to the per-effect runtime gate LP-11. L1 stays Stable (C9); l2-nodus-portability carries LP-12 as a pending Invariant-Compliance obligation reconciled at magic.task (LP-8/LP-9/LP-10/LP-11 precedent). |
| 1.4.0 | 2026-07-06 | Core Team | Added LP-11 (per-effect authorization seam) — an effectful step (model call / tool use / deferred NL-12 completion) MAY be gated by a host-supplied authorization decision evaluated before the effect, with the runtime guaranteeing decide → effect → observe ordering (the AuditProvider observer runs after, on the real result); fail-closed on deny/error/timeout with a typed `NODUS:*` code (never silently permits, never hangs); the decision is host-supplied through the now-concretized `PolicyProvider` role (LP-2, no policy vocabulary in core), a step declaring only its effect class for host matching and — per LP-10 — never authoring or relaxing the policy; the dynamic per-effect complement to the static whole-run LP-8 manifest (LP-8 = strip an unsatisfiable effect pre-run, LP-11 = deny a satisfiable effect per attempt); purely additive — no PolicyProvider = built-in allow-all = today's behaviour. §4.1 PolicyProvider row concretized (TBD resolved), new §4.7. The nodus realization of the new main l1-interception-model decide-class seam INT-1 / check-before-use ordering INT-2 / fail-closed guard direction INT-3 / strip-vs-deny axis INT-6. L1 stays Stable (C9); l2-nodus-portability carries LP-11 as a pending Invariant-Compliance obligation reconciled at magic.task (LP-8/LP-9/LP-10 precedent). |
| 1.3.0 | 2026-07-02 | Core Team | Added LP-10 (host-granted authority, never self-authored) — a workflow has no ambient authority: it declares capability needs (LP-8) and reads host-supplied policy (PolicyProvider) but MUST NOT grant itself a capability, relax a policy, or widen its reach; the language has no vocabulary to write policy, a step may request an outcome (NL-12) but never self-authorize; because a workflow may be assembled under untrusted influence (NL-11) its declared-capability manifest is the single consent point the host resolves before the run, so an injected instruction cannot escalate beyond the pre-consented manifest; granting stays host-side (LP-1/LP-2). The nodus realization of the new main l1-security SEC-10 authority-self-containment contract. L1 stays Stable (C9); l2-nodus-runtime carries LP-10 as a pending Invariant-Compliance obligation reconciled at magic.task. |
| 1.2.0 | 2026-07-02 | Core Team | Added LP-9 (extraction attestation) — the extraction bundle + capability manifest carry an independently-verifiable witness binding exact content-set + version; a host verifies before loading an imported workflow bundle and refuses fail-closed on missing/failed witness; signing/verification mechanism host-supplied (LP-2, no crypto dependency in core), witness never names a host type (LP-1). The nodus realization of the host attestation contract (main l1-attestation AT-1/AT-2/AT-6); a library moving between hosts (LP-3) proves integrity+authorship on arrival. |
| 1.1.0 | 2026-06-26 | Core Team | Added LP-8 (capability manifest + pre-run satisfiability validation, fail-fast); new §4.6; manifest reframed as the machine-checkable LP-3 portability contract. |
| 1.0.1 | 2026-06-24 | Core Team | AuditProvider row filled — references l1-nodus-observability.md; PolicyProvider TBD refined |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — portability contract, LP-1…LP-7, extension taxonomy, feedback lifecycle |

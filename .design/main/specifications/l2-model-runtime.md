# Model Runtime (Transport & Provider Connectivity)

**Version:** 1.0.0
**Status:** RFC
**Layer:** implementation
**Implements:** l1-model-runtime.md

## Overview

The concrete realization of the local model runtime for the Rust workspace: the **transport
layer** that turns a routing decision into an actual inference call. It binds the two
existing-but-unwired provider seams — the workspace contract's `ModelProvider` trait
(authoritative) and the workflow runtime's provider trait (bridged via a host-side adapter) —
to real serving backends: the six federated local REST providers from the technology-stack
catalog, and explicitly egress-gated remote APIs.

This spec exists to close a structural gap: model *selection* (router, credential lanes,
hardware fit) is fully specified and implemented, while no component can actually reach a
model — the workspace deliberately contains no HTTP transport today. This L2 defines that
transport: synchronous-first, thread-scoped, streaming-capable, and cancellable, without
introducing an async runtime into the core.

## Related Specifications

- [l1-model-runtime.md](l1-model-runtime.md) - The L1 parent (MR-1…MR-14) this spec realizes.
- [l2-technology-stack.md](l2-technology-stack.md) - §4.4 local provider catalog (six REST providers, probe discipline, endpoints) this transport federates to.
- [l2-model-router.md](l2-model-router.md) - Selection, scoring, and hardware-fit that sit *above* this transport; the router picks, this spec calls.
- [l2-model-error-recovery.md](l2-model-error-recovery.md) - Error taxonomy and retry/rotate/fallback applied to transport failures.
- [l2-crate-topology.md](l2-crate-topology.md) - The contract · domain · infra-crate · facade partition this transport crate slots into (DN-2 seam as a crate boundary).
- [l1-security.md](l1-security.md) - Egress gate and secret isolation governing remote backends and credentials.
- [l1-routing.md](l1-routing.md) - RTG-10 credential lanes carried through to the transport's authentication headers.

## 1. Motivation

Every model-consuming subsystem (orchestration, memory generators, triage, compaction,
nodus workflows) targets a provider seam, and every seam currently dead-ends: the contract
trait has no shipped implementation, and the workflow runtime executes against a built-in
stub. The router can *decide* which model should serve a request, record credential lanes,
and estimate hardware fit — and then nothing can perform the call. This is the single gap
that keeps the office's autonomy inert end to end.

The transport must respect two standing architectural facts: the workspace is synchronous by
design (the stack's provider probes are *parallel thread-scoped*, not async), and the
dependency policy is std-first — a third-party crate is added only when strictly necessary.
An HTTP+TLS client is such a necessity; an async runtime is not.

## 2. Constraints & Assumptions

- **No async runtime.** The core stays synchronous. Concurrency is bounded worker threads
  owned by the runtime component; streaming is delivered through a pull-based iterator /
  callback contract, not futures.
- **Minimal transport dependency.** One blocking HTTP client with rustls-based TLS
  (candidate: `ureq`-class), justified under the dependency policy as strictly necessary.
  No websocket/gRPC dependency — the entire provider catalog is REST.
- **Federated local serving.** Cronus does not reimplement weight storage or model serving
  in v1: it federates to installed local servers (the §4.4 catalog) that own acquisition,
  storage, and loading. The Cronus-native content-addressed store remains a later,
  spec-gated extension.
- **nodus stays dependency-free.** The bridge adapter that implements the workflow runtime's
  provider trait lives host-side (facade tier), delegating to the contract provider. nodus
  itself gains no dependency (LP-1 preserved).
- **Remote backends are opt-in.** Any non-loopback endpoint requires the security egress
  gate and a credential from the secret store; loopback providers require neither.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| MR-1 Local-first serving | Default provider set = the six loopback REST providers (stack §4.4); a non-loopback endpoint is constructible only through the egress-gated remote-backend path with an explicit user grant; no silent remote fallback — a failed local call surfaces through error-recovery, never re-routes off-device on its own. |
| MR-2 Provider-abstracted backends | One transport crate implements the contract's `ModelProvider` (load hints, generate-stream, embed, describe, unload hints) once per protocol family (`/v1`-compatible, provider-native REST); adding a provider is a new endpoint profile, never a caller change. The workflow-runtime provider trait is satisfied by a host-side adapter over the same contract implementation. |
| MR-3 Content-addressed model store | **Delegated (v1):** the federated local server owns manifests/blobs; the transport surfaces each provider's catalog (name, digest, size) through `describe`. A Cronus-native store is a disclosed deferral with an upgrade trigger (self-managed serving backend), per the frugality discipline. |
| MR-4 Verifiable named acquisition | **Delegated (v1):** pull-by-name is forwarded to providers that support it, progress-streamed to the caller; digest reported back post-pull. Acquisition is explicit (a `model pull` capability), never a hidden stall inside a generate call — a missing model fails fast with the acquisition hint. |
| MR-5 Portable model definition | **Delegated (v1):** definition-based customization forwards to the provider's definition mechanism where present; the transport records the resolved definition digest in `describe` output so a customized model stays reproducible. |
| MR-6 Explicit load/unload lifecycle | Load/unload hints map to the provider's residency controls (e.g. keep-alive/TTL parameters); resident-model introspection surfaces via the provider status endpoints; where a provider lacks a control, the transport reports the capability as absent rather than emulating it silently. |
| MR-7 Fit-gated hardware scheduling | The feasibility gate stays in the router's hardware-fit module; the transport contributes honest inputs (model size/quant from `describe`) and refuses a load the gate rejected. |
| MR-8 Streaming inference contract | Generate/chat/embed over the industry-compatible `/v1` shape (and provider-native shapes behind the same trait); incremental output is a pull-based token/event iterator fed by a worker thread; cancellation is a first-class handle (atomic flag + connection drop) honored mid-stream. |
| MR-9 Catalog management | List/details/remove forward to provider catalog endpoints through the same contract surface clients use; no manual file surgery — a provider without a management endpoint reports the gap explicitly. |
| MR-10 Managed server with observability | Federated servers are supervised, not owned: reachability probes (800 ms thread-scoped, per stack §4.4), health + resident state surfaced to the doctor and process-monitor planes; start/stop of provider processes is deferred to the doctor's authorized repair path. |
| MR-11 Thin clients over the server | CLI/TUI/GUI bind `model` verbs to the library methods only (`model list/status/pull/describe`); no frontend holds model state; offline provider ⇒ graceful degraded listing, marked unreachable. |
| MR-12 Versioned, reproducible references | Every completed call records the serving provider, model name, and digest (when the provider reports one) into the operational record, making "which weights answered" a stored fact. |
| MR-13 Multi-device placement | Delegated to the router's hardware-fit scoring; the transport passes through per-device residency reported by the provider where available and never fabricates placement data. |
| MR-14 Calibrated, honest estimates | The transport reports only measured facts (latency, tokens/s observed per call) tagged as measurements; estimates remain the router's output; the two are never merged in one field. |

> This table must be complete before the spec can reach RFC status.

## 4. Detailed Design

### 4.1 Crate placement and seam wiring

A new infra crate implements the contract seam, mirroring the store-local/auth-local
pattern from the crate topology:

```plaintext
crates/
├── contract/       # ModelProvider trait (exists, unchanged)
├── model-local/    # NEW: REST transport implementing contract::ModelProvider
├── domain/         # consumes the trait only — no transport dependency
└── core/ (facade)  # wires model-local into the engine; hosts the nodus bridge adapter
```

Dependency direction stays inward: `model-local → contract`; `domain` never depends on
`model-local`; the facade composes them. Adding this crate extends the topology the
crate-topology spec enumerates: implementation of this spec MUST be accompanied by a
crate-topology amendment registering `model-local` as a peer infra crate (same tier and
rules as `store-local`/`auth-local`) and by the corresponding CI boundary-guard update —
the guard's failure path is part of this spec's acceptance, mirroring how the topology
was landed. The nodus bridge adapter lives in the facade and
implements the workflow runtime's provider trait by delegating to the wired
`contract::ModelProvider` — nodus keeps zero dependencies, and workflow `gen` steps run
against real models the moment the facade is wired.

### 4.2 Synchronous streaming contract

```plaintext
[REFERENCE] — shape, not code
generate(request, cancel: CancelHandle) -> TokenStream
  TokenStream: blocking pull-iterator of Event { Token | ToolCall | Usage | Done | Error }
  CancelHandle: atomic flag; set → worker drops the connection, iterator yields Error(Cancelled)
```

A worker thread owns the HTTP connection and parses the provider's stream format (SSE or
NDJSON) into events pushed to a bounded channel; the caller pulls. Backpressure is the
bounded channel; a slow consumer never grows memory unbounded. One request = one worker;
total transport concurrency is capped by a small pool sized from the router's
concurrency budget.

### 4.3 Provider endpoint profiles

Each catalog provider is an **endpoint profile**: base URL, protocol family (`/v1`-compatible
or provider-native), capability flags (streaming, embeddings, pull, residency control,
digest reporting), and probe rules — data, not code. New providers are added by profile.
Probes follow the stack's discipline: thread-scoped, 800 ms timeout, loopback-only by
default, wildcard binds rejected.

### 4.4 Credentials and egress

Loopback profiles carry no credentials. A remote profile resolves its credential from the
secret store at call time (never cached in config), attaches it per the profile's auth
scheme, and is constructible only after the security egress grant for that endpoint. The
credential lane chosen by the router (RTG-10) is carried in the request metadata so the
transport uses the lane the router priced.

### 4.5 Failure surface

The transport maps wire failures onto the error-recovery taxonomy (connect-refused,
timeout, 4xx, 5xx, malformed-stream, cancelled) and never retries internally — retry,
rotate, and fallback belong to the error-recovery layer so policy stays in one place.

### 4.6 Frontend bindings

| Action | CLI | TUI | Library |
| --- | --- | --- | --- |
| list providers + reachability | `cronus model status` | `/model status` | `models.status()` |
| list available models | `cronus model list` | `/model list` | `models.list()` |
| model details (digest, size, params) | `cronus model describe <name>` | `/model describe <name>` | `models.describe(name)` |
| acquire by name | `cronus model pull <name>` | `/model pull <name>` | `models.pull(name, progress)` |

Per the shipped-surface honesty rule, these verbs appear on a frontend only once bound.

## 5. Implementation Notes

1. Endpoint profiles + probe reuse — the stack §4.4 probe logic is the discovery half; this
   crate adds the call half.
2. `/v1`-compatible generate path with streaming + cancellation — unlocks the largest
   provider subset first.
3. Facade wiring: contract provider into the engine; nodus bridge adapter.
4. Provider-native paths (pull, residency control) per capability flag.
5. Remote profiles behind the egress gate — last, after the local path is proven.

## 6. Drawbacks & Alternatives

- **Federation defers MR-3/4/5 depth:** v1 trusts the local server's store instead of a
  Cronus-native content-addressed store. Accepted as a disclosed deferral: the L1 contract
  is honored at the boundary (digests surfaced, acquisition explicit), and a native store
  remains a clean later extension behind the same trait.
- **Blocking transport limits massive fan-out:** a thread per in-flight stream caps
  practical concurrency well below an async design. Accepted: the product is single-user
  and on-device; the router's concurrency budget is the real ceiling, not the transport.
- **Alternative — adopt an async runtime now:** rejected; it would ripple `async` through
  the synchronous core and contradict the established thread-scoped probe design for no
  present need.
- **Alternative — one bespoke client per provider:** rejected; the catalog converges on
  REST with two protocol families, so profiles-over-one-client is strictly less code.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CONTRACT]` | `crates/contract/src/lib.rs` | The authoritative `ModelProvider` trait this crate implements |
| `[STACK]` | `.design/main/specifications/l2-technology-stack.md` | Provider catalog, endpoints, and probe discipline (§4.4) |
| `[ROUTER]` | `.design/main/specifications/l2-model-router.md` | Selection and hardware-fit sitting above this transport |
| `[RECOVERY]` | `.design/main/specifications/l2-model-error-recovery.md` | Error taxonomy the failure surface maps onto |

## Document History

| Version | Date | Notes |
| --- | --- | --- |
| 1.0.0 | 2026-07-16 | Initial RFC — closes the audit-identified transport gap: synchronous streaming REST transport crate (`model-local`) implementing the contract `ModelProvider` over the federated §4.4 provider catalog; host-side nodus bridge adapter; endpoint profiles; credential-lane + egress-gated remote path; MR-1…MR-14 compliance table (MR-3/4/5 delegated-with-disclosure in v1). |

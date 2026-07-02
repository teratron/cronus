# Cache-Stable Context

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Modern model providers bill a **byte-identical request prefix** at a steep discount
(a cache read is on the order of a tenth the price of fresh input) and charge a
one-time premium to create that cache entry. For a long-running, tool-heavy agent
session — where the same system instructions, tool schemas, and accumulated history
are resent on every turn — preserving that prefix's byte-identity across turns is the
single largest cost lever available, larger than compression. It is also fragile:
**mutating one byte of the cached prefix changes the cache key, drops the hit rate to
zero, and silently inflates the bill** while nothing visibly breaks.

This spec defines the discipline that makes cache stability a first-class,
system-wide property rather than an incidental optimization scattered across
implementations. It partitions the context into an immutable **frozen prefix** and a
mutable tail **live zone**, computes the boundary rather than guessing it, forbids
any transform from touching frozen bytes, and gates prefix-affecting actions by how
the caller is billed. It is the *where you may touch at all* concept, orthogonal to
`l1-context-compression`'s *what to shrink*.

## Related Specifications

- [l1-context-compression.md](l1-context-compression.md) — compression is a live-zone-only transform; it MUST obey CSC-2 and never mutate the frozen prefix.
- [l2-context-management.md](l2-context-management.md) — selection/summarization must not touch the frozen prefix; eviction operates below the live-zone floor only by dropping whole trailing turns, never rewriting cached ones.
- [l2-session-checkpoint.md](l2-session-checkpoint.md) — already models frozen, byte-identical system prompt blocks; this concept generalizes that to the whole prefix.
- [l1-routing.md](l1-routing.md) — cache-aware routing: keeping a session on the same upstream/credential lane keeps its cache warm (CSC-9 credential-lane discipline).
- [l1-generation-budget.md](l1-generation-budget.md) — the output-side token-economy sibling; this concept governs the input/prefix side.
- [l1-security.md](l1-security.md) — sacrosanct cryptographic/thinking fields (CSC-7) and credential-scope non-void (CSC-9).

## 1. Motivation

Three facts combine into a hard, easily-violated requirement:

1. **The prefix is the bill.** In a tool-heavy session the resent prefix (system +
   tools + all prior turns) dwarfs the new turn. Provider prompt caching makes that
   prefix nearly free to resend — *if* it is byte-identical to the previous request.
2. **Everything wants to touch the prefix.** Compression re-encodes tool outputs;
   context management trims and compacts; a "cache aligner" tries to normalize;
   telemetry rewrites; a fork re-emits tool schemas. Each is a chance to change one
   byte of the cached prefix and void the cache.
3. **The failure is silent.** A busted cache produces correct answers at 10× the
   input cost. Nothing throws; the only symptom is the bill. Without an explicit
   invariant, cache instability is invisible until an audit.

cronus already *knows* this locally — the agent-registry requires a fork's tool
schema to byte-match the parent's ("prompt-cache parity"), the session checkpoint
stores frozen system blocks, the learning loop is careful to use an auxiliary client
so it "never touches the main session's prompt cache," and the dashboard accounts
`cache_read_tokens`. But these are four disconnected precautions with no shared
contract. This spec is the contract they all instantiate: a single place that says
*the cached prefix is sacred; confine every mutation to the live zone*.

## 2. Constraints & Assumptions

- Cache stability is a token-economy property, orthogonal to compression: a transform
  may be aggressive *inside* the live zone yet must not alter one byte of the frozen
  prefix.
- The frozen boundary is a function of the request, not a guess: explicit provider
  cache markers plus structurally-always-frozen regions (system, tool schemas, prior
  turns) determine it.
- Provider cache semantics vary (marker syntax, TTL lanes, which fields are cached);
  this spec constrains the *discipline* (confinement, determinism, parity, policy),
  not any one provider's marker format.
- A conservative default is always safe: treating the whole request as frozen
  (passthrough) forgoes savings but never *costs* extra. Aggression is opt-in and
  billing-mode-gated.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **CSC-1 (Cache key is a cost lever):** preserving the byte-identity of the cached
  request prefix across turns is a first-class cost objective. Design decisions that
  trade prefix stability for marginal convenience MUST be justified against their
  cache-cost impact, not made incidentally.

- **CSC-2 (Frozen prefix / live zone partition):** every outbound request is
  partitioned into a **frozen prefix** (cached, immutable) and a tail **live zone**
  (the only region any transform may mutate). No transform — compression, trimming,
  injection, normalization, telemetry — may modify, reorder, or reserialize a byte at
  an index below the live-zone floor.

- **CSC-3 (Append-only history):** once a message has been sent upstream in a prior
  request within a session, its bytes are frozen for every subsequent request in that
  session. New content is appended; prior content is never rewritten. Eviction, when
  required, drops whole trailing turns from the tail — it never edits a cached turn.

- **CSC-4 (Byte-faithful passthrough):** content not selected for a live-zone
  mutation is forwarded byte-for-byte. A pass through the pipeline that reserializes
  JSON, reorders keys, renormalizes whitespace, or re-encodes unchanged content —
  thereby changing the cache key without changing meaning — is a defect.

- **CSC-5 (Deterministic edits):** every live-zone transform is deterministic in its
  declared inputs. No timestamps, random seeds, wall-clock, or ambient-state
  dependence. Two identical turns MUST produce byte-identical output so a repeated
  context keeps hitting the cache.

- **CSC-6 (Position preservation):** block and message order is part of the cache key.
  Transforms never reorder blocks or messages, even within the live zone, when doing
  so would change a cached boundary.

- **CSC-7 (Sacrosanct fields):** cryptographic signatures, encrypted or redacted
  reasoning/thinking payloads, and provider integrity tokens are never altered,
  truncated, compressed, or dropped, regardless of size. They are both correctness-
  critical and prefix-load-bearing.

- **CSC-8 (Schema parity):** tool/function schemas and system scaffolding are
  normalized to a single stable byte form and kept identical across related requests
  that should share a cache — most notably a parent session and a forked sub-agent.
  A fork that re-emits a semantically-equal but byte-different tool schema forfeits
  the shared cached prefix; parity is required.

- **CSC-9 (Billing-mode-gated policy):** the aggressiveness of any prefix-affecting
  action is gated by how the caller is billed. Under metered pay-as-you-go, aggressive
  optimization (including auto-inserting cache markers) is permitted. Under
  subscription or borrowed-session-credential modes, the system is conservative: it
  never auto-inserts markers that could void the credential's scope, never
  destabilizes an existing cache, and prefers passthrough. Lane selection itself must
  not leak or invalidate the chosen credential's scope.

- **CSC-10 (Observe-and-confine, never rewrite-to-align):** machinery that *mutates*
  the prefix in an attempt to "align" or "stabilize" the cache is forbidden — it
  destabilizes the very cache it targets (a per-instance realignment that rewrites the
  prefix changes the key every time). Stability is achieved by *confining* mutation to
  the live zone and *observing* the frozen boundary, never by rewriting frozen bytes.

- **CSC-11 (Cache-cost observability):** cache-read, cache-creation, fresh-input, and
  output tokens are accounted separately, so the saving of a hit — and the cost of a
  miss or a self-inflicted bust — is attributable per session and tunable. A cache
  regression MUST be detectable from telemetry, not only from the invoice.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The Frozen Prefix and the Live Zone

```text
[REFERENCE]
request = [ system ][ tools ][ turn_1 ][ turn_2 ] … [ turn_{n-1} ][ turn_n ]
          └──────────────── frozen prefix ───────────────┘└─ live zone ─┘
                    (cached, byte-identical)               (mutable tail)

- frozen prefix : system + tool schemas + every turn already sent upstream (CSC-2/3)
- live zone     : the latest turn the model will respond against — the ONLY mutable region
```

Compression, injection, and trimming operate exclusively inside the live zone. The
frozen prefix is forwarded byte-for-byte (CSC-4).

### 4.2 Computing the Frozen Boundary

The floor is computed, not assumed (CSC-2):

```text
[REFERENCE]
frozen_floor(request):
    n := 0
    for i, msg in enumerate(request.messages):
        if msg has an explicit provider cache marker:
            n := max(n, i + 1)            // marker is inclusive: msg[i] is cached
    // system + tool schemas are unconditionally frozen regardless of markers
    return n                              // messages[i < n] are immutable

live_zone(request) := request.messages[frozen_floor(request):]   // mutable tail only
```

Detection walks the parsed request structure (no textual pattern-matching over
serialized bytes, which is both fragile and a reserialization risk). When markers are
absent, the safe floor is "everything except the latest turn."

### 4.3 Sacrosanct Fields and Schema Parity

Two carve-outs are absolute (CSC-7, CSC-8):

- **Sacrosanct fields** — signatures, encrypted/redacted thinking payloads, integrity
  tokens — are copied verbatim; no transform inspects or rewrites them.
- **Schema parity** — tool/function schemas are serialized through one canonical
  normalizer so that a parent and its fork emit byte-identical schemas and share the
  cached prefix. Normalization is *stabilization* (a fixed key order, fixed
  whitespace), never *compression* of the schema.

### 4.4 Billing-Mode Policy Matrix

Aggressiveness is a function of the credential's billing mode (CSC-9):

| Action | Metered (pay-as-you-go) | Subscription / borrowed session |
| --- | --- | --- |
| Live-zone compression | Aggressive | Lossless-only / conservative |
| Auto-insert cache markers | Yes | **No** (could void scope) |
| Rewrite/normalize prefix | Only via stable normalizer | Passthrough-prefer |
| Inject provider-specific headers | Allowed | **No** (scope-safety / stealth) |
| Destabilize an existing cache | Never | Never |

The conservative column is always a safe subset of the aggressive one: it forgoes
savings, never incurs extra cost, and never risks the credential's scope.

### 4.5 The Anti-Pattern: Rewrite-to-Align

A tempting-but-wrong design tries to *improve* cache hits by rewriting the prefix into
a "canonical aligned" form each turn (CSC-10). Because the aligner's output depends on
per-instance state, it changes the prefix — and thus the key — on every request,
destroying the cache it meant to help. The correct posture is inverted: never touch
the prefix; compute the boundary; confine all edits to the live zone; and *measure*
the hit rate (CSC-11) rather than *engineer* it by rewriting.

### 4.6 Cache-Cost Accounting

Every response's usage is decomposed (CSC-11): `cache_read` (≈0.1× input),
`cache_creation` (one-time premium), fresh `input`, and `output`. A rising
`cache_creation`-to-`cache_read` ratio for a stable session is the signature of a
self-inflicted cache bust and is surfaced as an operational-health signal, not left
for the invoice to reveal.

## 5. Implementation Notes

1. Make the frozen boundary a single computed value threaded to every transform; a
   transform that receives it cannot accidentally reach below it (enforce at the type
   or assertion level).
2. Default to passthrough. Turn on aggression per billing mode, never globally.
3. Put a byte-equality gate in tests: "system + tools + prior turns are byte-identical
   at the upstream boundary before and after the pipeline" catches every CSC-2/CSC-4
   regression cheaply.
4. Route a session stickily to the same upstream/credential lane so its cache stays
   warm (composes with `l1-routing`).

## 6. Drawbacks & Alternatives

- **Forgone savings under conservatism:** subscription-mode passthrough leaves tokens
  on the table. Accepted: the alternative (aggressive optimization that busts a
  subscription cache or voids a scope) is strictly worse, and the metered path still
  captures the savings.
- **Coupling to provider cache semantics:** the boundary computation must track each
  provider's marker/cache rules. Mitigated by keeping the *discipline* provider-
  agnostic and isolating provider specifics to the boundary detector.
- **Alternative — fold into `l1-context-compression`:** rejected; cache stability
  constrains *every* prefix-touching subsystem (compression, trimming, telemetry,
  forking, routing), not just compression. It is a shared contract, not one stage's
  private rule — the same reason event routing became `l1-event-mesh` rather than an
  inbox detail.
- **Alternative — treat it as an L2 proxy detail:** rejected; the scattered L2
  precautions (schema parity, frozen blocks, auxiliary-client isolation, cache
  accounting) prove it is cross-cutting and deserves one L1 they all cite.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CTX-MGMT]` | `.design/main/specifications/l2-context-management.md` | Selection/summarization cascade that must operate below the live-zone floor only. |
| `[COMPRESS]` | `.design/main/specifications/l1-context-compression.md` | Live-zone-only compression transform bound by CSC-2. |
| `[CHECKPOINT]` | `.design/main/specifications/l2-session-checkpoint.md` | Frozen byte-identical system blocks this concept generalizes. |
| `[ROUTER]` | `.design/main/specifications/l2-model-router.md` | Cache-aware sticky routing that keeps a session's prefix warm. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-02 | Core Team | Initial spec — cache-stable context discipline elevating scattered L2 prompt-cache precautions into one L1 concept: cache key as a first-class cost lever (CSC-1), frozen-prefix/live-zone partition with a computed boundary (CSC-2), append-only history (CSC-3), byte-faithful passthrough (CSC-4), deterministic edits (CSC-5), position preservation (CSC-6), sacrosanct crypto/thinking fields (CSC-7), schema parity across forks (CSC-8), billing-mode-gated aggressiveness (CSC-9), observe-and-confine never rewrite-to-align anti-pattern (CSC-10), separated cache-cost accounting (CSC-11). Orthogonal to l1-context-compression (what to shrink) — this governs where mutation is allowed at all. Unifies l2-agent-registry prompt-cache parity, l2-session-checkpoint frozen blocks, l2-learning-loop cache isolation, l2-dashboard cache_read accounting. |

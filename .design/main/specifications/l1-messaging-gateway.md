# Messaging Gateway

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

A single gateway that makes the agent reachable from external messaging platforms — chat apps, email, and similar — through pluggable per-platform adapters that share one contract. The user talks to the agent wherever they already are; the agent stays platform-agnostic. One conversation follows the person across platforms rather than fragmenting per app. This is the outward-facing counterpart to the internal inbox (actor-to-actor messaging within a workspace): the gateway bridges *outside* channels to the agent, the inbox carries messages *between* actors inside it.

## Related Specifications

- [l1-routing.md](l1-routing.md) - Inbound messages are routed to the right office/session by the routing model.
- [l1-acp.md](l1-acp.md) - The gateway speaks to the agent over the agent protocol (sessions, streaming, interrupt).
- [l1-voice-input.md](l1-voice-input.md) - Voice memos from a platform are transcribed (on-device where possible) before reaching the agent.
- [l1-security.md](l1-security.md) - Per-sender authorization, loopback-default exposure, and message-content privacy.
- [l1-navigation-model.md](l1-navigation-model.md) - The Channels/Notifications surface that mirrors gateway activity in the UI.
- [l1-user-model.md](l1-user-model.md) - Cross-platform continuity is keyed to the principal the user model also describes.

## 1. Motivation

A personal agent that only lives in one terminal is tethered to one machine. The leverage of an always-available agent comes from reaching it where the user already is — a phone chat app on the move, email for long-form, the desktop for deep work — while it runs on a server that costs almost nothing when idle. The naive approach bolts each platform onto the agent directly, duplicating message handling and letting each platform's quirks (reply threading, message-length units, media formats) leak into the core.

A gateway inverts that: each platform is an adapter behind a uniform contract, the agent sees one normalized message stream regardless of source, and a conversation is bound to the *person* so they can start on one platform and continue on another. The hard parts are identity (which real principal is this sender?), continuity (same conversation across apps), authorization (reachability is not permission), and faithful delivery within each platform's constraints — which is exactly what this concept governs.

## 2. Constraints & Assumptions

- The agent core is platform-agnostic; all platform-specific behavior lives in adapters behind the common contract.
- Reachability over a platform does NOT imply authorization — an unpaired sender is untrusted by default.
- The gateway transports and routes; it does not itself reason. Agent reasoning happens over the agent protocol.
- Platforms differ in capabilities (threads, media, length limits, voice); adapters declare what they support and the contract degrades gracefully.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **MG-1 (One gateway, many adapters):** a single gateway bridges the agent to multiple external platforms via pluggable adapters that share one contract. Adding a platform means adding an adapter; it MUST NOT require changing the agent core.
- **MG-2 (Normalized message contract):** every adapter maps its platform to a uniform contract — inbound events (text, voice, media, reply context, sender identity) become normalized agent input, and agent output is delivered platform-natively, honoring that platform's constraints (length units, reply/thread semantics, media/audio handling). The agent never sees platform-specific shapes.
- **MG-3 (Identity pairing):** an external platform account is bound to an internal principal through an explicit, verifiable pairing step before any privileged interaction. Unpaired senders are untrusted. One principal MAY pair several platform accounts.
- **MG-4 (Cross-platform continuity):** a conversation is keyed to the principal, not to the platform; a paired user MAY continue the same conversation and context from a different platform. Continuity is the default; per-platform isolation is an explicit opt-in.
- **MG-5 (Authorization by resolved principal):** authorization (who may interact, at what trust level, with what capabilities) is enforced against the *resolved principal*, never inferred from mere platform reachability. Each inbound message carries its platform and sender identity for this resolution.
- **MG-6 (Faithful, ordered delivery):** outbound delivery is ordered per conversation and reliable — a failed send is retried or surfaced, never silently dropped. Long output is chunked to platform limits without corrupting content (encoding-safe splitting), and media is sent in the platform's appropriate form.
- **MG-7 (Exposure safety):** the gateway's control surface binds to loopback by default; any remote exposure is explicit, authenticated, and scoped (consistent with no-exfiltration and the loopback-bound control-plane discipline used elsewhere).
- **MG-8 (Transit privacy):** message content from external platforms is user data — never logged in the clear, never egressed beyond the platform's own delivery path; voice is transcribed on-device where the platform and host allow (consistent with voice-input).
- **MG-9 (Per-platform fault isolation):** a single adapter's failure is isolated to that platform — the gateway and other adapters keep running, and the failure is surfaced and auditable rather than taking down all channels.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 Adapter Contract

```text
[REFERENCE]
Adapter (per platform):
  capabilities() -> { threads, media, voice, max_len_unit, reply_semantics }
  inbound(event) -> NormalizedMessage { principal?, text, attachments[], reply_to?, platform, sender }
  outbound(NormalizedReply) -> platform-native send   // chunk to max_len, pick media/audio form
```

Adapters declare capabilities so the contract can degrade (e.g. no native threads → flat replies; no voice → request text). The core composes only against `NormalizedMessage` / `NormalizedReply`.

### 4.2 Pairing & Principal Resolution

```text
[REFERENCE]
inbound(event):
    principal := resolve(platform, sender)        // MG-3
    if principal is unpaired:
        offer_pairing(verifiable challenge)        // bind account → principal, or stay untrusted
        return                                     // no privileged action before pairing
    authorize(principal, requested_capability)     // MG-5
    route_to_session(principal, event)             // continuity by principal (MG-4)
```

### 4.3 Continuity & Delivery

A conversation key is `(principal, conversation)`, independent of platform, so a user paired on two apps shares one thread (MG-4). Outbound replies are ordered per conversation key and chunked encoding-safely to the originating (or user-chosen) platform's limits (MG-6).

### 4.4 Fault & Exposure

Each adapter runs isolated; a crash or auth failure marks that platform degraded (surfaced to operational health) without affecting others (MG-9). The control/API surface is loopback-bound unless an explicit, authenticated remote-exposure configuration is set (MG-7).

## 5. Drawbacks & Alternatives

- **Adapter maintenance burden:** each platform's quirks and API churn cost upkeep; justified because MG-1/MG-2 contain that churn in the adapter, never the core.
- **Continuity vs context bleed:** cross-platform continuity (MG-4) can surface a work conversation in a casual app; mitigated by per-platform isolation opt-in and authorization scoping (MG-5).
- **Alternative — one platform only (terminal):** simplest, but forfeits the always-available leverage of a server-resident agent reachable from anywhere.
- **Alternative — direct per-platform wiring into the agent:** rejected; leaks platform specifics into the core and duplicates auth/continuity logic per platform.

## Canonical References

| Alias | Path | Purpose |
|---|---|---|
| `[ROUTING]` | `.design/main/specifications/l1-routing.md` | Routes resolved inbound messages to the correct office/session. |
| `[ACP]` | `.design/main/specifications/l1-acp.md` | Agent protocol the gateway drives (sessions, streaming, interrupt). |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | Authorization, loopback-default exposure, and transit privacy. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-26 | Core Team | Initial spec — multi-platform messaging gateway: one gateway, pluggable per-platform adapters on a normalized contract, identity pairing, cross-platform continuity by principal, authorization by resolved principal, faithful ordered delivery, exposure safety, transit privacy, per-platform fault isolation (MG-1…MG-9). |

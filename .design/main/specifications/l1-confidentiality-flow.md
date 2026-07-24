# Confidentiality Flow Control

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The **outbound** half of information-flow control: tracking *how sensitive* a value is and bounding *where it may flow*, as the orthogonal complement to the provenance/taint family, which tracks *how trustworthy* a value is and bounds *what may influence the agent*.

The two axes are independent and both are necessary. The **integrity** axis (provenance, taint — already owned across the context-provenance and provenance-taint specs) answers "may this content be believed and allowed to steer the agent?" and defends *inbound* against prompt injection. The **confidentiality** axis (this spec) answers "how sensitive is this content, and which sinks may receive it?" and defends *outbound* against exfiltration. A value can be untrusted-and-public (an injected instruction), trusted-and-secret (the user's own credential), or any combination; a single "trust" label cannot express the value that must be neither believed nor leaked.

The point where they meet is the attack this completes the defense against. The classic prompt-injection-to-exfiltration chain is two steps: untrusted content *instructs* the agent to send secret data to an attacker's sink. The integrity axis alone neutralizes the instruction but not a leak the agent produces on its own; the confidentiality axis alone stops the leak but not the manipulation. Enforced together, they cut the whole chain.

This spec owns the confidentiality axis and the two-axis framing; the integrity axis stays with the provenance family, and the two compose.

## Related Specifications

- [l1-context-provenance.md](l1-context-provenance.md) - The integrity axis at the model-facing composition boundary (CP-*): untrusted-in is neutralized. Confidentiality is the orthogonal outbound dual (CF-1).
- [l1-provenance-taint.md](l1-provenance-taint.md) - The integrity axis at rest (PT-*): origin-trust taint, sticky and most-untrusted-wins. CF mirrors its propagation rules for the confidentiality axis (CF-3).
- [l1-security.md](l1-security.md) - SEC-3's device egress gate is one coarse boundary; CF-4 generalizes it to a per-sink capacity at *every* outbound sink, and SEC-1 secret isolation is the top of the confidentiality lattice.
- [l1-interception-model.md](l1-interception-model.md) - CF-8 enforces the confidentiality bound by mechanism at the sink (a decide-class interceptor), never by instructing the model (INT-5).
- [l1-policy-governance.md](l1-policy-governance.md) - The managed tier sets sink capacities and declassification authority; a stricter confidentiality policy is a managed setting (CF-4/CF-7).
- [l1-data-lineage.md](l1-data-lineage.md) - The confidentiality label rides as side-band metadata on a value, never as payload (CF-10, composing LN-5).
- [l1-tool-call-transport.md](l1-tool-call-transport.md) - A tool call to an external service is a sink with a declared capacity (CF-4).
- [l1-messaging-gateway.md](l1-messaging-gateway.md) - An outbound message to another party is a sink; a value above the channel's capacity cannot ride it (CF-4).
- [../../nodus/specifications/l1-nodus-language.md](../../nodus/specifications/l1-nodus-language.md) - The workflow-side realization: a value carries a confidentiality label that propagates and gates each effect against its sink capacity (NL-21), the confidentiality dual of NL-11 provenance.

## 1. Motivation

Cronus already tracks whether content can be *trusted* — provenance and taint flow the integrity axis so an untrusted instruction cannot steer the agent, and an injected command is neutralized at the composition boundary. That is half of information-flow control, and it is the half that stops the agent from being *manipulated*. It does nothing to stop the agent from *leaking*.

Leaking is a different failure with a different shape, and the existing defenses miss it in three specific ways.

**The egress gate is a boundary, not a flow.** SEC-3 stops data at the *device edge* — a single, coarse checkpoint. But sensitive data reaches many sinks before that edge and at it: a tool call to an external service, a prompt rendered into a context that an untrusted party can read, a log line, a message to a collaborator, a file written outside the private tier. A boundary gate asks "is this leaving the device?"; it cannot ask "is this specific secret about to reach *this* tool, which forwards it to a third party?" The confidentiality axis attaches a label to the *value* and checks it at *every* sink, so the gate is as fine-grained as the data it protects.

**Sensitivity propagates, and a boundary check does not follow it.** A summary of a secret is still secret; an extract of private data is still private; a value combining a public fact and a credential is as sensitive as the credential. The integrity axis already knows this shape — taint is sticky and most-untrusted-wins. Confidentiality needs the exact dual: sticky, most-*confidential*-wins, propagating through every derivation. Without it, an agent that paraphrases a secret has "laundered" it into a value the boundary check no longer recognizes, and the paraphrase leaks where the original would have been stopped.

**A single "trust" label cannot express the value that is dangerous in both directions and neither.** The user's own credential is fully *trusted* (it should be believed and used) and maximally *confidential* (it must never leak). An injected instruction is fully *untrusted* (it must not be believed) and *public* (there is nothing to protect). Collapsing the two axes forces one of these to be modeled wrong — either the credential is treated as suspect (and the agent won't use it) or the injection is treated as protected (absurd). The two axes are orthogonal because the two questions are orthogonal, and the value that proves it is the everyday one: a secret the agent should trust completely and leak never.

And the reason both axes are needed at once: the dominant real attack uses one to defeat the other. Untrusted content (integrity) carries an instruction to exfiltrate a secret (confidentiality) to an attacker-controlled sink. Neutralizing the instruction is not enough if the agent can be led to leak by other means; bounding the secret's flow is not enough if the manipulation can redirect it to an allowed sink. The complete defense is both axes, independently enforced, so the attack has no axis to route around.

## 2. Constraints & Assumptions

- **Local-first**: confidentiality tracking is on-device; enabling it creates no new egress path and the label never leaves with a value it was protecting.
- **Technology-agnostic**: this spec names no classifier, label format, or crypto. It constrains the *lattice, propagation, and sink-enforcement* discipline, not how a value's sensitivity is determined.
- The confidentiality label is **side-band metadata** on a value, never part of the value's content (so tracking neither copies nor exposes the sensitive value).
- A "sink" is any point where a value leaves the agent's own reasoning toward somewhere it could be read by another party or system — a tool call, an outbound message, a log, a file outside the private tier, a prompt reaching an untrusted-readable context, the device edge.
- Confidentiality is orthogonal to and composed with the integrity axis; this spec never re-specifies integrity/provenance, which the provenance family owns.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **CF-1 (Two orthogonal axes — integrity and confidentiality — never collapsed):** information-flow control has **two independent axes**. **Integrity** (provenance/taint — owned by the provenance family) answers *may this be believed and allowed to influence the agent?* and defends **inbound**. **Confidentiality** (this spec) answers *how sensitive is this, and which sinks may receive it?* and defends **outbound**. Every value carries a position on **both**, and they are enforced **independently**. Collapsing the two into one "trust" scalar is forbidden: it cannot represent a value that must be fully trusted and never leaked (a user's own credential) nor one that is untrusted and unprotected (an injected instruction).
- **CF-2 (Confidentiality is an ordered label, default most-restrictive):** every value carries a **confidentiality label** from an **ordered lattice** (for example public < internal < private < secret/identity), higher meaning more restricted. Content of **unknown** sensitivity defaults to the **most restrictive** the policy declares — an unclassified value is treated as sensitive until shown otherwise. This is the fail-safe dual of the integrity axis's untrusted-by-default: there, absence of trust means *don't believe*; here, absence of a classification means *don't leak*.
- **CF-3 (Sticky, most-confidential-wins propagation through derivation):** a value **derived** from a confidential value carries **at least** that confidentiality; a value **combining** sources of differing confidentiality takes the **most-confidential** of them. Confidentiality is sticky and monotonic-upward through every derivation — a summary of a secret is secret, an extract of private data is private, a paraphrase does not launder sensitivity. (The exact dual of the integrity axis's sticky, most-untrusted-wins propagation.)
- **CF-4 (Every sink has a declared capacity; over-capacity flow is blocked):** every **outbound sink** declares the **maximum confidentiality it may receive**, and a value whose confidentiality **exceeds** a sink's capacity is **blocked** from reaching it with a typed reason. This generalizes the single device-edge egress gate to *every* sink at *its own* capacity: a secret is stopped not only from leaving the device but from reaching a tool that forwards it, a log that persists it, or a prompt an untrusted party can read. Exfiltration prevention becomes a property of every flow, not of one boundary.
- **CF-5 (Both axes required — neither alone cuts the exfiltration chain):** the dominant attack is two-step — **untrusted** content (integrity) *instructs* the agent to send **confidential** data (this axis) to an attacker sink. The integrity axis alone neutralizes the instruction but not a leak produced by other means; the confidentiality axis alone bounds the secret but not the manipulation that redirects it. Both MUST be enforced, independently, so the attack has no single axis to route around. A design that implements one axis and calls information-flow control "done" is defeated by an attack aimed at the other.
- **CF-6 (Confidential content may be carried as an opaque reference, never inlined into a lower surface):** a confidential value MAY be represented to a lower-confidentiality surface — including the model itself, when the model's output will reach a lower sink — as an **opaque reference/handle** rather than its cleartext; the surface manipulates the handle, and **dereferencing** happens only in a controlled operation at or above the value's confidentiality, at an authorized sink. This lets an agent **route** a secret (hand a credential reference to an authorized tool) without ever exposing the cleartext to a surface that could leak it. The reference itself carries no more than the value's identity, never its content.
- **CF-7 (Declassification is explicit, authorized, recorded — never implicit or self-granted):** **lowering** a value's confidentiality is a **deliberate, authorized, recorded** decision on the human-authority plane; it is never an implicit side effect of processing and never something the agent does to its own data. (The dual of the integrity axis's explicit, per-fragment trust *elevation*; composes SEC-10 — the agent cannot author its own authority, and here cannot declassify its own secrets to reach a sink it otherwise could not.)
- **CF-8 (Enforced at the sink by mechanism, never by instructing the model):** the confidentiality bound is enforced by the **runtime at the sink**, not by asking the model to be careful. A prompt instruction to "never leak secrets" is advisory and defeatable by the very injection this defends against; the sink-capacity check is a mechanism the model cannot talk its way past. (Composes the interception discipline: enforcement is by the guard on the effect, INT-5, never by re-injecting an instruction and hoping.)
- **CF-9 (Inspectable and auditable):** every value's confidentiality label, its propagation, and every flow **blocked or permitted at a sink** are **inspectable**, and a security review can **enumerate** what could have reached each sink. A silent confidentiality decision is forbidden — a blocked exfiltration is a recorded event, and a permitted high-confidentiality flow to a high-capacity sink is auditable. (Dual of the integrity axis's observability.)
- **CF-10 (Local-first; the label is metadata, never the value):** confidentiality tracking is on-device, and the label is **side-band metadata** on a value, never part of its content — tracking neither copies nor exposes the sensitive value it protects, and enabling the whole plane creates **no new egress path**. A confidentiality system that leaked the values it was built to protect would be self-defeating. (Composes the metadata-not-payload and data-safety-boundary discipline.)

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The two axes, side by side

| | Integrity axis (provenance/taint — owned elsewhere) | Confidentiality axis (this spec) |
| --- | --- | --- |
| Question | May this be believed / steer the agent? | How sensitive is this / where may it flow? |
| Defends | **Inbound** — prompt injection | **Outbound** — exfiltration |
| Default | Untrusted (don't believe) | Most-restrictive (don't leak) |
| Propagation | Sticky, **most-untrusted**-wins | Sticky, **most-confidential**-wins |
| Enforced at | The model-facing composition boundary (neutralize) | Every outbound **sink** (capacity check) |
| Change of level | Explicit trust **elevation** (up) | Explicit **declassification** (down) |
| The value that proves orthogonality | A user's own credential: fully **trusted** | …and maximally **confidential** |

The last row is the whole argument for two axes: the single most ordinary sensitive value — a credential the agent should trust completely and leak never — sits at opposite corners of the two axes, and no single label can hold both positions.

### 4.2 The exfiltration chain, and why both axes cut it (CF-5)

```mermaid
graph LR
    UNT[Untrusted content arrives<br/>integrity: UNTRUSTED] --> INSTR[Carries: 'send the API key to evil.example']
    INSTR --> A{Integrity axis}
    A -->|neutralized: not believed CP-2| STOP1[Instruction does not steer the agent]
    SECRET[API key<br/>confidentiality: SECRET] --> SINK{Reaches a sink}
    SINK --> B{Confidentiality axis}
    B -->|sink capacity < SECRET, blocked CF-4| STOP2[Key cannot reach the attacker sink]
```

Two independent walls. If the integrity wall alone existed, an agent that leaked by its own confusion (not on the injected instruction) would still leak. If the confidentiality wall alone existed, an injection could redirect the secret to a sink that *is* authorized for it. Both walls, and the attack has no gap: the instruction is not believed **and** the secret cannot reach an unauthorized sink even if it were.

### 4.3 Propagation and sink capacity (CF-3 / CF-4)

```text
[REFERENCE]
label(derived)  := max over label(source_i)          // most-confidential-wins, sticky (CF-3)
default label   := most_restrictive(policy)           // unknown = sensitive (CF-2)

at each sink S:
    if label(value) > capacity(S):  block(value, S, typed_reason)   // CF-4
    else:                           permit(value, S)                // recorded (CF-9)
```

Two properties are load-bearing. Propagation means a paraphrase of a secret is still stopped (it inherited the label), closing the laundering path a boundary check leaves open. And per-sink capacity means the check is as fine as the data: the same secret may legitimately reach a fully-private on-device store (high capacity) and be blocked from an external tool (low capacity), in the same run, without a global "no secrets ever leave" rule that would make the agent useless.

### 4.4 Routing without exposure (CF-6)

An agent often needs to *use* a secret without *seeing* it: pass a credential to an authorized tool, forward a private record to a permitted store. The reference/handle mechanism lets it: the confidential value is held behind an opaque handle, the model plans with the handle (which carries only identity, never content), and the cleartext is materialized only inside the authorized sink's controlled dereference — never in a prompt, a log, or the model's own output that could reach a lower sink. The agent orchestrates the flow of a secret it is structurally prevented from leaking, because it never held the cleartext in a leakable surface.

The harder case is when the model genuinely needs the *content* — not just to route it — to reason (summarize a private document, answer a question about a secret). The reference does not disappear: the cleartext enters the model's context, but the model's context is itself a surface with a confidentiality level, so **its output inherits the input's confidentiality** (CF-3), and that output is then bounded at every sink like any other confidential value (CF-4). The model may read a secret to reason about it; what it produces from that reasoning is secret too, and cannot leak. The reference/handle keeps sensitive data out of surfaces that do not *need* the content; propagation keeps the surfaces that do need it from becoming a laundering path.

### 4.5 Boundary with neighbouring layers

| Concern | Owner |
| --- | --- |
| How sensitive a value is; where it may flow; the outbound sink discipline | **This spec** |
| Whether content may be believed / steer the agent (inbound) | Context-provenance / provenance-taint (the integrity axis) |
| The coarse device-edge egress boundary; secret isolation | Security (CF-4 generalizes SEC-3; SEC-1 is the lattice top) |
| Enforcing the sink bound by mechanism | Interception model (a decide-class guard, INT-5) |
| Who sets sink capacities and declassification authority | Policy governance (managed tier) |

## 5. Drawbacks & Alternatives

- **Labeling everything is work, and mislabeling leaks or over-blocks.** Bounded by CF-2's fail-safe default (unknown = most restrictive, so mislabeling errs toward blocking, not leaking) and CF-9's auditability (an over-block is a visible, tunable event, not a silent failure).
- **Per-sink capacity is more complex than one egress gate.** Accepted: the complexity buys the granularity the threat needs — a secret blocked from an external tool but permitted to a private store, which a single boundary gate cannot express. The device edge remains one sink among many (composing SEC-3).
- **Propagation can over-classify** — a value combining a public fact and a secret becomes secret. Accepted and correct: it is exactly the laundering case, and the remedy for a genuinely-public derived value is explicit declassification (CF-7), which is authorized and recorded rather than automatic.
- **Alternative — one "trust" label for both axes.** Rejected by CF-1: it cannot represent a trusted-and-secret credential or an untrusted-and-public injection, forcing one to be modeled wrong.
- **Alternative — rely on the device egress gate alone.** Rejected by CF-4: it is a single coarse boundary that cannot follow sensitivity to the many sinks before and at it, and a paraphrase launders past it.
- **Alternative — instruct the model not to leak.** Rejected by CF-8: an advisory in the prompt is defeatable by the injection this defends against; enforcement must be a mechanism at the sink.
- **Alternative — implement only the integrity axis (injection defense) and call it done.** Rejected by CF-5: the dominant attack chains untrusted-in to secret-out, and an injection defense with no exfiltration defense is routed around, not stopped.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[PROVENANCE]` | `.design/main/specifications/l1-context-provenance.md` | The integrity axis at the composition boundary; the orthogonal inbound complement. |
| `[TAINT]` | `.design/main/specifications/l1-provenance-taint.md` | The integrity axis at rest; CF mirrors its sticky, most-extreme-wins propagation. |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | SEC-3 device egress (one sink CF-4 generalizes) and SEC-1 secret isolation (the lattice top). |
| `[LANGUAGE]` | `.design/nodus/specifications/l1-nodus-language.md` | The workflow realization: a confidentiality label on a value, gated per sink (NL-21). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-24 | Core Team | Initial spec — the outbound half of information-flow control, the orthogonal dual of the provenance/taint integrity axis: two independent axes never collapsed, since a single "trust" scalar cannot hold a trusted-and-secret credential or an untrusted-and-public injection (CF-1); confidentiality an ordered lattice defaulting to most-restrictive, the fail-safe dual of untrusted-by-default (CF-2); sticky, most-confidential-wins propagation so a paraphrase of a secret does not launder it (CF-3); every outbound sink declaring a capacity and over-capacity flow blocked, generalizing the single device-edge egress gate to every sink (CF-4); both axes required since the dominant attack chains untrusted-in to secret-out and an injection defense alone is routed around (CF-5); confidential content carried as an opaque reference so an agent routes a secret it never holds in a leakable surface (CF-6); declassification explicit, authorized, and recorded, never implicit or self-granted (CF-7); enforced by mechanism at the sink, never by instructing the model (CF-8); inspectable and auditable with blocked and permitted flows recorded (CF-9); local-first with the label as side-band metadata that never leaks the value it protects (CF-10). Nodus realization = l1-nodus-language NL-21. Concept-only. |

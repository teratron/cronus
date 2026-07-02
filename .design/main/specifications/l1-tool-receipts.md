# Tool Receipts

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Tool receipts make a tool call **unforgeable by the model that requested it**: every
effectful action the agent takes produces a small, keyed proof — a receipt — that
binds the action's identity, inputs, result, and time, and is fed back to the model
as part of the tool result. The model can *read* and *echo* receipts, but it can
never *mint* a new valid one, because the signing secret never enters its context.

This closes a gap no other trust layer covers. A language model is a string
generator: nothing in the model itself prevents it from narrating an action it never
took ("I ran the deploy and it succeeded") or fabricating a result for an action that
actually failed. For an autonomous agent this is not merely a correctness problem —
it is a **deniability and hallucination-of-action problem**. Receipts give the runtime
(and, optionally, the human) a cheap, verifiable way to tell a genuinely-executed
action from a fabricated narration, and a real result from an invented one.

Receipts are the **execution-authenticity** member of the integrity family:
`l1-claim-verification` checks whether *output claims* are grounded in sources;
`l1-attestation` proves *artifact* integrity and authorship to third parties offline;
this concept proves that a *tool action* happened and that its *result is real* — an
in-process, runtime-verified guarantee against model fabrication.

## Related Specifications

- [l1-claim-verification.md](l1-claim-verification.md) — the output-side counterpart (are claims grounded in sources?); receipts are the execution-side counterpart (did the action happen, is the result real?). Complementary members of the anti-fabrication family.
- [l1-attestation.md](l1-attestation.md) — artifact/supply-chain integrity: asymmetric, offline, third-party-verifiable. Receipts are the deliberate opposite flavour: symmetric, in-process, runtime-only, per-action — distinct trust model, not a substitute.
- [l1-security.md](l1-security.md) — SEC-1 (the receipt secret is isolated like any secret), SEC-7 (receipts strengthen the audit trail from logged to provable); SEC-10 (receipts do not grant authority — they only witness action).
- [l1-execution-graph.md](l1-execution-graph.md) — EG-12 deferred/backgrounded actions define the honest coverage boundary (TR-8): a detached action surfaces its receipt on resumption, not before.
- [l1-operational-ledger.md](l1-operational-ledger.md) — data provenance; receipts add verifiable *execution* provenance beside it.
- [l1-nodus-observability.md](../../nodus/specifications/l1-nodus-observability.md) — the nodus realization (HO-9 execution-authenticity receipt on the trace).

## 1. Motivation

An agent with autonomy acts in the world — it runs commands, calls APIs, writes
files, moves money. The record of what it did is only as trustworthy as the layer
that produces it. Two failure modes are specific to LLM-driven agents and invisible
to ordinary logging:

- **Fabricated action.** The model emits text asserting it performed an action it
  never requested. Downstream readers (the user, a summary, a follow-up step) treat
  the assertion as fact. Nothing flags it.
- **Fabricated result.** The model requested a real action, the action failed or
  returned something inconvenient, and the model narrates a different, more plausible
  result. The true result is buried; the invented one propagates.

Ordinary audit logs do not help here, because the log and the model's *text* are two
different channels: the log may be correct while the model's user-facing narration
lies. The fix is to bind an unforgeable token to the actual execution and put it
where the model can see it but cannot reproduce it — so a claimed action without a
valid receipt, or a result whose receipt does not verify, is *detectably* fabricated.
The construct is deliberately cheap (a keyed MAC, negligible overhead) rather than a
heavyweight proof system, because it must run on every action without friction.

## 2. Constraints & Assumptions

- A receipt proves *authenticity of execution*, not *authorisation*. Whether an action
  should have run is the approval/authority layer's job (`l1-security` SEC-9/SEC-10,
  `l2-agent-autonomy`); receipts sit downstream of that decision.
- The verifier is the trusted runtime, which holds the signing secret. Receipts are
  intentionally **not** third-party or offline verifiable — that property belongs to
  `l1-attestation`, which pays for it with asymmetric keys and a heavier model.
- The reasoning model is the adversary of interest. The threat is fabrication *inside*
  the process, not a network attacker forging traffic between processes.
- The mechanism must be primitive-agnostic at this layer: the concept mandates a keyed,
  model-unforgeable proof; the concrete algorithm and encoding are an L2 choice.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **TR-1 Per-action receipt**: every effectful action invocation — allowed, auto-approved, or blocked — produces a receipt that binds the action's identity (name/kind), its inputs, its observed result (or the block reason), and a timestamp. No effectful action completes without one.
- **TR-2 Model-unforgeable**: the receipt is produced with a secret the reasoning model never sees. The model MAY echo receipts already present in its context, but it cannot produce a *new* valid receipt for an action that did not occur. A fabricated receipt string fails verification.
- **TR-3 Result authenticity**: the receipt covers the *actual observed result*. A result that has been altered or invented does not match its receipt — the model cannot substitute a fabricated result for a real call without detection.
- **TR-4 Existence authenticity (absence is signal)**: a narrated action with no matching, verifying receipt is treated as *fabricated*, not assumed true; a genuinely-executed action always carries a receipt. The runtime never silently upgrades an unreceipted claim to fact.
- **TR-5 Ephemeral, isolated secret**: the signing secret is ephemeral (bounded to a runtime session/process), held only in volatile memory, and MUST NOT be persisted, logged, placed in the model's context, or egressed (reaffirms SEC-1). Rotation on session restart is expected; a receipt need not survive across sessions.
- **TR-6 Runtime-verified, not third-party**: verification is performed by the trusted runtime holding the secret; the concept makes **no** claim of offline or third-party verifiability (its deliberate distinction from artifact attestation). Receipts carry no secret material and are safe to surface, log, and echo.
- **TR-7 Complement, never replacement**: receipts prove an action *happened* and its result is *real*; they do not decide whether it *should* have (authority/approval gate), do not force tool use, and do not judge whether output claims are grounded in sources (claim-verification). They compose with those layers and replace none.
- **TR-8 Honest coverage boundary**: receipts cover the actions the runtime actually witnesses. An action deferred or detached from the current turn (its result arriving asynchronously, `l1-execution-graph` EG-12) surfaces its receipt on resumption, not before. A surface that displays receipts MUST NOT imply coverage it does not have; coverage limits are explicit.
- **TR-9 Tamper-evident auditable record**: receipts are recorded as a verifiable per-action log that strengthens the security audit trail (SEC-7) from *logged* to *provable*. The record is append-only and observable; a mismatch on verification is itself an auditable event.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 What a receipt binds

A receipt is a keyed proof over the **whole action**, not just its name:

```text
[REFERENCE]
receipt := MAC(secret, action_kind ‖ inputs ‖ result ‖ timestamp)
surfaced as an opaque, secret-free token appended to the tool result:
        <receipt-prefix>-<time>-<digest>

verify(receipt, action_kind, inputs, result, timestamp) -> ok | mismatch
  ok       : this exact action, with this exact result, occurred
  mismatch : the action or the result was altered/fabricated
```

Because the result is inside the MAC (TR-3), changing the narrated result breaks the
receipt; because the secret is outside the model (TR-2), inventing a receipt breaks
verification. A stable, well-known token prefix lets a secret-scrubber pass receipts
through unredacted (they are safe to surface) rather than mistaking them for secrets.

### 4.2 Threat model — what receipts detect

| Model behaviour | Without receipts | With receipts |
| --- | --- | --- |
| Claims it ran an action, didn't | undetectable | no valid receipt → fabrication visible (TR-4) |
| Fabricates a result for a real call | undetectable | receipt mismatches on the result (TR-3) |
| Denies an action it did take | unverifiable | receipt in the record proves it (TR-9) |
| Invents a plausible receipt string | plausible | verification fails — no secret (TR-2) |

### 4.3 The secret

The signing secret is generated once at runtime-session start, kept only in memory,
and rotated on restart (TR-5). It is never written to disk, never logged, never sent
to the model, never egressed. Compromising durable storage therefore yields nothing:
there is no long-lived key to steal, and a captured old receipt cannot be re-verified
after rotation — acceptable, because the guarantee is scoped to *within* a live
session, which is exactly where model fabrication happens.

### 4.4 Boundaries (what receipts are not)

- **Not authorisation.** A receipt proves a call happened; the approval/authority
  gate (SEC-9/SEC-10) decides whether it may (TR-7).
- **Not grounding.** Receipts say nothing about whether a *claim* is supported by
  *sources* — that is claim-verification's job; the two compose.
- **Not third-party proofs.** A receipt is runtime-verifiable only (TR-6); when
  offline, cross-host, third-party verification is required, that is artifact
  attestation's territory, with its heavier asymmetric machinery.
- **Not total coverage.** Detached/background actions surface receipts on resumption,
  not within the originating turn (TR-8); the surface says so rather than implying
  full coverage.

## 5. Drawbacks & Alternatives

- **Symmetric, in-process only.** A receipt convinces the runtime, not an external
  auditor. This is a deliberate trade: the cheap symmetric MAC is what makes
  per-action receipts free enough to run on *everything*. When external proof is
  needed, `l1-attestation` is the right (heavier) tool; the two are complementary,
  not competitors.
- **Coverage honesty over coverage completeness.** Rather than pretend to cover
  asynchronous actions the turn cannot yet witness, TR-8 makes the boundary explicit.
  A dishonest "everything is receipted" surface would be worse than an honest partial one.
- **Alternative — trust the audit log alone.** Rejected: the log and the model's
  user-facing text are separate channels; a correct log does not stop the model from
  narrating a fabricated action or result to the user. Receipts bind proof into the
  channel the model speaks through.
- **Alternative — force every claim through claim-verification.** Rejected as a
  substitute: claim-verification checks grounding against sources, a different and more
  expensive question than "did this specific call execute and return this," which a
  keyed MAC answers in under a millisecond.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CLAIM-VERIFY]` | `.design/main/specifications/l1-claim-verification.md` | Output-side grounding counterpart; receipts are the execution-side complement |
| `[ATTESTATION]` | `.design/main/specifications/l1-attestation.md` | Artifact integrity (asymmetric, offline, third-party) — the contrasting attestation flavour |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | SEC-1 secret isolation of the receipt key; SEC-7 audit strengthened |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-02 | Core Team | Initial spec — tool-call execution authenticity via model-unforgeable per-action receipts: TR-1 per-action receipt, TR-2 model-unforgeable, TR-3 result authenticity, TR-4 existence authenticity (absence is signal), TR-5 ephemeral isolated secret, TR-6 runtime-verified not third-party, TR-7 complement not replacement, TR-8 honest coverage boundary (EG-12 deferred), TR-9 tamper-evident auditable record. The execution-authenticity sibling of l1-claim-verification (output grounding) and l1-attestation (artifact integrity); closes the LLM-as-string-generator fabricated-action/fabricated-result gap no other trust layer owned. nodus realization = l1-nodus-observability HO-9. |

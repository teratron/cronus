# Cooperative Status Projection

**Version:** 1.0.0
**Status:** RFC
**Layer:** concept

## Overview

The contract for turning **many concurrently-running, vendor-foreign coding-agent CLIs** — each with its own hook format, its own notion of a turn, its own willingness to say when it is done — into **one normalized, closed-vocabulary status feed** the host can render, route, and reason about identically regardless of which vendor produced it.

This is a continuous, multiplexed observation problem, and it is a different point in the design space from commissioning a single foreign-agent run and verifying its one result ([l1-foreign-agent-invocation.md](l1-foreign-agent-invocation.md)): here the foreign agent is the **primary, long-lived, interactively-driven work surface** — a terminal pane the user or the host is watching — and the host's job is to keep an honest, live picture of what it is doing across an entire session, not to grade one finished artifact at the end.

The central tension is that most vendors' **cooperative hooks are incomplete**: an agent that is interrupted (Ctrl+C, a dropped terminal) frequently never fires the hook that would announce it. A projection that reports only what hooks confirm silently freezes on "working" forever after every interruption; a projection that infers liveness from anything *else* — a terminal title, a heuristic on output shape — reintroduces exactly the untrustworthy signal the hook contract exists to replace. The resolution is a **declared, narrow, generation-fenced inference fallback**: guessed only where a vendor's hook contract is known to be lossy, guessed conservatively, and — critically — never allowed to survive contact with a genuinely fresher signal about the same turn.

## Related Specifications

- [l1-foreign-agent-invocation.md](l1-foreign-agent-invocation.md) — the one-shot commissioned-execution twin. FAI governs launching a foreign agent for a bounded task with a single verified outcome; this spec governs continuously projecting the live status of a foreign agent already running as an interactive peer. FAI's containment/deadline/re-derivation invariants do not apply here because nothing is being commissioned — the agent is being *watched*, not *delegated to*.
- [l1-remote-liveness-verdict.md](l1-remote-liveness-verdict.md) — RLV-4's generation-fencing (a termination signal is trusted only against the current incarnation) is the same discipline ASP-3 applies to a *status verdict* rather than an *existence verdict*; the two compose where a projected agent also runs on a remote, disconnection-prone host.
- [l1-work-liveness.md](l1-work-liveness.md) — WL-2's compare-and-clear claim release is the general form ASP-3 and ASP-4 both instantiate at a different grain (a displayed status, and a pane's addressing identity, rather than a work-unit claim).
- [l1-artifact-derived-observation.md](l1-artifact-derived-observation.md) — ADO's adapters read artifacts a tool leaves behind, out-of-band, having asked for nothing. This spec's hooks are the opposite: a live, in-band, cooperative channel the observed tool was built to emit into. Where a vendor supplies no such channel, ADO's artifact-reading approach is the fallback, not this spec's.
- [l1-agent-framework-skeleton.md](l1-agent-framework-skeleton.md) — AFS governs the design of agents *this system builds*; this spec governs observing agents this system did not build and cannot change, which is why its central problem (a lossy vendor hook contract) has no AFS analog.
- [l1-consent-binding.md](l1-consent-binding.md) — CB-3's "any change to a bound element lapses the grant" is the general shape ASP-4's authority-tiering rule realizes for pane/session addressing rather than human consent.

## 1. Motivation

Left unspecified, an implementation improvises, and each improvisation produces a status feed that looks fine until the exact moment it is trusted:

- **Universal inference.** Every vendor's silence is treated as interruptible the same way. A vendor whose hooks are actually reliable gets second-guessed by a heuristic that occasionally overrides a correct "still working" with a wrong "done."
- **Unguarded inference.** An interruption is inferred the instant output stops, without checking whether a child process, a background operation, or an explicit different-shaped signal from that same vendor explains the silence — producing a false "done" mid-legitimate-work.
- **Resurrection by a stale signal.** A hook message delayed in flight arrives after an interruption has already been correctly inferred and is applied anyway, flipping a correct verdict back to "working" for a turn that has already ended — indistinguishable, from the outside, from the exact zombie-status bug this design exists to prevent.
- **Identity hijack through address reuse.** A pane's addressing key is reassigned (a terminal detaches and reattaches, a legacy alias is recycled) and a stale or foreign hook payload, still carrying the old key, is allowed to stamp status onto whatever now holds that address.
- **Possession mistaken for authority.** An alias that was merely *persisted to disk and reloaded*, or *registered* without ever being verified against a live process, is trusted the same as one that was established through an actually-observed live transfer — letting a restored record retroactively acquire authority it never earned.
- **Untrusted persisted state trusted on load.** A corrupted, version-mismatched, or simply stale on-disk status record is loaded and rendered as current fact, because nothing in the load path re-establishes whether it is still true.
- **Self-inflicted noise counted as external anomaly.** A local ingest endpoint's own defensive cap — a request truncated because it exceeded a size guard — is counted the same as a genuinely malformed external request, inflating the very telemetry meant to describe the outside world.

## 2. Constraints & Assumptions

- **Vendors are not modifiable.** Their hook formats, their completeness, and their willingness to fire a cancellation hook are given; the projection adapts, and never assumes a fix upstream.
- **Silence is the normal case, not the exceptional one.** A working agent is silent between tool calls as a matter of course; the projection cannot treat silence itself as a signal of anything.
- **Addressing identities get reused.** A terminal's key, a session's handle, a pane's slot — all of these are recycled across a long-running host process's lifetime, and a projection that assumes stable addressing will eventually misroute.
- **The feed outlives any one process.** A host restart or a hydration from disk is a normal event in a long-running session, not a rare recovery path, so loading persisted status honestly is as central as producing it live.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **ASP-1 (Cooperative hooks are the primary channel; a closed status vocabulary, never a title or a heuristic):** an agent's status is read from the **channel it was built to report through** — a cooperative hook, an explicit lifecycle event — never inferred from a terminal title, a log-output heuristic, or any signal the vendor did not design as a status report. Every projected status resolves to one value from a small, closed, vendor-neutral set (working / blocked / waiting / done, or the platform's equivalent); a vendor-specific status string is translated into this set at the adapter boundary and never passed through raw.

- **ASP-2 (Inference is a declared, per-vendor, narrow fallback — never a universal default):** where a vendor's cooperative-hook contract is known to be **lossy** for a specific transition (most commonly: it does not reliably fire on interruption), the projection MAY synthesize the missing terminal status through a declared inference rule scoped to that vendor and that transition. Inference is **not** applied uniformly across vendors: a vendor whose hook contract is complete for a transition MUST NOT have that transition second-guessed by a heuristic built for a different vendor's gap. Before inferring, the rule checks for known **innocent explanations for silence** specific to that vendor's contract — a child process still active, an explicitly-reported background operation, a shape of silence that vendor's own hooks document as normal — and withholds inference where one applies.

- **ASP-3 (An inferred verdict is generation-fenced: only a signal correlated to the current turn may supersede it):** once a status is settled — whether reported directly or synthesized by ASP-2's inference — a later signal is accepted as superseding it only when that signal is correlated to the **current turn or generation**, not merely to the same agent or the same pane. A hook message that arrived late for the **same turn** the inference already closed MUST NOT resurrect the settled verdict; a signal that opens a **genuinely new** turn (a new prompt, a new invocation) MAY freely re-open working status. This is RLV-4's incarnation-matching and WL-2's compare-and-clear release, realized at the status-verdict grain: an inference is a verdict about a specific turn, and only evidence about that same turn's true outcome — arriving before the inference fired — would have changed it; evidence arriving after is evidence about something that no longer exists in the same form.

- **ASP-4 (Addressing identity is authorized by verified live transfer, never by mere possession or persistence):** where an agent's addressable identity (a pane key, a session handle) can be reassigned or aliased, a hook payload is routed to the entity that identity **currently, verifiedly** names — never to a stale or foreign identity it once named. Establishing authority over a reassigned identity requires a **verified live transfer** (the new holder demonstrably owns the live process at the moment of transfer); an identity that is merely **registered** in a table or **restored** from a persisted record, without ever passing through a verified live transfer, does not carry that authority and MUST NOT be treated as though it did — including after the record survives a restart. This composes CB-3's general rule (any change to a bound element lapses the grant) applied to inter-process addressing rather than human consent.

- **ASP-5 (Persisted status is validated on load, not trusted on load):** loading a previously-persisted status record passes through a declared validation ladder before it is rendered as current: a **version check** refuses a record written by an incompatible format rather than guessing at its shape; a **bounded-age check** drops entries older than a declared staleness cutoff regardless of their recorded status; **per-entry isolation** means one malformed entry is dropped without discarding the entries around it, and a wholly corrupt store degrades to *empty*, never to a thrown error or to a partial guess at its content; and every loaded entry starts **unconfirmed** until a live signal re-establishes it, with the unconfirmed marker itself **never persisted** — it is recomputed fresh on every load, so a record can never accidentally look permanently confirmed because one write path failed to re-stamp it. A mechanism that depends on a settled verdict (ASP-2's inference, most directly) MUST NOT act on an entry still in the unconfirmed state.

- **ASP-6 (A local ingest channel's own defenses do not inflate its own anomaly count):** where an ingest endpoint protects itself with a bound (a size cap, a rate limit) and that bound is what caused a request to fail or truncate, the resulting failure is **excluded** from any count meant to describe genuinely anomalous external input — a request destroyed by the endpoint's own defense is not evidence of a malicious or malformed caller, and counting it as such launders the endpoint's own protective behavior into a false signal about the outside world. An anomaly count also **deduplicates a repeated failure from the same cause** (a retry storm against the same bound) into a single reported incident rather than one per attempt, and names the route the failure occurred on so the report is actionable.

> An L2 implementation cannot reach RFC until every invariant above is addressed in its Invariant Compliance section.

## 4. Detailed Design

### 4.1 Two distinct problems this spec does not solve

- **Commissioning a foreign agent for a bounded task** — launching it, holding it to a posture, verifying its one result — is [l1-foreign-agent-invocation.md](l1-foreign-agent-invocation.md)'s subject. This spec's agent is already running, interactively, as the primary work surface; nobody is launching or grading it as a discrete job.
- **Reading artifacts a tool left behind** is [l1-artifact-derived-observation.md](l1-artifact-derived-observation.md)'s subject, used where a vendor supplies no cooperative channel at all. This spec's channel is the opposite case: a live, in-band, cooperative report the vendor built specifically to be read this way.

### 4.2 The inference decision, restated

```
turn goes silent
       |
does this vendor's hook contract reliably announce interruption? ── yes ──► trust the silence, no inference
       | no
does a known innocent-silence condition apply (child active, background op, ...)? ── yes ──► withhold inference
       | no
infer interrupted (ASP-2) — a settled verdict for THIS turn
       |
a later signal arrives for the SAME turn ── stale, discard (ASP-3)
a later signal opens a NEW turn ── accepted, re-open working (ASP-3)
```

### 4.3 Why authority tiering (ASP-4) cannot be collapsed into simple ownership

A naive model treats "this record names pane X" as sufficient to route to pane X. The failure this collapses is that a record can come to name pane X through three different paths with three different trust levels: it was **verified live** (the strongest — someone actually observed the transfer happen), it was **registered** (someone asserted it, possibly correctly, possibly from a race), or it was **restored** from disk (it was true once, and disk does not know if it still is). Treating all three as equally authoritative is what lets a stale or foreign alias hijack a pane it has no current claim to; the fix is that only the first path grants routing authority, and the other two remain descriptive until a live transfer actually happens.

### 4.4 Failure modes named

| Failure | What it looks like | Which invariant closes it |
| --- | --- | --- |
| Title/heuristic-based status | A confidently wrong status with no vendor behind it | **ASP-1** |
| Cross-vendor inference bleed | A reliable vendor's status second-guessed by another's heuristic | ASP-2 |
| Interruption inferred mid-legitimate-work | A false "done" while a child process is still active | ASP-2 |
| Zombie resurrection | A correct interrupted verdict flipped back by a delayed same-turn hook | **ASP-3** |
| Identity hijack | A foreign or stale key stamping status onto the wrong pane | **ASP-4** |
| Authority from mere possession | A restored alias treated as verified, routing to the wrong process | ASP-4 |
| Trusting corrupt or stale persisted state | A crashed load, or a confidently-rendered stale record | ASP-5 |
| Self-inflated anomaly telemetry | The endpoint's own size cap counted as an external attack | ASP-6 |

## 5. Implementation Notes

1. **Scope inference rules to the (vendor, transition) pair explicitly**, in a table or registry — never as a single global heuristic function that happens to have vendor-specific branches buried inside it (ASP-2).
2. **Carry the turn/generation identifier on every hook payload**, not just the agent/pane identifier, so ASP-3's fencing has something concrete to compare against.
3. **Make "verified live transfer" a distinct, observable event** in the routing state machine (ASP-4), not an inferred property derived after the fact from which table an entry happens to sit in.
4. **Run the ASP-5 validation ladder as a pipeline with an explicit drop-reason per stage**, so a corrupted store's failure mode is diagnosable rather than a silent empty result indistinguishable from "nothing was ever persisted."

## 6. Drawbacks & Alternatives

- **ASP-2's narrowness means some vendors simply get no interruption detection.** Accepted: a vendor with no reliable cancellation signal and no documented innocent-silence conditions cannot be guessed at safely, and reporting "working" honestly-but-wrongly is preferable to a heuristic invented without evidence.
- **ASP-3 means a genuinely-late but correct hook is discarded.** Accepted: the discarded case (a hook that really was about the closed turn, arriving late) is indistinguishable, on the wire, from a resurrection attempt, and treating every late signal as authoritative reopens the exact failure this invariant closes.
- **ASP-5's per-entry isolation costs a slightly more complex load path than "parse or fail."** The cost buys the difference between one bad row losing its own status and one bad row taking down the whole session's history.
- **Alternative — treat this as a special case of `l1-foreign-agent-invocation`.** Rejected: FAI's entire discipline (posture assertion, deadline, baseline diff, re-derived verification) presumes a bounded, commissioned, one-result relationship. An interactively-driven, continuously-observed peer session has no launch-to-verify boundary for any of that machinery to attach to.
- **Alternative — treat vendor hook normalization as an `l1-artifact-derived-observation` adapter.** Rejected: ADO's adapters are explicitly out-of-band and ask for nothing (ADO-1); this spec's hooks are an in-band, cooperative channel the vendor built for exactly this purpose, and ADO's discovery/fingerprint/incremental-read machinery does not fit a live push channel.
- **Alternative — treat a restored alias as provisionally authoritative until proven otherwise.** Rejected by ASP-4: "provisionally authoritative" is exactly the crack a hijack fits through, since nothing then forces the verification to ever actually happen before routing decisions are made on the assumption.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[FOREIGN]` | `.design/main/specifications/l1-foreign-agent-invocation.md` | The commissioned, bounded-task twin demarcated in §4.1 |
| `[RLV]` | `.design/main/specifications/l1-remote-liveness-verdict.md` | RLV-4's generation-fencing, the general form ASP-3 realizes |
| `[LIVENESS]` | `.design/main/specifications/l1-work-liveness.md` | WL-2's compare-and-clear, the general form ASP-3/ASP-4 realize |
| `[OBSERVE]` | `.design/main/specifications/l1-artifact-derived-observation.md` | The out-of-band twin demarcated in §4.1 |
| `[CONSENT]` | `.design/main/specifications/l1-consent-binding.md` | CB-3, the general grant-lapse pattern ASP-4 applies to addressing |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-01 | Core Team | Initial concept — normalizing many concurrently-running, vendor-foreign coding-agent CLIs into one closed-vocabulary status feed. Status is read from the vendor's cooperative hook channel, never a title or heuristic, and always resolves to a small closed vendor-neutral set (ASP-1); a missing-transition inference fallback is declared per (vendor, transition) pair, never applied universally, and withheld where a known innocent-silence condition explains the gap (ASP-2); a settled or inferred verdict is generation-fenced — only a signal correlated to the current turn may supersede it, a delayed same-turn signal never does — generalizing RLV-4/WL-2's incarnation-matching to a status verdict (ASP-3); addressing identity is authorized by a verified live transfer, never by mere registration or persisted-record possession, composing CB-3's grant-lapse-on-change rule (ASP-4); persisted status is validated on load through a version/age/per-entry-isolation ladder with an unconfirmed-until-reconfirmed default that is itself never persisted (ASP-5); and a local ingest channel's own protective bound never inflates its own external-anomaly count (ASP-6). Distilled from an adoption pass over an external desktop multi-agent orchestrator's per-vendor agent-status hook normalization layer. Concept-only. |

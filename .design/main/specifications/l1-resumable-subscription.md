# Resumable Subscription

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The contract between a **durable ordered log** and a **subscriber that will disconnect and come back**. Every long-lived observation surface in the office is one of these: a session transcript being watched from three windows, an agent inbox, a board's activity feed, a run's step events, a communication view over agents talking to each other. Each has a producer appending records and one or more consumers that lose the connection — a sleeping laptop, a dropped network, a reloaded window, a restarted client — and must rejoin without missing anything and without seeing anything twice.

The design turns on a small number of choices that are easy to get wrong once and then impossible to change. The record's own position in the log **is** the resume cursor, so resumption needs no side-channel sequence and no de-duplication. The durable write is the source of truth and the live notification is only a wake-up, so a lost notification costs a reconnect rather than an event. And the delivery guarantee is stated **precisely enough to be relied on**: exactly-once and gap-free above the cursor, *for the records this contract version can represent* — with the representability exception spelled out, because the alternative is a subscriber that wedges forever behind one record it will never be able to read.

The second half of the spec is about what the contract deliberately does **not** promise, since that is where consumers invent guarantees the producer never made — most commonly that the batch delivered first means "you are now caught up," and that a gap in positions means something was lost.

## Related Specifications

- [l1-event-mesh.md](l1-event-mesh.md) — EM-7 declares delivery semantics per subscription; EM-9 confines the mesh to in-process routing. This spec covers the other case: a durable log observed by a subscriber that may be remote and may be gone for hours.
- [l1-acp.md](l1-acp.md) — ACP-8's ordered event stream is the external projection of exactly this contract; RSB-3's cursor is what a reconnecting ACP client resumes on.
- [l1-peer-compatibility.md](l1-peer-compatibility.md) — RSB-5's representability skip is a direct consequence of PCO-2/PCO-3: a subscriber and a producer at different versions disagree about which record kinds exist, and one of them must not stall.
- [l1-record-evolution.md](l1-record-evolution.md) — REC-4/REC-10: whether an unfamiliar record can be *carried* rather than skipped; where it can, RSB-5 does not fire.
- [l1-observation-retention.md](l1-observation-retention.md) — How long the log keeps records; RSB-9's cursor-too-old case is where retention meets resumption.
- [l1-multi-device-sync.md](l1-multi-device-sync.md) — Several devices subscribing to the same logical activity from different sources; RSB-10's client-side merge.
- [l1-work-liveness.md](l1-work-liveness.md) — WL-9's exact-once fan-out is the producer-side sibling; this spec governs the consumer side of the same delivery.
- [l1-log-legibility.md](l1-log-legibility.md) — What a record says; this governs when and in what order it arrives.
- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — The surface consuming a stream decides what to reveal; RSB-6 forbids it from reading novelty out of the transport's framing.
- [l1-nodus-observability.md](../../nodus/specifications/l1-nodus-observability.md) — The DSL-grain trace stream; the same resumption contract applies to a workflow run watched across a disconnect.

## 1. Motivation

**Reconnection is the normal case, not the exceptional one.** Any surface a person leaves open for a day will disconnect: sleep, network change, window reload, client restart, engine restart. A stream whose contract only describes the connected state pushes every one of those into ad-hoc client code, and each client invents a different, subtly wrong recovery.

**De-duplication on resume is a cost that a good cursor choice removes entirely.** If records carry an identifier unrelated to their order, a resuming subscriber must re-request a window and reconcile it against what it already has — which requires stable identity, ordering, and a merge, in every client. If the record's position in the log *is* its identity, resumption is "send me everything above N" and there is nothing to reconcile.

**A live notification is not a durable fact, and treating it as one loses events.** When the notification and the write are both required for delivery, a dropped notification is a lost record. When the write is the truth and the notification merely wakes a fan-out, the same drop costs one reconnect and nothing else. The difference is invisible in testing and decisive in production.

**A version-skewed subscriber can wedge a stream forever.** A log shared between builds — or touched by a repair — will eventually contain a record the serving side cannot express in the version it negotiated. If such a record is held back until it can be delivered, the subscription stalls permanently at that position and the whole surface goes dark over one row. If it is retried, the retry loops. The only non-catastrophic behaviour is to skip it and advance — which means the contract must say so, out loud, because the consequence lands on the consumer.

**Consumers read meaning into framing, and the framing has none.** The first batch after connecting is a transport optimisation, not a statement about the world; a record arriving in a later frame may be from last week. Left unstated, every consumer eventually assumes "batch finished" means "caught up to now" and animates, badges, or scrolls on that assumption — then behaves absurdly on a reconnect that back-fills two hours of history.

## 2. Constraints & Assumptions

- The log is append-only and its records are immutable. Editing or deleting a delivered record is outside this contract and would invalidate the cursor's meaning.
- A single log has one authoritative ordering. Ordering *across* independent logs is not defined by this spec and is addressed by RSB-10.
- Subscribers may be at a different contract version than the producer (see peer compatibility); this is normal and must not stall anyone.
- Retention is finite. A cursor may fall out of the retained window (RSB-9).
- This spec governs delivery of an ordered record stream. It does not govern what records *mean*, who may see them, or how long they are kept — those are the legibility, confidentiality, and retention models.

## 3. Core Invariants (Layer 1 only)

- **RSB-1 (Append-only, immutable records):** the log is only ever appended to. A delivered record is never mutated or removed in place. This is what makes a position a durable identity and resumption reconciliation-free; any mechanism that needs to change a record's content emits a *new* record that supersedes it.

- **RSB-2 (The durable write is the source of truth; live notification is only a wake-up):** a record exists once it is committed to the log, independently of whether any subscriber was notified. The live fan-out is an optimisation that wakes open subscriptions; a lost or coalesced notification MUST cost at most a reconnect, never an event. Delivery correctness therefore never depends on the in-memory path being reliable.

- **RSB-3 (The record's position is the cursor; there is no second sequence):** the record's own monotonically increasing position in the log serves as both its identity and the resume cursor. A subscriber resumes by naming the highest position it has fully applied and receives only records above it. No separate sequence number, no window re-request, no client-side de-duplication.

- **RSB-4 (The stated guarantee is order and completeness above the cursor, and nothing more):** for every record above the subscriber's cursor that the negotiated contract version can represent, delivery is **exactly once, in ascending position order, with nothing skipped**, across the whole subscription including any initial batch. This is the entire promise. Anything a consumer additionally believes — about timing, about batching, about which frame carried what — is not part of the contract and MUST NOT be relied on.

- **RSB-5 (A record the serving side cannot represent is skipped and the cursor advances):** where the log contains a record whose kind the serving side cannot express in the negotiated contract version, it is **skipped**, the cursor moves past it, and the subscription continues. It is not held, not retried, and not surfaced. Holding it stalls the subscription permanently behind one record and takes the entire surface down; retrying loops forever. The correct remedy for a genuinely new record kind is a compatible contract version both sides negotiate — the skip is the interim behaviour that keeps an older subscriber working against a newer producer, never a substitute for that version.

- **RSB-6 (A position gap is not evidence of loss, and consumers MUST NOT react to one):** because RSB-5 advances the cursor past unrepresentable records, gaps in delivered positions are expected. A subscriber MUST NOT treat a gap as a missing record: re-requesting a lower cursor or retrying loops indefinitely, since the skip is deterministic and will recur on every attempt. Position continuity is not part of the guarantee; position *ordering* is.

- **RSB-7 (Framing carries no semantics about the world):** how records are packaged for transport — an initial batch, subsequent individual frames, any chunking — is a delivery decision and means nothing about recency, novelty, or completeness. In particular, the end of an initial batch does **not** mean "caught up to now": it is bounded, and a backlog beyond that bound continues to arrive in later frames indistinguishable from live activity. Novelty — what to animate, badge, notify on, or scroll to — MUST be derived from record content and the subscriber's own state, never from which frame carried a record.

- **RSB-8 (A closed vocabulary is used only where an unknown value makes the record unreadable):** a field that decides *which other fields on the record mean anything* is a closed set, because an unknown value leaves a consumer with nothing it can render; a field that merely annotates an otherwise-renderable record is open, and an unknown value degrades to showing it as given. This asymmetry is deliberate and is what determines whether an addition costs a version bump (RSB-5's skip) or costs nothing.

- **RSB-9 (Resumption failure is explicit and distinguishable from an empty gap):** a cursor that predates the retained window is not silently treated as "nothing new." The subscription reports that the requested position is no longer available, distinctly from reporting that there is nothing above it, so the consumer can re-baseline rather than concluding the world stood still.

- **RSB-10 (Records from independent logs are merged by the consumer, never fabricated into one order):** where one logical view spans several logs with disjoint record sets, each is subscribed to separately and merged consumer-side on record content. No producer-side merged ordering is invented, because none exists: clocks across sources may skew. This is safe exactly when causally related records share a single log — both ends of an exchange written to one log — so only causally unrelated records can misorder relative to each other. Where that condition does not hold, a merged view MUST NOT claim a global order.

- **RSB-11 (Subscription failure is per-source, never global):** in a multi-source view, one source's failure — unreachable, unsupported method, incompatible version — degrades that source's contribution to a visible, reasoned absence while every other source keeps streaming. A single source's problem MUST NOT take down the view.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The shape of the contract

```text
[REFERENCE]
open(log_id, since_cursor)  ->  ordered delivery of every representable record above since_cursor,
                                followed by live records as they are committed

record := { position, timestamp, kind, payload }     // position is total order AND identity
cursor := position of the highest record the subscriber has fully APPLIED

resume := open(log_id, cursor)                       // the gap, not the whole log
```

Two properties make resume trivial. Positions are **monotonic**, so "above N" is well-defined without a scan; records are **immutable** (RSB-1), so a record delivered once never needs to be delivered again with different content. Together they remove de-duplication entirely — not "make it cheap", remove it. The subscriber never sees a record twice and never reconciles a re-sent batch.

The cursor is the highest position **applied**, not the highest received. A subscriber that receives a record and fails to process it must not advance past it, or the resume that follows will skip work the log still held.

### 4.2 Write, then wake

```text
[REFERENCE]
append(record):
    commit to durable log            // the record now EXISTS (RSB-2)
    wake per-log emitter             // best-effort fan-out to open subscriptions

emitter drop / coalesce / crash  ->  subscriber notices on reconnect and resumes from cursor
                                     -> at most a delay, never a lost record
```

Inverting this — treating a successful fan-out as part of delivery — makes every in-memory failure a data-loss event and forces the producer to track per-subscriber acknowledgement. The chosen order is what lets the live path be entirely best-effort, which in turn is what lets it be cheap.

### 4.3 Representability, and the reason the skip is not optional

A log outlives any single contract version. It can contain records written by a newer build sharing the same store, records left by a repair, or records of a kind the negotiated version predates. The serving side has three options and only one of them is survivable:

| Option | Outcome |
| --- | --- |
| Hold the record until it can be delivered | The subscription stalls at that position **forever**. One record takes down the whole surface. |
| Deliver a placeholder or a fallback kind | The consumer receives something it cannot render and must guess — the exact ambiguity the contract avoids everywhere else. |
| **Skip it and advance the cursor** (RSB-5) | A silent hole. Everything else keeps working. |

The third is chosen, and its cost is paid by RSB-6: consumers must be told, in the contract, that gaps are normal — because the instinctive response to a gap is to re-request, and re-requesting a deterministic skip is an infinite loop.

It is worth being precise about the boundary with record evolution: RSB-5 fires only where the record genuinely **cannot be expressed**. Where the record's shape permits carrying an unfamiliar variant intact ([l1-record-evolution.md](l1-record-evolution.md) REC-4), it is delivered rather than skipped, and the consumer renders what it can. The skip is the last resort, not the first tolerance mechanism.

### 4.4 What framing does not mean

RSB-7 exists because the natural reading of a stream is wrong in a specific, repeatable way.

```text
[REFERENCE]
[ initial batch: bounded ] [ frame ] [ frame ] [ frame ] ...
                          ^
                          NOT "you are now caught up"
```

The initial batch is bounded for transport reasons. A backlog larger than that bound arrives in later frames, which are shaped identically to live activity. So "batch ended" is not a boundary between past and present, and a consumer that animates, badges, or notifies on later frames will do so for records from last week after a long disconnect.

The correct source of novelty is the record's own content — its timestamp, compared against what the subscriber already knew. This is unavoidable anyway in the multi-source case (RSB-10), where frames from several subscriptions interleave and no single stream's framing means anything globally; stating it as an invariant merely makes the single-source case honest about the same rule.

### 4.5 Closed and open vocabularies inside a record

RSB-8 gives a decision procedure that is usually applied by instinct and often backwards.

| Field role | Vocabulary | Unknown value behaves as |
| --- | --- | --- |
| Decides which other fields are meaningful (the record's *kind*) | **closed** | unreadable → RSB-5 skip; a new kind needs a version both sides negotiate |
| Annotates a record that renders fine without it (a reason, a label, a category) | **open** | shown as given; costs nothing to add |

The asymmetry is the point. Making the annotation closed forces a version bump for a change that could have been free. Making the kind open hands the consumer a record whose fields it cannot interpret and invites it to guess — and a guess about the shape of unfamiliar data is how a viewer renders a confidently wrong account of what happened.

### 4.6 Several logs, one view

```text
[REFERENCE]
view over sources S1..Sn:
    for each Si:  open(Si, cursor_i)          // independent subscriptions, disjoint records
    merge frames consumer-side by record timestamp
    one Si fails -> that source shows a reasoned absence; the rest keep streaming  (RSB-11)
```

No producer-side merge exists because none can be correct: wall clocks skew across sources, and no source can order records it never saw. The consumer-side merge is honest about being approximate — and it is *safe* precisely under the condition RSB-10 names: causally related records share one log. When both ends of an exchange are written to the same log, causality is never violated by the merge; only unrelated activity can appear in the wrong relative order, which is a cosmetic imprecision rather than a false account.

Where that condition cannot be met, the view must not present a single ordered timeline at all — grouping per source is truthful; a merged order is not.

### 4.7 Demarcation

| Neighbour | Its question | Why it is not this |
| --- | --- | --- |
| [l1-event-mesh.md](l1-event-mesh.md) | How is an in-process event routed to handlers? | The mesh is routing inside one engine (EM-9), with no durability and no resume. This is a durable log with a disconnecting consumer. |
| [l1-observation-retention.md](l1-observation-retention.md) | How long is a record kept? | Retention decides what still exists; this decides how what exists is delivered. They meet only at RSB-9. |
| [l1-work-liveness.md](l1-work-liveness.md) | Is the work still moving, and who owns it? | WL-9's exact-once fan-out is the producer's obligation on dispatch; this is the consumer's contract on observation. |
| [l1-multi-device-sync.md](l1-multi-device-sync.md) | How do replicas converge on shared state? | Sync reconciles divergent *state*; this delivers an ordered *record* stream. A subscription never merges — it only reads. |

### 4.8 Nodus relevance

| Element | nodus seam | Note |
| --- | --- | --- |
| Position-as-cursor (RSB-3) | run trace ordering | A run trace already has a total step order; that order is the resume cursor for a detached observer, with no separate sequence. |
| Write-then-wake (RSB-2) | observability sink | The durable trace record is the truth; live streaming to an attached watcher is best-effort. |
| Representability skip (RSB-5) | trace records from a newer runtime | An older viewer reading a newer run's trace skips step kinds it cannot express rather than refusing the whole trace. |
| Open vs closed vocabulary (RSB-8) | step kind vs step outcome annotation | The step *kind* decides which fields mean anything (closed); an outcome reason is an annotation (open). |
| Per-source degradation (RSB-11) | multi-run dashboard | One unreachable run's trace darkens that run's lane only. |

## 5. Drawbacks & Alternatives

- **The silent skip (RSB-5) hides real corruption.** A record damaged badly enough to be unrepresentable vanishes with no signal to the subscriber. Accepted deliberately: the subscriber can do nothing with a record it has no schema for, and every alternative signal invites guessing. The producer side still records the skip in its own diagnostics — the silence is toward the *subscriber*, not toward the operator.

- **Position-as-cursor couples the wire to the store's ordering.** Changing how records are stored can change what a cursor means. Constrained by RSB-1: as long as the log is append-only and immutable, the position is stable by construction. A store change that violates that is a major contract change, not an implementation detail.

- **RSB-7 pushes work onto every consumer.** Deriving novelty from content is more work than reading it off the frame. Held anyway: the cheaper reading is wrong after any long disconnect, and it fails in the most visible way possible — a burst of notifications for stale activity.

- **Consumer-side merge (RSB-10) is approximate.** Two unrelated records from different sources may appear in the wrong order. Accepted as the honest option; the alternative is a producer-side order that would have to be invented from skewed clocks, which is the same imprecision with a false claim of authority attached.

- **Alternative — per-subscriber acknowledgement and server-side retry.** Rejected: it makes the producer stateful per subscriber, which does not survive many transient subscribers, and it buys nothing that RSB-3's cursor does not already provide more cheaply.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MESH]` | `.design/main/specifications/l1-event-mesh.md` | EM-7/EM-9 — the in-process sibling and its explicit scope boundary. |
| `[ACP]` | `.design/main/specifications/l1-acp.md` | ACP-8 ordered event stream — the external projection of this contract. |
| `[PEERS]` | `.design/main/specifications/l1-peer-compatibility.md` | PCO-2/PCO-8 — why a subscriber and producer disagree about record kinds, and how that is negotiated. |
| `[RECORDS]` | `.design/main/specifications/l1-record-evolution.md` | REC-4/REC-10 — when an unfamiliar record can be carried rather than skipped. |
| `[RETENTION]` | `.design/main/specifications/l1-observation-retention.md` | What still exists to resume from; RSB-9's boundary. |
| `[LIVENESS]` | `.design/main/specifications/l1-work-liveness.md` | WL-9 — the producer-side exact-once obligation. |
| `[NODUS-OBS]` | `.design/nodus/specifications/l1-nodus-observability.md` | The DSL-grain trace stream this contract applies to. |

## Document History

| Version | Date | Author | Change |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-20 | Core Team | Initial spec — the resumable ordered-log subscription contract: append-only immutable records as the basis for a durable position identity (RSB-1); the durable write as source of truth with live notification demoted to a best-effort wake-up, so a dropped notification costs a reconnect and never an event (RSB-2); the record's own position serving as both identity and resume cursor, removing de-duplication and window re-request entirely, with the cursor tracking what was *applied* rather than received (RSB-3); a precisely bounded guarantee — exactly-once, ascending, gap-free above the cursor for representable records, and nothing else (RSB-4); the representability skip, chosen because holding stalls a subscription permanently behind one record and retrying loops, with a negotiated version as the real remedy (RSB-5); the corollary that position gaps are not evidence of loss and must never be retried, since the skip is deterministic and recurs (RSB-6); framing carries no world semantics — a bounded initial batch is a transport decision, "batch ended" is not "caught up", and novelty derives from record content and subscriber state (RSB-7); closed vocabulary only where an unknown value makes the record unreadable, open where it merely annotates, which decides whether an addition costs a version (RSB-8); explicit, distinguishable reporting when a cursor predates retention rather than silent "nothing new" (RSB-9); consumer-side merge across independent logs with no invented producer-side order, safe exactly because causally related records share one log (RSB-10); and per-source degradation so one failing source never darkens a multi-source view (RSB-11). Demarcated from the in-process event mesh, retention, work liveness and device sync in §4.7; nodus mapping to the run-trace stream. Concept-only. Distilled from an adoption pass over an external multi-provider agent-orchestration desktop client whose activity views are resumable per-host subscriptions merged client-side. |

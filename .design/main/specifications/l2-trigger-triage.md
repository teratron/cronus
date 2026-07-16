# Trigger Triage

**Version:** 1.0.1
**Status:** Stable
**Layer:** implementation
**Implements:** l1-orchestration.md, l1-scheduler-model.md

## Overview

The trigger intake pipeline: every inbound signal — chat message, webhook, cron fire, internal event, sub-agent spawn request — is wrapped in a `TriggerEnvelope`, classified by a local LLM (with cloud fallback), and routed to one of four outcomes: Drop, Notify, SpawnReactor, or SpawnOrchestrator. A dedup cache suppresses duplicate fires within a configurable window.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) - ORC-1/ORC-2 topology and delegation model.
- [l1-scheduler-model.md](l1-scheduler-model.md) - Cron and event-driven fire actions.
- [l2-scheduler.md](l2-scheduler.md) - Event-driven triggers (§4.8) that produce `TriggerEnvelope` objects.
- [l2-agent-session.md](l2-agent-session.md) - Sessions spawned by SpawnReactor and SpawnOrchestrator.
- [l2-orchestration.md](l2-orchestration.md) - SpawnOrchestrator hands off to the goal-execution loop.

## 1. Motivation

Without a triage layer, every inbound signal spawns a full agent session. This wastes resources on spam, duplicate events, and low-value notifications. A fast, local classifier separates signals by urgency and scope before anything expensive starts — and the cloud fallback keeps classification reliable for ambiguous inputs.

## 2. Constraints & Assumptions

- The local classifier runs in-process and must return a decision in under 200 ms for the common case.
- Cloud fallback is optional; if unavailable (offline, rate-limited), the pipeline falls to rule-based classification.
- The dedup cache is in-memory, per-process; restart clears it.
- Triage does not inspect full payload content — only short classification excerpts — so user data is not read in this pipeline.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| ORC-1 One orchestrator | `SpawnOrchestrator` always creates work under the manager; triage never spawns peer orchestrators. |
| SCH-3 Single action | Each `TriggerEnvelope` maps to exactly one `TriageDecision`; no multi-dispatch. |

## 4. Detailed Design

### 4.1 TriggerEnvelope

Every inbound signal is normalized into a `TriggerEnvelope` before entering the pipeline:

```text
[REFERENCE]
TriggerEnvelope {
  id:           String,                       // UUID generated at intake
  source_type:  SourceType,
  source_id:    Option<String>,               // e.g. schedule_id, webhook_id
  payload:      TriggerPayload,               // classification inputs (not full content)
  received_at:  Instant,
  workspace_id: String,
  metadata:     HashMap<String, String>,      // source-specific key-value pairs
}

SourceType:
  "chat_message"      // inbound from the interactive session
  "webhook"           // HTTP webhook delivery
  "cron"              // scheduled fire from the scheduler
  "event"             // internal event (card_status_changed, tool_call_blocked, etc.)
  "sub_agent_spawn"   // sub-agent requesting a resource or decision from its parent
```

`TriggerPayload` contains only classification-relevant fields: a text excerpt (≤ 200 tokens), `event_kind` (if present), and an `urgency_hint` set by the source. Full payload content is not read by triage.

### 4.2 TriageDecision

```text
[REFERENCE]
TriageDecision:
  Drop            { reason: String }
  Notify          { message: String, channel: String }
  SpawnReactor    { agent_tier: "chat", task: String }
  SpawnOrchestrator { goal: String, priority: "low" | "normal" | "high" }
```

| Decision | Meaning |
| --- | --- |
| `Drop` | Signal is discarded (spam, duplicate, rate-limited). Logged at DEBUG. |
| `Notify` | A notification is written to the log or notification channel. No agent is spawned. |
| `SpawnReactor` | A lightweight `Chat`-tier agent handles the signal in a single turn. |
| `SpawnOrchestrator` | The signal becomes a goal handed to the orchestrator's goal-execution loop. |

### 4.3 Triage pipeline

```text
[REFERENCE]
triage(envelope: TriggerEnvelope) -> TriageDecision:

  1. rate_limit_check(envelope)
       → if exceeded: return Drop { reason: "rate_limit" }

  2. dedup_check(envelope)
       → if duplicate within window: return Drop { reason: "dedup_suppressed" }

  3. payload_guard(envelope)
       → malformed or oversized payload: return Drop { reason: "payload_invalid" }

  4. classify(envelope) -> (TriageDecision, confidence: f32)
       → local LLM classifier (§4.4)
       → if confidence < CONFIDENCE_THRESHOLD (0.8): cloud_classify(envelope)
       → if cloud unavailable: rule_classify(envelope)

  5. record_dedup(envelope)

  6. return decision
```

### 4.4 Local classifier

The local classifier is a small, CPU-only quantized classification model loaded at agent startup. It receives the normalized `TriggerPayload` and returns a `(TriageDecision, confidence: f32)` pair.

- **Latency target**: ≤ 200 ms p99 on the host CPU.
- **Interface**: pluggable via the extension registry (kind: `plugin`, interface: `TriageClassifier`).
- **Inputs**: `source_type`, payload text excerpt, `event_kind` (if any), `urgency_hint`.
- **Confidence**: a float in [0, 1]. Below `CONFIDENCE_THRESHOLD` (default 0.8), the cloud fallback is triggered.

If the local model is unavailable (initialization failure, not loaded), the pipeline skips directly to rule-based classification.

### 4.5 Cloud fallback

When local confidence is below threshold, the pipeline calls the configured cloud model with a structured classification prompt. The cloud call:

- Uses the primary model from the model router (may be rerouted to a cheaper model by the router's cost policy).
- Receives the same `TriggerPayload` fields as the local classifier (no full content).
- Returns a `(TriageDecision, confidence)` pair. Below-threshold cloud results fall to rule classification.

If the cloud call fails (timeout, rate limit, provider error), the pipeline falls to rule classification.

### 4.6 Rule-based fallback

Deterministic fallback when both classifiers are unavailable or below confidence:

| Condition | Decision |
| --- | --- |
| `source_type == "cron"` | `SpawnReactor` |
| `source_type == "webhook"` | `SpawnReactor` |
| `source_type == "event"` and `event_kind` contains `"error"` | `SpawnOrchestrator` |
| `source_type == "chat_message"` | `SpawnReactor` |
| `source_type == "sub_agent_spawn"` | `SpawnReactor` |
| all others | `Notify` |

### 4.7 Dedup cache

```text
[REFERENCE]
DeduplicateCache {
  store:      HashMap<(SourceType, ContentHash), Instant>,
  window_sec: u32,   // configurable per source_type
                     // defaults: event=60, webhook=30, cron=0 (off), chat_message=0 (off)
}
```

`ContentHash` is a truncated SHA-256 of the payload text excerpt and `event_kind` (if present). The cache is populated AFTER a successful dispatch (step 5 in §4.3), so a triage failure does not prevent re-delivery.

### 4.8 Rate limiting

Each `SourceType` has an independent rate limit (configurable; defaults: `webhook` 100 req/min, `event` 500 req/min, others unlimited). Envelopes exceeding the limit receive `Drop { reason: "rate_limit" }` and are counted in the audit log.

### 4.9 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| list recent triggers | `cronus trigger list [--limit N]` | `/trigger list` | `trigger.list(n) -> TriggerEnvelope[]` |
| view history | `cronus trigger history [--since <datetime>]` | `/trigger history` | `trigger.history(since) -> TriageRecord[]` |
| replay a trigger | `cronus trigger replay <id>` | `/trigger replay <id>` | `trigger.replay(id) -> TriageDecision` |

## 5. Drawbacks & Alternatives

- **CPU-only local model is a bundled dependency**: if no quantized classifier is included, the pipeline falls to rule-based classification, which is less accurate. The extension registry allows dropping in a custom classifier.
- **Disclosed simplification (FR-6)** `[ADDED]`: the shipped local classifier is a stub that always reports 0.0 confidence, so every signal currently resolves through the rule-based fallback path — the rule path is the honest default today, not a degraded accident. Upgrade trigger: a bundled (or extension-registry-installed) quantized classifier; the seam is internals-only, callers unchanged.
- **Dedup is best-effort**: in-memory cache; process restart allows a short re-fire window. For strict dedup, persist the cache to SQLite (deferred).
- **No multi-dispatch**: one signal → one decision. A signal that legitimately needs both a Notify and a SpawnReactor must be handled by a post-dispatch hook on the reactor.
- **Alternative — no triage, always spawn**: simpler, but unbounded agent spawning on high-volume event streams would exhaust compute and API budget.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORC]` | `.design/main/specifications/l1-orchestration.md` | Orchestration invariants |
| `[SCHED_MODEL]` | `.design/main/specifications/l1-scheduler-model.md` | Fire action model |
| `[SCHED]` | `.design/main/specifications/l2-scheduler.md` | Event-driven trigger source |
| `[SESSION]` | `.design/main/specifications/l2-agent-session.md` | Sessions spawned by triage |

## Document History

| Version | Date | Notes |
| --- | --- | --- |
| 1.0.1 | 2026-07-16 | Disclosed simplification (FR-6) recorded in §5: the shipped local classifier stub always reports 0.0 confidence, so classification currently resolves via the rule-based fallback; upgrade trigger = a bundled or extension-installed quantized classifier. History table added with this entry. |

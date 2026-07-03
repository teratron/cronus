# Office Deliberation

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-deliberation.md

## Overview

The concrete deliberation engine in `crates/core`: an orchestrator-initiated round runner that dispatches independent parallel arguments (no cross-reading), synthesizes them into a final decision with attribution, and writes an immutable round entry to the deliberation log. The log is backed by the inbox SQLite store under a distinct message type and surfaces in the Channels sidebar tab. Parallel argument generation reuses the orchestration wave-execution model.

## Related Specifications

- [l1-deliberation.md](l1-deliberation.md) — the model this implements (DL-1…DL-5).
- [l2-orchestration.md](l2-orchestration.md) — wave-based parallel execution + budget protocol for rounds.
- [l2-inbox.md](l2-inbox.md) — SQLite store backing log entries (`deliberation_round` message type).
- [l2-navigation.md](l2-navigation.md) — the Channels tab surfacing the deliberation log.
- [l2-role-catalog.md](l2-role-catalog.md) — preset roles hired on demand for specialty diversity (DL-2).

## 1. Motivation

The model requires independent reasoning, orchestrator finality, and a visible audit trail. Gathering arguments in parallel with no cross-reading enforces independence (DL-1); reusing the inbox store avoids a parallel persistence layer; the append-only log makes the office's reasoning legible in Channels.

## 2. Constraints & Assumptions

- Only the orchestrator initiates a round; workers cannot self-initiate.
- Participation defaults to 3; each round declares a token budget upfront.
- The orchestrator always decides — no voting/majority rule.
- Budget exhaustion truncates arguments with a visible marker; the round still concludes with a decision.

## 3. Invariant Compliance (Layer 2)

| L1 Invariant | Implementation |
| --- | --- |
| DL-1 Independent reasoning | Each participant runs in its own isolated session; arguments dispatch as a parallel wave (orchestration) with no participant reading another's output before all return. |
| DL-2 Specialty diversity | Participant selection prefers active workers with overlapping specialties, else hires preset-catalog roles for the round (released after), and may mix model tiers for perspective breadth. |
| DL-3 Orchestrator finality | After all arguments return, the orchestrator synthesizes and emits the decision; no vote/majority code path exists. |
| DL-4 Append-only log | On close, the full round entry is written immutably to the deliberation log; there is no update/delete path for a round row. |
| DL-5 Budget-bounded | Each round declares `token_budget`; an argument exceeding its slice is truncated with a `truncated=true` marker, never silently dropped. |

## 4. Detailed Design

### 4.1 Round runner

```text
[REFERENCE]
run_round(question, n, budget):
  participants := select(question, n)                 // DL-2
  args := parallel_wave(participants, question, budget/n)   // DL-1 no cross-read
  decision := orchestrator.synthesize(args)           // DL-3
  entry := { round_id, opened_at, question, participants, arguments: args,
             decision, reasoning, token_budget, truncated }
  deliberation_log.append(entry)                      // DL-4 immutable
  return decision
```

Wall-clock ≈ max(argument time) + synthesis, since arguments run in parallel.

### 4.2 Participant selection

Priority: (1) active workers with overlapping specialty; (2) preset roles hired for the round then released; (3) mixed model tiers for diversity. The orchestrator MAY designate one participant a **devil's advocate** via a per-round system-prompt constraint (not a separate agent type), skippable for low-stakes rounds.

### 4.3 Log entry

Stored in the inbox store, message type `deliberation_round`: `{round_id, opened_at, question, participants[{role, model}], arguments[{role, position_summary, key_points[], confidence}], decision, reasoning, token_budget, truncated}`. Surfaced in Channels with per-argument attribution, explicit decision+reasoning, and searchable/filterable history.

## 5. Implementation Notes

1. Parallel argument generation uses the orchestration wave-execution model — no new parallelism primitive.
2. Log entries share the inbox SQLite store with a distinct `deliberation_round` message type for type-filtered queries.
3. Devil's-advocate is a per-round flag on one participant's session prompt.

## 6. Drawbacks & Alternatives

**Alternative — open group discussion (cross-reading)**: violates DL-1; premature convergence. Rejected.

**Alternative — majority vote**: discards argument quality; loses minority insight the orchestrator judges significant. Rejected — orchestrator synthesizes (DL-3).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-deliberation.md` | Invariants DL-1…DL-5 |
| `[ORCH]` | `.design/main/specifications/l2-orchestration.md` | Wave-parallel execution + budget |
| `[INBOX]` | `.design/main/specifications/l2-inbox.md` | Log entry SQLite backing |
| `[NAV]` | `.design/main/specifications/l2-navigation.md` | Channels tab surface |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-03 | Core Team | Initial implementation spec — orchestrator-initiated round runner, parallel independent arguments, synthesis with attribution, immutable log over the inbox store, Channels surface; maps DL-1…DL-5. |

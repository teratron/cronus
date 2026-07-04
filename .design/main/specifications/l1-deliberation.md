# Office Deliberation

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Office deliberation is a structured protocol for multi-worker debate. When the orchestrator faces a decision that benefits from independent perspectives — architectural choices, ambiguous requirements, risk evaluation — it assembles a deliberation round: selected workers reason independently from their specialties, and the orchestrator reads all arguments before making the final decision.

The deliberation log is also the primary mechanism for inter-role communication visibility. Users who want to understand HOW the office reached a decision — not just WHAT was decided — read the deliberation log in the Channels sidebar tab.

## Related Specifications

- [l1-office-model.md](l1-office-model.md) — orchestrator authority and role workers that participate
- [l1-orchestration.md](l1-orchestration.md) — delegation and budget protocol used to run deliberation rounds
- [l1-navigation-model.md](l1-navigation-model.md) — Channels tab where the deliberation log surfaces
- [l2-inbox.md](l2-inbox.md) — storage backing for deliberation log entries

## 1. Motivation

Two problems in multi-agent offices motivate deliberation:

1. **Convergence without diversity**: if one agent proposes a solution and others simply affirm it, the multi-agent benefit is illusory. Deliberation forces each participant to reason independently before synthesis.

2. **Invisible decision-making**: autonomous offices make hundreds of decisions per session. Without a visible reasoning trail, users cannot understand, audit, or trust the office's judgment. The deliberation log makes reasoning legible.

## 2. Constraints & Assumptions

- Deliberation is initiated by the orchestrator; workers cannot initiate a deliberation round themselves.
- Participation is bounded (default: 3 participants); larger panels increase cost with diminishing returns.
- The orchestrator always makes the final decision. Deliberation is advisory; it cannot override orchestrator authority.
- Each round operates within a token budget declared before the round opens; if budget is exhausted, the orchestrator decides on available evidence (no round is abandoned without a decision).

## 3. Core Invariants

- **DL-1 Independent reasoning**: each participant generates its argument before seeing any other participant's output. Arguments are gathered in parallel; no participant is influenced by another during argument generation.
- **DL-2 Specialty diversity**: participants are selected to maximise coverage of distinct specialties relevant to the question. If no diverse-enough set of active workers exists, the orchestrator may instantiate temporary specialist workers for the round.
- **DL-3 Orchestrator finality**: the orchestrator issues the final decision after reading all arguments. No voting rule or majority mechanism determines the outcome — the orchestrator synthesises and decides.
- **DL-4 Append-only log**: every deliberation round is written to the deliberation log immediately upon close. The log is immutable; past decisions cannot be retroactively edited. The log is the office's audit trail.
- **DL-5 Budget-bounded execution**: each round declares a token budget upfront. Arguments that exceed the budget are truncated with a marker; they are not silently discarded. Truncation is visible in the log.

## 4. Detailed Design

### 4.1 Deliberation Round Lifecycle

```text
[REFERENCE]
1. OPEN    — orchestrator defines: question, participants (N workers), token budget
2. ARGUE   — N workers generate arguments in parallel (DL-1); no cross-reading
3. SYNTHESISE — orchestrator reads all N arguments in sequence
4. DECIDE  — orchestrator emits: decision text + reasoning + argument attribution
5. LOG     — full round entry written to deliberation log (DL-4)
```

Total wall-clock time ≈ max(individual argument times) + orchestrator synthesis time, since arguments run in parallel.

### 4.2 Participant Selection

Priority order:

1. Active workers in the office whose declared specialties overlap with the question domain
2. Preset-catalog roles with relevant specialties, hired on demand for the round and released afterward
3. Workers on different model tiers — mixing a high-capability reasoning model with a cost-efficient model broadens perspective diversity

Additionally, the orchestrator MAY designate one participant as a **devil's advocate** — tasked with arguing against the most plausible answer. This is applied via a system-prompt constraint on that participant's session, not a separate agent type.

### 4.3 Deliberation Log Entry Format

Each log entry stored in the Channels tab:

| Field | Content |
| --- | --- |
| `round_id` | Unique identifier |
| `opened_at` | ISO-8601 timestamp |
| `question` | The question posed |
| `participants` | List of `{role_name, model_used}` |
| `arguments` | Per-participant: `{role, position_summary, key_points[], confidence}` |
| `decision` | Orchestrator's final decision |
| `reasoning` | Which arguments influenced the decision and why |
| `token_budget` | Declared / actual used |
| `truncated` | `true` if any argument was cut short |

### 4.4 Communication Visibility

The deliberation log is the structured form of "inter-worker chat" visible to the user. Unlike unstructured chat, it provides:

- Clear attribution per argument (which worker said what)
- Explicit decision and reasoning (what the orchestrator concluded and why)
- Searchable, filterable history of all deliberation rounds in the office session

This is the office's public reasoning surface — not private, not ephemeral.

## 5. Implementation Notes

1. Parallel argument generation uses the wave-based parallel execution model from `l2-orchestration.md`.
2. Log entries share the SQLite backing store with the inbox (`l2-inbox.md`) but use a distinct message type to support type-filtered queries.
3. The devil's advocate designation is a per-round setting; the orchestrator may skip it for low-stakes decisions.

## 6. Drawbacks & Alternatives

**Alternative: open group discussion** — participants read each other's prior arguments and iterate. Violates DL-1; cross-contamination causes premature convergence.

**Alternative: majority vote** — the option with the most votes wins. Rejected: majority vote discards argument quality. Minority insights that the orchestrator judges significant would be lost.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORCHESTRATION]` | `.design/main/specifications/l1-orchestration.md` | Delegation and budget used to run rounds |
| `[INBOX]` | `.design/main/specifications/l2-inbox.md` | Backing store for log entries |
| `[OFFICE-MODEL]` | `.design/main/specifications/l1-office-model.md` | Orchestrator authority (DL-3) and role hiring (DL-2) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — DL-1…DL-5, round lifecycle, participant selection, log format, communication visibility |

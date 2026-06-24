# Lookahead Planning

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Lookahead planning is the office's capability to simulate the consequences of a proposed action sequence N steps ahead before committing to it. Like a chess player calculating lines before moving, an agent with lookahead evaluates likely downstream effects and reverts to a safer path when the simulation reveals unacceptable consequences. The capability applies to high-stakes decisions where actions are expensive to reverse, and is bounded by a declared token budget to remain economically viable.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) — the orchestrator invokes lookahead before high-impact delegation decisions
- [l1-deliberation.md](l1-deliberation.md) — deliberation produces arguments; lookahead simulates the consequences of each option
- [l1-office-control.md](l1-office-control.md) — irreversible actions (branch merge, deploy, file deletion) are primary lookahead triggers
- [l1-quality-standards.md](l1-quality-standards.md) — lookahead is one quality gate before high-impact operations

## 1. Motivation

Autonomous agents acting without lookahead are reactive: they commit to the next step without considering whether it leads toward a viable end state. A small early decision (wrong branch strategy, premature schema migration, deleting a shared module) can cascade into hours of remediation. Lookahead provides a cheap simulation pass — a "thinking step" before an expensive or irreversible action step — trading tokens for avoided rework.

The cost is bounded: lookahead is not applied to every action (prohibitively expensive) but is triggered by a declared set of high-impact action types. The depth N of the lookahead is bounded by a budget.

## 2. Constraints & Assumptions

- Lookahead is probabilistic, not deterministic: it reasons about likely consequences under an assumed model, not a guaranteed simulation of external systems.
- The simulation runs in the orchestrator's context, using the office's current knowledge state. It does not execute real commands.
- Lookahead depth (N steps) and token budget are declared per trigger category, not per individual call.
- A failed simulation (budget exhausted before a conclusion) falls back to the standard approval gate (ORC-9) rather than blocking execution entirely.

## 3. Core Invariants

- **LP-1 Trigger-scoped application**: lookahead MUST only fire for actions explicitly listed in the trigger catalog (§4.2). Applying it to every action is not permitted — cost must be proportional to reversibility risk.
- **LP-2 No real execution during simulation**: the simulation MUST NOT issue real tool calls, file writes, process spawns, or network requests. The simulation is a "what if" reasoning pass over the agent's knowledge model. Simulated actions produce only predicted consequences, never actual side effects.
- **LP-3 Budget-bounded depth**: lookahead depth and token budget are declared upfront and enforced as hard limits. A simulation that exceeds the budget is terminated; the agent receives whatever partial conclusion was reached (LP-5 governs the response).
- **LP-4 Conclusion before commit**: before executing a high-impact action, the orchestrator MUST produce a lookahead conclusion (even if partial). The conclusion either confirms the action, proposes a modified action, or escalates to a human gate. The action is committed only after the conclusion is produced.
- **LP-5 Graceful fallback**: if lookahead budget is exhausted without a conclusive result, the action defaults to the standard approval gate (ORC-9 from `l1-orchestration.md`). Lookahead failure MUST NOT cause the action to silently proceed unchecked.
- **LP-6 Log persistence**: every lookahead conclusion (including partial ones) is appended to the office's decision log before the action is committed. The log entry records: trigger action, simulation depth reached, conclusion, and the action actually taken. The log is append-only.

## 4. Detailed Design

### 4.1 Simulation Process

```text
[REFERENCE]
Trigger: high-impact action A proposed by a worker or automation

Step 1  — EXTRACT: capture the current office state snapshot relevant to A
Step 2  — PROJECT: for each depth step d = 1..N:
            a. Simulate "what happens if A is executed in state S_d?"
            b. Derive predicted S_{d+1} under the agent's model
            c. Evaluate: is S_{d+1} acceptable? Does it lead toward goal?
               YES → continue to d+1
               NO  → record failure path; propose alternative A' or escalation
Step 3  — CONCLUDE:
            - CONFIRM(A)       → proceed with original action
            - MODIFY(A → A')   → replace A with A' before execution
            - ESCALATE         → route to approval gate (ORC-9); do not execute A
Step 4  — LOG: append {trigger, depth_reached, conclusion, action_taken} to decision log
Step 5  — EXECUTE (only if CONFIRM or MODIFY): commit the action (original or modified)
```

### 4.2 Trigger Catalog

Lookahead is triggered by these action categories. Depth and budget are per-category defaults; individual offices may configure overrides in Local Settings → Office.

| Category | Default depth N | Rationale |
| --- | --- | --- |
| Branch merge / PR creation | 3 | Merges can conflict with unreviewable changes; reversal is expensive |
| Schema migration | 4 | Schema changes cascade across all data; rollback is painful |
| File / directory deletion | 3 | Deleted files may be referenced downstream; undelete is not guaranteed |
| Dependency version upgrade | 3 | Breaking changes may be invisible in compilation but detonate at runtime |
| Security policy change | 5 | Policy weakening creates permanent exposure; hard to audit retrospectively |
| Mass refactor (>30% of files) | 3 | Wide scope amplifies small mistakes; incremental is always safer |
| Office architecture change | 5 | Role/topology restructuring affects all active delegations |

### 4.3 Conclusion Types

| Conclusion | Meaning | Next action |
| --- | --- | --- |
| `CONFIRM(A)` | Simulation found no blocking consequences within depth N | Execute A |
| `MODIFY(A → A')` | A better-bounded variant A' avoids identified risks | Execute A' (log original A) |
| `ESCALATE` | Simulation found unacceptable consequences; confidence too low to self-decide | Route to ORC-9 approval gate |
| `BUDGET_EXHAUSTED` | Simulation ran out of budget before conclusion | Route to ORC-9 approval gate (LP-5) |

### 4.4 Cost Control

The token cost of lookahead is controlled through three mechanisms:

1. **Trigger scoping** (LP-1): only declared high-impact categories are eligible.
2. **Depth budget**: small N (2–5) captures the most impactful near-term consequences without exploring the full decision tree.
3. **Early termination**: if any simulation step produces an unambiguously blocking consequence (e.g., deleting a directory referenced by 40 active imports), the simulation terminates early with ESCALATE rather than continuing to N.

## 5. Implementation Notes

1. The simulation step (Step 2) uses the orchestrator's context model and available static analysis — it does not require a separate "simulation engine" process.
2. Depth N=3 for most categories is sufficient: the most impactful consequences of most actions manifest within 3 derived state transitions. Deeper N yields diminishing returns at linear cost.
3. The decision log feeds the self-improvement loop (`l2-self-improvement.md`): lookahead conclusions that were overridden by actual outcomes provide calibration data.

## 6. Drawbacks & Alternatives

**Alternative: apply lookahead to every action** — maximum safety, prohibitive cost. Rejected; LP-1 scopes to high-impact only.

**Alternative: always escalate high-impact actions to humans** — no simulation cost, but breaks the autonomous office model. Lookahead allows the agent to self-resolve many cases without human involvement.

**Alternative: no lookahead, rely on rollback** — simpler, but rollback after the fact costs more than prevention and may not always be possible (schema migrations, external API calls, published artifacts).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORCHESTRATION]` | `.design/main/specifications/l1-orchestration.md` | ORC-9 approval gate that serves as fallback (LP-5) |
| `[DELIBERATION]` | `.design/main/specifications/l1-deliberation.md` | Multi-perspective argument generation that feeds into lookahead conclusions |
| `[QUALITY]` | `.design/main/specifications/l1-quality-standards.md` | Quality gate context for high-impact actions |
| `[SELF-IMPROVE]` | `.design/main/specifications/l2-self-improvement.md` | Consumes decision log entries for calibration |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — LP-1…LP-6, simulation process, 7-category trigger catalog, conclusion types, cost control |

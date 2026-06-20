# Context Management

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-orchestration.md

## Overview

Concrete context-window management for agent sessions: adaptive input-token budget that auto-scales to a model's discovered context window, a trim cascade that drops low-priority content before high-priority content, and an LLM-driven compaction step that summarizes older conversation turns when utilization exceeds 85%. A message `_protected` field prevents active documents from being trimmed.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) - ORC-3 context isolation that this spec concretizes.
- [l2-agent-session.md](l2-agent-session.md) - TurnContext and IterationBudget; context management runs as a prologue step.
- [l2-model-router.md](l2-model-router.md) - Context window size is a routing signal; this spec reads the discovered window.
- [l2-context-router.md](l2-context-router.md) - Memory and rules injection; these system messages are subject to trim cascade.

## 1. Motivation

A production agent system must run profitably on any model — a 4k local model and a 1M cloud model need different budgets. Without adaptive scaling, a 1M-context model silently uses only 6k tokens (the safe default), never leveraging its capacity. Without a trim cascade and compaction, long sessions fail with `context_length_exceeded` at provider boundaries. This spec prevents both failure modes.

## 2. Constraints & Assumptions

- The input token budget is a soft limit: the session loop trims before sending, not by truncating the actual API call.
- Context window size is discovered from model metadata; when unknown, conservative defaults apply.
- Compaction summarizes via the same LLM (or a cheaper utility model); if the summary call fails, the session continues without compaction (degrade gracefully).
- Messages marked `_protected` are never dropped regardless of budget pressure.
- System messages that are `research_spinoff_from` (seeded research reports) are treated as essential alongside the first system message.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| ORC-3 Context isolation | Trim and compact are session-scoped; no cross-session context leaks. |
| ORC-5 Tool sandbox | Sanitize orphaned `tool` messages before trimming to avoid provider API errors. |

## 4. Detailed Design

### 4.1 Adaptive input token budget

```text
[REFERENCE]
compute_input_token_budget(
  configured: u32,         // user-configured value (or default 6000)
  context_length: u32,     // discovered model context window (0 = unknown)
  explicit: bool,          // true if user set a NON-default value
  default: u32 = 6000,
  headroom: f64 = 0.85,
  hard_max: u32 = 200_000
) -> u32
```

Rules:
- **Explicit user budget:** honoured exactly, only clamped to the model's window when that window is known. Never further capped by `hard_max` (user's deliberate choice wins).
- **Auto (not explicit, window known):** `min(floor(context_length * headroom), hard_max)` — fills up to 85% of the model's window, capped at 200k so very large windows don't send pathologically large prompts.
- **Auto (not explicit, window unknown):** use `configured` if set, else `default` (conservative 6k). **Do not scale off an unproven window.**

The "materialized default" problem: the settings-save path writes the default into the config file, making a stored default look like an explicit choice. Guard against this: a stored value equal to `DEFAULT_BUDGET` is still treated as auto.

### 4.2 Trim cascade

When estimated token count exceeds `context_length - reserve_tokens` (reserve: 512), apply in order — stop when under budget:

```text
Priority (highest = drop LAST):
  0. _protected messages — never dropped
  1. First system message (preset prompt) — never dropped
  2. Research-primer system messages — never dropped
  3. Recent 10 conversation turns — dropped last
  4. Extra system messages (memory, RAG injections) — dropped first
  5. Truncate first system message (> 2000 chars → truncate + notice)
  6. Drop older conversation turns (keep recent 10)
  7. Truncate current user message (never drop; add head+tail notice)
  8. Truncate tool_call arguments in oversized assistant messages
```

Step 4 is tried before step 6 because injected memory/RAG messages are cheaper to lose than conversation turns — the user's intent history matters more than auxiliary context.

**Tool message sanitization** (runs before trim):

- Drop `role: "tool"` messages with no preceding `assistant.tool_calls` (orphans from front-trimming).
- Drop or strip `tool_calls` from assistant messages whose tool responses were all trimmed away (dangling calls trigger provider validation errors).

### 4.3 Compaction

When session utilization reaches `COMPACT_THRESHOLD = 85%` of context window:

```text
[REFERENCE]
maybe_compact(session, endpoint_url, model, messages) -> (messages, context_length, was_compacted)
```

Algorithm:
1. Estimate token usage. If `usage / context_length < 0.85`, skip.
2. If fewer than 4 conversation turns, skip (not worth compacting).
3. Split conversation at midpoint: `older = convo[:n//2]`, `recent = convo[n//2:]`.
4. Build compaction text from `older` (truncate each message to 2000 chars).
5. Call the utility model (or the session model if no utility model is configured) with the self-summary prompt.
6. On success: replace `older` with a `role: "system"` summary message; keep `recent` intact.
7. On failure: return original messages unchanged (`was_compacted = false`).

Self-summary format (structured, dense, ≤1000 tokens):

```text
[REFERENCE]
## Conversation Summary
**Turns summarized:** {count}  |  **Compactions so far:** {n}

### User Goal
<one sentence>

### What Was Done
- <completed actions, specific values: paths, names, versions, URLs>

### Current State
<last discussed topic, current system/code state>

### Pending / Next Steps
- <remaining work, blockers, open questions>

### Key Context
- <constraints, preferences, specific values not to lose>
```

The session history is updated in-place after compaction; the summary message carries `{compacted: true, summarized_count: n}` in its metadata.

### 4.4 Message protection

Any message with `_protected: true` is excluded from the trim cascade entirely. This flag is set on:
- The "active document" message injected when the user has a document open for editing.
- Other caller-defined must-keep injections.

Protected messages are subtracted from the available budget first, then the remaining budget governs trim decisions.

### 4.5 Command surface

Context management runs automatically as a prologue step in the session loop; there are no user-facing commands. Observability:

| Signal | Where |
| --- | --- |
| Compaction occurred | Structured log at INFO: `{used} -> {new_used} tokens, {older_count} messages summarized` |
| Trim applied | Structured log at INFO: `Trimmed to {n} tokens ({m} messages)` |
| Budget computed | Per-turn metric: `{configured, context_length, effective_budget}` |

## 5. Drawbacks & Alternatives

- **Compaction loses exact tool-call history:** the summary captures what happened but not the raw tool inputs/outputs. Acceptable — the summary preserves the facts needed to continue; exact replay is not a goal.
- **85% compaction threshold may trigger too early on rapid-fire turns:** a single very long turn can cross 85% before the older history is worth summarizing. Mitigation: the "< 4 turns" guard prevents premature compaction.
- **Utility model for compaction may have different quality:** if the utility model is much smaller than the session model, summaries may be coarser. Fallback: use session model if no utility model configured.
- **Alternative — fixed sliding window:** discard oldest N turns. Rejected: loses information abruptly and has no awareness of which messages are high/low priority (RAG injection vs user intent).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORC]` | `.design/main/specifications/l1-orchestration.md` | ORC-3 context isolation |
| `[SESSION]` | `.design/main/specifications/l2-agent-session.md` | TurnContext + IterationBudget |
| `[ROUTER]` | `.design/main/specifications/l2-model-router.md` | Context window discovery |

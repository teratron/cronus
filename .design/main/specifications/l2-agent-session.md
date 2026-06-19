# Agent Session Loop

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-orchestration.md, l1-routing.md

## Overview

The concrete structure of a single agent turn: the per-turn context object that the prologue produces, the iteration budget that caps autonomous tool calls, the context compression thresholds, and the pluggable context engine interface that decides when and how to compact the message list.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) - Orchestration model (delegation, judge, budget loop).
- [l2-model-router.md](l2-model-router.md) - Model selection for each turn's API call.
- [l2-context-router.md](l2-context-router.md) - Memory/session context injected in the turn prologue.
- [l2-model-error-recovery.md](l2-model-error-recovery.md) - Error taxonomy and recovery actions per iteration.
- [l2-learning-loop.md](l2-learning-loop.md) - Post-turn review hook gated by `should_review_memory`.

## 1. Motivation

An agent turn drives prompt construction, the tool-calling loop, error recovery, and post-turn hooks. Without structure, all of this is a single monolithic function. Extracting a `TurnContext` (the prologue's output, consumed by the loop) and an `IterationBudget` (a thread-safe ceiling on tool calls) separates the setup phase from the execution phase, enables independent testing, and makes the orchestration invariants enforceable in code.

## 2. Constraints & Assumptions

- A "turn" is one user message plus all model calls and tool calls needed to produce a response.
- The prologue runs once per turn; the tool-calling loop may run many times.
- Subagents receive independent budgets; a parent agent does not donate its remaining budget to a subagent.
- Context compression may create a new session (resetting `conversation_history`); that side effect is the prologue's last step.
- Input sanitization (surrogate stripping, injection blocking) always runs in the prologue, never later.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| ORC-4 Budget | `IterationBudget.consume()` is called before each tool-call loop iteration; exhaustion aborts with a user-visible message. |
| ORC-6 Context isolation | Each subagent gets its own `TurnContext`; no shared mutable turn state crosses delegation boundaries. |
| ORC-8 Briefing | `TurnContext.should_review_memory` gates the post-turn memory/skill review hook. |
| SEC-1 Input validation | Sanitization of the user message (surrogates, non-ASCII in identifiers) always runs before any LLM call. |

## 4. Detailed Design

### 4.1 TurnContext — per-turn prologue output

The prologue builds one `TurnContext` per user message and hands it to the loop. The loop only reads from it (and appends to `messages`). No turn state is re-built mid-loop.

```text
[REFERENCE]
TurnContext {
  user_message: String,              // Sanitized inbound (surrogates stripped)
  original_user_message: Any,        // Clean copy for transcripts and memory queries
  messages: Vec<Message>,            // Working message list (loop appends here)
  conversation_history: Option<Vec<Message>>, // May be reset by compression
  active_system_prompt: Option<String>,        // May be rebuilt by compression
  effective_task_id: String,
  turn_id: String,                   // UUID, unique per turn
  current_turn_user_idx: usize,      // Index of the user message in `messages`
  should_review_memory: bool,        // Whether post-turn learning loop fires
  plugin_user_context: String,       // Context injected by pre_llm_call hooks
  ext_prefetch_cache: String,        // External-memory prefetch, reused across iterations
}
```

### 4.2 IterationBudget — autonomous call ceiling

Each agent instance (parent or subagent) holds one `IterationBudget`. Budgets are independent — total iterations across a parent and all its subagents may exceed the parent's own cap.

```text
[REFERENCE]
IterationBudget {
  max_total: u32,         // Parent default: 90; subagent default: 50
  used: AtomicU32,
}
impl IterationBudget {
  consume() -> bool    // True if allowed; increments counter. Returns false when exhausted.
  refund()             // Reclaim one iteration (for non-LLM tool calls, e.g. script execution)
  remaining() -> u32
}
```

The `refund()` mechanism keeps purely mechanical tool calls (data lookups, script execution, file reads) from depleting the budget against conversational or planning turns.

### 4.3 Context engine interface

One context engine is active per session. Engines are pluggable via the extension registry (kind: `plugin`, interface: `ContextEngine`). The default engine is `Compressor` (LLM-based summarization).

```text
[REFERENCE]
trait ContextEngine {
  name() -> &str

  // Called after every LLM response with normalized usage data
  update_from_response(usage: Usage)

  // Return true if compaction should fire this turn
  should_compress(prompt_tokens: u32) -> bool

  // Compact the message list and return the shortened version.
  // The engine MAY summarize, build a DAG, or do anything else.
  // The returned list must be a valid message sequence.
  compress(
    messages: Vec<Message>,
    current_tokens: u32,
    focus_topic: Option<&str>,
  ) -> Vec<Message>

  on_session_start()
  on_session_end()

  // Readable by the caller for display and preflight checks
  threshold_percent: f32   // Default 0.75 — fire when prompt >= 75% of context window
  protect_first_n: u32     // Default 3 — preserve first N non-system messages verbatim
  protect_last_n: u32      // Default 6 — preserve last N messages verbatim
  context_length: u32
  compression_count: u32
}
```

The `protect_first_n` and `protect_last_n` fields are in addition to the system prompt, which is always implicitly preserved.

### 4.4 Prologue steps (order)

1. Guard stdio for background-thread safety.
2. Sanitize user message: strip surrogates and non-ASCII in identifiers.
3. Reset iteration counter; assign `turn_id` (UUID).
4. Restore cached system prompt, or rebuild it if missing or stale.
5. Run `pre_llm_call` plugin hooks; collect `plugin_user_context`.
6. Preflight: if `context_engine.should_compress()`, compress now (may create new session).
7. Prefetch external memory; cache result in `ext_prefetch_cache`.
8. Set `should_review_memory` based on turn flags and memory-nudge counter.
9. Return `TurnContext`.

### 4.5 Loop exit conditions

The tool-calling loop terminates when:
- The model returns a final answer (no pending tool calls).
- `IterationBudget.consume()` returns `false` (budget exhausted).
- An error is classified as non-retryable and no fallback is available.
- A stop signal (user interrupt, timeout) is received.

## 5. Drawbacks & Alternatives

- **Independent subagent budgets:** total iterations may exceed the parent's cap. Justified — a subagent solving a delegation should not penalize the parent's own remaining turns.
- **Fixed protect_first_n/last_n defaults:** simple and predictable; context engines may override via the interface.
- **Alternative — share budget across parent and all subagents:** makes delegation unpredictable and hard to debug; rejected.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORC]` | `.design/main/specifications/l1-orchestration.md` | Budget and isolation invariants |
| `[ROUTER]` | `.design/main/specifications/l2-model-router.md` | Model selection per turn |
| `[RECOVERY]` | `.design/main/specifications/l2-model-error-recovery.md` | Error handling per iteration |
| `[LEARN]` | `.design/main/specifications/l2-learning-loop.md` | Post-turn review hook |

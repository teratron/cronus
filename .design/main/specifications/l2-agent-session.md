# Agent Session Loop

**Version:** 1.0.4
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
- [l2-agent-autonomy.md](l2-agent-autonomy.md) - `ActionTracker` and approval gate interact with the `action_budget_stop` hook (§4.7).

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

### 4.6 Tool-call loop engine

The tool-calling loop is structured around four pluggable seams that allow testing and extension without modifying the core loop.

#### Seams

```text
[REFERENCE]
ToolSource
  // Provides the tool catalog for the session. Built once at session start.
  // Rebuilt only on explicit extension install/uninstall, never mid-turn.
  tools() -> Vec<ToolDefinition>
  reload()  // called by extension registry on activation/deactivation

ProgressReporter
  // Receives streaming tokens and tool events for display.
  on_token(text: &str)
  on_tool_start(tool_name: &str, params: &ToolParams)
  on_tool_end(tool_name: &str, result: &ToolResult)

TurnObserver
  // Post-tool-call observation hook (used by the learning loop).
  observe(tool_name: &str, result_summary: &str, duration_ms: u64)

CheckpointStrategy
  // Decides when to persist turn state to disk.
  should_checkpoint(tool_calls_since_last: u32, elapsed_ms: u64) -> bool
  // Default: every 10 tool calls OR every 60 seconds, whichever comes first.
```

#### KV-cache stability

The system prompt (persona, workspace rules, skill preambles) is built **once per session** at session start and kept identical for all turns. It is never modified after construction. Dynamic context — memory recalls, updated card state, injected tool results — is added as `role: "user"` messages so that the immutable system-prompt prefix remains intact in the provider's KV cache. This avoids full-context re-encoding on every turn.

The `ToolSource` catalog follows the same rule: it is built once and only rebuilt on explicit extension changes, not on each tool call. New tools added mid-session by extension installs take effect at the next turn boundary.

#### Oversized result summarizer

When a tool result exceeds `OVERSIZED_RESULT_THRESHOLD` tokens (default: 8 000), the loop detours to a summarizer sub-agent before passing the result to the main model:

```text
[REFERENCE]
oversized_result_handler(tool_name, result, threshold):
  if token_count(result) <= threshold:
    return result
  summary = summarizer_subagent.run(
    prompt  = "Summarize the following tool result for {tool_name}:",
    content = result,
    budget  = 500 tokens
  )
  return summary
```

**Circuit breaker**: after 3 consecutive summarizer failures (timeout, model error, output exceeds budget), the loop stops calling the summarizer for this turn and instead truncates the raw result to `threshold` tokens with a prepended warning:

```text
[REFERENCE]
"[RESULT TRUNCATED: summarizer unavailable after 3 attempts. Showing first {threshold} tokens.]"
```

The circuit breaker resets at the start of the next turn.

#### Missing-command self-healing

If the model produces a tool call for a name not present in the current `ToolSource` catalog, the loop attempts self-healing before returning an error:

1. Check whether a known skill provides the missing tool name (skill registry lookup).
2. If found: auto-activate the skill, reload the `ToolSource`, re-execute the tool call.
3. If not found: return a structured `ToolNotFoundResult { tool_name, available_tools_hint }` so the model can recover gracefully.

Self-healing is attempted at most once per missing tool name per turn; a second miss for the same name returns the error immediately.

### 4.7 Turn lifecycle hooks

#### Stop hooks (inline, before the model sees results)

Stop hooks run synchronously within the tool-calling loop and can terminate the turn early:

```text
[REFERENCE]
StopHook: (context: &TurnContext, budget: &IterationBudget) -> Option<StopSignal>

StopSignal {
  reason:          "budget_exhausted" | "max_iter" | "action_cap_exceeded" | "interrupt",
  message_to_user: String,    // displayed to the user
  remaining:       Option<u32>,
}
```

Built-in stop hooks:

| Hook | Trigger | Signal reason |
| --- | --- | --- |
| `budget_stop` | `IterationBudget.consume()` returns `false` | `budget_exhausted` |
| `max_iter_stop` | Model-context iteration ceiling derived from context window | `max_iter` |
| `action_budget_stop` | `ActionTracker.record()` returns `ActionCapError` | `action_cap_exceeded` |

When any stop hook fires, the loop breaks immediately and `StopSignal.message_to_user` is returned as the turn's final response.

#### InterruptFence

A shared `Arc<AtomicBool>` is checked synchronously before three operations in each loop iteration:

1. Tool execution (before the tool is called).
2. Sub-agent spawn (before creating the child `TurnContext`).
3. Provider API call (before sending the prompt).

```text
[REFERENCE]
InterruptFence { flag: Arc<AtomicBool> }

impl InterruptFence {
  check() -> Result<(), Interrupted>
    // Returns Err(Interrupted) if the flag is set.
  signal()
    // Set by the user interrupt handler (Ctrl-C / TUI stop button).
}
```

When `check()` returns `Err`, the current operation is abandoned and the turn exits with `Interrupted` status. The fence is **cleared** (reset to `false`) at the start of each new turn so that a subsequent turn can proceed normally.

#### Post-turn hooks (background, after response is sent)

Post-turn hooks run as background tasks after the model's response is delivered. They do not block the turn or affect the response.

| Hook | Function |
| --- | --- |
| `archivist` | Writes a condensed turn summary to the session archive (`<ws>/sessions/<session_id>/archive.jsonl`) |
| `learning_loop` | Triggers the skill-review pipeline; gated by `TurnContext.should_review_memory` |
| `cost_log` | Appends token cost to the workspace budget ledger (`<ws>/budget/ledger.jsonl`) |
| `episodic_indexer` | Extracts episodic memory candidates from the turn and queues them for the memory store |

Post-turn hooks are fire-and-forget; failures are logged at WARN and do not surface to the user.

### 4.8 Text loop detection

When a model gets stuck in a repetitive output pattern (e.g., printing the same explanation, calling the same tool with the same arguments), the loop detector identifies the stall and injects a recovery prompt:

```text
[REFERENCE]
TEXT_LOOP_BUFFER_SIZE   = 6    // sliding window of recent assistant steps to compare
TEXT_LOOP_TRIGGER_COUNT = 3    // consecutive near-identical steps that trip detection
TEXT_LOOP_MAX_RECOVERY  = 2    // max recovery injections per turn before giving up

REPEATED_STEP_THRESHOLD = 3    // consecutive identical action signatures → nudge

normalizeForLoopDetection(text: String) -> String:
  // Lowercase, collapse whitespace, strip punctuation variants → canonical form
  // for comparison. Two steps that produce the same normalized form are "identical".

detectTextLoop(recent_steps: Vec<String>) -> bool:
  normalized = recent_steps.map(normalizeForLoopDetection)
  // Check whether the last TEXT_LOOP_TRIGGER_COUNT steps are all equivalent.
  return normalized[−TEXT_LOOP_TRIGGER_COUNT..].all_equal()
```

Recovery behavior:

1. **Mild recovery** (`RECOVERY_PROMPT_MILD`): inject as a new user message asking the model to try a different approach. Fired on first detection.
2. **Strong recovery** (`RECOVERY_PROMPT_STRONG`): more directive prompt instructing the model to stop its current approach and take an explicit alternative step. Fired on subsequent detections.
3. After `TEXT_LOOP_MAX_RECOVERY` recovery injections in a single turn, the loop is treated as non-recoverable; the turn is terminated with an `Interrupted` stop signal.

The `REPEATED_STEP_THRESHOLD` guard is a complementary check for action-signature repetition (same tool name + same argument hash), independent of text normalization.

### 4.9 Goal re-entry cap

When a turn is driven by an active goal (autonomous `/goal` execution), the main-loop re-entry counter bounds the total number of model re-entries per turn to prevent a never-satisfiable goal condition from burning tokens indefinitely:

```text
[REFERENCE]
MAX_GOAL_REACT = 12

goalReentryLoop(goal: Goal, turn_context: &mut TurnContext):
  for react_count in 0..MAX_GOAL_REACT:
    outcome = runTurn(turn_context)
    if outcome is Satisfied(goal): return outcome
    if outcome is Interrupted: return outcome
    if goal.evaluate(outcome) == NotMet:
      turn_context.append(nextGoalStep(goal, outcome))
    // continue to next re-entry
  // cap reached
  return HardStop {
    reason: "goal_react_cap_exceeded",
    message_to_user: "Goal not satisfied after {MAX_GOAL_REACT} attempts. Stopping.",
  }
```

`MAX_GOAL_REACT` (12) is higher than the plugin hook's `MAX_PRE_REACT` (3) because main-session goals are structurally larger tasks with more steps.

The cap is per-turn, not per-goal: a goal that spans multiple user turns accumulates re-entries independently across each turn.

### 4.10 Session server mode (daemon)

`cronus serve` starts a daemon that exposes agent sessions over HTTP, allowing remote
clients (IDE extensions, SDKs, web UIs) to drive Cronus without spawning a new process
per connection. The daemon maintains a pool of long-lived agent sessions; clients
connect via **ACP Streamable HTTP transport** — a single `/acp` endpoint mounted
alongside the existing REST session surface.

#### ACP HTTP endpoint

```text
[REFERENCE]
POST   /acp  — send a JSON-RPC request or notification (response: 200 on initialize;
                202 Accepted for all other methods — response is on the SSE stream)
GET    /acp  — open a long-lived SSE stream (connection-scoped or session-scoped)
DELETE /acp  — terminate this connection and cancel all its owned sessions

Auth: Authorization: Bearer <token> (same middleware as the REST surface).
SSRF guard (l2-security.md §4.4) applies to any outbound URL derived from client input.

JSON-RPC methods accepted on POST /acp:
  initialize               — mint Acp-Connection-Id; return protocol version + capabilities
  session/new              — create an agent session; response on connection stream
  session/load             — restore a saved session
  session/resume           — resume an existing session
  session/prompt           — send a prompt; response streams on the session stream
  session/cancel           — (notification) cancel the in-flight prompt
  session/close            — close a session
  session/list             — enumerate active sessions
  session/set_config_option — change model or approval mode (standard ACP method;
                              configId: "model" | "mode")
  _cronus/session/context   — get session context status (vendor extension)
  _cronus/session/update_metadata — patch session metadata
  _cronus/workspace/mcp     — MCP server status
  _cronus/workspace/skills  — loaded skills
  _cronus/workspace/env     — environment variable summary
  _cronus/session/heartbeat — extend the connection idle TTL (vendor extension)
  JSON-RPC response object  — client answer to an agent→client request (e.g. permission)
```

#### Identity layers

Three identity scopes are stacked to correlate connection, session, and message:

```text
[REFERENCE]
Acp-Connection-Id  (HTTP header) — transport binding; minted by the daemon at initialize.
                                   Must be present on all /acp requests after initialization.
Acp-Session-Id     (HTTP header) — required on session-scoped GET and session-scoped POSTs.
                                   Must match sessionId inside the JSON-RPC params.
sessionId          (JSON-RPC param) — inside method params; cross-checked against the header.
                                      Mismatch → INVALID_PARAMS error.

Session ownership: a connection may only subscribe to or prompt sessions it created via
session/new / session/load / session/resume.
Unowned session access → 403 Forbidden or INVALID_PARAMS.

Loopback detection: captured at the TCP layer on each request (127.0.0.0/8, ::1,
::ffff:127.*) and threaded into permission decisions ("local-only" policy).
```

#### Two-tier SSE streams

```text
[REFERENCE]
Connection-scoped stream: GET /acp with Acp-Connection-Id only (no Acp-Session-Id).
  Receives: session/new response, session/load response, connection-level notifications.

Session-scoped stream: GET /acp with Acp-Connection-Id + Acp-Session-Id.
  Receives: session/update notifications (streaming agent output),
            agent→client requests (session/request_permission),
            final prompt result frame ({id, result: {stopReason}}).

Framing: standard SSE format ("data: <JSON-RPC object>\n\n"); events carry a monotonic id.
Resume: Last-Event-ID header resumes from the connection's ring-buffer EventBus
        (same ring-buffer maxQueued = 256 as the REST surface).

Reconnect: a client may close and re-open the session-scoped SSE stream (e.g., network glitch)
without losing the in-flight prompt. The new stream attaches to the live session binding;
the old stream's onClose does NOT abort promptAbort if a fresh stream is now live.
```

#### Extension namespace

```text
[REFERENCE]
Standard ACP methods are never renamed.
Cronus-specific capabilities without a standard ACP equivalent use vendor-namespaced names:
  _cronus/<area>/<verb>   (ACP spec reserves the _ prefix for extensions)

Extensions are advertised at initialize under:
  agentCapabilities._meta: { "cronus": { ... } }

Clients feature-detect before use; unknown _cronus/* methods return method-not-found (-32601).
```

#### Connection lifecycle

```text
[REFERENCE]
ConnectionRegistry {
  max_connections: usize,   // cap: 64; 503 when exceeded (logged to stderr)
  idle_ttl_ms:     u64,    // default: 30 min; reap idle connections
}

touch() resets the idle clock. Called on:
  - any valid POST from this client
  - each SSE heartbeat write (prevents long-running prompts from being idle-reaped)

On connection reap: log entry + close SSE streams + abort promptAbort on each owned session.

Pre-attach buffer (frames queued before the client opens GET /acp):
  capped at 256 frames, drop-oldest policy.

AbortController lifecycle for session streams:
  Each GET /acp session stream installs a fresh AbortController — never reused.
  Closing the old stream AFTER the new one is attached; avoids aborting the
  new stream's event pump on reconnect.
```

#### Permission round-trip

```text
[REFERENCE]
When an in-flight prompt requires user approval:

Agent → client (on session-scoped SSE stream):
  { "id": "_cronus_perm_N", "method": "session/request_permission", "params": { ... } }

Client → agent (POST /acp, JSON-RPC response object):
  { "id": "_cronus_perm_N", "result": { "approved": true } }
  OR
  { "id": "_cronus_perm_N", "error": { ... } }

Daemon-allocated ids use the string form "_cronus_perm_N" (N monotonic) to avoid
collisions with client-supplied numeric ids.

Pending permissions are tracked in the connection registry by JSON-RPC id.
On connection teardown or session close: outstanding permissions are cancelled via
cancelAbandonedPermission — never left blocking an in-flight prompt.
On malformed client vote (result: {}): cancel the permission and release the mediator
so the agent is not permanently stalled.
```

#### Dual-transport policy

```text
[REFERENCE]
The /acp endpoint is ADDITIVE — it runs alongside the existing REST session routes.
Both transports share one underlying session engine; no state duplication.
A single session may be driven concurrently by REST and /acp clients (multi-client attach
is intentional; the bearer-token + single-workspace bind remains the trust boundary).

Disable: CRONUS_SERVE_ACP_HTTP=0 env var prevents mounting /acp routes at startup.

Migration path: once the ACP Streamable HTTP specification ratifies and SDKs ship,
the REST surface may be reframed as a thin compatibility shim over /acp (separate PR).
```

### 4.11 MCP tool session contract

[ADDED] The contract governing how Cronus exposes its capabilities to MCP clients and how
tool responses must be shaped to maintain agent trust across the session lifetime.

#### Server instructions as single source of truth

The `initialize` response carries the primary agent-facing guidance for all tools exposed
by the Cronus MCP server. This text is emitted once at session start and surfaced in the
agent's system prompt by every compliant MCP client — it is the **only** place tool guidance
lives. Instructions must NOT be duplicated in agent-config files (`CLAUDE.md`, `.cursor/rules/`,
`AGENTS.md`, etc.); writing duplicate blocks into those files means updating the server
changes nothing for agents that already have the cached copy.

Two variants are emitted based on workspace state:

```text
[REFERENCE]
ACTIVE   — emitted when a Cronus session is fully initialized and tools are available.
           Contains: tool-selection-by-intent, common chains, anti-patterns, limitations.
           `tools/list` returns the full exposed surface.

INACTIVE — emitted when the session cannot serve tools (e.g. workspace not yet ready,
           missing required configuration). Contains: one short note that the toolset is
           inactive this session plus one action the user can take.
           `tools/list` returns an EMPTY list — absence is the one signal an agent
           cannot misread. A non-empty but uniformly-failing tool list teaches
           abandonment through error accumulation.
```

Inactive variant MUST be ≤ 3 lines. The agent should not consume context explaining a
toolset it cannot use.

#### Error response taxonomy

MCP tool calls return one of two shapes. The choice is permanent for the session: a single
`isError: true` response early in a session observably causes agents to stop calling the
entire toolset for the rest of that session (abandonment learned from negative signal).
Reserve it accordingly.

```text
[REFERENCE]
ToolCallOutcome:
  Success {
    content: Vec<ToolContent>,
    isError: false,     // default; omit in wire format
  }

  GuidedRecovery {
    content: Vec<ToolContent>,   // plain-text guidance what to do instead
    isError: false,              // SUCCESS-SHAPED — agent continues calling tools
  }

  HardRefusal {
    content: Vec<ToolContent>,   // brief explanation (1-2 sentences)
    isError: true,               // reserved for "stop trying" conditions only
  }

GuidedRecovery is correct for:
  - Workspace not yet initialized (guide: user can run `cronus init`)
  - Symbol/entity not found (guide: try a search with fewer terms)
  - Tool parameters out of range (guide: valid range or alternative)

HardRefusal is correct for:
  - Security path refusals (sensitive directories blocked by policy)
  - Genuine server malfunctions (not recoverable by agent action)
  - A HardRefusal MUST NOT include retry guidance — abandonment is the intended outcome.
```

#### Input / output size limits

All tool call handlers enforce bounds before touching any state:

```text
[REFERENCE]
MAX_QUERY_INPUT_CHARS  = 10_000   // free-form text: query, task, symbol name
MAX_PATH_INPUT_CHARS   =  4_096   // path-like inputs: filePath, glob pattern
MAX_OUTPUT_CHARS       = 15_000   // total tool response to prevent context bloat

Inputs exceeding the limit → GuidedRecovery (not HardRefusal):
  "Input exceeds the maximum length of {N} characters. Please shorten the query."

Output exceeding the limit is truncated server-side at the nearest sentence/line
boundary with a trailing note: "… [output truncated to {MAX_OUTPUT_CHARS} characters]"
```

These bounds protect against hostile or buggy MCP clients sending oversized payloads (OOM,
runaway FTS scans) without treating it as an error that would teach abandonment.

#### Tool surface gating by context size

The number of tools exposed via `tools/list` may be reduced when the estimated prompt-context
cost of the full surface exceeds a threshold. This is the tool-profile concept:

```text
[REFERENCE]
ToolSurfaceProfile:
  full     — every registered tool available (default for interactive sessions)
  core     — 5–8 high-frequency tools only (search + context + recall)
  headless — 3–4 tools only (context retrieval + one write surface)

Switching triggers (automatic, not user-facing):
  full    → core:     accumulated_context_tokens >= PROFILE_DOWNGRADE_THRESHOLD
  core    → headless: tool_call_count >= PROFILE_HEADLESS_THRESHOLD

Constants:
  PROFILE_DOWNGRADE_THRESHOLD =  60_000   // estimated tokens in conversation so far
  PROFILE_HEADLESS_THRESHOLD  = 100       // total tool calls in the session
```

An agent consuming too many tokens on structural lookups gets a narrower surface that
preserves budget for high-value calls. The switch is transparent (tools simply disappear
from `tools/list`); no notification is sent.

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

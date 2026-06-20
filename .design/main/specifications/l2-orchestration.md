# Orchestration

**Version:** 1.0.2
**Status:** Stable
**Layer:** implementation
**Implements:** l1-orchestration.md

## Overview

The concrete coordination mechanics: how the orchestrator delegates via assigned board cards, how agents message each other, how executors are isolated, how the independent judge and the budget circuit-breaker bound a `/goal` run, how the topology adapts as the office grows, and how orchestration state persists for resume.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) - The protocol this implements.
- [l2-kanban-board.md](l2-kanban-board.md) - Delegation creates/assigns cards; `done` consumes quality gate results.
- [l1-office-model.md](l1-office-model.md) - Roles and adaptive staffing the orchestrator delegates to (concrete role catalog specified separately).
- [l2-quality-pipeline.md](l2-quality-pipeline.md) - Gate results the orchestrator requires before `done`.
- [l2-cli.md](l2-cli.md) - `/goal`, `plan`, `task`, `run`, `status` entry points.
- [l2-agent-autonomy.md](l2-agent-autonomy.md) - Approval gate that governs high-impact orchestrator decisions and agent hires.
- [l2-trigger-triage.md](l2-trigger-triage.md) - Triage hands `SpawnOrchestrator` decisions to this layer's goal-execution loop.

## 1. Motivation

The protocol needs concrete, local, resumable mechanics: a delegation channel (the board), an inter-agent channel (messages), isolation (per-agent sessions), a judge call, and budget accounting — all file-backed so an unattended run survives restarts.

## 2. Constraints & Assumptions

- Delegation rides on the existing board; no parallel task store.
- Executors run in isolated sessions; only summaries return to the orchestrator.
- The judge is a separate evaluation (may use a different/cheaper model) than the executor.
- Budgets come from configuration (per-run and per-agent).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| ORC-1 One orchestrator | The office `manager` (from `<ws>/config.json`) is the sole coordinator; it issues delegations, not implementations. |
| ORC-2 Adaptive topology | When direct reports or domain spread cross a threshold, the orchestrator hires a sub-manager and re-parents agents under it. |
| ORC-3 Intent → board | Plans decompose into board cards (`<ws>/kanban/`) assigned to roles. |
| ORC-4 Delegate/monitor | The orchestrator watches card states and re-assigns/unblocks; it never self-assigns implementation. |
| ORC-5 Context isolation | Each delegated card runs in its own agent session (`<ws>/sessions/`); only a result/summary returns. |
| ORC-6 Judged termination | A `/goal` loop calls an independent judge to decide completion before finishing. |
| ORC-7 Budget circuit-breaker | A per-run budget/iteration counter pauses the loop when exhausted (ties to per-agent budgets). |
| ORC-8 Synchronization | Scheduled briefing routines reconcile agent state into shared office state. |
| ORC-9 Approval gate | High-impact actions require a recorded approval (sub-manager or client) before execution. |
| ORC-10 Resumable | Goal/plan/delegation state persists (board + office state) and reloads on restart. |

## 4. Detailed Design

### 4.1 Delegation and messaging

- **Delegation:** the orchestrator creates a board card with an `assignee` (a hired role) and the task reference; a missing role triggers a hire (role catalog).
- **Messaging:** agents exchange messages (a mailbox) for hand-offs and questions; the orchestrator is the hub for cross-role coordination.
- **Shared task list:** the board is the single shared work list; claiming/assignment is atomic to avoid two agents grabbing the same card.

### 4.2 Context-isolated execution

Each delegated unit runs in a fresh agent session with only the context it needs (its card, relevant memory, role persona). Intermediate reasoning stays in that session; the orchestrator receives a concise result and the card's new state (ORC-5) — preventing context rot over long runs.

### 4.3 `/goal` run with judge + budget

```mermaid
graph TD
    START[/goal text/] --> INIT[init run: budget, iteration=0]
    INIT --> LOOP[plan -> delegate -> execute]
    LOOP --> JUDGE[independent judge evaluates goal]
    JUDGE -->|met| FINISH[finish + report]
    JUDGE -->|not met| DEC{budget/iterations left?}
    DEC -->|yes| LOOP
    DEC -->|no| PAUSE[hard-stop: pause + report why]
```

- **Judge:** a separate evaluation step (distinct from the executor, may use a cheaper model) returns met / not-met with reasoning; only it can end the run as "done" (ORC-6).
- **Budget:** the run carries a token/cost/iteration budget; on exhaustion it pauses safely and reports remaining work (ORC-7).

### 4.4 Adaptive topology mechanism

The orchestrator tracks team size and domain spread; crossing a threshold causes it to hire a sub-manager, assign it a department, and re-parent the relevant agents' `reportsTo`. Shrinking reverses this (release sub-manager, flatten). <!-- TBD: concrete thresholds for promoting/flattening sub-managers -->

### 4.5 Command surface

The `/goal` entry point is the autonomy trigger; planning/execution reuse existing core commands.

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| start autonomous goal | `cronus goal <text>` | `/goal <text>` | `orchestrator.goal(text) -> Run` |
| view run progress | `cronus status` | `/status` | `orchestrator.status() -> RunStatus` |
| stop a run | `cronus goal stop` | `/goal stop` | `orchestrator.stop() -> void` |

### 4.6 Agent tier hierarchy

Agents are classified into three tiers at definition time. The tier determines the allowed toolsets, iteration budget defaults, and workspace scope.

```text
[REFERENCE]
AgentTier: "chat" | "reasoning" | "worker"

AgentDefinition {
  name:             String,
  tier:             AgentTier,
  role_id:          Option<String>,          // links to a hired role in the role catalog
  toolsets:         Vec<String>,             // declared toolset names
  iteration_budget: Option<u32>,             // overrides tier default
  workspace_scope:  "session" | "execution" | "isolated",
}
```

| Tier | Typical use | Default iteration budget | Toolset restrictions | Workspace scope |
| --- | --- | --- | --- | --- |
| `chat` | Short interactive turns, clarifications, brief Q&A | 10 | Read-only tools only; `shell` and `code_execution` forbidden | `session` |
| `reasoning` | Planning, analysis, multi-step problem decomposition | 50 | All tools except `code_execution`; `shell` allowed read-only | `session` or `execution` |
| `worker` | Full implementation, file editing, command execution | 90 | All toolsets; must have an execution workspace | `execution` or `isolated` |

#### Static loader validation

At agent definition load time (extension registry activation, workspace bootstrap), each `AgentDefinition` is validated against its tier:

- `chat` agents declaring `shell` or `code_execution` toolsets are rejected with a hard load error.
- `worker` agents without an `execution` or `isolated` workspace scope are rejected.
- Unknown toolset names are rejected (no implicit fallback).

Validation failures are logged as `AgentDefinitionError` and prevent the agent from being loaded. They do not affect other agents in the same workspace.

#### Runtime spawn depth limit

Agents can spawn sub-agents, which can spawn further sub-agents, but the chain is capped:

```text
[REFERENCE]
MAX_SPAWN_DEPTH: u32 = 3
```

When an agent at depth 3 attempts to spawn a child, the call returns `MaxSpawnDepthError` to the parent. The error is surfaced in the parent's tool result and logged. The orchestrator is always at depth 0; its direct delegates are at depth 1, and so on.

#### Toolkit action ranking

When a `reasoning` or `worker` agent has a large tool catalog (> 50 entries), an action-ranking pass narrows the effective tool list before each provider call:

```text
[REFERENCE]
rank_tools(task_description: &str, tools: &[ToolDefinition], top_n: u32) -> Vec<ToolDefinition>:
  scores = []
  for tool in tools:
    verb_score  = verb_overlap(task_description, tool.name)   // CPU-only: match leading verb
    token_score = token_overlap(task_description, tool.description)  // unigram overlap
    scores.push((tool, verb_score + token_score))
  scores.sort_by(|a, b| b.score.partial_cmp(&a.score))
  return scores[..top_n].map(|s| s.tool)
```

- **Default `top_n`**: 20.
- **Ranking is CPU-only** (no LLM call): verb detection extracts the leading verb from the task description and checks if it appears in the tool name; token overlap counts shared unigrams between the task description and the tool's one-line description.
- Ranked tools are placed first in the tool list sent to the provider; remaining tools are omitted.
- Action ranking runs only when the catalog exceeds the threshold; smaller catalogs are sent in full.

### 4.7 Permission ruleset evaluation

Tool calls and agent actions that require user approval go through a ruleset evaluator.
Rulesets are ordered lists of rules; **the last matching rule wins** (not the first).
This allows coarse-grained allow/deny earlier in the list to be overridden by fine-grained
rules appended by the current context.

```text
[REFERENCE]
PermissionRule {
  permission: Pattern,   // e.g. "fs:write:*"
  pattern:    Pattern,   // e.g. "/home/**"
  action:     "allow" | "deny" | "ask"
}

evaluate(permission: String, path: String, rulesets: Vec<Ruleset>) -> PermissionResult:
  // merge rulesets in order; then findLast matching rule
  merged = concat(rulesets)
  rule = merged.findLast(|r|
    Wildcard.match(permission, r.permission) AND Wildcard.match(path, r.pattern)
  )
  return rule?.action ?? "ask"   // default: ask the user

// "ask" triggers an async approval cycle:
PendingApproval {
  id:       PermissionId,
  info:     ApprovalRequest,
  deferred: Deferred<void, RejectedError | CorrectedError>,
}
```

#### Deferred async approval

When `evaluate()` returns `"ask"`, a `Deferred` is created and the pending approval is
registered. The agent suspends at the approval point; execution resumes when the user
responds.

```text
[REFERENCE]
request_approval(request: ApprovalRequest) -> Effect<void, RejectedError>:
  deferred = Deferred.make()
  pending_approvals.insert(request.id, { info: request, deferred })
  yield* deferred.await()   // agent suspends here

approve(id: PermissionId):
  pending_approvals.remove(id)?.deferred.resolve(())

reject(id: PermissionId):
  pending_approvals.remove(id)?.deferred.reject(RejectedError)
```

#### Scope-based finalizer cleanup

All pending approvals are registered in a scoped context. When the scope closes (session
end, agent cancellation, turn abort), the finalizer rejects every unresolved Deferred:

```text
[REFERENCE]
scope.finalizer:
  for (id, entry) in pending_approvals.drain():
    entry.deferred.reject(RejectedError { reason: "scope_closed" })
```

This prevents the agent from being permanently suspended waiting for an approval that
will never arrive (e.g., after a client disconnects).

### 4.8 Agent mode and on-demand generation

#### Agent mode

Every agent definition carries a `mode` that determines which messages it receives in a
multi-agent session:

```text
[REFERENCE]
AgentMode: "subagent" | "primary" | "all"

"primary"  — receives the conversation from the user's perspective;
             owns the session lifecycle; default for interactive agents.
"subagent" — receives only the delegation request from its parent;
             returns a structured result; never sees the full user history.
"all"      — receives every message regardless of source; used for monitor/observer
             roles that need full visibility (e.g., duplicate-PR detectors, audit agents).
```

`mode` is declared in the agent definition frontmatter (see `l2-extension-registry.md §4.10`).
The orchestrator uses it when routing a delegation to select the correct message slice.

#### On-demand agent generation

When the orchestrator needs an agent for a task that has no pre-defined definition, it can
generate one on demand from a description:

```text
[REFERENCE]
generate_agent(description: String, model?: ModelRef) -> AgentDefinition:
  // Issues a structured LLM call with the description as input.
  // Returns a validated AgentDefinition (name, mode, tool access, system prompt).
  // The generated definition is ephemeral — not persisted to the registry unless
  // the user explicitly saves it via `cronus ext add --from-generated <id>`.
```

Generated agents receive a `source: "generated"` tag; the quality pipeline scans them
before the orchestrator uses them for the first time.

## 5. Drawbacks & Alternatives

- **Judge model choice:** too weak a judge mis-confirms; too strong is costly. Mitigated by making the judge model configurable.
- **Mailbox overhead:** message passing adds bookkeeping; justified for coherent multi-agent hand-offs.
- **Alternative — orchestrator executes trivial tasks itself:** rejected; it erodes ORC-1 clarity. Trivial work still goes through a (cheap) role.
- **MAX_SPAWN_DEPTH = 3:** arbitrary but prevents runaway recursion; configurable via workspace config for teams that need deeper delegation chains.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[PROTOCOL]` | `.design/main/specifications/l1-orchestration.md` | Invariants this implements |
| `[BOARD]` | `.design/main/specifications/l2-kanban-board.md` | Delegation channel |
| `[OFFICE]` | `.design/main/specifications/l1-office-model.md` | Roles and staffing that execute delegated work |

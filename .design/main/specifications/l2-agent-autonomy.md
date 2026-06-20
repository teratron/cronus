# Agent Autonomy

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-security.md, l1-orchestration.md

## Overview

The concrete autonomy ladder and its enforcement machinery: a three-tier `AutonomyLevel` that controls how much a running agent can do without pausing for human approval, a per-call risk classifier that maps every tool operation to a `CommandRiskLevel`, an `ActionTracker` that enforces rolling hourly caps, and an approval gate lifecycle that parks high-impact calls until the user decides.

## Related Specifications

- [l1-security.md](l1-security.md) - SEC-6 sandbox, SEC-7 audit.
- [l1-orchestration.md](l1-orchestration.md) - ORC-9 approval gate.
- [l2-tool-security.md](l2-tool-security.md) - Tool guard that produces `SuspendedPermission` escalated into this gate.
- [l2-scheduler.md](l2-scheduler.md) - Cron context bypasses the interactive approval path.
- [l2-security.md](l2-security.md) - Secret handling; audit log destination.
- [l2-orchestration.md](l2-orchestration.md) - Orchestrator triggers approvals for sub-manager promotions and agent hires.

## 1. Motivation

An autonomous agent can issue shell commands, write files, and call external APIs. The blast radius of an unchecked agent is unbounded. The autonomy ladder constrains it at design time (what tier is this agent?) and at runtime (how many actions this hour? does this specific call need a human?). A single enforcement point — `SecurityPolicy::gate_decision` — ensures every code path through the engine passes the same check.

## 2. Constraints & Assumptions

- The autonomy level is set per agent instance; it cannot be elevated by a model-produced tool call.
- The approval gate is interactive-only: background and cron contexts receive a derived policy (non-destructive actions auto-allowed, destructive auto-denied).
- `ActionTracker` counts are session-scoped; process restart resets them.
- Always-forbidden paths are unconditional and cannot be overridden by any approval.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| SEC-6 Sandbox | Every shell/code tool call passes `gate_decision` before execution; always-forbidden patterns are blocked before any sandbox spawn. |
| SEC-7 Auditable | Every gate decision (Allow/Prompt/Block) appends to the audit log with reason and tool context. |
| ORC-9 Approval gate | `GateDecision::Prompt` parks the call in an `ApprovalRequest` with a 10-minute TTL; outcome is recorded. |

## 4. Detailed Design

### 4.1 Autonomy level

```text
[REFERENCE]
AutonomyLevel: "supervised" | "semi_autonomous" | "autonomous"
```

| Level | Behavior |
| --- | --- |
| `supervised` | Every tool call that is not `read`-class requires explicit approval before execution. |
| `semi_autonomous` | `read` and `write` calls auto-proceed; `network`, `install`, and `destructive` calls require approval. |
| `autonomous` | All calls auto-proceed except `destructive`-class, which always require approval. Always-forbidden paths are blocked unconditionally regardless of level. |

The level is stored in the agent's runtime config and loaded at session start. It cannot be changed from within a running session by a model-produced action.

### 4.2 Command risk classification

Every tool call is classified into one of five risk levels before the gate runs:

```text
[REFERENCE]
CommandRiskLevel: "read" | "write" | "network" | "install" | "destructive"
```

| Class | Examples |
| --- | --- |
| `read` | File reads, directory listings, memory recall, web fetch (GET) |
| `write` | File writes/edits, database updates, environment variable writes |
| `network` | HTTP POST/PUT/PATCH, WebSocket connections, DNS lookups for non-read URLs |
| `install` | Package manager calls, extension installs, binary downloads |
| `destructive` | File deletes, process kills, config resets, `rm`, `DROP TABLE`, data purges |

Classification is performed by `ToolOperation::classify(tool_name: &str, params: &ToolParams) -> CommandRiskLevel`. When a tool is unknown, the default classification is `write` (conservative).

### 4.3 SecurityPolicy

`SecurityPolicy` is assembled once at agent session start from the agent's `AutonomyConfig` and the workspace directory. It is immutable for the session lifetime.

```text
[REFERENCE]
AutonomyConfig {
  level: AutonomyLevel,
  max_actions_per_hour: u32,            // default 60; 0 = unlimited
  workspace_dir: PathBuf,               // all file paths validated against this boundary
  always_forbidden_extra: Vec<String>,  // operator-added patterns (appended to built-ins)
}

SecurityPolicy {
  level: AutonomyLevel,
  max_actions_per_hour: u32,
  workspace_dir: PathBuf,
  forbidden_patterns: Vec<Regex>,  // compiled from always_forbidden_extra + built-ins
}

SecurityPolicy::gate_decision(
  class: CommandRiskLevel,
  level: AutonomyLevel,
  context: ExecutionContext,   // "interactive" | "background" | "cron"
) -> GateDecision

GateDecision: Allow | Prompt | Block { reason: String }
```

Gate matrix (interactive context):

| Class | supervised | semi_autonomous | autonomous |
| --- | --- | --- | --- |
| `read` | Allow | Allow | Allow |
| `write` | Prompt | Allow | Allow |
| `network` | Prompt | Prompt | Allow |
| `install` | Prompt | Prompt | Allow |
| `destructive` | Prompt | Prompt | Prompt |

Background/cron context: `read` and `write` → Allow; `network` and `install` → Allow (operator must configure `autonomous` level for background agents); `destructive` → Block unconditionally (no approval in unattended context).

### 4.4 Always-forbidden paths

The following are blocked before the gate runs — no autonomy level and no approval can override them:

- Shell patterns: `rm -rf /`, `sudo rm -rf`, `mkfs`, `dd if=/dev/`
- Any file path resolving outside `workspace_dir` after symlink expansion
- Any file path matching `.env` or secrets-tier patterns (see `l2-security.md §4.1`)

A call matching an always-forbidden pattern returns `GateDecision::Block` immediately and is logged as `HARD_BLOCKED` in the audit trail.

### 4.5 ActionTracker

`ActionTracker` enforces the rolling hourly cap independently of the gate decision.

```text
[REFERENCE]
ActionTracker {
  session_counts: HashMap<CommandRiskLevel, u32>,
  hourly_window:  RollingWindow,   // sliding 60-minute window of action timestamps
  max_actions_per_hour: u32,
}

impl ActionTracker {
  record(class: CommandRiskLevel) -> Result<(), ActionCapError>
  // Pushes a timestamp; evicts entries older than 60 min.
  // If count after push >= max_actions_per_hour: Err(ActionCapError).

  session_total() -> u32
  hourly_count()  -> u32
}
```

`ActionCapError` produces `GateDecision::Block { reason: "hourly_action_cap_exceeded" }` and is surfaced to the user as a soft stop. The cap slides continuously — no forced cooldown period.

### 4.6 Approval gate lifecycle

When `gate_decision` returns `Prompt` in an interactive context, the tool call is parked as an `ApprovalRequest`:

```text
[REFERENCE]
ApprovalRequest {
  id:           String,              // UUID
  tool_name:    String,
  tool_kind:    String,              // "file" | "shell" | "network" | "install" | "other"
  target:       Option<String>,      // primary affected resource (path, command, URL)
  summary:      String,              // human-readable single-sentence description
  paths:        Vec<String>,         // up to 5 affected paths
  risk_class:   CommandRiskLevel,
  ttl_secs:     u32,                 // default 600 (10 minutes)
  created_at:   Instant,
  requires_user_confirmation: true,
}
```

#### Lifecycle

1. `ApprovalRequest` is created and sent to the interactive session's approval UI.
2. The agent suspends execution of the specific tool call; other turn logic is unaffected.
3. The user chooses: Allow once / Allow for session / Deny / Deny and explain.
4. **TTL expiry**: no response within `ttl_secs` → auto-Deny; agent receives `ApprovalDenied { reason: "timeout" }`.
5. **Background/cron bypass**: `destructive`-class → auto-Deny; all others → auto-Allow. Bypass is logged.
6. All outcomes (Allow / Deny / Timeout) are written to the audit log.

"Allow for session" grants approval for calls with the same `(tool_name, target)` pair for the remainder of the session without re-prompting. This approval is not persisted across sessions.

### 4.7 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| get current level | `cronus agent autonomy get` | `/agent autonomy get` | `agent.autonomy.level() -> AutonomyLevel` |
| set level | `cronus agent autonomy set --level <supervised\|semi\|auto>` | `/agent autonomy set …` | `agent.autonomy.set_level(level) -> void` |
| set hourly cap | `cronus agent autonomy set --max-actions-per-hour <N>` | `/agent autonomy set …` | `agent.autonomy.set_cap(n) -> void` |
| view action counts | `cronus agent autonomy status` | `/agent autonomy status` | `agent.autonomy.status() -> ActionTrackerStatus` |

## 5. Drawbacks & Alternatives

- **10-minute TTL may feel short**: the timer is visible to the user; they can re-trigger the action if it expires. Extending it increases the window for stale approvals.
- **Hourly cap is session-scoped**: process restart resets counts. For multi-session continuity, a persistent ledger (e.g. in the budget engine) would be needed — deferred to the budget engine spec.
- **`write`-class auto-allowed in `semi_autonomous`**: file writes are common and this avoids approval fatigue. The tool guard still runs for every write call (path containment, hard blocks).
- **Alternative — per-tool allowlists instead of risk classes**: more granular but requires maintenance as new tools are added. The risk-class model is more stable; per-tool overrides can be layered on top.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | SEC-6/SEC-7 invariants |
| `[ORC]` | `.design/main/specifications/l1-orchestration.md` | ORC-9 approval gate |
| `[TOOLSEC]` | `.design/main/specifications/l2-tool-security.md` | Tool guard escalation |
| `[SCHED]` | `.design/main/specifications/l2-scheduler.md` | Cron context — bypass rules |

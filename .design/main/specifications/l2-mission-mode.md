# Mission Mode

**Version:** 1.0.2
**Status:** Stable
**Layer:** implementation
**Implements:** l1-orchestration.md

## Overview

Mission Mode is a focused, two-phase autonomous execution unit: the agent first explores the scope and writes a structured plan (PRD), then iterates on implementation until every acceptance criterion is satisfied or a safety limit is reached. Unlike the ongoing office work loop, a mission has a clear start, a user-controlled checkpoint between phases, and a deterministic termination condition.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) - Autonomous execution and budget circuit-breaker.
- [l2-orchestration.md](l2-orchestration.md) - Goal/judge/budget loop; missions are the execution engine for a single delegated goal.
- [l2-kanban-board.md](l2-kanban-board.md) - A mission may reference a board card as its task source.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - Workspace path where missions are stored.
- [l2-security.md](l2-security.md) - Sandbox constrains what the executing agent can do.
- [l2-tool-security.md](l2-tool-security.md) - Tool guard is active throughout mission execution.

## 1. Motivation

Long-running autonomous tasks need a checkpoint: the agent should not blindly execute an unreviewed plan. Mission Mode inserts exactly one human-controlled checkpoint — after planning and before execution — without requiring continuous supervision. The two-phase design also makes the acceptance criteria explicit (prd.json) so termination is deterministic rather than LLM-decided.

## 2. Constraints & Assumptions

- A mission runs within a single workspace; it operates on the workspace's filesystem context.
- Phase 1 (planning) has full tool access; Phase 2 (execution) has scaffolding tools deactivated so the agent codes and reasons but does not directly invoke build/package-manager commands.
- The acceptance criteria (user stories with `passes` flags) are the sole termination signal; the agent does not self-terminate based on its own judgment.
- A `max_iterations` guard prevents runaway loops when stories never pass.
- Mission state files are written atomically; a crash during a mission leaves the state readable for resume.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| ORC-6 Judged termination | Termination is determined by `prd.json.userStories[*].passes` — all true → done. No LLM self-assessment in the loop. |
| ORC-7 Budget circuit-breaker | `max_iterations` is the mission-level circuit-breaker; it stops the loop with a partial-completion report. |
| ORC-9 Approval gate | User must confirm PRD before Phase 2 starts — this is the mission's mandatory approval checkpoint. |
| ORC-10 Resumable | `prd.json` and `loop_config.json` persist state; a restart re-enters Phase 2 from the last known story states. |

## 4. Detailed Design

### 4.1 Mission phases

```mermaid
graph TD
    START[user: /mission start "task text"] --> P1[Phase 1: Exploration + PRD]
    P1 --> PRESENT[agent presents prd.json to user]
    PRESENT --> CONFIRM{user confirms?}
    CONFIRM -->|edits + confirms| P2[Phase 2: Execution loop]
    CONFIRM -->|rejects| REVISE[agent revises prd.json → loop back to PRESENT]
    P2 --> CHECK{all stories pass?}
    CHECK -->|yes| DONE[mission complete]
    CHECK -->|no, within budget| ITER[inject continuation → next iteration]
    ITER --> P2
    CHECK -->|max_iterations reached| PARTIAL[partial completion report]
```

### 4.2 Phase 1 — Exploration and PRD generation

The agent receives the task description and explores the workspace context (codebase, existing board cards, memory). It then writes `prd.json` with the structured plan.

Full tool access is available in Phase 1 — the agent may read files, query memory, run searches, and write artifacts.

Phase 1 ends when the agent finishes its exploration turn and a valid `prd.json` exists. The agent presents the PRD to the user and pauses. Control returns to the user for review, optional edits to `prd.json`, and explicit confirmation before Phase 2 starts.

### 4.3 PRD format

```text
[REFERENCE]
prd.json {
  project: String,          // short project/task name
  description: String,      // one-paragraph task summary
  userStories: [
    {
      id: String,           // e.g. "US-001"
      title: String,        // one-line story title
      story: String,        // "As a … I want … so that …" or acceptance text
      passes: bool          // false initially; agent sets true when story is satisfied
    }
  ]
}
```

User stories are the unit of work and the unit of verification. The agent sets `passes: true` on a story when it has satisfied the acceptance criteria and verified the outcome. A story with `passes: false` triggers another iteration.

### 4.4 Phase 2 — Execution loop

After user confirmation, the loop runs:

```
while not all_stories_pass(prd) and iteration < max_iterations:
    agent.run_turn(context)
    prd = read_prd(loop_dir)
    iteration += 1

if all_stories_pass(prd):
    emit mission_complete(passed=total, total=total)
else:
    emit mission_max_iterations(passed=count_passed, total=total, max=max_iterations)
```

Each iteration the agent receives a continuation message summarizing remaining stories and may use any tools except the deactivated scaffolding set (package-manager CLIs, build runners). This constraint is intentional: the agent writes correct code; builds and tests are verified by external CI or the user — not by the agent invoking them directly within the loop.

### 4.5 State files

Every mission lives in an isolated directory:

```plaintext
<ws>/missions/<mission-id>/
├── loop_config.json   # environment metadata (git, session, paths)
├── prd.json           # task list — worker updates `passes`
├── progress.txt       # append-only iteration log
└── task.md            # original task description (read-only)
```

`<mission-id>` is generated at creation time from a timestamp: `mission-YYYYMMDD-HHMMSS`.

#### loop_config.json

```text
[REFERENCE]
loop_config.json {
  session_id: String,
  branch_name: String,
  git_installed: bool,
  is_git_repo: bool,
  default_branch: String,   // "main" | "master" | <current>
  current_branch: String,
  repo_root: String
}
```

Git context is detected asynchronously at mission creation. `is_git_repo: false` is a valid state (no-VCS workspace). The `session_id` links the loop directory back to its originating session, enabling `get_active_loop_dir` to find the correct mission when multiple missions exist.

#### progress.txt

Append-only log, seeded with a `## Codebase Patterns` section that the agent populates with reusable insights discovered during the run. New entries are appended after each iteration summary. This file is never truncated during a mission.

### 4.6 Resuming a mission

On restart, the agent can call `mission resume` to re-enter Phase 2 from the persisted state:

1. Locate the most recent `missions/mission-*/loop_config.json` matching the current session.
2. Read `prd.json` — stories with `passes: true` are already done; only false stories remain.
3. Re-enter the execution loop from the current story count.

Missions are not automatically resumed on process restart; the user issues `mission resume` explicitly.

### 4.7 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| start mission | `cronus mission start "<task>"` | `/mission start …` | `mission.start(task) -> Mission` |
| confirm PRD (enter Phase 2) | `cronus mission confirm` | `/mission confirm` | `mission.confirm() -> void` |
| show status | `cronus mission status` | `/mission status` | `mission.status() -> MissionStatus` |
| list missions | `cronus mission list` | `/mission list` | `mission.list() -> Mission[]` |
| resume | `cronus mission resume [<id>]` | `/mission resume` | `mission.resume(id?) -> void` |
| abort | `cronus mission abort [<id>]` | `/mission abort` | `mission.abort(id?) -> void` |

### 4.8 Discuss-phase decision capture

Before a mission is planned, a structured discussion phase extracts implementation decisions. The discussion agent is a thinking partner, not an interviewer: it identifies ambiguous implementation choices ("gray areas"), presents them to the user, and captures concrete decisions for downstream planners and researchers to act on.

#### Decision document format (CONTEXT.md)

```text
[REFERENCE]
# Phase [N]: [Name] — Context

**Gathered:** [date]
**Status:** Ready for planning

<domain>
## Phase Boundary
[Clear scope statement from the roadmap — fixed; discussion clarifies HOW to implement, never WHETHER to add new capabilities]
</domain>

<decisions>
## Implementation Decisions

### [Topic area]
- **D-01:** [Specific decision — concrete enough for planner to act without asking again]
- **D-02:** [Another decision if applicable]

### Claude's Discretion
[Areas where user explicitly deferred to the agent — agent has flexibility here]
</decisions>

<deferred>
## Deferred Ideas
[Ideas that surfaced but belong in other phases. Captured so they're not lost, but explicitly out of scope for this phase.]
</deferred>

<canonical_refs>
## Canonical References
[Specs, ADRs, or design docs that agents must read before planning or implementing — full relative paths]
</canonical_refs>
```

#### Decision ID traceability (D-NN)

Each decision is assigned a stable ID (D-01, D-02, …). Planner agents must:

- Reference the decision ID in task actions ("per D-03")
- Include at least one task implementing every locked decision
- Never include tasks implementing a deferred idea

Decisions are binding; the planner honors them even when research suggests a different approach. Any conflict between a locked decision and research findings is documented with an explanation ("using X per D-02; research suggested Y").

#### Scope guardrail

Discussion scope is fixed by the roadmap boundary. New capabilities cannot be added during discussion:

- **Allowed:** clarifying HOW to implement what is already in scope (layout, behavior, edge cases)
- **Not allowed:** adding new capabilities ("what about also adding X?")

When the user suggests out-of-scope work, the agent captures it under "Deferred Ideas" — not lost, not acted on.

#### Gray area identification

Gray areas are implementation choices the user cares about that could go multiple ways and would change the delivered result. The agent identifies them by reading the phase goal and considering what would be visible/observable to the user:

- Things users SEE, CALL, RUN, or READ
- Choices between multiple valid implementations where the user has a preference
- Behaviors on edge cases or empty states that are product decisions, not engineering decisions

### 4.9 Phase lifecycle state machine

Mission phases follow a deterministic status progression. Each phase status is machine-readable and gates whether planning, execution, or verification may run:

```text
[REFERENCE]
Phase status values:
  Pending      — Phase exists in roadmap; no discussion or planning has started
  Planned      — CONTEXT.md or PLAN.md files exist; planning complete, not yet executing
  In Progress  — Execution is running; some plans have summaries, some do not
  Executed     — All plans have SUMMARY.md files; awaiting verification
  Needs Review — Execution complete but verification has FAIL items without overrides
  Complete     — All SUMMARY.md files exist AND VERIFICATION.md shows status: passed

Status transition guards:
  Planned → In Progress  : at least one executor has committed work
  Executed → Complete    : VERIFICATION.md with status: passed added
  Complete → replanned   : HARD BLOCK unless --force flag; prevents overwriting shipped evidence
```

The `Complete` status is a hard gate: replanning a closed phase silently rewrites plan documents that no longer match the shipped code. An explicit override flag is required, and a warning banner must be emitted if that flag is used.

Prior-phase completeness scan runs before any phase advance:

- Plans without matching summaries (execution started, not completed) → warning with deferred-backlog option
- Prior phases with unresolved VERIFICATION.md FAIL items → hard stop

### 4.10 SUMMARY.md rich frontmatter

Each plan produces a SUMMARY.md after completion. The frontmatter carries a structured dependency graph and tech-tracking metadata that downstream agents and future planning phases use to understand what each plan built and how it relates to adjacent work:

```text
[REFERENCE]
SUMMARY.md frontmatter:

---
phase: XX-name
plan: NN
subsystem: [primary category: auth, payments, ui, api, database, infra, testing, etc.]
tags: [searchable tech terms: jwt, stripe, react, postgres, prisma]

# Dependency graph
requires:
  - phase: [prior phase or plan this depends on]
    provides: [what that phase built that this plan uses]
provides:
  - [bullet list of what this plan built/delivered]
affects: [list of phase names or keywords that will need this context]

# Tech tracking
tech-stack:
  added: [libraries/tools added in this plan]
  patterns: [architectural/code patterns established]

key-files:
  created: [important files created]
  modified: [important files modified]

key-decisions:
  - "Decision 1"
  - "Decision 2"

patterns-established:
  - "Pattern 1: description"

requirements-completed: []  # REQUIRED — all requirement IDs from this plan's `requirements` field

# Metrics
duration: Xmin
completed: YYYY-MM-DD
---
```

The `requires/provides/affects` graph enables future planner agents to detect what prior plans built and whether they need to load earlier summaries for context. The `requirements-completed` field is mandatory: it must copy every requirement ID from the plan's own `requirements` frontmatter field so that coverage can be verified across the entire phase.

### 4.11 Operation mode ladder

Missions can run at multiple intensity levels, controlling how aggressively the agent applies simplification, verification depth, and context loading. The active mode persists for the duration of the session and is resolved from three sources in priority order.

#### Intensity levels

| Mode | Behavior |
| --- | --- |
| `lite` | Minimal overhead — plan quickly, execute with reduced verification depth. Use for low-stakes or well-understood tasks. |
| `full` | Default mode — full discuss-phase, complete planning, all quality gates. Balanced cost and thoroughness. |
| `ultra` | Maximum rigor — extended adversarial review, goal-backward verification with multiple passes, exhaustive must_haves coverage. Use for high-stakes or novel work. |
| `off` | Disable mission mode — the agent runs as a standard session without structured planning or execution phases. |

#### Resolution hierarchy

The active mode is resolved at session start using a three-source priority chain:

```text
[REFERENCE]
Mode resolution order (first source that provides a value wins):

  1. Environment variable: CRONUS_MISSION_MODE=lite|full|ultra|off
     Set by CI/CD pipelines, launch scripts, or per-session shell exports.
     Highest priority — overrides all other sources.

  2. Workspace config file: <ws>/config.json → { "missionMode": "full" }
     Set by the user once per workspace. Survives session restarts.
     Mid-priority — overrides the compiled default but not the env var.

  3. Compiled default: full
     Always available. No setup required — missions work out-of-box.
```

The agent reads the resolved mode at session start and applies it to all phases (discuss/plan/execute/verify) for the current session. The mode does not auto-advance or escalate mid-mission unless explicitly changed.

#### Flag file state tracking

The active mission mode is written to a flag file so that status line renderers and hooks can read it without running the full resolution chain:

```text
[REFERENCE]
Flag file: <ws>/.mission-mode
Content: plain text, one of: lite | full | ultra | off | (empty = no mission active)

Write events:
  - Written when a mission starts (contains the resolved mode for this run)
  - Updated when the user changes mode mid-session (/mission mode ultra)
  - Cleared (or written "off") when a mission completes or is aborted

Read by:
  - Status line hook: renders [MISSION], [MISSION:FULL], [MISSION:ULTRA] etc.
  - Pre-phase hooks: may skip expensive steps when mode is lite
  - Resume logic: verifies the persisted mode matches the current config before re-entering
```

The flag file is the cross-session state signal — it lets peripheral tooling observe mission state without importing core logic.

## 5. Drawbacks & Alternatives

- **Deactivated scaffolding tools:** the agent cannot run `cargo build` or `npm test` directly. This means acceptance criteria that require a passing build must be written as observable file-state checks, not build-output checks. Mitigation: the spec guidance for writing user stories should note that `passes` is set by the agent examining artifacts, not by executing toolchains.
- **PRD as sole termination signal:** if the agent writes a story with poorly-chosen acceptance criteria, the mission may loop indefinitely until `max_iterations`. Mitigation: the user reviews and can edit `prd.json` before Phase 2; `max_iterations` is the backstop.
- **No parallel story execution:** stories execute sequentially within one agent session. A future enhancement could run independent stories in parallel agent forks (requires story-level dependency declarations in prd.json).
- **Alternative — continuous autonomous loop without checkpoint:** rejected. The Phase 1 → user-confirm → Phase 2 pattern is the safety contract; removing the checkpoint eliminates the user's ability to catch a badly-scoped plan before execution.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORC]` | `.design/main/specifications/l1-orchestration.md` | Autonomy invariants |
| `[L2ORC]` | `.design/main/specifications/l2-orchestration.md` | Goal/judge/budget loop |
| `[LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | Workspace missions directory |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

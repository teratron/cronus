# Mission Mode

**Version:** 1.0.0
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

# Version Control

**Version:** 1.0.1
**Status:** Stable
**Layer:** concept

## Overview

Version control covers two scopes: the office's internal staging area (a lightweight, cooperative versioning layer used during task execution) and the office's interaction with external git-based remotes (commit discipline, branching strategy, and role-based authority). The two scopes are connected: the internal staging area is the pre-commit buffer; its output is the conventional git commit.

## Related Specifications

- [l1-office-model.md](l1-office-model.md) — roles whose responsibilities include git and remote operations
- [l2-execution-workspace.md](l2-execution-workspace.md) — git worktrees used as isolated staging areas during execution
- [l2-kanban-board.md](l2-kanban-board.md) — cards drive the task boundary at which a commit is produced
- [l1-quality-standards.md](l1-quality-standards.md) — quality gate that must pass before a commit is permitted

## 1. Motivation

Autonomous workers modifying source files need a controlled handoff back to the shared repository. Without a defined protocol, concurrent workers corrupt shared history; without clear role authority, any worker could push to protected branches. Equally, task-level iteration benefits from a virtual staging layer — a place where changes accumulate and can be reviewed before hitting the real commit log, which must remain clean.

## 2. Constraints & Assumptions

- Every office workspace is backed by a git repository. Non-git workspaces are out of scope.
- The external remote is one of a class of compatible hosting platforms (hosting platform-neutral — the interface is the git wire protocol and the platform's standard PR/MR workflow).
- Commit discipline applies to all workers; branching strategy is per-office, set in Local Settings.
- The virtual staging area is an in-process layer; it does not persist across application restarts independently — its output is always a real git commit.

## 3. Core Invariants

- **VC-1 Role-based commit authority**: only the worker roles explicitly authorized in the office's version control policy may commit or push to a given branch. The orchestrator enforces this; no worker self-promotes authority.
- **VC-2 Quality gate before commit**: no commit may be produced until all required quality gates for the changed artifact type have passed. A commit that bypasses quality gates is a protocol violation regardless of which role requested it.
- **VC-3 Card-aligned commit boundary**: each commit corresponds to the unit of work defined by a kanban card or an explicit sub-task within one. Commits that span multiple cards or that contain work not traceable to any card are prohibited.
- **VC-4 Virtual staging atomicity**: the virtual staging area accumulates changes atomically. A failed or cancelled task MUST NOT partially flush its staging area to the real commit log; the entire set of changes from the task is either committed or discarded.
- **VC-5 Remote isolation by branch**: no worker pushes directly to the main/trunk branch. All work lands via a branch and a pull/merge request, regardless of branching strategy. This rule is unconditional — no autonomy level overrides it.
- **VC-6 Non-git versioning out of scope**: document versioning (e.g., semantic version fields in spec files) and artifact versioning (e.g., binary releases) are handled by their own lifecycle protocols. This spec covers only git-backed source versioning.

## 4. Detailed Design

### 4.1 Virtual Staging Area

The virtual staging area (VSA) is a per-task, in-memory accumulation layer built on top of git worktrees. Workers accumulate file mutations in an isolated worktree without touching the main branch working tree. When the task reaches a commit-eligible state:

```text
[REFERENCE]
1. Quality gates run against the staged content in the worktree
2. On pass: all staged mutations are collected into a single git commit
3. Commit message is generated from: card title + change summary + token-log excerpt
4. The worktree is finalized: commit is pushed to the task branch
5. On fail: the worktree is discarded; no partial commit is written (VC-4)
```

Workers never write directly to the main working tree during task execution. All writes go through the worktree. This provides a review surface (the diff between the worktree and main) before the commit is sealed.

### 4.2 Branching Strategies

The office declares one of two branching strategies in Local Settings → Git. The strategy determines which roles may create branches and how long branches live.

| Strategy | Description | Branch lifetime | Who merges |
| --- | --- | --- | --- |
| **Trunk-based** | All work branches are short-lived (≤1 day); PRs are small and frequent; trunk is always deployable | Short (hours to one day) | Orchestrator or designated Maintainer role |
| **Git Flow** | Long-lived feature and release branches; merges are gated on milestones | Long (days to weeks) | Designated Release Manager role after milestone review |

The office uses trunk-based by default; Git Flow is selected when the office manages versioned software releases with explicit release cycles.

Regardless of strategy:

- `main`/`trunk` is always protected (VC-5)
- Hotfix branches follow trunk-based semantics even in Git Flow offices (speed constraint)

### 4.3 Role-Based Git Authority

| Role | May create branch | May commit to branch | May push to branch | May merge PR/MR | May push to main | May configure remote |
| --- | --- | --- | --- | --- | --- | --- |
| Worker (any) | Yes (task branch only) | Yes (own worktree) | No | No | No | No |
| Orchestrator | Yes | Yes | Yes (task branch) | No | No | No |
| Maintainer | Yes | Yes | Yes (any branch) | Yes | No | No |
| Release Manager | Yes | Yes | Yes (any branch) | Yes | No | Yes |
| Manager (CEO) | No (delegates) | No (delegates) | No | Yes (final) | No | Yes |

Authority levels are enforced by the orchestrator at the time of the git operation, not at commit time. A worker that requests an unauthorized operation is halted and the request is escalated.

### 4.4 Commit Message Convention

All commits produced by office workers MUST follow the Conventional Commits format:

```text
<type>(<scope>): <short summary>

[optional body]
[optional footer: card reference]
```

Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`.
Scope: the subsystem or module changed (e.g., `kanban`, `scheduler`, `voice-input`).
Card reference footer: `Refs: #<card-id>` (non-optional; enables card traceability, VC-3).

### 4.5 Remote Hosting Platform Compatibility

The office version control layer is platform-neutral at the protocol level. Platform-specific features (PR templates, CI/CD hooks, issue linkage) are exposed as integrations in Local Settings → Integrations. Supported platform classes:

- Self-hosted git remotes (bare repositories over SSH or HTTPS)
- Cloud-hosted managed platforms (compatible with the git wire protocol + REST/GraphQL API for PR/MR lifecycle)

Platform integrations are installed as office-scoped extensions. The core version control spec does not depend on any specific platform.

## 5. Implementation Notes

1. The VSA is implemented using the git worktree subsystem from `l2-execution-workspace.md` — no new worktree infrastructure is needed; this spec layers policy on top of existing worktree support.
2. VC-1 authority checking is a guard in the orchestrator's delegation layer, called before any `git push` or `git merge` command is dispatched to any worker.
3. Commit message generation is a harness-engineered step in the task completion workflow — a nodus `@steps` block that reads the VSA diff and produces a Conventional Commits message conformant string.

## 6. Drawbacks & Alternatives

**Alternative: direct commits to main by any worker** — simplest model, no branching overhead. Rejected: violates VC-5; a single bad commit from an autonomous worker can break shared state with no recovery buffer.

**Alternative: single universal branching strategy** — one strategy for all offices. Rejected: release-managing offices (shipping versioned binaries) have materially different lifecycle requirements than personal-project offices.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[EXEC-WS]` | `.design/main/specifications/l2-execution-workspace.md` | Git worktree backing the VSA (VC-4) |
| `[QUALITY]` | `.design/main/specifications/l2-quality-pipeline.md` | Quality gates required before commit (VC-2) |
| `[KANBAN]` | `.design/main/specifications/l2-kanban-board.md` | Card boundary that defines commit scope (VC-3) |
| `[OFFICE-MODEL]` | `.design/main/specifications/l1-office-model.md` | Role taxonomy informing git authority table (VC-1) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — VC-1…VC-6, virtual staging area, trunk-based vs Git Flow, role authority table, commit convention, remote compatibility |
| 1.0.1 | 2026-07-10 | Core Team | Fixed broken Related Specifications link — the quality gate is an L1 concept (l1-quality-standards.md), the l2- prefix pointed at a non-existent file |

# Development Workflow — Cronus Implementation

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-development-workflow.md

## Overview

Cronus implementation of the agent-assisted development workflow. Covers: the bundled skill catalog that ships with Cronus, the session-bootstrap mechanism that activates the workflow at session start, dispatch templates for implementer and reviewer agents, script helpers for artifact management, the model-tier selection table, the progress ledger format, and workspace setup/finish mechanics.

## Related Specifications

- [l1-development-workflow.md](l1-development-workflow.md) — Concept spec; all DW-1…DW-10 invariants.
- [l2-extension-registry.md](l2-extension-registry.md) — SKILL.md format, agent definition, session-start hook integration.
- [l2-orchestration.md](l2-orchestration.md) — Agent dispatch, worktree lifecycle, permission model.
- [l2-quality-pipeline.md](l2-quality-pipeline.md) — Review protocol, adversarial checks, SARIF output.
- [l2-agent-constitution.md](l2-agent-constitution.md) — Skill delivery modes and lifecycle hooks.
- [l2-plugin-hooks.md](l2-plugin-hooks.md) — `SessionStart` hook API; `additionalContext` injection.
- [l2-version-control.md](l2-version-control.md) — Worktree slug naming, boot sequence, cleanup.
- [l2-context-management.md](l2-context-management.md) — Context window budgets; motivation for file-handoff discipline.

## 4. Invariant Compliance

| L1 Invariant | Implementation |
| --- | --- |
| DW-1 (Mandatory pipeline) | The `coordinator` skill structures the five stages as an ordered todo list; each stage transition is gated on the previous stage's exit artifact being present on disk. |
| DW-2 (Design gate) | The `design` skill includes a `<HARD-GATE>` instruction block; implementation skills (`writing-plans`, `coordinator`) are listed as the only permitted next steps after human approval. |
| DW-3 (Task isolation) | The `task-brief` script extracts a single task to a uniquely named temp file; the coordinator dispatch contains only: brief file path, one-paragraph context, prior-task interface declarations, and report file path. No accumulated history. |
| DW-4 (Two-stage quality gate) | The reviewer dispatch template requires two explicit verdict lines — `Spec Compliance: ✅/❌` and `Task Quality: Approved / Needs fixes`. A report missing either verdict is treated as BLOCKED by the coordinator. |
| DW-5 (Durable progress ledger) | On every task approval, the coordinator appends one record to `.cronus/dev/progress.md`. At startup and after any compaction signal, the coordinator reads this file and skips tasks listed as complete. |
| DW-6 (Workspace isolation) | The `workspace-setup` skill invokes the platform-native `EnterWorktree` tool when available; falls back to `git worktree add`. All implementation work runs inside the worktree. |
| DW-7 (Model-tier assignment) | The coordinator sets an explicit `model:` field on every agent dispatch per the tier table in §5.5. Omitting `model:` is treated as a coordinator defect in review. |
| DW-8 (Human checkpoints) | The `workspace-finish` skill presents four structured options; Option 4 (Discard) requires the user to type the literal word `discard`. No merge/push/delete executes before the human selects an option. |
| DW-9 (Review neutrality) | The reviewer dispatch template states: "Treat the implementer's report as unverified claims. A stated rationale never changes a finding's severity." No prior-task rationale is included in reviewer context. |
| DW-10 (Critical findings block) | On `Task Quality: Needs fixes`, the coordinator dispatches ONE fix agent with all Critical + Important findings combined; then re-dispatches the reviewer. Minor findings are appended to `.cronus/dev/progress.md` for the final review. |

## 5. Detailed Design

### 5.1 Bundled Skill Catalog

The following skills ship with Cronus under `extensions/bundled/skills/` and are automatically discovered by the extension registry (SKILL.md discovery algorithm from `l2-extension-registry.md`). Each skill uses the standard SKILL.md format.

| Skill | Trigger Signal | Pipeline Stage |
| --- | --- | --- |
| `design` | Any creative, feature, or modification request before code is written | Design |
| `workspace-setup` | After design approval, before plan execution | Design → Plan transition |
| `writing-plans` | Approved design document available | Plan |
| `coordinator` | Plan file ready; execution mode selected | Execute → Review → Deliver |
| `task-reviewer` | Dispatched per task by the coordinator | Review (per-task) |
| `code-review` | Dispatched by coordinator after all tasks pass | Review (whole-branch) |
| `workspace-finish` | After whole-branch review passes | Deliver |
| `tdd` | Any feature or fix implementation task | Execute |
| `debugging` | Failing test or runtime error encountered | Execute |
| `verify-completion` | Before reporting task DONE | Execute |
| `parallel-agents` | Tasks with provably disjoint file sets identified | Execute (optional) |
| `receiving-review` | Code-review report handed to implementer for fixes | Execute (fix loop) |

**Session bootstrap:** The `SessionStart` hook (configured in `extensions/bundled/hooks.json`) reads `skills/bootstrap/SKILL.md` and injects its content as `additionalContext` via the hook output format defined in `l2-plugin-hooks.md`. The bootstrap skill explains the skill system and lists available skills; it activates on every new session and after context compaction.

### 5.2 Design Skill

Trigger: any creative, feature, or modification request before code is written.

```
HARD-GATE: Do NOT invoke any implementation skill, write any code, or
scaffold any project until the human has approved the design document.
This applies regardless of perceived simplicity. (DW-2)
```

Process:

1. Explore current project context (files, docs, recent commits).
2. Scope check: if the request covers multiple independent subsystems, offer decomposition before proceeding; each subsystem gets its own design → plan → implementation cycle.
3. Ask one clarifying question per message (multiple-choice preferred). Focus on: purpose, constraints, success criteria.
4. Propose 2–3 approaches with trade-offs; recommend one with reasoning. Lead with the recommendation.
5. Present design in sections scaled to complexity; get human approval after each section.
6. Write the design document to `docs/specs/YYYY-MM-DD-<topic>.md` and commit.
7. Self-review: scan for placeholders, contradictions, scope issues, and ambiguous requirements. Fix inline.
8. Ask the human to review the written spec before proceeding.
9. Invoke `writing-plans` (the only permitted next skill from Design).

### 5.3 Writing-Plans Skill

Trigger: approved design document.

Every plan document begins with a mandatory header:

```markdown
# [Feature Name] Implementation Plan

> **For agentic workers:** Use the `coordinator` skill to execute this plan task-by-task.

**Goal:** [one sentence describing what this builds]
**Architecture:** [2–3 sentences on approach]
**Tech Stack:** [key technologies / library versions]

## Global Constraints

[Project-wide requirements — version floors, dependency limits, naming conventions,
platform requirements — one line each, exact values copied verbatim from the spec.]
```

Each task block specifies:

```markdown
### Task N: [Component Name]

**Files:**
- Create: `exact/path/to/file.rs`
- Modify: `exact/path/to/existing.rs:42-78`
- Test: `crates/foo/tests/test_component.rs`

**Interfaces:**
- Consumes: [what this task uses from earlier tasks — exact signatures]
- Produces: [what later tasks depend on — exact function names, parameter and return types]

- [ ] Step 1: Write the failing test
- [ ] Step 2: Run it to verify it fails
- [ ] Step 3: Write minimal implementation
- [ ] Step 4: Run tests and verify they pass
- [ ] Step 5: Commit
```

No placeholders. Every step contains the actual content an implementer needs: exact file paths, complete code in code blocks, exact commands with expected output.

**Self-review after writing:** Spec coverage (can every requirement be traced to a task?), placeholder scan, type/signature consistency across tasks.

**Execution handoff:** After saving the plan, present two modes: coordinator (recommended) or inline execution.

### 5.4 Coordinator Skill

Trigger: plan file ready.

**Startup:**

```bash
cat "$(git rev-parse --show-toplevel)/.cronus/dev/progress.md"
```

Tasks listed there as complete are DONE — skip them. Resume at the first incomplete task.

**Per-task loop (DW-3 through DW-10):**

```
for each incomplete task:
  1. record BASE_SHA = git rev-parse HEAD
  2. run: scripts/task-brief PLAN_FILE N  → BRIEF_FILE path
  3. dispatch implementer agent (§5.6 template)
  4. receive: status, commits, one-line test summary, report file path
  5. run: scripts/review-package BASE_SHA HEAD  → DIFF_FILE path
  6. dispatch reviewer agent (§5.7 template)
  7. if Critical/Important findings:
       dispatch ONE fix agent with all findings → reviewer re-reviews
  8. if Task Quality: Approved:
       append record to .cronus/dev/progress.md
       continue to next task
```

**No check-ins between tasks.** The only stop conditions are: BLOCKED status the coordinator cannot resolve, genuine ambiguity preventing progress, or all tasks complete.

**After all tasks:**

```
dispatch code-review agent (whole-branch, most-capable model)
  BASE = git merge-base main HEAD
  run: scripts/review-package BASE HEAD  → DIFF_FILE
if Critical/Important findings: dispatch ONE fix agent → re-review
invoke workspace-finish skill
```

### 5.5 Model-Tier Selection Table

| Task Type | Signal | Tier |
| --- | --- | --- |
| Mechanical transcription | Plan provides complete code; 1–2 files | Cheapest viable |
| Isolated feature | Clear spec, 1–2 files, no design judgment | Cheapest viable |
| Multi-file integration | Interface coordination across 3+ files | Standard |
| Debugging / architecture judgment | Broad codebase understanding required | Most capable |
| Task reviewer — small mechanical diff | <200 LOC, single concern | Standard |
| Task reviewer — complex / concurrent | Locking, shared state, security surface | Most capable |
| Final whole-branch review | Always | Most capable |

The coordinator sets `model:` explicitly on every agent dispatch. An omitted `model:` field silently inherits the session's most expensive model — this is a coordinator defect (DW-7).

**Turn count beats token price:** Cheap models routinely take 2–3× the turns on multi-step work, costing more overall. Use the cheapest tier only when the task's plan text contains the complete code to write (transcription, not reasoning).

### 5.6 Implementer Dispatch Template

```
description: "Implement Task N: [task name]"
model: [MODEL — required; choose per §5.5]
prompt: |
  You are implementing Task N: [task name].

  Read your task brief first: [BRIEF_FILE]
  It contains the full task text with exact values to use verbatim.

  ## Context
  [One paragraph: where this task fits, dependencies, interfaces
   produced by prior tasks that the brief cannot know.]

  ## Before You Begin
  If you have questions about requirements, approach, or assumptions —
  ask them now, before starting work. Do not guess or proceed on unstated assumptions.

  ## Your Job
  1. Implement exactly what the task specifies (nothing more, nothing less).
  2. Write tests following TDD: failing test first, then minimal implementation.
  3. Verify implementation works with the full test suite before committing.
  4. Commit your work.
  5. Self-review (completeness, quality, YAGNI, test hygiene). Fix issues now.
  6. Write full report to [REPORT_FILE]; report back with status only.

  ## Report Back (≤15 lines)
  - Status: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | BLOCKED
  - Commits: short SHA + subject for each
  - One-line test summary (e.g. "42/42 passing, output pristine")
  - Concerns (if any)
  - Report file path: [REPORT_FILE]

  DONE_WITH_CONCERNS = completed but you have doubts about correctness.
  BLOCKED = cannot complete; describe specifically what you are stuck on.
  NEEDS_CONTEXT = need information not provided; describe exactly what.
  Never silently produce work you are unsure about.
```

### 5.7 Reviewer Dispatch Template

```
description: "Review Task N (spec + quality)"
model: [MODEL — required; choose per §5.5]
prompt: |
  You are reviewing one task's implementation: spec compliance first,
  then code quality. This is a task-scoped gate — the broad whole-branch
  review happens separately after all tasks complete.

  Read the task brief: [BRIEF_FILE]

  Global constraints binding this task:
  [GLOBAL_CONSTRAINTS — exact values from plan's Global Constraints section]

  Read the implementer's report: [REPORT_FILE]

  Diff file: [DIFF_FILE]
  Read it once — it contains the commit list, stat summary, and full diff
  with context. Do not re-run git commands. Do not read changed files
  separately unless a hunk you must judge is cut off mid-function.
  Do not crawl the codebase. Inspect code outside the diff only to
  evaluate a concrete, named cross-cutting risk (lock ordering, API contract,
  shared mutable state) — one focused check per named risk.

  Treat the implementer's report as unverified claims. A stated rationale
  ("left it per YAGNI", "kept it simple deliberately") never changes a
  finding's severity. Judge the code on its merits (DW-9).

  ## Part 1: Spec Compliance
  Missing / Extra / Misunderstood requirements?
  ⚠️ = requirement unverifiable from this diff alone (lives in unchanged code
       or spans tasks) — report with what the coordinator should check.

  ## Part 2: Code Quality
  Clean separation of concerns? Proper error handling? DRY without premature
  abstraction? Edge cases handled? Tests verify real behavior, not mocks?

  ## Output Format
  Begin directly with the spec-compliance verdict — no preamble.

  ### Spec Compliance: ✅ Compliant | ❌ Issues found
  [details with file:line if ❌]
  ⚠️ Cannot verify from diff: [what to check, why]

  ### Strengths
  [Specific, evidence-backed. Accurate praise builds trust for the rest.]

  ### Issues
  #### Critical (Must Fix)
  #### Important (Should Fix)
  #### Minor (Nice to Have)
  [Each: file:line — what's wrong — why it matters — how to fix]

  ### Task Quality: Approved | Needs fixes
  [1–2 sentence technical assessment]
```

### 5.8 Progress Ledger Format

Location: `$(git rev-parse --show-toplevel)/.cronus/dev/progress.md`

This file is git-tracked within the feature branch. Worktree scratch directories are git-ignored; the ledger is not — it survives branch operations and is readable after compaction.

```markdown
# Development Progress

Plan: docs/specs/2026-06-24-feature-name.md
Branch: feat/feature-name
Base: abc1234

## Tasks

| # | Name | Status | Commits | Review |
|---|------|--------|---------|--------|
| 1 | Component scaffold | Complete | abc1234..def5678 | Clean |
| 2 | Core logic | Complete | def5678..789abcd | Clean (1 minor logged) |
| 3 | Integration tests | In Progress | — | — |

## Minor Findings (for final whole-branch review)

- Task 2: `crates/foo/src/lib.rs:87` — magic constant, extract to named symbol
```

**Ledger discipline:**

- Write the Task N record in the same coordinator turn as the review approval — never batch.
- On resume (after compaction or restart), read the ledger before dispatching any task.
- If `git log` and the ledger disagree, trust `git log` for commit existence; trust the ledger for review status.

### 5.9 Script Helpers

Three helper scripts under `extensions/bundled/scripts/`:

**`task-brief PLAN_FILE N`**
Extracts task N from the plan to a uniquely named temp file and prints the path. The coordinator passes this path in the implementer dispatch. The implementer sees only its task — never the full plan. Implementers dispatched in sequence each receive an independent brief extract; the coordinator never concatenates brief outputs.

**`review-package BASE HEAD`**
Produces a diff package file containing:

- Commit list: `git log --oneline BASE..HEAD`
- Stat summary: `git diff --stat BASE..HEAD`
- Full diff with context: `git diff -U10 BASE..HEAD`

Prints the unique file path. The reviewer reads this file once; the content never enters the coordinator's context. BASE must be the commit recorded before dispatching the implementer — never `HEAD~1`, which silently truncates multi-commit tasks.

**`progress-ledger [check | append TASK_N STATUS COMMITS]`**
`check` mode: reads `.cronus/dev/progress.md` and outputs the first incomplete task number (or "all-done"). Used at startup and after any compaction signal.
`append` mode: writes one record row to the ledger.

### 5.10 Workspace Setup Skill

1. Detect if already in an isolated workspace: compare `git rev-parse --git-dir` with `--git-common-dir`. Inside a linked worktree (and not a submodule): skip creation, report the existing path.
2. If a platform-native worktree tool (`EnterWorktree`) is available, use it. Otherwise use `git worktree add`. Never mix both.
3. Before `git worktree add`: verify the target directory is listed in `.gitignore` (`git check-ignore -q <dir>`). If not, add and commit the ignore entry first.
4. Run project setup: `cargo build` / `pnpm install` / `go mod download` as auto-detected.
5. Run the full test suite. If tests fail, report failures and ask the human whether to proceed.

Report:

```
Workspace ready at <path>
Branch: <name>
Tests: <N> passing, 0 failures
```

### 5.11 Workspace Finish Skill

1. Run full test suite; block if any failures.
2. Detect workspace state (linked worktree vs. normal repo vs. detached HEAD).
3. Present exactly four options to the human (three if detached HEAD):

```
Implementation complete. What would you like to do?

1. Merge back to <base-branch> locally
2. Push and create a Pull Request
3. Keep the branch as-is
4. Discard this work
```

1. Execute the chosen option:
   - **Merge:** `cd` to main repo root → `git checkout <base>` → `git pull` → `git merge <feature>` → run tests → cleanup worktree (Step 5) → `git branch -d <feature>`.
   - **Push PR:** `git push -u origin <feature>`. Do NOT remove the worktree.
   - **Keep:** Report path. Do NOT remove the worktree.
   - **Discard:** Require the human to type `discard`. `cd` to main repo root → cleanup worktree (Step 5) → `git branch -D <feature>`.

2. Worktree cleanup (Merge and Discard only):

```bash
cd "$(git -C "$(git rev-parse --git-common-dir)/.." rev-parse --show-toplevel)"
git worktree remove "$WORKTREE_PATH"
git worktree prune
```

Only clean up worktrees under `.worktrees/` or `worktrees/` — never clean up worktrees that the platform created independently.

### 5.12 Session Bootstrap Hook

The `extensions/bundled/hooks.json` file registers a `SessionStart` hook:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"${CRONUS_EXTENSION_ROOT}/scripts/session-start\"",
            "async": false
          }
        ]
      }
    ]
  }
}
```

The `session-start` script reads `skills/bootstrap/SKILL.md` and emits the `additionalContext` JSON format defined in `l2-plugin-hooks.md`. The bootstrap skill content lists all bundled skills with their triggers; it activates the coordinator pattern at session start and re-injects after compaction.

## 6. Implementation Notes

1. Bundled skills are loaded from `extensions/bundled/skills/` by the extension registry; they are preset extensions (EXT-5) with read-only manifests.
2. Script helpers (`task-brief`, `review-package`, `progress-ledger`) may be shell scripts on Unix/macOS and Rust binaries for Windows portability; the coordinator invokes them via the Bash tool or equivalent.
3. The progress ledger path (`.cronus/dev/progress.md`) is workspace-relative and written to disk; it survives context compaction because it is not held in agent memory.
4. The `model:` field in dispatch templates must be populated by the coordinator at dispatch time, not left as a template placeholder. Static lint of dispatch prompts should flag a missing `model:` field.

## 7. Drawbacks & Alternatives

**Per-task agent invocations add cost:** Implementer + reviewer = at minimum two agent invocations per task. For plans with many small tasks, consider consolidating steps during the Plan stage. Minimum viable task size: independently reviewable.

**Sequential execution by default:** DW-3 context isolation is compatible with parallel execution when tasks have disjoint file sets. The `parallel-agents` skill handles this case explicitly; the plan should annotate disjoint task groups when parallel dispatch is safe.

**Script helpers require a shell or binary:** On sandboxed environments where Bash is unavailable, the coordinator must replicate script logic inline (git commands, temp-file creation). The coordinator's skill description should document the fallback behavior.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[L1-DW]` | `.design/main/specifications/l1-development-workflow.md` | All DW-1…DW-10 invariants |
| `[EXT-REG]` | `.design/main/specifications/l2-extension-registry.md` | SKILL.md format and skill discovery algorithm |
| `[HOOKS]` | `.design/main/specifications/l2-plugin-hooks.md` | SessionStart hook API and additionalContext format |
| `[ORCH]` | `.design/main/specifications/l2-orchestration.md` | Agent dispatch, worktree lifecycle, permission model |
| `[QP]` | `.design/main/specifications/l2-quality-pipeline.md` | Review protocol and adversarial checks |

<!-- Downstream agent instruction: Load ALL files listed above BEFORE writing any code.
     Do not rely on memory or inference for content from these files. -->

## Document History

| Version | Date | Change |
| --- | --- | --- |
| 1.0.0 | 2026-06-24 | Initial Stable — bundled skill catalog, dispatch templates, model-tier table, progress ledger, script helpers, workspace lifecycle |

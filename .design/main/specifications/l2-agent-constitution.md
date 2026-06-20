# Agent Constitution

**Version:** 1.0.1
**Status:** Stable
**Layer:** implementation
**Implements:** l1-office-model.md, l1-memory-model.md

## Overview

The concrete mechanism for workspace-scoped agent identity and cross-session memory: a small set of plaintext files that give an agent its persona, user profile, tool cheat sheet, and periodic check list. These files let an agent "wake up fresh" each session while retaining accumulated knowledge — the persona is rebuilt from the files, not from conversation history.

## Related Specifications

- [l1-office-model.md](l1-office-model.md) - Agent persona, roles, and client interaction.
- [l1-memory-model.md](l1-memory-model.md) - Memory scopes; workspace-level persistence.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - Workspace directory layout that hosts the constitution files.
- [l2-memory-store.md](l2-memory-store.md) - Structured memory (vector+FTS); constitution files are unstructured counterpart.
- [l2-scheduler.md](l2-scheduler.md) - Heartbeat action that reads HEARTBEAT.md periodic tasks.

## 1. Motivation

A fresh session context has no memory of prior interactions. Structured memory (the vector/FTS store) covers factual recall; the constitution files cover *identity* — the agent's name, persona, working style, and accumulated knowledge about the user. Constitution files are plain Markdown, human-readable and user-editable, with a lightweight YAML frontmatter header that controls when each file is read. They need no migration and work without a database.

## 2. Constraints & Assumptions

- Constitution files live under `<ws>/constitution/`; the workspace controls which set is active.
- Each file has YAML frontmatter: `summary` (one-line, for index display) and `read_when` (list of load conditions).
- BOOTSTRAP.md is a first-run ritual: it is read on first session, guides initial setup, then is **deleted** — its own last instruction is to delete itself. Subsequent sessions find it absent and do nothing.
- HEARTBEAT.md is optional: an empty file (or one containing only comments) suppresses all heartbeat calls — the agent never proactively pings if it has nothing to check.
- The agent may update PROFILE.md and MEMORY.md over time; SOUL.md and HEARTBEAT.md have a stable workspace-level template and are typically user-edited.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| MOD-1 Workspace isolation | Constitution files are per-workspace; employees of different offices carry different profiles. |
| MEM-1 Multi-scope | Constitution files are workspace-scope; global-scope preferences live in the home workspace. |
| SEC-1 Secret isolation | Constitution files never store secrets (secrets go to `.env` / keychain). MEMORY.md may store aliases, not credentials. |

## 4. Detailed Design

### 4.1 File inventory

```plaintext
<ws>/constitution/
├── SOUL.md        # Core persona principles — defines how the agent behaves
├── PROFILE.md     # Agent identity + accumulated user profile
├── MEMORY.md      # Agent's personal cheat sheet (aliases, tool settings)
├── HEARTBEAT.md   # Periodic check task list (empty → no heartbeat)
└── BOOTSTRAP.md   # First-run ritual (deleted after completion)
```

All files use the same frontmatter schema:

```yaml
---
summary: "One-line description — shown in constitution index"
read_when:
  - condition that triggers loading this file
---
```

### 4.2 SOUL.md — persona principles

Read at the start of every session. Contains the agent's working principles: collaborative stance, transparency, when to ask vs. act, how to handle uncertainty, confidentiality expectations. Not identity-specific data — the same SOUL.md template applies across a role type, whereas PROFILE.md is instance-specific.

```yaml
---
summary: "Core persona principles for this agent role"
read_when:
  - Session start
---
```

Key principle themes:

- Be genuinely helpful, not performative.
- Have opinions; give recommendations, not just options.
- Be resourceful before asking; search before interrupting.
- Earn trust incrementally; ask for access as needed.
- Treat access as intimacy — never accumulate permissions beyond the current task.
- Sessions start fresh, but the files are the memory — always read them before acting.

### 4.3 PROFILE.md — identity and user profile

```yaml
---
summary: "Agent identity and user profile"
read_when:
  - Bootstrapping a workspace manually
  - First session of the day
---
```

Two sections:

**Agent identity** — set during bootstrap and refined by the user:

```plaintext
## Identity
- Name:      (assigned during bootstrap)
- Role type: (e.g. engineer, researcher, analyst)
- Vibe:      (e.g. sharp + methodical, warm + curious)
```

**User profile** — built over time by the agent as it learns about the user:

```plaintext
## User Profile
- Name:
- What to call them:
- Notes:

### Context
(What do they care about? What projects are they working on?
 What annoys them? What helps them most? Updated after sessions.)
```

The agent appends to `### Context` as it learns; it never overwrites the identity block without permission.

### 4.4 MEMORY.md — tool cheat sheet

```yaml
---
summary: "Agent long-term memory — tool setup and lessons learned"
read_when:
  - Bootstrapping a workspace manually
---
```

Free-form cheat sheet the agent builds: SSH aliases, workspace-specific paths, known environment quirks, patterns that have proven useful. Not credentials — those stay in `.env`.

Example entries:

```plaintext
### SSH
- home-server → 192.168.1.100, user: admin

### Known Quirks
- The build system caches in ~/.local/share/<app>; clearing it fixes stale artifacts.
```

### 4.5 HEARTBEAT.md — periodic check list

```yaml
---
summary: "Workspace template for HEARTBEAT.md"
read_when:
  - Bootstrapping a workspace manually
---
```

A list of tasks for the agent to check proactively on each heartbeat schedule fire. **An empty file (or comments only) suppresses all heartbeat calls** — the scheduler skips the heartbeat action entirely when HEARTBEAT.md is empty.

When populated, entries are simple task lines:

```plaintext
- Check for stalled tasks in the kanban board
- Summarize unread messages
- Verify connectivity to configured services
```

The heartbeat scheduler fires these on the workspace's `heartbeat`-action schedule.

### 4.6 BOOTSTRAP.md — first-run ritual

```yaml
---
summary: "First-run ritual — guides initial workspace setup"
read_when:
  - First session (file is absent after completion)
---
```

Read exactly once per workspace. Guides the agent through the initial setup ceremony:

1. Learn the agent's name, role, and vibe from the user.
2. Update `PROFILE.md` identity block with learned values.
3. Open and explain `SOUL.md` together with the user.
4. **Delete `BOOTSTRAP.md`** — this is the last instruction; its absence on future sessions signals bootstrap is complete.

The agent must not skip the deletion step. If BOOTSTRAP.md is present at session start, bootstrap is incomplete.

### 4.7 Load order

When loading a session, the agent reads constitution files as follows:

1. Check for BOOTSTRAP.md — if present, run bootstrap ritual (§4.6) and stop normal load.
2. Load SOUL.md (always).
3. Load PROFILE.md and MEMORY.md (when `read_when` conditions match current context).
4. Check HEARTBEAT.md to determine whether a heartbeat schedule is meaningful.

Each file's `read_when` list guides lazy loading; files not matching conditions are skipped for that session.

### 4.8 Persona wizard

When a new agent workspace is first bootstrapped interactively, BOOTSTRAP.md guides a 3-question wizard to populate the persona fields before the agent's first active session. The wizard runs through the CLI or TUI bootstrap ritual; non-interactive installs accept `--agent-name`, `--voice`, and `--focus` flags instead.

```text
[REFERENCE]
Wizard prompts (in order):
  1. "What should we call this assistant?"        → PROFILE.md Identity.Name
  2. "Voice style (one word: direct / warm / clear / playful / analytical)"
                                                  → PROFILE.md Identity.Vibe
  3. "Primary job-to-be-done (one line)"          → SOUL.md Focus header

Behavior rules:
  - Existing values are displayed as defaults; Enter accepts them.
  - A blank answer keeps the existing default — never clears it.
  - After the wizard completes, PROFILE.md and SOUL.md are updated atomically (write-then-rename).
  - The wizard exit line mirrors the result: "Persona set: <Name>, <Voice>, <Focus>"
```

The wizard also offers an optional GitHub backup for the constitution directory — if the user answers yes and the `gh` CLI is present, the workspace constitution folder is pushed to a private repository. This is opt-in and skipped silently when `gh` is absent.

### 4.9 Communication contract

SOUL.md must include a **Communication contract** section that constrains how the agent presents output. This section is read at every session start (it is part of SOUL.md's `read_when: [Session start]`).

```text
[REFERENCE]
Required subsections within SOUL.md Communication contract:

Style rules:
  - BLUF (bottom line up front) — lead every response with the conclusion, not the buildup.
  - No hedging language ("I think", "you might find", "it seems").
  - Short, declarative sentences. No em-dash abuse.
  - Practitioner voice — describe situations and decisions, not features.

Contradiction handling:
  - Flag contradictions explicitly; do NOT silently resolve them.
  - Record the contradiction in <ws>/contradictions.md with the conflicting sources.
  - Ask for the user's decision rather than picking a side.

Source citation:
  - Cite vault sources with [[wikilink]] syntax.
  - Only fall back to web research when the vault has no relevant material.
  - Search the vault first for any question about the user's domain.

Vocabulary preferences (per workspace):
  - A "use:" list and an "avoid:" list; both are free-form.
  - The lists travel in SOUL.md (user-editable).
  - Example avoid patterns: AI jargon, corporate buzzwords, named AI persona mascots.

Does / does-NOT-do contract:
  - Two short lists in SOUL.md: what the agent actively does, and what it refuses.
  - Does-NOT-do entries are invariants, not suggestions — they require explicit user override to lift.
  - Examples of does-NOT-do invariants:
      "modify reviewed/stable documents without explicit instruction"
      "push to a remote git repository without being asked"
      "fabricate information not present in sources or web research"
      "resolve contradictions silently"
```

The does/does-NOT-do contract is particularly valuable when the agent is running scheduled tasks autonomously — it limits the blast radius of any misfire without requiring the user to be present.

## 5. Drawbacks & Alternatives

- **Plaintext size vs. structured memory:** PROFILE.md and MEMORY.md grow unbounded. Long-term, entries should be migrated to the structured memory store; the constitution files remain the user-visible identity layer.
- **Concurrent writes (multi-agent):** two agents updating PROFILE.md simultaneously could cause lost updates. Mitigation: the `manager` role (orchestrator) is the designated writer; others read-only unless specifically granted write access.
- **SOUL.md drift:** if the user edits SOUL.md, the agent's behavior changes on next session. This is by design — SOUL.md is a first-class configuration surface.
- **Alternative — store identity in structured DB:** more queryable, but not user-editable without tooling. Plain files win for transparency and auditability.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[OFFICE]` | `.design/main/specifications/l1-office-model.md` | Persona and role model |
| `[MEMORY]` | `.design/main/specifications/l1-memory-model.md` | Memory scopes and lifecycle |
| `[LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | Workspace layout |
| `[SCHED]` | `.design/main/specifications/l2-scheduler.md` | Heartbeat schedule action |

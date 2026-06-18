# Kanban Board

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-kanban-model.md

## Overview

The concrete realization of the office board: where the single canonical board lives in the workspace, how cards and their state are stored, how transitions and blocking work, how `done` cards are auto-archived to a durable store, and the board command surface across CLI / TUI / library.

## Related Specifications

- [l1-kanban-model.md](l1-kanban-model.md) - The board model this implements.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - The `kanban/` location within a workspace.
- [l2-core-library.md](l2-core-library.md) - Hosts the board service and the archival job.
- [l2-cli.md](l2-cli.md) - Command grammar standard the board commands follow.

## 1. Motivation

The model requires a single per-office board, office-driven movement, and automatic, non-destructive archival. A file-backed board under the workspace keeps it isolated and inspectable; an archival job keeps the active board lean without losing history.

## 2. Constraints & Assumptions

- One board per workspace, stored under `<ws>/kanban/`; archived cards under `<ws>/kanban/archive/`.
- The fixed state set from the model; no user-defined boards in v0.1.0.
- The frontend holds no logic; board operations are core calls (INV-2).
- Card content references the office's tasks; the board does not duplicate task bodies.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| KAN-1 Canonical pipeline | A single board with the fixed ordered states; state is an enum, not user-editable. |
| KAN-2 Office-managed | Manager/agents call `board.move`; the client UI is read-first; no client setup required. |
| KAN-3 Auto-archival | A scheduled archival job moves `done` cards meeting the condition into `<ws>/kanban/archive/`. |
| KAN-4 Non-destructive archive | Archived cards are moved (not deleted); they remain readable in the archive store. |
| KAN-5 Card = unit of work | Each card record references a task and carries `state`; `blocked` requires a `reason`. |
| KAN-6 One board / isolation | Exactly one board per `<ws>/kanban/`; no cross-office board. |
| KAN-7 Traceable transitions | Each move appends a transition record (from, to, actor, time, reason). |

## 4. Detailed Design

### 4.1 Storage

```plaintext
<ws>/kanban/
├── board.json        # board meta + ordered state set + card index
├── cards/<card-id>.json   # one card per file (state, task ref, reason?, history[])
└── archive/<card-id>.json # auto-archived done cards (history preserved)
```

Card record (conceptual):

```text
[REFERENCE]
{
  id, task_ref, state,            // state in {triage,todo,ready,running,blocked,done}
  reason,                          // required when state = blocked (KAN-5)
  history[ {from, to, actor, at, reason} ],  // KAN-7 traceability
  created_at, updated_at
}
```

### 4.2 Transitions

```mermaid
graph LR
    TRIAGE[triage] --> TODO[todo] --> READY[ready] --> RUNNING[running] --> DONE[done]
    RUNNING --> BLOCKED[blocked] --> READY
```

Forward flow is normal; backward moves (`blocked → ready`, `running → todo`) are allowed but explicit. Every move appends to the card's `history`.

### 4.3 Auto-archival job

A scheduled job (run by the core, owned operationally by the archivist/curator) scans `done` cards and moves those meeting the condition into `archive/`. Default condition is configurable. <!-- TBD: default condition — age threshold (e.g. N days in done) vs only-on-project-closure -->

### 4.4 Command surface

Board operations across all three surfaces, conforming to the CLI grammar standard (verb-first, explicit verbs; see `l2-cli.md` §4.4). The library method is the source.

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| show board | `cronus board show` | `/board show` | `board.show() -> Board` |
| list cards | `cronus board list [--state <s>]` | `/board list …` | `board.list({state?}) -> Card[]` |
| move card | `cronus board move <card-id> <state>` | `/board move <card-id> <state>` | `board.move(cardId, state) -> Card` |
| block card | `cronus board block <card-id> --reason <r>` | `/board block …` | `board.block(cardId, reason) -> Card` |
| unblock card | `cronus board unblock <card-id>` | `/board unblock <card-id>` | `board.unblock(cardId) -> Card` |
| archive (manual override) | `cronus board archive <card-id>` | `/board archive <card-id>` | `board.archive(cardId) -> void` |

Cards are created from the office's tasks by the manager (not a client command in v0.1.0); the client surface is primarily `show`/`list`. `move`/`block`/`unblock` are the office's operations, exposed for tooling/automation.

## 5. Drawbacks & Alternatives

- **File-per-card vs single board file:** per-card files ease concurrent updates and history; the index in `board.json` keeps reads cheap.
- **Alternative — store board rows in SQLite:** viable later for large boards; v0.1.0 uses files for inspectability (consistent with STO-8). <!-- TBD: switch to SQLite-backed board if card counts grow large -->
- **Alternative — manual archive column:** rejected (KAN-3/OFF-5).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-kanban-model.md` | Invariants this board satisfies |
| `[LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | `kanban/` location in a workspace |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

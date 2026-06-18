# Scheduler

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-scheduler-model.md

## Overview

The concrete scheduler: a friendly recurrence model (alarm-clock style presets) with an optional raw cron expression for power users, per-workspace storage, the timezone/firing semantics, and the schedule command surface across CLI / TUI / library. Board de-duplication for routine fires is deferred per the model.

## Related Specifications

- [l1-scheduler-model.md](l1-scheduler-model.md) - The model this implements.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - The `schedules/` location within a workspace.
- [l2-kanban-board.md](l2-kanban-board.md) - Where `routine` fires may place cards.
- [l2-cli.md](l2-cli.md) - Command grammar standard the schedule commands follow.

## 1. Motivation

The model wants recurrence-first scheduling that a non-technical client can use, while still allowing exact control. A friendly schedule object covers the common cases; an optional cron field serves the rest. File-backed per-workspace storage keeps schedules isolated and inspectable.

## 2. Constraints & Assumptions

- Per-workspace storage under `<ws>/schedules/`.
- Friendly recurrence and raw cron are mutually exclusive on a single schedule.
- Firing uses the host clock and the schedule's timezone (default: host timezone).
- The frontend holds no logic; scheduling is a core service (INV-2).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| SCH-1 Two kinds | `kind: recurring|oneshot`; one-shot sets `delete_after_fire: true` and is removed after firing. |
| SCH-2 Recurrence expressiveness | `recurrence` presets (weekdays/weekends/daily/days/interval + times) OR a raw `cron` string. |
| SCH-3 Single declared action | `action: heartbeat|routine|reminder` — exactly one per schedule. |
| SCH-4 Wake produces no card | `heartbeat` fires call only the office wake entry point; no board API is invoked. |
| SCH-5 Workspace-scoped | Schedule files live under `<ws>/schedules/`; firing targets that office only. |
| SCH-6 Autonomous & durable | Schedules persist as files; the scheduler service reloads and re-arms them on restart. |
| SCH-7 Lifecycle control | `enabled` flag plus create/edit/delete operations; disabling stops firing without deletion. |

## 4. Detailed Design

### 4.1 Schedule object (conceptual)

```text
[REFERENCE]
{
  id, name,
  kind: "recurring" | "oneshot",
  action: "heartbeat" | "routine" | "reminder",
  recurrence: {                 // friendly model (mutually exclusive with cron)
    preset: "weekdays" | "weekends" | "daily" | "days" | "interval",
    days?: ["mon","tue", ...],  // for preset "days"
    times?: ["09:00", "18:30"], // local times
    interval?: "15m"            // for preset "interval" (heartbeat cadence)
  },
  cron?: "0 9 * * 1-5",         // advanced: raw cron, instead of recurrence
  at?: "2026-07-01T09:00",      // for oneshot
  start_at?, end_at?,           // optional window
  timezone,                     // default: host
  enabled: true,
  delete_after_fire: false      // true for oneshot (SCH-1)
}
```

### 4.2 Storage

```plaintext
<ws>/schedules/
└── <schedule-id>.json   # one schedule per file
```

### 4.3 Firing

The scheduler service evaluates due schedules against the host clock + timezone, fires the declared action, and for one-shot schedules deletes the file after firing. On restart it reloads all files and re-arms (SCH-6). Missed-fire handling during downtime is configurable. <!-- TBD: missed-fire behavior (skip vs catch-up) -->

### 4.4 Board interaction (deferred)

`heartbeat` never touches the board (SCH-4). `routine` may create a board card; the de-duplication/coalescing policy that would prevent repeated routine fires from accumulating duplicate cards is **deferred** and will be decided after real-world testing — v0.1.0 does not implement coalescing. <!-- TBD: routine-fire board de-duplication policy (coalesce vs skip) -->

### 4.5 Command surface

Schedule operations across all three surfaces, conforming to the CLI grammar standard (verb-first, explicit verbs; see `l2-cli.md` §4.4). The library method is the source.

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| create (friendly) | `cronus schedule create <name> --action <a> --when <preset> [--time HH:MM] [--days mon,tue] [--every 15m]` | `/schedule create …` | `schedule.create(name, spec) -> Schedule` |
| create (one-shot) | `cronus schedule create <name> --action <a> --once --at <datetime>` | `/schedule create … --once` | `schedule.create(name, spec) -> Schedule` |
| create (raw cron) | `cronus schedule create <name> --action <a> --cron "<expr>"` | `/schedule create … --cron …` | `schedule.create(name, spec) -> Schedule` |
| list | `cronus schedule list` | `/schedule list` | `schedule.list() -> Schedule[]` |
| show | `cronus schedule show <id>` | `/schedule show <id>` | `schedule.get(id) -> Schedule` |
| set | `cronus schedule set <id> [--when …] [--cron …] [--time …] [--enabled true|false]` | `/schedule set <id> …` | `schedule.update(id, patch) -> Schedule` |
| enable | `cronus schedule enable <id>` | `/schedule enable <id>` | `schedule.enable(id) -> Schedule` |
| disable | `cronus schedule disable <id>` | `/schedule disable <id>` | `schedule.disable(id) -> Schedule` |
| delete | `cronus schedule delete <id>` | `/schedule delete <id>` | `schedule.delete(id) -> void` |
| run now | `cronus schedule run <id>` | `/schedule run <id>` | `schedule.runNow(id) -> void` |

## 5. Drawbacks & Alternatives

- **Two recurrence representations (friendly + cron):** a small translation/validation cost; justified by serving both audiences (SCH-2).
- **File-per-schedule:** simple and inspectable; if schedules grow large, an index or SQLite-backed store can be introduced later (consistent with STO-8).
- **Coalescing deferred:** accepted risk of duplicate routine cards until tuned in real use (§4.4).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-scheduler-model.md` | Invariants this scheduler satisfies |
| `[LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | `schedules/` location in a workspace |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

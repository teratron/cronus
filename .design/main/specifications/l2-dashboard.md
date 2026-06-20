# Dashboard

**Version:** 1.0.1
**Status:** Stable
**Layer:** implementation
**Implements:** l1-dashboard.md

## Overview

The concrete dashboard: which metrics it computes, the state it reads them from, the per-office and home building-aggregate views, where view layout persists, and the `dashboard` command.

## Related Specifications

- [l1-dashboard.md](l1-dashboard.md) - The model this implements.
- [l2-kanban-board.md](l2-kanban-board.md) - Board state for work metrics.
- [l2-app-ui.md](l2-app-ui.md) - The Dashboard surface in the app shell.
- [l2-cli.md](l2-cli.md) - Command grammar standard.

## 1. Motivation

The model needs concrete metric definitions and sources so the dashboard is a faithful, live projection that costs nothing to keep consistent.

## 2. Constraints & Assumptions

- Metrics are computed on read from existing state; only cosmetic layout is stored under `<ws>/dashboard/`.
- The home aggregate reads across `<state>/workspaces/*` read-only.
- The frontend renders; metric computation is a core call (INV-2).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| DSH-1 Projection | Metrics computed from board, sessions, cost ledger, schedules, memory; nothing authoritative stored. |
| DSH-2 Live | Recomputed on state-change events / refresh. |
| DSH-3 Per-office + building | `dashboard show` for the active office; in home, an aggregate across all offices. |
| DSH-4 Observational | Read-only; no office operations from the dashboard. |
| DSH-5 Privacy | Metrics local; sharing only via telemetry opt-in. |
| DSH-6 Isolation | Office dashboard reads only its workspace; building aggregate reads across offices read-only. |

## 4. Detailed Design

### 4.1 Metrics and sources

| Metric | Source |
| --- | --- |
| cards by state, throughput, cycle time, blocked | `<ws>/kanban/` |
| active agents, running tasks, recent sessions | roster + `<ws>/sessions/` |
| cost/budget usage | cost ledger (per office/agent budgets) |
| schedule status (upcoming/overdue, heartbeat) | `<ws>/schedules/` |
| memory size by scope, recent learnings | memory stores |

### 4.2 Layout storage

```plaintext
<ws>/dashboard/
└── layout.json   # cosmetic widget arrangement; presentation only
```

### 4.3 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| show office dashboard | `cronus dashboard` | `/dashboard` | `dashboard.show() -> Dashboard` |
| building aggregate (home) | `cronus dashboard building` | `/dashboard building` | `dashboard.building() -> Dashboard` |

### 4.4 Surface catalog

The dashboard organizes content into named surfaces. Each surface is a bounded read-only or lightly-interactive view; no surface triggers agent operations directly (DSH-4). The active surface is tracked in `layout.json` as a persisted preference.

```text
[REFERENCE]
Surface catalog (all surfaces read-only toward domain state):

Today        — Hero surface. Synthesized standup brief (from the agent's latest standup skill
               run); vault/knowledge activity timeline with relative timestamps; keyboard
               quick-launch for pinned skills (up to 4); 30-day write-sparkline showing
               output volume trend; agent runtime status panel (next scheduled run countdown).

Skills       — Skill pack browser. Pack cards grouped by function (e.g. ceo / engineering /
               finance). Click into a pack → skill cards. Click skill → input form → run.
               Output streams back; artifact reveals in an accent-bordered panel with
               "Open in Knowledge Base" action. Source indicator: shipped vs. user-override.

Knowledge    — Tree view of the workspace knowledge base on the left, recently-updated list
               on the right with status-tinted pills (draft / reviewed / stable). Markdown
               render with [[wikilink]] resolution, code highlighting, frontmatter inspector.

Journal      — Chronological personal entries. User-owned zone — agent reads but does not
               write here (except via explicit journal-scaffold skill invocations).

Sources      — Ingest surface. Drag-drop PDFs, URLs, or transcripts → land as "unread" in
               the sources zone. The agent picks them up via a skill and synthesizes wiki
               pages; ingest status updates here.

Automations  — Live view of every scheduled job registered with the workspace cron system.
               Shows: job name, last-run status, next-run timestamp, enabled/paused state.
               Surface-level pause/resume actions (write-path via the bridge pattern — see
               §4.6 below).

Settings     — Runtime config, theme toggle, update-check status, search-tool detection
               report, token status, vault path.
```

### 4.5 Session health score

The dashboard computes a health score [0–100] for each active role/agent instance. The score is a read-only diagnostic — it is never stored; always recomputed from live state.

```text
[REFERENCE]
health_score(profile) -> u8:
  score = 100

  // Error log pressure
  if error_lines > 100: score -= 15
  elif error_lines > 50: score -= 10
  elif error_lines > 20: score -= 5

  // Memory/process pressure
  if memory_mb > 300: score -= 20
  elif memory_mb > 200: score -= 10

  // Engagement (messages per session ratio)
  if session_count > 0 AND (message_count / session_count) < 5:
    score -= 10

  // Activity trend (last 7 days vs prior 7 days)
  if trend_window >= 14 days:
    recent_avg = avg(daily_sessions, last 7 days)
    prior_avg  = avg(daily_sessions, days 8–14)
    if prior_avg > 0 AND (recent_avg / prior_avg) < 0.5:
      score -= 15

  return clamp(score, 0, 100)

Color coding:
  score >= 80  → green
  score >= 60  → amber
  score <  60  → red
```

### 4.6 Alert thresholds

Alerts are surfaced in the Today surface and in the Automations surface. Each alert has a severity, a human-readable title/detail pair, the metric value, and the threshold that triggered it.

```text
[REFERENCE]
Alert { severity: critical | warning | info, title, detail, metric, current, threshold }

Threshold table:
  memory_mb > 300      → critical  "Memory Pressure"      detail: risk of OOM
  memory_mb > 200      → warning   "High Memory Usage"    detail: monitor closely
  error_lines > 100    → warning   "Elevated Errors"      detail: review for patterns
  session_count == 0   → info      "Inactive Profile"     detail: gateway up but unused
  activity_drop > 50%  → warning   "Activity Drop"        detail: session avg dropped N%

Sorting: critical first, then warning, then info; same-severity sorted by recency.
Display cap: show up to 10 alerts in the UI; the full list is available via dashboard.alerts().
```

### 4.7 Graceful degradation (runtime offline)

When the agent runtime's HTTP API is offline, the dashboard degrades gracefully to disk-based reads rather than showing an error state.

```text
[REFERENCE]
Degradation map (HTTP endpoint → disk fallback):

  GET /api/jobs               → read <state>/cron/jobs.json directly
  GET /health/detailed        → read <state>/gateway_state.json directly
  Session listing             → read <state>/state.db via SQLite (WAL mode, readonly)

  Fallback contract:
    - Disk reads are attempted silently; a banner ("Runtime offline — showing cached state")
      is shown once in the Settings surface; all other surfaces render normally from disk data.
    - Disk fallback is read-only: pause/resume/run actions in the Automations surface are
      disabled while offline (not hidden — shown as disabled with a tooltip).
    - The Today surface shows the most recent standup from the knowledge base rather than
      triggering a new run.
    - Health score and alerts are suppressed (data not current) and replaced with an
      "Offline — connect runtime for live health data" notice.
```

## 5. Drawbacks & Alternatives

- **Recompute cost on busy offices:** mitigated by event-driven incremental metric updates.
- **Alternative — persist computed metrics:** rejected; storing derived numbers risks drift (DSH-1). Only layout persists.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DASHBOARD]` | `.design/main/specifications/l1-dashboard.md` | Invariants this implements |
| `[BOARD]` | `.design/main/specifications/l2-kanban-board.md` | Work-metric source |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

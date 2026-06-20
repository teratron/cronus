# Self-Improvement

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-memory-model.md

## Overview

The self-improvement subsystem tracks what the agent gets wrong and distills what it gets right into a brief surface consulted at the start of every task. Five signal types feed into a single `build_brief()` call that joins them against the current working context (project + files + optional task type and domain).

## Related Specifications

- [l1-memory-model.md](l1-memory-model.md) - The model this subsystem extends.
- [l2-memory-store.md](l2-memory-store.md) - Episode store whose outcomes feed calibration and mistake pipelines.
- [l2-learning-loop.md](l2-learning-loop.md) - Dream cycle that runs the `templates` extraction phase.
- [l2-agent-session.md](l2-agent-session.md) - Session outcomes trigger ask-back generation and calibration updates.
- [l2-github-issue.md](l2-github-issue.md) - Error fingerprinting that feeds the mistake log.

## 1. Motivation

A system that only remembers facts gets smarter about the world; a system that also tracks its own failure modes gets smarter about how it works. Calibration catches overconfidence before it compounds. Mistake logs stop the same error from appearing twice in the same file. Should-have-asked prevents repeat blind spots. Ask-backs turn confused sessions into answered questions. Reasoning templates accumulate proven playbooks from successful episodes.

## 2. Constraints & Assumptions

- All five signal tables share the same SQLite database as the background job queue (`<state>/jobs.sqlite`), avoiding an extra database file.
- Store-open failures for any section produce empty signals, not errors. A partial brief is better than no brief.
- The brief surface joins all five signals at task start; sections are omitted entirely when empty.
- Ask-backs are system-generated (drafted by the session pipeline, not the human); they surface as the highest-priority item in the brief.
- Calibration operates at `(task_type, project)` granularity — broad enough to have signal, narrow enough to be actionable.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| MEM-6 Compounding, non-destructive | Calibration updates are additive; mistakes are append-only; templates are upserted in place on re-runs. |
| MEM-7 Ownership split | Session outcomes write calibration and ask-back rows; the dream cycle's `templates` phase writes templates; no agent writes tables directly. |
| MEM-8 Classified & tagged | Mistakes carry a `category` column; should-have-asked rows carry a `trigger` tag for deterministic lookup. |

## 4. Detailed Design

### 4.1 Calibration buckets

Per-`(task_type, project)` tracking of declared versus verified success ratios. The key signal is **overconfidence**: the agent declaring success more often than outcomes are actually verified.

#### Schema

```sql
-- [REFERENCE] illustrative, not final DDL
CREATE TABLE calibration_buckets (
    task_type        TEXT NOT NULL,
    project          TEXT NOT NULL,
    declared_success INTEGER NOT NULL DEFAULT 0,
    verified_success INTEGER NOT NULL DEFAULT 0,
    declared_failure INTEGER NOT NULL DEFAULT 0,
    refuted_success  INTEGER NOT NULL DEFAULT 0,
    last_updated     INTEGER NOT NULL,
    PRIMARY KEY (task_type, project)
);
```

#### Overconfidence formula

```text
[REFERENCE]
overconfidence  = max(0, 1 − verified_success / max(declared_success, 1))
verified_ratio  = verified_success / max(declared_success, 1)
```

#### Brief warning gate

```text
[REFERENCE]
CALIBRATION_MIN_SAMPLE_FOR_WARN = 5    // minimum declared_success before warning fires
VERIFIED_RATIO_WARN_THRESHOLD   = 0.50 // warn when < 50 % of declared successes are verified
```

When `declared_success >= 5` AND `verified_ratio < 0.50`, a calibration warning is shown in the brief for the matching `(task_type, project)` pair. This urges the agent to verify outcomes before declaring done, rather than accumulating false successes.

### 4.2 Mistake log

An anchored correction log. Each row records what went wrong, which files were involved, and what the correct approach was. The brief surfaces the top-N mistake categories for files about to be edited, so recurring failure patterns reach the agent before it repeats them.

#### Schema

```sql
-- [REFERENCE] illustrative, not final DDL
CREATE TABLE mistakes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project     TEXT NOT NULL,
    category    TEXT NOT NULL,             -- e.g. "lifetime_annotations", "auth_order"
    episode_id  TEXT,                      -- source episode ID, when available
    files       TEXT NOT NULL DEFAULT '[]', -- JSON array of file paths
    description TEXT NOT NULL,
    correction  TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);
CREATE INDEX mistakes_proj_cat ON mistakes(project, category, created_at DESC);
```

#### Brief query

```text
[REFERENCE]
top_categories_for_files(project, files, limit) -> Vec<CategoryCount>:
  SELECT category,
         count(*)       AS count,
         max(created_at) AS last_seen
  FROM mistakes
  WHERE project = ? AND json_overlaps(files, ?)
  GROUP BY category
  ORDER BY count DESC
  LIMIT ?
```

`CategoryCount` carries an optional `source_project` field to enable cross-project mode tagging (§4.6.1).

### 4.3 Should-have-asked

Pre-task question gaps: situations where a clarifying question asked before starting would have prevented a mistake or resolved an ambiguity. Keyed by `(project, trigger)` where `trigger` is a normalized context signal derived from the files being edited (e.g. editing `src/auth/middleware.rs` → trigger `"edit_auth_middleware"`).

#### Schema

```sql
-- [REFERENCE] illustrative, not final DDL
CREATE TABLE should_have_asked (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    project    TEXT NOT NULL,
    trigger    TEXT NOT NULL,             -- normalized kebab/snake_case context signal
    question   TEXT NOT NULL,
    answer     TEXT NOT NULL,
    episode_id TEXT,
    files      TEXT NOT NULL DEFAULT '[]', -- JSON array
    created_at INTEGER NOT NULL
);
CREATE INDEX shas_by_proj_trigger ON should_have_asked(project, trigger, created_at DESC);
```

#### Brief query

```text
[REFERENCE]
triggers_for_files(project, files, limit) -> Vec<ShouldHaveAsked>:
  SELECT DISTINCT trigger, *
  FROM should_have_asked
  WHERE project = ? AND json_overlaps(files, ?)
  ORDER BY created_at DESC
  LIMIT ?
```

The trigger string normalisation allows path-based pattern matching without brittle exact-path comparisons. Future revisions may embed triggers and match by semantic similarity instead.

### 4.4 Ask-backs

System-generated clarifying questions. When a session ends in failure or partial outcome with vague user intent, the session pipeline drafts one question and queues it as a pending ask-back for the next interaction.

**Key invariant**: at most one pending ask-back per project at any time. This prevents question flooding — only the single most-relevant open question is surfaced in the brief.

#### Schema

```sql
-- [REFERENCE] illustrative, not final DDL
CREATE TABLE ask_backs (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    project    TEXT NOT NULL,
    episode_id TEXT NOT NULL,
    question   TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'pending', -- pending|served|dismissed
    model      TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    served_at  INTEGER
);
-- At-most-one-pending-per-project debounce (enforced at DB level):
CREATE UNIQUE INDEX ask_backs_one_pending_per_proj
    ON ask_backs(project) WHERE status = 'pending';
```

The partial UNIQUE INDEX enforces the invariant at the database level: an `INSERT` for a project that already has a pending ask-back fails immediately, so only the first question per session is stored without application-layer coordination.

#### Status transitions

```text
pending  → served     (surfaced in brief; agent confirmed the user saw it)
pending  → dismissed  (agent rejected the question as irrelevant or stale)
served   → (terminal)
dismissed → (terminal)
```

#### Brief integration

The pending ask-back is always the **first** signal in the brief and is rendered with a high-priority marker. The user must answer it before the next task starts; the answer feeds back into `should_have_asked` once resolved.

### 4.5 Reasoning templates

Proven step-sequences extracted from successful episode clusters by the dream cycle's `templates` phase. Keyed by `(task_type, domain)` — one template per pair, upserted in place on each re-extraction so the row accumulates evidence over time rather than duplicating.

#### Schema

```sql
-- [REFERENCE] illustrative, not final DDL
CREATE TABLE reasoning_templates (
    id                TEXT NOT NULL PRIMARY KEY,
    task_type         TEXT NOT NULL,
    domain            TEXT NOT NULL,
    name              TEXT NOT NULL,
    steps             TEXT NOT NULL,               -- JSON array of step strings
    evidence_episodes TEXT NOT NULL,               -- JSON array of episode IDs
    success_rate      REAL NOT NULL,
    times_used        INTEGER NOT NULL DEFAULT 0,
    model             TEXT NOT NULL,
    created_at        INTEGER NOT NULL,
    last_used         INTEGER,
    UNIQUE (task_type, domain)
);
```

#### Dream cycle extraction

The `templates` dream phase groups successful episodes by `(task_type, domain)` and clusters those with at least `templates_min_evidence` members (default: 3). For each qualifying cluster, the extraction model produces a `name` and `steps[]` JSON array. The phase upserts into `reasoning_templates` — same row updated on re-run, accumulating `evidence_episodes` and refreshing `success_rate`.

#### Brief lookup

```text
[REFERENCE]
get_by_pair(task_type, domain) -> Option<Template>:
  SELECT * FROM reasoning_templates
  WHERE task_type = ? AND domain = ?
  LIMIT 1
```

The template is only surfaced when both `task_type` and `domain` are provided to `build_brief`. A template with a high `success_rate` and many `evidence_episodes` is a strong signal.

### 4.6 Brief surface

A single call that joins all five signals against the current working context. Called once at task start when the file set is known. Each signal source opens its own store connection independently; any failure yields an empty/`None` for that section without failing the brief.

#### API

```text
[REFERENCE]
build_brief(
    project:   &str,
    files:     &[String],
    task_type: Option<&str>,
    domain:    Option<&str>,
) -> Brief
```

#### Brief struct

```text
[REFERENCE]
Brief {
    project:                   String,
    files:                     Vec<String>,
    task_type:                 Option<String>,
    domain:                    Option<String>,
    pending_ask_back:          Option<AskBack>,            // §4.4 — highest priority
    template:                  Option<Template>,            // §4.5 — only when task+domain provided
    top_correction_categories: Vec<CategoryCount>,         // §4.2 — top-N for these files
    should_have_asked_triggers: Vec<ShouldHaveAsked>,      // §4.3 — trigger-matched to files
    calibration_warning:       Option<CalibrationWarning>, // §4.1 — fires on low verified ratio
}
```

#### Constants

```text
[REFERENCE]
BRIEF_TOP_CATEGORIES            = 5    // max correction categories surfaced per brief
BRIEF_TOP_ASKS                  = 5    // max should-have-asked rows surfaced per brief
VERIFIED_RATIO_WARN_THRESHOLD   = 0.50 // calibration section fires below this
CALIBRATION_MIN_SAMPLE_FOR_WARN = 5    // minimum declared_success for calibration section
```

#### Rendering order

1. **Pending ask-back** (§4.4) — most actionable, highest priority
2. **Reasoning template** (§4.5) — proven playbook for this task type
3. **Top correction categories** (§4.2) — what keeps going wrong in these files
4. **Should-have-asked triggers** (§4.3) — questions to ask before starting
5. **Calibration warning** (§4.1) — low verified-success rate on this task type

Sections are omitted entirely when empty to preserve context-window budget.

#### 4.6.1 Cross-project mode

When `cross_project = true`, the correction categories (§4.2) and should-have-asked (§4.3) sections include rows from other projects whose `files` lists overlap with the current file set. Each foreign row carries a `source_project` field so the renderer can tag it `[from <project>]`. The ask-back (§4.4), calibration (§4.1), and template (§4.5) sections are always project-scoped — those signals do not generalise across project boundaries.

## 5. Drawbacks & Alternatives

- **Trigger normalization is heuristic**: path-to-trigger mapping covers file-level patterns but misses cross-cutting concerns. Future: embed triggers and match by semantic similarity.
- **At-most-one ask-back**: may miss multiple orthogonal open questions from a single session. The constraint is a deliberate anti-flooding measure; multi-question support would require a queue with relevance ranking.
- **One template per (task_type, domain)**: a single template per pair may not cover all sub-patterns within a domain. Future: allow multiple ranked templates, or template hierarchies.
- **Calibration ratio as overconfidence proxy**: a simple declared-vs-verified ratio is cheap but noisy. An outcome-verification pass (comparing stated results to observable facts) would be more precise but requires LLM calls per session.
- **Alternative — shared mistake table with memory store**: the mistake log could live in the per-scope memory SQLite rather than the jobs database. Chosen against: the jobs database is already shared infrastructure; mistake data is operational, not episodic knowledge.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MEMORY]` | `.design/main/specifications/l2-memory-store.md` | Episode store that feeds calibration |
| `[LEARNING]` | `.design/main/specifications/l2-learning-loop.md` | Dream cycle that extracts templates |
| `[SESSION]` | `.design/main/specifications/l2-agent-session.md` | Session outcomes that write ask-backs |
| `[ERRORS]` | `.design/main/specifications/l2-github-issue.md` | Error fingerprinting feeding mistake log |

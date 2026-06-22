---
phase: 5
name: "Office Work Engine"
status: Done
subsystem: "crates/core (new modules)"
requires:
  - "memory store (crates/core/src/memory/) — Phase 4 T-4A01"
  - "agent session loop (crates/core/src/session/) — Phase 4 T-4C01"
  - "agent autonomy (crates/core/src/autonomy.rs) — Phase 4 T-4C05"
  - "session checkpoint (crates/core/src/checkpoint.rs) — Phase 4 T-4C03"
  - "model router (crates/core/src/router/) — Phase 4 T-4B01"
  - "workspace management (crates/core/src/workspace.rs) — Phase 4 T-4D01"
  - "codegraph crate (crates/codegraph/) — Phase 4 T-4A03"
  - "CLI command dispatch (crates/cli/src/commands.rs) — Phase 3 T-3A01"
provides:
  - "Tool Security module (crates/core/src/tool_security.rs)"
  - "Role Catalog module (crates/core/src/roles/)"
  - "Kanban Board module (crates/core/src/kanban/)"
  - "Scheduler module (crates/core/src/scheduler/)"
  - "Budget Engine module (crates/core/src/budget.rs)"
  - "Execution Workspace module (crates/core/src/exec_workspace.rs)"
  - "Quality Pipeline module (crates/core/src/quality.rs)"
  - "Extension Registry module (crates/core/src/extensions/)"
  - "Plugin Hooks module (crates/core/src/hooks.rs)"
  - "Agent Registry module (crates/core/src/agent_registry.rs)"
  - "Learning Loop module (crates/core/src/learning.rs)"
  - "Phase 5 CLI commands (role/board/schedule/ext/check/registry subgroups)"
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Phase 5 Tasks — Office Work Engine

**Phase:** 5
**Status:** Done
**Strategic Goal:** Grow the infrastructure that runs actual work — roles, a board, a scheduler, budget enforcement, isolated execution workspaces, quality gates, extensions, hooks, and the agent registry. Together these modules transform the subsystem layer (Phase 4) into a complete autonomous work engine. CLI commands attach to each subsystem as it lands. Orchestration and coordination (Phase 6) wires them together into a full autonomous office; Phase 5 delivers the parts.

> **Scope boundary:** Phase 5 owns the runtime core of each listed subsystem. Orchestration delegation (`/goal`, judge loops, briefings), trigger triage, mission mode, and deep research are Phase 6 responsibilities. TUI rendering is Phase 7. All cross-phase dependencies are represented as seam traits with no-op or stub defaults at Phase 5.
> **Test convention:** All tests reside in `crates/<crate>/tests/` directories — no inline `#[cfg(test)]` blocks in source files. Items required by tests are made `pub`. This applies to every crate touched in this phase.
> **Dependency policy:** New dependencies require explicit justification. Anticipated additions for Phase 5: `cron` (cron expression parsing — no std alternative), `notify` (filesystem watcher for config hot-reload seam — no std alternative), `git2` (git worktree lifecycle in execution workspace — no std alternative).

## Atomic Checklist

Track A — Security Foundation (no cross-Phase-5 deps; gates T-5C01, T-5D02)

- [x] [T-5A01] Tool Security — static skill scanner (8 attack categories + extended 16-category taxonomy) + runtime tool guard (10 threat categories, `ToolExecutionLevel`, hard-blocked patterns, path containment) + `ToolPolicy` (plan_mode, guide_only) + prompt injection hardening (`UNTRUSTED_CONTEXT_POLICY`, `untrusted_context_message`) + guardrail pipeline (vision-bridge, pii-masker, prompt-injection) + SARIF output + audit log entries

Track B — Work Model (parallel to A; sequential within track)

- [x] [T-5B01] Role Catalog — preset catalog (25 built-in roles across 5 categories), hire/fire mechanics (blueprint copy → state instance), custom role creation, adapter protocol (callable/status-reporting/fully-instrumented), `ContextDelivery` (fat_payload/thin_ping), config revisions with rollback, `AgentConfigRevision`, `role list|hire|create|show|fire` CLI commands
- [x] [T-5B02] Kanban Board — board storage (`<ws>/kanban/board.json`, `cards/`, `runs/`, `events/`, `comments/`, `archive/`), card CRUD with state machine (triage→todo→ready→running→blocked→done), `reason` required for blocked, KAN-7 transition history, auto-archival of done cards, `board list|show|add|move|block|done|archive` CLI commands (no cross-Phase-5 deps; depends on Phase 4 workspace)
- [x] [T-5B03] Scheduler — `Schedule` object (recurring/oneshot), `RecurrencePreset` (weekdays/weekends/daily/days/interval), raw cron expression support, three action kinds (heartbeat/routine/reminder), timezone-aware firing, per-workspace file storage (`<ws>/schedules/`), fire-time prompt injection scan (uses T-5A01 skill scanner), `CronPromptInjectionBlocked` exception, isolated cron session execution (session key, model preflight, run log, delivery dispatch, failure notification), `schedule list|add|edit|delete|run` CLI commands (depends on T-5B02)
- [x] [T-5B04] Budget Engine — `BudgetPolicy` hierarchy (workspace→role→card), cost event ingestion, hard-stop enforcement (budget-exhausted transitions card to blocked), monthly reset, `BudgetSnapshot` reporting, integration with kanban `done` transition, `budget show|set|reset` CLI commands (depends on T-5B01 + T-5B02)
- [x] [T-5B05] Execution Workspace — isolated execution environments per card, git worktree lifecycle (slug naming: `<ws>-<card_id>-<ts>`, boot sequence, `isPristine` check, reset+prune, workspace events), no-remote-git contract, finalize write-back gate (guard prevents writing to main tree until quality gates pass), `exec list|create|finalize|discard` CLI commands (depends on T-5B02 + Phase 4 security seams)

Track C — Extensions (depends on T-5A01 + T-5B01)

- [x] [T-5C01] Extension Registry — `extension.json` manifest validation (kind: skill/mcp-server/plugin; permissions: fs/network/secrets), lifecycle states (discovered→permitted→active→inactive), component auto-discovery (`commands/`, `agents/`, `skills/`, `hooks/` default dirs + manifest overrides), `${PLUGIN_ROOT}` portable path, SKILL.md trigger format, agent definition frontmatter (name/description/model/color/tools), command definition frontmatter (dynamic tokens: $ARGUMENTS/$1/@file/!bash!/`${PLUGIN_ROOT}`), MCP transport variants (stdio/sse/http/ws, tool naming `mcp__<plugin>_<server>__<tool>`), skill generation pipeline (curator distills patterns → candidate skill → `<state>/extensions/skills/`), `ext list|add|remove|scan|activate|deactivate` CLI commands (depends on T-5A01 + T-5B01)
- [x] [T-5C02] Plugin Hooks — `HookEvent` taxonomy (9 events: PreToolUse/PostToolUse/Stop/SubagentStop/SessionStart/SessionEnd/UserPromptSubmit/PreCompact/Notification), `hooks.json` format with matcher syntax, parallel hook execution model, `ActorMatcher` filter, actor.preStop/postStop ReAct loops, aggregated decision, file hooks auto-discovery, sequential external plugin loading, rule evaluation engine (block/warn, AND conditions, 6 operators), hook security model (input validation, path safety, quoting, timeouts), `HookEvent` observability events (depends on T-5C01 + Phase 4 agent-session seams)
- [x] [T-5C03] Learning Loop — post-turn background review fork (spawns a sub-agent seam after turn completion), skill package format (`SKILL.md` + optional assets), curator role (pattern extraction from session history → candidate skill diff → approval gate), `LearningConfig` (enabled flag, min_turns_before_review, min_confidence_to_propose), `learn list|approve|reject` CLI commands (depends on T-5C01 + Phase 4 memory-store + agent-session seams)

Track D — Quality & Agent Management (parallel to C; depends on T-5A01 + T-5B01 + T-5B02)

- [x] [T-5D01] Quality Pipeline — per-language toolchain map (Rust/TS/Python/Go), `GateKind` enum (tests/lint/type_format/benchmarks/security/review/refactor), `GateResult` (pass/fail/warn/skipped), gate runner (lang detection via project markers, tool invocation, result recording), `QLY-1` board integration (done transition requires green gate report for the card), `QLY-7` blocking (failed required gate returns non-zero + written to card history), `QLY-3` conditional gates (benchmarks on perf-tagged changes, security on sec-sensitive), SARIF output for security gate results, `check run|show|history` CLI commands (depends on T-5B02)
- [x] [T-5D02] Agent Registry — `AgentDefinition` schema (name/mode/native/hidden/permission/model/model_ref/steps/tool_allowlist), 7 built-in agents (work/code/plan/edit/search/test/refactor), `Ruleset` permission merge (last-key-wins), `model_ref` resolution via model-router (never-throw; unknown group → run-default), user config override layer (disable/rename/reprogram without source change), registry rebuild on workspace config change, fork-agent checkpoint-writer contract (seam from Phase 4 checkpoint), generate-from-description API (seam returning stub at Phase 5; real LLM call wires in Phase 6), `agent list|show|create|disable|enable` CLI commands (depends on T-5B01 + T-5A01 + Phase 4 model-router + session-checkpoint seams)

Track T — Validation

- [x] [T-5T01] Phase 5 integration tests + CLI command additions — integration tests for each subsystem in `crates/core/tests/`; cross-subsystem round-trips (hire role → create card → schedule run → budget enforcement → finalize workspace → quality gate → archive); CLI smoke tests for all Phase 5 command groups; `cargo test --workspace && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check` fully green

## Detailed Tracking

### [T-5A01] Tool Security

- **Spec:** l2-tool-security.md (full)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus tool_security` passes: skill scanner assigns CRITICAL finding to a `command_injection` pattern; skill with no HIGH/CRITICAL findings is marked `is_safe=true`; risk score computed correctly (CRITICAL=+50, HIGH=+25, 1.3× executable multiplier); hard-blocked pattern `rm -rf /` is denied with no escalation path; tool guard intercepts a path traversal `../../etc/passwd` and returns CRITICAL; `ToolExecutionLevel::smart` auto-allows LOW findings and escalates MEDIUM+; `SuspendedPermission` payload constructed correctly for an approval escalation; `ToolPolicy` in `plan_mode` blocks write tools not on the allowlist; `pii-masker` guardrail redacts an email address in request content; `untrusted_context_message` wraps content with `<<<UNTRUSTED_SOURCE_DATA>>>` delimiter; `_escape_guard_markers` neutralizes a literal delimiter inside the content; SARIF output produced for a scan with one HIGH finding.
- **Notes:** New module `crates/core/src/tool_security.rs` (or `crates/core/src/tool_security/` if multi-file). Skill scanner: `SkillScanner::scan(skill_dir: &Path) -> ScanResult`; 8 attack categories as `SignatureRule` structs with compiled regex patterns; `ScanFinding` with confidence; `is_safe = no CRITICAL or HIGH findings`; risk score formula: severity weights summed, × 1.3 if `has_executable_scripts`, capped at 100; risk bands (0–20 LOW/SAFE, 21–50 MEDIUM/CAUTION, 51–80 HIGH/DO_NOT_INSTALL, 81–100 CRITICAL/hard-block). Runtime tool guard: `ToolGuard::evaluate(tool_name, params) -> ToolGuardResult`; 10 threat categories with guardian assignments; hard-blocked patterns list (unconditional deny); path containment check (symlink-followed, boundary = agent cwd); `SuspendedPermission` construction on MEDIUM+ escalation. `ToolPolicy`: `disabled_tools`, `hidden_tools`, `block_all_tool_calls`, `disable_mcp`; `plan_mode` allowlist-as-denylist inversion; `guide_only` regex detection. Guardrail pipeline: `BaseGuardrail` trait, `GuardrailContext`, `GuardrailResult`; fail-open (guardrail error → skip, log WARN); `pii-masker` (pre+post call, configurable patterns in `<state>/guardrails/pii-patterns.json`); `prompt-injection` guardrail (modes: warn/block/log, default=warn); `vision-bridge` stub (seam returning no-op at Phase 5). Prompt injection hardening: `UNTRUSTED_CONTEXT_POLICY` system preamble (protected, priority 1); `untrusted_context_message(label, content)` builder; `_escape_guard_markers` delimiter neutralization. Audit log: `<ws>/audit.log` append with `{ timestamp, layer, tool_name?, finding_id, category, severity, outcome }`. SARIF output: `SarifLog` 2.1.0 mapping. Extended vulnerability taxonomy (§4.9): 16 categories, automatic-CRITICAL rules (AST8/TT3/TT5/TP2), MCP checks (LP1–LP4, TP1–TP4). No new external deps. Tests in `crates/core/tests/tool_security.rs`.

### [T-5B01] Role Catalog

- **Spec:** l2-role-catalog.md (full), l1-roles.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus role_catalog` passes: preset catalog lists all 25 built-in roles; hire copies blueprint to state directory with correct `config.json` (model/reportsTo/hired_from); fire archives role memory and removes from active roster; re-hire after fire restores the instance; custom role creation writes new entry with `hired_from: custom`; derived custom role writes `hired_from: <preset>`; presets are not mutated (ROL-7) — writing to preset dir returns an error; config revision recorded on config change with monotonically increasing `revisionNumber`; rollback applies `beforeConfig` as a new revision (source=rollback); `role list|hire|create|show|fire` CLI commands exit 0 with expected output.
- **Notes:** New module `crates/core/src/roles/` with sub-files: `mod.rs`, `catalog.rs`, `instance.rs`, `adapter.rs`, `revision.rs`. `RoleDefinition` struct: embedded from `<program>/employees/<role>/config.json` templates. `HiredInstance` struct: `{ id, name, hired_from, reports_to, config: RoleConfig, created_at }`. Hire: `fs::copy` blueprint → `<state>/employees/<id>/` + create `memory/`, `skills/`, `skins/` subdirs + write initial `config.json`. Fire: move `memory/` to `<state>/employees/<id>/archive-<ts>/` + remove from active roster (SQLite `hired_agents` table). `AgentConfigRevision`: stored in `<state>/employees/<id>/config-revisions/<revision>.json`. Adapter protocol: `AdapterLevel { Callable | StatusReporting | FullyInstrumented }`, `ContextDelivery { FatPayload | ThinPing }`. CLI additions: `Command::Role { sub: RoleCommand }` in `crates/cli/src/cli.rs`. No new external deps. Tests in `crates/core/tests/role_catalog.rs`.

### [T-5B02] Kanban Board

- **Spec:** l2-kanban-board.md (full), l1-kanban-model.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus kanban_board` passes: board initialised under `<ws>/kanban/board.json` with correct state set; card created in `triage` state; transition triage→todo→ready→running records a history entry (KAN-7) for each move; blocking a card without a reason returns an error (KAN-5); transition to done appends to events log; auto-archival job moves `done` cards to `archive/` without deletion (KAN-4); archived card is still readable; `board list|show|add|move|block|done` CLI commands exit 0; `board move` returns an error for an invalid state transition.
- **Notes:** New module `crates/core/src/kanban/` with sub-files: `mod.rs`, `board.rs`, `card.rs`, `archival.rs`, `events.rs`. `CardState` enum: `Triage | Todo | Ready | Running | Blocked | Done`. `Card` struct fields: id, task_ref, state, reason (Option — required when Blocked), assignee, priority, skills, workspace_kind, workspace_path, max_retries, history, created_at, updated_at. Storage: one JSON file per card at `<ws>/kanban/cards/<card_id>.json`; `board.json` holds board meta + card index. Transition guard: `fn transition(card, to, actor, reason) -> Result<Card>` — validates state machine edges; appends to `history[]` and `events/<card_id>.jsonl`. Archival job: `fn archive_done_cards(ws_path) -> Result<usize>` — moves `done` cards meeting age threshold to `archive/`; threshold configurable in workspace config (default: 0 = immediate). CLI: `Command::Board { sub: BoardCommand }`. Reuses `rusqlite` for card index queries when needed; primary storage is JSON files. No new external deps. Tests in `crates/core/tests/kanban_board.rs`.

### [T-5B03] Scheduler

- **Spec:** l2-scheduler.md (full), l1-scheduler-model.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus scheduler` passes: schedule created with `recurring` kind and `weekdays` preset fires on the next weekday occurrence; one-shot schedule sets `delete_after_fire: true` and is removed after first fire; `heartbeat` action calls wake entry point without touching the board; fire-time prompt injection scan blocks a schedule payload containing injection content (`CronPromptInjectionBlocked`); timezone-aware firing computes correct next-fire for a non-UTC timezone; `schedule list|add|edit|delete|run` CLI commands exit 0; cron expression `0 9 * * 1-5` next-fire computes correctly; isolated cron session execution records a run log entry.
- **Notes:** New module `crates/core/src/scheduler/` with sub-files: `mod.rs`, `schedule.rs`, `fire.rs`, `cron_session.rs`. `Schedule` struct: id, name, kind (Recurring/Oneshot), action (Heartbeat/Routine/Reminder), recurrence (`RecurrencePreset` + times), cron (Option), at (Option — oneshot), timezone, enabled, delete_after_fire, created_at, next_fire_at. Storage: `<ws>/schedules/<schedule_id>.json` per schedule; scheduler service reloads all on startup. `RecurrencePreset` enum: Weekdays/Weekends/Daily/Days/Interval. Cron expression parsing: `cron` crate (justified: no std alternative for cron expression parsing). Fire-time injection scan: calls `ToolScanner::scan_content(assembled_prompt)` using T-5A01 skill scanner patterns; raises `CronPromptInjectionBlocked` when CRITICAL/HIGH findings detected. Isolated cron session: `CronSession { session_key: Uuid, model_preflight: ModelPreflightResult, run_log: Vec<RunLogEntry> }`; delivery dispatch seam (real ACP daemon wires in Phase 7); failure notification seam (no-op at Phase 5). No new external deps except `cron`. Tests in `crates/core/tests/scheduler.rs`.

### [T-5B04] Budget Engine

- **Spec:** l2-budget-engine.md (full)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus budget_engine` passes: budget policy hierarchy resolves workspace → role → card (more specific wins); cost event ingested correctly increments the running total; hard-stop fires when running total >= policy limit and transitions the card to `blocked` via kanban seam; monthly reset zeros the counter on the first fire after month rollover; `BudgetSnapshot` contains correct period, spent, and remaining values; `budget show` CLI exits 0 with budget data; `budget set` updates the workspace policy; over-budget cost event returns `Err(BudgetExhausted)`.
- **Notes:** New module `crates/core/src/budget.rs`. `BudgetPolicy` struct: `{ workspace_id: Option<String>, role_id: Option<String>, card_id: Option<String>, limit_usd: f64, period: BudgetPeriod }`. `BudgetPeriod` enum: `Monthly | Weekly | Daily | Unlimited`. `CostEvent` struct: `{ session_id, card_id?, role_id?, amount_usd: f64, tokens_used: u32, model: String, timestamp: u64 }`. Policy resolution: most-specific wins (card > role > workspace); stored in SQLite `budget_policies` table and `cost_events` table. Hard-stop enforcement: `fn ingest_cost(conn, event) -> Result<BudgetStatus>` — after insert, query running total against active policy; if over limit, call kanban seam `fn block_card(card_id, reason) -> Result<()>` (seam returns Ok(()) at Phase 5 if kanban not yet wired; real call uses T-5B02). Monthly reset: SQL `WHERE period_start < ?` query on month boundary; run at scheduler fire time (Scheduler seam → fires reset at midnight on first day of month). CLI: `Command::Budget { sub: BudgetCommand }`. No new external deps. Tests in `crates/core/tests/budget_engine.rs`.

### [T-5B05] Execution Workspace

- **Spec:** l2-execution-workspace.md (full)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus exec_workspace` passes: workspace created with slug `<ws>-<card_id>-<ts>` under `<state>/exec-workspaces/`; boot sequence initialises git worktree at the slug path; `isPristine` returns true for a newly created worktree and false after a file change; reset+prune cleans untracked files and restores pristine state; no-remote-git contract enforced — an attempt to `git push` inside the worktree returns an error; finalize write-back gate refuses to merge until all required quality gates pass (gate check is a seam returning pass at Phase 5); `exec list|create|finalize|discard` CLI commands exit 0; discard removes the worktree directory.
- **Notes:** New module `crates/core/src/exec_workspace.rs`. `ExecWorkspace` struct: `{ id, card_id, ws_id, slug: String, path: PathBuf, state: ExecState, created_at, finalized_at }`. `ExecState` enum: `Active | Finalizing | Finalized | Discarded`. Slug format: `{ws_id}-{card_id}-{unix_ts}` (kebab, lowercase). Git worktree lifecycle: `git2::Repository::worktree(slug, &path, None)` for creation; `isPristine()` = `repo.statuses(None)?.is_empty()`; reset+prune: `repo.reset(&head_commit, git2::ResetType::Hard, None)` + `repo.cleanup_state()`; worktrees stored in SQLite `exec_workspaces` table. No-remote-git contract: no remote configured on the worktree repo; `git push` invocation returns `Err(ExecError::RemoteGitForbidden)`. Finalize write-back gate: `fn finalize(ws_id) -> Result<()>` checks quality gate seam `fn gate_result(card_id) -> GateStatus` (returns `GateStatus::Pass` at Phase 5; real check uses T-5D01 in Phase 6 wiring). Workspace events: `WorkspaceEvent { Created | Booted | PristineCheck | Reset | Finalized | Discarded }` emitted to the event bus seam. Justified dep: `git2` (git worktree API — no std alternative). Tests in `crates/core/tests/exec_workspace.rs`.

### [T-5C01] Extension Registry

- **Spec:** l2-extension-registry.md (full), l1-extensions.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus extension_registry` passes: manifest with valid kind/permissions validates successfully; manifest missing required field returns `ValidationError`; extension lifecycle transitions discovered→permitted→active→inactive in correct order; `active` state required before tool invocation — calling a `discovered` extension returns an error; preset extensions loaded from `<program>/extensions/` as read-only; custom extensions created under `<state>/extensions/`; component auto-discovery finds a `skills/` dir and registers its contents; MCP transport stub connects via stdio (seam); skill generation: curator seam deposits a candidate skill to `<state>/extensions/skills/`; `ext list|add|remove|scan|activate|deactivate` CLI commands exit 0; `ext scan` returns scan result from T-5A01 skill scanner.
- **Notes:** New module `crates/core/src/extensions/` with sub-files: `mod.rs`, `manifest.rs`, `registry.rs`, `discovery.rs`, `skill_gen.rs`, `mcp.rs`. `ExtensionManifest` struct: id, kind (`Skill | McpServer | Plugin`), name, version, source (`Preset | Custom | Generated`), capabilities, permissions (`fs/network/secrets`), entry, connect. Manifest validation: JSON schema check against `extension.json` format; unknown permission scopes → error; overly broad permissions (wildcard) → warning surfaced at grant gate. `ExtensionState` enum: `Discovered | Permitted | Active | Inactive`. Registry backed by SQLite `extensions` table. Auto-discovery: scan `commands/`, `agents/`, `skills/`, `hooks/` in default dirs + manifest-declared dirs; parse frontmatter from SKILL.md/agent definition. `${PLUGIN_ROOT}` resolution: expanded to the extension's root dir at activation time. MCP transport seam: `McpTransport` trait with `Stdio` and `Http` variants; `Stdio` spawns a process (seam stub at Phase 5); real process management wires in Phase 6. Skill generation: `CuratorSeam` trait returning `CandidateSkill { content: String, trigger: String }`; no-op stub at Phase 5. Integrates with T-5A01 for scan-on-activation. CLI: `Command::Ext { sub: ExtCommand }`. No new external deps. Tests in `crates/core/tests/extension_registry.rs`.

### [T-5C02] Plugin Hooks

- **Spec:** l2-plugin-hooks.md (full)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus plugin_hooks` passes: `HookEvent::PreToolUse` fires before tool execution and can block it; `HookEvent::PostToolUse` fires after and cannot block; `HookEvent::Stop` aggregated decision combines deny from one hook with allow from another (deny wins); `hooks.json` parsed correctly with matcher syntax (tool name pattern, event type); hooks run in parallel for the same event; `ActorMatcher` filters hooks for a specific actor ID; rule evaluation engine evaluates AND conditions with 6 operators; input validation rejects a hook command containing shell metacharacters; hook timeout (configurable, default 10s) cancels a slow hook with a WARN log; file hooks discovered from `hooks/` dir in workspace; `HookEvent` observability events appended to the audit log.
- **Notes:** New module `crates/core/src/hooks.rs`. `HookEvent` enum: `PreToolUse | PostToolUse | Stop | SubagentStop | SessionStart | SessionEnd | UserPromptSubmit | PreCompact | Notification`. `HookEntry` struct: `{ id, event: HookEvent, matcher: ActorMatcher, command: String, timeout_ms: u64 }`. `ActorMatcher` struct: `{ actor_id: Option<String>, tool_name: Option<String>, event_filter: HookEvent }`. `HookResult` enum: `Allow | Block(String) | Warn(String)`. Parallel execution: hooks for same event run concurrently (via `std::thread::spawn` stubs at Phase 5; real async wires in Phase 6); aggregated decision = deny if any hook returns Block. Rule evaluation engine: `RuleCondition { field: String, op: RuleOp, value: String }` with `RuleOp { Eq | Ne | Contains | StartsWith | EndsWith | Matches }` (AND semantics); evaluated per hook entry. Hook security: path safety check (command must not escape workspace root); quoting enforcement (args passed as `Vec<String>`, not shell-concatenated); input validation (block metacharacters `;|>&$\``). `hooks.json` format: `{ hooks: [{ event, matcher, command, timeout_ms? }] }`; file auto-discovered at `<ws>/hooks.json` and `<ws>/hooks/*.json`. CLI: `Command::Hook { sub: HookCommand }` (list/add/remove). No new external deps. Tests in `crates/core/tests/plugin_hooks.rs`.

### [T-5C03] Learning Loop

- **Spec:** l2-learning-loop.md (full)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus learning_loop` passes: post-turn review fork spawns a sub-agent seam after turn completion when `min_turns_before_review` is met; sub-agent seam is called with session history slice; curator seam returns a `CandidateSkill` when pattern extraction succeeds; candidate skill deposited to `<state>/extensions/skills/<id>/SKILL.md`; approval gate creates a pending entry visible to `learn list`; approved skill becomes active in the extension registry; rejected skill is removed; `LearningConfig` with `enabled: false` disables all review forks; `learn list|approve|reject` CLI commands exit 0.
- **Notes:** New module `crates/core/src/learning.rs`. `LearningConfig` struct: `{ enabled: bool, min_turns_before_review: u32, min_confidence_to_propose: f64 }`. `PostTurnReviewFork`: seam trait `fn fork_review(session_history: &[SessionEntry]) -> Result<Option<CandidateSkill>>`; no-op stub at Phase 5 (real LLM sub-agent wires in Phase 6). `CandidateSkill` struct: `{ id: String, trigger: String, content: String, confidence: f64, source_session_id: String }`. Approval gate: SQLite table `skill_candidates { id, content, trigger, confidence, status: pending|approved|rejected, created_at }`. `approve(id)` calls extension registry `fn activate(manifest)` seam. CLI: `Command::Learn { sub: LearnCommand }`. No new external deps. Tests in `crates/core/tests/learning_loop.rs`.

### [T-5D01] Quality Pipeline

- **Spec:** l2-quality-pipeline.md (full), l1-quality-standards.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus quality_pipeline` passes: language auto-detected as Rust from `Cargo.toml`, TypeScript from `package.json`, Python from `pyproject.toml`, Go from `go.mod`; gate runner invokes the correct tool for the detected language; gate result (pass/fail/warn/skipped) written to SQLite; `done` transition blocked by kanban seam when a required gate is not passing (QLY-7); conditional gate (benchmark) skipped when card is not tagged `perf`; `GateResult::Fail` written to card history; `check run|show|history` CLI commands exit 0; SARIF output produced for a security gate result with a finding.
- **Notes:** New module `crates/core/src/quality.rs`. `GateKind` enum: `Tests | Lint | TypeFormat | Benchmarks | Security | Review | Refactor`. `GateResult` struct: `{ gate: GateKind, status: GateStatus, output: String, duration_ms: u64, sarif: Option<SarifLog> }`. `GateStatus` enum: `Pass | Fail | Warn | Skipped`. Language detection: `fn detect_language(project_root: &Path) -> Language` — checks project markers in order (Cargo.toml → Rust, package.json → TypeScript, pyproject.toml/setup.py → Python, go.mod → Go, else Unknown). Tool invocation: `fn run_gate(gate, project_root) -> GateResult` — spawns the appropriate tool as a child process; `Rust::Tests` → `cargo test`, `Rust::Lint` → `cargo clippy -- -D warnings`, `Rust::TypeFormat` → `cargo fmt --check`, etc. Gate results stored in SQLite `gate_results` table (keyed by card_id + gate_kind + run_id). Kanban integration: `DoneTransitionGuard` seam trait; real wiring completes when T-5B02 + T-5D01 are both active; guard seam returns `Ok` at Phase 5 if kanban not yet wired. SARIF output: reuses T-5A01's `SarifLog` type for security gate findings. CLI: `Command::Check { sub: CheckCommand }`. No new external deps. Tests in `crates/core/tests/quality_pipeline.rs`.

### [T-5D02] Agent Registry

- **Spec:** l2-agent-registry.md (full), l1-roles.md and l1-orchestration.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus agent_registry` passes: 7 built-in agents loaded on registry init (work/code/plan/edit/search/test/refactor); `resolve(name) -> AgentDefinition` returns a definition for a built-in name; `resolve` for an unknown name with no user config returns `Err(NotFound)`; user config entry disables a built-in (config `disable: true`) and resolves to `Err(Disabled)`; user config entry overrides `model_ref` of a built-in; `model_ref` resolution falls back to run-default for an unknown group; `Ruleset` permission merge applies last-key-wins for same key pattern; fork-agent checkpoint-writer contract: registry returns a `CheckpointWriter` seam (uses Phase 4 `FileCheckpointWriter`); generate-from-description stub returns a placeholder `AgentDefinition`; registry rebuilds on workspace config change signal; `agent list|show|create|disable|enable` CLI commands exit 0.
- **Notes:** New module `crates/core/src/agent_registry.rs`. `AgentDefinition` struct: name, description, mode (`Primary | SubAgent | All`), native, hidden, temperature, top_p, color, permission (`Ruleset`), model (`Option<ModelSpec>`), model_ref (`Option<String>`), variant, prompt, steps, tool_allowlist, options. `AgentMode` enum: `Primary | SubAgent | All`. 7 built-in agents embedded as `const` definitions. `Ruleset` permission merge: `fn merge(base: &Ruleset, override: &Ruleset) -> Ruleset` — last-key-wins on same key patterns; union on distinct patterns. `model_ref` resolution: calls `ModelRouter::resolve_group(ref_str)` seam; unknown group → `ModelSpec::run_default()`. Fork-agent checkpoint-writer: `fn checkpoint_writer_for(agent_id) -> Box<dyn CheckpointWriter>` — returns `FileCheckpointWriter` from Phase 4; used by orchestration layer (Phase 6). Generate-from-description: `fn generate(description: &str) -> Result<AgentDefinition>` — seam trait returning a stub `AgentDefinition` with placeholder values at Phase 5; real LLM call wires in Phase 6. Registry rebuild: `fn rebuild(&mut self, ws_config: &WorkspaceConfig)` — called when extension install or manual config edit is detected. CLI: `Command::Registry { sub: RegistryCommand }`. No new external deps. Tests in `crates/core/tests/agent_registry.rs`.

### [T-5T01] Phase 5 integration tests + CLI command additions

- **Spec:** l2-cli.md §4 (command additions for Phase 5 subsystems), all Phase 5 specs
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-cli` passes for all Phase 5 CLI commands; cross-subsystem integration round-trip: `workspace create → role hire → kanban card add → schedule add → exec workspace create → quality gate run → ext add → agent registry resolve` all exit 0 and produce consistent state; `cargo test --workspace` fully green; `cargo clippy --all-targets -- -D warnings` zero lints; `cargo fmt --all -- --check` passes.
- **Notes:** CLI command additions in `crates/cli/src/commands.rs`: `Command::Role`, `Command::Board`, `Command::Schedule`, `Command::Budget`, `Command::Exec`, `Command::Check`, `Command::Ext`, `Command::Learn`, `Command::Registry` — each delegates to the corresponding `crates/core` module function with zero domain logic in CLI. Integration smoke tests extend `crates/cli/tests/cli_smoke.rs` with one success case per Phase 5 command group. `crates/core/tests/integration.rs` extended with Phase 5 cross-subsystem round-trips. All tests in `tests/` directories. Final gate: `cargo test --workspace && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`.

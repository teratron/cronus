---
phase: 6
name: "Orchestration & Autonomy"
status: Done
subsystem: "crates/core (new modules)"
requires:
  - "tool security (crates/core/src/tool_security.rs) — Phase 5 T-5A01"
  - "kanban board (crates/core/src/kanban/) — Phase 5 T-5B02"
  - "scheduler (crates/core/src/scheduler.rs) — Phase 5 T-5B03"
  - "agent registry (crates/core/src/agent_registry.rs) — Phase 5 T-5D02"
  - "agent session loop (crates/core/src/session/) — Phase 4 T-4C01"
  - "agent autonomy (crates/core/src/autonomy.rs) — Phase 4 T-4C05"
  - "context management (crates/core/src/context_mgmt.rs) — Phase 4 T-4C04"
  - "model router (crates/core/src/router/) — Phase 4 T-4B01"
  - "CLI command dispatch (crates/cli/src/commands.rs) — Phase 3 T-3A01"
provides:
  - "Orchestration engine (crates/core/src/orchestration.rs)"
  - "Trigger Triage module (crates/core/src/trigger_triage.rs)"
  - "Mission Mode module (crates/core/src/mission.rs)"
  - "Deep Research module (crates/core/src/research.rs)"
  - "Phase 6 CLI commands (goal/trigger/mission/research subgroups)"
key_files:
  created:
    - "crates/core/src/orchestration.rs"
    - "crates/core/src/trigger_triage.rs"
    - "crates/core/src/mission.rs"
    - "crates/core/src/research.rs"
    - "crates/core/tests/orchestration.rs"
    - "crates/core/tests/trigger_triage.rs"
    - "crates/core/tests/mission_mode.rs"
    - "crates/core/tests/deep_research.rs"
  modified:
    - "crates/core/src/lib.rs"
    - "crates/cli/src/cli.rs"
    - "crates/cli/src/commands.rs"
    - "crates/cli/tests/cli_smoke.rs"
patterns_established:
  - "Judge-budget loop: orchestrator delegates then calls independent judge; budget is the circuit-breaker"
  - "Seam-first: classifier, search backend, and LLM judge are pluggable via extension registry"
  - "Untrusted-context wrapping: all external content wrapped before model injection"
duration_minutes: ~
---

# Phase 6 Tasks — Orchestration & Autonomy

**Phase:** 6
**Status:** In Progress
**Strategic Goal:** Wire the Phase 5 subsystems into a coordinating autonomous office. The orchestrator delegates via board cards, an independent judge evaluates goal completion, a budget circuit-breaker stops runaway loops, a trigger-triage pipeline classifies inbound signals without spawning full sessions for low-value events, Mission Mode gives the agent a safe two-phase execution path for complex tasks, and Deep Research gives it an iterative multi-round search capability. CLI commands attach to each new subsystem as it lands.

> **Scope boundary:** Phase 6 owns the coordination layer over the Phase 5 runtime. TUI rendering is Phase 7. Desktop application is Phase 8. Operational hardening (doctor, backup, self-improvement, telemetry) is Phase 9. At Phase 6, the local triage classifier and the search backend are stubs returning low-confidence / empty results respectively — real integrations wire in when the extension registry provides a `TriageClassifier` or `SearchProvider` plugin. The LLM judge for `/goal` is a seam calling the model router with a structured prompt; the research report is marked `_protected` but compaction is a Phase 4 seam already present.
> **Test convention:** All tests reside in `crates/<crate>/tests/` directories — no inline `#[cfg(test)]` blocks in source files. Items required by tests are made `pub`.
> **Dependency policy:** No new external crates required for Phase 6 — all coordination logic uses `std`, `rusqlite` (already present), and the Phase 4/5 core modules via seams.

## Atomic Checklist

Track A — Orchestration Core (gates B, C; no Phase-6 internal deps)

- [x] [T-6A01] Orchestration engine — delegation via board card assignment (`assignee` field), `/goal` run with judge+budget circuit-breaker, `AgentTier` enum (chat/reasoning/worker) with static validation (chat rejects shell/code_execution toolsets; worker requires execution/isolated scope), `MAX_SPAWN_DEPTH = 3` constant + `SpawnDepthError` on overflow, `rank_tools(task_description, tools, top_n=20)` CPU-only action ranking (verb_score + token_score unigram overlap), `PermissionRule` + `evaluate()` last-match-wins ruleset evaluation, `AgentToolResult.terminate` all-or-nothing batch semantics, `AgentLoopConfig` hooks struct (beforeToolCall/afterToolCall/shouldStopAfterTurn/prepareNextTurn/transformContext/getSteeringMessages/getFollowUpMessages), per-file mutation queue (`withFileMutationQueue`), three-tier durability model (Delegation/Scheduled/Queue), `GoalRun` state (budget/iteration/status), judge evaluation seam (structured model router call returning met/not-met), `cronus goal <text>`, `cronus goal stop`, `cronus status`, `cronus change graph|next|split|status` CLI commands

Track B — Trigger Triage (depends on T-6A01)

- [x] [T-6B01] Trigger Triage — `TriggerEnvelope` (id/source_type/payload/received_at/workspace_id/metadata), `SourceType` enum (chat_message/webhook/cron/event/sub_agent_spawn), `TriggerPayload` (excerpt ≤200 tokens, event_kind, urgency_hint), `TriageDecision` enum (Drop/Notify/SpawnReactor/SpawnOrchestrator), `triage()` pipeline (rate_limit_check → dedup_check → payload_guard → classify → record_dedup), local classifier stub (returns confidence=0.0 → forces rule fallback; pluggable via extension registry `TriageClassifier` interface), cloud fallback seam (model router call; skipped if local confidence ≥ CONFIDENCE_THRESHOLD=0.8), rule-based fallback table (deterministic: cron/webhook/chat→SpawnReactor; event+error→SpawnOrchestrator; others→Notify), `DeduplicateCache` (`HashMap<(SourceType, ContentHash), Instant>`, per-source window defaults: event=60s/webhook=30s/cron=off/chat=off), rate limiting per SourceType (webhook=100/min, event=500/min, others=unlimited), `cronus trigger list|history|replay` CLI commands

Track C — Autonomous Execution (parallel; both depend on T-6A01)

- [x] [T-6C01] Mission Mode — `Mission` struct (id/task/phase/status/created_at), `MissionPhase` enum (Exploration/Execution), `PrdDocument` (project/description/userStories), `UserStory` (id/title/story/passes), `LoopConfig` (session_id/branch_name/git_installed/is_git_repo/default_branch/current_branch/repo_root), `MissionMode` enum (Lite/Full/Ultra/Off) with three-source resolution (env `CRONUS_MISSION_MODE` > `config.json` missionMode > compiled default `Full`), flag file `<ws>/.mission-mode`, execution loop (`while !all_stories_pass && iteration < max_iterations`), discuss-phase CONTEXT.md writing with D-NN decision IDs, structured clarification protocol (`.planning/clarifications.md`, up to 7 questions, source: user/prd/inferred), session intent classification (`WorkType` enum + `PhaseIntent` enum written to SUMMARY.md frontmatter), `SpecPersistenceModel` enum (FlowForward/FlowBack/LivingSpec, default=FlowForward), proposal artifact (`proposal.md` lifecycle: draft→ready→accepted/rejected), state files at `<ws>/missions/<mission-id>/` (loop_config.json/prd.json/progress.txt/task.md), `cronus mission start|confirm|status|list|resume|abort` CLI commands

- [x] [T-6C02] Deep Research — `ResearchJob` (id/question/status/rounds_completed/max_rounds/created_at), `ResearchPlan` (sub_questions/key_topics/success_criteria), `ResearchQuery` (round/queries: Vec<String>), `FetchedPage` (url/title/content/retrieved_at), content filter (min_chars=200, dedup by URL, error-page detection), untrusted-content wrapping (delegates to `tool_security::untrusted_context_message`), `ResearchRound` (Think→Plan→Search→Extract per round), `ResearchReport` (question/sub_questions_answered/report_body/citations/rounds_completed/success_criteria_met/partial_reason), date-grounding preamble injection on every query-generation and evaluation prompt, max_rounds circuit breaker (default=5, configurable), report marked `_protected` in session, state storage at `<ws>/research/<job-id>/`, search backend stub (returns empty results; pluggable via extension registry `SearchProvider` interface), `cronus research start|status|report|list|cancel` CLI commands

Track T — Validation

- [x] [T-6T01] Phase 6 integration tests + CLI command additions — unit + integration tests for each Phase 6 module in `crates/core/tests/` (orchestration.rs, trigger_triage.rs, mission_mode.rs, deep_research.rs); cross-module round-trip (triage envelope → SpawnOrchestrator decision → goal run → judge evaluation seam → budget circuit-breaker); mission mode round-trip (start mission → write PRD → confirm → execution loop iteration → partial completion on max_iterations=1); deep research round-trip (start job → plan generated → round executed with stub backend → partial report emitted); CLI smoke tests for all Phase 6 command groups appended to `crates/cli/tests/cli_smoke.rs`; `cargo test --workspace && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check` fully green

## Detailed Tracking

### [T-6A01] Orchestration Engine

- **Spec:** l2-orchestration.md (full), l1-orchestration.md (invariants)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus orchestration` passes: `rank_tools` with a catalog >50 entries returns exactly `top_n=20` tools ranked by verb+token score; `rank_tools` with ≤50 entries returns the full catalog; `evaluate()` returns the last matching rule's action (not the first); an empty ruleset returns `"ask"` (default); `AgentTier::Chat` definition with `shell` in toolsets is rejected with a hard validation error; `AgentTier::Worker` definition with `session` scope is rejected; `GoalRun` initialised at iteration=0 pauses when iterations exhausted; spawn depth counter increments on delegation and returns `SpawnDepthError` at depth 4 (parent at 3); `withFileMutationQueue` serializes two concurrent writes to the same path; `AgentToolResult` batch with one non-terminate tool does NOT trigger early stop; `goal` CLI exits 0; `goal stop` CLI exits 0; `change next` CLI exits 0.
- **Notes:** New module `crates/core/src/orchestration.rs`. `AgentTier` enum: `Chat | Reasoning | Worker`. Static validation: `fn validate_tier(def: &AgentDefinition) -> Result<(), AgentDefinitionError>`. `MAX_SPAWN_DEPTH: u32 = 3` constant. `SpawnDepthError { depth: u32 }`. `rank_tools(task: &str, tools: &[ToolDefinition], top_n: usize) -> Vec<ToolDefinition>`: verb_score = leading verb of `task` found in `tool.name` as token (0 or 1); token_score = unigram intersection count between task words and tool.description words; sort descending by sum; take min(top_n, len). `PermissionRule { permission: String, pattern: String, action: PermissionAction }` where `PermissionAction: Allow | Deny | Ask`. `evaluate(permission, path, rulesets) -> PermissionAction`: merge rulesets in order, `find_last` matching rule, default `Ask`. `GoalRun { id, goal, budget_usd: f64, max_iterations: u32, iteration: u32, status: GoalRunStatus }` where `GoalRunStatus: Running | Paused | Complete | Failed`. Judge seam: `fn evaluate_goal(run: &GoalRun, summary: &str) -> JudgeVerdict` returns `JudgeVerdict::Met | JudgeVerdict::NotMet { reason }`. `AgentToolResult { content: Vec<String>, terminate: bool }`. Batch rule: `fn should_terminate_batch(results: &[AgentToolResult]) -> bool = results.iter().all(|r| r.terminate)`. `AgentLoopConfig` struct with function-pointer or `Option<Box<dyn Fn(...)>>` hooks. `withFileMutationQueue` seam: `Arc<Mutex<HashMap<PathBuf, ()>>>` — acquire lock for the path before write. Tier-3 durability: `QueuedTask { id, card_id, requirements: Vec<String>, claimed_at: Option<u64>, completed_at: Option<u64> }` stored in SQLite `queued_tasks` table. CLI additions: `Command::Goal { sub: GoalCommand }` and `Command::Change { sub: ChangeCommand }`. Tests in `crates/core/tests/orchestration.rs`.

### [T-6B01] Trigger Triage

- **Spec:** l2-trigger-triage.md (full), l1-orchestration.md (ORC-1), l1-scheduler-model.md (SCH-3)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus trigger_triage` passes: `triage()` on a cron envelope returns `SpawnReactor` (rule fallback); `triage()` on an event envelope with `event_kind` containing "error" returns `SpawnOrchestrator`; duplicate webhook within 30s window returns `Drop { reason: "dedup_suppressed" }`; cron envelopes bypass dedup (window=0); oversized payload returns `Drop { reason: "payload_invalid" }`; `ContentHash` of identical payload text is identical; two envelopes with different text produce different hashes; rate-limit exceeded returns `Drop { reason: "rate_limit" }`; `trigger list` CLI exits 0; `trigger replay` CLI exits 0 (no-op seam).
- **Notes:** New module `crates/core/src/trigger_triage.rs`. `SourceType` enum: `ChatMessage | Webhook | Cron | Event | SubAgentSpawn`. `TriggerPayload { excerpt: String, event_kind: Option<String>, urgency_hint: Option<String> }` — excerpt truncated to 200 tokens (approx 200 words). `TriggerEnvelope { id: String, source_type: SourceType, source_id: Option<String>, payload: TriggerPayload, received_at: u64, workspace_id: String, metadata: HashMap<String, String> }`. `TriageDecision` enum: `Drop { reason: String } | Notify { message: String, channel: String } | SpawnReactor { agent_tier: String, task: String } | SpawnOrchestrator { goal: String, priority: String }`. `DeduplicateCache { store: HashMap<(SourceType, Vec<u8>), u64>, window_sec_by_source: HashMap<SourceType, u32> }` — `Vec<u8>` is SHA-256 of (excerpt + event_kind). `ContentHash` computed via `std::collections::hash_map::DefaultHasher` (no dep needed) or simple SHA-256 using existing digest patterns in codebase. Local classifier stub: `fn local_classify(env: &TriggerEnvelope) -> (TriageDecision, f32) = (rule_classify(env), 0.0)` — always returns 0.0 confidence, forcing rule fallback. Cloud fallback seam: returns rule result when model router returns error. Rate limit state: `HashMap<SourceType, RateWindow>` where `RateWindow { count: u32, window_start: u64 }`. CLI: `Command::Trigger { sub: TriggerCommand }`. `TriggerRecord` stored in SQLite `trigger_log` table (id/source_type/decision/received_at). Tests in `crates/core/tests/trigger_triage.rs`.

### [T-6C01] Mission Mode

- **Spec:** l2-mission-mode.md (full), l1-orchestration.md (ORC-6/ORC-7/ORC-9/ORC-10)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus mission_mode` passes: mission created with id `mission-YYYYMMDD-HHMMSS` format under `<ws>/missions/`; Phase 1 end writes valid `prd.json` with at least one story with `passes: false`; confirm transitions to Phase 2; execution loop with `max_iterations=1` emits partial completion when no stories pass; `all_stories_pass` returns true only when all stories have `passes: true`; `MissionMode` env var `CRONUS_MISSION_MODE=lite` overrides config file; config file `missionMode: ultra` overrides compiled default; flag file `.mission-mode` written on mission start and cleared on abort; `WorkType::Bugfix` classification routes to rework gate; `ClarificationStatus::Pending` blocks plan generation; `mission start` CLI exits 0; `mission status` CLI exits 0; `mission abort` CLI exits 0.
- **Notes:** New module `crates/core/src/mission.rs`. `MissionMode` enum: `Lite | Full | Ultra | Off`. Resolution: `fn resolve_mode(ws_config: &WorkspaceConfig) -> MissionMode` checks env var first, then `ws_config.mission_mode`, then `Full`. `Mission { id: String, task: String, phase: MissionPhase, status: MissionStatus, mode: MissionMode, created_at: u64 }`. `MissionPhase: Exploration | Execution`. `MissionStatus: Planning | AwaitingConfirm | Running | Complete | Partial | Aborted`. `PrdDocument { project: String, description: String, user_stories: Vec<UserStory> }`. `UserStory { id: String, title: String, story: String, passes: bool }`. `fn all_stories_pass(prd: &PrdDocument) -> bool = prd.user_stories.iter().all(|s| s.passes)`. `LoopConfig` struct: mirrors `loop_config.json` schema. State files written as JSON at `<ws>/missions/<mission-id>/`. `progress.txt`: `OpenOptions::append(true)` writes. Flag file: `<ws>/.mission-mode` written as plain text. `WorkType` enum: `Feature | Bugfix | Refactor | Review | Docs | Test | Config`. `PhaseIntent` enum: `Planning | Implementation | Debugging | Review | Verification | Exploration`. `fn classify_work_type(prompt: &str) -> WorkType`: keyword matching on leading verb and key terms. `ClarificationItem { question: String, answer: Option<String>, source: ClarificationSource, locked: bool }`. `ClarificationSource: User | Prd | Inferred`. Plan generation blocked until all non-locked `ClarificationItem`s have an answer. `SpecPersistenceModel: FlowForward | FlowBack | LivingSpec`. CLI: `Command::Mission { sub: MissionCommand }`. Tests in `crates/core/tests/mission_mode.rs`.

### [T-6C02] Deep Research

- **Spec:** l2-deep-research.md (full), l1-orchestration.md (ORC-1)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus deep_research` passes: `ResearchJob` created with correct id and `max_rounds=5` default; date-grounding preamble contains today's date in `MMMM DD, YYYY` format (use a fixed test date); content filter rejects pages shorter than 200 chars; content filter deduplicates by URL (second fetch of same URL dropped); search stub returns empty vec, triggering partial report after round 1 with `success_criteria_met: false`; `partial_reason` non-empty when circuit breaker fires; untrusted-content wrapping calls `tool_security::untrusted_context_message` with correct label; report marked as `_protected` in the returned report struct; `research start` CLI exits 0; `research list` CLI exits 0; `research cancel` CLI exits 0.
- **Notes:** New module `crates/core/src/research.rs`. `ResearchJob { id: String, question: String, status: ResearchStatus, rounds_completed: u8, max_rounds: u8, created_at: u64 }`. `ResearchStatus: Planning | Running | Complete | Partial | Cancelled`. `ResearchPlan { sub_questions: Vec<String>, key_topics: Vec<String>, success_criteria: String }`. `Citation { url: String, title: String, retrieved_at: u64 }`. `ResearchReport { question: String, sub_questions_answered: Vec<String>, report_body: String, citations: Vec<Citation>, rounds_completed: u8, success_criteria_met: bool, partial_reason: Option<String>, is_protected: bool }`. Date-grounding preamble: `fn date_grounding_preamble(now_ms: u64) -> String` formats the timestamp as `"Today's date is {Month DD, YYYY} ({YYYY-MM-DD}).\n..."`. Content filter: `fn filter_page(page: &FetchedPage, seen_urls: &mut HashSet<String>) -> bool` — rejects if `content.len() < 200` or URL already in `seen_urls`. Untrusted wrapping: `tool_security::untrusted_context_message(url, content)` re-used directly. Search stub: `fn search_stub(_query: &str) -> Vec<FetchedPage> = vec![]`. Search backend is pluggable via `SearchProvider` trait (seam, returns stub at Phase 6). State storage: `<ws>/research/<job-id>/plan.json`, `rounds/<n>.json`, `report.json`. CLI: `Command::Research { sub: ResearchCommand }`. Tests in `crates/core/tests/deep_research.rs`.

### [T-6T01] Phase 6 Integration Tests + CLI Command Additions

- **Spec:** all Phase 6 specs (integration coverage)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test --workspace` passes with 0 failures; `cargo clippy --all-targets -- -D warnings` returns 0 warnings; `cargo fmt --all -- --check` passes; all Phase 6 CLI smoke tests in `crates/cli/tests/cli_smoke.rs` exit 0 (goal / trigger list / mission / research list); cross-module round-trip test (trigger envelope → triage → SpawnOrchestrator decision recorded) passes; mission mode round-trip (start → prd written → confirm → max_iterations=1 partial) passes; deep research round-trip (start → plan generated → stub round → partial report) passes.
- **Notes:** New test files in `crates/core/tests/`: `orchestration.rs`, `trigger_triage.rs`, `mission_mode.rs`, `deep_research.rs`. Cross-module round-trip in `orchestration.rs`: create `TriggerEnvelope` with `SourceType::Event` + event_kind "error" → call `triage()` → assert `SpawnOrchestrator` → create `GoalRun` with max_iterations=1 → call judge seam → assert `JudgeVerdict::NotMet` → assert `GoalRunStatus::Paused`. CLI smoke tests appended to `crates/cli/tests/cli_smoke.rs`: `goal_stop_exits_0`, `trigger_list_exits_0`, `mission_list_exits_0`, `research_list_exits_0`. No new external deps. Full quality gate: `cargo test --workspace && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`.

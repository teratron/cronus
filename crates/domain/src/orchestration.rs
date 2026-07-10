//! Orchestration engine — delegation, goal/judge/budget loop, and tier hierarchy.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// ── Agent tier hierarchy ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTier {
    Chat,
    Reasoning,
    Worker,
}

impl AgentTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentTier::Chat => "chat",
            AgentTier::Reasoning => "reasoning",
            AgentTier::Worker => "worker",
        }
    }

    pub fn default_iteration_budget(&self) -> u32 {
        match self {
            AgentTier::Chat => 10,
            AgentTier::Reasoning => 50,
            AgentTier::Worker => 90,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentWorkspaceScope {
    Session,
    Execution,
    Isolated,
}

impl AgentWorkspaceScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentWorkspaceScope::Session => "session",
            AgentWorkspaceScope::Execution => "execution",
            AgentWorkspaceScope::Isolated => "isolated",
        }
    }
}

/// A tiered agent definition for static validation.
#[derive(Debug, Clone)]
pub struct TieredAgentDef {
    pub name: String,
    pub tier: AgentTier,
    pub toolsets: Vec<String>,
    pub workspace_scope: AgentWorkspaceScope,
    pub iteration_budget: Option<u32>,
}

/// Error produced when a `TieredAgentDef` violates tier constraints.
#[derive(Debug)]
pub enum AgentDefinitionError {
    /// Chat agents may not declare shell or code_execution toolsets.
    ChatForbiddenToolset { toolset: String },
    /// Worker agents must have execution or isolated workspace scope.
    WorkerRequiresExecutionScope,
    /// Unknown toolset name in the definition.
    UnknownToolset { name: String },
}

impl std::fmt::Display for AgentDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentDefinitionError::ChatForbiddenToolset { toolset } => {
                write!(f, "chat agent may not declare toolset '{toolset}'")
            }
            AgentDefinitionError::WorkerRequiresExecutionScope => {
                write!(
                    f,
                    "worker agent must use 'execution' or 'isolated' workspace scope"
                )
            }
            AgentDefinitionError::UnknownToolset { name } => {
                write!(f, "unknown toolset '{name}'")
            }
        }
    }
}

impl std::error::Error for AgentDefinitionError {}

const FORBIDDEN_CHAT_TOOLSETS: &[&str] = &["shell", "code_execution"];
const KNOWN_TOOLSETS: &[&str] = &[
    "read",
    "write",
    "shell",
    "code_execution",
    "search",
    "memory",
    "browser",
];

/// Validate that a `TieredAgentDef` conforms to its declared tier's constraints.
pub fn validate_tier(def: &TieredAgentDef) -> Result<(), AgentDefinitionError> {
    for ts in &def.toolsets {
        if !KNOWN_TOOLSETS.contains(&ts.as_str()) {
            return Err(AgentDefinitionError::UnknownToolset { name: ts.clone() });
        }
    }
    if def.tier == AgentTier::Chat {
        for ts in &def.toolsets {
            if FORBIDDEN_CHAT_TOOLSETS.contains(&ts.as_str()) {
                return Err(AgentDefinitionError::ChatForbiddenToolset {
                    toolset: ts.clone(),
                });
            }
        }
    }
    if def.tier == AgentTier::Worker && def.workspace_scope == AgentWorkspaceScope::Session {
        return Err(AgentDefinitionError::WorkerRequiresExecutionScope);
    }
    Ok(())
}

// ── Spawn depth ──────────────────────────────────────────────────────────────

/// Maximum allowed agent spawn depth. Orchestrator is at depth 0.
pub const MAX_SPAWN_DEPTH: u32 = 3;

/// Returned when an agent at `MAX_SPAWN_DEPTH` tries to spawn a child.
#[derive(Debug)]
pub struct SpawnDepthError {
    pub depth: u32,
}

impl std::fmt::Display for SpawnDepthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "max spawn depth {} reached (current depth: {})",
            MAX_SPAWN_DEPTH, self.depth
        )
    }
}

impl std::error::Error for SpawnDepthError {}

/// Tracks the current spawn depth for a delegation chain.
#[derive(Debug, Default)]
pub struct SpawnDepthCounter {
    current: u32,
}

impl SpawnDepthCounter {
    pub fn new() -> Self {
        SpawnDepthCounter { current: 0 }
    }

    /// Increment depth. Returns `SpawnDepthError` if the current depth is at the limit.
    pub fn try_increment(&mut self) -> Result<(), SpawnDepthError> {
        if self.current >= MAX_SPAWN_DEPTH {
            return Err(SpawnDepthError {
                depth: self.current,
            });
        }
        self.current += 1;
        Ok(())
    }

    pub fn decrement(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    pub fn depth(&self) -> u32 {
        self.current
    }
}

// ── Toolkit action ranking ───────────────────────────────────────────────────

/// A tool available to an agent.
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}

/// Threshold above which action ranking is applied.
const RANKING_THRESHOLD: usize = 50;

/// CPU-only action ranking: verb + unigram overlap scores.
///
/// Returns up to `top_n` tools sorted by relevance. When the catalog has ≤ 50
/// entries the full list is returned unsorted (no ranking needed).
pub fn rank_tools(task: &str, tools: &[ToolDefinition], top_n: usize) -> Vec<ToolDefinition> {
    if tools.len() <= RANKING_THRESHOLD {
        return tools.to_vec();
    }

    let task_words: Vec<&str> = task.split_whitespace().collect();
    let task_verb = task_words.first().copied().unwrap_or("");

    let mut scored: Vec<(usize, f64)> = tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let verb_score = if !task_verb.is_empty()
                && tool.name.to_lowercase().contains(&task_verb.to_lowercase())
            {
                1.0
            } else {
                0.0
            };
            let desc_words: Vec<&str> = tool.description.split_whitespace().collect();
            let token_score = task_words
                .iter()
                .filter(|w| {
                    desc_words
                        .iter()
                        .any(|d| d.to_lowercase() == w.to_lowercase())
                })
                .count() as f64;
            (i, verb_score + token_score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
        .into_iter()
        .take(top_n)
        .map(|(i, _)| tools[i].clone())
        .collect()
}

// ── Permission ruleset evaluation ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionAction {
    Allow,
    Deny,
    Ask,
}

impl PermissionAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionAction::Allow => "allow",
            PermissionAction::Deny => "deny",
            PermissionAction::Ask => "ask",
        }
    }
}

/// A single rule in a permission ruleset.
#[derive(Debug, Clone)]
pub struct PermissionRule {
    /// Permission being checked, e.g. `"fs:write:*"`.
    pub permission: String,
    /// Path or glob pattern, e.g. `"/home/**"`.
    pub pattern: String,
    pub action: PermissionAction,
}

/// Simple wildcard match (`*` matches any sequence of characters).
fn wildcard_match(text: &str, pattern: &str) -> bool {
    wildcard_match_inner(text.as_bytes(), pattern.as_bytes())
}

fn wildcard_match_inner(text: &[u8], pattern: &[u8]) -> bool {
    match (text, pattern) {
        (_, [b'*', rest @ ..]) => {
            // '*' can match 0 or more chars
            wildcard_match_inner(text, rest)
                || (!text.is_empty() && wildcard_match_inner(&text[1..], pattern))
        }
        ([t, tr @ ..], [p, pr @ ..]) => (*t == *p || *p == b'?') && wildcard_match_inner(tr, pr),
        ([], []) => true,
        _ => false,
    }
}

/// Evaluate a permission request against an ordered list of rulesets.
///
/// Rules from all rulesets are concatenated; **the last matching rule wins**.
/// Returns `Ask` when no rule matches (default: ask the user).
pub fn evaluate(
    permission: &str,
    path: &str,
    rulesets: &[Vec<PermissionRule>],
) -> PermissionAction {
    let mut result: Option<PermissionAction> = None;
    for ruleset in rulesets {
        for rule in ruleset {
            if wildcard_match(permission, &rule.permission) && wildcard_match(path, &rule.pattern) {
                result = Some(rule.action);
            }
        }
    }
    result.unwrap_or(PermissionAction::Ask)
}

// ── Tool terminate batch semantics ───────────────────────────────────────────

/// Result returned by a single tool invocation.
#[derive(Debug, Clone)]
pub struct AgentToolResult {
    pub content: Vec<String>,
    /// When true, this tool signals that the agent loop should stop — but only
    /// if ALL tools in the current batch also return `terminate: true`.
    pub terminate: bool,
}

/// Returns `true` only when every result in the batch sets `terminate: true`.
pub fn should_terminate_batch(results: &[AgentToolResult]) -> bool {
    !results.is_empty() && results.iter().all(|r| r.terminate)
}

// ── Goal run / judge / budget ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalRunStatus {
    Running,
    /// Paused because the iteration or budget limit was reached.
    Paused,
    Complete,
    Failed,
}

impl GoalRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GoalRunStatus::Running => "running",
            GoalRunStatus::Paused => "paused",
            GoalRunStatus::Complete => "complete",
            GoalRunStatus::Failed => "failed",
        }
    }
}

/// State of a single `/goal` run.
#[derive(Debug, Clone)]
pub struct GoalRun {
    pub id: String,
    pub goal: String,
    pub budget_usd: f64,
    pub max_iterations: u32,
    pub iteration: u32,
    pub status: GoalRunStatus,
}

impl GoalRun {
    pub fn new(id: String, goal: String, budget_usd: f64, max_iterations: u32) -> Self {
        GoalRun {
            id,
            goal,
            budget_usd,
            max_iterations,
            iteration: 0,
            status: GoalRunStatus::Running,
        }
    }

    /// Advance by one iteration and check the circuit-breaker.
    pub fn tick(&mut self) {
        self.iteration += 1;
        if self.iteration >= self.max_iterations && self.status == GoalRunStatus::Running {
            self.status = GoalRunStatus::Paused;
        }
    }

    pub fn mark_complete(&mut self) {
        self.status = GoalRunStatus::Complete;
    }

    pub fn mark_failed(&mut self) {
        self.status = GoalRunStatus::Failed;
    }

    pub fn is_exhausted(&self) -> bool {
        self.status == GoalRunStatus::Paused || self.status == GoalRunStatus::Failed
    }
}

/// Outcome of the independent judge evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JudgeVerdict {
    Met,
    NotMet { reason: String },
}

/// Judge seam — currently returns `NotMet` with the summary as the reason.
/// A real LLM call wires in when the model router has provider credentials.
pub fn evaluate_goal(_run: &GoalRun, summary: &str) -> JudgeVerdict {
    if summary.is_empty() {
        JudgeVerdict::NotMet {
            reason: "no summary provided".to_string(),
        }
    } else {
        JudgeVerdict::NotMet {
            reason: format!("judge seam: goal criteria not yet machine-verifiable — {summary}"),
        }
    }
}

// ── Tool execution mode ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolExecutionMode {
    #[default]
    Sequential,
    Parallel,
}

/// Agent loop configuration hooks (declared as a plain struct;
/// hook invocations wire in from the TUI/app layer in later phases).
#[derive(Debug, Default)]
pub struct AgentLoopConfig {
    pub tool_execution_mode: ToolExecutionMode,
    pub max_iterations: u32,
    /// When true, steering messages are injected before each provider call.
    pub inject_steering: bool,
}

impl AgentLoopConfig {
    pub fn new(max_iterations: u32) -> Self {
        AgentLoopConfig {
            tool_execution_mode: ToolExecutionMode::Sequential,
            max_iterations,
            inject_steering: false,
        }
    }
}

// ── Per-file mutation queue ───────────────────────────────────────────────────

/// Process-global per-file serialization lock.
///
/// Callers acquire the lock for a given path, perform their write, then drop
/// the guard. Concurrent operations on different files are never blocked.
#[derive(Debug, Default)]
pub struct FileMutationQueue {
    locks: Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>,
}

impl FileMutationQueue {
    pub fn new() -> Self {
        FileMutationQueue::default()
    }

    /// Return the per-file lock for `path`. Creates one if not present.
    pub fn acquire(&self, path: &Path) -> Arc<Mutex<()>> {
        let canonical = path.to_path_buf();
        let mut map = self.locks.lock().unwrap_or_else(|e| e.into_inner());
        Arc::clone(
            map.entry(canonical)
                .or_insert_with(|| Arc::new(Mutex::new(()))),
        )
    }
}

// ── Three-tier durability: Tier-3 queue ──────────────────────────────────────

/// A task in the durable SQLite-backed queue (Tier-3 durability).
#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub id: String,
    pub card_id: String,
    pub requirements: Vec<String>,
    pub claimed_at: Option<u64>,
    pub completed_at: Option<u64>,
}

// ── Change dependency metadata ────────────────────────────────────────────────

/// Per-change metadata for the `cronus change` command surface.
#[derive(Debug, Clone, Default)]
pub struct ChangeMetadata {
    pub id: String,
    pub depends_on: Vec<String>,
    pub provides: Vec<String>,
    pub requires: Vec<String>,
    pub touches: Vec<String>,
    pub why: String,
}

impl ChangeMetadata {
    pub fn new(id: impl Into<String>) -> Self {
        ChangeMetadata {
            id: id.into(),
            ..Default::default()
        }
    }
}

/// Detect cycles in a set of change dependencies. Returns true if no cycles.
pub fn changes_are_acyclic(changes: &[ChangeMetadata]) -> bool {
    let index: HashMap<&str, &ChangeMetadata> =
        changes.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut visiting: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();

    fn dfs<'a>(
        id: &'a str,
        index: &HashMap<&'a str, &'a ChangeMetadata>,
        visiting: &mut std::collections::HashSet<&'a str>,
        visited: &mut std::collections::HashSet<&'a str>,
    ) -> bool {
        if visiting.contains(id) {
            return false;
        }
        if visited.contains(id) {
            return true;
        }
        visiting.insert(id);
        if let Some(node) = index.get(id) {
            for dep in &node.depends_on {
                if !dfs(dep.as_str(), index, visiting, visited) {
                    return false;
                }
            }
        }
        visiting.remove(id);
        visited.insert(id);
        true
    }

    for change in changes {
        if !dfs(&change.id, &index, &mut visiting, &mut visited) {
            return false;
        }
    }
    true
}

/// List changes that have no unresolved `depends_on` entries.
pub fn unblocked_changes(changes: &[ChangeMetadata]) -> Vec<&ChangeMetadata> {
    let all_ids: std::collections::HashSet<&str> = changes.iter().map(|c| c.id.as_str()).collect();
    changes
        .iter()
        .filter(|c| {
            c.depends_on
                .iter()
                .all(|dep| !all_ids.contains(dep.as_str()))
        })
        .collect()
}

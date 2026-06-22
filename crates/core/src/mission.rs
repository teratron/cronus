//! Mission Mode — two-phase autonomous goal execution with user checkpoint.

use std::path::{Path, PathBuf};

// ── Operation mode ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissionMode {
    Lite,
    #[default]
    Full,
    Ultra,
    Off,
}

impl MissionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MissionMode::Lite => "lite",
            MissionMode::Full => "full",
            MissionMode::Ultra => "ultra",
            MissionMode::Off => "off",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "lite" => Some(MissionMode::Lite),
            "full" => Some(MissionMode::Full),
            "ultra" => Some(MissionMode::Ultra),
            "off" => Some(MissionMode::Off),
            _ => None,
        }
    }
}

/// Three-source priority resolution for mission mode.
///
/// Priority: env var `CRONUS_MISSION_MODE` → `config_value` → compiled default `Full`.
pub fn resolve_mode(config_value: Option<&str>) -> MissionMode {
    if let Ok(env_val) = std::env::var("CRONUS_MISSION_MODE")
        && let Some(m) = MissionMode::from_name(&env_val)
    {
        return m;
    }
    if let Some(val) = config_value
        && let Some(m) = MissionMode::from_name(val)
    {
        return m;
    }
    MissionMode::Full
}

// ── Mission phases and status ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionPhase {
    Exploration,
    Execution,
}

impl MissionPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            MissionPhase::Exploration => "exploration",
            MissionPhase::Execution => "execution",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionStatus {
    Planning,
    AwaitingConfirm,
    Running,
    Complete,
    /// Max iterations reached before all stories passed.
    Partial,
    Aborted,
}

impl MissionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MissionStatus::Planning => "planning",
            MissionStatus::AwaitingConfirm => "awaiting_confirm",
            MissionStatus::Running => "running",
            MissionStatus::Complete => "complete",
            MissionStatus::Partial => "partial",
            MissionStatus::Aborted => "aborted",
        }
    }
}

// ── PRD document ──────────────────────────────────────────────────────────────

/// A single user story with acceptance criteria.
#[derive(Debug, Clone)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    pub story: String,
    pub passes: bool,
}

impl UserStory {
    pub fn new(id: impl Into<String>, title: impl Into<String>, story: impl Into<String>) -> Self {
        UserStory {
            id: id.into(),
            title: title.into(),
            story: story.into(),
            passes: false,
        }
    }
}

/// The Product Requirements Document — source of truth for mission termination.
#[derive(Debug, Clone, Default)]
pub struct PrdDocument {
    pub project: String,
    pub description: String,
    pub user_stories: Vec<UserStory>,
}

impl PrdDocument {
    pub fn new(project: impl Into<String>, description: impl Into<String>) -> Self {
        PrdDocument {
            project: project.into(),
            description: description.into(),
            user_stories: Vec::new(),
        }
    }

    pub fn add_story(&mut self, story: UserStory) {
        self.user_stories.push(story);
    }
}

/// Returns true only when every user story has `passes: true`.
pub fn all_stories_pass(prd: &PrdDocument) -> bool {
    !prd.user_stories.is_empty() && prd.user_stories.iter().all(|s| s.passes)
}

// ── Loop config ───────────────────────────────────────────────────────────────

/// Git and session context for a mission. Mirrors `loop_config.json`.
#[derive(Debug, Clone, Default)]
pub struct LoopConfig {
    pub session_id: String,
    pub branch_name: String,
    pub git_installed: bool,
    pub is_git_repo: bool,
    pub default_branch: String,
    pub current_branch: String,
    pub repo_root: String,
}

// ── Mission ───────────────────────────────────────────────────────────────────

/// A running or completed mission.
#[derive(Debug, Clone)]
pub struct Mission {
    pub id: String,
    pub task: String,
    pub phase: MissionPhase,
    pub status: MissionStatus,
    pub mode: MissionMode,
    pub created_at: u64,
    pub iteration: u32,
    pub max_iterations: u32,
}

impl Mission {
    /// Generate a mission ID from a timestamp (ms since epoch).
    pub fn make_id(created_at_ms: u64) -> String {
        // Convert ms to seconds for the date part
        let secs = created_at_ms / 1000;
        let (y, mo, d, h, mi, s) = epoch_to_datetime(secs);
        format!("mission-{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
    }

    pub fn new(
        id: String,
        task: String,
        mode: MissionMode,
        max_iterations: u32,
        created_at: u64,
    ) -> Self {
        Mission {
            id,
            task,
            phase: MissionPhase::Exploration,
            status: MissionStatus::Planning,
            mode,
            created_at,
            iteration: 0,
            max_iterations,
        }
    }

    /// Transition to Phase 2 after the user confirms the PRD.
    pub fn confirm(&mut self) {
        self.phase = MissionPhase::Execution;
        self.status = MissionStatus::Running;
    }

    /// Advance one execution iteration; returns whether the loop should stop.
    pub fn tick(&mut self, prd: &PrdDocument) -> bool {
        self.iteration += 1;
        if all_stories_pass(prd) {
            self.status = MissionStatus::Complete;
            return true;
        }
        if self.iteration >= self.max_iterations {
            self.status = MissionStatus::Partial;
            return true;
        }
        false
    }

    pub fn abort(&mut self) {
        self.status = MissionStatus::Aborted;
    }
}

// ── Session intent classification ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkType {
    Feature,
    Bugfix,
    Refactor,
    Review,
    Docs,
    Test,
    Config,
}

impl WorkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkType::Feature => "feature",
            WorkType::Bugfix => "bugfix",
            WorkType::Refactor => "refactor",
            WorkType::Review => "review",
            WorkType::Docs => "docs",
            WorkType::Test => "test",
            WorkType::Config => "config",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseIntent {
    Planning,
    Implementation,
    Debugging,
    Review,
    Verification,
    Exploration,
}

impl PhaseIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            PhaseIntent::Planning => "Planning",
            PhaseIntent::Implementation => "Implementation",
            PhaseIntent::Debugging => "Debugging",
            PhaseIntent::Review => "Review",
            PhaseIntent::Verification => "Verification",
            PhaseIntent::Exploration => "Exploration",
        }
    }
}

/// Classify work type from first-prompt keywords.
pub fn classify_work_type(prompt: &str) -> WorkType {
    let lower = prompt.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let first = words.first().copied().unwrap_or("");

    if matches!(first, "fix" | "bug" | "patch" | "error" | "broken")
        || lower.contains("bugfix")
        || lower.contains("bug fix")
    {
        return WorkType::Bugfix;
    }
    if matches!(first, "refactor" | "rewrite" | "clean" | "optimize") || lower.contains("refactor")
    {
        return WorkType::Refactor;
    }
    if matches!(first, "review" | "audit" | "inspect" | "explain") {
        return WorkType::Review;
    }
    if matches!(first, "doc" | "readme" | "comment" | "docstring") || lower.contains("docs") {
        return WorkType::Docs;
    }
    if matches!(first, "test" | "spec" | "unit" | "integration" | "e2e") {
        return WorkType::Test;
    }
    if matches!(first, "config" | "env" | "setup" | "docker" | "deploy") {
        return WorkType::Config;
    }
    WorkType::Feature
}

/// Classify phase intent from first-prompt keywords.
pub fn classify_phase_intent(prompt: &str) -> (PhaseIntent, u8) {
    let lower = prompt.to_lowercase();
    if lower.contains("plan") || lower.contains("architect") || lower.contains("design") {
        return (PhaseIntent::Planning, 80);
    }
    if lower.contains("fix") || lower.contains("error") || lower.contains("exception") {
        return (PhaseIntent::Debugging, 75);
    }
    if lower.contains("review") || lower.contains("audit") || lower.contains("explain") {
        return (PhaseIntent::Review, 80);
    }
    if lower.contains("test") || lower.contains("validate") || lower.contains("assert") {
        return (PhaseIntent::Verification, 80);
    }
    if lower.contains("build") || lower.contains("create") || lower.contains("implement") {
        return (PhaseIntent::Implementation, 75);
    }
    (PhaseIntent::Exploration, 60)
}

// ── Clarification protocol ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClarificationSource {
    User,
    Prd,
    Inferred,
}

impl ClarificationSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClarificationSource::User => "user",
            ClarificationSource::Prd => "prd",
            ClarificationSource::Inferred => "inferred",
        }
    }
}

/// A single clarification question.
#[derive(Debug, Clone)]
pub struct ClarificationItem {
    pub question: String,
    pub answer: Option<String>,
    pub source: ClarificationSource,
    pub locked: bool,
    pub skipped: bool,
}

impl ClarificationItem {
    pub fn new(question: impl Into<String>) -> Self {
        ClarificationItem {
            question: question.into(),
            answer: None,
            source: ClarificationSource::User,
            locked: false,
            skipped: false,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.answer.is_some() || self.skipped
    }
}

/// Returns true when all non-locked items are resolved (plan generation can proceed).
pub fn clarifications_complete(items: &[ClarificationItem]) -> bool {
    items.iter().filter(|i| !i.locked).all(|i| i.is_resolved())
}

// ── Spec persistence model ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpecPersistenceModel {
    #[default]
    FlowForward,
    FlowBack,
    LivingSpec,
}

impl SpecPersistenceModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpecPersistenceModel::FlowForward => "flow-forward",
            SpecPersistenceModel::FlowBack => "flow-back",
            SpecPersistenceModel::LivingSpec => "living-spec",
        }
    }
}

// ── Proposal artifact ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Draft,
    Ready,
    Accepted,
    Rejected,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalStatus::Draft => "draft",
            ProposalStatus::Ready => "ready",
            ProposalStatus::Accepted => "accepted",
            ProposalStatus::Rejected => "rejected",
        }
    }
}

/// A change proposal (root node of the artifact dependency graph).
#[derive(Debug, Clone)]
pub struct Proposal {
    pub change_id: String,
    pub created: String,
    pub status: ProposalStatus,
    pub mode: MissionMode,
    pub intent: String,
}

impl Proposal {
    pub fn new(
        change_id: impl Into<String>,
        created: impl Into<String>,
        mode: MissionMode,
        intent: impl Into<String>,
    ) -> Self {
        Proposal {
            change_id: change_id.into(),
            created: created.into(),
            status: ProposalStatus::Draft,
            mode,
            intent: intent.into(),
        }
    }

    pub fn mark_ready(&mut self) {
        if self.status == ProposalStatus::Draft {
            self.status = ProposalStatus::Ready;
        }
    }

    pub fn accept(&mut self) {
        if self.status == ProposalStatus::Ready {
            self.status = ProposalStatus::Accepted;
        }
    }

    pub fn reject(&mut self) {
        if self.status == ProposalStatus::Ready {
            self.status = ProposalStatus::Rejected;
        }
    }

    /// Once accepted, the proposal is immutable.
    pub fn is_immutable(&self) -> bool {
        self.status == ProposalStatus::Accepted
    }
}

// ── Mission store (file-backed) ───────────────────────────────────────────────

/// Returns the directory for storing mission state files.
pub fn mission_dir(workspace_root: &Path, mission_id: &str) -> PathBuf {
    workspace_root.join("missions").join(mission_id)
}

/// Write the flag file indicating the active mission mode.
pub fn write_mode_flag(workspace_root: &Path, mode: MissionMode) -> std::io::Result<()> {
    let flag = workspace_root.join(".mission-mode");
    std::fs::write(flag, mode.as_str())
}

/// Clear the mission mode flag file on completion or abort.
pub fn clear_mode_flag(workspace_root: &Path) -> std::io::Result<()> {
    let flag = workspace_root.join(".mission-mode");
    std::fs::write(flag, "off")
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Convert Unix seconds to (year, month, day, hour, min, sec) — no external deps.
fn epoch_to_datetime(secs: u64) -> (u32, u8, u8, u8, u8, u8) {
    let s = secs % 60;
    let total_min = secs / 60;
    let mi = (total_min % 60) as u8;
    let total_hr = total_min / 60;
    let h = (total_hr % 24) as u8;
    let total_days = total_hr / 24;

    // Days since 1970-01-01
    let mut year = 1970u32;
    let mut days = total_days;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 0u8;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = (i + 1) as u8;
            break;
        }
        days -= md;
    }
    let day = (days + 1) as u8;
    (year, month, day, h, mi, s as u8)
}

fn is_leap(y: u32) -> bool {
    y.is_multiple_of(4) && !y.is_multiple_of(100) || y.is_multiple_of(400)
}

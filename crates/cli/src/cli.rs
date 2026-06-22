use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "cronus",
    about = "Cronus — workflow automation toolkit",
    version
)]
pub struct Cli {
    /// Output format
    #[arg(long, short = 'f', global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a Cronus workspace in the target directory
    Init {
        /// Target path (defaults to the current directory)
        path: Option<PathBuf>,
    },
    /// Show the current workspace status
    Status,
    /// Manage workflow (.nodus) files
    Workflow {
        #[command(subcommand)]
        sub: WorkflowCommand,
    },
    /// Manage named workspaces
    Workspace {
        #[command(subcommand)]
        sub: WorkspaceCommand,
    },
    /// Manage memory entries
    Memory {
        #[command(subcommand)]
        sub: MemoryCommand,
    },
    /// Code graph operations
    Codegraph {
        #[command(subcommand)]
        sub: CodegraphCommand,
    },
    /// Agent management
    Agent {
        #[command(subcommand)]
        sub: AgentCommand,
    },
    /// Role catalog: hire, fire, and manage agent roles
    Role {
        #[command(subcommand)]
        sub: RoleCommand,
    },
    /// Kanban board: track work cards through a state machine
    Board {
        #[command(subcommand)]
        sub: BoardCommand,
    },
    /// Scheduler: manage recurring and one-shot schedules
    Schedule {
        #[command(subcommand)]
        sub: ScheduleCommand,
    },
    /// Budget: track and enforce cost policies
    Budget {
        #[command(subcommand)]
        sub: BudgetCommand,
    },
    /// Execution workspaces: isolated git worktrees per card
    Exec {
        #[command(subcommand)]
        sub: ExecCommand,
    },
    /// Quality gates: run lint/test/format checks
    Check {
        #[command(subcommand)]
        sub: CheckCommand,
    },
    /// Extensions: manage skills, MCP servers, and plugins
    Ext {
        #[command(subcommand)]
        sub: ExtCommand,
    },
    /// Learning loop: review and approve proposed skills
    Learn {
        #[command(subcommand)]
        sub: LearnCommand,
    },
    /// Agent registry: list and manage agent definitions
    Registry {
        #[command(subcommand)]
        sub: RegistryCommand,
    },
    /// Goal runs: start and manage autonomous goal sessions
    Goal {
        #[command(subcommand)]
        sub: GoalCommand,
    },
    /// Trigger triage: classify inbound signals and manage dispatch history
    Trigger {
        #[command(subcommand)]
        sub: TriggerCommand,
    },
    /// Mission mode: two-phase autonomous goal execution
    Mission {
        #[command(subcommand)]
        sub: MissionCommand,
    },
    /// Deep research: iterative search-and-synthesize jobs
    Research {
        #[command(subcommand)]
        sub: ResearchCommand,
    },
    /// Change graph: inspect and sequence pending changes
    Change {
        #[command(subcommand)]
        sub: ChangeCommand,
    },
}

#[derive(Subcommand)]
pub enum MemoryCommand {
    /// Store a key-value memory entry
    Store {
        /// Entry title / key
        key: String,
        /// Entry body / value
        value: String,
    },
    /// Search memory entries by keyword
    Search {
        /// Search query
        query: String,
    },
    /// Delete a memory entry by ID
    Forget {
        /// Entry ID to delete
        id: String,
    },
}

#[derive(Subcommand)]
pub enum CodegraphCommand {
    /// Index a directory or file
    Index {
        /// Path to index
        path: std::path::PathBuf,
    },
    /// Search the code graph by keyword
    Search {
        /// Search query
        query: String,
    },
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Show agent identity files and status
    Constitution,
    /// Show agent runtime status
    Status,
}

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Create a new workspace
    Create {
        /// Workspace ID (lowercase kebab-case, e.g. my-project)
        id: String,
        /// Human-readable name
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// Root path for the workspace (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// List all workspaces
    List,
    /// Switch the active workspace
    Switch {
        /// Workspace ID to activate
        id: String,
    },
    /// Delete a workspace
    Delete {
        /// Workspace ID to delete
        id: String,
    },
    /// Check the status of a workspace
    Check {
        /// Workspace ID to inspect
        id: String,
    },
}

#[derive(Subcommand)]
pub enum RoleCommand {
    /// List hired instances (or preset catalog with --presets)
    List {
        #[arg(long)]
        presets: bool,
    },
    /// Hire a role from the preset catalog
    Hire {
        preset: String,
        #[arg(long)]
        name: Option<String>,
    },
    /// Show a hired role instance
    Show { id: String },
    /// Create a custom role
    Create { id: String, display_name: String },
    /// Fire (remove) a hired role
    Fire { id: String },
}

#[derive(Subcommand)]
pub enum BoardCommand {
    /// List board cards
    List,
    /// Show a card
    Show { id: String },
    /// Add a card
    Add { id: String, task_ref: String },
    /// Move a card to a new state
    Move {
        id: String,
        state: String,
        #[arg(long, default_value = "cli")]
        actor: String,
    },
    /// Block a card with a reason
    Block { id: String, reason: String },
    /// Mark a card as done
    Done {
        id: String,
        #[arg(long, default_value = "cli")]
        actor: String,
    },
    /// Archive all eligible done cards
    Archive,
}

#[derive(Subcommand)]
pub enum ScheduleCommand {
    /// List all schedules
    List,
    /// Add a recurring schedule
    Add {
        id: String,
        name: String,
        #[arg(long, default_value = "daily")]
        preset: String,
    },
    /// Delete a schedule
    Delete { id: String },
    /// Fire a schedule immediately
    Run {
        id: String,
        #[arg(long, default_value = "run scheduled task")]
        prompt: String,
    },
}

#[derive(Subcommand)]
pub enum BudgetCommand {
    /// Show current budget usage
    Show,
    /// Set a workspace budget limit (USD)
    Set { limit: f64 },
    /// Reset budget counters
    Reset,
}

#[derive(Subcommand)]
pub enum ExecCommand {
    /// List execution workspaces
    List,
    /// Create an execution workspace for a card
    Create { ws_id: String, card_id: String },
    /// Finalize an execution workspace
    Finalize { id: String },
    /// Discard an execution workspace
    Discard { id: String },
}

#[derive(Subcommand)]
pub enum CheckCommand {
    /// Run quality gates for a card
    Run {
        card_id: String,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Show gate results for a card
    Show { card_id: String },
    /// Show gate result history for a card
    History { card_id: String },
}

#[derive(Subcommand)]
pub enum ExtCommand {
    /// List registered extensions
    List,
    /// Add an extension by manifest path
    Add { path: PathBuf },
    /// Remove an extension
    Remove { id: String },
    /// Scan an extension for security issues
    Scan { path: PathBuf },
    /// Activate an extension
    Activate { id: String },
    /// Deactivate an extension
    Deactivate { id: String },
}

#[derive(Subcommand)]
pub enum LearnCommand {
    /// List pending skill proposals
    List,
    /// Approve a skill proposal
    Approve { id: String },
    /// Reject a skill proposal
    Reject { id: String },
}

#[derive(Subcommand)]
pub enum RegistryCommand {
    /// List all agent definitions
    List,
    /// Show an agent definition
    Show { name: String },
    /// Create a custom agent entry
    Create { name: String, description: String },
    /// Disable an agent
    Disable { name: String },
    /// Enable a previously disabled agent
    Enable { name: String },
}

#[derive(Subcommand)]
pub enum GoalCommand {
    /// Start a new goal run
    Start {
        /// Goal description
        goal: String,
        /// Budget limit in USD
        #[arg(long, default_value_t = 5.0)]
        budget: f64,
        /// Max iterations before pausing
        #[arg(long, default_value_t = 10)]
        max_iter: u32,
    },
    /// Stop a running goal
    Stop {
        /// Goal run ID
        id: String,
    },
    /// Show status of a goal run
    Status {
        /// Goal run ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum TriggerCommand {
    /// List recent trigger history
    List,
    /// Show the triage history for an envelope
    History {
        /// Envelope ID
        id: String,
    },
    /// Replay a stored trigger envelope
    Replay {
        /// Envelope ID to replay
        id: String,
    },
}

#[derive(Subcommand)]
pub enum MissionCommand {
    /// Start a new mission
    Start {
        /// Task description
        task: String,
        /// Mission mode (lite, full, ultra, off)
        #[arg(long, default_value = "full")]
        mode: String,
    },
    /// Confirm a mission's exploration phase and begin execution
    Confirm {
        /// Mission ID
        id: String,
    },
    /// Show mission status
    Status {
        /// Mission ID
        id: String,
    },
    /// List all missions
    List,
    /// Resume a paused mission
    Resume {
        /// Mission ID
        id: String,
    },
    /// Abort a running mission
    Abort {
        /// Mission ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ResearchCommand {
    /// Start a new research job
    Start {
        /// Research question
        question: String,
        /// Maximum research rounds
        #[arg(long, default_value_t = 5)]
        max_rounds: u8,
    },
    /// Show the status of a research job
    Status {
        /// Job ID
        id: String,
    },
    /// Display the research report for a completed job
    Report {
        /// Job ID
        id: String,
    },
    /// List all research jobs
    List,
    /// Cancel a running research job
    Cancel {
        /// Job ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ChangeCommand {
    /// Show the change dependency graph
    Graph,
    /// List the next unblocked changes
    Next,
    /// Split a change into independent sub-changes
    Split {
        /// Change ID to split
        id: String,
    },
    /// Show status of a change
    Status {
        /// Change ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Generate a new workflow skeleton
    Scaffold {
        /// Workflow name (used as the file stem)
        name: String,
        /// Custom output path (defaults to ./<name>.nodus)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Validate a workflow file and report diagnostics
    Validate {
        /// Workflow file to validate
        file: PathBuf,
    },
    /// Execute a workflow file
    Run {
        /// Workflow file to execute
        file: PathBuf,
        /// Input values as a JSON object (e.g. '{"key":"val"}')
        #[arg(long)]
        input: Option<String>,
    },
    /// Transpile a workflow to a different representation
    Transpile {
        /// Workflow file to transpile
        file: PathBuf,
        /// Render human-prose form
        #[arg(long, conflicts_with = "compact")]
        human: bool,
        /// Render compact NODUS form (default)
        #[arg(long, conflicts_with = "human")]
        compact: bool,
    },
}

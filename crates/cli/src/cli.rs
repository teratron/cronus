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
    /// Self-healing: run health checks, optionally applying safe repairs
    Doctor {
        /// Apply safe, deterministic repairs (risky findings still escalate)
        #[arg(long)]
        fix: bool,
    },
    /// Back up the state tier (secrets and cache excluded)
    Backup {
        #[command(subcommand)]
        sub: BackupCommand,
    },
    /// Restore a backup into the current state tier
    Restore {
        /// Backup id (as shown by `cronus backup list`)
        backup: String,
    },
    /// Manage background activation (l2-service-activation)
    Activation {
        #[command(subcommand)]
        sub: ActivationCommand,
    },
    /// Run and inspect governed autonomous loops (l2-loop-runner)
    Loop {
        #[command(subcommand)]
        sub: LoopCommand,
    },
    /// Office archetypes: a prior on staffing, never a roster (l2-archetype-catalog)
    Archetype {
        #[command(subcommand)]
        sub: ArchetypeCommand,
    },
    /// Knowledge base: named collections with hybrid semantic+keyword retrieval (l2-knowledge-store)
    Knowledge {
        #[command(subcommand)]
        sub: KnowledgeCommand,
    },
}

#[derive(Subcommand)]
pub enum KnowledgeCommand {
    /// Create a named, access-controlled document collection (KB-1)
    CollectionCreate {
        /// Collection id
        id: String,
        /// Display name
        name: String,
    },
    /// Ingest a plain-text/JSON record into a collection (KB-5)
    Add {
        /// Target collection id
        #[arg(long)]
        collection: String,
        /// Document id
        #[arg(long = "id")]
        doc_id: String,
        /// Document display name
        #[arg(long)]
        name: String,
        /// The text content to chunk, embed, and index
        text: String,
    },
    /// Ingest a web page by URL (KB-5). `http://` only — see the
    /// `HttpUrlFetcher` disclosed scope (`https://` refused with a clear
    /// message; no `robots.txt`/rate-limit policy yet)
    AddUrl {
        #[arg(long)]
        collection: String,
        #[arg(long = "id")]
        doc_id: String,
        #[arg(long)]
        name: String,
        url: String,
    },
    /// Hybrid semantic+keyword retrieval over one or more collections
    /// (KB-1/KB-6/KB-7) — RRF-fused, KB-4-access-gated
    Query {
        /// Target collection ids (repeatable)
        #[arg(long = "collection", required = true)]
        collections: Vec<String>,
        /// Result count
        #[arg(long, default_value_t = 5)]
        top_k: usize,
        text: String,
    },
}

#[derive(Subcommand)]
pub enum ArchetypeCommand {
    /// List archetypes — the shipped catalog, or the office's active one
    List {
        /// Show the program-tier catalog (shipped + declared-blocked)
        #[arg(long)]
        catalog: bool,
        /// Show the office's currently active archetype
        #[arg(long)]
        active: bool,
    },
    /// Show one archetype's pool, shape, and seed (or its blocked reason)
    Info {
        /// Archetype id
        id: String,
        /// Include the deviation counters + validation status
        #[arg(long)]
        deviations: bool,
    },
    /// Apply an archetype, or return to the archetype-free default. Changes
    /// what the manager expects, never staff — non-destructive by construction
    Set {
        /// Archetype id to apply
        id: Option<String>,
        /// Return the office to the archetype-free state
        #[arg(long)]
        clear: bool,
    },
    /// Create a custom archetype by copying a preset into the state tier
    Create {
        /// Name for the new custom archetype
        name: String,
        /// Preset id to copy from
        #[arg(long = "from")]
        from: String,
    },
}

#[derive(Subcommand)]
pub enum BackupCommand {
    /// Create a backup
    Create {
        /// Destination path (defaults to `<state>/backups/backup-<timestamp>`)
        #[arg(long = "to")]
        to: Option<PathBuf>,
        /// Include the regenerable logs directory (excluded by default)
        #[arg(long)]
        include_logs: bool,
    },
    /// List backups under the state tier's `backups/` directory
    List,
}

#[derive(Subcommand)]
pub enum ActivationCommand {
    /// Print the observed activation state — read from the OS, never a
    /// remembered value (BA-8)
    Status,
    /// Register background activation for a mode (BA-5: an autonomy grant,
    /// not a preference — disclosed and confirmed before it takes effect)
    Enable {
        #[arg(long, value_enum)]
        mode: ActivationModeArg,
        /// Skip the interactive confirmation. Required in a non-interactive
        /// context (a script, a CI runner) — `enable` otherwise refuses,
        /// because an unattended grant would silently satisfy the consent
        /// moment BA-5 requires a human to see.
        #[arg(long)]
        acknowledge_unattended_execution: bool,
    },
    /// Remove whatever activation registration is currently active (BA-7:
    /// removed and verified, never left partially registered)
    Disable,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum ActivationModeArg {
    /// Starts with the user's session; ends when it ends; no elevation
    Login,
    /// Boot-started, survives logout; registration requires elevation
    System,
}

#[derive(Subcommand)]
pub enum LoopCommand {
    /// Run an execution loop over a unit until its oracle says done or the
    /// ceiling fires. This first cut's oracle is a file-existence check —
    /// full worktree/budget/model-router composition is future work.
    Run {
        /// Path whose existence is this unit's done-condition
        #[arg(long)]
        file: PathBuf,
        /// Maximum iterations before the ceiling stops the loop
        #[arg(long, default_value_t = 1)]
        max_iter: u32,
    },
    /// Run an evolution loop over a harness. Unavailable in this workspace:
    /// there is no CLI-nameable harness registry yet (the domain-tier
    /// `run_evolution` exists and is tested; wiring a real harness resource
    /// is future work — INV-9, never a silent success stub)
    Evolve {
        /// Harness identifier (accepted for the command shape; not yet
        /// resolvable to anything)
        harness_id: String,
    },
    /// Inspect a run's mutation ledger
    Log {
        /// Run id, as printed by `cronus loop run`
        run_id: String,
    },
    /// Show a run's manifest/oracle/ceiling
    Show {
        /// Run id, as printed by `cronus loop run`
        run_id: String,
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
    /// Skill packages: import, create, and inspect status
    Skill {
        #[command(subcommand)]
        sub: SkillCommand,
    },
}

#[derive(Subcommand)]
pub enum SkillCommand {
    /// Import and convert a foreign skill package
    Import {
        /// Path to the foreign skill file or package root
        path: PathBuf,
    },
    /// Author a new skill from a natural-language prompt
    Create {
        /// Description of the desired skill
        #[arg(long)]
        prompt: String,
    },
    /// Show conversion and review status for one or all tracked skills
    Status {
        /// Skill id (all tracked skills if omitted)
        id: Option<String>,
    },
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

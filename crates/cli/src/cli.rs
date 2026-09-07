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

/// Bound at runtime by the installation half's own generated grammar now
/// (`crate::installation`, `activation enable --mode`) — the `ArchetypeCommand`/
/// `BackupCommand`/`ActivationCommand`/`WorkspaceCommand` enums those verbs
/// used to derive from are tombstoned, but `enable`'s handler still takes
/// this small value type rather than a bare string.
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum ActivationModeArg {
    /// Starts with the user's session; ends when it ends; no elevation
    Login,
    /// Boot-started, survives logout; registration requires elevation
    System,
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

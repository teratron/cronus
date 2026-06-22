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

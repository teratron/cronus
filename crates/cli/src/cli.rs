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

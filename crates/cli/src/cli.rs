use clap::Parser;

use crate::output::OutputFormat;

/// The launcher's own top-level grammar: just the global `--format` flag and
/// the `--help`/`--version` clap provides for free. Every verb this product
/// ships now comes from one of two projections built at runtime
/// (`crate::installation`'s hand-declared grammar, `crate::generated`'s
/// registry-derived one) rather than from a field on this struct — a
/// compile-time subcommand enum is exactly the shape INV-9 forbids as a
/// source of truth, since an unshipped action would need to be representable
/// in it.
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

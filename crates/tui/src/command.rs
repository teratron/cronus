//! Slash-command parsing and the command catalog.
//!
//! The command bar accepts `/verb arg…` lines. This module turns a line into a
//! structured [`SlashCommand`], holds the [`CATALOG`] of known commands (mirroring
//! the CLI's top-level verb set so the two surfaces stay at parity, INV-3), and
//! classifies a submitted line into a [`CommandOutcome`]. Dispatch to the core is
//! a separate concern — this module never calls the core (INV-2).

/// A parsed slash command: the verb plus its whitespace-separated arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashCommand {
    /// The command verb (without the leading `/`).
    pub verb: String,
    /// Positional arguments following the verb.
    pub args: Vec<String>,
}

/// Why a command-bar line is not a runnable slash command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The line does not start with `/`.
    NotACommand,
    /// The line is `/` with no verb.
    Empty,
}

/// Parse a command-bar line into a [`SlashCommand`].
///
/// Syntactic only — it does not check the verb against the [`CATALOG`]; use
/// [`classify`] for that. Leading/trailing whitespace is ignored.
pub fn parse(input: &str) -> Result<SlashCommand, ParseError> {
    let body = input
        .trim()
        .strip_prefix('/')
        .ok_or(ParseError::NotACommand)?;
    let mut parts = body.split_whitespace();
    let verb = parts.next().ok_or(ParseError::Empty)?;
    Ok(SlashCommand {
        verb: verb.to_string(),
        args: parts.map(str::to_string).collect(),
    })
}

/// One entry in the slash-command catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    /// Verb name (without the leading `/`).
    pub name: &'static str,
    /// One-line summary shown by `/help`.
    pub summary: &'static str,
}

/// The slash-command catalog.
///
/// Each entry past `help` mirrors a CLI top-level command (same verb), so a TUI
/// slash command always has a CLI counterpart — the parity the validation track
/// asserts structurally. `help` is the TUI's own discovery affordance (the CLI
/// uses `--help`).
pub const CATALOG: &[CommandSpec] = &[
    CommandSpec {
        name: "help",
        summary: "List available slash commands",
    },
    CommandSpec {
        name: "init",
        summary: "Initialize a Cronus workspace",
    },
    CommandSpec {
        name: "status",
        summary: "Show the current workspace status",
    },
    CommandSpec {
        name: "workflow",
        summary: "Manage workflow (.nodus) files",
    },
    CommandSpec {
        name: "workspace",
        summary: "Manage named workspaces",
    },
    CommandSpec {
        name: "memory",
        summary: "Manage memory entries",
    },
    CommandSpec {
        name: "codegraph",
        summary: "Code graph operations",
    },
    CommandSpec {
        name: "agent",
        summary: "Agent management",
    },
    CommandSpec {
        name: "role",
        summary: "Role catalog: hire, fire, manage roles",
    },
    CommandSpec {
        name: "board",
        summary: "Kanban board: track work cards",
    },
    CommandSpec {
        name: "schedule",
        summary: "Scheduler: recurring and one-shot",
    },
    CommandSpec {
        name: "budget",
        summary: "Budget: track and enforce cost policies",
    },
    CommandSpec {
        name: "exec",
        summary: "Execution workspaces (git worktrees)",
    },
    CommandSpec {
        name: "check",
        summary: "Quality gates: lint/test/format",
    },
    CommandSpec {
        name: "ext",
        summary: "Extensions: skills, MCP servers, plugins",
    },
    CommandSpec {
        name: "learn",
        summary: "Learning loop: review proposed skills",
    },
    CommandSpec {
        name: "registry",
        summary: "Agent registry: list and manage",
    },
    CommandSpec {
        name: "goal",
        summary: "Goal runs: autonomous goal sessions",
    },
    CommandSpec {
        name: "trigger",
        summary: "Trigger triage: classify inbound signals",
    },
    CommandSpec {
        name: "mission",
        summary: "Mission mode: two-phase execution",
    },
    CommandSpec {
        name: "research",
        summary: "Deep research: search-and-synthesize",
    },
    CommandSpec {
        name: "change",
        summary: "Change graph: inspect pending changes",
    },
];

/// The catalog entry for `verb`, if it is a known command.
pub fn lookup(verb: &str) -> Option<&'static CommandSpec> {
    CATALOG.iter().find(|c| c.name == verb)
}

/// Whether `verb` is a known slash command.
pub fn is_known(verb: &str) -> bool {
    lookup(verb).is_some()
}

/// Every known verb name, in catalog order.
pub fn names() -> impl Iterator<Item = &'static str> {
    CATALOG.iter().map(|c| c.name)
}

/// The `/help` discovery listing: one `"/verb — summary"` line per command.
pub fn help_lines() -> Vec<String> {
    CATALOG
        .iter()
        .map(|c| format!("/{:<9} — {}", c.name, c.summary))
        .collect()
}

/// What submitting a command-bar line resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// Show the help listing (the discovery surface).
    Help,
    /// A recognized command ready to dispatch (executed by the dispatch step).
    Run(SlashCommand),
    /// An inline error to surface to the user.
    Error(String),
}

/// Classify a submitted command-bar line into an outcome.
pub fn classify(input: &str) -> CommandOutcome {
    match parse(input) {
        Err(ParseError::NotACommand) => CommandOutcome::Error("commands start with /".to_string()),
        Err(ParseError::Empty) => CommandOutcome::Error("type a command, e.g. /help".to_string()),
        Ok(cmd) if cmd.verb == "help" => CommandOutcome::Help,
        Ok(cmd) if is_known(&cmd.verb) => CommandOutcome::Run(cmd),
        Ok(cmd) => CommandOutcome::Error(format!("unknown command: /{} (try /help)", cmd.verb)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_parse_extracts_verb_and_args() {
        let cmd = parse("/goal start my-objective").expect("parses");
        assert_eq!(cmd.verb, "goal");
        assert_eq!(cmd.args, vec!["start", "my-objective"]);
    }

    #[test]
    fn command_parse_ignores_surrounding_whitespace() {
        let cmd = parse("   /status   ").expect("parses");
        assert_eq!(cmd.verb, "status");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn command_parse_rejects_non_slash_input() {
        assert_eq!(parse("status"), Err(ParseError::NotACommand));
    }

    #[test]
    fn command_parse_rejects_bare_slash() {
        assert_eq!(parse("/"), Err(ParseError::Empty));
        assert_eq!(parse("/   "), Err(ParseError::Empty));
    }

    #[test]
    fn command_parse_help_lists_known_commands() {
        let listing = help_lines();
        // Discovery surface includes representative CLI-mirrored verbs.
        for verb in ["status", "goal", "board", "memory"] {
            assert!(
                listing.iter().any(|l| l.contains(&format!("/{verb} "))),
                "/help should list /{verb}"
            );
        }
        assert_eq!(listing.len(), CATALOG.len());
    }

    #[test]
    fn command_parse_classifies_help_run_and_unknown() {
        assert_eq!(classify("/help"), CommandOutcome::Help);
        assert!(matches!(classify("/status"), CommandOutcome::Run(c) if c.verb == "status"));
        assert!(matches!(classify("/frobnicate"), CommandOutcome::Error(_)));
        assert!(matches!(classify("hello"), CommandOutcome::Error(_)));
    }

    #[test]
    fn command_parse_catalog_verbs_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for name in names() {
            assert!(seen.insert(name), "duplicate catalog verb: {name}");
        }
    }

    /// The CLI's top-level command verbs (crates/cli/src/cli.rs `Command` enum),
    /// kebab-cased. Mirrored here so parity can be asserted WITHOUT the TUI taking
    /// a dependency on `cronus-cli` — that dependency is itself forbidden (INV-2),
    /// and the parity check below would otherwise have to violate the very rule it
    /// guards. `help` is the TUI's own discovery affordance (the CLI uses `--help`).
    const EXPECTED_CLI_VERBS: &[&str] = &[
        "init",
        "status",
        "workflow",
        "workspace",
        "memory",
        "codegraph",
        "agent",
        "role",
        "board",
        "schedule",
        "budget",
        "exec",
        "check",
        "ext",
        "learn",
        "registry",
        "goal",
        "trigger",
        "mission",
        "research",
        "change",
    ];

    #[test]
    fn parity_matrix_every_slash_command_maps_to_a_cli_verb() {
        for spec in CATALOG {
            if spec.name == "help" {
                continue; // TUI-only discovery affordance, no CLI verb.
            }
            assert!(
                EXPECTED_CLI_VERBS.contains(&spec.name),
                "/{} has no CLI counterpart — a TUI-only command violates INV-3 parity",
                spec.name
            );
        }
    }

    #[test]
    fn parity_matrix_covers_the_full_cli_verb_set() {
        // Parity in the other direction: no CLI verb is missing from the catalog.
        for verb in EXPECTED_CLI_VERBS {
            assert!(
                is_known(verb),
                "CLI verb /{verb} is missing from the TUI catalog"
            );
        }
    }

    #[test]
    fn parity_matrix_crate_depends_on_core_not_the_cli() {
        // Structural INV-2 guard: the manifest links the engine tier it
        // actually needs (here, `cronus-domain` — the TUI uses no adapter
        // functionality), never the CLI.
        let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        assert!(
            manifest.contains("cronus-domain = { workspace = true }"),
            "the TUI must link the domain crate (cronus-domain)"
        );
        assert!(
            !manifest.contains("cronus-cli"),
            "the TUI must not depend on cronus-cli (INV-2 inward dependency)"
        );
    }
}

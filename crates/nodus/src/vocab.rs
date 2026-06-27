//! The language vocabulary as data, kept separate from the logic that consults
//! it. Updating the command set, tones, reserved variables, or transpiler verbs
//! is a data edit here — no logic recompile of the lexer / parser / validator /
//! transpiler, which only call the query methods on [`Schema`]. This realizes
//! the "schema is data, not code" property the runtime relies on so a workflow
//! is always interpreted against a loaded vocabulary contract.
//!
//! This module is the in-tree *builtin* schema; it mirrors the reference
//! vocabulary verbatim so ported workflows lex, validate, and transpile
//! identically. Loading an *external* schema file at run time is a later
//! refinement — [`Schema`]'s query surface is the seam it would back.

/// Version of the builtin vocabulary (tracks the reference schema it mirrors).
pub const BUILTIN_SCHEMA_VERSION: &str = "0.4.6";

/// Known command names (ALL_CAPS). The lexer tags an ALL_CAPS word as a
/// `CommandName` only when it appears here; anything else lexes as a generic
/// identifier. 51 commands total; `RUN` is the macro meta-command that
/// bypasses schema vocabulary checks and is recognized before the validation pass.
pub const KNOWN_COMMANDS: &[&str] = &[
    "FETCH",
    "STORE",
    "LOAD",
    "APPEND",
    "MERGE",
    "ANALYZE",
    "SCORE",
    "COMPARE",
    "GEN",
    "REFINE",
    "TRANSLATE",
    "SUMMARIZE",
    "VALIDATE",
    "ROUTE",
    "ESCALATE",
    "NOTIFY",
    "PUBLISH",
    "QUERY_KB",
    "REMEMBER",
    "RECALL",
    "FORGET",
    "LOG",
    "WAIT",
    "TONE",
    "DEBUG",
    "WRITE",
    "MKDIR",
    "FILE_EXISTS",
    "FILL",
    "PARSE",
    "EXECUTE",
    "SIMULATE",
    "EXTRACT",
    "FILTER",
    "EXECUTE_TEST",
    "READ_FILE",
    "SCAN_DIR",
    "ENV",
    "DATE",
    "COUNTER",
    "GIT",
    "QUERY_GIT",
    "MOVE",
    "COPY",
    "TRANSPILE",
    "HASH",
    "PARSE_MD_HEADER",
    "PARSE_INDEX",
    "VERSION_BUMP",
    "GENERATE_DOC",
    "RUN",
];

/// Tone values accepted by the `+tone=` modifier and the `TONE` command.
pub const VALID_TONES: &[&str] = &[
    "warm",
    "neutral",
    "formal",
    "casual",
    "urgent",
    "empathetic",
    "brand",
];

/// Reserved variable names (with leading `$`) that carry runtime-defined meaning.
pub const RESERVED_VARIABLES: &[&str] = &[
    "$in",
    "$out",
    "$error",
    "$meta",
    "$raw",
    "$draft",
    "$ctx",
    "$user",
    "$session",
    "$log",
    "$flags",
    "$quality",
    "$sentiment",
    "$confidence",
    "$memory",
    "$kb_results",
];

/// Runtime-owned variable names that user pipeline targets must not shadow (NL-8).
/// These are set exclusively by the executor and are not writable by workflow steps.
/// Writable reserved variables (`$out`, `$raw`, `$draft`, `$log`, `$quality`,
/// `$sentiment`, `$confidence`) are intentionally excluded — commands assign to them.
pub const RUNTIME_OWNED_VARIABLES: &[&str] = &[
    "$in",
    "$error",
    "$meta",
    "$ctx",
    "$user",
    "$session",
    "$flags",
    "$memory",
    "$kb_results",
];

/// Compact command → human-form verb (consumed by the transpiler). A command
/// not listed here falls back to title-casing its own name.
pub const TRANSPILER_VERB_MAP: &[(&str, &str)] = &[
    ("FETCH", "Fetch"),
    ("ANALYZE", "Analyze"),
    ("GEN", "Generate"),
    ("VALIDATE", "Validate"),
    ("ROUTE", "Route to"),
    ("ESCALATE", "Escalate to"),
    ("LOG", "Log"),
    ("PUBLISH", "Publish"),
    ("NOTIFY", "Notify"),
    ("REFINE", "Refine"),
    ("TONE", "Set tone to"),
    ("REMEMBER", "Store in memory:"),
    ("RECALL", "Load from memory:"),
    ("QUERY_KB", "Search knowledge base for"),
    ("RUN", "Run macro"),
];

/// Runtime error codes surfaced in the structured result's `errors` list.
pub mod error_code {
    /// A `!!` absolute rule was violated.
    pub const RULE_VIOLATION: &str = "NODUS:RULE_VIOLATION";
    /// A workflow file failed to parse.
    pub const PARSE_ERROR: &str = "NODUS:PARSE_ERROR";
    /// A `~UNTIL` loop hit its `MAX:n` limit.
    pub const MAX_REACHED: &str = "NODUS:MAX_REACHED";
    /// A step failed at run time.
    pub const EXECUTION_FAILED: &str = "NODUS:EXECUTION_FAILED";
    /// A variable was referenced before assignment.
    pub const UNDEFINED_VAR: &str = "NODUS:UNDEFINED_VAR";
    /// A `ROUTE(wf:name)` target does not exist.
    pub const ROUTE_NOT_FOUND: &str = "NODUS:ROUTE_NOT_FOUND";
    /// Two `!!` rules contradict each other.
    pub const RULE_CONFLICT: &str = "NODUS:RULE_CONFLICT";
    /// The schema version does not match the workflow.
    pub const SCHEMA_MISMATCH: &str = "NODUS:SCHEMA_MISMATCH";
    /// The workflow ran without a loaded schema.
    pub const NO_SCHEMA: &str = "NODUS:NO_SCHEMA";
    /// No `@ON:` trigger matched the current input.
    pub const NO_TRIGGER: &str = "NODUS:NO_TRIGGER";
    /// A step error reached no `@err:` handler.
    pub const UNHANDLED_ERROR: &str = "NODUS:UNHANDLED_ERROR";
    /// A workflow's capability manifest is unsatisfiable by the active host (LP-8).
    pub const CAPABILITY_UNMET: &str = "NODUS:CAPABILITY_UNMET";
}

/// Is `name` a known command?
pub fn is_known_command(name: &str) -> bool {
    KNOWN_COMMANDS.contains(&name)
}

/// The loaded vocabulary contract a workflow is interpreted against.
///
/// In this foundation it is the builtin schema; the query methods are the
/// stable seam the parser and validator depend on, independent of where the
/// vocabulary is sourced from.
#[derive(Debug, Clone)]
pub struct Schema {
    version: &'static str,
    host_commands: Vec<String>,
    host_reserved: Vec<String>,
}

impl Schema {
    /// The in-tree builtin vocabulary.
    pub fn builtin() -> Self {
        Schema {
            version: BUILTIN_SCHEMA_VERSION,
            host_commands: Vec::new(),
            host_reserved: Vec::new(),
        }
    }

    /// Build an extended schema merging the builtin baseline with host additions.
    ///
    /// Host commands that collide with `KNOWN_COMMANDS` are silently deduplicated.
    /// Host reserved variables that collide with `RESERVED_VARIABLES` are silently
    /// deduplicated. This preserves LP-4: builtin constants are never mutated.
    pub fn with_provider(provider: &dyn crate::portability::SchemaProvider) -> Self {
        let host_commands = provider
            .host_commands()
            .iter()
            .filter(|&&cmd| !is_known_command(cmd))
            .map(|&cmd| cmd.to_string())
            .collect();

        let host_reserved = provider
            .host_reserved_variables()
            .iter()
            .filter(|&&var| !RESERVED_VARIABLES.contains(&var))
            .map(|&var| var.to_string())
            .collect();

        Schema {
            version: BUILTIN_SCHEMA_VERSION,
            host_commands,
            host_reserved,
        }
    }

    /// The schema version this vocabulary declares.
    pub fn version(&self) -> &str {
        self.version
    }

    /// All known command names.
    pub fn commands(&self) -> &'static [&'static str] {
        KNOWN_COMMANDS
    }

    /// All host-added command names (those not in the builtin `KNOWN_COMMANDS`).
    pub fn host_commands(&self) -> &[String] {
        &self.host_commands
    }

    /// Is `name` a known command (builtin or host-added)?
    pub fn is_command(&self, name: &str) -> bool {
        is_known_command(name) || self.host_commands.iter().any(|c| c == name)
    }

    /// Is `name` a host-added command (i.e., not in the builtin `KNOWN_COMMANDS`)?
    pub fn is_host_command(&self, name: &str) -> bool {
        self.host_commands.iter().any(|c| c == name)
    }

    /// Is `name` (with leading `$`) a reserved variable (builtin or host-added)?
    pub fn is_reserved_var(&self, name: &str) -> bool {
        RESERVED_VARIABLES.contains(&name) || self.host_reserved.iter().any(|v| v == name)
    }

    /// Is `name` (with leading `$`) a runtime-owned variable that user steps must not shadow?
    pub fn is_runtime_owned(&self, name: &str) -> bool {
        RUNTIME_OWNED_VARIABLES.contains(&name)
    }

    /// Is `tone` a valid tone value?
    pub fn is_valid_tone(&self, tone: &str) -> bool {
        VALID_TONES.contains(&tone)
    }

    /// Human-form verb for a command, if one is mapped.
    pub fn transpiler_verb(&self, command: &str) -> Option<&'static str> {
        TRANSPILER_VERB_MAP
            .iter()
            .find(|(k, _)| *k == command)
            .map(|(_, v)| *v)
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_schema_lists_known_commands() {
        let schema = Schema::builtin();
        assert!(schema.is_command("GEN"));
        assert!(schema.is_command("VALIDATE"));
        assert!(!schema.is_command("NOT_A_COMMAND"));
        assert_eq!(schema.commands().len(), KNOWN_COMMANDS.len());
    }

    #[test]
    fn builtin_schema_knows_reserved_vars_and_tones() {
        let schema = Schema::builtin();
        assert!(schema.is_reserved_var("$draft"));
        assert!(!schema.is_reserved_var("$nope"));
        assert!(schema.is_valid_tone("warm"));
        assert!(!schema.is_valid_tone("sarcastic"));
    }

    #[test]
    fn transpiler_verb_maps_known_and_misses_unknown() {
        let schema = Schema::builtin();
        assert_eq!(schema.transpiler_verb("GEN"), Some("Generate"));
        assert_eq!(schema.transpiler_verb("ROUTE"), Some("Route to"));
        assert_eq!(schema.transpiler_verb("FETCH_ALL"), None);
    }

    #[test]
    fn version_is_reported() {
        assert_eq!(Schema::builtin().version(), BUILTIN_SCHEMA_VERSION);
    }

    #[test]
    fn run_is_known_command() {
        let schema = Schema::builtin();
        assert!(schema.is_command("RUN"), "RUN must be a known command");
        assert_eq!(
            BUILTIN_SCHEMA_VERSION, "0.4.6",
            "version bump must accompany RUN addition"
        );
    }

    #[test]
    fn runtime_owned_is_subset_of_reserved() {
        for var in RUNTIME_OWNED_VARIABLES {
            assert!(
                RESERVED_VARIABLES.contains(var),
                "{var} in RUNTIME_OWNED_VARIABLES but missing from RESERVED_VARIABLES"
            );
        }
    }

    #[test]
    fn runtime_owned_excludes_writable_reserved() {
        let writable = [
            "$out",
            "$raw",
            "$draft",
            "$log",
            "$quality",
            "$sentiment",
            "$confidence",
        ];
        for var in writable {
            assert!(
                !RUNTIME_OWNED_VARIABLES.contains(&var),
                "{var} must not be in RUNTIME_OWNED_VARIABLES (it is user-writable)"
            );
        }
    }

    #[test]
    fn with_provider_extends_schema() {
        use crate::portability::SchemaProvider;

        struct TestProvider;
        impl SchemaProvider for TestProvider {
            fn host_commands(&self) -> &[&str] {
                &["MY_CMD"]
            }
            fn host_reserved_variables(&self) -> &[&str] {
                &["$myvar"]
            }
        }

        let schema = Schema::with_provider(&TestProvider);
        assert!(
            schema.is_command("MY_CMD"),
            "host command must be recognized"
        );
        assert!(
            schema.is_host_command("MY_CMD"),
            "MY_CMD must be flagged as host command"
        );
        assert!(
            schema.is_command("GEN"),
            "builtin command must remain recognized"
        );
        assert!(
            !schema.is_host_command("GEN"),
            "builtin command must not be flagged as host"
        );
    }

    #[test]
    fn with_provider_deduplicates_builtins() {
        use crate::portability::SchemaProvider;

        struct CollidingProvider;
        impl SchemaProvider for CollidingProvider {
            fn host_commands(&self) -> &[&str] {
                &["GEN"]
            }
            fn host_reserved_variables(&self) -> &[&str] {
                &[]
            }
        }

        let schema = Schema::with_provider(&CollidingProvider);
        assert!(schema.is_command("GEN"), "GEN must remain recognized");
        assert!(
            !schema.is_host_command("GEN"),
            "GEN must not be treated as host-added"
        );
        assert!(
            schema.host_commands().is_empty(),
            "deduplicated GEN must not appear in host list"
        );
    }
}

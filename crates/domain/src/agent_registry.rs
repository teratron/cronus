//! Agent registry — runtime catalog of agent definitions.
//!
//! Built-in agents loaded first; user config applies on top (highest precedence).
//! `model_ref` resolution falls back to run-default for unknown group names.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Primary,
    SubAgent,
    All,
}

impl AgentMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentMode::Primary => "primary",
            AgentMode::SubAgent => "subagent",
            AgentMode::All => "all",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Ruleset {
    pub entries: HashMap<String, String>,
}

impl Ruleset {
    /// Last-key-wins merge: override entries take precedence.
    pub fn merge(base: &Ruleset, override_: &Ruleset) -> Ruleset {
        let mut merged = base.entries.clone();
        for (k, v) in &override_.entries {
            merged.insert(k.clone(), v.clone());
        }
        Ruleset { entries: merged }
    }
}

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub description: Option<String>,
    pub mode: AgentMode,
    pub native: bool,
    pub hidden: bool,
    pub temperature: Option<f32>,
    pub permission: Ruleset,
    pub model_ref: Option<String>,
    pub steps: Option<u32>,
    pub tool_allowlist: Option<Vec<String>>,
    pub disabled: bool,
}

impl AgentDefinition {
    fn builtin(name: &str, description: &str, mode: AgentMode) -> Self {
        AgentDefinition {
            name: name.to_string(),
            description: Some(description.to_string()),
            mode,
            native: true,
            hidden: false,
            temperature: None,
            permission: Ruleset::default(),
            model_ref: Some("default".to_string()),
            steps: None,
            tool_allowlist: None,
            disabled: false,
        }
    }
}

static BUILTIN_NAMES: &[(&str, &str, AgentMode)] = &[
    (
        "work",
        "Default interactive agent; full permission profile",
        AgentMode::Primary,
    ),
    ("code", "Code generation and modification", AgentMode::All),
    (
        "plan",
        "Planning and task decomposition",
        AgentMode::SubAgent,
    ),
    ("edit", "Targeted file editing", AgentMode::SubAgent),
    ("search", "Codebase and web search", AgentMode::SubAgent),
    ("test", "Test authoring and execution", AgentMode::SubAgent),
    (
        "refactor",
        "Code refactoring and cleanup",
        AgentMode::SubAgent,
    ),
];

#[derive(Debug)]
pub enum RegistryError {
    NotFound(String),
    Disabled(String),
    UnknownBuiltin(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::NotFound(n) => write!(f, "agent not found: {n}"),
            RegistryError::Disabled(n) => write!(f, "agent is disabled: {n}"),
            RegistryError::UnknownBuiltin(n) => write!(f, "unknown built-in agent: {n}"),
        }
    }
}

impl std::error::Error for RegistryError {}
pub type Result<T> = std::result::Result<T, RegistryError>;

pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        for (name, desc, mode) in BUILTIN_NAMES {
            agents.insert(
                name.to_string(),
                AgentDefinition::builtin(name, desc, *mode),
            );
        }
        AgentRegistry { agents }
    }

    pub fn resolve(&self, name: &str) -> Result<&AgentDefinition> {
        let def = self
            .agents
            .get(name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        if def.disabled {
            return Err(RegistryError::Disabled(name.to_string()));
        }
        Ok(def)
    }

    /// Apply a user-config override. A `disable: true` entry disables the built-in.
    pub fn apply_user_config(&mut self, name: &str, disable: bool, model_ref: Option<String>) {
        if let Some(def) = self.agents.get_mut(name) {
            if disable {
                def.disabled = true;
            }
            if let Some(mr) = model_ref {
                def.model_ref = Some(mr);
            }
        } else {
            // Custom agent
            let mut def = AgentDefinition::builtin(name, "", AgentMode::All);
            def.native = false;
            def.disabled = disable;
            if let Some(mr) = model_ref {
                def.model_ref = Some(mr);
            }
            self.agents.insert(name.to_string(), def);
        }
    }

    pub fn list_active(&self) -> Vec<&AgentDefinition> {
        self.agents.values().filter(|d| !d.disabled).collect()
    }

    pub fn builtin_count(&self) -> usize {
        self.agents.values().filter(|d| d.native).count()
    }

    /// Insert a custom agent definition into the registry.
    pub fn register_custom(&mut self, def: AgentDefinition) {
        self.agents.insert(def.name.clone(), def);
    }

    /// Generate a stub AgentDefinition from a description (seam; real LLM wiring deferred).
    pub fn generate_from_description(name: &str, description: &str) -> AgentDefinition {
        AgentDefinition {
            name: name.to_string(),
            description: Some(description.to_string()),
            mode: AgentMode::All,
            native: false,
            hidden: false,
            temperature: None,
            permission: Ruleset::default(),
            model_ref: Some("default".to_string()),
            steps: Some(50),
            tool_allowlist: None,
            disabled: false,
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

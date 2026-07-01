//! Extension registry — skills, MCP servers, and plugins.
//!
//! Foundation: manifest validation, lifecycle state machine, auto-discovery.
//! MCP transport and sandbox enforcement wiring is deferred.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionKind {
    Skill,
    McpServer,
    Plugin,
}

impl ExtensionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtensionKind::Skill => "skill",
            ExtensionKind::McpServer => "mcp-server",
            ExtensionKind::Plugin => "plugin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionSource {
    Preset,
    Custom,
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionState {
    Discovered,
    Permitted,
    Active,
    Inactive,
}

impl ExtensionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtensionState::Discovered => "discovered",
            ExtensionState::Permitted => "permitted",
            ExtensionState::Active => "active",
            ExtensionState::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionPermissions {
    pub fs: Vec<String>,
    pub network: Vec<String>,
    pub secrets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExtensionManifest {
    pub id: String,
    pub kind: ExtensionKind,
    pub name: String,
    pub version: String,
    pub source: ExtensionSource,
    pub capabilities: Vec<String>,
    pub permissions: ExtensionPermissions,
}

#[derive(Debug)]
pub enum RegistryError {
    InvalidManifest(String),
    NotFound(String),
    InvalidTransition {
        from: ExtensionState,
        to: ExtensionState,
    },
    InactiveExtension(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::InvalidManifest(s) => write!(f, "invalid manifest: {s}"),
            RegistryError::NotFound(id) => write!(f, "extension not found: {id}"),
            RegistryError::InvalidTransition { from, to } => {
                write!(f, "invalid transition: {:?} → {:?}", from, to)
            }
            RegistryError::InactiveExtension(id) => write!(f, "extension not active: {id}"),
        }
    }
}

impl std::error::Error for RegistryError {}
pub type Result<T> = std::result::Result<T, RegistryError>;

pub fn validate_manifest(manifest: &ExtensionManifest) -> Result<()> {
    if manifest.id.is_empty() {
        return Err(RegistryError::InvalidManifest("id is required".to_string()));
    }
    if manifest.name.is_empty() {
        return Err(RegistryError::InvalidManifest(
            "name is required".to_string(),
        ));
    }
    if manifest.version.is_empty() {
        return Err(RegistryError::InvalidManifest(
            "version is required".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug)]
pub struct ExtensionRegistry {
    entries: HashMap<String, (ExtensionManifest, ExtensionState)>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        ExtensionRegistry {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, manifest: ExtensionManifest) -> Result<()> {
        validate_manifest(&manifest)?;
        self.entries
            .insert(manifest.id.clone(), (manifest, ExtensionState::Discovered));
        Ok(())
    }

    pub fn state(&self, id: &str) -> Option<ExtensionState> {
        self.entries.get(id).map(|(_, s)| *s)
    }

    pub fn transition(&mut self, id: &str, to: ExtensionState) -> Result<()> {
        let entry = self
            .entries
            .get_mut(id)
            .ok_or_else(|| RegistryError::NotFound(id.to_string()))?;
        let from = entry.1;
        let valid = matches!(
            (from, to),
            (ExtensionState::Discovered, ExtensionState::Permitted)
                | (ExtensionState::Permitted, ExtensionState::Active)
                | (ExtensionState::Active, ExtensionState::Inactive)
                | (ExtensionState::Inactive, ExtensionState::Active)
        );
        if !valid {
            return Err(RegistryError::InvalidTransition { from, to });
        }
        entry.1 = to;
        Ok(())
    }

    pub fn require_active(&self, id: &str) -> Result<&ExtensionManifest> {
        let (manifest, state) = self
            .entries
            .get(id)
            .ok_or_else(|| RegistryError::NotFound(id.to_string()))?;
        if *state != ExtensionState::Active {
            return Err(RegistryError::InactiveExtension(id.to_string()));
        }
        Ok(manifest)
    }

    pub fn list(&self) -> Vec<(&ExtensionManifest, ExtensionState)> {
        self.entries.values().map(|(m, s)| (m, *s)).collect()
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

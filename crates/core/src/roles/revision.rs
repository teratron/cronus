//! Config revision tracking for hired role instances.

use std::path::Path;
use super::Result;

#[derive(Debug, Clone)]
pub struct AgentConfigRevision {
    pub id: u32,
    pub agent_id: String,
    pub revision_number: u32,
    pub before_config: String,
    pub after_config: String,
    pub changed_keys: Vec<String>,
    pub source: RevisionSource,
    pub rolled_back_from: Option<u32>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionSource {
    Patch,
    Rollback,
}

impl RevisionSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            RevisionSource::Patch => "patch",
            RevisionSource::Rollback => "rollback",
        }
    }
}

impl AgentConfigRevision {
    pub fn new_patch(
        agent_id: &str,
        revision_number: u32,
        before_config: String,
        after_config: String,
        changed_keys: Vec<String>,
        created_at: u64,
    ) -> Self {
        AgentConfigRevision {
            id: revision_number,
            agent_id: agent_id.to_string(),
            revision_number,
            before_config,
            after_config,
            changed_keys,
            source: RevisionSource::Patch,
            rolled_back_from: None,
            created_at,
        }
    }

    /// Save this revision as an append-only JSON file.
    pub fn save_to(&self, revisions_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(revisions_dir)?;
        let filename = format!("{:04}.json", self.revision_number);
        let content = format!(
            "{{\"revision_number\":{},\"source\":\"{}\",\"changed_keys\":{:?},\"created_at\":{}}}",
            self.revision_number,
            self.source.as_str(),
            self.changed_keys,
            self.created_at,
        );
        std::fs::write(revisions_dir.join(filename), content)?;
        Ok(())
    }

    /// List all revision numbers for an agent's revision directory.
    pub fn list_revision_numbers(revisions_dir: &Path) -> Result<Vec<u32>> {
        if !revisions_dir.exists() {
            return Ok(Vec::new());
        }
        let mut numbers = Vec::new();
        for entry in std::fs::read_dir(revisions_dir)?.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if let Ok(n) = s.trim_end_matches(".json").parse::<u32>() {
                numbers.push(n);
            }
        }
        numbers.sort_unstable();
        Ok(numbers)
    }

    /// Create a rollback revision from a prior revision number.
    pub fn rollback_from(
        agent_id: &str,
        from_revision: u32,
        before_config: String,
        rolled_back_config: String,
        new_revision_number: u32,
        created_at: u64,
    ) -> Self {
        AgentConfigRevision {
            id: new_revision_number,
            agent_id: agent_id.to_string(),
            revision_number: new_revision_number,
            before_config,
            after_config: rolled_back_config,
            changed_keys: vec!["*".to_string()],
            source: RevisionSource::Rollback,
            rolled_back_from: Some(from_revision),
            created_at,
        }
    }
}

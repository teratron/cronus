//! Hired role instances — mutable copies under the state tier.

use std::path::Path;
use super::{AdapterLevel, ContextDelivery, Result, RoleCategory};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HiredFrom {
    Preset(String),
    DerivedFrom(String),
    Custom,
}

impl HiredFrom {
    pub fn as_str(&self) -> String {
        match self {
            HiredFrom::Preset(id) => id.clone(),
            HiredFrom::DerivedFrom(id) => id.clone(),
            HiredFrom::Custom => "custom".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HiredInstance {
    pub id: String,
    pub preset_id: String,
    pub hired_from: HiredFrom,
    pub reports_to: Option<String>,
    pub display_name: String,
    pub category: RoleCategory,
    pub adapter_level: AdapterLevel,
    pub context_delivery: ContextDelivery,
}

impl HiredInstance {
    /// Serialize and save the instance config to disk.
    pub fn save_to(&self, dir: &Path) -> Result<()> {
        let config = format!(
            "{{\"id\":\"{}\",\"hired_from\":\"{}\",\"display_name\":\"{}\",\"category\":\"{}\"}}",
            self.id,
            self.hired_from.as_str(),
            self.display_name,
            self.category.as_str(),
        );
        std::fs::write(dir.join("config.json"), config)?;
        Ok(())
    }

    /// Load an instance from a directory.
    pub fn load_from(dir: &Path) -> Result<Option<Self>> {
        let config_path = dir.join("config.json");
        if !config_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&config_path)?;
        // Minimal JSON parse: extract "id" and "hired_from" fields
        let id = extract_json_str(&content, "id").unwrap_or_default();
        let hired_from_str = extract_json_str(&content, "hired_from").unwrap_or_default();
        let display_name = extract_json_str(&content, "display_name").unwrap_or_default();

        let hired_from = if hired_from_str == "custom" {
            HiredFrom::Custom
        } else {
            HiredFrom::Preset(hired_from_str.clone())
        };

        Ok(Some(HiredInstance {
            id,
            preset_id: hired_from_str,
            hired_from,
            reports_to: None,
            display_name,
            category: RoleCategory::Custom,
            adapter_level: AdapterLevel::Callable,
            context_delivery: ContextDelivery::ThinPing,
        }))
    }
}

/// Minimal JSON string value extractor for the pattern `"key":"value"`.
fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

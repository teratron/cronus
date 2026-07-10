//! Session data migration — v1 → v2 → v3.
//!
//! Session format evolves between Cronus versions. Migrations are applied
//! eagerly on load; each is idempotent.

pub const CURRENT_SESSION_VERSION: u32 = 3;

/// Minimal session data representation for migration purposes.
#[derive(Debug, Clone)]
pub struct SessionData {
    pub version: u32,
    pub id: Option<String>,
    pub parent_id: Option<String>,
    /// Raw message kind strings (pre-rename).
    pub message_kinds: Vec<String>,
}

impl SessionData {
    pub fn new(version: u32) -> Self {
        SessionData {
            version,
            id: None,
            parent_id: None,
            message_kinds: Vec::new(),
        }
    }

    /// Apply all pending migrations to reach `CURRENT_SESSION_VERSION`.
    pub fn migrate(mut self) -> Self {
        if self.version < 2 {
            self = migrate_v1_to_v2(self);
        }
        if self.version < 3 {
            self = migrate_v2_to_v3(self);
        }
        self
    }
}

/// v1 → v2: add id/parentId tree structure.
fn migrate_v1_to_v2(mut data: SessionData) -> SessionData {
    if data.id.is_none() {
        data.id = Some(format!("session-migrated-{}", data.version));
    }
    data.version = 2;
    data
}

/// v2 → v3: rename `hookMessage` → `custom` in message kinds.
fn migrate_v2_to_v3(mut data: SessionData) -> SessionData {
    for kind in &mut data.message_kinds {
        if kind == "hookMessage" {
            *kind = "custom".to_owned();
        }
    }
    data.version = 3;
    data
}

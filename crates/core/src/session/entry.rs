//! Session entry taxonomy.

/// The type of a session history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEntry {
    Message,
    CustomMessage,
    Compaction,
    BranchSummary,
    ThinkingLevelChange,
    ModelChange,
    Custom,
    Label,
    SessionInfo,
}

impl SessionEntry {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionEntry::Message => "message",
            SessionEntry::CustomMessage => "customMessage",
            SessionEntry::Compaction => "compaction",
            SessionEntry::BranchSummary => "branchSummary",
            SessionEntry::ThinkingLevelChange => "thinkingLevelChange",
            SessionEntry::ModelChange => "modelChange",
            SessionEntry::Custom => "custom",
            SessionEntry::Label => "label",
            SessionEntry::SessionInfo => "sessionInfo",
        }
    }
}

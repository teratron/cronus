//! Kanban board — single per-workspace board with file-backed cards.
//!
//! Fixed state set: triage → todo → ready → running → blocked → done.
//! Each transition appends a history entry (KAN-7). Done cards are auto-archived
//! to `<ws>/kanban/archive/` without deletion (KAN-4).

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum KanbanError {
    InvalidTransition { from: CardState, to: CardState },
    BlockedRequiresReason,
    CardNotFound(String),
    Io(std::io::Error),
}

impl fmt::Display for KanbanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KanbanError::InvalidTransition { from, to } =>
                write!(f, "invalid transition: {from:?} → {to:?}"),
            KanbanError::BlockedRequiresReason =>
                write!(f, "blocked state requires a reason"),
            KanbanError::CardNotFound(id) => write!(f, "card not found: {id}"),
            KanbanError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for KanbanError {}
impl From<std::io::Error> for KanbanError {
    fn from(e: std::io::Error) -> Self {
        KanbanError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, KanbanError>;

// ── Card state ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    Triage,
    Todo,
    Ready,
    Running,
    Blocked,
    Done,
}

impl CardState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardState::Triage => "triage",
            CardState::Todo => "todo",
            CardState::Ready => "ready",
            CardState::Running => "running",
            CardState::Blocked => "blocked",
            CardState::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "triage" => Some(CardState::Triage),
            "todo" => Some(CardState::Todo),
            "ready" => Some(CardState::Ready),
            "running" => Some(CardState::Running),
            "blocked" => Some(CardState::Blocked),
            "done" => Some(CardState::Done),
            _ => None,
        }
    }

    /// Returns true when the transition from self → to is valid.
    pub fn can_transition_to(self, to: CardState) -> bool {
        matches!(
            (self, to),
            (CardState::Triage, CardState::Todo)
                | (CardState::Todo, CardState::Ready)
                | (CardState::Ready, CardState::Running)
                | (CardState::Running, CardState::Done)
                | (CardState::Running, CardState::Blocked)
                | (CardState::Blocked, CardState::Ready)
                | (CardState::Blocked, CardState::Todo)
                | (CardState::Todo, CardState::Triage) // allow triage-back
        )
    }
}

// ── Card types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransitionRecord {
    pub from: CardState,
    pub to: CardState,
    pub actor: String,
    pub at: u64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: String,
    pub task_ref: String,
    pub state: CardState,
    pub reason: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<Priority>,
    pub skills: Vec<String>,
    pub max_retries: u8,
    pub history: Vec<TransitionRecord>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Card {
    pub fn new(id: &str, task_ref: &str, created_at: u64) -> Self {
        Card {
            id: id.to_string(),
            task_ref: task_ref.to_string(),
            state: CardState::Triage,
            reason: None,
            assignee: None,
            priority: None,
            skills: Vec::new(),
            max_retries: 0,
            history: Vec::new(),
            created_at,
            updated_at: created_at,
        }
    }

    /// Serialize the card to a simple JSON string (including full history).
    fn to_json(&self) -> String {
        let history_items: Vec<String> = self.history.iter().map(|r| {
            format!(
                "{{\"from\":\"{}\",\"to\":\"{}\",\"actor\":\"{}\",\"at\":{}}}",
                r.from.as_str(), r.to.as_str(), r.actor, r.at
            )
        }).collect();
        format!(
            "{{\"id\":\"{}\",\"task_ref\":\"{}\",\"state\":\"{}\",\"created_at\":{},\"updated_at\":{},\"history\":[{}]}}",
            self.id, self.task_ref, self.state.as_str(), self.created_at, self.updated_at,
            history_items.join(",")
        )
    }

    /// Minimal deserialize from JSON — extracts only the fields we need.
    fn from_json(json: &str) -> Option<Self> {
        let id = extract_json_str(json, "id")?;
        let task_ref = extract_json_str(json, "task_ref").unwrap_or_default();
        let state_str = extract_json_str(json, "state").unwrap_or_else(|| "triage".to_string());
        let state = CardState::parse(&state_str).unwrap_or(CardState::Triage);
        let created_at = extract_json_u64(json, "created_at").unwrap_or(0);
        let updated_at = extract_json_u64(json, "updated_at").unwrap_or(created_at);
        let history = extract_json_history(json);
        Some(Card {
            id,
            task_ref,
            state,
            reason: None,
            assignee: None,
            priority: None,
            skills: Vec::new(),
            max_retries: 0,
            history,
            created_at,
            updated_at,
        })
    }
}

// ── Board ─────────────────────────────────────────────────────────────────────

/// Single per-workspace kanban board.
pub struct Board {
    kanban_dir: PathBuf,
}

impl Board {
    pub fn new(kanban_dir: PathBuf) -> Self {
        Board { kanban_dir }
    }

    pub fn open(ws_path: &Path) -> Self {
        Board { kanban_dir: ws_path.join("kanban") }
    }

    fn cards_dir(&self) -> PathBuf {
        self.kanban_dir.join("cards")
    }

    fn archive_dir(&self) -> PathBuf {
        self.kanban_dir.join("archive")
    }

    fn events_dir(&self) -> PathBuf {
        self.kanban_dir.join("events")
    }

    fn card_path(&self, card_id: &str) -> PathBuf {
        self.cards_dir().join(format!("{card_id}.json"))
    }

    fn archive_path(&self, card_id: &str) -> PathBuf {
        self.archive_dir().join(format!("{card_id}.json"))
    }

    /// Ensure board directories exist.
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(self.cards_dir())?;
        fs::create_dir_all(self.archive_dir())?;
        fs::create_dir_all(self.events_dir())?;
        fs::create_dir_all(self.kanban_dir.join("runs"))?;
        fs::create_dir_all(self.kanban_dir.join("comments"))?;
        // Write board.json meta
        let board_json = r#"{"state_set":["triage","todo","ready","running","blocked","done"]}"#;
        fs::write(self.kanban_dir.join("board.json"), board_json)?;
        Ok(())
    }

    /// Add a new card (starts in Triage).
    pub fn add_card(&self, id: &str, task_ref: &str, now: u64) -> Result<Card> {
        fs::create_dir_all(self.cards_dir())?;
        let card = Card::new(id, task_ref, now);
        self.save_card(&card)?;
        Ok(card)
    }

    fn save_card(&self, card: &Card) -> Result<()> {
        fs::create_dir_all(self.cards_dir())?;
        let json = card.to_json();
        fs::write(self.card_path(&card.id), json)?;
        Ok(())
    }

    /// Load a card by ID.
    pub fn get_card(&self, id: &str) -> Result<Option<Card>> {
        let path = self.card_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&path)?;
        Ok(Card::from_json(&json))
    }

    /// Move a card to a new state, recording the transition in its history.
    pub fn move_card(
        &self,
        card_id: &str,
        to: CardState,
        actor: &str,
        reason: Option<String>,
        now: u64,
    ) -> Result<Card> {
        let mut card = self
            .get_card(card_id)?
            .ok_or_else(|| KanbanError::CardNotFound(card_id.to_string()))?;

        if to == CardState::Blocked && reason.is_none() {
            return Err(KanbanError::BlockedRequiresReason);
        }
        if !card.state.can_transition_to(to) {
            return Err(KanbanError::InvalidTransition { from: card.state, to });
        }

        let record = TransitionRecord {
            from: card.state,
            to,
            actor: actor.to_string(),
            at: now,
            reason: reason.clone(),
        };
        card.history.push(record);
        card.state = to;
        card.reason = reason;
        card.updated_at = now;

        self.save_card(&card)?;
        self.append_event(card_id, &format!("{{\"type\":\"transition\",\"to\":\"{}\",\"at\":{now}}}", to.as_str()))?;
        Ok(card)
    }

    /// Append a line to the card's event log.
    fn append_event(&self, card_id: &str, event: &str) -> Result<()> {
        use std::io::Write;
        let path = self.events_dir().join(format!("{card_id}.jsonl"));
        let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{event}")?;
        Ok(())
    }

    /// List all active cards (not archived).
    pub fn list_cards(&self) -> Result<Vec<Card>> {
        if !self.cards_dir().exists() {
            return Ok(Vec::new());
        }
        let mut cards = Vec::new();
        for entry in fs::read_dir(self.cards_dir())?.flatten() {
            if entry.path().extension().is_some_and(|e| e == "json")
                && let Ok(json) = fs::read_to_string(entry.path())
                && let Some(card) = Card::from_json(&json)
            {
                cards.push(card);
            }
        }
        Ok(cards)
    }

    /// Archive all done cards by moving them to archive/ (non-destructive).
    pub fn archive_done_cards(&self) -> Result<usize> {
        fs::create_dir_all(self.archive_dir())?;
        let cards = self.list_cards()?;
        let mut count = 0;
        for card in cards {
            if card.state == CardState::Done {
                let src = self.card_path(&card.id);
                let dst = self.archive_path(&card.id);
                fs::rename(&src, &dst)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Load an archived card (still readable after archival, KAN-4).
    pub fn get_archived_card(&self, id: &str) -> Result<Option<Card>> {
        let path = self.archive_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&path)?;
        Ok(Card::from_json(&json))
    }

    pub fn board_json_exists(&self) -> bool {
        self.kanban_dir.join("board.json").exists()
    }
}

// ── Minimal JSON helpers ─────────────────────────────────────────────────────

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_json_u64(json: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{key}\":");
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Extract the `"history"` array from a card JSON, rebuilding TransitionRecords.
fn extract_json_history(json: &str) -> Vec<TransitionRecord> {
    let marker = "\"history\":[";
    let Some(array_start) = json.find(marker) else { return Vec::new() };
    let inner_start = array_start + marker.len();
    let rest = &json[inner_start..];
    // find the matching closing bracket
    let Some(array_end) = rest.find(']') else { return Vec::new() };
    let array = &rest[..array_end];

    let mut records = Vec::new();
    // each entry is delimited by `{...}`
    let mut pos = 0;
    while let Some(obj_start) = array[pos..].find('{') {
        let abs_start = pos + obj_start;
        let Some(obj_end) = array[abs_start..].find('}') else { break };
        let obj = &array[abs_start..abs_start + obj_end + 1];

        let from_str = extract_json_str(obj, "from").unwrap_or_default();
        let to_str = extract_json_str(obj, "to").unwrap_or_default();
        let actor = extract_json_str(obj, "actor").unwrap_or_default();
        let at = extract_json_u64(obj, "at").unwrap_or(0);

        if let (Some(from), Some(to)) = (CardState::parse(&from_str), CardState::parse(&to_str)) {
            records.push(TransitionRecord { from, to, actor, at, reason: None });
        }
        pos = abs_start + obj_end + 1;
    }
    records
}

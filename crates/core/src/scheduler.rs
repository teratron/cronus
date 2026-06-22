use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fmt, io};

use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;

use crate::tool_security::{SkillScanner, now_ms};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SchedulerError {
    Io(io::Error),
    InvalidCronExpression(String),
    NotFound(String),
    NotEnabled(String),
    CronPromptInjectionBlocked { schedule_id: String, detail: String },
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::Io(e) => write!(f, "I/O error: {e}"),
            SchedulerError::InvalidCronExpression(e) => {
                write!(f, "invalid cron expression: {e}")
            }
            SchedulerError::NotFound(id) => write!(f, "schedule not found: {id}"),
            SchedulerError::NotEnabled(id) => write!(f, "schedule is disabled: {id}"),
            SchedulerError::CronPromptInjectionBlocked {
                schedule_id,
                detail,
            } => {
                write!(
                    f,
                    "prompt injection detected in schedule '{schedule_id}': {detail}"
                )
            }
        }
    }
}

impl From<io::Error> for SchedulerError {
    fn from(e: io::Error) -> Self {
        SchedulerError::Io(e)
    }
}

// ── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleKind {
    Recurring,
    Oneshot,
}

impl ScheduleKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScheduleKind::Recurring => "recurring",
            ScheduleKind::Oneshot => "oneshot",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "recurring" => Some(ScheduleKind::Recurring),
            "oneshot" => Some(ScheduleKind::Oneshot),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleAction {
    Heartbeat,
    Routine,
    Reminder,
}

impl ScheduleAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScheduleAction::Heartbeat => "heartbeat",
            ScheduleAction::Routine => "routine",
            ScheduleAction::Reminder => "reminder",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "heartbeat" => Some(ScheduleAction::Heartbeat),
            "routine" => Some(ScheduleAction::Routine),
            "reminder" => Some(ScheduleAction::Reminder),
            _ => None,
        }
    }
}

/// Convenience presets that expand into 6-field cron expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecurrencePreset {
    Weekdays,
    Weekends,
    Daily,
    Days(Vec<u8>),
    Interval(u64),
}

impl RecurrencePreset {
    /// Expand the preset into a 6-field cron expression:
    /// `<sec> <min> <hour> <dom> <month> <dow>`
    pub fn to_cron(&self) -> String {
        match self {
            RecurrencePreset::Weekdays => "0 0 9 * * 1-5".to_string(),
            RecurrencePreset::Weekends => "0 0 9 * * SAT,SUN".to_string(),
            RecurrencePreset::Daily => "0 0 9 * * *".to_string(),
            RecurrencePreset::Days(days) => {
                let list: Vec<String> = days.iter().map(|d| d.to_string()).collect();
                format!("0 0 9 * * {}", list.join(","))
            }
            RecurrencePreset::Interval(_) => "0 * * * * *".to_string(),
        }
    }
}

// ── Schedule ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub kind: ScheduleKind,
    pub action: ScheduleAction,
    /// The resolved 6-field cron expression used for firing.
    pub cron_expr: String,
    /// Oneshot fire time (Unix millis). Present when `kind == Oneshot`.
    pub at_ms: Option<u64>,
    /// IANA timezone name (e.g. `"UTC"`, `"America/New_York"`).
    pub timezone: String,
    pub enabled: bool,
    pub delete_after_fire: bool,
    pub created_at: u64,
    /// Cached next-fire Unix millis (updated on schedule write).
    pub next_fire_at: Option<u64>,
}

impl Schedule {
    /// Build a recurring schedule from a preset.
    pub fn recurring(
        id: impl Into<String>,
        name: impl Into<String>,
        preset: &RecurrencePreset,
        action: ScheduleAction,
    ) -> Self {
        let created_at = now_ms();
        let cron_expr = preset.to_cron();
        let next_fire_at = next_fire_utc(&cron_expr).ok();
        Schedule {
            id: id.into(),
            name: name.into(),
            kind: ScheduleKind::Recurring,
            action,
            cron_expr,
            at_ms: None,
            timezone: "UTC".to_string(),
            enabled: true,
            delete_after_fire: false,
            created_at,
            next_fire_at,
        }
    }

    /// Build a oneshot schedule.
    pub fn oneshot(
        id: impl Into<String>,
        name: impl Into<String>,
        at_ms: u64,
        action: ScheduleAction,
    ) -> Self {
        let created_at = now_ms();
        Schedule {
            id: id.into(),
            name: name.into(),
            kind: ScheduleKind::Oneshot,
            action,
            cron_expr: String::new(),
            at_ms: Some(at_ms),
            timezone: "UTC".to_string(),
            enabled: true,
            delete_after_fire: true,
            created_at,
            next_fire_at: Some(at_ms),
        }
    }

    fn to_json(&self) -> String {
        let at_ms = match self.at_ms {
            Some(v) => format!("{v}"),
            None => "null".to_string(),
        };
        let next = match self.next_fire_at {
            Some(v) => format!("{v}"),
            None => "null".to_string(),
        };
        format!(
            "{{\"id\":\"{}\",\"name\":\"{}\",\"kind\":\"{}\",\"action\":\"{}\",\
             \"cron_expr\":\"{}\",\"at_ms\":{},\"timezone\":\"{}\",\
             \"enabled\":{},\"delete_after_fire\":{},\"created_at\":{},\"next_fire_at\":{}}}",
            self.id,
            self.name,
            self.kind.as_str(),
            self.action.as_str(),
            self.cron_expr,
            at_ms,
            self.timezone,
            self.enabled,
            self.delete_after_fire,
            self.created_at,
            next,
        )
    }

    fn from_json(json: &str) -> Option<Self> {
        let id = extract_str(json, "id")?;
        let name = extract_str(json, "name")?;
        let kind = ScheduleKind::parse(&extract_str(json, "kind")?)?;
        let action = ScheduleAction::parse(&extract_str(json, "action")?)?;
        let cron_expr = extract_str(json, "cron_expr").unwrap_or_default();
        let at_ms = extract_u64(json, "at_ms");
        let timezone = extract_str(json, "timezone").unwrap_or_else(|| "UTC".to_string());
        let enabled = extract_bool(json, "enabled").unwrap_or(true);
        let delete_after_fire = extract_bool(json, "delete_after_fire").unwrap_or(false);
        let created_at = extract_u64(json, "created_at").unwrap_or(0);
        let next_fire_at = extract_u64(json, "next_fire_at");
        Some(Schedule {
            id,
            name,
            kind,
            action,
            cron_expr,
            at_ms,
            timezone,
            enabled,
            delete_after_fire,
            created_at,
            next_fire_at,
        })
    }
}

// ── CronSession ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunResult {
    Ok,
    InjectionBlocked,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct RunLogEntry {
    pub schedule_id: String,
    pub fired_at: u64,
    pub action: ScheduleAction,
    pub result: RunResult,
}

#[derive(Debug)]
pub struct CronSession {
    pub session_key: String,
    pub run_log: Vec<RunLogEntry>,
}

// ── Scheduler ─────────────────────────────────────────────────────────────────

pub struct Scheduler {
    storage_dir: PathBuf,
}

impl Scheduler {
    pub fn new(storage_dir: impl Into<PathBuf>) -> Result<Self, SchedulerError> {
        let dir = storage_dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Scheduler { storage_dir: dir })
    }

    pub fn add(&self, schedule: &Schedule) -> Result<(), SchedulerError> {
        let json = schedule.to_json();
        fs::write(self.path_for(&schedule.id), json)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Schedule, SchedulerError> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(SchedulerError::NotFound(id.to_string()));
        }
        let json = fs::read_to_string(&path)?;
        Schedule::from_json(&json).ok_or_else(|| SchedulerError::NotFound(id.to_string()))
    }

    pub fn list(&self) -> Result<Vec<Schedule>, SchedulerError> {
        let mut schedules = Vec::new();
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "json")
                && let Ok(json) = fs::read_to_string(entry.path())
                && let Some(sched) = Schedule::from_json(&json)
            {
                schedules.push(sched);
            }
        }
        Ok(schedules)
    }

    pub fn delete(&self, id: &str) -> Result<(), SchedulerError> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(SchedulerError::NotFound(id.to_string()));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// Fire a schedule with the given assembled prompt.
    ///
    /// Scans the prompt for injection attacks before execution. Removes
    /// oneshot schedules with `delete_after_fire` set after a successful fire.
    pub fn fire(&self, id: &str, prompt: &str) -> Result<CronSession, SchedulerError> {
        let sched = self.get(id)?;
        if !sched.enabled {
            return Err(SchedulerError::NotEnabled(id.to_string()));
        }

        let scan = SkillScanner::scan_content(prompt, "schedule.md");
        if !scan.is_safe {
            let detail = scan
                .findings
                .first()
                .map(|f| format!("{:?}", f.category))
                .unwrap_or_else(|| "unknown".to_string());
            return Err(SchedulerError::CronPromptInjectionBlocked {
                schedule_id: id.to_string(),
                detail,
            });
        }

        let fired_at = now_ms();
        let entry = RunLogEntry {
            schedule_id: id.to_string(),
            fired_at,
            action: sched.action.clone(),
            result: RunResult::Ok,
        };

        if sched.kind == ScheduleKind::Oneshot && sched.delete_after_fire {
            let _ = self.delete(id);
        }

        Ok(CronSession {
            session_key: format!("cron-{id}-{fired_at}"),
            run_log: vec![entry],
        })
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.storage_dir.join(format!("{id}.json"))
    }
}

// ── Cron helpers ──────────────────────────────────────────────────────────────

/// Compute the next fire timestamp (Unix millis, UTC) for a 6-field cron expression.
pub fn next_fire_utc(cron_expr: &str) -> Result<u64, SchedulerError> {
    let sched = CronSchedule::from_str(cron_expr)
        .map_err(|e| SchedulerError::InvalidCronExpression(e.to_string()))?;
    let next: DateTime<Utc> = sched
        .upcoming(Utc)
        .next()
        .ok_or_else(|| SchedulerError::InvalidCronExpression("no upcoming fire time".into()))?;
    Ok(next.timestamp_millis() as u64)
}

// ── JSON helpers ──────────────────────────────────────────────────────────────

fn extract_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_u64(json: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    if rest.starts_with("null") {
        return None;
    }
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn extract_bool(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

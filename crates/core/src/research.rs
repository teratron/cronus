//! Deep Research — iterative Think→Plan→Search→Extract→Synthesize engine.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── Research job ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchStatus {
    Planning,
    Running,
    Complete,
    Partial,
    Cancelled,
}

impl ResearchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResearchStatus::Planning => "planning",
            ResearchStatus::Running => "running",
            ResearchStatus::Complete => "complete",
            ResearchStatus::Partial => "partial",
            ResearchStatus::Cancelled => "cancelled",
        }
    }
}

/// Default maximum research rounds.
pub const DEFAULT_MAX_ROUNDS: u8 = 5;

/// Minimum content length to accept a fetched page.
pub const MIN_CONTENT_CHARS: usize = 200;

/// A running or completed research job.
#[derive(Debug, Clone)]
pub struct ResearchJob {
    pub id: String,
    pub question: String,
    pub status: ResearchStatus,
    pub rounds_completed: u8,
    pub max_rounds: u8,
    pub created_at: u64,
}

impl ResearchJob {
    pub fn new(
        id: impl Into<String>,
        question: impl Into<String>,
        max_rounds: u8,
        created_at: u64,
    ) -> Self {
        ResearchJob {
            id: id.into(),
            question: question.into(),
            status: ResearchStatus::Planning,
            rounds_completed: 0,
            max_rounds,
            created_at,
        }
    }

    pub fn start(&mut self) {
        self.status = ResearchStatus::Running;
    }

    pub fn cancel(&mut self) {
        self.status = ResearchStatus::Cancelled;
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ResearchStatus::Complete | ResearchStatus::Partial | ResearchStatus::Cancelled
        )
    }
}

// ── Research plan ─────────────────────────────────────────────────────────────

/// Structured research plan generated in Phase 1 of the loop.
#[derive(Debug, Clone, Default)]
pub struct ResearchPlan {
    /// 3–6 specific sub-questions to investigate.
    pub sub_questions: Vec<String>,
    pub key_topics: Vec<String>,
    /// One sentence: what a complete answer looks like.
    pub success_criteria: String,
}

impl ResearchPlan {
    pub fn new(success_criteria: impl Into<String>) -> Self {
        ResearchPlan {
            sub_questions: Vec::new(),
            key_topics: Vec::new(),
            success_criteria: success_criteria.into(),
        }
    }
}

// ── Fetched page ──────────────────────────────────────────────────────────────

/// A page retrieved during a research round.
#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub url: String,
    pub title: String,
    pub content: String,
    pub retrieved_at: u64,
}

impl FetchedPage {
    pub fn new(
        url: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        retrieved_at: u64,
    ) -> Self {
        FetchedPage {
            url: url.into(),
            title: title.into(),
            content: content.into(),
            retrieved_at,
        }
    }
}

// ── Content filter ────────────────────────────────────────────────────────────

/// Filter a page for quality. Returns false if the page should be discarded.
pub fn filter_page(page: &FetchedPage, seen_urls: &mut HashSet<String>) -> bool {
    if page.content.len() < MIN_CONTENT_CHARS {
        return false;
    }
    if page.content.is_empty() {
        return false;
    }
    if seen_urls.contains(&page.url) {
        return false;
    }
    seen_urls.insert(page.url.clone());
    true
}

// ── Untrusted content wrapping ────────────────────────────────────────────────

/// Wrap external content as untrusted before injecting into the model context.
///
/// Delegates to `tool_security::untrusted_context_message`; re-exported here
/// for use within the research engine without importing tool_security directly.
pub fn wrap_untrusted(label: &str, content: &str) -> String {
    crate::tool_security::untrusted_context_message(label, content)
}

// ── Date-grounding preamble ───────────────────────────────────────────────────

/// Generate a date-grounding preamble from a Unix millisecond timestamp.
///
/// Prevents the model from using its training-cutoff year when generating
/// search queries.
pub fn date_grounding_preamble(now_ms: u64) -> String {
    let secs = now_ms / 1000;
    let (year, month, day) = epoch_to_ymd(secs);
    let month_name = MONTH_NAMES[(month as usize).saturating_sub(1).min(11)];
    format!(
        "Today's date is {month_name} {day:02}, {year} ({year}-{month:02}-{day:02}).\n\
         When a query needs a year or refers to \"latest\"/\"current\"/\"this year\",\n\
         use {year} or relative wording — never a year inferred from training data.\n"
    )
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

// ── Citation ──────────────────────────────────────────────────────────────────

/// A bibliographic reference in the final report.
#[derive(Debug, Clone)]
pub struct Citation {
    pub url: String,
    pub title: String,
    pub retrieved_at: u64,
}

// ── Research report ───────────────────────────────────────────────────────────

/// Final output of a research job.
#[derive(Debug, Clone)]
pub struct ResearchReport {
    pub question: String,
    pub sub_questions_answered: Vec<String>,
    pub report_body: String,
    pub citations: Vec<Citation>,
    pub rounds_completed: u8,
    pub success_criteria_met: bool,
    /// Non-empty when `success_criteria_met = false`.
    pub partial_reason: Option<String>,
    /// When true, the report is marked as `_protected` in the session.
    pub is_protected: bool,
}

impl ResearchReport {
    pub fn partial(question: impl Into<String>, rounds: u8, reason: impl Into<String>) -> Self {
        ResearchReport {
            question: question.into(),
            sub_questions_answered: Vec::new(),
            report_body: String::new(),
            citations: Vec::new(),
            rounds_completed: rounds,
            success_criteria_met: false,
            partial_reason: Some(reason.into()),
            is_protected: true,
        }
    }

    pub fn complete(
        question: impl Into<String>,
        report_body: impl Into<String>,
        citations: Vec<Citation>,
        rounds: u8,
    ) -> Self {
        ResearchReport {
            question: question.into(),
            sub_questions_answered: Vec::new(),
            report_body: report_body.into(),
            citations,
            rounds_completed: rounds,
            success_criteria_met: true,
            partial_reason: None,
            is_protected: true,
        }
    }
}

// ── Search provider seam ──────────────────────────────────────────────────────

/// Pluggable search backend. The stub returns an empty result list.
/// A real provider registers via the extension registry (`SearchProvider` interface).
pub fn search_stub(_query: &str) -> Vec<FetchedPage> {
    vec![]
}

// ── Research engine ───────────────────────────────────────────────────────────

/// Drive one round of the research loop.
///
/// Returns the pages that passed the content filter (may be empty when the
/// search backend is the stub).
pub fn run_round(
    queries: &[String],
    seen_urls: &mut HashSet<String>,
    now_ms: u64,
) -> Vec<FetchedPage> {
    let mut accepted = Vec::new();
    for query in queries {
        for page in search_stub(query) {
            let _ = now_ms; // retrieved_at would be set by the real fetcher
            if filter_page(&page, seen_urls) {
                accepted.push(page);
            }
        }
    }
    accepted
}

/// Build a partial report when max_rounds is reached without meeting criteria.
pub fn build_partial_report(job: &ResearchJob, plan: &ResearchPlan) -> ResearchReport {
    let reason = format!(
        "max_rounds ({}) reached; success criteria not met: {}",
        job.max_rounds, plan.success_criteria
    );
    ResearchReport::partial(&job.question, job.rounds_completed, reason)
}

// ── Job storage path ──────────────────────────────────────────────────────────

pub fn job_dir(workspace_root: &Path, job_id: &str) -> PathBuf {
    workspace_root.join("research").join(job_id)
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn epoch_to_ymd(secs: u64) -> (u32, u8, u8) {
    let total_days = secs / 86400;
    let mut year = 1970u32;
    let mut days = total_days;
    loop {
        let days_in_year: u64 = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u8;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = (i + 1) as u8;
            break;
        }
        days -= md;
    }
    let day = (days + 1) as u8;
    (year, month, day)
}

fn is_leap(y: u32) -> bool {
    y.is_multiple_of(4) && !y.is_multiple_of(100) || y.is_multiple_of(400)
}

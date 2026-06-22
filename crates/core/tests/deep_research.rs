use cronus::research::{
    DEFAULT_MAX_ROUNDS, FetchedPage, MIN_CONTENT_CHARS, ResearchJob, ResearchPlan, ResearchReport,
    ResearchStatus, build_partial_report, date_grounding_preamble, filter_page, job_dir, run_round,
    search_stub, wrap_untrusted,
};
use std::collections::HashSet;

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn default_max_rounds_is_five() {
    assert_eq!(DEFAULT_MAX_ROUNDS, 5);
}

#[test]
fn min_content_chars_is_two_hundred() {
    assert_eq!(MIN_CONTENT_CHARS, 200);
}

// ── ResearchJob lifecycle ─────────────────────────────────────────────────────

#[test]
fn new_research_job_starts_in_planning() {
    let job = ResearchJob::new("j1", "What is Rust?", DEFAULT_MAX_ROUNDS, 0);
    assert_eq!(job.status, ResearchStatus::Planning);
    assert_eq!(job.rounds_completed, 0);
    assert_eq!(job.max_rounds, DEFAULT_MAX_ROUNDS);
}

#[test]
fn start_transitions_to_running() {
    let mut job = ResearchJob::new("j2", "query", DEFAULT_MAX_ROUNDS, 0);
    job.start();
    assert_eq!(job.status, ResearchStatus::Running);
}

#[test]
fn cancel_produces_cancelled_status() {
    let mut job = ResearchJob::new("j3", "query", DEFAULT_MAX_ROUNDS, 0);
    job.start();
    job.cancel();
    assert_eq!(job.status, ResearchStatus::Cancelled);
    assert!(job.is_terminal());
}

#[test]
fn planning_status_is_not_terminal() {
    let job = ResearchJob::new("j4", "query", DEFAULT_MAX_ROUNDS, 0);
    assert!(!job.is_terminal());
}

// ── Date grounding preamble ───────────────────────────────────────────────────

#[test]
fn date_grounding_preamble_contains_year() {
    // 2024-01-15 00:00:00 UTC = 1705276800000 ms
    let preamble = date_grounding_preamble(1_705_276_800_000);
    assert!(preamble.contains("2024"), "preamble must contain year 2024");
    assert!(
        preamble.contains("January"),
        "preamble must contain month name"
    );
    assert!(
        preamble.contains("2024-01-15"),
        "preamble must contain ISO date"
    );
}

#[test]
fn date_grounding_preamble_mentions_training_data() {
    let preamble = date_grounding_preamble(1_705_276_800_000);
    assert!(
        preamble.contains("training data"),
        "preamble must warn about training-data dates"
    );
}

#[test]
fn date_grounding_preamble_format_multiline() {
    let preamble = date_grounding_preamble(1_705_276_800_000);
    assert!(
        preamble.contains('\n'),
        "preamble should span multiple lines"
    );
}

// ── Content filter ────────────────────────────────────────────────────────────

#[test]
fn filter_page_rejects_short_content() {
    let mut seen = HashSet::new();
    let page = FetchedPage::new("https://example.com", "Title", "too short", 0);
    assert!(!filter_page(&page, &mut seen));
}

#[test]
fn filter_page_accepts_long_content() {
    let mut seen = HashSet::new();
    let content = "x".repeat(MIN_CONTENT_CHARS);
    let page = FetchedPage::new("https://example.com/long", "Title", content, 0);
    assert!(filter_page(&page, &mut seen));
}

#[test]
fn filter_page_deduplicates_same_url() {
    let mut seen = HashSet::new();
    let content = "y".repeat(MIN_CONTENT_CHARS);
    let page = FetchedPage::new("https://example.com/dup", "Title", content.clone(), 0);
    assert!(
        filter_page(&page, &mut seen),
        "first visit should be accepted"
    );
    let page2 = FetchedPage::new("https://example.com/dup", "Title", content, 0);
    assert!(
        !filter_page(&page2, &mut seen),
        "second visit of same URL should be rejected"
    );
}

#[test]
fn filter_page_accepts_different_urls_with_same_content() {
    let mut seen = HashSet::new();
    let content = "z".repeat(MIN_CONTENT_CHARS);
    let p1 = FetchedPage::new("https://a.com", "A", content.clone(), 0);
    let p2 = FetchedPage::new("https://b.com", "B", content, 0);
    assert!(filter_page(&p1, &mut seen));
    assert!(
        filter_page(&p2, &mut seen),
        "different URL with same content must not be deduped"
    );
}

// ── Search stub ───────────────────────────────────────────────────────────────

#[test]
fn search_stub_returns_empty_vec() {
    let results = search_stub("best Rust async runtimes");
    assert!(results.is_empty(), "stub must always return empty");
}

// ── run_round with stub backend ───────────────────────────────────────────────

#[test]
fn run_round_with_stub_returns_empty() {
    let mut seen = HashSet::new();
    let queries = vec!["Rust ownership".to_string(), "async Rust".to_string()];
    let pages = run_round(&queries, &mut seen, 0);
    assert!(pages.is_empty());
}

// ── Partial report construction ───────────────────────────────────────────────

#[test]
fn build_partial_report_produces_partial_report() {
    let mut job = ResearchJob::new("j5", "What is ownership?", 3, 0);
    job.rounds_completed = 3;
    let plan = ResearchPlan::new("A complete explanation of ownership");
    let report = build_partial_report(&job, &plan);
    assert!(!report.success_criteria_met);
    assert!(report.partial_reason.is_some());
    let reason = report.partial_reason.unwrap();
    assert!(reason.contains("3"), "reason must reference max_rounds");
    assert!(reason.contains("ownership") || reason.contains("success criteria"));
}

#[test]
fn partial_report_is_protected() {
    let job = ResearchJob::new("j6", "question", 5, 0);
    let plan = ResearchPlan::new("criteria");
    let report = build_partial_report(&job, &plan);
    assert!(report.is_protected, "partial reports must be protected");
}

// ── ResearchReport constructors ───────────────────────────────────────────────

#[test]
fn complete_report_sets_success_criteria_met() {
    let report = ResearchReport::complete("question", "report body", vec![], 3);
    assert!(report.success_criteria_met);
    assert!(report.partial_reason.is_none());
    assert!(report.is_protected);
}

#[test]
fn partial_report_via_constructor() {
    let report = ResearchReport::partial("question", 5, "ran out of rounds");
    assert!(!report.success_criteria_met);
    assert_eq!(report.rounds_completed, 5);
    assert!(report.is_protected);
    assert!(report.partial_reason.unwrap().contains("ran out"));
}

// ── Untrusted content wrapping ────────────────────────────────────────────────

#[test]
fn wrap_untrusted_includes_label() {
    let wrapped = wrap_untrusted("web-page", "some content here");
    assert!(
        wrapped.contains("web-page"),
        "wrapped output must include the label, got: {wrapped}"
    );
}

#[test]
fn wrap_untrusted_includes_content() {
    let wrapped = wrap_untrusted("source", "sentinel content xyz");
    assert!(
        wrapped.contains("sentinel content xyz"),
        "wrapped output must include the content"
    );
}

// ── Job directory path ────────────────────────────────────────────────────────

#[test]
fn job_dir_path_structure() {
    let root = std::path::Path::new("/workspace");
    let dir = job_dir(root, "job-001");
    assert_eq!(dir, std::path::PathBuf::from("/workspace/research/job-001"));
}

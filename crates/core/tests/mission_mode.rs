use cronus_core::mission::{
    ClarificationItem, Mission, MissionMode, MissionPhase, MissionStatus, PhaseIntent, PrdDocument,
    Proposal, ProposalStatus, UserStory, WorkType, all_stories_pass, clarifications_complete,
    classify_phase_intent, classify_work_type, clear_mode_flag, mission_dir, resolve_mode,
    write_mode_flag,
};

// ── Mission ID format ─────────────────────────────────────────────────────────

#[test]
fn mission_id_has_expected_prefix_and_format() {
    // 2024-01-15 00:00:00 UTC = 1705276800 seconds = 1705276800000 ms
    let id = Mission::make_id(1_705_276_800_000);
    assert!(
        id.starts_with("mission-"),
        "id must start with 'mission-', got: {id}"
    );
    let parts: Vec<&str> = id.splitn(3, '-').collect();
    // parts = ["mission", "YYYYMMDD", "HHMMSS"]
    assert_eq!(parts.len(), 3);
    assert_eq!(
        parts[1].len(),
        8,
        "date part must be 8 digits, got: {}",
        parts[1]
    );
    assert_eq!(
        parts[2].len(),
        6,
        "time part must be 6 digits, got: {}",
        parts[2]
    );
}

#[test]
fn mission_id_known_timestamp() {
    // 2024-01-15 11:30:45 UTC = 1705318245 s
    let id = Mission::make_id(1_705_318_245_000);
    assert_eq!(id, "mission-20240115-113045");
}

// ── all_stories_pass ──────────────────────────────────────────────────────────

#[test]
fn all_stories_pass_true_when_all_passing() {
    let mut prd = PrdDocument::new("Proj", "Description");
    let mut s1 = UserStory::new("US1", "Login", "As a user I can log in");
    s1.passes = true;
    let mut s2 = UserStory::new("US2", "Logout", "As a user I can log out");
    s2.passes = true;
    prd.add_story(s1);
    prd.add_story(s2);
    assert!(all_stories_pass(&prd));
}

#[test]
fn all_stories_pass_false_when_one_failing() {
    let mut prd = PrdDocument::new("Proj", "Description");
    let mut s1 = UserStory::new("US1", "Login", "As a user I can log in");
    s1.passes = true;
    let s2 = UserStory::new("US2", "Register", "As a user I can register");
    prd.add_story(s1);
    prd.add_story(s2); // passes = false by default
    assert!(!all_stories_pass(&prd));
}

#[test]
fn all_stories_pass_false_for_empty_prd() {
    let prd = PrdDocument::new("Empty", "No stories");
    assert!(!all_stories_pass(&prd), "empty PRD must not pass");
}

// ── Mission lifecycle ─────────────────────────────────────────────────────────

#[test]
fn mission_starts_in_exploration_planning() {
    let m = Mission::new("m1".into(), "task".into(), MissionMode::Full, 10, 0);
    assert_eq!(m.phase, MissionPhase::Exploration);
    assert_eq!(m.status, MissionStatus::Planning);
}

#[test]
fn confirm_transitions_to_execution_running() {
    let mut m = Mission::new("m2".into(), "task".into(), MissionMode::Full, 10, 0);
    m.confirm();
    assert_eq!(m.phase, MissionPhase::Execution);
    assert_eq!(m.status, MissionStatus::Running);
}

#[test]
fn tick_with_max_one_iteration_produces_partial() {
    let mut m = Mission::new("m3".into(), "task".into(), MissionMode::Full, 1, 0);
    m.confirm();
    let mut prd = PrdDocument::new("P", "D");
    prd.add_story(UserStory::new("US1", "T", "story")); // passes = false
    let done = m.tick(&prd);
    assert!(done);
    assert_eq!(m.status, MissionStatus::Partial);
}

#[test]
fn tick_completes_when_all_stories_pass() {
    let mut m = Mission::new("m4".into(), "task".into(), MissionMode::Full, 5, 0);
    m.confirm();
    let mut prd = PrdDocument::new("P", "D");
    let mut s = UserStory::new("US1", "T", "story");
    s.passes = true;
    prd.add_story(s);
    let done = m.tick(&prd);
    assert!(done);
    assert_eq!(m.status, MissionStatus::Complete);
}

#[test]
fn abort_sets_aborted_status() {
    let mut m = Mission::new("m5".into(), "task".into(), MissionMode::Full, 5, 0);
    m.abort();
    assert_eq!(m.status, MissionStatus::Aborted);
}

// ── resolve_mode priority ─────────────────────────────────────────────────────

#[test]
fn resolve_mode_env_overrides_config() {
    // SAFETY: test runs single-threaded or with unique key; no concurrent readers
    unsafe { std::env::set_var("CRONUS_MISSION_MODE", "ultra") };
    let mode = resolve_mode(Some("lite"));
    unsafe { std::env::remove_var("CRONUS_MISSION_MODE") };
    assert_eq!(mode, MissionMode::Ultra, "env must win over config");
}

#[test]
fn resolve_mode_config_used_when_no_env() {
    unsafe { std::env::remove_var("CRONUS_MISSION_MODE") };
    let mode = resolve_mode(Some("off"));
    assert_eq!(mode, MissionMode::Off);
}

#[test]
fn resolve_mode_defaults_to_full() {
    unsafe { std::env::remove_var("CRONUS_MISSION_MODE") };
    let mode = resolve_mode(None);
    assert_eq!(mode, MissionMode::Full);
}

#[test]
fn resolve_mode_invalid_env_falls_to_config() {
    unsafe { std::env::set_var("CRONUS_MISSION_MODE", "garbage") };
    let mode = resolve_mode(Some("lite"));
    unsafe { std::env::remove_var("CRONUS_MISSION_MODE") };
    assert_eq!(mode, MissionMode::Lite);
}

// ── Flag file I/O ─────────────────────────────────────────────────────────────

#[test]
fn write_and_clear_mode_flag() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cronus_test_{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");

    write_mode_flag(&dir, MissionMode::Ultra).expect("write flag");
    let content = std::fs::read_to_string(dir.join(".mission-mode")).expect("read flag");
    assert_eq!(content, "ultra");

    clear_mode_flag(&dir).expect("clear flag");
    let content = std::fs::read_to_string(dir.join(".mission-mode")).expect("read after clear");
    assert_eq!(content, "off");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mission_dir_path_structure() {
    let root = std::path::Path::new("/workspace");
    let dir = mission_dir(root, "mission-20240115-120000");
    assert_eq!(
        dir,
        std::path::PathBuf::from("/workspace/missions/mission-20240115-120000")
    );
}

// ── Work type classification ──────────────────────────────────────────────────

#[test]
fn classify_bugfix_from_fix_verb() {
    assert_eq!(
        classify_work_type("fix the null pointer exception"),
        WorkType::Bugfix
    );
}

#[test]
fn classify_bugfix_from_keyword() {
    assert_eq!(
        classify_work_type("add a bug fix for the session expiry"),
        WorkType::Bugfix
    );
}

#[test]
fn classify_refactor_from_verb() {
    assert_eq!(
        classify_work_type("refactor the auth module"),
        WorkType::Refactor
    );
}

#[test]
fn classify_feature_as_default() {
    assert_eq!(
        classify_work_type("add dark mode support"),
        WorkType::Feature
    );
}

#[test]
fn classify_test_from_verb() {
    assert_eq!(classify_work_type("test the payment flow"), WorkType::Test);
}

#[test]
fn classify_docs_from_verb() {
    assert_eq!(classify_work_type("doc the public API"), WorkType::Docs);
}

// ── Phase intent classification ───────────────────────────────────────────────

#[test]
fn classify_implementation_intent() {
    let (intent, confidence) = classify_phase_intent("build the authentication service");
    assert_eq!(intent, PhaseIntent::Implementation);
    assert!(confidence >= 70);
}

#[test]
fn classify_debugging_intent() {
    let (intent, _) = classify_phase_intent("fix the null pointer error");
    assert_eq!(intent, PhaseIntent::Debugging);
}

#[test]
fn classify_planning_intent() {
    let (intent, _) = classify_phase_intent("plan the architecture for the API");
    assert_eq!(intent, PhaseIntent::Planning);
}

#[test]
fn classify_exploration_as_default() {
    let (intent, confidence) = classify_phase_intent("look around the codebase");
    assert_eq!(intent, PhaseIntent::Exploration);
    assert!(confidence < 75, "exploration confidence should be low");
}

// ── Clarifications ────────────────────────────────────────────────────────────

#[test]
fn clarifications_complete_when_all_answered() {
    let mut item = ClarificationItem::new("What language?");
    item.answer = Some("Rust".to_string());
    assert!(clarifications_complete(&[item]));
}

#[test]
fn clarifications_incomplete_when_unanswered() {
    let item = ClarificationItem::new("What language?");
    assert!(!clarifications_complete(&[item]));
}

#[test]
fn clarifications_complete_when_skipped() {
    let mut item = ClarificationItem::new("Optional question");
    item.skipped = true;
    assert!(clarifications_complete(&[item]));
}

#[test]
fn locked_items_excluded_from_completeness_check() {
    let mut locked = ClarificationItem::new("Locked");
    locked.locked = true;
    // locked item with no answer — must not block completion
    assert!(clarifications_complete(&[locked]));
}

#[test]
fn mixed_items_incomplete_when_one_unanswered_unlocked() {
    let mut answered = ClarificationItem::new("Q1");
    answered.answer = Some("yes".into());
    let unanswered = ClarificationItem::new("Q2");
    assert!(!clarifications_complete(&[answered, unanswered]));
}

// ── Proposal lifecycle ────────────────────────────────────────────────────────

#[test]
fn proposal_starts_as_draft() {
    let p = Proposal::new("c1", "2024-01-15", MissionMode::Full, "add feature");
    assert_eq!(p.status, ProposalStatus::Draft);
    assert!(!p.is_immutable());
}

#[test]
fn proposal_mark_ready_then_accept() {
    let mut p = Proposal::new("c1", "2024-01-15", MissionMode::Full, "add feature");
    p.mark_ready();
    assert_eq!(p.status, ProposalStatus::Ready);
    p.accept();
    assert_eq!(p.status, ProposalStatus::Accepted);
    assert!(p.is_immutable());
}

#[test]
fn proposal_mark_ready_then_reject() {
    let mut p = Proposal::new("c1", "2024-01-15", MissionMode::Full, "add feature");
    p.mark_ready();
    p.reject();
    assert_eq!(p.status, ProposalStatus::Rejected);
}

#[test]
fn proposal_cannot_skip_ready_state() {
    let mut p = Proposal::new("c1", "2024-01-15", MissionMode::Full, "feature");
    // accept() requires status == Ready; while Draft, it is a no-op
    p.accept();
    assert_eq!(p.status, ProposalStatus::Draft);
}

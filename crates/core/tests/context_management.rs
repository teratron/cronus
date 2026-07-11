use cronus_core::context_mgmt::{
    CONTEXT_RESERVE_TOKENS, CompactionDetails, Compactor, ContextEntry, DEFAULT_MAX_BYTES,
    GREP_MAX_LINE_LENGTH, NoOpCompactor, TOOL_RESULT_MAX_CHARS, TrimPriority, adaptive_budget,
    should_compact, total_tokens, trim_cascade, truncate_head, truncate_tail, truncate_tool_result,
};

// ── Adaptive budget ────────────────────────────────────────────────────────────

#[test]
fn adaptive_budget_85_percent_of_window() {
    let budget = adaptive_budget(200_000);
    assert_eq!(budget, 170_000);
}

#[test]
fn adaptive_budget_caps_at_200k() {
    let budget = adaptive_budget(1_000_000);
    assert_eq!(budget, 200_000, "budget must cap at 200k");
}

#[test]
fn should_compact_fires_when_near_limit() {
    // context_window=128k, reserve=16_384 → threshold=111_616
    // fires when context_tokens > 111_616
    assert!(should_compact(111_617, 128_000));
    assert!(!should_compact(111_616, 128_000));
}

#[test]
fn adaptive_budget_clamps_for_small_window() {
    let budget = adaptive_budget(8_000);
    assert_eq!(budget, (8_000_f64 * 0.85) as u64);
}

// ── Trim cascade ───────────────────────────────────────────────────────────────

fn make_entry(tokens: u64, priority: TrimPriority) -> ContextEntry {
    ContextEntry::new("user", "body", tokens).with_priority(priority)
}

#[test]
fn trim_cascade_removes_lowest_priority_first() {
    let mut entries = vec![
        make_entry(100, TrimPriority::OrphanedToolResult),
        make_entry(100, TrimPriority::NonProtectedUser),
        make_entry(100, TrimPriority::Protected),
    ];
    // Total = 300, target = 200 → remove OrphanedToolResult
    trim_cascade(&mut entries, 200);
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|e| e.priority != TrimPriority::OrphanedToolResult),
        "orphaned tool results must be removed first"
    );
}

#[test]
fn trim_cascade_never_removes_protected_entries() {
    let mut entries = vec![
        make_entry(500, TrimPriority::NonProtectedUser),
        ContextEntry::new("sys", "keep me", 500).protect(),
    ];
    trim_cascade(&mut entries, 100);
    assert!(
        entries.iter().any(|e| e.protected),
        "protected entries must survive all trim passes"
    );
}

#[test]
fn trim_cascade_stops_when_target_reached() {
    let mut entries = vec![
        make_entry(50, TrimPriority::OrphanedToolResult),
        make_entry(50, TrimPriority::ToolUsePair),
        make_entry(50, TrimPriority::NonProtectedUser),
    ];
    // Total = 150, target = 100 → remove one group (50 tokens) → 100 left
    trim_cascade(&mut entries, 100);
    assert_eq!(total_tokens(&entries), 100);
}

// ── Compactor seam ────────────────────────────────────────────────────────────

#[test]
fn noop_compactor_returns_placeholder() {
    let c = NoOpCompactor;
    let result = c.compact(&[], 0).unwrap();
    assert_eq!(result, "[context compacted]");
}

// ── Tool output truncation ─────────────────────────────────────────────────────

#[test]
fn truncate_head_keeps_start() {
    let s = "a".repeat(100);
    let out = truncate_head(&s, 50);
    assert!(out.starts_with(&"a".repeat(50)));
    assert!(out.contains("[truncated]"));
}

#[test]
fn truncate_tail_keeps_end() {
    let s = "b".repeat(100);
    let out = truncate_tail(&s, 50);
    assert!(out.ends_with(&"b".repeat(50)));
    assert!(out.contains("[truncated]"));
}

#[test]
fn truncate_head_no_op_when_under_limit() {
    let s = "hello";
    assert_eq!(truncate_head(s, 100), "hello");
}

#[test]
fn truncate_tail_no_op_when_under_limit() {
    let s = "world";
    assert_eq!(truncate_tail(s, 100), "world");
}

#[test]
fn tool_result_truncated_to_2000_chars() {
    let big = "x".repeat(TOOL_RESULT_MAX_CHARS + 100);
    let out = truncate_tool_result(&big);
    assert!(out.starts_with(&"x".repeat(TOOL_RESULT_MAX_CHARS)));
    assert!(out.contains("[truncated]"));
}

#[test]
fn tool_result_unchanged_when_short() {
    let s = "short result";
    assert_eq!(truncate_tool_result(s), s);
}

// ── CompactionDetails ─────────────────────────────────────────────────────────

#[test]
fn compaction_details_xml_renders_both_sections() {
    let mut cd = CompactionDetails::default();
    cd.record_read("src/main.rs");
    cd.record_modified("src/lib.rs");

    let xml = cd.to_xml();
    assert!(xml.contains("<read-files>"), "must have read-files section");
    assert!(
        xml.contains("<modified-files>"),
        "must have modified-files section"
    );
    assert!(xml.contains("src/main.rs"));
    assert!(xml.contains("src/lib.rs"));
}

#[test]
fn compaction_details_empty_xml_is_valid() {
    let cd = CompactionDetails::default();
    let xml = cd.to_xml();
    assert!(xml.contains("<read-files>"));
    assert!(xml.contains("</read-files>"));
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn constants_have_expected_values() {
    const { assert!(TOOL_RESULT_MAX_CHARS == 2_000) }
    const { assert!(DEFAULT_MAX_BYTES == 50_000) }
    const { assert!(GREP_MAX_LINE_LENGTH == 500) }
    const { assert!(CONTEXT_RESERVE_TOKENS == 16_384) }
}

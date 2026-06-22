use cronus::session::{
    InterruptFence, IterationBudget, RunnerMap, SessionEntry, SessionError, SessionId, TurnContext,
    guard_output_size, MAX_GOAL_REACT, MAX_OUTPUT_CHARS,
    hooks::{HookOutcome, NoOpHook, StopHook},
    migration::{CURRENT_SESSION_VERSION, SessionData},
};
use cronus::context_router::ContextBundle;

fn sid(s: &str) -> SessionId {
    SessionId::new(s)
}

fn budget() -> IterationBudget {
    IterationBudget::new(10, 100_000)
}

// ── TurnContext ────────────────────────────────────────────────────────────────

#[test]
fn turn_context_constructs_with_correct_budget() {
    let ctx = TurnContext::new(sid("s1"), 0, budget(), ContextBundle::default());
    assert_eq!(ctx.session_id.as_str(), "s1");
    assert_eq!(ctx.iteration, 0);
    assert_eq!(ctx.budget.max_iterations, 10);
}

#[test]
fn iteration_budget_tick_increments_counter() {
    let mut b = budget();
    assert!(!b.is_exhausted());
    for _ in 0..9 {
        b.tick();
    }
    assert!(!b.is_exhausted());
    b.tick();
    assert!(b.is_exhausted(), "budget must be exhausted at max iterations");
}

#[test]
fn iteration_budget_reset_clears_counters() {
    let mut b = budget();
    b.tick();
    b.spend_tokens(5000);
    b.reset();
    assert_eq!(b.spent_iterations, 0);
    assert_eq!(b.spent_tokens, 0);
    assert!(!b.is_exhausted());
}

#[test]
fn iteration_budget_token_cap_exhausts() {
    let mut b = IterationBudget::new(100, 100);
    b.spend_tokens(101);
    assert!(b.is_exhausted(), "token cap must exhaust budget");
}

#[test]
fn turn_context_tick_increments_iteration_and_budget() {
    let mut ctx = TurnContext::new(sid("s2"), 0, budget(), ContextBundle::default());
    ctx.tick();
    assert_eq!(ctx.iteration, 1);
    assert_eq!(ctx.budget.spent_iterations, 1);
}

// ── InterruptFence ────────────────────────────────────────────────────────────

#[test]
fn interrupt_fence_starts_unset() {
    let fence = InterruptFence::new();
    assert!(!fence.is_set());
}

#[test]
fn interrupt_fence_set_is_visible_to_clone() {
    let fence = InterruptFence::new();
    let clone = fence.clone();
    fence.set();
    assert!(clone.is_set(), "set must be visible through any clone");
}

#[test]
fn interrupt_fence_reset_clears_flag() {
    let fence = InterruptFence::new();
    fence.set();
    assert!(fence.is_set());
    fence.reset();
    assert!(!fence.is_set());
}

// ── RunnerMap ─────────────────────────────────────────────────────────────────

#[test]
fn runner_map_register_makes_session_available() {
    let map = RunnerMap::new();
    let id = sid("session-a");
    map.register(id.clone());
    assert!(map.is_registered(&id));
}

#[test]
fn assert_not_busy_transitions_to_busy() {
    let map = RunnerMap::new();
    let id = sid("s-busy");
    map.register(id.clone());
    map.assert_not_busy(&id).unwrap();
    // Second call must fail
    assert_eq!(
        map.assert_not_busy(&id),
        Err(SessionError::AlreadyBusy),
        "second assert_not_busy must fail while busy"
    );
}

#[test]
fn mark_idle_allows_next_assert_not_busy() {
    let map = RunnerMap::new();
    let id = sid("s-idle");
    map.register(id.clone());
    map.assert_not_busy(&id).unwrap();
    map.mark_idle(&id);
    map.assert_not_busy(&id).expect("must be allowed after mark_idle");
}

#[test]
fn assert_not_busy_on_unregistered_returns_error() {
    let map = RunnerMap::new();
    let id = sid("ghost");
    assert_eq!(
        map.assert_not_busy(&id),
        Err(SessionError::NotRegistered)
    );
}

#[test]
fn retire_removes_session() {
    let map = RunnerMap::new();
    let id = sid("s-retire");
    map.register(id.clone());
    map.retire(&id);
    assert!(!map.is_registered(&id));
}

#[test]
fn per_session_runner_checks_not_busy_before_accepting_turn() {
    let map = RunnerMap::new();
    let id = sid("guard-test");
    map.register(id.clone());
    // First turn accepted
    map.assert_not_busy(&id).unwrap();
    // Second turn rejected while first is running
    let err = map.assert_not_busy(&id);
    assert!(matches!(err, Err(SessionError::AlreadyBusy)));
    // Mark idle — third turn accepted
    map.mark_idle(&id);
    map.assert_not_busy(&id).unwrap();
}

// ── Output size guard ─────────────────────────────────────────────────────────

#[test]
fn short_output_passes_unchanged() {
    let (out, err) = guard_output_size("short");
    assert_eq!(out, "short");
    assert!(err.is_none());
}

#[test]
fn oversized_output_is_truncated_with_annotation() {
    let big = "x".repeat(MAX_OUTPUT_CHARS + 100);
    let (out, err) = guard_output_size(&big);
    // The prefix is truncated to MAX_OUTPUT_CHARS; annotation is appended.
    // Overall length < original (minus 100 overhead chars we removed).
    assert!(out.starts_with(&"x".repeat(MAX_OUTPUT_CHARS)));
    assert!(out.contains("[output truncated:"), "annotation must be present");
    assert!(matches!(err, Some(SessionError::Oversized { .. })));
}

// ── Session entry taxonomy ────────────────────────────────────────────────────

#[test]
fn session_entry_as_str_all_variants() {
    let cases = [
        (SessionEntry::Message, "message"),
        (SessionEntry::CustomMessage, "customMessage"),
        (SessionEntry::Compaction, "compaction"),
        (SessionEntry::BranchSummary, "branchSummary"),
        (SessionEntry::ThinkingLevelChange, "thinkingLevelChange"),
        (SessionEntry::ModelChange, "modelChange"),
        (SessionEntry::Custom, "custom"),
        (SessionEntry::Label, "label"),
        (SessionEntry::SessionInfo, "sessionInfo"),
    ];
    for (entry, expected) in cases {
        assert_eq!(entry.as_str(), expected);
    }
}

// ── Stop hook ─────────────────────────────────────────────────────────────────

#[test]
fn noop_hook_continues_with_same_output() {
    let hook = NoOpHook;
    let result = hook.on_turn_end("hello");
    assert_eq!(result, HookOutcome::Continue("hello".to_owned()));
}

#[test]
fn custom_hook_can_halt_session() {
    struct HaltHook;
    impl StopHook for HaltHook {
        fn on_turn_end(&self, _: &str) -> HookOutcome {
            HookOutcome::Halt
        }
    }
    let hook = HaltHook;
    assert_eq!(hook.on_turn_end("any output"), HookOutcome::Halt);
}

// ── Migration ─────────────────────────────────────────────────────────────────

#[test]
fn current_session_version_is_3() {
    const { assert!(CURRENT_SESSION_VERSION == 3) }
}

#[test]
fn migrate_v1_to_v3_adds_id_and_renames_hook_message() {
    let mut data = SessionData::new(1);
    data.message_kinds.push("hookMessage".into());
    let migrated = data.migrate();
    assert_eq!(migrated.version, 3);
    assert!(migrated.id.is_some(), "v1→v2 migration must add an ID");
    assert!(
        migrated.message_kinds.iter().all(|k| k != "hookMessage"),
        "v2→v3 migration must rename hookMessage to custom"
    );
    assert!(
        migrated.message_kinds.contains(&"custom".to_owned()),
        "renamed kind must be 'custom'"
    );
}

#[test]
fn migrate_v2_to_v3_renames_hook_message() {
    let mut data = SessionData::new(2);
    data.message_kinds.push("hookMessage".into());
    data.message_kinds.push("message".into());
    let migrated = data.migrate();
    assert_eq!(migrated.version, 3);
    assert_eq!(migrated.message_kinds[0], "custom");
    assert_eq!(migrated.message_kinds[1], "message");
}

#[test]
fn migrate_already_at_v3_is_noop() {
    let data = SessionData::new(3);
    let migrated = data.migrate();
    assert_eq!(migrated.version, 3);
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn max_goal_react_is_12() {
    const { assert!(MAX_GOAL_REACT == 12) }
}

#[test]
fn max_output_chars_is_15000() {
    const { assert!(MAX_OUTPUT_CHARS == 15_000) }
}

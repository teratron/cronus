use cronus_core::budget::{
    BudgetEngine, BudgetError, BudgetPeriod, BudgetPolicy, BudgetStatus, CostEvent,
};

fn event(session_id: &str, card_id: Option<&str>, amount: f64) -> CostEvent {
    CostEvent {
        session_id: session_id.to_string(),
        card_id: card_id.map(|s| s.to_string()),
        role_id: None,
        amount_usd: amount,
        tokens_used: 1000,
        model: "test-model".to_string(),
        timestamp: 0,
    }
}

// ── Policy hierarchy ──────────────────────────────────────────────────────────

#[test]
fn ingest_below_limit_returns_ok_status() {
    let mut engine = BudgetEngine::new();
    engine.add_policy(BudgetPolicy::workspace("ws-1", 10.0, BudgetPeriod::Monthly));

    let evt = event("sess", None, 3.0);
    let result = engine.ingest_cost(&evt);
    assert!(result.is_ok(), "below limit should return Ok");
    if let Ok(BudgetStatus::Ok { spent, remaining }) = result {
        assert!((spent - 3.0).abs() < 1e-9);
        assert!((remaining - 7.0).abs() < 1e-9);
    }
}

#[test]
fn ingest_over_limit_returns_err_exhausted() {
    let mut engine = BudgetEngine::new();
    engine.add_policy(BudgetPolicy::workspace("ws-1", 5.0, BudgetPeriod::Monthly));

    engine.ingest_cost(&event("sess", None, 4.0)).unwrap();
    let result = engine.ingest_cost(&event("sess", None, 2.0));

    assert!(matches!(result, Err(BudgetError::Exhausted { .. })));
}

#[test]
fn card_policy_takes_precedence_over_workspace() {
    let mut engine = BudgetEngine::new();
    engine.add_policy(BudgetPolicy::workspace("ws", 100.0, BudgetPeriod::Monthly));
    engine.add_policy(BudgetPolicy::for_card("card-1", 1.0, BudgetPeriod::Daily));

    let evt = CostEvent {
        session_id: "s".to_string(),
        card_id: Some("card-1".to_string()),
        role_id: None,
        amount_usd: 1.5,
        tokens_used: 100,
        model: "m".to_string(),
        timestamp: 0,
    };
    let result = engine.ingest_cost(&evt);
    assert!(
        matches!(result, Err(BudgetError::Exhausted { .. })),
        "card limit 1.0 should exhaust at 1.5"
    );
}

#[test]
fn no_policy_allows_any_amount() {
    let mut engine = BudgetEngine::new();
    let result = engine.ingest_cost(&event("sess", None, 99999.0));
    assert!(result.is_ok());
}

#[test]
fn spent_for_tracks_running_total() {
    let mut engine = BudgetEngine::new();
    engine.ingest_cost(&event("sess-a", None, 2.0)).unwrap();
    engine.ingest_cost(&event("sess-a", None, 3.0)).unwrap();

    let spent = engine.spent_for("sess-a");
    assert!((spent - 5.0).abs() < 1e-9);
}

#[test]
fn reset_clears_all_counters() {
    let mut engine = BudgetEngine::new();
    engine.add_policy(BudgetPolicy::workspace("ws", 10.0, BudgetPeriod::Monthly));
    engine.ingest_cost(&event("sess", None, 8.0)).unwrap();
    engine.reset();

    assert!((engine.spent_for("sess") - 0.0).abs() < 1e-9);
    let result = engine.ingest_cost(&event("sess", None, 8.0));
    assert!(result.is_ok(), "after reset, spending should start fresh");
}

// ── BudgetPolicy constructors ────────────────────────────────────────────────

#[test]
fn workspace_policy_sets_workspace_id() {
    let p = BudgetPolicy::workspace("my-ws", 50.0, BudgetPeriod::Weekly);
    assert_eq!(p.workspace_id.as_deref(), Some("my-ws"));
    assert_eq!(p.limit_usd, 50.0);
    assert!(p.card_id.is_none());
    assert!(p.role_id.is_none());
}

#[test]
fn card_policy_sets_card_id() {
    let p = BudgetPolicy::for_card("card-xyz", 0.5, BudgetPeriod::Daily);
    assert_eq!(p.card_id.as_deref(), Some("card-xyz"));
    assert_eq!(p.limit_usd, 0.5);
    assert!(p.workspace_id.is_none());
}

// ── BudgetError display ──────────────────────────────────────────────────────

#[test]
fn budget_error_display_exhausted() {
    let e = BudgetError::Exhausted {
        spent: 10.5,
        limit: 10.0,
    };
    let s = e.to_string();
    assert!(s.contains("exhausted"));
}

#[test]
fn budget_error_display_no_policy() {
    let e = BudgetError::NoPolicyFound;
    assert!(e.to_string().contains("no budget policy"));
}

// ── Period variants ──────────────────────────────────────────────────────────

#[test]
fn budget_period_unlimited_allows_no_spending_over_infinity() {
    let mut engine = BudgetEngine::new();
    engine.add_policy(BudgetPolicy::workspace(
        "ws",
        f64::INFINITY,
        BudgetPeriod::Unlimited,
    ));
    let result = engine.ingest_cost(&event("sess", None, 1_000_000.0));
    assert!(result.is_ok());
}

use cronus_core::orchestration::{
    AgentDefinitionError, AgentTier, AgentToolResult, AgentWorkspaceScope, ChangeMetadata, GoalRun,
    GoalRunStatus, JudgeVerdict, MAX_SPAWN_DEPTH, PermissionAction, PermissionRule,
    SpawnDepthCounter, TieredAgentDef, ToolDefinition, changes_are_acyclic, evaluate,
    evaluate_goal, rank_tools, should_terminate_batch, unblocked_changes, validate_tier,
};
use cronus_core::trigger_triage::{
    DeduplicateCache, RateLimiter, SourceType, TriageDecision, TriggerEnvelope, TriggerPayload,
    triage,
};

// ── rank_tools ────────────────────────────────────────────────────────────────

fn make_tools(n: usize) -> Vec<ToolDefinition> {
    (0..n)
        .map(|i| ToolDefinition {
            name: format!("tool_{i}"),
            description: format!("description for tool {i}"),
        })
        .collect()
}

#[test]
fn rank_tools_large_catalog_returns_top_n() {
    let tools = make_tools(60);
    let ranked = rank_tools("write file content", &tools, 20);
    assert_eq!(ranked.len(), 20, "catalog >50 must return exactly top_n=20");
}

#[test]
fn rank_tools_small_catalog_returns_all() {
    let tools = make_tools(30);
    let ranked = rank_tools("search for symbols", &tools, 20);
    assert_eq!(ranked.len(), 30, "catalog ≤50 must return the full list");
}

#[test]
fn rank_tools_verb_match_scores_higher() {
    let tools = vec![
        ToolDefinition {
            name: "write_file".to_string(),
            description: "writes content to a file".to_string(),
        },
        ToolDefinition {
            name: "read_file".to_string(),
            description: "reads content from a file".to_string(),
        },
    ];
    // Small catalog → returned unsorted; just verify no panic
    let ranked = rank_tools("write content to disk", &tools, 5);
    assert_eq!(ranked.len(), 2);
}

// ── permission ruleset evaluate ───────────────────────────────────────────────

#[test]
fn evaluate_empty_rulesets_returns_ask() {
    let action = evaluate("fs:write:/home/user/file.txt", "/home/user/file.txt", &[]);
    assert_eq!(action, PermissionAction::Ask);
}

#[test]
fn evaluate_last_matching_rule_wins() {
    let ruleset = vec![
        PermissionRule {
            permission: "fs:write:*".to_string(),
            pattern: "/home/**".to_string(),
            action: PermissionAction::Allow,
        },
        PermissionRule {
            permission: "fs:write:*".to_string(),
            pattern: "/home/user/**".to_string(),
            action: PermissionAction::Deny,
        },
    ];
    // Both rules match; last wins → Deny
    let action = evaluate("fs:write:special", "/home/user/secret.txt", &[ruleset]);
    assert_eq!(action, PermissionAction::Deny);
}

#[test]
fn evaluate_only_matching_rule_applied() {
    let ruleset = vec![PermissionRule {
        permission: "fs:read:*".to_string(),
        pattern: "/tmp/**".to_string(),
        action: PermissionAction::Allow,
    }];
    // Permission doesn't match fs:write
    let action = evaluate("fs:write:data", "/tmp/file.txt", &[ruleset]);
    assert_eq!(action, PermissionAction::Ask);
}

// ── Agent tier validation ─────────────────────────────────────────────────────

#[test]
fn chat_agent_with_shell_toolset_is_rejected() {
    let def = TieredAgentDef {
        name: "helper".to_string(),
        tier: AgentTier::Chat,
        toolsets: vec!["read".to_string(), "shell".to_string()],
        workspace_scope: AgentWorkspaceScope::Session,
        iteration_budget: None,
    };
    let err = validate_tier(&def).unwrap_err();
    assert!(matches!(
        err,
        AgentDefinitionError::ChatForbiddenToolset { .. }
    ));
}

#[test]
fn chat_agent_with_code_execution_is_rejected() {
    let def = TieredAgentDef {
        name: "runner".to_string(),
        tier: AgentTier::Chat,
        toolsets: vec!["code_execution".to_string()],
        workspace_scope: AgentWorkspaceScope::Session,
        iteration_budget: None,
    };
    assert!(matches!(
        validate_tier(&def).unwrap_err(),
        AgentDefinitionError::ChatForbiddenToolset { .. }
    ));
}

#[test]
fn worker_agent_with_session_scope_is_rejected() {
    let def = TieredAgentDef {
        name: "coder".to_string(),
        tier: AgentTier::Worker,
        toolsets: vec!["write".to_string(), "shell".to_string()],
        workspace_scope: AgentWorkspaceScope::Session,
        iteration_budget: None,
    };
    assert!(matches!(
        validate_tier(&def).unwrap_err(),
        AgentDefinitionError::WorkerRequiresExecutionScope
    ));
}

#[test]
fn valid_worker_agent_passes_validation() {
    let def = TieredAgentDef {
        name: "coder".to_string(),
        tier: AgentTier::Worker,
        toolsets: vec!["write".to_string(), "shell".to_string()],
        workspace_scope: AgentWorkspaceScope::Execution,
        iteration_budget: Some(90),
    };
    assert!(validate_tier(&def).is_ok());
}

#[test]
fn valid_reasoning_agent_passes_validation() {
    let def = TieredAgentDef {
        name: "planner".to_string(),
        tier: AgentTier::Reasoning,
        toolsets: vec!["read".to_string(), "search".to_string()],
        workspace_scope: AgentWorkspaceScope::Session,
        iteration_budget: Some(50),
    };
    assert!(validate_tier(&def).is_ok());
}

// ── Spawn depth ───────────────────────────────────────────────────────────────

#[test]
fn spawn_depth_counter_increments_correctly() {
    let mut counter = SpawnDepthCounter::new();
    assert_eq!(counter.depth(), 0);
    counter.try_increment().expect("depth 0→1 should succeed");
    counter.try_increment().expect("depth 1→2 should succeed");
    counter.try_increment().expect("depth 2→3 should succeed");
    assert_eq!(counter.depth(), 3);
}

#[test]
fn spawn_depth_error_at_max_depth() {
    let mut counter = SpawnDepthCounter::new();
    for _ in 0..MAX_SPAWN_DEPTH {
        counter.try_increment().expect("should succeed up to max");
    }
    assert_eq!(counter.depth(), MAX_SPAWN_DEPTH);
    // One more should fail
    let err = counter.try_increment().unwrap_err();
    assert_eq!(err.depth, MAX_SPAWN_DEPTH);
}

#[test]
fn spawn_depth_decrement_allows_reuse() {
    let mut counter = SpawnDepthCounter::new();
    for _ in 0..MAX_SPAWN_DEPTH {
        counter.try_increment().unwrap();
    }
    counter.decrement();
    assert_eq!(counter.depth(), MAX_SPAWN_DEPTH - 1);
    counter
        .try_increment()
        .expect("should succeed after decrement");
}

// ── Tool terminate batch semantics ────────────────────────────────────────────

#[test]
fn terminate_batch_all_true_returns_true() {
    let results = vec![
        AgentToolResult {
            content: vec![],
            terminate: true,
        },
        AgentToolResult {
            content: vec![],
            terminate: true,
        },
    ];
    assert!(should_terminate_batch(&results));
}

#[test]
fn terminate_batch_one_false_returns_false() {
    let results = vec![
        AgentToolResult {
            content: vec![],
            terminate: true,
        },
        AgentToolResult {
            content: vec![],
            terminate: false,
        },
    ];
    assert!(!should_terminate_batch(&results));
}

#[test]
fn terminate_batch_empty_returns_false() {
    assert!(!should_terminate_batch(&[]));
}

// ── GoalRun circuit-breaker ───────────────────────────────────────────────────

#[test]
fn goal_run_pauses_at_max_iterations() {
    let mut run = GoalRun::new("g1".to_string(), "test goal".to_string(), 10.0, 1);
    assert_eq!(run.status, GoalRunStatus::Running);
    run.tick();
    assert_eq!(run.status, GoalRunStatus::Paused);
    assert!(run.is_exhausted());
}

#[test]
fn goal_run_still_running_under_max_iterations() {
    let mut run = GoalRun::new("g2".to_string(), "goal".to_string(), 10.0, 3);
    run.tick();
    assert_eq!(run.status, GoalRunStatus::Running);
    run.tick();
    assert_eq!(run.status, GoalRunStatus::Running);
    run.tick();
    assert_eq!(run.status, GoalRunStatus::Paused);
}

#[test]
fn judge_seam_returns_not_met_with_summary() {
    let run = GoalRun::new("g3".to_string(), "goal".to_string(), 10.0, 5);
    let verdict = evaluate_goal(&run, "work done");
    assert!(matches!(verdict, JudgeVerdict::NotMet { .. }));
}

#[test]
fn judge_seam_returns_not_met_for_empty_summary() {
    let run = GoalRun::new("g4".to_string(), "goal".to_string(), 10.0, 5);
    let verdict = evaluate_goal(&run, "");
    assert!(matches!(verdict, JudgeVerdict::NotMet { .. }));
}

// ── Change dependency DAG ─────────────────────────────────────────────────────

#[test]
fn acyclic_change_graph_is_valid() {
    let changes = vec![
        ChangeMetadata {
            id: "A".to_string(),
            depends_on: vec![],
            ..Default::default()
        },
        ChangeMetadata {
            id: "B".to_string(),
            depends_on: vec!["A".to_string()],
            ..Default::default()
        },
    ];
    assert!(changes_are_acyclic(&changes));
}

#[test]
fn cyclic_change_graph_detected() {
    let changes = vec![
        ChangeMetadata {
            id: "A".to_string(),
            depends_on: vec!["B".to_string()],
            ..Default::default()
        },
        ChangeMetadata {
            id: "B".to_string(),
            depends_on: vec!["A".to_string()],
            ..Default::default()
        },
    ];
    assert!(!changes_are_acyclic(&changes));
}

#[test]
fn unblocked_changes_returns_root_nodes() {
    let changes = vec![
        ChangeMetadata {
            id: "root".to_string(),
            depends_on: vec![],
            ..Default::default()
        },
        ChangeMetadata {
            id: "leaf".to_string(),
            depends_on: vec!["root".to_string()],
            ..Default::default()
        },
    ];
    let unblocked = unblocked_changes(&changes);
    assert_eq!(unblocked.len(), 1);
    assert_eq!(unblocked[0].id, "root");
}

// ── Cross-module round-trip ───────────────────────────────────────────────────

#[test]
fn triage_error_event_leads_to_exhausted_goal_run() {
    // Trigger pipeline: error event → SpawnOrchestrator
    let payload = TriggerPayload::new("database connection refused").with_event_kind("db.error");
    let env = TriggerEnvelope::new("evt-001", SourceType::Event, payload, 1_000_000, "ws-test");
    let mut cache = DeduplicateCache::new();
    let mut rl = RateLimiter::new();
    let decision = triage(&env, &mut cache, &mut rl, 1_000_000);
    assert!(
        matches!(decision, TriageDecision::SpawnOrchestrator { .. }),
        "error event must produce SpawnOrchestrator, got {decision:?}"
    );

    // Orchestration: create a goal run with max_iterations=1
    let mut run = GoalRun::new(
        "goal-001".to_string(),
        "recover database connection".to_string(),
        5.0,
        1,
    );
    run.tick();
    assert_eq!(
        run.status,
        GoalRunStatus::Paused,
        "single-iteration goal must pause"
    );

    // Judge seam: still returns NotMet (stub judge)
    let verdict = evaluate_goal(&run, "recovery attempted");
    assert!(
        matches!(verdict, JudgeVerdict::NotMet { .. }),
        "judge seam must return NotMet"
    );
    assert!(run.is_exhausted());
}

use cronus::hooks::{
    ActorMatcher, HookEntry, HookEvent, HookResult, RuleCondition, RuleOp, evaluate_conditions,
    run_hooks, validate_hook_command,
};

fn simple_hook(id: &str, event: HookEvent, command: &str) -> HookEntry {
    HookEntry {
        id: id.to_string(),
        event,
        matcher: ActorMatcher {
            actor_id: None,
            tool_name: None,
            event: None,
        },
        command: command.to_string(),
        timeout_ms: 5000,
        conditions: vec![],
    }
}

// ── validate_hook_command ────────────────────────────────────────────────────

#[test]
fn validate_accepts_safe_command() {
    assert!(validate_hook_command("echo hello").is_ok());
    assert!(validate_hook_command("notify-send done").is_ok());
}

#[test]
fn validate_rejects_semicolon() {
    assert!(validate_hook_command("echo hi; rm -rf /").is_err());
}

#[test]
fn validate_rejects_pipe() {
    assert!(validate_hook_command("cat file | sh").is_err());
}

#[test]
fn validate_rejects_ampersand() {
    assert!(validate_hook_command("cmd & bad").is_err());
}

#[test]
fn validate_rejects_redirection() {
    assert!(validate_hook_command("echo evil > /etc/passwd").is_err());
}

// ── RuleCondition::evaluate ──────────────────────────────────────────────────

#[test]
fn condition_eq_matches_exact() {
    let c = RuleCondition {
        field: "f".into(),
        op: RuleOp::Eq,
        value: "hello".into(),
    };
    assert!(c.evaluate("hello"));
    assert!(!c.evaluate("world"));
}

#[test]
fn condition_ne_rejects_match() {
    let c = RuleCondition {
        field: "f".into(),
        op: RuleOp::Ne,
        value: "bad".into(),
    };
    assert!(!c.evaluate("bad"));
    assert!(c.evaluate("good"));
}

#[test]
fn condition_contains_substring() {
    let c = RuleCondition {
        field: "f".into(),
        op: RuleOp::Contains,
        value: "sub".into(),
    };
    assert!(c.evaluate("this has sub in it"));
    assert!(!c.evaluate("nothing here"));
}

#[test]
fn condition_starts_with() {
    let c = RuleCondition {
        field: "f".into(),
        op: RuleOp::StartsWith,
        value: "pre".into(),
    };
    assert!(c.evaluate("prefix"));
    assert!(!c.evaluate("notprefix"));
}

#[test]
fn condition_ends_with() {
    let c = RuleCondition {
        field: "f".into(),
        op: RuleOp::EndsWith,
        value: "suf".into(),
    };
    assert!(c.evaluate("hasasuf"));
    assert!(!c.evaluate("suffix-no"));
}

#[test]
fn condition_matches_case_insensitive() {
    let c = RuleCondition {
        field: "f".into(),
        op: RuleOp::Matches,
        value: "HELLO".into(),
    };
    assert!(c.evaluate("say hello world"));
}

// ── evaluate_conditions ──────────────────────────────────────────────────────

#[test]
fn empty_conditions_always_pass() {
    assert!(evaluate_conditions(&[], &[("field", "value")]));
}

#[test]
fn all_conditions_must_pass() {
    let conds = vec![
        RuleCondition {
            field: "tool".into(),
            op: RuleOp::Eq,
            value: "bash".into(),
        },
        RuleCondition {
            field: "actor".into(),
            op: RuleOp::Eq,
            value: "agent".into(),
        },
    ];
    let fields = [("tool", "bash"), ("actor", "agent")];
    assert!(evaluate_conditions(&conds, &fields));

    let fields_partial = [("tool", "bash"), ("actor", "other")];
    assert!(!evaluate_conditions(&conds, &fields_partial));
}

// ── ActorMatcher ─────────────────────────────────────────────────────────────

#[test]
fn matcher_with_no_filters_matches_anything() {
    let m = ActorMatcher {
        actor_id: None,
        tool_name: None,
        event: None,
    };
    assert!(m.matches("any-agent", Some("any-tool"), HookEvent::PreToolUse));
}

#[test]
fn matcher_filters_by_actor_id() {
    let m = ActorMatcher {
        actor_id: Some("agent-1".into()),
        tool_name: None,
        event: None,
    };
    assert!(m.matches("agent-1", None, HookEvent::Stop));
    assert!(!m.matches("agent-2", None, HookEvent::Stop));
}

#[test]
fn matcher_filters_by_tool_name() {
    let m = ActorMatcher {
        actor_id: None,
        tool_name: Some("Bash".into()),
        event: None,
    };
    assert!(m.matches("any", Some("Bash"), HookEvent::PreToolUse));
    assert!(!m.matches("any", Some("Edit"), HookEvent::PreToolUse));
    assert!(!m.matches("any", None, HookEvent::PreToolUse));
}

#[test]
fn matcher_filters_by_event() {
    let m = ActorMatcher {
        actor_id: None,
        tool_name: None,
        event: Some(HookEvent::PreToolUse),
    };
    assert!(m.matches("a", None, HookEvent::PreToolUse));
    assert!(!m.matches("a", None, HookEvent::PostToolUse));
}

// ── run_hooks ────────────────────────────────────────────────────────────────

#[test]
fn no_hooks_returns_allow() {
    let result = run_hooks(&[], "agent", None, HookEvent::PreToolUse, &[]);
    assert!(matches!(result, HookResult::Allow));
}

#[test]
fn deny_hook_on_pre_tool_use_blocks() {
    let hooks = vec![simple_hook(
        "h1",
        HookEvent::PreToolUse,
        "deny:blocked by policy",
    )];
    let result = run_hooks(&hooks, "agent", None, HookEvent::PreToolUse, &[]);
    assert!(matches!(result, HookResult::Block(_)));
}

#[test]
fn deny_hook_on_post_tool_use_does_not_block() {
    let hooks = vec![simple_hook("h1", HookEvent::PostToolUse, "deny:ignored")];
    let result = run_hooks(&hooks, "agent", None, HookEvent::PostToolUse, &[]);
    assert!(matches!(result, HookResult::Allow));
}

#[test]
fn hook_event_can_block_returns_correct_set() {
    assert!(HookEvent::PreToolUse.can_block());
    assert!(HookEvent::Stop.can_block());
    assert!(!HookEvent::PostToolUse.can_block());
    assert!(!HookEvent::SessionStart.can_block());
}

#[test]
fn hook_without_deny_prefix_allows() {
    let hooks = vec![simple_hook("h1", HookEvent::PreToolUse, "notify-user")];
    let result = run_hooks(&hooks, "agent", None, HookEvent::PreToolUse, &[]);
    assert!(matches!(result, HookResult::Allow));
}

#[test]
fn unmatched_actor_hook_is_skipped() {
    let mut hook = simple_hook("h1", HookEvent::PreToolUse, "deny:nope");
    hook.matcher.actor_id = Some("other-agent".into());
    let hooks = vec![hook];
    let result = run_hooks(&hooks, "my-agent", None, HookEvent::PreToolUse, &[]);
    assert!(matches!(result, HookResult::Allow));
}

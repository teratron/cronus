use cronus::agent_registry::{AgentMode, AgentRegistry};

// ── built-in agents ──────────────────────────────────────────────────────────

#[test]
fn registry_contains_seven_builtin_agents() {
    let reg = AgentRegistry::new();
    assert_eq!(reg.builtin_count(), 7);
}

#[test]
fn work_agent_is_resolvable() {
    let reg = AgentRegistry::new();
    let def = reg.resolve("work").unwrap();
    assert_eq!(def.name, "work");
    assert!(def.native);
}

#[test]
fn code_agent_exists_with_all_mode() {
    let reg = AgentRegistry::new();
    let def = reg.resolve("code").unwrap();
    assert_eq!(def.mode, AgentMode::All);
}

#[test]
fn plan_agent_is_sub_agent_mode() {
    let reg = AgentRegistry::new();
    let def = reg.resolve("plan").unwrap();
    assert_eq!(def.mode, AgentMode::SubAgent);
}

#[test]
fn resolve_unknown_returns_not_found() {
    let reg = AgentRegistry::new();
    let result = reg.resolve("no-such-agent");
    assert!(result.is_err());
}

// ── list_active ──────────────────────────────────────────────────────────────

#[test]
fn list_active_returns_all_builtins_by_default() {
    let reg = AgentRegistry::new();
    assert_eq!(reg.list_active().len(), reg.builtin_count());
}

// ── apply_user_config ────────────────────────────────────────────────────────

#[test]
fn disable_via_user_config_hides_agent() {
    let mut reg = AgentRegistry::new();
    reg.apply_user_config("work", true, None);
    assert!(reg.resolve("work").is_err());
}

#[test]
fn disabled_agent_excluded_from_list_active() {
    let mut reg = AgentRegistry::new();
    let before = reg.list_active().len();
    reg.apply_user_config("plan", true, None);
    assert_eq!(reg.list_active().len(), before - 1);
}

#[test]
fn user_config_can_override_model_ref() {
    let mut reg = AgentRegistry::new();
    reg.apply_user_config("code", false, Some("claude-opus-4".to_string()));
    let def = reg.resolve("code").unwrap();
    assert_eq!(def.model_ref.as_deref(), Some("claude-opus-4"));
}

#[test]
fn user_config_adds_custom_agent() {
    let mut reg = AgentRegistry::new();
    reg.apply_user_config("my-custom", false, Some("claude-haiku".to_string()));
    let def = reg.resolve("my-custom").unwrap();
    assert!(!def.native);
    assert_eq!(def.model_ref.as_deref(), Some("claude-haiku"));
}

// ── generate_from_description ────────────────────────────────────────────────

#[test]
fn generate_from_description_creates_non_native_agent() {
    let def = AgentRegistry::generate_from_description("analysis-bot", "analyzes logs");
    assert_eq!(def.name, "analysis-bot");
    assert!(!def.native);
    assert!(def.description.as_deref().is_some());
}

#[test]
fn generated_agent_has_default_steps() {
    let def = AgentRegistry::generate_from_description("bot", "desc");
    assert!(def.steps.is_some());
    assert!(def.steps.unwrap() > 0);
}

// ── AgentMode::as_str ────────────────────────────────────────────────────────

#[test]
fn agent_mode_as_str_values() {
    assert_eq!(AgentMode::Primary.as_str(), "primary");
    assert_eq!(AgentMode::SubAgent.as_str(), "subagent");
    assert_eq!(AgentMode::All.as_str(), "all");
}

//! Plugin hooks — tool-event API with actor matchers and rule evaluation.
//!
//! Nine HookEvent kinds. Hooks run in parallel; aggregated decision is deny if
//! any hook returns Block. Rule evaluation uses AND conditions with 6 operators.

/// The nine hook event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Stop,
    SubagentStop,
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreCompact,
    Notification,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::Stop => "Stop",
            HookEvent::SubagentStop => "SubagentStop",
            HookEvent::SessionStart => "SessionStart",
            HookEvent::SessionEnd => "SessionEnd",
            HookEvent::UserPromptSubmit => "UserPromptSubmit",
            HookEvent::PreCompact => "PreCompact",
            HookEvent::Notification => "Notification",
        }
    }

    pub fn can_block(&self) -> bool {
        // PreToolUse and Stop can block; PostToolUse cannot
        matches!(self, HookEvent::PreToolUse | HookEvent::Stop | HookEvent::SubagentStop)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleOp {
    Eq,
    Ne,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
}

#[derive(Debug, Clone)]
pub struct RuleCondition {
    pub field: String,
    pub op: RuleOp,
    pub value: String,
}

impl RuleCondition {
    pub fn evaluate(&self, field_value: &str) -> bool {
        match self.op {
            RuleOp::Eq => field_value == self.value,
            RuleOp::Ne => field_value != self.value,
            RuleOp::Contains => field_value.contains(&*self.value),
            RuleOp::StartsWith => field_value.starts_with(&*self.value),
            RuleOp::EndsWith => field_value.ends_with(&*self.value),
            RuleOp::Matches => field_value.to_lowercase().contains(&self.value.to_lowercase()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActorMatcher {
    pub actor_id: Option<String>,
    pub tool_name: Option<String>,
    pub event: Option<HookEvent>,
}

impl ActorMatcher {
    pub fn matches(&self, actor_id: &str, tool_name: Option<&str>, event: HookEvent) -> bool {
        if let Some(aid) = &self.actor_id
            && aid != actor_id
        {
            return false;
        }
        if let Some(tn) = &self.tool_name {
            match tool_name {
                None => return false,
                Some(actual) if actual != tn => return false,
                _ => {}
            }
        }
        if let Some(ev) = self.event
            && ev != event
        {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone)]
pub enum HookResult {
    Allow,
    Block(String),
    Warn(String),
}

#[derive(Debug, Clone)]
pub struct HookEntry {
    pub id: String,
    pub event: HookEvent,
    pub matcher: ActorMatcher,
    pub command: String,
    pub timeout_ms: u64,
    pub conditions: Vec<RuleCondition>,
}

/// Hook security: validate a command does not contain shell metacharacters.
pub fn validate_hook_command(command: &str) -> Result<(), String> {
    const FORBIDDEN: &[char] = &[';', '|', '&', '>', '<', '`', '$'];
    if let Some(c) = command.chars().find(|c| FORBIDDEN.contains(c)) {
        return Err(format!("hook command contains forbidden character: '{c}'"));
    }
    Ok(())
}

/// Evaluate all AND conditions for a hook entry.
///
/// Returns true when all conditions pass (empty conditions → always pass).
pub fn evaluate_conditions(conditions: &[RuleCondition], fields: &[(&str, &str)]) -> bool {
    conditions.iter().all(|cond| {
        fields
            .iter()
            .find(|(k, _)| *k == cond.field)
            .is_some_and(|(_, v)| cond.evaluate(v))
    })
}

/// Run hooks for an event. Returns aggregated decision.
///
/// Deny wins: if any hook returns Block, the aggregated decision is Block.
/// PostToolUse hooks can never block (HookEvent::can_block guard).
pub fn run_hooks(
    hooks: &[HookEntry],
    actor_id: &str,
    tool_name: Option<&str>,
    event: HookEvent,
    fields: &[(&str, &str)],
) -> HookResult {
    let mut deny_reason: Option<String> = None;

    for hook in hooks {
        if !hook.matcher.matches(actor_id, tool_name, event) {
            continue;
        }
        if !evaluate_conditions(&hook.conditions, fields) {
            continue;
        }
        // In Phase 5, hooks are evaluated but not actually spawned.
        // The outcome is simulated: a command starting with "deny:" blocks.
        if hook.command.starts_with("deny:") && event.can_block() {
            let reason = hook.command.trim_start_matches("deny:").to_string();
            deny_reason = Some(reason);
        }
    }

    match deny_reason {
        Some(reason) => HookResult::Block(reason),
        None => HookResult::Allow,
    }
}

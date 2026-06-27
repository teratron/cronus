//! Transpiler — lossless conversion between the compact NODUS form and the
//! human-readable HUMAN form.
//!
//! Both directions route through the [`crate::ast`] node model, so a compact →
//! AST → compact round-trip is guaranteed to produce an AST that compares equal
//! to the original (the dual-representation invariant). The human form is not
//! re-parseable back to AST; it is a one-way readable rendering.
//!
//! Verb mapping and vocabulary are consumed from [`crate::vocab`] constants so
//! the transpiler stays in sync with the schema without duplicating data.

use crate::ast::{
    CommandCall, Conditional, ForLoop, MapBlock, RuleKind, Step, Stmt, SwitchBlock, UntilLoop,
    WorkflowFile,
};
use crate::vocab::TRANSPILER_VERB_MAP;

/// Converts a [`WorkflowFile`] AST between the compact NODUS and readable
/// HUMAN representations.
pub struct Transpiler;

impl Transpiler {
    // ─── NODUS → HUMAN ───────────────────────────────────────────────────────

    /// Render the AST as a plain-language HUMAN mode description.
    pub fn to_human(ast: &WorkflowFile) -> String {
        let mut lines: Vec<String> = Vec::new();

        if let Some(h) = &ast.header {
            lines.push(format!("WORKFLOW: {}", h.name));
        }
        lines.push(String::new());

        if !ast.triggers.is_empty() {
            let desc: Vec<String> = ast
                .triggers
                .iter()
                .map(|t| Self::humanize_trigger(&t.condition))
                .collect();
            lines.push(format!("TRIGGER: {}", desc.join(", ")));
            lines.push(String::new());
        }

        if let Some(input) = &ast.input_decl
            && !input.fields.is_empty()
        {
            let parts: Vec<String> = input
                .fields
                .iter()
                .map(|f| {
                    let mut s = f.name.clone();
                    if f.optional {
                        if let Some(d) = &f.default {
                            s.push_str(&format!(" (default: {d})"));
                        } else {
                            s.push_str(" (optional)");
                        }
                    }
                    s
                })
                .collect();
            lines.push(format!("INPUT: {}", parts.join(", ")));
        }

        if let Some(ctx) = &ast.context_decl
            && !ctx.contexts.is_empty()
        {
            lines.push(format!("CONTEXT: load {}", ctx.contexts.join(", ")));
        }

        lines.push(String::new());

        if !ast.rules.is_empty() || !ast.preferences.is_empty() {
            lines.push("RULES:".to_string());
            for rule in &ast.rules {
                let kind = match rule.kind {
                    Some(RuleKind::Never) => "NEVER",
                    Some(RuleKind::Always) => "ALWAYS",
                    None => "",
                };
                lines.push(format!(
                    "  - {} {}",
                    kind,
                    Self::humanize_rule(&rule.content)
                ));
            }
            for pref in &ast.preferences {
                let mut s = format!("Prefer {} over {}", pref.preferred, pref.over);
                if let Some(c) = &pref.condition {
                    s.push_str(&format!(" if {c}"));
                }
                lines.push(format!("  - {s}"));
            }
            lines.push(String::new());
        }

        if !ast.steps.is_empty() {
            lines.push("STEPS:".to_string());
            for step in &ast.steps {
                let desc = Self::humanize_step(step);
                if !desc.is_empty() {
                    lines.push(format!("  {}. {}", step.number, desc));
                }
            }
            lines.push(String::new());
        }

        if let Some(out) = &ast.output_decl {
            lines.push(format!("OUTPUT: {}", Self::humanize_var(&out.variable)));
        }

        if let Some(err) = &ast.error_decl {
            lines.push(format!("ON ERROR: {}", Self::humanize_error(&err.raw)));
        }

        for test in &ast.tests {
            lines.push(String::new());
            lines.push(format!("TEST CASE: {}", test.name));
            if !test.input.is_empty() {
                let inputs: Vec<String> = test
                    .input
                    .iter()
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect();
                lines.push(format!("  Inputs: {}", inputs.join(", ")));
            }
            if !test.expected.is_empty() {
                let asserts: Vec<String> = test
                    .expected
                    .iter()
                    .map(|(k, v)| format!("{} → {}", Self::humanize_var(k), v))
                    .collect();
                lines.push(format!("  Expects: {}", asserts.join(", ")));
            }
            if !test.tags.is_empty() {
                lines.push(format!("  Tags: {}", test.tags.join(", ")));
            }
        }

        lines.join("\n")
    }

    // ─── HUMAN → NODUS ───────────────────────────────────────────────────────

    /// Reconstruct the compact NODUS symbolic form from the AST.
    pub fn to_nodus(ast: &WorkflowFile) -> String {
        let mut lines: Vec<String> = Vec::new();

        if let Some(h) = &ast.header {
            if h.version.is_empty() {
                lines.push(format!("§wf:{}", h.name));
            } else {
                lines.push(format!("§wf:{} {}", h.name, h.version));
            }
        }

        if let Some(rt) = &ast.runtime {
            lines.push("§runtime: {".to_string());
            lines.push(format!("  core:    {}", rt.core));
            if !rt.extends.is_empty() {
                lines.push(format!("  extends: [{}]", rt.extends.join(", ")));
            }
            if !rt.agents.is_empty() {
                let parts: Vec<String> =
                    rt.agents.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                lines.push(format!("  agents:  {{ {} }}", parts.join(", ")));
            }
            lines.push(format!("  mode:    {}", rt.mode));
            lines.push("}".to_string());
        }

        for trigger in &ast.triggers {
            lines.push(format!(
                "@ON: {} \u{2192} {}",
                trigger.condition, trigger.action
            ));
        }

        lines.push(String::new());

        for rule in &ast.rules {
            let kind = match rule.kind {
                Some(RuleKind::Never) => "NEVER",
                Some(RuleKind::Always) => "ALWAYS",
                None => "",
            };
            lines.push(format!("!!{kind}: {}", rule.content));
        }
        for pref in &ast.preferences {
            let mut s = format!("!PREF: {} OVER {}", pref.preferred, pref.over);
            if let Some(c) = &pref.condition {
                s.push_str(&format!(" IF {c}"));
            }
            lines.push(s);
        }

        lines.push(String::new());

        if let Some(input) = &ast.input_decl {
            let fields: Vec<String> = input
                .fields
                .iter()
                .map(|f| {
                    let mut s = f.name.clone();
                    if f.optional {
                        s.push('?');
                    }
                    if !f.type_name.is_empty() && f.type_name != "any" {
                        s.push_str(&format!(": {}", f.type_name));
                    }
                    if let Some(d) = &f.default {
                        s.push_str(&format!(" = {d}"));
                    }
                    s
                })
                .collect();
            lines.push(format!("@in: {{ {} }}", fields.join(", ")));
        }

        if let Some(ctx) = &ast.context_decl {
            lines.push(format!("@ctx: [{}]", ctx.contexts.join(", ")));
        }

        if let Some(out) = &ast.output_decl {
            lines.push(format!("@out: {}", out.variable));
        }

        if let Some(err) = &ast.error_decl {
            lines.push(format!("@err: {}", err.raw));
        }

        lines.push(String::new());

        if !ast.steps.is_empty() {
            lines.push("@steps:".to_string());
            for step in &ast.steps {
                let line = Self::nodus_step(step);
                if !line.is_empty() {
                    lines.push(format!("  {}. {}", step.number, line));
                }
            }
        }

        for test in &ast.tests {
            lines.push(String::new());
            lines.push(format!("@test:{} {{", test.name));
            let has_structured =
                !test.input.is_empty() || !test.expected.is_empty() || !test.tags.is_empty();
            if has_structured {
                if !test.input.is_empty() {
                    lines.push("  input:".to_string());
                    for (k, v) in &test.input {
                        lines.push(format!("    {k}: {v}"));
                    }
                }
                if !test.expected.is_empty() {
                    lines.push("  expected:".to_string());
                    for (k, v) in &test.expected {
                        lines.push(format!("    {k}: {v}"));
                    }
                }
                if !test.tags.is_empty() {
                    lines.push(format!("  tags: [{}]", test.tags.join(", ")));
                }
            } else if !test.raw_lines.is_empty() {
                // Backward compat: emit raw token values joined as a single body line
                lines.push(format!("  {}", test.raw_lines.join(" ")));
            }
            lines.push("}".to_string());
        }

        lines.join("\n")
    }

    // ─── Humanizers ──────────────────────────────────────────────────────────

    fn humanize_trigger(condition: &str) -> String {
        let cond = condition.trim();
        if cond.starts_with("new_") {
            return format!("when a {} is received", cond.replace('_', " "));
        }
        if cond.contains("schedule:") {
            let time = cond
                .split_once("schedule:")
                .map(|x| x.1)
                .unwrap_or("")
                .trim();
            return format!("every day at {time}");
        }
        if cond.contains("webhook:") {
            let hook = cond
                .split_once("webhook:")
                .map(|x| x.1)
                .unwrap_or("")
                .trim();
            return format!("when {} webhook fires", hook.replace('_', " "));
        }
        if cond.contains("CONTAINS") {
            return format!("when {}", cond.to_lowercase());
        }
        format!("when {cond}")
    }

    fn humanize_rule(content: &str) -> String {
        content
            .to_lowercase()
            .replace("$out", "the output")
            .replace("$error.level", "error level")
    }

    fn humanize_step(step: &Step) -> String {
        if !step.comment.is_empty() {
            let text = step.comment.trim_start_matches(';').trim();
            let text = text.trim_start_matches('\u{2014}').trim();
            return text.to_string();
        }
        match &step.body {
            Some(Stmt::Command(cmd)) => Self::humanize_command(cmd),
            Some(Stmt::Conditional(cond)) => Self::humanize_conditional(cond),
            Some(Stmt::ForLoop(fl)) => Self::humanize_for(fl),
            Some(Stmt::UntilLoop(ul)) => Self::humanize_until(ul),
            Some(Stmt::Parallel(_)) => "Run the following in parallel".to_string(),
            Some(Stmt::Switch(sw)) => Self::humanize_switch(sw),
            Some(Stmt::Map(mb)) => Self::humanize_map(mb),
            Some(Stmt::Comment(c)) => c.text.trim_start_matches(';').trim().to_string(),
            Some(Stmt::VarRef(v)) => v.name.clone(),
            None => String::new(),
        }
    }

    fn humanize_map(mb: &MapBlock) -> String {
        let mut s = format!(
            "Map {} over {}",
            Self::humanize_command(&mb.command),
            Self::humanize_var(&mb.collection)
        );
        if let Some(target) = &mb.target {
            s.push_str(&format!(
                " \u{2192} store as {}",
                Self::humanize_var(target)
            ));
        }
        s
    }

    fn humanize_switch(sw: &SwitchBlock) -> String {
        let mut s = format!("Switch on {}", Self::humanize_var(&sw.scrutinee));
        for (value, action) in &sw.arms {
            s.push_str(&format!("; {value} → {}", Self::humanize_command(action)));
        }
        if let Some(default) = &sw.default {
            s.push_str(&format!(
                "; otherwise → {}",
                Self::humanize_command(default)
            ));
        }
        s
    }

    fn humanize_command(cmd: &CommandCall) -> String {
        let verb = TRANSPILER_VERB_MAP
            .iter()
            .find(|(k, _)| *k == cmd.name)
            .map(|(_, v)| *v)
            .unwrap_or(cmd.name.as_str());

        let args = cmd.args.join(", ");
        let mut desc = if args.is_empty() {
            verb.to_string()
        } else {
            format!("{verb} {args}")
        };

        if !cmd.flags.is_empty() {
            desc.push_str(&format!(" (extract: {})", cmd.flags.join(", ")));
        }
        if !cmd.validators.is_empty() {
            let rules: Vec<&str> = cmd
                .validators
                .iter()
                .map(|v| v.trim_start_matches('^'))
                .collect();
            desc.push_str(&format!(" against rules: {}", rules.join(", ")));
        }
        if let Some(target) = &cmd.pipeline_target {
            desc.push_str(&format!(
                " \u{2192} store as {}",
                Self::humanize_var(target)
            ));
        }
        desc
    }

    fn humanize_conditional(cond: &Conditional) -> String {
        let mut s = format!("IF {}", cond.condition);
        if let Some(action) = &cond.action {
            s.push_str(&format!(" \u{2192} {}", Self::humanize_command(action)));
        }
        if cond.break_flag {
            s.push_str(", STOP");
        }
        if cond.halt_flag {
            s.push_str(", HALT");
        }
        if cond.pause_flag {
            s.push_str(", PAUSE");
        }
        s
    }

    fn humanize_for(fl: &ForLoop) -> String {
        format!(
            "For each {} in {}",
            Self::humanize_var(&fl.variable),
            Self::humanize_var(&fl.collection)
        )
    }

    fn humanize_until(ul: &UntilLoop) -> String {
        let mut s = format!("Repeat until {}", ul.condition);
        if let Some(max) = ul.max_iterations {
            s.push_str(&format!(" (max {max} attempts)"));
        }
        s
    }

    fn humanize_var(var: &str) -> String {
        if var.is_empty() {
            return String::new();
        }
        var.trim_start_matches('$').replace('.', " \u{2192} ")
    }

    fn humanize_error(raw: &str) -> String {
        if raw.contains("ESCALATE") && raw.to_lowercase().contains("human") {
            return "escalate to human".to_string();
        }
        raw.to_string()
    }

    // ─── NODUS reconstructors ─────────────────────────────────────────────────

    fn nodus_step(step: &Step) -> String {
        match &step.body {
            Some(Stmt::Command(cmd)) => Self::nodus_command(cmd),
            Some(Stmt::Comment(c)) => format!(";; {}", c.text),
            _ => {
                if !step.comment.is_empty() {
                    format!(";; {}", step.comment)
                } else {
                    String::new()
                }
            }
        }
    }

    fn nodus_command(cmd: &CommandCall) -> String {
        let mut parts = vec![format!("{}({})", cmd.name, cmd.args.join(", "))];
        for (mod_name, mod_val) in &cmd.modifiers {
            if !mod_val.is_empty() {
                parts.push(format!("{mod_name}={mod_val}"));
            } else {
                parts.push(mod_name.clone());
            }
        }
        for v in &cmd.validators {
            parts.push(v.clone());
        }
        for f in &cmd.flags {
            parts.push(format!("~{f}"));
        }
        if let Some(target) = &cmd.pipeline_target {
            parts.push(format!("\u{2192} {target}"));
        }
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        AbsoluteRule, CommandCall, ContextDecl, ErrorDecl, FileHeader, FileType, InputDecl,
        InputField, OutputDecl, Preference, RuleKind, Step, Stmt, Trigger, WorkflowFile,
    };
    use crate::parser::Parser;

    fn minimal_wf() -> WorkflowFile {
        WorkflowFile {
            header: Some(FileHeader {
                file_type: FileType::Workflow,
                name: "greet".to_string(),
                version: "v1.0".to_string(),
            }),
            triggers: vec![Trigger {
                condition: "new_message".to_string(),
                action: "handle".to_string(),
            }],
            rules: vec![AbsoluteRule {
                kind: Some(RuleKind::Never),
                content: "publish WITHOUT validate".to_string(),
            }],
            preferences: vec![Preference {
                preferred: "brief".to_string(),
                over: "verbose".to_string(),
                condition: Some("$user.is_vip".to_string()),
            }],
            input_decl: Some(InputDecl {
                fields: vec![
                    InputField {
                        name: "query".to_string(),
                        ..Default::default()
                    },
                    InputField {
                        name: "tone".to_string(),
                        type_name: "string".to_string(),
                        optional: true,
                        default: Some("neutral".to_string()),
                        ..Default::default()
                    },
                ],
            }),
            context_decl: Some(ContextDecl {
                contexts: vec!["user_profile".to_string()],
            }),
            output_decl: Some(OutputDecl {
                variable: "$out".to_string(),
            }),
            error_decl: Some(ErrorDecl {
                raw: "ESCALATE(human)".to_string(),
                handler: None,
            }),
            steps: vec![
                Step {
                    number: 1,
                    body: Some(Stmt::Command(CommandCall {
                        name: "ANALYZE".to_string(),
                        args: vec!["$in.query".to_string()],
                        flags: vec!["sentiment".to_string()],
                        pipeline_target: Some("$sentiment".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                Step {
                    number: 2,
                    body: Some(Stmt::Command(CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["reply".to_string()],
                        modifiers: vec![("+tone".to_string(), "warm".to_string())],
                        validators: vec!["^len:280".to_string()],
                        pipeline_target: Some("$out".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn nodus_command_reconstruction() {
        let cmd = CommandCall {
            name: "GEN".to_string(),
            args: vec!["reply".to_string()],
            modifiers: vec![("+tone".to_string(), "warm".to_string())],
            validators: vec!["^len:280".to_string()],
            flags: vec!["extract_key".to_string()],
            pipeline_target: Some("$draft".to_string()),
        };
        let out = Transpiler::nodus_command(&cmd);
        assert_eq!(
            out,
            "GEN(reply) +tone=warm ^len:280 ~extract_key \u{2192} $draft"
        );
    }

    #[test]
    fn nodus_command_no_args() {
        let cmd = CommandCall {
            name: "LOG".to_string(),
            ..Default::default()
        };
        assert_eq!(Transpiler::nodus_command(&cmd), "LOG()");
    }

    #[test]
    fn human_command_verb_map() {
        let gen_cmd = CommandCall {
            name: "GEN".to_string(),
            args: vec!["summary".to_string()],
            ..Default::default()
        };
        assert!(Transpiler::humanize_command(&gen_cmd).starts_with("Generate summary"));

        let fetch = CommandCall {
            name: "FETCH".to_string(),
            args: vec!["$url".to_string()],
            pipeline_target: Some("$raw".to_string()),
            ..Default::default()
        };
        let out = Transpiler::humanize_command(&fetch);
        assert!(out.starts_with("Fetch $url"));
        assert!(out.contains("store as raw"));

        // Unknown command falls back to its own name.
        let unknown = CommandCall {
            name: "CUSTOM_OP".to_string(),
            ..Default::default()
        };
        assert!(Transpiler::humanize_command(&unknown).starts_with("CUSTOM_OP"));
    }

    #[test]
    fn human_command_with_flags_and_validators() {
        let cmd = CommandCall {
            name: "ANALYZE".to_string(),
            args: vec!["$text".to_string()],
            flags: vec!["sentiment".to_string(), "tone".to_string()],
            validators: vec!["^len:500".to_string()],
            ..Default::default()
        };
        let out = Transpiler::humanize_command(&cmd);
        assert!(out.contains("(extract: sentiment, tone)"), "got: {out}");
        assert!(out.contains("against rules: len:500"), "got: {out}");
    }

    #[test]
    fn human_trigger_patterns() {
        assert_eq!(
            Transpiler::humanize_trigger("new_mention"),
            "when a new mention is received"
        );
        assert_eq!(
            Transpiler::humanize_trigger("schedule:08:00"),
            "every day at 08:00"
        );
        assert_eq!(
            Transpiler::humanize_trigger("webhook:order_placed"),
            "when order placed webhook fires"
        );
        assert_eq!(
            Transpiler::humanize_trigger("$in CONTAINS urgent"),
            "when $in contains urgent"
        );
        assert_eq!(
            Transpiler::humanize_trigger("user_request"),
            "when user_request"
        );
    }

    #[test]
    fn to_human_sections_present() {
        let wf = minimal_wf();
        let out = Transpiler::to_human(&wf);

        assert!(out.contains("WORKFLOW: greet"), "header: {out}");
        assert!(out.contains("TRIGGER:"), "trigger: {out}");
        assert!(out.contains("INPUT:"), "input: {out}");
        assert!(out.contains("CONTEXT: load user_profile"), "ctx: {out}");
        assert!(out.contains("RULES:"), "rules: {out}");
        assert!(out.contains("NEVER"), "rule kind: {out}");
        assert!(out.contains("Prefer brief over verbose"), "pref: {out}");
        assert!(out.contains("STEPS:"), "steps: {out}");
        assert!(out.contains("1."), "step 1: {out}");
        assert!(out.contains("2."), "step 2: {out}");
        assert!(out.contains("OUTPUT:"), "output: {out}");
        assert!(out.contains("ON ERROR:"), "error: {out}");
    }

    #[test]
    fn to_nodus_sections_present() {
        let wf = minimal_wf();
        let out = Transpiler::to_nodus(&wf);

        assert!(out.contains("§wf:greet v1.0"), "header: {out}");
        assert!(out.contains("@ON:"), "trigger: {out}");
        assert!(out.contains("!!NEVER:"), "rule: {out}");
        assert!(out.contains("!PREF:"), "pref: {out}");
        assert!(out.contains("@in:"), "in: {out}");
        assert!(out.contains("@ctx:"), "ctx: {out}");
        assert!(out.contains("@out: $out"), "out: {out}");
        assert!(out.contains("@err: ESCALATE(human)"), "err: {out}");
        assert!(out.contains("@steps:"), "steps: {out}");
        assert!(out.contains("1. ANALYZE"), "step 1: {out}");
        assert!(out.contains("~sentiment"), "flag: {out}");
        assert!(out.contains("2. GEN"), "step 2: {out}");
        assert!(out.contains("+tone=warm"), "modifier: {out}");
        assert!(out.contains("^len:280"), "validator: {out}");
    }

    #[test]
    fn round_trip_ast_equality() {
        // Parse a compact workflow to AST₁.
        let source = "\
§wf:rt_test v1.0
@ON: new_request → handle
!!NEVER: publish WITHOUT validate
!PREF: brief OVER verbose IF $user.is_vip
@in: { query, tone?: string = neutral }
@ctx: [user_profile]
@out: $out
@err: ESCALATE(human)
@steps:
  1. ANALYZE($in.query) ~sentiment → $sentiment
  2. GEN(reply) +tone=warm ^len:280 → $out
";
        let ast1 = Parser::parse(source).expect("parse source");

        // Reconstruct NODUS from AST₁.
        let nodus2 = Transpiler::to_nodus(&ast1);

        // Re-parse to AST₂ and compare.
        let ast2 = Parser::parse(&nodus2).expect("re-parse reconstructed NODUS");
        assert_eq!(ast1, ast2, "ASTs differ after round-trip:\n{nodus2}");
    }

    #[test]
    fn humanize_var_strips_dollar_and_replaces_dot() {
        assert_eq!(Transpiler::humanize_var("$out.data"), "out \u{2192} data");
        assert_eq!(Transpiler::humanize_var("$draft"), "draft");
        assert_eq!(Transpiler::humanize_var(""), "");
    }

    #[test]
    fn humanize_map_renders_collection_and_target() {
        use crate::ast::MapBlock;
        let mb = MapBlock {
            collection: "$items".to_string(),
            command: CommandCall {
                name: "GEN".to_string(),
                args: vec!["$it".to_string()],
                ..Default::default()
            },
            target: Some("$out".to_string()),
        };
        let human = Transpiler::humanize_map(&mb);
        assert!(human.contains("Map"), "got: {human}");
        assert!(human.contains("over items"), "got: {human}");
        assert!(human.contains("store as out"), "got: {human}");
    }

    #[test]
    fn humanize_switch_renders_arms_and_default() {
        use crate::ast::SwitchBlock;
        let sw = SwitchBlock {
            scrutinee: "$category".to_string(),
            arms: vec![(
                "urgent".to_string(),
                CommandCall {
                    name: "ROUTE".to_string(),
                    ..Default::default()
                },
            )],
            default: Some(CommandCall {
                name: "LOG".to_string(),
                ..Default::default()
            }),
        };
        let human = Transpiler::humanize_switch(&sw);
        assert!(human.contains("Switch on category"), "got: {human}");
        assert!(human.contains("urgent"), "got: {human}");
        assert!(human.contains("otherwise"), "got: {human}");
    }

    #[test]
    fn humanize_conditional_renders_halt_and_pause() {
        use crate::ast::Conditional;
        let halt = Conditional {
            condition: "$r > 0.9".to_string(),
            halt_flag: true,
            ..Default::default()
        };
        assert!(
            Transpiler::humanize_conditional(&halt).contains("HALT"),
            "human form must render !HALT"
        );
        let pause = Conditional {
            condition: "$r > 0.5".to_string(),
            pause_flag: true,
            ..Default::default()
        };
        assert!(
            Transpiler::humanize_conditional(&pause).contains("PAUSE"),
            "human form must render !PAUSE"
        );
    }

    #[test]
    fn humanize_error_escalate() {
        assert_eq!(
            Transpiler::humanize_error("ESCALATE(human)"),
            "escalate to human"
        );
        assert_eq!(Transpiler::humanize_error("LOG($error)"), "LOG($error)");
    }
}

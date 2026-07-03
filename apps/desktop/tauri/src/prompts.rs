//! Per-provider system prompt dispatch and the XML environment context.
//!
//! Providers interpret instruction format and tone differently, so the
//! system prompt is a provider-keyed variant behind one dispatch function —
//! the single decision point. The environment block is an XML envelope with
//! stable machine-parseable landmarks, emitted once at session start and
//! kept byte-stable across turns (KV-cache stability).

/// Model-family refinements inside a provider.
fn is_o_series(model_id: &str) -> bool {
    let mut chars = model_id.chars();
    chars.next() == Some('o') && chars.next().is_some_and(|c| c.is_ascii_digit())
}

fn is_codex(model_id: &str) -> bool {
    model_id.contains("codex")
}

fn is_kimi(model_id: &str) -> bool {
    model_id.contains("kimi")
}

/// The single dispatch point: resolve the provider-specific system prompt
/// variant. Variants encode provider quirks (delimiter style, persona
/// framing, tool-calling guidance); model families branch within a provider.
pub fn build_system_prompt(provider_id: &str, model_id: &str) -> &'static str {
    match provider_id {
        "anthropic" => ANTHROPIC_PROMPT,
        "openai" if is_o_series(model_id) => O_SERIES_PROMPT,
        "openai" if is_codex(model_id) => CODEX_PROMPT,
        "openai" => GPT_PROMPT,
        "google" => GEMINI_PROMPT,
        "openrouter" if is_kimi(model_id) => KIMI_PROMPT,
        _ => DEFAULT_PROMPT,
    }
}

// Variant bodies live here as distinct constants; the real persona text is
// assembled by the core session layer — these carry the provider framing.
const ANTHROPIC_PROMPT: &str =
    "You are a Cronus office agent. Use XML-tagged sections for structured content.";
const O_SERIES_PROMPT: &str =
    "You are a Cronus office agent. Reason step by step internally; answer concisely.";
const CODEX_PROMPT: &str =
    "You are a Cronus office agent focused on code. Prefer diffs and file paths.";
const GPT_PROMPT: &str =
    "You are a Cronus office agent. Use markdown sections; keep instructions imperative.";
const GEMINI_PROMPT: &str =
    "You are a Cronus office agent. Keep responses grounded; cite tool results explicitly.";
const KIMI_PROMPT: &str =
    "You are a Cronus office agent. Keep outputs compact; avoid speculative content.";
const DEFAULT_PROMPT: &str =
    "You are a Cronus office agent. Follow the task instructions precisely.";

/// Escape a text node for XML (structural landmarks stay parseable even
/// when paths or descriptions carry special characters).
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The environment snapshot captured once at session start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvContext {
    pub working_directory: String,
    /// Present only when execution happens in a worktree distinct from the
    /// working directory.
    pub worktree: Option<String>,
    pub git_branch: String,
    pub git_clean: bool,
    pub platform: String,
    /// Session-start instant, ISO-8601; never updated mid-session.
    pub date: String,
    pub model_id: String,
    pub provider_id: String,
}

impl EnvContext {
    /// Render the `<env>` envelope. Deterministic: the same context always
    /// yields the same bytes (KV-cache stability).
    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<env>\n");
        xml.push_str(&format!(
            "  <working_directory>{}</working_directory>\n",
            xml_escape(&self.working_directory)
        ));
        if let Some(worktree) = &self.worktree {
            xml.push_str(&format!(
                "  <worktree>{}</worktree>\n",
                xml_escape(worktree)
            ));
        }
        xml.push_str("  <git_status>\n");
        xml.push_str(&format!(
            "    <branch>{}</branch>\n",
            xml_escape(&self.git_branch)
        ));
        xml.push_str(&format!("    <clean>{}</clean>\n", self.git_clean));
        xml.push_str("  </git_status>\n");
        xml.push_str(&format!(
            "  <platform>{}</platform>\n",
            xml_escape(&self.platform)
        ));
        xml.push_str(&format!("  <date>{}</date>\n", xml_escape(&self.date)));
        xml.push_str("  <model>\n");
        xml.push_str(&format!("    <id>{}</id>\n", xml_escape(&self.model_id)));
        xml.push_str(&format!(
            "    <provider>{}</provider>\n",
            xml_escape(&self.provider_id)
        ));
        xml.push_str("  </model>\n");
        xml.push_str("</env>");
        xml
    }
}

/// One active reference directory surfaced to the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub name: String,
    pub path: String,
    pub description: String,
}

/// Render the `<available_references>` block: one `<reference>` per entry.
pub fn references_xml(references: &[Reference]) -> String {
    let mut xml = String::from("<available_references>\n");
    for reference in references {
        xml.push_str("  <reference>\n");
        xml.push_str(&format!(
            "    <name>{}</name>\n",
            xml_escape(&reference.name)
        ));
        xml.push_str(&format!(
            "    <path>{}</path>\n",
            xml_escape(&reference.path)
        ));
        xml.push_str(&format!(
            "    <description>{}</description>\n",
            xml_escape(&reference.description)
        ));
        xml.push_str("  </reference>\n");
    }
    xml.push_str("</available_references>");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_key_selects_the_matching_variant() {
        assert_eq!(
            build_system_prompt("anthropic", "claude-sonnet-4-6"),
            ANTHROPIC_PROMPT
        );
        assert_eq!(build_system_prompt("openai", "o3-mini"), O_SERIES_PROMPT);
        assert_eq!(build_system_prompt("openai", "gpt-5-codex"), CODEX_PROMPT);
        assert_eq!(build_system_prompt("openai", "gpt-4.1"), GPT_PROMPT);
        assert_eq!(
            build_system_prompt("google", "gemini-2.5-pro"),
            GEMINI_PROMPT
        );
        assert_eq!(
            build_system_prompt("openrouter", "moonshot/kimi-k2"),
            KIMI_PROMPT
        );
        assert_eq!(
            build_system_prompt("unknown-provider", "some-model"),
            DEFAULT_PROMPT
        );
    }

    #[test]
    fn model_family_branches_stay_inside_their_provider() {
        // A kimi model on a non-openrouter provider does not leak the kimi variant.
        assert_eq!(build_system_prompt("openai", "kimi-like"), GPT_PROMPT);
        // "openai" without an o-digit prefix is not o-series.
        assert_eq!(build_system_prompt("openai", "omega"), GPT_PROMPT);
    }

    #[test]
    fn env_block_has_the_expected_structure() {
        let ctx = EnvContext {
            working_directory: "C:/Projects/cronus".into(),
            worktree: Some("C:/Projects/cronus-wt".into()),
            git_branch: "main".into(),
            git_clean: true,
            platform: "Windows 10 / win32 x64".into(),
            date: "2026-07-02T19:30:00Z".into(),
            model_id: "claude-sonnet-4-6".into(),
            provider_id: "anthropic".into(),
        };
        let xml = ctx.to_xml();
        assert!(xml.starts_with("<env>"));
        assert!(xml.ends_with("</env>"));
        for tag in [
            "<working_directory>C:/Projects/cronus</working_directory>",
            "<worktree>C:/Projects/cronus-wt</worktree>",
            "<branch>main</branch>",
            "<clean>true</clean>",
            "<platform>Windows 10 / win32 x64</platform>",
            "<date>2026-07-02T19:30:00Z</date>",
            "<id>claude-sonnet-4-6</id>",
            "<provider>anthropic</provider>",
        ] {
            assert!(xml.contains(tag), "missing landmark: {tag}");
        }
    }

    #[test]
    fn env_block_is_deterministic_and_omits_an_absent_worktree() {
        let ctx = EnvContext {
            working_directory: "/p".into(),
            worktree: None,
            git_branch: "dev".into(),
            git_clean: false,
            platform: "linux".into(),
            date: "2026-07-02T00:00:00Z".into(),
            model_id: "m".into(),
            provider_id: "p".into(),
        };
        assert_eq!(ctx.to_xml(), ctx.to_xml(), "byte-stable for KV-cache");
        assert!(!ctx.to_xml().contains("<worktree>"));
        assert!(ctx.to_xml().contains("<clean>false</clean>"));
    }

    #[test]
    fn references_block_renders_one_entry_per_reference_and_escapes() {
        let refs = vec![
            Reference {
                name: "project-root".into(),
                path: "/path/to/project".into(),
                description: "Primary workspace".into(),
            },
            Reference {
                name: "docs & specs".into(),
                path: "/p/<docs>".into(),
                description: "Reference <material>".into(),
            },
        ];
        let xml = references_xml(&refs);
        assert_eq!(xml.matches("<reference>").count(), 2);
        assert!(xml.contains("<name>project-root</name>"));
        assert!(xml.contains("docs &amp; specs"));
        assert!(xml.contains("/p/&lt;docs&gt;"));
        assert!(xml.starts_with("<available_references>"));
        assert!(xml.ends_with("</available_references>"));
    }
}

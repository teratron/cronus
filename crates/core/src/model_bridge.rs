//! The nodus↔`InferenceBackend` bridge (l1-model-runtime §4.1, MR-2).
//!
//! `nodus` is a zero-dependency workflow runtime whose model-backed steps
//! (`GEN`, `ANALYZE`) drive its own `ModelProvider` trait — a minimal
//! synchronous surface (`generate(prompt) -> String`, `analyze`). The
//! transport realizes `contract::InferenceBackend` (a streaming call
//! surface). This bridge, living in the facade so nodus stays
//! dependency-free (LP-1), satisfies `nodus::ModelProvider` by collapsing an
//! `InferenceBackend` stream into the `String` nodus expects — the one place
//! the two provider vocabularies meet.
//!
//! It replaces `nodus`'s built-in `StubProvider` (the `[STUB gen(...)]`
//! label) with real generation once a backend is wired.

use std::sync::Arc;

use cronus_contract::{CancelHandle, GenerateRequest, InferenceBackend, StreamEvent};
use nodus::{ModelProvider, Value};

/// Adapts a `contract::InferenceBackend` to `nodus::ModelProvider`.
///
/// Holds the backend behind an `Arc` (cheap to share and to move into the
/// executor) plus the model name each call targets.
pub struct NodusModelBridge {
    backend: Arc<dyn InferenceBackend>,
    model: String,
}

impl NodusModelBridge {
    pub fn new(backend: Arc<dyn InferenceBackend>, model: impl Into<String>) -> Self {
        NodusModelBridge {
            backend,
            model: model.into(),
        }
    }

    /// Drive one generation to completion, concatenating token text.
    ///
    /// `nodus::ModelProvider::generate` has no error channel, so a transport
    /// error ends collection and returns whatever text arrived before it
    /// (possibly empty) — the honest projection available within the trait's
    /// `-> String` shape; non-text events (tool calls, usage) are not folded
    /// into the generated text.
    fn collect(&self, prompt: &str, parameters: Vec<(String, String)>) -> String {
        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            parameters,
        };
        let mut out = String::new();
        for event in self.backend.generate_stream(request, CancelHandle::new()) {
            match event {
                StreamEvent::Token(t) => out.push_str(&t),
                StreamEvent::Done | StreamEvent::Error(_) => break,
                StreamEvent::ToolCall { .. } | StreamEvent::Usage { .. } => {}
            }
        }
        out
    }
}

impl ModelProvider for NodusModelBridge {
    fn model_id(&self) -> &str {
        &self.model
    }

    fn generate(&self, prompt: &str, modifiers: &[(String, String)]) -> String {
        self.collect(prompt, modifiers.to_vec())
    }

    fn analyze(&self, text: &str, flags: &[String]) -> Value {
        // Realize `analyze` over the one call surface the backend exposes
        // (generation): ask for a JSON verdict, parse it, and project each
        // requested flag. A flag the model did not answer stays `Null` —
        // never a fabricated score (contrast the stub's constant 0.9).
        let raw = self.collect(&build_analysis_prompt(text, flags), Vec::new());
        parse_analysis(&raw, flags)
    }
}

fn build_analysis_prompt(text: &str, flags: &[String]) -> String {
    let flag_list = flags.join(", ");
    format!(
        "Analyze the following text and respond with a single JSON object whose keys are \
         exactly [{flag_list}]. Each value is a number in 0.0..1.0 for a score, or a short \
         string for a label. Respond with only the JSON object.\n\nText:\n{text}"
    )
}

/// Parse a model's analysis response into a `nodus::Value::Map`, one entry
/// per requested flag in order. Robust to prose or code-fence wrapping: the
/// first `{`..last `}` span is parsed as JSON. Any flag absent from the
/// parsed object — or a wholly unparseable response — yields `Value::Null`
/// for that flag, so the map never invents a verdict.
fn parse_analysis(raw: &str, flags: &[String]) -> Value {
    let json =
        extract_json_object(raw).and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
    let entries = flags
        .iter()
        .map(|flag| {
            let value = json
                .as_ref()
                .and_then(|j| j.get(flag))
                .map(json_to_nodus)
                .unwrap_or(Value::Null);
            (flag.clone(), value)
        })
        .collect();
    Value::Map(entries)
}

/// Return the `{`..`}` substring (inclusive) of the first JSON object in
/// `raw`, or `None` if there is no balanced-looking object.
fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end > start {
        Some(&raw[start..=end])
    } else {
        None
    }
}

fn json_to_nodus(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Text(s.clone()),
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cronus_contract::{
        GenerateRequest, InferenceError, ModelDescriptor, PullProgress, ResidencyHint,
    };

    /// A test backend that yields a fixed script of tokens for every
    /// generation — enough to exercise the bridge without a network.
    struct ScriptedBackend {
        tokens: Vec<&'static str>,
    }

    impl InferenceBackend for ScriptedBackend {
        fn generate_stream(
            &self,
            _request: GenerateRequest,
            _cancel: CancelHandle,
        ) -> Box<dyn Iterator<Item = StreamEvent> + Send> {
            let mut events: Vec<StreamEvent> = self
                .tokens
                .iter()
                .map(|t| StreamEvent::Token(t.to_string()))
                .collect();
            events.push(StreamEvent::Done);
            Box::new(events.into_iter())
        }

        fn embed(&self, _model: &str, _input: &str) -> Result<Vec<f32>, InferenceError> {
            Err(InferenceError::Unsupported)
        }

        fn describe(&self, model: &str) -> Result<ModelDescriptor, InferenceError> {
            Ok(ModelDescriptor {
                name: model.to_string(),
                ..Default::default()
            })
        }

        fn pull(&self, _model: &str) -> Box<dyn Iterator<Item = PullProgress> + Send> {
            Box::new(std::iter::once(PullProgress::Done { digest: None }))
        }

        fn set_residency(&self, _model: &str, _hint: ResidencyHint) -> Result<(), InferenceError> {
            Ok(())
        }
    }

    fn bridge(tokens: Vec<&'static str>) -> NodusModelBridge {
        NodusModelBridge::new(Arc::new(ScriptedBackend { tokens }), "test-model")
    }

    #[test]
    fn nodus_bridge_generate_concatenates_the_stream() {
        let b = bridge(vec!["Hello", ", ", "world"]);
        assert_eq!(b.generate("greet", &[]), "Hello, world");
        assert_eq!(b.model_id(), "test-model");
    }

    #[test]
    fn nodus_bridge_analyze_returns_a_real_flag_map_from_the_model() {
        // The backend "returns" a JSON verdict; the bridge projects the
        // requested flags — a real map, not the stub's constant 0.9.
        let b = bridge(vec![r#"{"intent": "buy", "urgency": 0.8}"#]);
        let flags = vec!["intent".to_string(), "urgency".to_string()];
        let result = b.analyze("I want this now", &flags);
        assert_eq!(
            result,
            Value::Map(vec![
                ("intent".to_string(), Value::Text("buy".to_string())),
                ("urgency".to_string(), Value::Float(0.8)),
            ])
        );
    }

    #[test]
    fn nodus_bridge_analyze_leaves_unanswered_flags_null_never_fabricated() {
        // The model answered only "intent"; "toxicity" was not in the
        // response, so it stays Null rather than getting a made-up score.
        let b = bridge(vec![r#"here you go: {"intent": "browse"} — done"#]);
        let flags = vec!["intent".to_string(), "toxicity".to_string()];
        let result = b.analyze("just looking", &flags);
        assert_eq!(
            result,
            Value::Map(vec![
                ("intent".to_string(), Value::Text("browse".to_string())),
                ("toxicity".to_string(), Value::Null),
            ])
        );
    }

    #[test]
    fn nodus_bridge_drives_a_real_nodus_gen_step() {
        // End-to-end: a nodus workflow with a GEN step, run with the bridge
        // as its ModelProvider, writes the concatenated stream into `$out` —
        // proving the bridge replaces the built-in stub in the real runtime.
        let source = r#"
§wf:greet v1.0
§runtime: { core: schema.nodus }
@in:  { name: text }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.name) → $out
"#;
        let input = Value::Map(vec![("name".to_string(), Value::Text("Ada".to_string()))]);
        let result = nodus::run_with_provider(
            source,
            "greet.nodus",
            Some(input),
            bridge(vec!["Hi ", "there"]),
        )
        .expect("workflow runs");

        assert_eq!(result.status, nodus::Status::Ok);
        assert_eq!(
            result.vars.get("out"),
            Some(&Value::Text("Hi there".to_string()))
        );
        // And crucially NOT the stub label.
        assert_ne!(
            result.vars.get("out"),
            Some(&Value::Text("[STUB gen(Ada) tone=brand]".to_string()))
        );
    }
}

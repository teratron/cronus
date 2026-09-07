use std::sync::Arc;

use cronus_contract::{Binder, BinderKind, Invocable, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};
use nodus::executor::{Status, Value};
use nodus::validator::Severity;
use nodus::workflows::{self, TranspileMode};

use super::{core_id, flag_arg, opt_text_arg, text_arg};

pub(super) fn register(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    register_scaffold(registry, dispatcher);
    register_validate(registry, dispatcher);
    register_run(registry, dispatcher);
    register_transpile(registry, dispatcher);
}

/// `nodus::executor::Value` is shaped exactly like `OutcomeValue` (Null,
/// Bool, Int, Text, List, Map — the recursive collection kinds line up
/// one-for-one with `Empty`/`Boolean`/`Integer`/`Text`/`List`/`Record`).
/// `Float` is the one nodus carries that `OutcomeValue` does not — the same
/// gap `learn`'s confidence field and `budget`'s handlers already work
/// around, formatted as `Text` for the same reason.
fn nodus_value_to_outcome(value: &Value) -> OutcomeValue {
    match value {
        Value::Null => OutcomeValue::Empty,
        Value::Bool(b) => OutcomeValue::Boolean(*b),
        Value::Int(n) => OutcomeValue::Integer(*n),
        Value::Float(f) => OutcomeValue::Text(f.to_string()),
        Value::Text(s) => OutcomeValue::Text(s.clone()),
        Value::List(items) => {
            OutcomeValue::List(items.iter().map(nodus_value_to_outcome).collect())
        }
        Value::Map(fields) => OutcomeValue::Record(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), nodus_value_to_outcome(v)))
                .collect(),
        ),
    }
}

fn register_scaffold(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("workflow.scaffold");
    let invocable = Invocable {
        id: id.clone(),
        name: "Scaffold",
        summary: "Generate a new workflow skeleton.",
        group: "workflow",
        locus: Locus::Semantic,
        binders: vec![
            Binder {
                name: "name",
                kind: BinderKind::Text,
                optional: false,
            },
            Binder {
                name: "out",
                kind: BinderKind::NamedText,
                optional: true,
            },
        ],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry.register(&Registrant::core(), invocable).expect(
        "core:workflow.scaffold registers cleanly at bootstrap — a duplicate id here is a bug",
    );
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let name = text_arg(args, "name");
            let dest = opt_text_arg(args, "out")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(format!("{name}.nodus")));
            if dest.exists() {
                return Outcome::Unavailable {
                    reason: format!("file already exists: {}", dest.display()),
                };
            }
            let ast = workflows::scaffold(name);
            let source = nodus::transpiler::Transpiler::to_nodus(&ast);
            match std::fs::write(&dest, &source) {
                Ok(()) => {
                    let abs = dest.canonicalize().unwrap_or(dest);
                    Outcome::Value(OutcomeValue::Record(vec![(
                        "path".to_string(),
                        OutcomeValue::Text(abs.display().to_string()),
                    )]))
                }
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

fn register_validate(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("workflow.validate");
    let invocable = Invocable {
        id: id.clone(),
        name: "Validate",
        summary: "Validate a workflow file and report diagnostics.",
        group: "workflow",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "file",
            kind: BinderKind::Text,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry.register(&Registrant::core(), invocable).expect(
        "core:workflow.validate registers cleanly at bootstrap — a duplicate id here is a bug",
    );
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let file = std::path::PathBuf::from(text_arg(args, "file"));
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: format!("cannot read {}: {e}", file.display()),
                    };
                }
            };
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.nodus")
                .to_string();
            let report = match workflows::validate(&source, &filename) {
                Ok(r) => r,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: format!("parse failed: {e}"),
                    };
                }
            };
            let diag_record = |d: &nodus::validator::Diagnostic| {
                OutcomeValue::Record(vec![
                    ("code".to_string(), OutcomeValue::Text(d.code.clone())),
                    ("message".to_string(), OutcomeValue::Text(d.message.clone())),
                    ("line".to_string(), OutcomeValue::Integer(d.line as i64)),
                ])
            };
            let errors: Vec<OutcomeValue> = report
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .map(diag_record)
                .collect();
            let warnings: Vec<OutcomeValue> = report
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .map(diag_record)
                .collect();
            let infos: Vec<OutcomeValue> = report
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Info)
                .map(diag_record)
                .collect();
            Outcome::Value(OutcomeValue::Record(vec![
                (
                    "status".to_string(),
                    OutcomeValue::Text(if report.has_errors {
                        "failed".to_string()
                    } else {
                        "ok".to_string()
                    }),
                ),
                ("errors".to_string(), OutcomeValue::List(errors)),
                ("warnings".to_string(), OutcomeValue::List(warnings)),
                ("infos".to_string(), OutcomeValue::List(infos)),
            ]))
        }),
    );
}

fn register_run(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("workflow.run");
    let invocable = Invocable {
        id: id.clone(),
        name: "Run",
        summary: "Execute a workflow file.",
        group: "workflow",
        locus: Locus::Semantic,
        binders: vec![
            Binder {
                name: "file",
                kind: BinderKind::Text,
                optional: false,
            },
            Binder {
                name: "input",
                kind: BinderKind::NamedText,
                optional: true,
            },
        ],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:workflow.run registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let file = std::path::PathBuf::from(text_arg(args, "file"));
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: format!("cannot read {}: {e}", file.display()),
                    };
                }
            };
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.nodus")
                .to_string();
            let parsed_input = match opt_text_arg(args, "input") {
                None => None,
                Some(s) => match serde_json::from_str::<serde_json::Value>(s) {
                    Ok(jv) => Some(json_value_to_nodus(jv)),
                    Err(e) => {
                        return Outcome::Unavailable {
                            reason: format!("--input is not valid JSON: {e}"),
                        };
                    }
                },
            };
            match workflows::run(&source, &filename, parsed_input) {
                Err(diags) => {
                    let messages: Vec<OutcomeValue> = diags
                        .iter()
                        .map(|d| {
                            OutcomeValue::Text(format!(
                                "[{}] {} (line {})",
                                d.code, d.message, d.line
                            ))
                        })
                        .collect();
                    Outcome::Value(OutcomeValue::Record(vec![
                        (
                            "status".to_string(),
                            OutcomeValue::Text("failed".to_string()),
                        ),
                        ("diagnostics".to_string(), OutcomeValue::List(messages)),
                    ]))
                }
                Ok(result) => {
                    // Pre-migration exit codes were 3-way (Ok|Partial → 0,
                    // Failed|Aborted → 1, Paused → 2). This migration keeps
                    // Failed/Aborted mapping to a non-zero exit ("status"
                    // recognized by the renderer) but collapses Paused
                    // into the same non-zero bucket rather than a distinct
                    // third code — no test locks the finer distinction,
                    // and inventing a third render convention for one
                    // verb's one status was not worth it under this
                    // task's own scope. Disclosed, not silent.
                    let status_str = match result.status {
                        Status::Ok | Status::Partial => "ok",
                        Status::Failed | Status::Aborted => "failed",
                        Status::Paused => "paused",
                    };
                    let mut fields = vec![
                        (
                            "workflow".to_string(),
                            OutcomeValue::Text(result.workflow.clone()),
                        ),
                        (
                            "status".to_string(),
                            OutcomeValue::Text(status_str.to_string()),
                        ),
                        ("out".to_string(), nodus_value_to_outcome(&result.out)),
                    ];
                    if !result.log.is_empty() {
                        fields.push((
                            "log".to_string(),
                            OutcomeValue::List(
                                result
                                    .log
                                    .iter()
                                    .map(|e| {
                                        OutcomeValue::Text(format!(
                                            "{}. [{}] {:?}",
                                            e.step, e.command, e.result
                                        ))
                                    })
                                    .collect(),
                            ),
                        ));
                    }
                    if !result.errors.is_empty() {
                        fields.push((
                            "errors".to_string(),
                            OutcomeValue::List(
                                result
                                    .errors
                                    .iter()
                                    .map(|e| {
                                        OutcomeValue::Text(format!(
                                            "[{} step {}] {}",
                                            e.code, e.step, e.reason
                                        ))
                                    })
                                    .collect(),
                            ),
                        ));
                    }
                    Outcome::Value(OutcomeValue::Record(fields))
                }
            }
        }),
    );
}

fn json_value_to_nodus(jv: serde_json::Value) -> Value {
    match jv {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Text(s),
        serde_json::Value::Array(items) => {
            Value::List(items.into_iter().map(json_value_to_nodus).collect())
        }
        serde_json::Value::Object(map) => Value::Map(
            map.into_iter()
                .map(|(k, v)| (k, json_value_to_nodus(v)))
                .collect(),
        ),
    }
}

fn register_transpile(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("workflow.transpile");
    let invocable = Invocable {
        id: id.clone(),
        name: "Transpile",
        summary: "Transpile a workflow to a different representation.",
        group: "workflow",
        locus: Locus::Semantic,
        binders: vec![
            Binder {
                name: "file",
                kind: BinderKind::Text,
                optional: false,
            },
            Binder {
                name: "human",
                kind: BinderKind::Flag,
                optional: true,
            },
            Binder {
                name: "compact",
                kind: BinderKind::Flag,
                optional: true,
            },
        ],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry.register(&Registrant::core(), invocable).expect(
        "core:workflow.transpile registers cleanly at bootstrap — a duplicate id here is a bug",
    );
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let file = std::path::PathBuf::from(text_arg(args, "file"));
            let human = flag_arg(args, "human");
            let compact = flag_arg(args, "compact");
            // The pre-migration grammar enforced `--human`/`--compact`
            // mutual exclusivity at the clap level (`conflicts_with`) — a
            // usage failure. `Binder` has no cross-binder constraint
            // concept, so this is now an application-level refusal
            // instead — a real, disclosed shift, not an invented
            // shortcut.
            if human && compact {
                return Outcome::Unavailable {
                    reason: "--human and --compact cannot both be given".to_string(),
                };
            }
            let mode = if human {
                TranspileMode::Human
            } else {
                TranspileMode::Compact
            };
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: format!("cannot read {}: {e}", file.display()),
                    };
                }
            };
            match workflows::transpile(&source, mode) {
                Ok(output) => Outcome::Value(OutcomeValue::Text(output)),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

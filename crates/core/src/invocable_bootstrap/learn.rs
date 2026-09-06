use std::sync::Arc;

use cronus_contract::{Binder, BinderKind, Invocable, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};
use cronus_domain::learning::LearningApprovalGate;

use super::{core_id, text_arg};

pub(super) fn register(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    register_list(registry, dispatcher);
    register_approve(registry, dispatcher);
    register_reject(registry, dispatcher);
}

fn register_list(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("learn.list");
    let invocable = Invocable {
        id: id.clone(),
        name: "List",
        summary: "List pending skill proposals.",
        group: "learn",
        locus: Locus::Semantic,
        binders: Vec::new(),
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:learn.list registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|_args| {
            // `LearningApprovalGate::new()` is explicitly documented as an
            // in-memory gate — the same ephemeral-per-call shape
            // `memory`/`codegraph`/`exec`/`check` already carry, preserved
            // exactly rather than fixed here.
            let gate = LearningApprovalGate::new();
            Outcome::Value(OutcomeValue::List(
                gate.list_pending()
                    .into_iter()
                    .map(|s| {
                        OutcomeValue::Record(vec![
                            ("id".to_string(), OutcomeValue::Text(s.id.clone())),
                            ("trigger".to_string(), OutcomeValue::Text(s.trigger.clone())),
                            (
                                "confidence".to_string(),
                                // No `OutcomeValue::Float` exists — the same
                                // gap `BinderKind` has for input values, on
                                // the output side. A displayed value needs
                                // no round-trip parse, so formatting it as
                                // `Text` is a reasonable, low-risk choice
                                // rather than inventing a new contract-tier
                                // variant under this task's own pressure.
                                OutcomeValue::Text(format!("{:.2}", s.confidence)),
                            ),
                        ])
                    })
                    .collect(),
            ))
        }),
    );
}

fn register_approve(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("learn.approve");
    let invocable = Invocable {
        id: id.clone(),
        name: "Approve",
        summary: "Approve a skill proposal.",
        group: "learn",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "id",
            kind: BinderKind::Text,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:learn.approve registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let proposal_id = text_arg(args, "id");
            let mut gate = LearningApprovalGate::new();
            if gate.approve(proposal_id) {
                Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(proposal_id.to_string()),
                )]))
            } else {
                Outcome::Unavailable {
                    reason: format!("skill proposal '{proposal_id}' not found"),
                }
            }
        }),
    );
}

fn register_reject(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("learn.reject");
    let invocable = Invocable {
        id: id.clone(),
        name: "Reject",
        summary: "Reject a skill proposal.",
        group: "learn",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "id",
            kind: BinderKind::Text,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:learn.reject registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let proposal_id = text_arg(args, "id");
            let mut gate = LearningApprovalGate::new();
            if gate.reject(proposal_id) {
                Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(proposal_id.to_string()),
                )]))
            } else {
                Outcome::Unavailable {
                    reason: format!("skill proposal '{proposal_id}' not found"),
                }
            }
        }),
    );
}

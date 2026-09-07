use std::sync::Arc;

use cronus_contract::{Binder, BinderKind, Invocable, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::budget::{BudgetEngine, BudgetPeriod, BudgetPolicy};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};

use super::{core_id, float_arg};

pub(super) fn register(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    register_show(registry, dispatcher);
    register_set(registry, dispatcher);
    register_reset(registry, dispatcher);
}

fn register_show(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("budget.show");
    let invocable = Invocable {
        id: id.clone(),
        name: "Show",
        summary: "Show current budget usage.",
        group: "budget",
        locus: Locus::Semantic,
        binders: Vec::new(),
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:budget.show registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|_args| {
            // `BudgetEngine::new()` is ephemeral per call — the same
            // residual `memory`/`codegraph`/`exec`/`learn` already carry,
            // preserved exactly rather than fixed here.
            let engine = BudgetEngine::new();
            let spent = engine.spent_for("default");
            // No `OutcomeValue::Float` exists — the same gap `learn`'s
            // confidence field already worked around; a displayed value
            // needs no round-trip parse, so `Text` is fine.
            Outcome::Value(OutcomeValue::Record(vec![(
                "spent".to_string(),
                OutcomeValue::Text(format!("{spent:.4}")),
            )]))
        }),
    );
}

fn register_set(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("budget.set");
    let invocable = Invocable {
        id: id.clone(),
        name: "Set",
        summary: "Set a workspace budget limit (USD).",
        group: "budget",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "limit",
            kind: BinderKind::Float,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:budget.set registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let limit = float_arg(args, "limit");
            let mut engine = BudgetEngine::new();
            let policy = BudgetPolicy::workspace("default", limit, BudgetPeriod::Monthly);
            engine.add_policy(policy);
            Outcome::Value(OutcomeValue::Record(vec![(
                "limit".to_string(),
                OutcomeValue::Text(format!("{limit:.4}")),
            )]))
        }),
    );
}

fn register_reset(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("budget.reset");
    let invocable = Invocable {
        id: id.clone(),
        name: "Reset",
        summary: "Reset budget counters.",
        group: "budget",
        locus: Locus::Semantic,
        binders: Vec::new(),
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:budget.reset registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|_args| {
            let mut engine = BudgetEngine::new();
            engine.reset();
            Outcome::Value(OutcomeValue::Record(vec![(
                "result".to_string(),
                OutcomeValue::Text("reset".to_string()),
            )]))
        }),
    );
}

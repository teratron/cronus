use std::sync::Arc;

use cronus_contract::{Binder, BinderKind, Invocable, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::exec_workspace::ExecWorkspaceManager;
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};
use cronus_domain::tool_security::now_ms;

use super::{core_id, text_arg};

fn base_dir() -> std::path::PathBuf {
    cronus_domain::paths::Paths::os_native()
        .resolve(cronus_domain::paths::Root::State)
        .join("exec-workspaces")
}

pub(super) fn register(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    register_list(registry, dispatcher);
    register_create(registry, dispatcher);
    register_finalize(registry, dispatcher);
    register_discard(registry, dispatcher);
}

fn register_list(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("exec.list");
    let invocable = Invocable {
        id: id.clone(),
        name: "List",
        summary: "List execution workspaces.",
        group: "exec",
        locus: Locus::Semantic,
        binders: Vec::new(),
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:exec.list registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|_args| {
            // `ExecWorkspaceManager::new()` is genuinely in-memory (its own
            // doc comment says so), so a fresh one here sees nothing a
            // separate `create` dispatch wrote — the same ephemeral-per-
            // call shape `memory`/`codegraph` already carry, preserved
            // exactly (SP-9/SP-10) rather than fixed under this task's
            // pressure, recorded separately.
            let mgr = ExecWorkspaceManager::new();
            Outcome::Value(OutcomeValue::List(
                mgr.list()
                    .iter()
                    .map(|w| {
                        OutcomeValue::Record(vec![
                            ("id".to_string(), OutcomeValue::Text(w.id.clone())),
                            (
                                "state".to_string(),
                                OutcomeValue::Text(w.state.as_str().to_string()),
                            ),
                        ])
                    })
                    .collect(),
            ))
        }),
    );
}

fn register_create(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("exec.create");
    let invocable = Invocable {
        id: id.clone(),
        name: "Create",
        summary: "Create an execution workspace for a card.",
        group: "exec",
        locus: Locus::Semantic,
        binders: vec![
            Binder {
                name: "ws_id",
                kind: BinderKind::Text,
                optional: false,
            },
            Binder {
                name: "card_id",
                kind: BinderKind::Text,
                optional: false,
            },
        ],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:exec.create registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let ws_id = text_arg(args, "ws_id");
            let card_id = text_arg(args, "card_id");
            let dir = base_dir();
            let _ = std::fs::create_dir_all(&dir);
            let mut mgr = ExecWorkspaceManager::new();
            match mgr.create(ws_id, card_id, &dir, now_ms()) {
                Ok(w) => Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(w.id.clone()),
                )])),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

fn register_finalize(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("exec.finalize");
    let invocable = Invocable {
        id: id.clone(),
        name: "Finalize",
        summary: "Finalize an execution workspace.",
        group: "exec",
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
        .expect("core:exec.finalize registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let ws_id = text_arg(args, "id");
            let mut mgr = ExecWorkspaceManager::new();
            // `gate_passed: true` hardcoded — the exact pre-existing CLI
            // behavior, not something this migration decides anew.
            match mgr.finalize(ws_id, true, now_ms()) {
                Ok(()) => Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(ws_id.to_string()),
                )])),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

fn register_discard(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("exec.discard");
    let invocable = Invocable {
        id: id.clone(),
        name: "Discard",
        summary: "Discard an execution workspace.",
        group: "exec",
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
        .expect("core:exec.discard registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let ws_id = text_arg(args, "id");
            let mut mgr = ExecWorkspaceManager::new();
            match mgr.discard(ws_id) {
                Ok(()) => Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(ws_id.to_string()),
                )])),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

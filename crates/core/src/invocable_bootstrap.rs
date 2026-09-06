//! Facade composition for the invocable registry (§4.3, EP-12): assembles
//! the registry and dispatcher, and registers every core invocable through
//! the same [`InvocableRegistry::register`] a contribution calls — no
//! private, privileged registration path exists here or anywhere else.
//!
//! Lives in the facade, not `cronus-domain`, because it binds real
//! [`Engine`] behavior to handlers — composition, not domain logic.

use std::sync::Arc;

use cronus_contract::{
    ArgValue, Binder, BinderKind, Invocable, InvocableId, Locus, Outcome, OutcomeValue, Stability,
};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};
use cronus_domain::{Capabilities, Engine};

use crate::memory::{MemoryEntry, MemoryKind, MemorySource, MemoryStore};

/// Assemble the facade's invocable registry and dispatcher around `engine`,
/// with every core invocable registered through the public door.
pub fn bootstrap(engine: Engine) -> (InvocableRegistry, Dispatcher) {
    let mut registry = InvocableRegistry::new();
    let mut dispatcher = Dispatcher::new();
    let engine = Arc::new(engine);

    register_version(&mut registry, &mut dispatcher, engine);
    register_memory_store(&mut registry, &mut dispatcher);
    register_memory_search(&mut registry, &mut dispatcher);
    register_memory_forget(&mut registry, &mut dispatcher);

    (registry, dispatcher)
}

fn register_version(
    registry: &mut InvocableRegistry,
    dispatcher: &mut Dispatcher,
    engine: Arc<Engine>,
) {
    let id = InvocableId::new("core:version")
        .expect("core-authored literal identity must be well-formed — a bug if it isn't");
    let invocable = Invocable {
        id: id.clone(),
        name: "Version",
        summary: "Show the engine/product version.",
        group: "status",
        // Diagnoses the product's own installation, not the user's work —
        // reclassified from `Semantic` once `Locus::Installation` existed
        // to fit it. Unlike `core:status` (moved to the CLI frontend's own
        // installation grammar once that frontend needed to declare its
        // installation verbs in one place — the facade is not that
        // frontend), the product version is genuinely a fact any surface
        // might want, so it stays registered here rather than moving with
        // it.
        locus: Locus::Installation,
        binders: Vec::<Binder>::new(),
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:version registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(move |_args| Outcome::Value(OutcomeValue::Text(engine.version().to_string()))),
    );
}

/// Read one required, already-bound `Text` argument. `bind()` (IB-2)
/// guarantees this argument exists and matches its declared `BinderKind`
/// before the handler ever runs, so a miss here is a bug elsewhere, not a
/// caller condition to report gracefully — the empty-string fallback keeps
/// this function panic-free without pretending the case is expected.
fn text_arg<'a>(args: &'a cronus_contract::ArgValues, name: &str) -> &'a str {
    match args.get(name) {
        Some(ArgValue::Text(value)) => value.as_str(),
        _ => "",
    }
}

fn register_memory_store(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = InvocableId::new("core:memory.store")
        .expect("core-authored literal identity must be well-formed — a bug if it isn't");
    let invocable = Invocable {
        id: id.clone(),
        name: "Store",
        summary: "Store a key-value memory entry.",
        group: "memory",
        locus: Locus::Semantic,
        binders: vec![
            Binder {
                name: "key",
                kind: BinderKind::Text,
                optional: false,
            },
            Binder {
                name: "value",
                kind: BinderKind::Text,
                optional: false,
            },
        ],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:memory.store registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let key = text_arg(args, "key");
            let value = text_arg(args, "value");
            let store = match MemoryStore::open_in_memory() {
                Ok(store) => store,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: e.to_string(),
                    };
                }
            };
            let entry = MemoryEntry::new(
                MemoryKind::ProjectContext,
                MemorySource::System,
                key.to_string(),
                value.to_string(),
            );
            match store.add(entry) {
                Ok(entry_id) => Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(entry_id.to_string()),
                )])),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

fn register_memory_search(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = InvocableId::new("core:memory.search")
        .expect("core-authored literal identity must be well-formed — a bug if it isn't");
    let invocable = Invocable {
        id: id.clone(),
        name: "Search",
        summary: "Search memory entries by keyword.",
        group: "memory",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "query",
            kind: BinderKind::Text,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:memory.search registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let query = text_arg(args, "query");
            let store = match MemoryStore::open_in_memory() {
                Ok(store) => store,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: e.to_string(),
                    };
                }
            };
            // A zero-item list and a genuinely empty result are different
            // facts (§4.5's zero-count-list fixture guards exactly this):
            // a completed search that matched nothing is still a List, not
            // Empty — Empty would claim the search itself produced nothing
            // to report, which is not what a clean zero-match search means.
            match store.search_fts(query, 10) {
                Ok(entries) => Outcome::Value(OutcomeValue::List(
                    entries
                        .into_iter()
                        .map(|entry| {
                            OutcomeValue::Record(vec![
                                ("id".to_string(), OutcomeValue::Text(entry.id.to_string())),
                                ("title".to_string(), OutcomeValue::Text(entry.title)),
                            ])
                        })
                        .collect(),
                )),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

fn register_memory_forget(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = InvocableId::new("core:memory.forget")
        .expect("core-authored literal identity must be well-formed — a bug if it isn't");
    let invocable = Invocable {
        id: id.clone(),
        name: "Forget",
        summary: "Delete a memory entry by id.",
        group: "memory",
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
        .expect("core:memory.forget registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let entry_id = text_arg(args, "id");
            let store = match MemoryStore::open_in_memory() {
                Ok(store) => store,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: e.to_string(),
                    };
                }
            };
            match store.delete(entry_id) {
                Ok(true) => Outcome::Value(OutcomeValue::Record(vec![(
                    "id".to_string(),
                    OutcomeValue::Text(entry_id.to_string()),
                )])),
                // "Not found" has no dedicated Outcome variant today — the
                // closest honest fit is Unavailable (the resource this
                // operation depends on, a matching entry, could not be
                // reached), recorded here rather than silently assumed
                // correct forever; a future spec amendment may want a
                // dedicated "not found" shape.
                Ok(false) => Outcome::Unavailable {
                    reason: format!("entry '{entry_id}' not found"),
                },
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}

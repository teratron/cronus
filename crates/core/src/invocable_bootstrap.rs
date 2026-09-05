//! Facade composition for the invocable registry (§4.3, EP-12): assembles
//! the registry and dispatcher, and registers every core invocable through
//! the same [`InvocableRegistry::register`] a contribution calls — no
//! private, privileged registration path exists here or anywhere else.
//!
//! Lives in the facade, not `cronus-domain`, because it binds real
//! [`Engine`] behavior to handlers — composition, not domain logic.

use std::sync::Arc;

use cronus_contract::{Binder, Invocable, InvocableId, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};
use cronus_domain::{Capabilities, Engine};

/// Assemble the facade's invocable registry and dispatcher around `engine`,
/// with every core invocable registered through the public door.
pub fn bootstrap(engine: Engine) -> (InvocableRegistry, Dispatcher) {
    let mut registry = InvocableRegistry::new();
    let mut dispatcher = Dispatcher::new();
    let engine = Arc::new(engine);

    register_status(&mut registry, &mut dispatcher, Arc::clone(&engine));
    register_version(&mut registry, &mut dispatcher, engine);

    (registry, dispatcher)
}

fn register_status(
    registry: &mut InvocableRegistry,
    dispatcher: &mut Dispatcher,
    engine: Arc<Engine>,
) {
    let id = InvocableId::from("core:status");
    let invocable = Invocable {
        id: id.clone(),
        name: "Status",
        summary: "Show the current engine status.",
        group: "status",
        locus: Locus::Semantic,
        binders: Vec::<Binder>::new(),
        stability: Stability::Shipped,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:status registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(move |_args| Outcome::Value(OutcomeValue::Text(engine.status()))),
    );
}

fn register_version(
    registry: &mut InvocableRegistry,
    dispatcher: &mut Dispatcher,
    engine: Arc<Engine>,
) {
    let id = InvocableId::from("core:version");
    let invocable = Invocable {
        id: id.clone(),
        name: "Version",
        summary: "Show the engine/product version.",
        group: "status",
        locus: Locus::Semantic,
        binders: Vec::<Binder>::new(),
        stability: Stability::Shipped,
    };
    registry
        .register(&Registrant::core(), invocable)
        .expect("core:version registers cleanly at bootstrap — a duplicate id here is a bug");
    dispatcher.attach(
        id,
        Arc::new(move |_args| Outcome::Value(OutcomeValue::Text(engine.version().to_string()))),
    );
}

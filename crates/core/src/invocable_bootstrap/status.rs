use std::sync::Arc;

use cronus_contract::{Binder, Invocable, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};
use cronus_domain::{Capabilities, Engine};

use super::core_id;

pub(super) fn register(
    registry: &mut InvocableRegistry,
    dispatcher: &mut Dispatcher,
    engine: Arc<Engine>,
) {
    let id = core_id("version");
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

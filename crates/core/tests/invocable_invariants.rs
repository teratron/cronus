//! Facade-level invariant sweep for the invocable registry bootstrap
//! (EP-12, INV-7) — exercised through the real facade export chain
//! (`cronus_core::invocable_bootstrap::bootstrap`, `cronus_core::invocable::*`),
//! not a stand-in.

use cronus_contract::{Invocable, InvocableId, Locus, Outcome, OutcomeValue, Stability};
use cronus_core::Engine;
use cronus_core::invocable::{CONTRIBUTE_GRANT, Dispatcher, InvocableRegistry, Registrant};
use cronus_core::invocable_bootstrap::bootstrap;

/// EP-12: core invocables are registered through the same public door a
/// contribution uses — proven by registering a real contribution into the
/// exact registry `bootstrap` populated, alongside the core invocables it
/// already holds, with no separate API involved.
#[test]
fn core_invocables_and_a_contribution_share_one_registration_door() {
    let (mut registry, _dispatcher) = bootstrap(Engine::new());

    assert!(
        registry
            .resolve(&InvocableId::from("core:status"))
            .is_some(),
        "bootstrap must register core:status"
    );
    assert!(
        registry
            .resolve(&InvocableId::from("core:version"))
            .is_some(),
        "bootstrap must register core:version"
    );

    let contribution = Invocable {
        id: InvocableId::from("myext:hello"),
        name: "Hello",
        summary: "A contributed invocable proving the shared door.",
        group: "test",
        locus: Locus::Semantic,
        binders: Vec::new(),
        stability: Stability::Shipped,
    };
    let registrant = Registrant::extension("myext", "src-1").with_grant(CONTRIBUTE_GRANT);

    // The SAME `register` the bootstrap used for "core:status"/"core:version"
    // accepts this contribution — no privileged, core-only path exists.
    assert!(registry.register(&registrant, contribution).is_ok());
    assert!(
        registry
            .resolve(&InvocableId::from("myext:hello"))
            .is_some()
    );
}

/// INV-7: dispatch output is masked against configured secrets — the single
/// boundary point, proven directly on the real `Dispatcher` rather than a
/// stand-in redaction call.
#[test]
fn dispatch_output_passes_through_the_shared_redaction_path() {
    let mut registry = InvocableRegistry::new();
    let mut dispatcher = Dispatcher::new();

    let id = InvocableId::from("core:echo-secret");
    registry
        .register(
            &Registrant::core(),
            Invocable {
                id: id.clone(),
                name: "Echo secret",
                summary: "Returns a value containing a known secret, for the redaction test.",
                group: "test",
                locus: Locus::Semantic,
                binders: Vec::new(),
                stability: Stability::Shipped,
            },
        )
        .unwrap();
    dispatcher.attach(
        id.clone(),
        std::sync::Arc::new(|_args| {
            Outcome::Value(OutcomeValue::Text("token=sk-LIVE-777 ready".to_string()))
        }),
    );
    dispatcher.set_secrets(vec!["sk-LIVE-777".to_string()]);

    let invocation = cronus_contract::Invocation {
        id,
        args: cronus_contract::ArgValues::new(),
        caller: cronus_contract::Surface::Cli,
    };
    let outcome = dispatcher.dispatch(&registry, &invocation);

    match outcome {
        Outcome::Value(OutcomeValue::Text(text)) => {
            assert!(!text.contains("sk-LIVE-777"), "secret must be masked");
            assert!(text.contains("***"), "masked output must show the mask");
        }
        other => panic!("expected Value(Text), got {other:?}"),
    }
}

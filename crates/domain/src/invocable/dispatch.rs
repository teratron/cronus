//! The execution half: bind an invocation's arguments against its
//! invocable's declared binders, and only then run the attached handler.
//!
//! Binding and execution are deliberately two separate steps with a hard
//! ordering (IB-2): every declared binder is checked before the handler
//! runs at all, so a rejection is a structural fact ("the handler did not
//! run") rather than something the handler itself would have to notice and
//! report. This module's own [`bind`] can produce only two of the four
//! rejection modes — [`RejectionMode::Absent`] (nothing supplied for a
//! required binder) and [`RejectionMode::IllShaped`] (a supplied value's
//! shape does not match the binder's declared kind) — because the values it
//! checks ([`ArgValue`]) are already fully parsed, typed, in-memory data.
//! [`RejectionMode::Unreadable`] (a source could not be obtained) and
//! [`RejectionMode::Malformed`] (present but not parseable at all) describe
//! failures that happen *before* a value becomes an [`ArgValue`] — reading
//! an external source, parsing raw surface input — and that is I/O and
//! surface-specific work this crate's no-I/O tier must not perform (§4.3);
//! it belongs to whatever surface constructs an [`ArgValues`] from raw
//! input, upstream of dispatch. Both modes remain fully representable and
//! located by [`Rejection`] itself; this module just is not where they
//! originate.

use std::collections::HashMap;

use cronus_contract::{
    ArgValue, ArgValues, Invocable, InvocableId, Invocation, Outcome, Rejection, RejectionMode,
};

use super::registry::InvocableRegistry;

/// The executable behavior attached to one invocable. Kept separate from
/// the invocable's descriptor (its catalog entry) so a surface can read the
/// catalog — for help, completion, or the desktop's `catalog()` — without
/// needing a handler to exist at all.
pub type Handler = Box<dyn Fn(&ArgValues) -> Outcome + Send + Sync>;

/// Checks one supplied value against one binder's declared kind.
fn matches_kind(value: &ArgValue, kind: cronus_contract::BinderKind) -> bool {
    use cronus_contract::BinderKind;
    matches!(
        (value, kind),
        (ArgValue::Text(_), BinderKind::Text)
            | (ArgValue::Integer(_), BinderKind::Integer)
            | (ArgValue::Boolean(_), BinderKind::Boolean)
            | (ArgValue::Flag, BinderKind::Flag)
    )
}

/// Bind every declared binder against `args`, in declared order, stopping at
/// the first rejection (a stable, learnable order — the same malformed
/// input always fails at the same binder). `None` means every binder is
/// satisfied and the body may run.
pub fn bind(descriptor: &Invocable, args: &ArgValues) -> Option<Rejection> {
    for binder in &descriptor.binders {
        match args.get(binder.name) {
            None if binder.optional => continue,
            None => {
                return Some(Rejection {
                    binder: binder.name,
                    mode: RejectionMode::Absent,
                    detail: "no value supplied for a required binder".to_string(),
                });
            }
            Some(value) if !matches_kind(value, binder.kind) => {
                return Some(Rejection {
                    binder: binder.name,
                    mode: RejectionMode::IllShaped,
                    detail: format!(
                        "supplied value does not match the declared {:?} shape",
                        binder.kind
                    ),
                });
            }
            Some(_) => continue,
        }
    }
    None
}

/// Holds the executable behavior attached to each invocable, and runs
/// bind-before-invoke over a [`InvocableRegistry`]'s descriptors.
#[derive(Default)]
pub struct Dispatcher {
    handlers: HashMap<InvocableId, Handler>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            handlers: HashMap::new(),
        }
    }

    /// Attach the executable behavior for an invocable. Independent of
    /// registry registration — the two may happen in either order, and a
    /// descriptor with no attached handler is a normal, catalog-readable
    /// state (dispatch reports it as unavailable, never panics).
    pub fn attach(&mut self, id: InvocableId, handler: Handler) {
        self.handlers.insert(id, handler);
    }

    /// Resolve `invocation` against `registry`, bind its arguments, and —
    /// only if binding succeeds — run the attached handler. Every path
    /// returns a structured [`Outcome`]; none of them is a Rust panic or an
    /// unstructured error (IB-3).
    pub fn dispatch(&self, registry: &InvocableRegistry, invocation: &Invocation) -> Outcome {
        let Some(descriptor) = registry.resolve(&invocation.id) else {
            return Outcome::Unavailable {
                reason: format!("no such invocable: {}", invocation.id.as_str()),
            };
        };

        if let Some(rejection) = bind(descriptor, &invocation.args) {
            return Outcome::Rejected(rejection);
        }

        match self.handlers.get(&invocation.id) {
            Some(handler) => handler(&invocation.args),
            None => Outcome::Unavailable {
                reason: format!("no handler attached: {}", invocation.id.as_str()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use cronus_contract::{Binder, BinderKind, Locus, OutcomeValue, Stability, Surface};

    use super::super::registry::Registrant;
    use super::*;

    fn card_add_descriptor() -> Invocable {
        Invocable {
            id: InvocableId::from("core:board.add"),
            name: "Add card",
            summary: "Add a card to the board",
            group: "board",
            locus: Locus::Semantic,
            binders: vec![
                Binder {
                    name: "id",
                    kind: BinderKind::Text,
                    optional: false,
                },
                Binder {
                    name: "task_ref",
                    kind: BinderKind::Text,
                    optional: true,
                },
            ],
            stability: Stability::Shipped,
        }
    }

    fn invocation_with(args: ArgValues) -> Invocation {
        Invocation {
            id: InvocableId::from("core:board.add"),
            args,
            caller: Surface::Cli,
        }
    }

    #[test]
    fn a_binding_failure_leaves_no_side_effect() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&Registrant::core(), card_add_descriptor())
            .unwrap();

        let ran = Arc::new(AtomicUsize::new(0));
        let ran_in_handler = Arc::clone(&ran);
        let mut dispatcher = Dispatcher::new();
        dispatcher.attach(
            InvocableId::from("core:board.add"),
            Box::new(move |_args| {
                ran_in_handler.fetch_add(1, Ordering::SeqCst);
                Outcome::Value(OutcomeValue::Empty)
            }),
        );

        // No "id" supplied — a required binder is absent.
        let outcome = dispatcher.dispatch(&registry, &invocation_with(ArgValues::new()));

        assert!(matches!(outcome, Outcome::Rejected(_)));
        assert_eq!(
            ran.load(Ordering::SeqCst),
            0,
            "the handler must not run when binding rejects"
        );
    }

    #[test]
    fn a_satisfied_optional_binder_absent_still_invokes_the_handler() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&Registrant::core(), card_add_descriptor())
            .unwrap();

        let mut dispatcher = Dispatcher::new();
        dispatcher.attach(
            InvocableId::from("core:board.add"),
            Box::new(|_args| Outcome::Value(OutcomeValue::Text("added".to_string()))),
        );

        let mut args = ArgValues::new();
        args.insert("id", ArgValue::Text("card-1".to_string()));
        // "task_ref" is optional and genuinely absent — not a rejection.
        let outcome = dispatcher.dispatch(&registry, &invocation_with(args));

        assert_eq!(
            outcome,
            Outcome::Value(OutcomeValue::Text("added".to_string()))
        );
    }

    #[test]
    fn a_present_but_wrong_shaped_value_rejects_as_ill_shaped_with_its_binder_location() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&Registrant::core(), card_add_descriptor())
            .unwrap();
        let dispatcher = Dispatcher::new();

        let mut args = ArgValues::new();
        args.insert("id", ArgValue::Boolean(true)); // declared Text, supplied Boolean
        let outcome = dispatcher.dispatch(&registry, &invocation_with(args));

        match outcome {
            Outcome::Rejected(rejection) => {
                assert_eq!(rejection.binder, "id");
                assert_eq!(rejection.mode, RejectionMode::IllShaped);
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[test]
    fn dispatch_against_an_unknown_invocable_reports_unavailable_not_a_panic() {
        let registry = InvocableRegistry::new();
        let dispatcher = Dispatcher::new();
        let outcome = dispatcher.dispatch(&registry, &invocation_with(ArgValues::new()));
        assert!(matches!(outcome, Outcome::Unavailable { .. }));
    }

    #[test]
    fn a_registered_descriptor_with_no_attached_handler_is_unavailable_not_a_panic() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&Registrant::core(), card_add_descriptor())
            .unwrap();
        let dispatcher = Dispatcher::new(); // nothing attached

        let mut args = ArgValues::new();
        args.insert("id", ArgValue::Text("card-1".to_string()));
        let outcome = dispatcher.dispatch(&registry, &invocation_with(args));

        assert!(matches!(outcome, Outcome::Unavailable { .. }));
    }
}

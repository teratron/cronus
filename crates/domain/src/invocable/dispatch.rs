//! The execution half: bind an invocation's arguments against its
//! invocable's declared binders, and only then run the attached handler —
//! under containment when the handler belongs to a contribution (EP-6).
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
//!
//! A core invocable's handler runs directly — it is the trusted kernel, and
//! a core panic is a real defect that must propagate, not a contained
//! failure papered over. A contribution's handler runs on a bounded,
//! panic-caught path: Rust cannot safely preempt a running thread, so the
//! bound is enforced by handing the call to a fresh, unscoped thread and
//! giving up on waiting for it past the bound — the orphaned thread runs to
//! completion on its own and its (now-unwanted) result is discarded, but
//! the caller is never held past the bound, and no dispatch call ever
//! blocks indefinitely or ends in an unstructured panic.

use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use cronus_contract::{
    ArgValue, ArgValues, Invocable, InvocableId, Invocation, Outcome, Rejection, RejectionMode,
};

use super::registry::{CORE_IDENTITY, InvocableRegistry};

/// The executable behavior attached to one invocable. Kept separate from
/// the invocable's descriptor (its catalog entry) so a surface can read the
/// catalog — for help, completion, or the desktop's `catalog()` — without
/// needing a handler to exist at all. `Arc` rather than `Box`: a
/// contribution's handler is cloned onto the bounded thread that runs it
/// (§ above), which needs a `'static`, cheaply-shareable handle rather than
/// a borrow of `self`.
pub type Handler = Arc<dyn Fn(&ArgValues) -> Outcome + Send + Sync>;

/// How long a contributed invocable's handler is given before dispatch
/// gives up on it (EP-6). Core invocables are not bounded at all — see the
/// module doc.
pub const CONTRIBUTION_TIME_BOUND: Duration = Duration::from_secs(5);

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

/// Run a contribution's handler under containment (EP-6): a panic is caught
/// and never unwinds into the caller, and the caller is never held past
/// `bound`. Either failure resolves to [`Outcome::Unavailable`], its reason
/// naming `contributor` — a rejection attributed to the contribution, never
/// a core fault and never a hang.
fn run_contribution(
    handler: &Handler,
    args: &ArgValues,
    contributor: &str,
    bound: Duration,
) -> Outcome {
    let handler = Arc::clone(handler);
    let args = args.clone();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let result = panic::catch_unwind(AssertUnwindSafe(|| handler(&args)));
        // If the receiver already gave up (timed out), this send finds no
        // one listening — that is not itself an error to report anywhere;
        // the timeout branch below has already produced the Outcome.
        let _ = tx.send(result);
    });

    match rx.recv_timeout(bound) {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(_panic_payload)) => Outcome::Unavailable {
            reason: format!("contribution '{contributor}' panicked while handling this invocation"),
        },
        Err(_timeout_or_disconnected) => Outcome::Unavailable {
            reason: format!("contribution '{contributor}' exceeded its time bound"),
        },
    }
}

/// Holds the executable behavior attached to each invocable, and runs
/// bind-before-invoke over a [`InvocableRegistry`]'s descriptors.
pub struct Dispatcher {
    handlers: HashMap<InvocableId, Handler>,
    contribution_bound: Duration,
    secrets: Vec<String>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            handlers: HashMap::new(),
            contribution_bound: CONTRIBUTION_TIME_BOUND,
            secrets: Vec::new(),
        }
    }

    /// A dispatcher whose contribution time bound is `bound` rather than
    /// the production default — for tests that must observe timeout
    /// behavior without waiting out [`CONTRIBUTION_TIME_BOUND`].
    pub fn with_contribution_bound(bound: Duration) -> Self {
        Dispatcher {
            handlers: HashMap::new(),
            contribution_bound: bound,
            secrets: Vec::new(),
        }
    }

    /// Attach the executable behavior for an invocable. Independent of
    /// registry registration — the two may happen in either order, and a
    /// descriptor with no attached handler is a normal, catalog-readable
    /// state (dispatch reports it as unavailable, never panics).
    pub fn attach(&mut self, id: InvocableId, handler: Handler) {
        self.handlers.insert(id, handler);
    }

    /// Replace the secret values every dispatched [`Outcome`] is masked
    /// against (INV-7). Starts empty — populating it from the core's real
    /// secret store is a separate, recorded obligation this method makes
    /// possible but does not itself discharge.
    pub fn set_secrets(&mut self, secrets: Vec<String>) {
        self.secrets = secrets;
    }

    /// Resolve `invocation` against `registry`, bind its arguments, run the
    /// attached handler if binding succeeds, and mask the result against
    /// the configured secrets — the single point every dispatched
    /// [`Outcome`] passes through, regardless of which branch produced it.
    /// Every path returns a structured `Outcome`; none of them is a Rust
    /// panic or an unstructured error (IB-3).
    pub fn dispatch(&self, registry: &InvocableRegistry, invocation: &Invocation) -> Outcome {
        let outcome = self.dispatch_unmasked(registry, invocation);
        let secret_refs: Vec<&str> = self.secrets.iter().map(String::as_str).collect();
        crate::redact::redact_outcome(outcome, &secret_refs)
    }

    fn dispatch_unmasked(&self, registry: &InvocableRegistry, invocation: &Invocation) -> Outcome {
        let Some(descriptor) = registry.resolve(&invocation.id) else {
            return Outcome::Unavailable {
                reason: format!("no such invocable: {}", invocation.id.as_str()),
            };
        };

        if let Some(rejection) = bind(descriptor, &invocation.args) {
            return Outcome::Rejected(rejection);
        }

        let Some(handler) = self.handlers.get(&invocation.id) else {
            return Outcome::Unavailable {
                reason: format!("no handler attached: {}", invocation.id.as_str()),
            };
        };

        if invocation.id.qualifier() == CORE_IDENTITY {
            // The trusted kernel: runs directly, unbounded, uncaught. A
            // core panic is a real defect and must propagate as one.
            handler(&invocation.args)
        } else {
            run_contribution(
                handler,
                &invocation.args,
                invocation.id.qualifier(),
                self.contribution_bound,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use cronus_contract::{Binder, BinderKind, Locus, OutcomeValue, Stability, Surface};

    use super::super::registry::{CONTRIBUTE_GRANT, Registrant};
    use super::*;

    fn contributed_descriptor() -> Invocable {
        Invocable {
            id: InvocableId::from("myext:risky"),
            name: "Risky",
            summary: "A contributed invocable used to exercise containment.",
            group: "test",
            locus: Locus::Semantic,
            binders: Vec::new(),
            stability: Stability::Shipped,
        }
    }

    fn granted_registrant() -> Registrant {
        Registrant::extension("myext", "src-1").with_grant(CONTRIBUTE_GRANT)
    }

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
            Arc::new(move |_args: &ArgValues| {
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
            Arc::new(|_args: &ArgValues| Outcome::Value(OutcomeValue::Text("added".to_string()))),
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

    #[test]
    fn a_panicking_contribution_resolves_to_unavailable_attributed_to_it_and_does_not_unwind() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&granted_registrant(), contributed_descriptor())
            .unwrap();

        let mut dispatcher = Dispatcher::new();
        dispatcher.attach(
            InvocableId::from("myext:risky"),
            Arc::new(|_args: &ArgValues| panic!("boom")),
        );

        let invocation = Invocation {
            id: InvocableId::from("myext:risky"),
            args: ArgValues::new(),
            caller: Surface::Cli,
        };
        // If the panic unwound into this thread rather than being caught on
        // its own thread, this call itself would abort the test process —
        // reaching the assertion below is part of the proof.
        let outcome = dispatcher.dispatch(&registry, &invocation);

        match outcome {
            Outcome::Unavailable { reason } => assert!(
                reason.contains("myext"),
                "reason must attribute the failure to the contribution: {reason}"
            ),
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[test]
    fn a_contribution_exceeding_its_time_bound_resolves_to_unavailable_within_the_bound() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&granted_registrant(), contributed_descriptor())
            .unwrap();

        let bound = Duration::from_millis(50);
        let mut dispatcher = Dispatcher::with_contribution_bound(bound);
        dispatcher.attach(
            InvocableId::from("myext:risky"),
            Arc::new(|_args: &ArgValues| {
                std::thread::sleep(Duration::from_secs(60));
                Outcome::Value(OutcomeValue::Empty)
            }),
        );

        let invocation = Invocation {
            id: InvocableId::from("myext:risky"),
            args: ArgValues::new(),
            caller: Surface::Cli,
        };

        let started = std::time::Instant::now();
        let outcome = dispatcher.dispatch(&registry, &invocation);
        let elapsed = started.elapsed();

        assert!(
            elapsed < Duration::from_secs(5),
            "dispatch must give up at the configured bound, not wait out the handler; took {elapsed:?}"
        );
        match outcome {
            Outcome::Unavailable { reason } => assert!(
                reason.contains("myext"),
                "reason must attribute the timeout to the contribution: {reason}"
            ),
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[test]
    fn a_core_invocable_runs_unbounded_and_a_panic_there_is_not_contained() {
        // The trusted kernel is not subject to the contribution containment
        // path at all — proven by taking the code path that WOULD apply
        // containment (a real panic) and confirming it is the CALLER, not
        // this module, that decides whether to catch it. `catch_unwind`
        // here belongs to the test, standing in for "the core's own
        // surrounding code", not to dispatch.
        let mut registry = InvocableRegistry::new();
        registry
            .register(&Registrant::core(), card_add_descriptor())
            .unwrap();
        let mut dispatcher = Dispatcher::new();
        dispatcher.attach(
            InvocableId::from("core:board.add"),
            Arc::new(|_args: &ArgValues| panic!("a genuine core defect")),
        );

        let mut args = ArgValues::new();
        args.insert("id", ArgValue::Text("card-1".to_string()));
        let invocation = invocation_with(args);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dispatcher.dispatch(&registry, &invocation)
        }));
        assert!(
            result.is_err(),
            "a core panic must propagate, not resolve to a contained Outcome"
        );
    }
}

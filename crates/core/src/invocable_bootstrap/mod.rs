//! Facade composition for the invocable registry (§4.3, EP-12): assembles
//! the registry and dispatcher, and registers every core invocable through
//! the same [`InvocableRegistry::register`] a contribution calls — no
//! private, privileged registration path exists here or anywhere else.
//!
//! Lives in the facade, not `cronus-domain`, because it binds real
//! [`Engine`] behavior to handlers — composition, not domain logic.
//!
//! One submodule per registered group, not one growing flat file: each
//! subsystem's registration pulls in a different `cronus_domain`/adapter
//! module, and mixing them all into a single file is the "small, well-named
//! module" boundary the project's own conventions ask for, not a
//! convenience this facade is exempt from.

use std::sync::Arc;

use cronus_contract::{ArgValue, ArgValues, InvocableId};
use cronus_domain::Engine;
use cronus_domain::invocable::{Dispatcher, InvocableRegistry};

mod agent;
mod check;
mod codegraph;
mod exec;
mod learn;
mod memory;
mod role;
mod status;

/// Assemble the facade's invocable registry and dispatcher around `engine`,
/// with every core invocable registered through the public door.
pub fn bootstrap(engine: Engine) -> (InvocableRegistry, Dispatcher) {
    let mut registry = InvocableRegistry::new();
    let mut dispatcher = Dispatcher::new();
    let engine = Arc::new(engine);

    status::register(&mut registry, &mut dispatcher, engine);
    memory::register(&mut registry, &mut dispatcher);
    codegraph::register(&mut registry, &mut dispatcher);
    agent::register(&mut registry, &mut dispatcher);
    role::register(&mut registry, &mut dispatcher);
    exec::register(&mut registry, &mut dispatcher);
    check::register(&mut registry, &mut dispatcher);
    learn::register(&mut registry, &mut dispatcher);

    (registry, dispatcher)
}

/// A literal, core-authored identity. `expect` is correct here, not a
/// shortcut: a malformed literal is a bug in this file, never a caller
/// condition — the same judgment call `InvocableId::new` callers elsewhere
/// in this facade already make.
fn core_id(tail: &str) -> InvocableId {
    InvocableId::new(format!("core:{tail}"))
        .expect("core-authored literal identity must be well-formed — a bug if it isn't")
}

/// Read one required, already-bound `Text` argument. `bind()` (IB-2)
/// guarantees this argument exists and matches its declared `BinderKind`
/// before the handler ever runs, so a miss here is a bug elsewhere, not a
/// caller condition to report gracefully — the empty-string fallback keeps
/// this function panic-free without pretending the case is expected.
fn text_arg<'a>(args: &'a ArgValues, name: &str) -> &'a str {
    match args.get(name) {
        Some(ArgValue::Text(value)) => value.as_str(),
        _ => "",
    }
}

/// Read one optional `Text`/`NamedText` argument — genuinely absent when
/// the caller supplied nothing, not defaulted to empty (that would make an
/// explicit empty string and a real absence indistinguishable).
fn opt_text_arg<'a>(args: &'a ArgValues, name: &str) -> Option<&'a str> {
    match args.get(name) {
        Some(ArgValue::Text(value)) => Some(value.as_str()),
        _ => None,
    }
}

/// Read one `Flag` argument. Absent means false — a flag binder is never
/// required, so this never needs to distinguish "absent" from "false".
fn flag_arg(args: &ArgValues, name: &str) -> bool {
    matches!(args.get(name), Some(ArgValue::Flag))
}

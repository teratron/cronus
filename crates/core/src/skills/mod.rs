//! Skill system — the concrete realization of the `skill` extension kind.
//! Landing so far: the two-tier stores and shadowing precedence ([`store`]),
//! the canonical package model ([`package`]), the built-in command surface
//! ([`commands`]), the execution model ([`exec`]), the conversion pipeline
//! ([`convert`]), and prompt synthesis ([`synthesize`]).

pub mod commands;
pub mod convert;
pub mod exec;
pub mod package;
pub mod store;
pub mod synthesize;

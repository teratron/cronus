//! Validator — structural checks plus the lint catalog (errors / warnings /
//! info) run before execution.
//!
//! Forthcoming: checks the AST against the loaded [`crate::vocab::Schema`]
//! (unknown vocabulary fails) and the lint rules (undefined variables,
//! ordering constraints, publish-after-validate, loop bounds, …). Block-class
//! findings halt; warning/info findings do not.

//! Workflow-language runtime: lexer, parser/AST, validator (+ lint), executor,
//! transpiler. Self-contained crate that `cronus-core` depends on; extractable
//! to its own repository later.
//!
//! Skeleton only — the behavior-preserving port is implemented in Phase 2
//! (vertical slice: lexer -> parser -> transpiler -> executor -> validator).

/// Placeholder until the runtime port lands. Lets the crate build as a workspace
/// member without yet exposing the (forthcoming) parse/validate/run API.
pub fn placeholder() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_builds() {
        placeholder();
    }
}

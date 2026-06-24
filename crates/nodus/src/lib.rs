//! Declarative workflow DSL runtime — lexer, parser/AST, validator (+ lint),
//! executor, and transpiler. The crate is `std`-only with zero external
//! dependencies: embed it anywhere Rust runs.
//!
//! # Quick start
//!
//! ```rust
//! use nodus::{workflows, executor::Status};
//!
//! let source = r#"
//! §wf:greet v1.0
//! §runtime: { core: schema.nodus }
//! @in:  { name: text }
//! @out: $out
//! @err: ESCALATE(human)
//! @steps:
//!   1. GEN($in.name) → $out
//!   2. LOG($out)
//! "#;
//!
//! let result = workflows::run(source, "greet.nodus", None).unwrap();
//! assert_eq!(result.status, Status::Ok);
//! ```
//!
//! # Workflow lifecycle
//!
//! | Step | Function | Purpose |
//! | --- | --- | --- |
//! | Scaffold | [`workflows::scaffold`] | Minimal valid AST |
//! | Validate | [`workflows::validate`] | All lint diagnostics |
//! | Run | [`workflows::run`] | Execute with built-in stub |
//! | Transpile | [`workflows::transpile`] | Compact or human form |
//!
//! # Design
//!
//! The vocabulary lives in [`vocab`] as data separate from the logic that
//! consults it — updating the language is a data change, not a code change.
//! The [`executor::ModelProvider`] trait is the sole extension point for real
//! AI model integration; the built-in [`executor::StubProvider`] satisfies the
//! interface without I/O, enabling fully in-process tests.

mod error;

pub mod ast;
pub mod lexer;
pub mod observability;
pub mod parser;
pub mod vocab;

// Pipeline modules.
pub mod executor;
pub mod transpiler;
pub mod validator;
pub mod workflows;

pub use error::{Error, Result, Span};
pub use executor::{Executor, ModelProvider, RunResult, Status, StubProvider, Value};
pub use lexer::{Lexer, Token, TokenType};
pub use observability::{
    AuditProvider, ExecutionEvent, FieldDescriptor, LoopType, NoopAuditProvider, RunManifest,
    RunStatus,
};
pub use parser::Parser;
pub use transpiler::Transpiler;
pub use validator::{Diagnostic, Severity, Validator};
pub use vocab::Schema;
pub use workflows::{
    TestReport, TestResult, TranspileMode, ValidationReport, run, run_with_audit,
    run_with_provider, run_with_provider_and_audit, scaffold, test, transpile,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_surface_assembles() {
        // The re-exported lexer + schema compose through the crate root.
        let schema = Schema::builtin();
        let tokens = Lexer::tokenize_str("LOG($out)").expect("tokenizes");
        assert_eq!(tokens.first().map(|t| t.ty), Some(TokenType::CommandName));
        assert!(schema.is_command(&tokens[0].value));
    }

    #[test]
    fn errors_render_with_position() {
        let err = Error::Parse {
            span: Span::new(3, 7),
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "parse error at 3:7: unexpected token");
    }
}

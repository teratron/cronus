//! Workflow-language runtime: lexer, parser/AST, validator (+ lint), executor,
//! and transpiler. A self-contained crate that `cronus` depends on and
//! links in-process, so it runs everywhere the core runs with no external
//! language process. Kept as its own crate (not fused into the core) so it can
//! be extracted to a standalone repository later.
//!
//! The crate is a behavior-preserving port of the reference workflow-language
//! implementation, built as a vertical slice: lexer → parser → transpiler →
//! executor → validator. The vocabulary lives in [`vocab`] as data, separate
//! from the logic that consults it, so updating the language is a data change.
//!
//! ## Status
//!
//! Implemented: the crate scaffold, the builtin vocabulary [`vocab::Schema`],
//! and the [`lexer`]. The parser, transpiler, executor, and validator modules
//! exist as the layout the remaining front-end and runtime tracks fill in.

mod error;

pub mod ast;
pub mod lexer;
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
pub use parser::Parser;
pub use transpiler::Transpiler;
pub use validator::{Diagnostic, Severity, Validator};
pub use vocab::Schema;
pub use workflows::{
    TestReport, TestResult, TranspileMode, ValidationReport, run, run_with_provider, scaffold,
    test, transpile,
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

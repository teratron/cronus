//! Crate-wide error type for the workflow-language runtime.
//!
//! One enum spans every pipeline stage (lex / parse / validate / execute) so a
//! caller matches a single `Result` regardless of where a failure originated.
//! The runtime error *codes* (the `NODUS:*` strings the structured result
//! reports) live in [`crate::vocab::error_code`]; this type is the in-process
//! Rust error surfaced by the library API.

use std::fmt;

/// A 1-based source position (line, column) for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number within the line.
    pub column: u32,
}

impl Span {
    /// Construct a span from a line and column.
    pub fn new(line: u32, column: u32) -> Self {
        Span { line, column }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Every failure mode of the runtime, tagged by the stage that produced it.
///
/// `Lex` / `Parse` carry a source position; `Validate` / `Execute` carry a
/// `NODUS:*` code (see [`crate::vocab::error_code`]) so the structured result
/// can report it verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Tokenization failed at a position.
    Lex { span: Span, message: String },
    /// The token stream did not match the grammar.
    Parse { span: Span, message: String },
    /// A validation (lint) rule blocked execution.
    Validate { code: String, message: String },
    /// A step or control-flow rule failed at run time.
    Execute { code: String, message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex { span, message } => write!(f, "lex error at {span}: {message}"),
            Error::Parse { span, message } => write!(f, "parse error at {span}: {message}"),
            Error::Validate { code, message } => write!(f, "[{code}] {message}"),
            Error::Execute { code, message } => write!(f, "[{code}] {message}"),
        }
    }
}

impl std::error::Error for Error {}

/// Crate result alias.
pub type Result<T> = std::result::Result<T, Error>;

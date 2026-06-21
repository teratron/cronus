//! Parser — grammar-driven construction of the AST from the lexer's token
//! stream.
//!
//! Forthcoming (front-end track): the largest module. Consumes
//! [`crate::lexer::Token`]s and the loaded [`crate::vocab::Schema`], and
//! produces [`crate::ast`] nodes, reporting structural violations as typed
//! parse errors rather than panicking.

//! Code graph — symbol extraction, SQLite + FTS5 index, RRF fusion,
//! and community detection.
//!
//! Tree-sitter grammar dependencies are deferred until the
//! language binaries are bundled. For now, the extractor is a seam
//! trait with a Rust-regex stub that parses `fn <name>` patterns.

pub mod community;
pub mod dedup;
pub mod extractor;
pub mod index;
pub mod search;

//! Transpiler — lossless conversion between the compact and human renderings.
//!
//! Forthcoming: both directions route through [`crate::ast`], so a round-trip
//! preserves logic (the dual-representation guarantee). Consumes the verb map
//! from [`crate::vocab`].

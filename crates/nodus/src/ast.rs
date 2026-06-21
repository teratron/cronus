//! AST node model — the shared shape both the compact and human forms parse to.
//!
//! Forthcoming (front-end track): typed nodes for the workflow header, runtime
//! block, trigger, inputs, context, constraints, steps, control flow, output,
//! and error handler. Kept as the single contract consumed by the parser,
//! transpiler, executor, and validator.

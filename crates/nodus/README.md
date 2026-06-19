# nodus (Rust library)

The workflow-language runtime: lexer, parser/AST, validator (+ lint), executor, transpiler.
A behavior-preserving Rust port of the workflow DSL. Self-contained crate that `core`
depends on; extractable to its own repository later if reused elsewhere.

Schema and grammar are loaded as data (not compiled logic). Workflow steps bind to
Cronus subsystems via the core (memory, HITL, orchestration, quality, model router).

# Nodus

A declarative workflow DSL and self-contained Rust runtime for AI-augmented automation pipelines.

## Overview

Nodus workflows describe inputs, outputs, hard constraints, soft preferences, and a bounded step
body. The runtime validates, executes, and transpiles `.nodus` files with zero external dependencies.

## Usage

```rust
use nodus::workflows;

let source = r#"
§wf:greet v1.0
§runtime: { core: schema.nodus }
@in:  { name: text }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.name) → $out
  2. LOG($out)
"#;

let result = workflows::run(source, "greet.nodus", None)
    .expect("validation must pass");

assert_eq!(result.status, nodus::executor::Status::Ok);
```

## Workflow lifecycle

| Step | Function | Purpose |
| --- | --- | --- |
| Scaffold | `workflows::scaffold(name)` | Generate a minimal valid AST |
| Validate | `workflows::validate(source, filename)` | Lint — returns all diagnostics |
| Run | `workflows::run(source, filename, input)` | Execute with built-in stub provider |
| Transpile | `workflows::transpile(source, mode)` | Convert to compact or human form |

## Features

No optional features. The crate is `std`-only and embeds everything it needs.

## License

MIT

# Workflow Runtime

**Version:** 1.2.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-workflow-language.md

## Overview

The concrete realization of the workflow language: a Rust runtime **crate inside the Cronus monorepo** (`crates/nodus`) — lexer, parser, validator (lint), executor, and transpiler — that the core depends on and links **in-process**, so it runs everywhere the core runs (desktop and mobile) with no external language process. It is kept as a self-contained crate (not fused into the core) so it can be **extracted to a standalone crate later** if it outgrows Cronus; for now it is vendored in-tree because no other consumer needs it. The core wires its step handlers to Cronus subsystems. Execution is schema-driven, validated, and bounded.

## Related Specifications

- [l1-workflow-language.md](l1-workflow-language.md) - The language model this runtime implements.
- [l2-core-library.md](l2-core-library.md) - The core depends on this runtime crate and binds its steps.
- [l2-source-layout.md](l2-source-layout.md) - Where this in-monorepo crate sits in the Cronus workspace.
- [l2-orchestration.md](l2-orchestration.md) - Delegated work / `/goal` loops execute workflows.
- [l2-model-router.md](l2-model-router.md) - Generation/analysis steps route models here.
- [l2-cli.md](l2-cli.md) - Command grammar standard for `workflow` commands.

## 1. Motivation

The language must run on every Cronus target — including the mobile thin client and the always-on hub — without a heavy external interpreter. Implementing the runtime in the Rust core (rather than embedding a separate language runtime) keeps it embeddable, fast, and dependency-free, satisfying the hub-and-spoke and mobile constraints.

## 2. Constraints & Assumptions

- The runtime is an in-monorepo Rust crate the core depends on; it links in-process (no external language runtime is bundled, no separate process).
- A formal grammar drives the parser; a schema is loaded before execution.
- The port preserves behavior parity with the reference implementation (same sample workflows produce equivalent validation, execution, and transpilation results).
- Steps call core subsystems through internal interfaces; the runtime owns no domain logic of its own beyond control flow.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| WFL-1 Dual representation | The transpiler converts compact ↔ human losslessly; both parse to the same AST. |
| WFL-2 Schema contract | The validator loads the schema first; unknown vocabulary fails validation. |
| WFL-3 Hard constraints | The executor enforces declared hard constraints; a violation halts and escalates, regardless of caller. |
| WFL-4 Preferences soft | Preferences are advisory inputs to steps; never override hard constraints. |
| WFL-5 Validate before run | `run` invokes the validator (lint rules) first; parse/undefined-var errors halt. |
| WFL-6 Bounded execution | The executor enforces max-iteration/budget limits and honors halt/pause. |
| WFL-7 Subsystem-bound | Command handlers dispatch to memory, HITL, orchestration, quality, and the model router. |
| WFL-8 Result contract | Every run returns a structured result (success/failure) and runs the declared error handler. |
| WFL-9 Human view | The client surface renders the human form via the transpiler. |

## 4. Detailed Design

### 4.1 Pipeline (all in the Rust core)

```mermaid
graph LR
    SRC[workflow source] --> LEX[lexer] --> PARSE[parser -> AST]
    PARSE --> VAL[validator + lint]
    VAL --> EXEC[executor]
    PARSE --> TRANS[transpiler: compact <-> human]
    EXEC --> RES[structured result]
```

A formal grammar specification drives the parser; porting from the reference implementation's grammar is the starting point. Lint rules (errors/warnings/info) run in the validator.

### 4.2 Step binding

```mermaid
graph TD
    STEP[workflow step] --> DISP{command category}
    DISP -->|generate/analyze| MR[model router]
    DISP -->|remember/recall/forget| MEM[memory subsystem]
    DISP -->|ask/confirm/escalate| HITL[human-in-the-loop]
    DISP -->|route| ORCH[orchestration]
    DISP -->|validate| QA[quality gates]
    DISP -->|fetch/read/log/publish| SVC[core services]
```

The runtime is the scripting layer; each command handler calls the owning subsystem (WFL-7), so workflows compose existing capabilities rather than duplicating them.

### 4.3 Embeddability

Because the runtime is a Rust crate the core links in-process, it executes on desktop and mobile alike — there is no separate language process on any target. The always-on hub runs workflows for autonomous routines/goals; the mobile thin client can validate/preview and run foreground workflows. Keeping it a self-contained crate (not fused into the core) preserves a clean seam for later extraction while still linking statically into the core build.

### 4.4 Command surface

Workflow operations conform to the CLI grammar standard (see `l2-cli.md` §4.4).

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| scaffold | `cronus workflow new <name>` | `/workflow new <name>` | `workflows.scaffold(name) -> Workflow` |
| validate | `cronus workflow validate <file>` | `/workflow validate <file>` | `workflows.validate(ref) -> Report` |
| run | `cronus workflow run <file>` | `/workflow run <file>` | `workflows.run(ref, input) -> Result` |
| transpile | `cronus workflow transpile <file> --to <compact\|human>` | `/workflow transpile …` | `workflows.transpile(ref, mode) -> string` |
| test | `cronus workflow test [<file>]` | `/workflow test …` | `workflows.test(ref?) -> Report` |

### 4.5 Port architecture & strategy

The crate is a behavior-preserving port of the reference implementation (~5k lines across six modules) into Rust. Modules and their responsibilities:

| Module | Responsibility | Reference scope |
| --- | --- | --- |
| `lexer` | tokenize the compact form | ~tokens + symbols/operators |
| `parser` → `ast` | build the AST per the formal grammar | grammar-driven; largest module |
| `validator` | structure + lint rules (errors/warnings/info) | the lint catalog |
| `executor` | step dispatch, control flow, bounded execution | command handlers + control keywords |
| `transpiler` | compact ↔ human, lossless | rendering both forms |

Schema and grammar are **data, not code**: the vocabulary schema and the formal grammar ship as resources the crate loads (so updating the language does not require recompiling logic).

**Incremental order (vertical slice first):**

1. `lexer` + `parser` + `ast` — parse a sample workflow to an AST.
2. `transpiler` — compact ↔ human round-trip on that AST (proves WFL-1).
3. minimal `executor` — a couple of commands (`log`, `generate`) end-to-end (proves WFL-7/8).
4. `validator` + full lint rules (proves WFL-5).
5. full command set + control flow (`?if`/`?switch`/`~retry`/`~map`/`!halt`/`!pause`).

**Parity testing:** the reference implementation's sample workflows + lint cases form a golden corpus; the Rust crate must produce equivalent validation verdicts, execution results, and transpilation output. <!-- TBD: extract the reference test corpus into shared fixtures -->

## 5. Drawbacks & Alternatives

- **Porting effort:** re-implementing lexer/parser/validator/executor/transpiler in Rust is real work; mitigated by an existing formal grammar and lint catalog to port from.
- **Schema drift vs runtime:** the runtime must track the schema version it supports. <!-- TBD: runtime↔schema version compatibility checks -->
- **Alternative — embed an external interpreter:** rejected; it breaks the embeddable/mobile constraint (no heavy runtime on device).
- **Alternative — standalone crate in its own repository:** deferred; vendored in-tree for now since no other consumer needs it. The self-contained crate boundary keeps later extraction cheap if that changes.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[LANG]` | `.design/main/specifications/l1-workflow-language.md` | Invariants this runtime implements |
| `[CORE]` | `.design/main/specifications/l2-core-library.md` | The core that hosts the runtime |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

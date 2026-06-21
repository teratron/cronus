---
phase: 3
name: "Stem — CLI (`crates/cli`)"
status: Done
subsystem: "crates/cli (command-line frontend)"
requires:
  - "core public contract (core::Capabilities / Engine) — Phase 1 T-1C01"
  - "nodus library API (workflows::*) — Phase 2 T-2F01"
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 3 Tasks — Stem: CLI (`crates/cli`)

**Phase:** 3
**Status:** Done
**Strategic Goal:** Grow the first usable surface from the seed — the `cronus` binary. It parses shell commands and flags, delegates to core/nodus, and renders results as text. No domain logic lives here (INV-2); the CLI is a thin binding layer over the library API. Phase 3 delivers: command framework + grammar skeleton + `help`, `init`, `status`, `workflow` command group (scaffold/validate/run/transpile). Further commands attach in Phases 4–6 as their subsystems land.

> **Scope boundary**: Phase 3 owns the command framework and the commands available now — `help`, `init`, `status`, and the full `workflow` sub-group (backed by `crates/nodus`). Memory, orchestration, kanban, and goal commands are added in their owning phases (4–6). The TUI mirrors these in Phase 7.
> **Grammar rule (l2-cli.md §4.4):** `cronus <noun> <verb> [<id>] [--flag <value>]` — verb-first, explicit verbs, no terse aliases. The library method is the source of truth; CLI/TUI are thin bindings.
> **Dependency guard**: CLI links `crates/core` and `crates/nodus` directly (no IPC on hub host). Both library contracts are stable (Phase 1 T-1C01, Phase 2 T-2F01 Done).

## Atomic Checklist

Track A — Crate scaffold (gates all CLI tracks)

- [x] [T-3A01] `crates/cli` binary scaffold — Cargo.toml + main.rs + clap command skeleton + exit codes + output mode

Track B — Core commands

- [x] [T-3B01] `cronus help` + `cronus status` — usage discovery + current workspace/phase display
- [x] [T-3B02] `cronus init [path]` — workspace initialization via core state bootstrap

Track C — Workflow command group

- [x] [T-3C01] `cronus workflow scaffold <name>` — generates `.nodus` file skeleton
- [x] [T-3C02] `cronus workflow validate <file>` — validates + renders diagnostics; exits non-zero on errors
- [x] [T-3C03] `cronus workflow run <file> [--input '{}']` — executes workflow + renders RunResult
- [x] [T-3C04] `cronus workflow transpile <file> [--human|--compact]` — renders transpiled form to stdout

Track T — Validation & parity

- [x] [T-3T01] CLI integration tests + command exit-code contract

## Detailed Tracking

### [T-3A01] `crates/cli` binary scaffold

- **Spec:** l2-cli.md §4, §4.4 (command grammar)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo build -p cli` exits 0; `cronus --help` lists `help`, `init`, `status`, `workflow` sub-commands; `cronus --version` prints version from Cargo.toml; `cronus unknown-cmd` exits code 2 (usage error); `--format json` flag is accepted by the root command.
- **Notes:** Use `clap` with derive macros (`#[derive(Parser, Subcommand)]`). Root command: `Cli { format: OutputFormat, command: Command }` where `OutputFormat` is `Text | Json`. `Command` enum has `Help`, `Init`, `Status`, `Workflow`. Exit code convention: 0 = success, 1 = runtime/logical error, 2 = usage/parse error. `OutputFormat` is propagated to all sub-handlers via a shared `Context` struct. `cargo clippy --all-targets -- -D warnings` must pass. Justified dependency: `clap` (argument parsing — no viable std alternative).

### [T-3B01] `cronus help` + `cronus status`

- **Spec:** l2-cli.md §4.1, §4.2 (help/status commands; non-interactive output mode)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cronus help` exits 0 and prints the same text as `cronus --help`; `cronus status` exits 0 and prints the workspace name + current phase (reads from `core::state::State`); both commands honour `--format json` (status outputs a JSON object with `workspace` and `phase` keys); unit test `status_reads_state` passes.
- **Notes:** `help` is provided free by clap — just verify the output. `status` reads `core::state::State` (or equivalent from T-1C01) and renders it. If the state file is absent (workspace not initialized), exit 1 with a clear message: `"No workspace initialized. Run 'cronus init' first."` Text renderer: one key-value line per field; JSON renderer: `serde_json` object (justified new dep for JSON output — no std alternative). No domain logic.

### [T-3B02] `cronus init [path]`

- **Spec:** l2-cli.md §4.1; l2-filesystem-layout.md (OS-native path + state bootstrap)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cronus init` in a temp dir creates the state file (check with `cargo test -p cli cmd_init_creates_state`); `cronus init /tmp/test_ws` creates `/tmp/test_ws/` and the state file inside; running `cronus init` twice in the same directory prints a warning and exits 0 (idempotent); `--format json` outputs `{"result":"initialized","path":"<abs-path>"}`.
- **Notes:** Calls `core::state` bootstrap (T-1C01). Creates `.cronus/` directory layout under the target path per l2-filesystem-layout.md. CLI side: parse optional `path` positional arg, default to `std::env::current_dir()`. Error case: permission denied → exit 1 with message. No domain logic — pure delegation + rendering.

### [T-3C01] `cronus workflow scaffold <name>`

- **Spec:** l2-cli.md §4.1; l2-workflow-runtime.md §4.4 (library `scaffold`)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cronus workflow scaffold hello_world` writes `hello_world.nodus` to the current directory; the written file parses without error via `nodus::Parser::parse`; `--out path/to/out.nodus` writes to a custom path; if the target file already exists, exit 1 with message "File already exists: `<path>`"; `cargo test -p cli cmd_workflow_scaffold` passes.
- **Notes:** Calls `nodus::workflows::scaffold(name)`, then `nodus::transpiler::Transpiler::to_nodus(&ast)` to serialize, then writes to disk. Output: `"Scaffolded: <path>"` on success. JSON mode: `{"result":"scaffolded","path":"<abs-path>"}`.

### [T-3C02] `cronus workflow validate <file>`

- **Spec:** l2-cli.md §4.1; l2-workflow-runtime.md §4.4 (library `validate`)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cronus workflow validate simple_log.nodus` exits 0 and prints `No errors.` (or W/I-class diagnostics); `cronus workflow validate lint_missing_runtime.nodus` exits 1 and prints the E001 diagnostic; `--format json` outputs `{"errors":[...],"warnings":[...],"infos":[...]}` arrays; `cargo test -p cli cmd_workflow_validate` passes.
- **Notes:** Calls `nodus::workflows::validate(source, filename)`. Text renderer: one line per diagnostic — `[E001] Missing §runtime block (line 0)`. Exit code: 0 if no Error-severity diagnostics, 1 otherwise. Filename is derived from the path's file name component for the validation call.

### [T-3C03] `cronus workflow run <file> [--input '{"key":"val"}']`

- **Spec:** l2-cli.md §4.1; l2-workflow-runtime.md §4.4 (library `run`)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cronus workflow run simple_log.nodus` exits 0 and prints `Status: Ok`; `cronus workflow run lint_missing_runtime.nodus` exits 1 and prints the E001 rejection; `--input '{"query":"hello"}'` seeds `$in.query`; `--format json` outputs the full `RunResult` as JSON; `cargo test -p cli cmd_workflow_run` passes.
- **Notes:** Calls `nodus::workflows::run(source, filename, input)`. Parse `--input` as JSON into `Value::Map`. Text renderer: `Status: <status>`, then `Log:` section with one line per `LogEntry`, then `Out: <out>` if not Null. On validation failure (Err path), print diagnostics and exit 1. On runtime failure (`Status::Failed`), print errors and exit 1.

### [T-3C04] `cronus workflow transpile <file> [--human|--compact]`

- **Spec:** l2-cli.md §4.1; l2-workflow-runtime.md §4.4 (library `transpile`)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cronus workflow transpile simple_log.nodus` (default compact) prints the compact NODUS form to stdout and exits 0; `cronus workflow transpile simple_log.nodus --human` prints the human-prose form containing `WORKFLOW` and `STEPS`; output is pipeable (`cronus workflow transpile simple_log.nodus > out.nodus`); `cargo test -p cli cmd_workflow_transpile` passes.
- **Notes:** Calls `nodus::workflows::transpile(source, mode)`. Default mode is `TranspileMode::Compact` (spec §4.1 parity: agents author the compact form). `--human` switches to `TranspileMode::Human`. Prints directly to stdout (no "Out:" prefix — the output IS the document). Exit 1 on parse error.

### [T-3T01] CLI integration tests + exit-code contract

- **Spec:** l2-cli.md §3 (INV-2: no domain logic in CLI), §4.2 (headless invocation + exit codes)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cli` — all tests pass; exit-code contract: 0 = success, 1 = logical/runtime error, 2 = usage error; each command in T-3B01..T-3C04 has at least one `#[cfg(test)]` unit test in its module; a `tests/cli_smoke.rs` integration test runs `cronus --help` and `cronus workflow validate` end-to-end via `std::process::Command`. `cargo clippy --all-targets -- -D warnings` clean.
- **Notes:** `tests/cli_smoke.rs` links the binary as a subprocess (`std::process::Command::new(env!("CARGO_BIN_EXE_cli"))`). Tests cover: help exits 0, unknown command exits 2, validate clean exits 0, validate error exits 1, run clean exits 0, transpile outputs non-empty. No mocking of core/nodus — tests drive the real library code.

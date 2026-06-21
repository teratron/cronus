# Agents Instructions

Cronus is a polyglot monorepo: a Rust workspace (`crates/`) for the engine and binaries, a JS/TS layer (`packages/`) for the UI, and a Tauri shell (`apps/`). These rules cover how agents write and verify code. Source files reference no design/spec artifacts — restate rationale in plain language.

## Rust (`crates/`)

### Implementation

- **Clean Code**: idiomatic Rust; small, well-named modules; dependencies point inward to `core`.
- **Dependency Policy**: prefer the standard library; add a third-party crate only when strictly necessary and justified.
- **Error Handling**: no `unwrap()` / `panic!()` on production paths — use `Result` / `Option`. Reserve panics for truly unrecoverable invariants.
- **Tests (mandatory)**: every feature or fix has tests. Unit tests in a `#[cfg(test)] mod tests` inside the source file; integration tests in that crate's `crates/<crate>/tests/`.
- **Benchmarks**: performance-critical logic gets benches in that crate's `crates/<crate>/benches/`.
- **Logging**: descriptive `tracing` (or `log`) events; critical errors logged with enough context for session-level debugging.
- **Secrets & data**: never read or log secrets in the clear; never write secrets to tracked files; keep user data on-device unless the user explicitly authorizes egress.

### Verification (per affected crate or workspace-wide)

- `cargo check` — compiles.
- `cargo clippy --all-targets -- -D warnings` — zero lints.
- `cargo test` — all tests pass.
- `cargo bench` — when performance-relevant.
- `cargo fmt --all` — consistent style.

## Frontend (`packages/` — TypeScript / React)

### Implementation

- **Presentation only**: the UI calls the core over the IPC bridge; no business logic in TypeScript.
- **Type-safe**: no `any` on public surfaces; externalize all user-facing strings (localization); honor the theme/design-token system.

### Verification (per affected package)

- `pnpm -C packages/<pkg> test` (vitest) — tests pass.
- lint + format (`biome`) — zero errors.
- `tsc --noEmit` — type-checks.
- `fallow audit --changed-since <base>` — no new dead code, duplication, circular dependencies, or architecture-boundary violations (structural gate; `--format json` in CI). The boundary rules enforce presentation-only UI with inward-pointing dependencies.

## Conventions

- **CLI grammar**: verb-first with flags, explicit verbs, command groups — `cronus <group> <verb> [--flags]`. The TUI mirrors each with a leading slash; the library method is the source of truth and the CLI/TUI are thin bindings.
- **Definition of done**: a change is done only when its required quality gates pass — tests + lint + type/format always; benchmarks for performance-relevant changes; security review for sensitive ones.
- **One source of truth**: domain logic lives in the core; frontends and shells hold none.

## Completion Protocol (Mandatory Checklist)

Before declaring a task complete, verify:

- [ ] Required quality gates are green for every touched crate/package (tests + lint + type/format).
- [ ] No `unwrap()` / `panic!()` on production paths; no secrets in tracked files or logs.
- [ ] New or changed behavior is covered by tests (unit in-module; integration per-crate).
- [ ] Technical content (code, comments, docs) in English; conversational replies in Russian.
- [ ] No design/spec-layer references leaked into source files.

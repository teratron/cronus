# Project Context

**Generated:** 2026-07-31

## Active Technologies

- Node.js
- Rust

## Core Project Structure

```plaintext
.
├── .agents/
│   ├── rules/
│   ├── skills/
│   └── workflows/
├── .cargo/
│   └── config.toml
├── .claude/
│   ├── commands/
│   ├── rules/
│   ├── scheduled_tasks.lock
│   └── skills/
├── .codex/
│   ├── prompts/
│   ├── rules/
│   └── skills/
├── .design/
│   ├── .cache/
│   ├── .graph-cache/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── graph-before.json
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── UX-UI - разбор 3 кейсов.md
│   ├── UX-дизайн - 6 психологических принципов.md
│   ├── desktop.drawio.svg
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── release.drawio.svg
│   ├── technology-stack-research.md
│   └── ui-ux.md
├── .env
├── .env.example
├── .github/
│   ├── dependabot.yml
│   └── workflows/
├── .gitignore
├── .markdownlint.json
├── .release/
│   ├── program/
│   └── state/
├── AGENTS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── apps/
│   └── desktop/
├── biome.json
├── crates/
│   ├── activation-os/
│   ├── auth-local/
│   ├── cli/
│   ├── codegraph/
│   ├── contract/
│   ├── core/
│   ├── domain/
│   ├── model-local/
│   ├── nodus/
│   ├── store-local/
│   └── tui/
├── docs/
│   └── README.md
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── rust-toolchain.toml
└── scripts/
    └── check-domain-boundary.mjs
```

## Recent Changes

- T-24T01: Added 4 unit tests in `portability.rs` (`effect_class_of` classification + gate strings) and 5 integration tests in `tests/portability.rs` (permit/deny on both `ModelCall` and `Deferred`, plus a `NoopPolicyProvider` regression). Testing caught a wrong assumption empirically: a denied step's pipeline target is not *absent* from `RunResult.vars` but stays at its seeded default (`Value::Null` for `out`), since reserved variables are always pre-populated at context construction — fixed the assertions accordingly, which also strengthened the previously-vacuous permitted-effect checks
- T-24T02: `cargo test -p nodus` — 444 passed (was 435; +9), 0 failed; clippy `-D warnings` clean; fmt found one line-wrap violation in the new tests (no logic change), fixed; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 25 — Effect Risk-Class Declaration (l1-nodus-portability §4.12 · l2-nodus-portability §4.10 · LP-16) (2026-07-31)

- T-25A01: Extended `execute_command`'s existing LP-11 `context` literal with a small loop reading `+reversible`/`+external`/`+value` off `cmd.modifiers`, pushing each stripped-prefix pair only when present — descriptors are omitted, never defaulted, when a step carries no matching modifier. No new DSL grammar, AST field, `Value` kind, or `PolicyProvider` signature change; the only production-code change in this phase
- T-25C01: `l2-nodus-portability` 1.6.0 → 1.6.1 — §4.10 header + §4.10.2's `[REFERENCE]` pseudocode (corrected to show the real inline literal, not the illustrative separate `build_context` function), §3.1's LP-16 row, its Leverage paragraph's invariant count, and §5 item 7 reconciled to Implemented/Done; `INDEX.md` row + top-level version synced
- T-25T01: Added 3 integration tests in `tests/portability.rs` — a decorated `GEN` step's descriptors reach a capturing `PolicyProvider`'s `context` verbatim (`risk_descriptors_reach_context_when_declared`); an undecorated step's `context` omits all three keys entirely (`undeclared_risk_descriptors_are_absent_from_context_not_defaulted`); a `NoopPolicyProvider`/plain-`run` regression proving the modifiers are inert without a consulting host (`risk_descriptors_are_inert_without_a_policy_provider`). All three passed on first run — no empirical surprises, unlike Phase 24's reserved-variable-seeding catch. `cargo test -p nodus` — 447 passed (was 444; +3), 0 failed; clippy `-D warnings` clean; fmt found one line-wrap violation in the new tests (no logic change), fixed; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]` in `executor.rs`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 26 — Uncaught-Error Handler Dispatch (l1-nodus-language §4.3/NL-9 · l2-nodus-error-dispatch §4.1–§4.5) (2026-07-31)

- T-26A01: `execute_inner`'s main step loop gained a structural dispatch check — `errors_before_this_step` captured before `run_step_with_retry`, compared after; a `Signal`-free step that left a new `ctx.errors` entry behind populates `$error` via a new `error_to_value(&RuntimeError) -> Value` helper, dispatches the declared `@err:` handler via the ordinary `execute_command` path (reusing the triggering step's own `step_num`), then `break`s the loop. No new type, field, error code, or `PolicyProvider`/`AuditProvider` change
- T-26C01: `l2-nodus-error-dispatch.md` 1.0.0 → 1.0.1 — §4.3's pseudocode corrected (`wf` → `ast`, collapsed to a let-chain per `clippy::collapsible_if`, dropped the unused `err_dispatched` flag); §4.5 corrected — `try_parse_command_from_string` returns `None` only for genuinely empty `@err:` text, never for unrecognized-but-non-empty text (any non-empty text becomes `Some(CommandCall)` and dispatches harmlessly as `UNKNOWN_COMMAND`). `l2-nodus-errors.md` 1.1.1 → 1.1.2 and `l2-nodus-runtime.md` 1.3.1 → 1.3.2 — both NL-9 rows updated from "specified"/"realized by" to Implemented. `INDEX.md` rows + top-level version synced
- T-26T01: Added `ERR_HANDLER_WF`/`EMPTY_ERR_HANDLER_WF`/`NO_ERR_HANDLER_WF`/`RETRY_ERR_HANDLER_WF` fixtures, `DialogRejects`, `DenyOncePolicy` (denies exactly the first `evaluate` call), and 5 integration tests in `tests/portability.rs` covering: dispatch on `POLICY_DENIED` and `DIALOG_REJECTED`, no-handler/empty-handler no-op, and retry-then-succeed never dispatching. All 5 passed on first run
- T-26T02: **Found and fixed one real regression beyond the spec's own anticipated blast radius**: `tests/control_flow.rs`'s pre-existing `retry_reruns_failing_step_up_to_bound` asserted "the step after an exhausted retry still runs" — true only because dispatch didn't exist yet; `RETRY_TIMEOUT_WF` already declares `@err: ESCALATE(human)`, and `l1-nodus-language.md`'s own `~RETRY:n` row has always said exhaustion "routes to `@err:`", so the old assertion was pinning exactly the gap this phase closes. Corrected to assert the handler dispatches and the following step does not run. Reviewed Phase 24's `policy_denies_model_call_effect`/`policy_denies_deferred_effect` and confirmed their assertions still hold, not vacuously — no changes needed. `cargo test -p nodus` — 452 passed (was 447; +5), 0 failed; clippy `-D warnings` clean after one `collapsible_if` fix; fmt clean after auto-fix; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]` (the one production `.expect()` is structurally unreachable-as-`None`, proven by its guard condition); `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)


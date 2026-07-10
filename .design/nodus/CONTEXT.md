# Project Context

**Generated:** 2026-07-10

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
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
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
│   ├── cli/
│   ├── codegraph/
│   ├── core/
│   ├── nodus/
│   └── tui/
├── docs/
│   └── README.md
├── firebase-debug.log
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
└── rust-toolchain.toml
```

## Recent Changes

## Phase 9 — Closed Vocabulary Registries (l2-nodus-registries) (2026-06-27)

- T-9A01: Added `KNOWN_FLAGS` (12 analysis extractors), `KNOWN_VALIDATORS` (12 validator names), and `PRIMITIVE_TYPES` (10 field types) closed registries to `vocab.rs`
- T-9A02: Added `Schema::is_known_flag` / `is_known_validator` (matches the pre-colon name, so `len:32` resolves to `len`) / `is_known_type` query methods
- T-9B01: Added advisory validator diagnostics `W011` (unknown `~flag`), `W012` (unknown `^validator`), `W013` (unknown `@in` field type); warnings never set `ValidationReport::has_errors`, so unknown host vocabulary degrades gracefully (NL-1/NL-7/NL-9 strengthening)
- T-9T01: `cargo test -p nodus` — 228 passed (was 222; +6), 0 failed; clippy `-D warnings` clean; fmt clean; doc only the pre-existing `test`-fn baseline; SDD §6 clean; no fixture regressed to an error (registry checks are advisory)

## Phase 10 — Human-in-the-Loop Dialog (l2-nodus-dialog) (2026-06-27)

- T-10A: Added `ASK`/`CONFIRM` to `vocab::KNOWN_COMMANDS`; added `Status::Paused` and `Signal::Pause` to the executor
- T-10B: Added `DialogOutcome` (Answer/Pause/Timeout/Rejected), the `DialogProvider` trait, and the synchronous `DefaultDialogProvider` (resolves from `+default`, else `Pause`; no I/O)
- T-10C: Executor `handle_dialog` dispatch — `Answer` binds the typed value; `Pause` suspends (`Status::Paused`) with a `ResumeDescriptor` (workflow + var snapshot + step index) and no later step; `Timeout`/`Rejected` push `NODUS:DIALOG_TIMEOUT`/`NODUS:DIALOG_REJECTED`; events carry length descriptors only (DG-7)
- T-10D: Added `ExtensionRole::Dialog`; `CapabilityManifest::from_workflow` requires it for an `ASK`/`CONFIRM` lacking `+default` (refactored the command walker to inspect modifiers); `HostCapabilities::builtin()` omits Dialog
- T-10E: Added `run_with_dialog` / `run_with_dialog_and_audit` (workflows.rs) + `lib.rs` re-exports of `DialogProvider`/`DialogOutcome`/`DefaultDialogProvider`/`ResumeDescriptor`
- T-10T: `tests/dialog.rs` — 7 DG-invariant integration tests (default resolution, pause+resume descriptor, typed binding, timeout/rejection errors, manifest Dialog derivation); `cargo test -p nodus` — 237 passed (was 228; +9); clippy `-D warnings` clean; fmt clean; doc only the pre-existing baseline; SDD §6 clean


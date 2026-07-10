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

- T-9D03: telemetry — opt-in gate (default off, no-op recording while opted out), closed metric-name allowlist, a payload enum with no free-text variant, opt-out drops the queue
- T-9T01: cross-subsystem hardening integration — a seeded secret proven absent from a real backup archive, a real restored tier, and a scrubbed report preview; both consent gates (report, telemetry) proven to block by default; workspace-wide `cargo test`/`clippy -D warnings`/`fmt --check` all clean
- Verify: `cargo test --workspace` green across all 5 crates; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean

## Phase 12 — Skill System (Two-Tier Stores & Canonical Stack) — 2026-07-10

- T-12A01: `skills::store` — two-tier stores (`Preset`/`State`/`Workspace`) with shadowing precedence (`resolve`: workspace > state > preset, first match wins); `write` structurally rejects `SkillTier::Preset` (the program tier is seeded once via `with_presets`, never through `write`); an override identical to the shadowed preset returns `WriteOutcome::IdenticalToPreset`, not an error
- T-12A02: `skills::package` — canonical package validation: required `SKILL.md`/`extension.json`, every other top-level entry checked against a closed allow-list (rejects `scripts/` and any other unknown material), `origin/` contents exempted from the allow-list entirely (never classified), manifest `kind` must be `Skill` (reuses `extensions::ExtensionManifest`/`validate_manifest`, EXT-9)
- T-12B01: `skills::commands` — `CommandSpec` registry (id/category/input_schema/required_grants/surface_version, the last stamped only from a module constant); `CommandRegistry::check_dispatch` validates input against schema before checking `RequiredGrant::{Fs,Network,Secrets}` against the caller's manifest permissions
- T-12B02: `skills::exec` — `WorkflowRuntime` trait seam; `activate()` short-circuits to `ActivationResult::InstructionOnly` before touching the runtime at all when degraded or workflow-less; otherwise validates, executes, then runs a per-call grant check on every dispatched `OperationStep`
- T-12C01: `skills::convert` — the six-stage pipeline (verify → classify → retain → transpile → degrade → report) as one pure function; a missing/invalid witness denies before anything else is inspected; any unmapped script demotes the whole package to `Degradation::InstructionOnly`; every original is preserved under `origin/` regardless of outcome; the final `validate_package` call is the atomicity gate — nothing lands on `Err` because nothing is written until a caller lands the returned `Ok` value
- T-12C02: `skills::synthesize` — lands already-authored content (the model call itself is a seam) as `source: generated`; the one genuine lint at this layer is workflow-pair completeness (`workflow.nd`/`workflow.md` must land together or not at all); `ActivationPolicy::RequiresReview` is the only variant — the spec's open TBD resolved conservatively, no auto-activate path exists
- T-12D01: `cronus ext skill import|create|status` — nested under the existing `Ext` group (no new top-level command group); `import` reads one file and runs it through `convert()`; `create` runs an instruction-only `AuthoredSkill` through `synthesize()`; `status` reports store origin/degradation/pending-review via a separately-testable `compute_status()`; extended `SkillEntry` with `degraded`/`pending_review` fields through a backward-compatible `with_status()` builder
- T-12T01: invariant-compliance sweep — 10 integration tests in `crates/core/tests/skill_system.rs`, one per Invariant Compliance row (EXT-1/2/3/4+6/5+STO-1/7/8/9/11, STO-3), exercising convert→store, convert→exec, and synthesize→store together; CLI/TUI/library parity confirmed structurally (TUI's `/ext` slash command already covers the nested `skill` group at its existing granularity — no TUI-crate change needed)
- Verify: `cargo test --workspace` green across all crates (core 314 lib + 10 integration, cli 37 unit + 28 smoke, tui 198, nodus 34, codegraph, all others); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all` clean


# Project Context

**Generated:** 2026-09-12

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
├── .artifacts/
├── .cargo/
│   └── config.toml
├── .claude/
├── .codex/
├── .design/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── main/
│   ├── nodus/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── UX-UI - разбор 3 кейсов.md
│   ├── UX-дизайн - 6 психологических принципов.md
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── qa-remediation-progress.md
│   ├── references.md
│   ├── reverse-derivation-mechanism.md
│   ├── technology-stack-research.md
│   └── ui-ux.md
├── .env.example
├── .fallowrc.jsonc
├── .gitattributes
├── .github/
│   ├── dependabot.yml
│   └── workflows/
├── .gitignore
├── .magic/
├── .markdownlint.json
├── .qwen/
├── .release/
│   ├── program/
│   ├── project/
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
│   ├── conformance/
│   ├── contract/
│   ├── core/
│   ├── domain/
│   ├── model-local/
│   ├── nodus/
│   ├── simulation/
│   ├── store-local/
│   └── tui/
├── docs/
│   ├── README.md
│   ├── building.ru.md
│   ├── simulation.md
│   └── simulation.ru.md
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── rust-toolchain.toml
├── scripts/
│   └── check-domain-boundary.mjs
└── simulations/
    ├── README.md
    ├── broad/
    └── short/
```

## Recent Changes

- T-23D01: deferred-action lifecycle — `defer`/`resolve_deferred`; a detached action registers `Pending{action_id}` with no tag at dispatch, and mints only on correlated completion with the real observed result
- T-23T01: `crates/core/tests/tool_receipts_invariants.rs` — 11 tests, one per TR-1…TR-9 through the real facade export chain, plus 2 leak-path tests (redacted `Debug`, receipt token survives `redact::redact`)
- Verify: `cargo test --workspace` green across 3 consecutive full runs (75 `test result: ok` blocks each, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean

## Phase 30 — Usage-Simulation Harness (2026-09-11)

- T-30A01: the ports-tier `cronus-simulation` crate — `World::build`/`build_with_base` refuse a base that resolves inside this repository's own working tree; `cwd()`/`global_dir()` split so `CRONUS_PORTABLE_DIR` (the product's own existing portable-mode seam) isolates OS-native roots without four different per-OS overrides
- T-30A02: `Transcript::record` — every invocation captured byte-for-byte (argv/stdin/stdout/stderr/exit/duration) the moment it completes, bracketed by `World::state_digest()` before/after; new `product::resolve_binary` (explicit override, else sibling-of-current-exe discovery) after `CARGO_BIN_EXE_cronus` proved unavailable across package boundaries
- T-30B01: `replay::run` — a pinned route rebuilds a world and asserts each step's exit code with no agent in the loop; `tests/replays.rs` is the always-on lane, seeded with one hand-written case (`init-then-status.toml`); a positive control proves the check can fail
- T-30C01: `scenario::parse`/`validate` — refuses at parse time an empty or duplicate-id obligation set, no-evidence obligation, activity-shaped statement, indistinct pass signal, and an absence claim with no `positive_control`; a missing `bound` or an unknown top-level key is refused structurally by `serde`
- T-30C02: the root-level `simulations/` corpus — `README.md` plus the first `short`-tier scenario, `first-run-three-cards.md` (renamed from the plan's illustrative "three-boards" once the real CLI — one board, many cards — was checked); `tests/corpus.rs` guards the corpus against SDD-reference leaks with its own hand-rolled, self-tested matchers
- T-30D01: `run_state::RunState` — cross-process state as one `run-state.json` per world, since `world`/`run`/`note`/`verdict`/`finish` are separate subprocess invocations by design; `record_run` refuses once the scenario's bound is exhausted; `findings::RunOutcome` (pass/fail/incomplete/contaminated/void) computed from obligation verdicts alone, contamination checked first; the `cronus-sim` wrapper binary and its `pin` verb closing the discover→pin→guard loop into `tests/replays/`
- T-30D02: `coverage::CoverageReport` — the catalog complement (never a percentage) against the product's own shell-completion script, disclosed as a best-effort structural approximation rather than the exact locus/stability-filtered set no callable seam yet exposes; wired as `cronus-sim coverage`
- T-30T01: full-workspace validation — `cargo check`/`clippy -D warnings`/`fmt --check`/`test` all green in one pass (87 test-result blocks, 0 failed, no `-j 2` retry needed); USM-1…USM-12 mapped to their discharging code/tests, with USM-4/USM-6 named as partially discharged and USM-10 as a process discipline this phase's code does not mechanically check; structural and reference containment both clean on inspection
- Verify: `cargo test --workspace` green — 87 `test result: ok` blocks, 0 failed (37 of them new, in `cronus-simulation`); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean


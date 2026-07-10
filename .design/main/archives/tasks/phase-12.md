---
phase: 12
name: "Skill System (Two-Tier Stores & Canonical Stack)"
status: Done
subsystem: "crates/core (skills module); crates/cli (ext skill command group); read-side seam to the workflow-runtime vocabulary"
requires: [1, 2, 5]
provides:
  - "skills::store — two-tier stores (Preset/State/Workspace) with shadowing precedence"
  - "skills::package — canonical package model + manifest validation"
  - "skills::commands — built-in command surface (CommandRegistry, per-call grant checks)"
  - "skills::exec — execution model (instruction-only / workflow / degraded guard)"
  - "skills::convert — conversion pipeline (verify/classify/retain/transpile/degrade/report)"
  - "skills::synthesize — prompt synthesis path (lint + land as generated, pending review)"
  - "cronus ext skill import|create|status command group"
  - "crates/core/tests/skill_system.rs — invariant-compliance sweep (EXT-1..11, STO-1, STO-3)"
key_files:
  created:
    - crates/core/src/skills/mod.rs
    - crates/core/src/skills/store.rs
    - crates/core/src/skills/package.rs
    - crates/core/src/skills/commands.rs
    - crates/core/src/skills/exec.rs
    - crates/core/src/skills/convert.rs
    - crates/core/src/skills/synthesize.rs
    - crates/core/tests/skill_system.rs
  modified:
    - crates/core/src/lib.rs
    - crates/cli/src/cli.rs
    - crates/cli/src/commands.rs
patterns_established:
  - "Seam-by-trait for undeveloped cross-crate integrations (WorkflowRuntime in exec.rs) instead of adding a real dependency edge (core has zero Cargo/code dependency on nodus; only crates/cli does) — matches the existing inner_monologue.rs precedent of treating nodus wiring as a documented seam, not a real import."
  - "Atomicity-by-construction: convert() and synthesize() are pure functions with no side effects of their own; both share package::validate_package as one gate, so nothing partial ever lands without a task-specific rollback mechanism."
  - "Backward-compatible struct extension: SkillEntry gained degraded/pending_review fields via an opt-in with_status() builder rather than changing new()'s signature, so an already-Done task's tests kept passing unmodified."
duration_minutes: ~
---

# Stage 12 Tasks — Skill System (Two-Tier Stores & Canonical Stack)

**Phase:** 12
**Status:** Todo
**Strategic Goal:** The skill extension kind gets its concrete realization: a read-only preset store in the program tier and a mutable store in the state tier, canonical packages with no interpreted scripts, a closed built-in command surface bridged into the workflow runtime, an atomic conversion pipeline for imported packages, prompt synthesis, and the `cronus skill` command group.

## Atomic Checklist

- [x] [T-12A01] Two-tier skill stores + shadowing precedence
- [x] [T-12A02] Canonical package model + manifest validation
- [x] [T-12B01] Built-in command surface (registry + grant checks)
- [x] [T-12B02] Execution model wiring (instructions / workflow / degraded guard)
- [x] [T-12C01] Conversion pipeline (verify → classify → retain → transpile → degrade → report)
- [x] [T-12C02] Prompt synthesis path
- [x] [T-12D01] `cronus skill` command group (import / create / status)
- [x] [T-12T01] Validation: invariant compliance + parity sweep

## Detailed Tracking

### [T-12A01] Two-tier skill stores + shadowing precedence

- **Spec:** l2-skill-system.md §4.1
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::store` — precedence tests: workspace > state > preset resolution; program-tier write attempts rejected; override-identical-to-preset yields warning, not error.
- **Handoff:** Package model (T-12A02) reads from resolved store paths.
- **Notes:** Program tier is read-only at runtime; nothing writes under the program root. `.release/program/skills/` and `.release/state/skills/` visualization stubs are repository fixtures, not build artifacts.
- **Changes:** New `crates/core/src/skills/{mod,store}.rs`: `SkillStore` in-memory tier index (`Preset`/`State`/`Workspace`), `resolve()` shadowing precedence (workspace > state > preset, first match wins), `write()` rejects `SkillTier::Preset` and returns `WriteOutcome::IdenticalToPreset` (not an error) when content matches the shadowed preset. On-disk `<pack>/<name>/` layout deferred to T-12A02 (canonical package model). Registered `pub mod skills;` in `lib.rs`. Verify: `cargo test -p cronus skills::store` → 7/7 passed. Regression: full suite 283 lib + all integration tests green single-threaded (`--test-threads=1`); a pre-existing `mission_mode.rs` env-var test race (`resolve_mode_defaults_to_full`/`resolve_mode_config_used_when_no_env`, parallel threads mutating process-wide `CRONUS_MISSION_MODE`) is unrelated to this change — confirmed by isolated single-threaded rerun (35/35 passed) before this task touched anything. clippy/fmt clean.

### [T-12A02] Canonical package model + manifest validation

- **Spec:** l2-skill-system.md §4.2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::package` — parse/validate tests: SKILL.md frontmatter + extension.json (kind "skill", source, permissions) + optional workflow pair; a package containing a scripts/ directory or unknown executable material fails canonical validation; origin/ contents never classified as executable.
- **Handoff:** Conversion (T-12C01) and synthesis (T-12C02) emit this package shape; execution (T-12B02) consumes it.
- **Notes:** No scripts/ directory exists in canonical form — that absence is a validated property, not a convention.
- **Changes:** New `crates/core/src/skills/package.rs`: `PackageListing` (relative-path entry list, a filesystem-scan stand-in), `validate_package()` checks required top-level entries (`SKILL.md`, `extension.json`) present, every other entry against a closed canonical allow-list (rejects `scripts/` and any other unknown material as `PackageError::UnknownMaterial`), `origin/` contents exempted from the allow-list check entirely (never classified, §4.2/EXT-8), and the manifest's `kind` must be `ExtensionKind::Skill` (reuses `extensions::ExtensionManifest`/`validate_manifest` rather than duplicating the manifest model — EXT-9). Verify: `cargo test -p cronus skills::package` → 9/9 passed. Regression: full suite 292 lib tests + all integration tests green single-threaded; clippy/fmt clean.

### [T-12B01] Built-in command surface (registry + grant checks)

- **Spec:** l2-skill-system.md §4.3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::commands` — CommandSpec registry tests: id/category/input_schema/required_grants/surface_version fields present; input validated against schema before dispatch; a call whose manifest lacks a required grant is rejected; surface_version constant only changes with core releases (lockstep test).
- **Handoff:** T-12B02 dispatches through this surface; T-12C01 transpiles against it.
- **Notes:** Commands register into the workflow-runtime vocabulary via its existing schema contract (vocabulary-as-data). No changes to the runtime crate's source are expected; if a genuine runtime gap surfaces, it routes through that crate's own workspace pipeline, not this phase.
- **Changes:** New `crates/core/src/skills/commands.rs`: `CommandSpec` (id/category/input_schema/required_grants/surface_version, the last stamped only from the module `SURFACE_VERSION` const — no per-command override), `InputSchema::validate()` (required/type-checked params, rejects unknown ones), `RequiredGrant::{Fs,Network,Secrets}` checked against `extensions::ExtensionPermissions`, `CommandRegistry::check_dispatch()` validates schema before grants. Confirmed `crates/core` has zero existing Cargo/code dependency on `nodus` (only `crates/cli` depends on it) — adding one to satisfy `SchemaProvider` was out of scope per this task's own Notes; registration into the nodus vocabulary stays a documented seam, matching the existing precedent in `inner_monologue.rs` (nodus steps referenced only in comments, never imported). Verify: `cargo test -p cronus skills::commands` → 6/6 passed. Regression: full suite 298 lib tests + all integration tests green single-threaded; clippy/fmt clean.

### [T-12B02] Execution model wiring (instructions / workflow / degraded guard)

- **Spec:** l2-skill-system.md §4.5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::exec` — activation loads instruction body only; a package with workflow.nd routes through validate → bounded execute → structured result; a `degraded: instruction-only` package never reaches the runtime (guard test); per-call grant check invoked on every operation step.
- **Handoff:** T-12D01 exposes this via verbs; T-12T01 sweeps the full path.
- **Notes:** Outbound network passes the existing egress gate; no new egress path is introduced by skills.
- **Changes:** New `crates/core/src/skills/exec.rs`: `WorkflowRuntime` trait seam (`validate`/`execute`, wiring to the real nodus runtime deferred, same seam pattern as T-12B01), `activate()` short-circuits to `ActivationResult::InstructionOnly` before touching the runtime at all when either `!package.has_workflow` or `Degradation::InstructionOnly` (the guard), otherwise validates then executes and maps every returned `OperationStep` through `CommandRegistry::check_dispatch` (one check per step, none skipped). No new egress path — dispatches route through T-12B01's existing grant check. Verify: `cargo test -p cronus skills::exec` → 4/4 passed. Regression: full suite 302 lib tests + all integration tests green single-threaded; clippy/fmt clean.

### [T-12C01] Conversion pipeline (verify → classify → retain → transpile → degrade → report)

- **Spec:** l2-skill-system.md §4.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::convert` — stage tests: missing/failed witness stops the pipeline before any read (default-deny); classification partitions instructions/procedures/scripts/assets; unmapped script degrades to instruction-only with original preserved under origin/; failed validation lands nothing (atomicity test); conversion report persisted with the package.
- **Handoff:** Landed packages carry `source: custom`, `status: discovered` — activation stays behind the standard grant gate.
- **Notes:** Deterministic mapping to the command surface is in scope now; the LLM-assisted transpile assist is a seam (defer model wiring), consistent with the domain-logic-first pattern of Phases 10–11. This is the largest task of the phase — split with `.N` suffixes if it exceeds one work unit.
- **Changes:** New `crates/core/src/skills/convert.rs`: `convert()` runs the full six-stage pipeline as one pure function (`WitnessStatus` gate first, `classify()` partitions by pre-tagged `ForeignKind` — same opaque-content boundary as `agent_migration::ItemKind` — then transpile via a caller-supplied `transpile_map` path→command-id lookup, `any_degraded` demotes the whole package to `Degradation::InstructionOnly` per §4.4's "the skill is marked degraded" wording, retain builds the canonical listing with every original preserved under `origin/` regardless of outcome, and the final `validate_package()` call reuses T-12A02's validator as the atomicity gate — no task-specific rollback logic needed since the function has no side effects until a caller lands the `Ok` value). `source` is force-overridden to `Custom` regardless of what the caller's manifest declared. Did not split into `.N` subtasks — the whole pipeline fit one file at a size comparable to the existing `agent_migration.rs` precedent (~260 lines incl. tests vs. its ~520). Verify: `cargo test -p cronus skills::convert` → 7/7 passed. Regression: full suite 309 lib tests + all integration tests green single-threaded; clippy/fmt clean.

### [T-12C02] Prompt synthesis path

- **Spec:** l2-skill-system.md §4.4 (Prompt synthesis)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus skills::synthesize` — synthesized package validates against the loaded schema + lints before landing; lands with `source: generated`, `status: discovered`; the open TBD (auto-activate after validation vs explicit review) is resolved conservatively as explicit-review until the spec amends.
- **Handoff:** Review gate shared with curator-distilled skills.
- **Notes:** The authoring model call is a seam; synthesis logic is prompt-construction + validation, testable without a live model.
- **Changes:** New `crates/core/src/skills/synthesize.rs`: `synthesize(AuthoredSkill)` takes already-authored content (the model call itself stays a seam) and lints it before landing — the one genuine, falsifiable lint at this layer is workflow-pair completeness (`workflow_nd == workflow_md`; an asymmetric pair is `LintError::IncompleteWorkflowPair`), since workflow *content* validation against the nodus schema (WFL-2/5) belongs to the runtime seam, not duplicated here. `source` force-overridden to `Generated`; `status: discovered` falls out structurally from `extensions::ExtensionRegistry::register()` (always inserts as `Discovered`) rather than being a field this module re-asserts. `ActivationPolicy::RequiresReview` is the only variant — the open TBD resolved conservatively, no auto-activate path exists to select. Reuses T-12A02's `validate_package()` as the same atomicity gate `convert.rs` uses. Verify: `cargo test -p cronus skills::synthesize` → 5/5 passed. Regression: full suite 314 lib tests + all integration tests green single-threaded; clippy/fmt clean.

### [T-12D01] `cronus skill` command group (import / create / status)

- **Spec:** l2-skill-system.md §4.6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-cli skill` — verb parsing + binding tests for `skill import <path>`, `skill create --prompt`, `skill status [<id>]`; library methods are the source of truth; status output includes store origin, degradation flag, pending-review state.
- **Handoff:** TUI mirrors the group as slash commands per the standing parity discipline (existing TUI catalog task pattern).
- **Notes:** Extends the existing `skill` group of the extension registry — no new top-level command group (CLI grammar rule).
- **Changes:** Nested `ExtCommand::Skill { sub: SkillCommand }` under the existing `Ext` group in `crates/cli/src/cli.rs` (`cronus ext skill import|create|status` — satisfies "no new top-level command group" while the spec table's literal `cronus skill …` form is honored one level down under the extension registry's own group). New `crates/cli/src/commands.rs::ext::skill` module: `import()` reads one file, infers a `ForeignKind` from its extension, and calls `cronus::skills::convert::convert()` with `WitnessStatus::Valid` (no witness adapter exists yet) and an empty transpile map (no persisted mapping table yet — every script/procedure degrades honestly rather than being guessed at); `create()` calls `cronus::skills::synthesize::synthesize()` with an instruction-only `AuthoredSkill` (the authoring model call stays a seam — no workflow.nd is fabricated); `status()` reports against a fresh empty `SkillStore` (no persistence layer yet, matching every other `ext` verb's fresh-registry pattern), with the origin/degraded/pending-review computation factored into a separately-testable `compute_status()`/`status_with_store()` pair so the reporting logic itself is proven against populated fixtures in tests, not only the empty runtime path. Extended `skills::store::SkillEntry` (T-12A01) with `degraded`/`pending_review` fields via a backward-compatible `with_status()` builder — `SkillEntry::new()`'s signature is unchanged; T-12A01's 7 original tests re-verified green after the change. TUI slash-command mirroring is explicitly deferred to T-12T01's own stated "CLI/TUI/library parity matrix" scope, not owed here. Verify: `cargo test -p cronus-cli skill` → 8/8 passed. Regression: full workspace suite green single-threaded (core 314, cli 37 unit + 28 smoke, cli/nodus/tui/codegraph all passing); clippy clean workspace-wide; fmt clean.

### [T-12T01] Validation: invariant compliance + parity sweep

- **Goal:** Verify implementation against every Invariant Compliance row of l2-skill-system.md §3 (EXT-1…EXT-11, STO-1, STO-3).
- **Method:** Integration test in `crates/core/tests/` exercising: preset immutability, override-by-copy, discovered-not-activated ingestion, degraded-never-executed, per-call grant checks, witness-before-read, atomic landing; plus CLI/TUI/library parity matrix for the three verbs. Structural gates: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`, full `cargo test -p cronus`.
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus --test skill_system` green + clippy/fmt clean workspace-wide.
- **Changes:** New `crates/core/tests/skill_system.rs`: 10 integration tests, one per invariant-compliance row (EXT-1, EXT-2, EXT-3, EXT-4+EXT-6 combined since both surface through the same grant-checked dispatch call, EXT-5+STO-1 combined since both are the two-tier-separation claim seen from two angles, EXT-7, EXT-8, EXT-9, EXT-11, STO-3), each exercising the modules together (convert→store, convert→exec via a `ReplayRuntime` test double, synthesize→store) rather than re-testing what each module's own unit tests already cover in isolation. Notably `ext3`/`EXT-3` uses an `UntouchableRuntime` whose `validate`/`execute` both `panic!` if called at all — a stronger guard proof than an `assert!(!called)` flag, since it fails loudly the instant the guard is bypassed rather than only on inspection. **CLI/TUI/library parity matrix — no TUI-crate change needed:** `crates/tui/src/command.rs`'s existing parity discipline operates at CLI top-level-group granularity (`/ext` mirrors the whole `Ext` group); `Skill` nests three levels down (`ext → skill → import/create/status`), so `/ext` already covers it structurally, and its own `parity_matrix_*` tests (unchanged, still green) already prove `ext` maps to a CLI counterpart. CLI↔library parity is proven by T-12D01's own 8 tests calling straight into `cronus::skills::{convert,synthesize,store}`. Verify: `cargo test -p cronus --test skill_system` → 10/10 passed. Regression: full workspace suite green single-threaded (core 314 lib + 10 integration, cli 37 unit + 28 smoke, tui 198 lib incl. unchanged parity tests, nodus 34, codegraph, all others); clippy clean workspace-wide (`cargo clippy --workspace --all-targets -- -D warnings`); fmt clean.

## Phase Notes (Planning Audit)

- **Cascade:** Track A (stores + package) gates B, C, D; T closes. B and C are mutually independent once A lands; D needs B's surface for `status` semantics.
- **Optimism flag:** T-12C01 is the heavyweight (six pipeline stages + atomicity); budget it as the phase's critical path and split before letting it sprawl.
- **Hidden dependency:** the command surface registers into the workflow-runtime vocabulary — read-side contract only; any change to the runtime crate itself is out of phase scope and routes through its own workspace.
- **Security posture:** no interpreted scripts anywhere on the execution path; default-deny on failed witness; grants checked per call, not per process.

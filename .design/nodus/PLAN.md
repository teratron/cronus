# Implementation Plan

**Version:** 1.16.0
**Generated:** 2026-07-24
**Based on:** .design/nodus/INDEX.md v1.0.58
**Status:** Active

## Overview

Strategic plan for maturing nodus from an in-tree vendored crate to an independently extractable, production-ready workflow-language library. The arc runs: spec completeness → library hardening → extraction readiness → observability & extension framework → portability → testing → capability manifest (LP-8). Each new contract spec is implemented, then its L2 spec is synced to the realized Rust shape.

Execution mode: **Sequential** (spec correctness must precede hardening; hardening must precede extraction).

> **Sync (v1.16.0, 2026-07-24):** opened **Phase 13 — Declarative Configuration Surface**, decomposing the newly-authored `l2-nodus-config` (Stable, INDEX v1.0.58) into 9 atomic tasks across tracks A→C + validation (Sequential). This realizes **NL-20** — the net-new-weight obligation the v1.15.0 sync flagged: the `§config` field-declaration grammar now has a Stable L2 spec and a phased plan (parser stub → real parser + shape check + provider seam + `run_with_config`). The remaining v1.15.0 obligations (NL-19/21, LP-17/18/19, HO-14…HO-20) stay pending L2 realization specs in the Backlog.
>
> **Sync (v1.15.0, 2026-07-24):** registry advanced INDEX v1.0.47 → v1.0.57 through additive refinement of three already-`Stable`/`Done` L1 concept specs — **`l1-nodus-language` → 1.12.0** (NL-19 unforgeable frame markers & seam-anchored segmentation → main l1-tokenization-boundary; NL-20 `§config` validated declarative-configuration surface → main l1-declarative-configuration; NL-21 confidentiality-flow labeling → main l1-confidentiality-flow), **`l1-nodus-portability` → 1.13.0** (LP-17 settlement effect-sink, LP-18, LP-19 host-supplied exposure-switch seam), **`l1-nodus-observability` → 1.12.0** (HO-14 measurement-availability two-state numerics, HO-15 cross-run step identity, HO-16/HO-17, HO-18 exposure-switch manifest recording, HO-19 fault-identity contribution, HO-20 RunManifest-as-reproduction-recipe). All are additive invariants on specs whose L2 realizations are already `Done` — **carried as pending L2 Invariant-Compliance obligations; no new phase opened**: none yet has a `Stable` L2 spec defining its Rust shape, so no atomic task with a concrete `Verify` line can be authored (Verify-Line + Atomic-Task mandate). Net-new implementation weight concentrates in **NL-20** (`§config` field-declaration grammar — real lexer/parser/validator/AST work, not a host-supplied side-band), warranting a dedicated `l2-nodus-config` L2 spec before it can be phased. RULES parity re-synced v1.5.0 → v1.6.0. Environment (NE-1…NE-13) and Dialog (DG-1…DG-9) unchanged since Phase 12. (INDEX v1.0.57.)
>
> **Sync (v1.14.0, 2026-07-10):** opened **Phase 12 — Environment & Evaluation**, decomposing the newly-authored `l2-nodus-environment` (Stable) into 9 atomic tasks across tracks A→D + gates. The `l1-nodus-environment` Phase-0 concept-only marker is cleared — it is now realized by a phased L2. (INDEX v1.0.47.) Prior (v1.13.1): absorbed the orphaned l1-nodus-environment concept into Phase 0 and reconciled Phase 11 (Control-Flow) to Done+archived (its stale `[ ]` corrected to `[x]`). Registry raced ahead to INDEX v1.0.46+ through many additive refinement passes (NL-11…NL-18, LP-9…LP-16, HO-8…HO-13, NE-11…NE-13, DG-9/DG-10) — all additive invariants on already-planned Stable specs, carried as pending L2 Invariant-Compliance obligations, not new phases.

## Phase 0 — Requirements (Layer 1: Concept)

*Technology-agnostic language contracts. Must be Stable before Phase 1 begins.*

- [x] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1] — Stable
- [x] **Nodus Portability Contract** ([l1-nodus-portability.md](specifications/l1-nodus-portability.md)) [L1] — host neutrality + extension interface contract; LP-1…LP-7; ModelProvider + AuditProvider + future StorageProvider/PolicyProvider taxonomy; feedback distillation protocol; vocabulary layering model
- [x] **Nodus Observability Contract** ([l1-nodus-observability.md](specifications/l1-nodus-observability.md)) [L1] — execution observability protocol; HO-1…HO-6; AuditProvider role; 10-type event taxonomy (step_start/step_end/step_error/constraint_hit/branch_taken/loop_iteration/macro_enter/macro_exit/model_call/model_response); run manifest; data-safety boundary (no raw user text in traces)
- [x] **Nodus Environment & Evaluation Contract** ([l1-nodus-environment.md](specifications/l1-nodus-environment.md)) [L1] — the executable substrate an evaluation-driven improvement loop grades a workflow against: `EnvironmentProvider` as a 5th extension role beside Model/Audit/Schema/Dialog (NE-1); closed reset/step/evaluate lifecycle, deterministic per (task, seed, actions) (NE-2); trajectory as a projection onto the AuditProvider stream (NE-3); frozen-evaluation boundary (NE-4); typed reward-is-data (NE-5); addressable task catalog + profile (NE-6); instance isolation + idempotent release (NE-7); function-scoped auxiliary roles (NE-8); host-supplied metric neutrality (NE-9); capability-manifest `ExtensionRole::Environment` fail-fast (NE-10, LP-8); declared grading mode automated/judge/hybrid (NE-11); archivable content-addressable candidate tuple feeding a host outer-loop optimizer (NE-12, the l1-harness-optimization feed); budget-normalized graded runs (NE-13, the HX-11 realization). L2 realization `l2-nodus-environment` authored & Stable → implemented in **Phase 12**

## Phase 1 — Spec Completeness & Vocabulary Alignment

*Close open items in both specs; ensure the vocabulary table and Canonical References are authoritative.*

- [x] **Nodus DSL Language** ([l1-nodus-language.md](specifications/l1-nodus-language.md)) [L1]
  - ✅ `~PARALLEL` fail-fast error propagation documented (§4.4)
  - ✅ `RUN(@macro_name)` macro invocation syntax documented (§4.3)
  - ✅ Document History added (v1.0.1)
- [x] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - ✅ §4.6 verified: 50 commands match `vocab.rs::KNOWN_COMMANDS`; `BUILTIN_SCHEMA_VERSION` = "0.4.5"
  - ✅ `RUN` meta-command vocabulary gap documented
  - ✅ All Canonical References resolve; Document History added (v1.0.1)

## Phase 2 — Library Hardening ✓

*Build confidence required for safe extraction: golden test corpus, NL-invariant coverage map, public API stability baseline.*

- [x] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - ✅ Normative fixture corpus: `conditional.nodus`, `for_loop.nodus`, `parallel_join.nodus`, `macro_expand.nodus`
  - ✅ E013 (NL-8): validator rejects runtime-owned variable as pipeline target; `RUNTIME_OWNED_VARIABLES` constant added
  - ✅ E014 (NL-10): validator rejects forward references; per-step ordered tracking implemented
  - ✅ `RUN` added to `KNOWN_COMMANDS`; `BUILTIN_SCHEMA_VERSION` bumped to `"0.4.6"`
  - ✅ 142 tests pass (91 unit + 17 invariant + 34 parity); clippy clean
  - ✅ Extraction audit: zero external deps, zero intra-workspace imports; `Cargo.toml` workspace fields documented

## Phase 3 — Standalone Extraction ✓

*Prepare `crates/nodus` for publication as an independent library: sync the spec with Phase 2 implementation, harden the Cargo manifest for crates.io, document the public API, and produce the extraction artifacts (CI workflow, extraction procedure).*

- [x] **Nodus Runtime (Rust)** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2]
  - ✅ l2-nodus-runtime.md synced to v1.0.2: BUILTIN_SCHEMA_VERSION v0.4.6, 51 commands, E013/E014, RUNTIME_OWNED_VARIABLES
  - ✅ `Cargo.toml` workspace-delegated fields replaced; crates.io metadata added (description, keywords, categories, readme, homepage, documentation, docs.rs config)
  - ✅ `lib.rs` `//!` doc rewritten standalone; broken intra-doc links and SDD reference leaks removed; 0 `cargo doc` warnings
  - ✅ `crates/nodus/.github/workflows/ci.yml` written (check + test + clippy + fmt + doc)
  - ✅ `crates/nodus/EXTRACTION.md` written (7-step human extraction procedure)
  - ✅ 143 tests pass (91 unit + 17 invariant + 34 parity + 1 doctest); clippy clean

## Phase 4 — Observability & Extension Framework ✓

*Implement the full AuditProvider event taxonomy from `l1-nodus-observability.md` and complete the extension interface framework from `l1-nodus-portability.md`. Raises nodus from "extraction-ready" to "production-observable".*

- [x] **L2 Nodus Observability** ([l2-nodus-observability.md](specifications/l2-nodus-observability.md)) [L2] — full AuditProvider implementation: AuditProvider trait + ExecutionEvent 10-variant enum + NoopAuditProvider + RunManifest + FieldDescriptor; executor hook-point map (all 10 events); run_with_audit + run_with_provider_and_audit public API; 13-test plan; `Implements: l1-nodus-observability.md`
  - ✅ `observability.rs` module created; AuditProvider trait + 10-variant ExecutionEvent + NoopAuditProvider
  - ✅ `executor.rs` wired: StepStart/StepEnd/StepError/ConstraintHit/BranchTaken/LoopIteration/MacroEnter/MacroExit/ModelCall/ModelResponse
  - ✅ `run_with_audit` + `run_with_provider_and_audit` added to `workflows.rs` + re-exported from `lib.rs`
  - ✅ `tests/observability.rs`: observer_neutrality, run_with_audit_api, run_with_provider_and_audit_api
  - ✅ All tests pass; clippy clean; docs zero-warning

- [x] **Nodus Runtime delta** ([l2-nodus-runtime.md](specifications/l2-nodus-runtime.md)) [L2] — §4.1 updated (observability.rs added), §4.5 updated (run_with_audit/run_with_provider_and_audit), version bumped 1.0.2 → 1.0.3

## Phase 5 — Portability Implementation

*Implement the `SchemaProvider` vocabulary-extension seam and define the pending `StorageProvider`/`PolicyProvider` trait interfaces in `crates/nodus`. Raises nodus from "observability-capable" to "fully-portable" per the LP-invariants.*

> **Status:** Complete — all tracks A/B/C/T delivered. Atomic tasks in `archives/tasks/phase-5.md`.

- [x] **L2 Nodus Portability** ([l2-nodus-portability.md](specifications/l2-nodus-portability.md)) [L2]
  - ✅ `portability.rs` module: SchemaProvider + BuiltinSchemaProvider, StorageProvider + NoopStorageProvider, PolicyProvider + NoopPolicyProvider
  - ✅ `vocab.rs` delta: `Schema::with_provider()` constructor, `is_host_command()` helper, `host_commands`/`host_reserved` fields
  - ✅ `lexer.rs` delta: schema-aware lexing (`new_with_schema`, `tokenize_str_with_schema`, `extra_commands` field)
  - ✅ `parser.rs` delta: `parse_with_schema()` using extended lexer
  - ✅ `workflows.rs` delta: `run_with_schema` + `run_with_schema_and_audit` public functions
  - ✅ `lib.rs` re-exports for all portability types and new workflow functions
  - ✅ `l2-nodus-runtime.md` spec sync v1.0.3 → v1.0.4
  - ✅ 166 tests pass (107 unit + 17 invariant + 4 observability + 34 parity + 3 portability + 1 doctest); clippy clean; fmt clean; docs zero warnings

## Phase 6 — Testing Implementation ✓

*Implement the full `@test:` block contract from `l1-nodus-testing.md`. Raises nodus from a
stub-level runner to an assertion-evaluating test facility (NT-1…NT-10).*

> **Status:** Complete — all tracks A/B/C/T delivered. Atomic tasks in `archives/tasks/phase-6.md`.

- [x] **L2 Nodus Testing** ([l2-nodus-testing.md](specifications/l2-nodus-testing.md)) [L2]
  - ✅ `ast.rs`: structured `TestBlock` (input/expected/tags/raw_lines typed fields)
  - ✅ `parser.rs`: `parse_test_block()` populates structured fields; E015 on duplicate test name
  - ✅ `transpiler.rs`: round-trip fidelity for structured test blocks (to_nodus + to_human)
  - ✅ `workflows.rs`: `evaluate_test_block()` assertion evaluator; `test()` rewritten for per-block NT-1/NT-2/NT-3/NT-4/NT-5; `test_with_tags()` for NT-6
  - ✅ `validator.rs`: W001 (route uncovered), W002 (no expected:) diagnostics; E015 not double-reported
  - ✅ `tests/testing.rs`: 7 integration tests covering NT-1…NT-7
  - ✅ `l2-nodus-testing.md` v1.0.0 authored; NT-1…NT-10 compliance table; registered in INDEX.md
  - ✅ Quality gates: 204 tests pass (target 175+ met); clippy clean; fmt clean; docs zero new warnings

## Phase 7 — Capability Manifest (LP-8) ✓

*Implement the LP-8 capability manifest + pre-run satisfiability validation (fail-fast) from `l1-nodus-portability.md` §4.6 in `crates/nodus`. A workflow declares the extension-point roles / host commands / named capabilities it needs; the runtime validates that declaration against the active host before the first step runs and rejects fail-fast with the missing-capability set, never starting a partially-capable run. The same manifest is the machine-checkable two-host portability contract (LP-3). Completing this phase restabilizes `l2-nodus-portability` (RFC → Stable, C12.1 Stabilization Exception). Atomic tasks in `tasks/phase-7.md`.*

> **Status:** Complete — all tracks A/B/C/T/D delivered.

- [x] **L2 Nodus Portability** ([l2-nodus-portability.md](specifications/l2-nodus-portability.md)) [L2]
  - ✅ `portability.rs`: `ExtensionRole` enum, `CapabilityManifest` (roles/commands/capabilities), `HostCapabilities` (provides/has_command/satisfies + `builtin()`), `Missing` enum, pure `validate_manifest()` resolver
  - ✅ `CapabilityManifest::from_workflow` derives required roles by walking the AST (model command → Model; non-builtin command → Vocabulary)
  - ✅ `workflows.rs`: `run_with_manifest` + `run_with_manifest_and_audit` — fail-fast gate after lint validation, before executor boot; rejected runs emit no audit events
  - ✅ `vocab.rs`: `NODUS:CAPABILITY_UNMET` diagnostic; `lib.rs` re-exports
  - ✅ `tests/portability.rs`: LP-3 two-host substitution + pre-run purity (observer-neutrality) + gate rejection/acceptance
  - ✅ `l2-nodus-portability.md` §4.7 authored, §3 LP-8 row → Implemented; v1.0.0 → 1.1.0, RFC → Stable
  - ✅ 217 tests pass (was 204; +13); clippy clean; fmt clean; docs zero new warnings

## Phase 8 — Error Taxonomy (l2-nodus-errors) ✓

*Implement the 24-code error taxonomy from `l1-nodus-language.md` §4.6 in `crates/nodus`, per `l2-nodus-errors.md`. Adds `ErrorSeverity`/`ErrorCategory` metadata types, the per-code severity×category registry with an `error_meta()` lookup, and the supersede of the catch-all `NODUS:EXECUTION_FAILED`. This is the foundational cluster of the upstream-parity gap: the control-flow, dialog, and operator clusters all reference codes defined here. Codes whose features are not yet built (e.g. `SWITCH_NO_MATCH`, `DIALOG_*`, `KB_UNAVAILABLE`) are defined ahead and wired to emission when their cluster lands. Atomic tasks in `tasks/phase-8.md`.*

> **Status:** Complete — all tracks A/B/T delivered.

- [x] **L2 Nodus Errors** ([l2-nodus-errors.md](specifications/l2-nodus-errors.md)) [L2]
  - ✅ `vocab.rs`: `ErrorSeverity` (Error/Warn/Info) + `ErrorCategory` (Parse/Runtime/Validation/Routing/Memory/Test/Control/Dialog) enums
  - ✅ 14 new `error_code` constants (UNDEFINED_CMD … DIALOG_REJECTED); `error_meta()` static severity×category registry (24 canonical + CAPABILITY_UNMET)
  - ✅ `EXECUTION_FAILED` marked `#[deprecated]`, excluded from canonical registry (`error_meta` → `None`)
  - ✅ No live catch-all emission sites existed; validation-category codes defined-ahead pending the validator↔runtime bridge
  - ✅ Lockstep test (`error_registry_lockstep`) guards constant↔metadata sync; NL-1/2/4/9 preserved
  - ✅ 222 tests pass (was 217; +5); clippy clean; fmt clean; docs zero new warnings; SDD §6 clean

## Phase 9 — Closed Vocabulary Registries (l2-nodus-registries) ✓

*Implement the closed vocabulary registries from `l1-nodus-language.md` §4.6 (contents per `l2-nodus-runtime.md` §4.7(f)) in `crates/nodus`, per `l2-nodus-registries.md`. Adds `KNOWN_FLAGS`, `KNOWN_VALIDATORS`, and `PRIMITIVE_TYPES` as `vocab` data, the `Schema` query surface, and advisory (warning-severity) validator diagnostics for `~flag`/`^validator`/`@in` type tokens outside the registries. Strengthens NL-1/NL-7/NL-9; advisory-first so no existing workflow hard-breaks. Atomic tasks in `tasks/phase-9.md`.*

> **Status:** Complete — all tracks A/B/T delivered.

- [x] **L2 Nodus Registries** ([l2-nodus-registries.md](specifications/l2-nodus-registries.md)) [L2]
  - ✅ `vocab.rs`: `KNOWN_FLAGS` (12) + `KNOWN_VALIDATORS` (12, pre-colon name match) + `PRIMITIVE_TYPES` (10) constants
  - ✅ `Schema::is_known_flag` / `is_known_validator` / `is_known_type` query surface
  - ✅ `validator.rs`: advisory W011 (unknown `~flag`) / W012 (unknown `^validator`) / W013 (unknown `@in` type); warnings never set `has_errors`
  - ✅ 228 tests pass (was 222; +6); clippy clean; fmt clean; docs zero new warnings; SDD §6 clean; no fixture regressions

## Phase 10 — Human-in-the-Loop Dialog (l2-nodus-dialog) ✓

*Implement the dialog contract (`l1-nodus-dialog.md`, now Stable) in `crates/nodus`, per `l2-nodus-dialog.md`. Adds the `ASK`/`CONFIRM` commands, the `Status::Paused` run state + `ResumeDescriptor`, the `DialogProvider` extension point with a built-in synchronous `DefaultDialogProvider` (default-or-pause, no I/O), the `ExtensionRole::Dialog` manifest binding, executor dispatch, and the `run_with_dialog` combinators. The built-in resolver keeps non-interactive runs deterministic; true cross-invocation suspend/resume is a host concern over the `Status::Paused` signal. This is the largest remaining cluster — it touches vocab, executor, portability, and workflows. Atomic tasks in `tasks/phase-10.md`.*

> **Status:** Complete — all tracks A–E/T delivered.

- [x] **L2 Nodus Dialog** ([l2-nodus-dialog.md](specifications/l2-nodus-dialog.md)) [L2]
  - ✅ `vocab.rs`: `ASK`/`CONFIRM` in `KNOWN_COMMANDS`; `executor.rs`: `Status::Paused` + `Signal::Pause`
  - ✅ `DialogOutcome` (Answer/Pause/Timeout/Rejected) + `DialogProvider` trait + synchronous `DefaultDialogProvider` (default-or-pause)
  - ✅ Executor dispatch via `handle_dialog`; `ResumeDescriptor` on `RunResult` (workflow + var snapshot + step index); emits `DIALOG_TIMEOUT`/`DIALOG_REJECTED`/`PAUSED`; `FieldDescriptor`-only events (DG-7)
  - ✅ `ExtensionRole::Dialog` + `from_workflow` derivation (required only when a dialog lacks `+default`); `HostCapabilities::builtin()` omits Dialog
  - ✅ `run_with_dialog`/`run_with_dialog_and_audit` (workflows.rs) + lib re-exports
  - ✅ `tests/dialog.rs` (7 DG-invariant integration tests) + unit tests; 237 tests pass (was 228; +9); clippy clean; fmt clean; docs zero new warnings; SDD §6 clean

## Phase 11 — Control-Flow Constructs (l2-nodus-control-flow)

*Implement the v0.7 control constructs from `l1-nodus-language.md` §4.6 in `crates/nodus`, per `l2-nodus-control-flow.md`: `?SWITCH` multi-branch dispatch, `~MAP` collection transform, `~RETRY:n` bounded step retry, `!HALT` fatal stop, `!PAUSE` suspension. Implemented as vertical slices (each = lexer + AST + parser + executor + transpiler + validator + tests); reuses the existing `Status::Failed`/`Status::Paused`/`Signal::Pause` and `SWITCH_NO_MATCH`/`PAUSED` codes.*

> **Status:** Done (2026-06-27) — all four slices delivered across `crates/nodus`: `!HALT`/`!PAUSE` action flags (Signal::Halt, E016 halt-requires-escalate), `?SWITCH` first-match dispatch (+`*` default, SWITCH_NO_MATCH, W014 empty-arms), `~MAP` transform (implicit `$it`), `~RETRY:n` bounded retry (E017 enforcing 1≤n≤10, NL-5). 265 tests pass; clippy/fmt clean. l2-nodus-control-flow Stable at 1.0.0. See [archives/tasks/phase-11.md](archives/tasks/phase-11.md).

- [x] **L2 Nodus Control-Flow** ([l2-nodus-control-flow.md](specifications/l2-nodus-control-flow.md)) [L2] — Slice 1: `!HALT`/`!PAUSE` action flags (reuse Signal/Status). Slice 2: `?SWITCH` (+`*` default, `SWITCH_NO_MATCH`). Slice 3: `~MAP` (implicit `$it`). Slice 4: `~RETRY:n` (bounded, NL-5). Each slice carries lexer/AST/parser/executor/transpiler/validator + tests

## Phase 12 — Environment & Evaluation (l2-nodus-environment)

*Implement the Environment/Evaluation contract (`l1-nodus-environment`, Stable) in `crates/nodus`, per `l2-nodus-environment.md`. Adds the 5th extension role — a graded-run substrate the host's evaluation-driven improvement loop consumes. All-additive and pure in-tree: no new external dependency (the NE-12 content address is a std digest, not a crypto crate), no new `Status` (a `step` reuses the dialog suspend/resume shape), no new command, no new error category beyond reusing `NODUS:CAPABILITY_UNMET`. The trajectory rides existing audit events as an optional side-band descriptor — HO-6's closed taxonomy is preserved. Decomposed into tracks A→D + gates (Sequential); Track A first as everything references the trait. Atomic tasks in [tasks/phase-12.md](tasks/phase-12.md).*

- [x] **L2 Nodus Environment** ([l2-nodus-environment.md](specifications/l2-nodus-environment.md)) [L2] — Track A: `EnvironmentProvider` trait + `StubEnvironment` + deterministic lifecycle (NE-1/NE-2/NE-7). Track B: `Reward`/`EnvInteraction` trajectory side-band on existing events, no new `ExecutionEvent` variant (NE-3/NE-5). Track C: `ExtensionRole::Environment` + `builtin()` provides it via stub + `from_workflow` fail-fast (NE-10). Track D: `run_with_environment` frozen boundary + `EnvironmentProfile`/`Budget`/`GradingMode` hybrid-floor + `CandidateResult` digest (NE-4/6/11/12/13). Track T: NE-1…NE-13 integration suite + gates (zero-dep preserved)

## Phase 13 — Declarative Configuration Surface (l2-nodus-config)

*Implement the `§config` declarative-configuration surface (`l2-nodus-config`, Stable) in `crates/nodus`, per `l1-nodus-language.md` §4.1 (NL-20). Replaces the `§config` parser stub with a field-declaration AST + `parse_config`, a pure pre-run `check_config_values` shape check, a provenance-carrying + write-only-secret `AcceptedConfig` value model, a `ConfigProvider`/`ExtensionRole::Config` host-acceptance seam with a deterministic `DefaultConfigProvider`, and the `run_with_config[_and_audit]` entry points sequencing declaration → proposed → shape check → host acceptance → run. All-additive, zero new dependency (LP-1); reuses the existing capability-manifest (LP-8) and error-taxonomy (`CONFIG_INVALID`) machinery. Decomposed into tracks A→C + validation (Sequential); Track A first as the AST is referenced everywhere. Atomic tasks in [tasks/phase-13.md](tasks/phase-13.md).*

- [ ] **L2 Nodus Config** ([l2-nodus-config.md](specifications/l2-nodus-config.md)) [L2] — Track A: `ConfigDecl`/`ConfigField`/`FieldConstraint` AST + `parse_config` (rewire the `§config` deferral) + transpiler round-trip. Track B: `CONFIG_INVALID` code + pure `check_config_values` shape check + `AcceptedConfig` provenance/secret value model. Track C: `ConfigProvider` + `ExtensionRole::Config` + `builtin()` + `run_with_config[_and_audit]` + lib re-exports. Track T: NL-20 shape-check + secret-neutrality + LP-8 fail-fast + zero-dep validation suite

## Backlog

<!-- Pending L2 Invariant-Compliance obligations (additive invariants on already-Done L1 concept specs, INDEX v1.0.57) — awaiting L2 realization specs before they can be decomposed into verifiable atomic tasks:
  - NL-19 (tokenization-boundary seam discipline) + NL-21 (confidentiality-flow label side-band + per-sink gating): largely host-supplied, small nodus-core contribution — absorb into l2-nodus-runtime Invariant-Compliance in a focused pass.
  - NL-20 (§config declarative-configuration field-declaration grammar): RESOLVED — l2-nodus-config authored (Stable) and phased as Phase 13.
  - LP-17/LP-18/LP-19 (settlement effect-sink, exposure-switch seam): l2-nodus-portability Invariant-Compliance obligations.
  - HO-14…HO-20 (measurement availability, cross-run step identity, exposure-switch manifest recording, fault identity, RunManifest-as-reproduction-recipe): l2-nodus-observability RunManifest/side-band obligations. -->
<!-- Upstream parity gap v0.4.6 → v0.7 (l1-nodus-language §4.6 / l2-nodus-runtime §4.7) — remaining clusters needing focused spec authoring before they can be planned: operators/expressions (MATCHES/?./??/WHERE/FIRST/LAST/string-interpolation — note MATCHES/PCRE vs the zero-dependency LP-1 constraint is an open design fork), @needs selective schema loading (blocked: no external-schema loading yet), @ON(priority=N) (triggers not dispatched yet), macro execution (RUN(@x) body expansion — needs structured macro-body parsing). Addressed: error taxonomy → Phase 8; closed registries → Phase 9; HITL dialog → Phase 10 (l2-nodus-dialog, Stable); control constructs → Phase 11 (l2-nodus-control-flow, Stable). -->
<!-- StorageProvider/PolicyProvider executor integration deferred pending LP-3 satisfied (interfaces present in portability.rs; hook points + run_with_storage/run_with_policy variants pending the second documented host context). -->
<!-- Future: l2-nodus-transpiler.md — dedicated transpiler L2 spec (currently covered by l2-nodus-runtime.md §4). -->

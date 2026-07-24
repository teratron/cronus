# Project State

<!-- STATE.md — live project memory. Read FIRST in every workflow session. -->
<!-- Maximum 100 lines. Agent updates AFTER each completed action. -->

**Workspace:** nodus
**Updated:** 2026-07-24 12:45
**Phase:** 14 — Run-Manifest Identity & Reproducibility
**Status:** Done

## Current Position

- **Task:** Phase 14 complete — Run-Manifest Identity & Reproducibility (l2-nodus-observability §4.7)
- **Spec:** PLAN v1.17.0 / TASKS v2.9.0 / INDEX v1.0.59; RULES v1.6.0. 16 nodus specs Stable, phases 1–14 Done. l2-nodus-observability §4.7 (HO-12/15/18/19/20) implemented. Remaining: HO event-stream batch (HO-7…HO-11/13/14/16/17) + NL-19/21 (l2-nodus-runtime) + LP-17/18/19 (l2-nodus-portability) — see PLAN Backlog
- **Next Action:** Phase complete; run /magic.task nodus to plan the next phase (Backlog: HO event-stream batch needs a dedicated /magic.spec pass first; NL-19/21 + LP-17/18/19 are lighter compliance-table closures)

## Progress

```
Build phases 1–14 Done (Seed → Testing → Capability Manifest → Dialog → Control-Flow → Environment & Evaluation → Declarative Configuration Surface → Run-Manifest Identity & Reproducibility) | Phase 14: 7/7 tasks, Sequential A→B→C→T, all archived | Gates green: cargo 343 tests (was 335; +8) + clippy + fmt + doc; LP-1 zero-dep preserved
```

## Recent Decisions

- 2026-07-24 **Decision:** Phase 14 complete. `l2-nodus-observability` §4.7 implemented in `crates/nodus`: `ExecutionMode`/`SimFidelity`/`Determinism`/`ReproRecipe`/`FaultIdentity` types; `step_identity`/`fault_identity` on events; `RunManifest.{execution_mode,exposure_switches,repro}`; `Executor::execute_with_manifest_context` host-declaring entry point. Three design decisions caught during implementation, each a `[DR]`: (1) `step_identity(u32, &str)` not `step_identity(&Step)` — emission call sites carry the two definition pieces directly, not an AST reference; (2) `workflow_digest` hashes the parsed AST's `Debug` form, not raw source — `execute_inner` never sees source text (parsing happens in `workflows.rs`); (3) extended the private `execute_inner` (3 in-file callers) with the two host-declared params rather than touching `execute()`/`execute_with_params()`'s public signatures, adding one new public method for the declaring path instead. 343 tests pass (was 335; +8); clippy/fmt/doc clean (fixed 2 `derivable_impls` findings); zero new dependency (LP-1 preserved).

- 2026-07-24 **Decision:** Phase 14 opened — Run-Manifest Identity & Reproducibility. A spec-ahead-of-code L2 update (not a new orphan): l2-nodus-observability was already planned in Phase 4 (HO-1…HO-6), and its §4.7 (HO-12/15/18/19/20, v1.2.0) adds unrealized manifest-cluster content — recognized cognitively as a planning trigger (orphan-detection cannot surface it; only the SYNC_GAP warning fired). Decomposed §4.7 into 7 atomic tasks / 4 tracks (A manifest+event types/fields, B step_identity+fault_identity derivation, C repro population + host-supplied mode/switches, T validation), Sequential; each carries a concrete Verify line (C10). One [DR] deferred to implementation: the host-supplied `execution_mode`/`exposure_switches` entry-point shape (RunParams struct vs new run_with_* variant). All-additive/optional (HO-5/HO-6 preserved), zero-dep (LP-1). INDEX v1.0.58 → v1.0.59, PLAN v1.16.0 → v1.17.0, TASKS v2.8.0 → v2.9.0.

- 2026-07-24 **Decision:** Authored the observability manifest cluster. Discovered the observability backlog is bigger than flagged — the code realizes only HO-1…HO-6 + Phase-12 `env_trajectory`, so **HO-7…HO-20 are ALL pending** (14 invariants), too many for one phase. They split by data structure: HO-12/15/18/19/20 enrich `RunManifest`/`StepError` (one coherent "honest, cross-run-comparable, re-executable record" story); HO-7/8/9/11/13/16 + HO-14 + HO-10/17 enrich per-event descriptors (a separate story). Scoped this /magic.spec to the manifest cluster — updated l2-nodus-observability 1.1.0 → 1.2.0 with §4.7 (intended realization: `execution_mode`, `step_identity`, `exposure_switches`, `FaultIdentity`, `ReproRecipe`), Stable via C9 (spec-ahead-of-code, the l2-nodus-config precedent). Deferred the event-stream batch explicitly (nothing silently dropped). INDEX v1.0.58 → v1.0.59. Next: /magic.task opens the phase.

- 2026-07-24 **Decision:** Post-Phase-13 replan — no new phase (plan saturated). Pre-flight clean (16/16 Stable, no orphan, no SYNC_GAP, RULES parity v1.6.0 held). Phase 13 consumed the only orphan (l2-nodus-config/NL-20). Every remaining Backlog item (NL-19/21, LP-17/18/19, HO-14…HO-20) is an Invariant-Compliance obligation with no L2 realization spec, so none is decomposable into verifiable atomic tasks (C10). Computed next step (DA-6): /magic.spec nodus to author the L2 realizations — observability cluster HO-14…HO-20 (net-new RunManifest weight) is the natural next spec→task→run cycle; the language/portability clusters are lighter compliance-table closures. No PLAN/TASKS rewrite (already in lockstep with INDEX v1.0.58).

- 2026-07-24 **Decision:** Phase 13 complete. `l2-nodus-config` implemented in `crates/nodus`: `ConfigDecl`/`ConfigField`/`FieldConstraint` AST + `Parser::parse_config` (replaces the `§config` parser stub) + `Transpiler::config_to_nodus` round-trip; `NODUS:CONFIG_INVALID` code; pure `check_config_values` shape check + `AcceptedConfig` value model; `ConfigProvider`/`ConfigOutcome`/`DefaultConfigProvider` + `ExtensionRole::Config` (LP-8, `builtin()` provides it); `run_with_config`/`run_with_config_and_audit` public API. Two design decisions caught during implementation: (1) `Parser::parse`/`parse_with_schema` stay typed to `WorkflowFile` and give `§config` a precise redirect error naming `parse_config`, rather than the spec draft's literal "delegate" (which would require bolting an `Option<ConfigDecl>` onto the workflow AST) — `l2-nodus-config.md` patched 1.0.0→1.0.1 (no logic/status change) to match; (2) secret write-only is realized as an omission (never merged into `$in.config`) rather than a redaction filter, and full `Value`-level provenance tagging (NL-11) is explicitly scoped out as a pre-existing system-wide obligation, not something this feature could or should close alone. 335 tests pass (was 292; +43); clippy/fmt/doc clean; zero new dependency (LP-1 preserved).

- 2026-07-24 **Decision:** Phase 13 opened — Declarative Configuration Surface. `l2-nodus-config` (Stable) was an ORPHANED_SPEC (in INDEX, absent from PLAN); the No-Orphans guard pulled it into a new phase. Decomposed into 9 atomic tasks / 4 tracks (A AST+parser, B shape-check+error-code+value-model, C provider-seam+API, T validation), Sequential per the spec's §6 implementation order; each task carries a concrete Verify line (C10). Reuses LP-8 manifest + error taxonomy; all-additive, zero-dep (LP-1). Realizes NL-20 — clears the net-new-weight obligation from the v1.15.0 sync. INDEX v1.0.57 → v1.0.58, PLAN v1.15.0 → v1.16.0, TASKS v2.7.0 → v2.8.0.

- 2026-07-24 **Decision:** Sync-only re-plan (no new phase). Registry advanced INDEX v1.0.47 → v1.0.57 via additive refinement of three Done L1 concept specs (l1-nodus-language 1.12.0: NL-19/20/21; l1-nodus-portability 1.13.0: LP-17/18/19; l1-nodus-observability 1.12.0: HO-14…HO-20). All carried as pending L2 Invariant-Compliance obligations — none has a Stable L2 realization spec, so no verifiable atomic task can be authored (Verify-Line/Atomic-Task mandate). NL-20 (§config field-declaration grammar) is the net-new-weight item → needs a dedicated l2-nodus-config spec. RULES parity re-synced v1.5.0 → v1.6.0. PLAN v1.15.0 / TASKS v2.7.0.

- 2026-07-10 **Decision:** Phase 12 complete. `l2-nodus-environment` implemented in `crates/nodus`: new `environment.rs` (EnvironmentProvider trait + StubEnvironment + Reward/GradingMode/Budget/CandidateResult), `run_with_environment`/`run_with_environment_and_audit` public API, `ExtensionRole::Environment` (builtin() provides it via the stub, unlike Dialog), `EnvInteraction` trajectory side-band on `RunManifest` (no new `ExecutionEvent` variant — HO-6 preserved). Two design refinements caught during implementation: `evaluate`'s reward is delivered directly via `EnvRunResult.reward` rather than duplicated into the trajectory (it occurs after `run_complete` already fired); `grade()` takes an explicit `checker_passed: bool` rather than inferring pass/fail from the score (NE-9 metric neutrality). `max_tokens` on `Budget` is declared but not enforced (no token-accounting seam on `ModelProvider` — documented gap, StorageProvider/PolicyProvider precedent). 292 tests pass (was 265; +27); clippy/fmt/doc clean; zero new dependency (LP-1 preserved); downstream `cronus-cli` unaffected. `l1-nodus-environment`'s concept-only marker cleared.
- 2026-06-27 **Decision:** Phase 11 complete. Slice 4 `~RETRY:n` bounded step retry implemented in crates/nodus (lexer TildeRetry, Step.retry field, parser parse_retry_bound, executor run_step_with_retry with rollback-on-success/accumulate-on-exhaustion, validator E017 enforcing 1≤n≤10 per NL-5); 265 tests pass (+7). All four control-flow constructs (!HALT/!PAUSE, ?SWITCH, ~MAP, ~RETRY) now implemented.

- 2026-06-27 **Decision:** Phase 11 Slice 3 landed. `~MAP` collection transform implemented in crates/nodus (lexer TildeMap, MapBlock AST, parser parse_map + routing, executor execute_map binding $it + collecting into a list, transpiler human form); 258 tests pass (+5). Slice 4 (~RETRY) remains.

- 2026-06-27 **Decision:** Phase 11 Slice 2 landed. `?SWITCH` multi-branch dispatch implemented in crates/nodus (lexer QSwitch + Star tokens, SwitchBlock AST, parser parse_switch + routing, executor execute_switch first-match-wins + SWITCH_NO_MATCH, validator W014 empty-arms, transpiler human form); 253 tests pass (+8). Slices 3–4 (~MAP/~RETRY) remain.

- 2026-06-27 **Decision:** Phase 11 Slice 1 landed. `!HALT` / `!PAUSE` conditional action flags implemented in crates/nodus (lexer BangHalt/BangPause, Conditional.halt_flag/pause_flag, parser BranchFlags, executor Signal::Halt + branch_exit_signal, validator E016 halt-requires-escalate, transpiler human form); 245 tests pass (+8). Slices 2–4 (?SWITCH/~MAP/~RETRY) remain.

- 2026-06-27 **Decision:** Phase 10 complete. HITL dialog implemented in crates/nodus (ASK/CONFIRM, Status::Paused+ResumeDescriptor, DialogProvider+DefaultDialogProvider, ExtensionRole::Dialog, run_with_dialog); 237 tests pass (+9).

- 2026-06-27 **Decision:** Phase 9 complete. Closed vocabulary registries implemented in crates/nodus (KNOWN_FLAGS/KNOWN_VALIDATORS/PRIMITIVE_TYPES + Schema queries + advisory W011/W012/W013); 228 tests pass (+6).

- 2026-06-27 **Decision:** Phase 8 complete. 24-code error taxonomy implemented in crates/nodus/vocab.rs (ErrorSeverity/ErrorCategory enums, 14 new error_code constants, error_meta() registry, EXECUTION_FAILED deprecated, lockstep test); 222 tests pass (+5).

- 2026-06-27 **Decision:** Phase 7 complete. LP-8 capability manifest implemented in crates/nodus (CapabilityManifest/ExtensionRole/HostCapabilities/Missing + validate_manifest resolver + run_with_manifest gate + NODUS:CAPABILITY_UNMET + from_workflow); l2-nodus-portability v1.1.0 Stable; 217 tests pass (+13).

- 2026-06-24 **Decision:** Phase 6 complete. Delivers: `ast.rs` TestBlock structured fields (input/expected/tags), `parser.rs` E015 + input/expected parsing, `workflows.rs` evaluate_test_block() + test()/test_with_tags() NT-1…NT-10, `validator.rs` W001/W002, `tests/testing.rs` (7 integration tests), l2-nodus-testing.md v1.0.0 (Stable). 204 tests pass. Phase 6 archived.

- 2026-06-24 **Decision:** Phase 4 complete (reconciled). Delivers: `observability.rs` (AuditProvider trait, 10-variant ExecutionEvent enum, NoopAuditProvider, RunManifest, FieldDescriptor), executor.rs hook points for all 10 event types, `run_with_audit` + `run_with_provider_and_audit` public API, `tests/observability.rs` (observer_neutrality + API integration tests), l2-nodus-runtime.md v1.0.3 (spec sync). Phase 4 archived.

## Blockers

## Blocking Constraints

## Session Continuity

**Last Session Ended:** 2026-06-24
**Handoff File:** none
**Bootstrap Mode:** false

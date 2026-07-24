# Project State

<!-- STATE.md — live project memory. Read FIRST in every workflow session. -->
<!-- Maximum 100 lines. Agent updates AFTER each completed action. -->

**Workspace:** nodus
**Updated:** 2026-07-24
**Phase:** 13 — Declarative Configuration Surface
**Status:** Active

## Current Position

- **Task:** Phase 13 planned — Declarative Configuration Surface (l2-nodus-config); 9 tasks / 4 tracks A→C+T, Sequential; not started
- **Spec:** PLAN v1.16.0 / TASKS v2.8.0 / INDEX v1.0.58; RULES v1.6.0. 16 nodus specs Stable, phases 1–12 Done (Archived), Phase 13 Todo. l2-nodus-config authored (Stable) + phased — realizes NL-20 (§config). Remaining pending obligations: NL-19/21, LP-17/18/19, HO-14…HO-20 (see PLAN Backlog)
- **Next Action:** Run /magic.run nodus to execute Phase 13 — Track A first (Config AST `ConfigDecl`/`ConfigField` + `parse_config` replacing the §config parser stub)

## Progress

```
Build phases 1–12 Done (Seed → Testing → Capability Manifest → Dialog → Control-Flow → Environment & Evaluation); Phase 13 planned (Declarative Configuration Surface / l2-nodus-config — 9 tasks A→C+T, Sequential, not started) | Baseline gates: cargo 292 tests + clippy + fmt + doc; LP-1 zero-dep preserved
```

## Recent Decisions

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

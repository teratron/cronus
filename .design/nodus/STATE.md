# Project State

<!-- STATE.md — live project memory. Read FIRST in every workflow session. -->
<!-- Maximum 100 lines. Agent updates AFTER each completed action. -->

**Workspace:** nodus
**Updated:** 2026-07-02 06:01
**Phase:** 11 — Control-Flow Constructs (complete)
**Status:** Active

## Current Position

- **Task:** Phase 11 complete — all control-flow constructs (`!HALT`/`!PAUSE`, `?SWITCH`, `~MAP`, `~RETRY:n`) implemented in crates/nodus
- **Spec:** l2-nodus-control-flow.md v1.0.0 (Stable); all 13 specs Stable
- **Next Action:** Run /magic.task nodus to update the plan

## Progress

```
Overall: [6/6] ██████████ 100%
```

## Recent Decisions

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

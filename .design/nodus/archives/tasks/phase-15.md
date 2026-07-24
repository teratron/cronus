---
phase: 15
name: "Aggregation-Safe Event Stream"
status: Done
subsystem: "crates/nodus"
requires: [4, 14]
provides:
  - "Measurement { Taken(u64), Unavailable } — retyped on StepEnd/MacroExit/ModelResponse/RunManifest.elapsed_ms + LoopIteration.iteration_number"
  - "seq: u64 + correlation_id: String on all 10 ExecutionEvent variants; correlation_id == RunManifest.run_id (HO-7 identity)"
  - "Executor::emit — single seq-assigning emission choke point; resolve_correlation_id zero-dep fallback for empty run_id"
  - "RunManifest.event_count doubles as the HO-7 gap check (== highest seq + 1)"
key_files:
  created: []
  modified:
    - "crates/nodus/src/observability.rs"
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/src/lib.rs"
    - "crates/nodus/tests/observability.rs"
patterns_established:
  - "Fold type-surface + all-site-update tasks into one edit pass when Rust cannot compile a partial field change — report task boundaries separately in tracking even when physically folded (the Phase-14 precedent, reused here)"
  - "A single emission choke point (Executor::emit) makes a counter/field mismatch unrepresentable rather than merely absent; pin the discipline with a structural source-grep test, not just code review"
  - "Zero-dep id fallback: a process-local atomic counter over SystemTime+hash when a generated identifier must introduce no wall-clock reading into run metadata at all"
duration_minutes: ~
---

# Stage 15 Tasks — Aggregation-Safe Event Stream

**Phase:** 15
**Status:** Done
**Strategic Goal:** Realize `l2-nodus-observability.md` §4.8 (HO-7 + HO-14) in `crates/nodus` — make the event stream statistically trustworthy: a run-monotonic dense `seq` plus a run-scoped `correlation_id` on every event (order it, detect drops), and a two-state `Measurement` replacing every numeric that can fail to be obtained (aggregate it without a fabricated zero). All-additive to the taxonomy — no new `ExecutionEvent` variant (HO-6 preserved), no behavioural change to `RunResult` (HO-5 preserved); zero new dependency (LP-1). Sequential tracks A→B→C→T.

> **Scope note — nothing to implement for streaming merge.** §4.8 records HO-7's chunk-merge as *vacuous in core*: `ModelProvider::generate` returns a complete `String`, so no chunk ever exists at this layer and the fold is a host obligation (LP-2). Do not add merge machinery.

> **Churn warning.** Both Track-A tasks rewrite all 10 `ExecutionEvent` variants and every construction site (~20 emission sites in `executor.rs`, plus every test literal in `observability.rs` and `tests/observability.rs`). Rust will not compile a partially-applied field change, so each Track-A task lands its type change *together with* the sites it breaks — that is why their `Verify` is a whole-lib test run, not a per-file check.

## Atomic Checklist

- [x] [T-15A01] `Measurement` type + retype every obtainable-failure numeric (HO-14)
- [x] [T-15A02] `seq` + `correlation_id` on all 10 event variants + run-scoped binding (HO-7)
- [x] [T-15B01] Single `seq`-assigning emission choke point
- [x] [T-15B02] Manifest gap-check identity — `event_count` == highest `seq` + 1
- [x] [T-15C01] Fix the fabricated zero — `handle_dialog`'s `elapsed_ms: 0` → `Unavailable`
- [x] [T-15T01] Validation suite — dense seq, correlation identity, measurement honesty, neutrality, zero-dep

## Detailed Tracking

### [T-15A01] `Measurement` type + numeric retype (HO-14)

- **Spec:** l2-nodus-observability.md §4.8 (HO-14)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib observability::` → passed (part of 230 lib tests); `measurement_unavailable_is_not_taken_zero` + `measurement_round_trips_through_recording_provider` added; all 5 sites retyped, `FieldDescriptor` left untouched as specified.
- **Handoff:** the type T-15C01 spends and T-15A02 coexists with.
- **Notes:** Added `pub enum Measurement { Taken(u64), Unavailable }` (derives `Debug, Clone, Copy, PartialEq, Eq`; no `Default` — forces every site to explicitly choose) to `observability.rs`, re-exported from `lib.rs`. Retyped exactly the five sites: `StepEnd.elapsed_ms`, `MacroExit.elapsed_ms`, `ModelResponse.elapsed_ms`, `RunManifest.elapsed_ms`, `LoopIteration.iteration_number` (widened `u32`→`u64` inside `Taken`). `FieldDescriptor.field_count`/`total_bytes` left as plain `u32` per spec. Folded with T-15A02/B01/C01 in one edit pass — Rust does not compile a partially-applied field change, so all ~20 executor.rs sites + all observability.rs test literals were updated together; reported separately here per the task decomposition.

### [T-15A02] `seq` + `correlation_id` on every variant (HO-7)

- **Spec:** l2-nodus-observability.md §4.8 (HO-7)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability correlation_id_is_one_per_run_and_equals_manifest_run_id` → passed; all 10 variants carry `seq`/`correlation_id`.
- **Handoff:** the fields T-15B01 assigns and T-15B02 checks.
- **Notes:** Added `seq: u64` + `correlation_id: String` to all 10 `ExecutionEvent` variants. `ExecutionContext` gained a `correlation_id: String` field, bound once in `ExecutionContext::new(correlation_id)`, called from `execute_inner` as `ExecutionContext::new(resolve_correlation_id(run_id))`. **[DR-a] resolved:** chose the process-local atomic-counter fallback (`"run-{n}"`, zero-dep, no wall-clock reading at all) over `SystemTime`+hash — simpler and introduces no time-based input into run metadata whatsoever, not even for a generated id. `RunManifest.run_id` now reads `ctx.correlation_id.clone()` (not the raw `run_id` param) so the HO-7 identity (`correlation_id == RunManifest.run_id`) holds even on the `execute()`/`execute_with_params("", "")` path — verified no existing test asserted an empty `run_id` before making this change. **[DR-b] resolved:** kept `String` (not `Arc<str>`) — confirmed via code read that events already allocate `String`s eagerly elsewhere (`step_command` etc.) before reaching even `NoopAuditProvider`, so the correlation clone is consistent with the existing design, not a new regression; left as a documented future option if it ever proves material.

### [T-15B01] Single `seq`-assigning emission choke point

- **Spec:** l2-nodus-observability.md §4.8 (HO-7)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `grep -c "self.audit.record_event(" crates/nodus/src/executor.rs` → **1** (only inside `emit` itself; was 20 scattered call sites). `cargo test -p nodus --test observability seq_is_dense_and_gap_free_across_a_multi_event_run` → passed (12-event run: dense `0..11`, no duplicates, no holes). A structural regression guard `emit_choke_point_is_the_only_record_event_call_site` greps the source at test time so a future edit bypassing `emit` fails CI, not just review.
- **Handoff:** makes T-15B02's identity true by construction.
- **Notes:** Added `Executor::emit(&self, ctx: &mut ExecutionContext, build: impl FnOnce(u64, String) -> ExecutionEvent)` — reads `ctx.event_count` into the closure as `seq`, dispatches to `self.audit.record_event`, then increments the counter. Converted all 20 former `self.audit.record_event(...); ctx.event_count += 1;` pairs across `execute_conditional`, `execute_for`, `execute_until`, `execute_switch`, `execute_command`, `handle_dialog`, the `RUN` dispatch arm, `handle_gen`, `handle_analyze` to `self.emit(ctx, |seq, correlation_id| ExecutionEvent::Variant { ..., seq, correlation_id })`.

### [T-15B02] Manifest gap-check identity

- **Spec:** l2-nodus-observability.md §4.8 (HO-7)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability manifest_event_count_is_highest_seq_plus_one` → passed.
- **Handoff:** completes HO-7; feeds T-15T01.
- **Notes:** No new field — `RunManifest.event_count`'s doc comment now states it doubles as the HO-7 gap check (both derive from the same counter through `emit`). The identity holds by construction after T-15B01; this task's contribution is the pinning test plus the doc comment, so a future refactor that breaks the identity fails a test, not just an inspection.

### [T-15C01] Fix the fabricated zero (HO-14)

- **Spec:** l2-nodus-observability.md §4.8 (HO-14)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability dialog_step_elapsed_is_unavailable_timed_step_is_taken` → passed — dialog `StepEnd.elapsed_ms` is `Measurement::Unavailable`; an ordinary GEN `StepEnd.elapsed_ms` is `Measurement::Taken(_)`, distinguishable.
- **Handoff:** completes HO-14's concrete bite; feeds T-15T01.
- **Notes:** `handle_dialog`'s `StepEnd { elapsed_ms: 0 }` (the crate's one hardcoded zero, confirmed by grep before this phase started) is now `Measurement::Unavailable`. Did not add an `Instant` timer to `handle_dialog` — the dialog path legitimately has no meaningful own-duration (may suspend awaiting a human), and `Unavailable` is the honest value, not a placeholder to be filled in later.

### [T-15T01] Validation Task — aggregation-safety contract suite

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-observability.md` §4.8 (HO-7 + HO-14), and confirm HO-5 observer neutrality and LP-1 zero-dep still hold.
- **Method:** Extended `tests/observability.rs` with a T-15T01 section (7 new tests): `seq_is_dense_and_gap_free_across_a_multi_event_run`, `correlation_id_is_one_per_run_and_equals_manifest_run_id`, `manifest_event_count_is_highest_seq_plus_one`, `correlation_id_generated_when_run_id_empty`, `dialog_step_elapsed_is_unavailable_timed_step_is_taken`, `measurement_unavailable_is_never_equal_to_taken_zero`, `emit_choke_point_is_the_only_record_event_call_site` (structural source-grep guard). The multi-event fixture uses a `~FOR` loop (2 iterations) + `GEN` + `LOG` for event-type variety; caught and fixed a real bug in the fixture itself during authoring — `LOG` unconditionally locks `$out` regardless of target, so a loop-body `LOG` before a later `GEN(...) → $out` produced a spurious `RULE_VIOLATION`; reordered `GEN` first.
- **Status:** Done
- **Verify:** `cargo test -p nodus --test observability` → 19 passed (was 12; +7). Full-crate `cargo test -p nodus` → **352 passed** (was 345; +7), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean. `cargo fmt -p nodus -- --check` → clean (after one `cargo fmt` pass, no logic change). No `.unwrap()`/`panic!()`/`unreachable!()` introduced in production code (checked directly). `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; LP-1 zero-dep preserved.

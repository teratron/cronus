---
phase: 16
name: "Event Annotations, Cost, Lineage & Completeness"
status: Done
subsystem: "crates/nodus"
requires: [14, 15]
provides: ["EventAnnotations", "Anomaly", "Durability", "SourceRef", "TraceCompleteness", "classify_trace", "LoopType::Map"]
key_files:
  created: []
  modified:
    - crates/nodus/src/observability.rs
    - crates/nodus/src/executor.rs
    - crates/nodus/src/lib.rs
    - crates/nodus/tests/observability.rs
patterns_established:
  - "Single-carrier annotation field (EventAnnotations) absorbing multiple optional per-event concerns, instead of one field per concern, to avoid all-variant churn multiplying with every future annotation."
  - "Read-side pure classifiers (classify_trace) over the durable event stream, reusing an existing gap identity rather than adding a field or a second mechanism."
duration_minutes: ~
---

# Stage 16 Tasks — Event Annotations, Cost, Lineage & Completeness

**Phase:** 16
**Status:** Todo
**Strategic Goal:** Realize `l2-nodus-observability.md` §4.9 (HO-8, HO-9, HO-10, HO-11, HO-13, HO-16, HO-17) in `crates/nodus` — **closing all twenty HO invariants**. One `EventAnnotations` carrier field on every variant absorbs the four per-event annotations (receipt, message, anomaly, durability); the rest are targeted: token classes on `ModelResponse`, derivation lineage on collection events, and a read-side trace classifier with no field at all. Every field optional or defaulted, so a host declaring none emits a stream identical to §4.8's (HO-5/HO-6 preserved); zero new dependency (LP-1). Sequential tracks A→C + validation.

> **Churn note — one field, not four.** §4.9 deliberately routes HO-9/HO-11/HO-16/HO-17 through a single `annotations: EventAnnotations` field rather than four separate optional fields. Each of the 10 `ExecutionEvent` variants therefore gains exactly **one** field, and each construction site adds one `EventAnnotations::default()`. Do not "simplify" this into four flat fields — that quadruples the all-variant churn now and again for every future annotation.

> **Scope note — nothing to build for streaming.** HO-17's durability marker is reserved, not exercised: `ModelProvider::generate` returns a complete `String`, so nodus emits no transient events. Do **not** build an `emit_transient` companion with no caller (dead code). T-16C02 realizes HO-17 as a documented rule on `emit` plus a test pinning that core emits only `Durable` — the companion path lands when a host-facing streaming path does.

## Atomic Checklist

- [x] [T-16A01] `EventAnnotations` carrier — types + one field on all 10 variants (HO-9/11/16/17)
- [x] [T-16B01] HO-8 token classes on `ModelResponse` — 4 `Measurement` fields
- [x] [T-16B02] HO-13 derivation lineage + the missing `~MAP` emission
- [x] [T-16C01] HO-10 `classify_trace` — read-side classifier, no field
- [x] [T-16C02] HO-17 durable-only discipline — rule on `emit` + core-emits-`Durable` guard
- [x] [T-16T01] Validation suite — annotations, cost, lineage, completeness, neutrality, zero-dep

## Detailed Tracking

### [T-16A01] `EventAnnotations` carrier (HO-9, HO-11, HO-16, HO-17)

- **Spec:** l2-nodus-observability.md §4.9 (carrier)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib observability::` → passed (part of 238 lib tests); `event_annotations_default_is_all_none_and_durable` + `anomaly_unscored_is_never_normal` added.
- **Handoff:** the carrier every later task annotates; landed first.
- **Notes:** Added to `observability.rs`: `EventAnnotations { message, anomaly, receipt, durability }` (derives `Default`), `Anomaly { Anomalous, Normal, Unscored }`, `Durability { Durable, Transient }` (`#[derive(Default)]` + `#[default] Durable` — a manual `impl Default` was rejected by clippy, the Phase-14 lesson applied proactively). Added `annotations: EventAnnotations` to all 10 `ExecutionEvent` variants. Folded with T-16B01/B02/C02 in one edit pass — Rust does not compile a partially-applied field change, so all 20+1 (the new `~MAP` site) executor.rs emission sites and every observability.rs test literal were updated together; reported separately per the task decomposition. `receipt`/`message`/`anomaly` stay `None` everywhere (no host provider exists yet) — the carrier is reserved, not populated, exactly as spec'd.

### [T-16B01] HO-8 cost-attribution token classes

- **Spec:** l2-nodus-observability.md §4.9 (HO-8)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability model_response_token_classes_are_unavailable_not_zero` → passed — all four fields `Unavailable`, each asserted `!= Taken(0)`.
- **Handoff:** fed T-16T01.
- **Notes:** Added `input_tokens`, `output_tokens`, `cache_read_tokens`, `cache_creation_tokens` (all `Measurement`) to `ModelResponse` **only**. Populated `Measurement::Unavailable` at both emission sites (`handle_gen`, `handle_analyze`). §4.8's `Measurement` paid off exactly as intended — fields born correctly typed. Did not touch `ModelProvider`'s contract (extending it with a token-reporting method stays explicitly out of scope, per the spec).

### [T-16B02] HO-13 derivation lineage + the missing `~MAP` emission

- **Spec:** l2-nodus-observability.md §4.9 (HO-13)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability for_loop_has_no_derivation_map_has_correct_n_to_n_derivation` → passed — `~FOR`'s `LoopIteration` carries `derivation: None`; a 3-element `~MAP` emits exactly 3 `LoopIteration` events (LoopType::Map), each with one `SourceRef` whose `source_index` matches its position.
- **Handoff:** completed the lineage side-band; fed T-16T01.
- **Notes:** Added `SourceRef { producing_step, source_index }` + `derivation: Option<Vec<SourceRef>>` to `LoopIteration`; added a new `LoopType::Map` variant (distinguishing it from `For`/`Until` — mislabeling a transform as a plain loop would misinform a consumer). **Scope expansion realized as planned:** added the missing `LoopIteration` emission to `execute_map` (verified beforehand it emitted nothing — confirmed at plan time). `~FOR`/`~UNTIL` get `derivation: None` — plain iteration produces no mapped output to derive from; only `~MAP`'s N→N transform gets `Some(...)`. **Unplanned finding surfaced while testing this task:** driving `~MAP` through the public `run_with_audit` API triggers the validator's pre-existing `E004` ("$it used but never assigned") — the validator doesn't know `~MAP` binds `$it` implicitly at runtime, so it false-positives on *every* `~MAP` workflow, and no existing crate test had ever executed a `~MAP` workflow end-to-end (only parsed it) so this was previously undetected. Out of scope for an observability phase to fix; the integration test bypasses `run_with_audit`'s validate-gate and drives `Executor::execute` directly instead, exercising exactly what this phase changed. Flagged for a future validator fix (not filed as a new phase — noted here and in STATE.md for now).

### [T-16C01] HO-10 `classify_trace` — read-side, no field

- **Spec:** l2-nodus-observability.md §4.9 (HO-10)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib observability::classify_trace` (5 unit tests) + `cargo test -p nodus --test observability classify_trace_all_outcomes` → all four outcomes covered and passed, including a real run's trace classifying `Complete` and a synthetically-damaged one (last event popped) classifying `GapDamaged`.
- **Handoff:** completed the read-side honesty rule; fed T-16T01.
- **Notes:** Added `TraceCompleteness { Complete, GapDamaged, Truncated, Empty }` + `classify_trace(durable_events, manifest) -> TraceCompleteness` — pure, no new field, no new emission. Reuses §4.8's `event_count == highest seq + 1` identity directly rather than a second mechanism.

### [T-16C02] HO-17 durable-only discipline

- **Spec:** l2-nodus-observability.md §4.9 (HO-17)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability all_events_from_a_real_run_are_durable` → passed. `emit`'s doc comment now states the durable-only `seq` contract explicitly.
- **Handoff:** completed HO-17 — **all 20 HO invariants are now realized in `crates/nodus`.**
- **Notes:** Realized as (1) the rule documented on `Executor::emit`'s doc comment (found and fixed a pre-existing, unrelated defect while editing it: `execute_inner`'s own doc comment had been split in half by Phase 15's `emit` insertion, with `emit`'s doc sandwiched in the middle — reunited both), and (2) the integration test pinning that core emits only `Durable`. Did **not** build an `emit_transient` companion, per the phase-file guardrail — nothing would call it yet.

### [T-16T01] Validation Task — annotation & completeness contract suite

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-observability.md` §4.9 (HO-8/9/10/11/13/16/17), and confirm HO-5 observer neutrality and LP-1 zero-dep still hold. **All twenty HO invariants are realized.**
- **Method:** Extended `tests/observability.rs` with a T-16T01 section (5 new integration tests): `all_events_from_a_real_run_are_durable`, `all_events_from_a_real_run_have_unpopulated_annotations`, `model_response_token_classes_are_unavailable_not_zero`, `for_loop_has_no_derivation_map_has_correct_n_to_n_derivation`, `classify_trace_all_outcomes`; plus 8 new `observability.rs` unit tests for the carrier/lineage/classification types directly.
- **Status:** Done
- **Verify:** `cargo test -p nodus --test observability` → 24 passed (was 19; +5). Full-crate `cargo test -p nodus` → **365 passed** (was 352; +13 across lib+integration), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean. `cargo fmt -p nodus -- --check` → clean (after one `cargo fmt` pass, no logic change). No `.unwrap()`/`panic!()`/`unreachable!()` introduced in production code (checked directly). `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; LP-1 zero-dep preserved.

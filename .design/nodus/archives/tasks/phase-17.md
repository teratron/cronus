---
phase: 17
name: "~MAP Conformance & End-to-End Reachability"
status: Done
subsystem: "crates/nodus"
requires: [11, 16]
provides: ["~MAP reachable through workflows::run/run_with_audit", "map_transform.nodus fixture"]
key_files:
  created:
    - crates/nodus/tests/fixtures/map_transform.nodus
  modified:
    - crates/nodus/src/validator.rs
    - crates/nodus/tests/control_flow.rs
    - crates/nodus/tests/parity.rs
    - crates/nodus/tests/observability.rs
    - .design/nodus/specifications/l2-nodus-control-flow.md
patterns_established:
  - "Conformance-defect phases need no new spec when the code diverges from an already-Stable contract (C12.1) — plan directly against the existing spec section."
  - "A construct's own coverage gap (no end-to-end test through the public validated API) is itself diagnostic: the one construct never tested that way was the one that didn't work that way."
duration_minutes: ~
---

# Stage 17 Tasks — ~MAP Conformance & End-to-End Reachability

**Phase:** 17
**Status:** Todo
**Strategic Goal:** Restore conformance to `l2-nodus-control-flow.md` §4.3/§4.4 (Stable v1.0.0), which mandate `$it` as `~MAP`'s implicit per-element binding. The realized validator contradicts that mandate: `E004` ("$it used but never assigned") does not know about the implicit binding, so **every** `~MAP` workflow is rejected before it runs — `~MAP` is unreachable through every validated public entry point (`run`, `run_with_audit`, `run_with_provider`, …). Close the defect, then close the coverage hole that let it ship undetected. Sequential (Track B's tests cannot pass until Track A lands).

> **Why this went undetected for six phases.** Not a missing test in the ordinary sense: `tests/control_flow.rs` drives `!HALT`, `!PAUSE`, `?SWITCH`, and `~RETRY:n` end-to-end through `workflows::run`, but has **zero** `~MAP` coverage — `~MAP` had only parser/executor-level tests that construct the AST directly and never cross the validator gate. The one construct with no end-to-end test is the one construct that does not work end-to-end.
> **Scope note — one construct, not an audit.** `~MAP` is the *only* construct with an implicit binding the validator does not declare: `~FOR` declares its explicit loop variable (`collect_vars_for`), and `~UNTIL`/`~PARALLEL` introduce no bindings at all (verified at plan time). Do **not** widen this into a general validator audit or a fixture-corpus backfill for the other v0.7 constructs — those already have passing end-to-end tests, so there is no defect behind them. Corpus tidying is noted in the Backlog instead.

## Atomic Checklist

- [x] [T-17A01] Teach `E004` that `~MAP` binds `$it` — the conformance fix
- [x] [T-17B01] `~MAP` end-to-end coverage in `tests/control_flow.rs` + normative fixture
- [x] [T-17B02] Retire the Phase-16 `Executor::execute` workaround in `tests/observability.rs`
- [x] [T-17C01] Sync `l2-nodus-control-flow.md` §4.5 — record the `E004`↔`$it` interaction
- [x] [T-17T01] Validation suite — reachability, gates, zero-dep

## Detailed Tracking

### [T-17A01] Teach `E004` that `~MAP` binds `$it` — the conformance fix

- **Spec:** l2-nodus-control-flow.md §4.3 ("`$it` is the implicit per-element binding") + §4.4 (`Stmt::Map` → "bind $it, run the command")
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` → 43 passed, including the two new tests `e004_does_not_fire_on_map_implicit_it` and `e004_still_fires_on_it_used_outside_map`.
- **Handoff:** unblocked Track B.
- **Notes:** Localized to the `Stmt::Map(mb)` arm of `collect_vars_stmt` in `validator.rs`: it marked `mb.collection` used and walked `mb.command` (marking `$it` used), but never *declared* `$it` — one `declared.insert("$it".to_string())` line fixes it, mirroring how `collect_vars_for` declares `~FOR`'s explicit loop variable. Did **not** add `$it` to `vocab::RESERVED_VARIABLES` — confirmed that constant is consulted by other code paths and would declare `$it` globally, silently disabling the diagnostic for workflows with no `~MAP` at all. The existing collector uses one flat `declared` set with no scope stack (`~FOR`'s variable already leaks past its loop today); matched that precedent rather than introducing scoping for `~MAP` alone — proper lexical scoping for all binding constructs stays in the Backlog as a separate, non-additive design concern.

### [T-17B01] `~MAP` end-to-end coverage in `tests/control_flow.rs` + normative fixture

- **Spec:** l2-nodus-control-flow.md §3 (empty/non-list ⇒ empty list, never errors) + §4.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test control_flow` → 10 passed, including the three new tests (`map_transforms_each_element_producing_an_n_element_list`, `map_over_empty_collection_yields_empty_list_no_error`, `map_over_non_list_collection_yields_empty_list_no_error`), all driving `workflows::run`. `cargo test -p nodus --test parity` → 37 passed, including `validation::map_transform_no_block_errors`, `transpilation::compact_map_transform_round_trip`, `execution::map_transform_executes_ok`.
- **Handoff:** proved the defect is closed at the API surface, not just in the validator unit; fed T-17T01.
- **Notes:** Put the three behavioral assertions (N-element, empty, non-list) in `control_flow.rs` as inline consts, matching that file's own established style (it doesn't use the fixture corpus at all — its four sibling constructs are inline `const ..._WF` literals). Put the fixture itself (`tests/fixtures/map_transform.nodus`) and its validate/round-trip/execute-ok coverage in `parity.rs`, matching where `for_loop.nodus`/`parallel_join.nodus` etc. are actually wired (grepped first rather than assuming — the normative corpus lives in `parity.rs`, not `control_flow.rs`). This is the first v0.7 control construct in the fixture corpus; `?SWITCH`/`~RETRY`/`!HALT`/`!PAUSE` remain corpus-absent but already have passing coverage in `control_flow.rs`, so no defect sits behind that gap (left in Backlog as tidying, not pulled into this phase). Verified the Phase-15 `LOG`-locks-`$out` fixture bug does not recur: the fixture's `LOG($out)` is its last step with no write to `$out` afterward, so the lock is harmless.

### [T-17B02] Retire the Phase-16 `Executor::execute` workaround in `tests/observability.rs`

- **Spec:** l2-nodus-observability.md §4.9 (HO-13 derivation lineage)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability` → 24 passed, including `for_loop_has_no_derivation_map_has_correct_n_to_n_derivation` now driving `run_with_audit` like every other test in the file.
- **Handoff:** removed the phase's own recorded technical debt; fed T-17T01.
- **Notes:** Replaced the direct `nodus::parser::Parser::parse` + `Executor::with_audit(...).execute(...)` construction and its explanatory comment with a plain `run_with_audit(MAP_WF, ...)` call — the bypass's justification (E004 rejecting `~MAP`) no longer holds once T-17A01 lands, so leaving it in place would have silently stopped testing the validate-gate path for this fixture.

### [T-17C01] Sync `l2-nodus-control-flow.md` §4.5 — record the `E004`↔`$it` interaction

- **Spec:** l2-nodus-control-flow.md §4.5 (Validator)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** §4.5 now states `~MAP`'s implicit `$it` counts as declared for the pre-existing variable-declaration check, on `~FOR`'s existing file-wide-set terms (not per-construct scoping — verified this matches the actual T-17A01 implementation before writing it, rather than asserting a stricter scoping guarantee the code doesn't provide). Spec 1.0.0 → 1.0.1 with a Document History row; `INDEX.md` entry + top-level Version (1.0.61 → 1.0.62) + Last-Updated note updated in lockstep.
- **Handoff:** closed the documentation gap that let the defect ship.
- **Notes:** §4.5 enumerated only the three validator rules Phase 11 *added* and never recorded which *existing* rule the new `$it` binding invalidated — the implementation followed §4.5 faithfully and still shipped a contradiction with §4.3/§4.4. Patch-level sync only: status stays `Stable`, no design change, no new invariant. Precedent: Phase 13 patched `l2-nodus-config` 1.0.0 → 1.0.1 mid-run for the same reason (implementation revealed a spec imprecision).

### [T-17T01] Validation Task — reachability, gates, zero-dep

- **Goal:** Verify `~MAP` is reachable through the public validated API per `l2-nodus-control-flow.md` §4.3/§4.4, and confirm the fix narrows nothing else.
- **Method:** Full-crate `cargo test -p nodus`, `cargo clippy -p nodus --all-targets -- -D warnings`, `cargo fmt -p nodus -- --check`, a source grep for `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`, `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock`.
- **Status:** Done
- **Verify:** `cargo test -p nodus` → **373 passed** (was 365; +8: 2 validator unit tests, 3 control_flow integration tests, 3 parity integration tests), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean. `cargo fmt -p nodus -- --check` → clean, no reformat needed. Grep for `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]` in `validator.rs`/`executor.rs` → only pre-existing test-module hits; the two new `.expect("parse")` calls in T-17A01's tests are themselves inside `#[cfg(test)]`, matching the file's existing convention. `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; LP-1 zero-dep preserved. Regression check: the pre-existing `E004` behavior for non-`~MAP` workflows is unchanged (covered by `e004_still_fires_on_it_used_outside_map` plus the untouched pre-existing E004-adjacent tests).

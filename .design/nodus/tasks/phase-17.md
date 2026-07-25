---
phase: 17
name: "~MAP Conformance & End-to-End Reachability"
status: Todo
subsystem: "crates/nodus"
requires: [11, 16]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 17 Tasks — ~MAP Conformance & End-to-End Reachability

**Phase:** 17
**Status:** Todo
**Strategic Goal:** Restore conformance to `l2-nodus-control-flow.md` §4.3/§4.4 (Stable v1.0.0), which mandate `$it` as `~MAP`'s implicit per-element binding. The realized validator contradicts that mandate: `E004` ("$it used but never assigned") does not know about the implicit binding, so **every** `~MAP` workflow is rejected before it runs — `~MAP` is unreachable through every validated public entry point (`run`, `run_with_audit`, `run_with_provider`, …). Close the defect, then close the coverage hole that let it ship undetected. Sequential (Track B's tests cannot pass until Track A lands).

> **Why this went undetected for six phases.** Not a missing test in the ordinary sense: `tests/control_flow.rs` drives `!HALT`, `!PAUSE`, `?SWITCH`, and `~RETRY:n` end-to-end through `workflows::run`, but has **zero** `~MAP` coverage — `~MAP` had only parser/executor-level tests that construct the AST directly and never cross the validator gate. The one construct with no end-to-end test is the one construct that does not work end-to-end.

> **Scope note — one construct, not an audit.** `~MAP` is the *only* construct with an implicit binding the validator does not declare: `~FOR` declares its explicit loop variable (`collect_vars_for`), and `~UNTIL`/`~PARALLEL` introduce no bindings at all (verified at plan time). Do **not** widen this into a general validator audit or a fixture-corpus backfill for the other v0.7 constructs — those already have passing end-to-end tests, so there is no defect behind them. Corpus tidying is noted in the Backlog instead.

## Atomic Checklist

- [ ] [T-17A01] Teach `E004` that `~MAP` binds `$it` — the conformance fix
- [ ] [T-17B01] `~MAP` end-to-end coverage in `tests/control_flow.rs` + normative fixture
- [ ] [T-17B02] Retire the Phase-16 `Executor::execute` workaround in `tests/observability.rs`
- [ ] [T-17C01] Sync `l2-nodus-control-flow.md` §4.5 — record the `E004`↔`$it` interaction
- [ ] [T-17T01] Validation suite — reachability, gates, zero-dep

## Detailed Tracking

### [T-17A01] Teach `E004` that `~MAP` binds `$it` — the conformance fix

- **Spec:** l2-nodus-control-flow.md §4.3 ("`$it` is the implicit per-element binding") + §4.4 (`Stmt::Map` → "bind $it, run the command")
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` — a new unit test asserts a `~MAP` workflow produces **zero** `Severity::Error` diagnostics from `Validator::validate`; a second asserts `$it` used **outside** any `~MAP` still raises `E004` (the fix must not blanket-declare it).
- **Handoff:** unblocks every Track B task — their tests cannot pass until this lands.
- **Notes:** The defect is localized to the `Stmt::Map(mb)` arm of `collect_vars_stmt` in `validator.rs`: it marks `mb.collection` as *used* and walks `mb.command` (which marks `$it` used), but never *declares* `$it` — so `E004`'s `used.difference(&declared)` reports it. Fix by declaring `$it` in that arm before walking the command, mirroring how `collect_vars_for` declares `~FOR`'s explicit loop variable. **Do NOT add `$it` to `vocab::RESERVED_VARIABLES`** — that constant is consulted by other code paths and would declare `$it` globally, including in workflows with no `~MAP` at all, silently disabling a legitimate diagnostic. Note the existing collector uses one flat `declared` set with no scope stack (`~FOR`'s variable also leaks past its loop today); match that precedent rather than introducing scoping for `~MAP` alone — proper lexical scoping for all binding constructs is a separate concern, recorded in the Backlog.

### [T-17B01] `~MAP` end-to-end coverage in `tests/control_flow.rs` + normative fixture

- **Spec:** l2-nodus-control-flow.md §3 (empty/non-list ⇒ empty list, never errors) + §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** new tests in `crates/nodus/tests/control_flow.rs` drive **`workflows::run`** (the validated public entry point its four sibling constructs already use — *not* `Executor::execute`) and assert: an N-element collection transforms to a `Value::List` of N; an empty collection yields an empty list with no error; a non-list collection yields an empty list with no error. Plus `tests/fixtures/map_transform.nodus` parses, validates with zero errors, and round-trips through the transpiler (NL-6).
- **Handoff:** proves the defect is actually closed at the API surface, not just in the validator unit.
- **Notes:** This is the test that would have caught the defect in Phase 11. Use `workflows::run` deliberately — routing through the validator gate *is* the thing under test. The normative fixture corpus (`tests/fixtures/`, established in Phase 2) currently contains no v0.7 control construct at all (`~FOR`/`~UNTIL`/`~PARALLEL`/`RUN(@` only); `map_transform.nodus` is the first. Watch the fixture's step ordering: `LOG` unconditionally locks `$out`, so a `LOG` placed before a later `→ $out` pipeline produces a spurious `RULE_VIOLATION` (the Phase-15 fixture bug — do not reintroduce it).

### [T-17B02] Retire the Phase-16 `Executor::execute` workaround in `tests/observability.rs`

- **Spec:** l2-nodus-observability.md §4.9 (HO-13 derivation lineage)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability for_loop_has_no_derivation_map_has_correct_n_to_n_derivation` passes while driving `run_with_audit` (as every other test in that file does); the inline comment explaining the validate-gate bypass is removed along with the direct `nodus::parser::Parser::parse` + `Executor::with_audit(...).execute(...)` construction.
- **Handoff:** removes the phase's own recorded technical debt; feeds T-17T01.
- **Notes:** Phase 16 could not drive `~MAP` through `run_with_audit` because of exactly the defect T-17A01 fixes, so it bypassed the gate and documented why. With the fix in, that justification is void and the workaround must go — a bypass left in place after its cause is fixed silently stops testing the gate it was avoiding. This task exists to make sure the debt is actually collected rather than quietly inherited.

### [T-17C01] Sync `l2-nodus-control-flow.md` §4.5 — record the `E004`↔`$it` interaction

- **Spec:** l2-nodus-control-flow.md §4.5 (Validator)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** §4.5 states that `~MAP` binds `$it` for variable-declaration analysis (so `E004` must not flag it), spec version 1.0.0 → 1.0.1 with a Document History row, and `INDEX.md`'s entry updated in lockstep (header parity is a Pre-flight HALT condition next cycle if it drifts).
- **Handoff:** closes the documentation gap that produced the defect.
- **Notes:** §4.5 enumerates only the three validator rules Phase 11 *added* (`~RETRY` bounds, `!HALT`-requires-escalate, `?SWITCH`-empty-arms) and never records which *existing* rule the new binding invalidated — so the implementation followed §4.5 faithfully and still shipped a contradiction with §4.3/§4.4. Patch-level sync only: status stays `Stable`, no design change, no new invariant. Precedent: Phase 13 patched `l2-nodus-config` 1.0.0 → 1.0.1 mid-run when implementation revealed a spec imprecision.

### [T-17T01] Validation Task — reachability, gates, zero-dep

- **Goal:** Verify `~MAP` is reachable through the public validated API per `l2-nodus-control-flow.md` §4.3/§4.4, and confirm the fix narrows nothing else.
- **Method:** Full-crate `cargo test -p nodus` (expect 365 + the new tests, 0 failed); `cargo clippy -p nodus --all-targets -- -D warnings` clean; `cargo fmt -p nodus -- --check` clean; no `.unwrap()`/`panic!()`/`unreachable!()` introduced on production paths; `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` empty (LP-1 zero-dep). Regression check: the pre-existing `E004` tests still pass — the fix must not weaken the diagnostic for non-`~MAP` workflows.
- **Status:** Todo

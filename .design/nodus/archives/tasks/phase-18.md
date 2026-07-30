---
phase: 18
name: "Bounded Whole-Run Self-Restart"
status: Done
subsystem: "crates/nodus"
requires: [11, 13]
provides: ["restart_max declaration", "$restart/$restart_count reserved variables", "Executor::execute_with_restart", "E018/E019 validator rules", "NODUS:RESTART_LIMIT"]
key_files:
  created:
    - crates/nodus/tests/restart.rs
  modified:
    - crates/nodus/src/ast.rs
    - crates/nodus/src/parser.rs
    - crates/nodus/src/transpiler.rs
    - crates/nodus/src/validator.rs
    - crates/nodus/src/vocab.rs
    - crates/nodus/src/executor.rs
patterns_established:
  - "Wrap a bounded retry/restart loop AROUND the function that already builds fresh state on entry, rather than adding a reset routine inside it — freshness becomes structural, not maintained."
  - "Prefer validator-only enforcement for structural AST rules (matching E010/E013/E014/E016/E017) over duplicating a runtime Signal-based check, when the crate's public Executor API can be invoked directly with an unvalidated AST anyway — a runtime check would be the first redundant enforcement layer in the crate, not a safety net."
duration_minutes: ~
---

# Stage 18 Tasks — Bounded Whole-Run Self-Restart

**Phase:** 18
**Status:** Todo
**Strategic Goal:** Realize `l2-nodus-restart.md` (Stable v1.0.0) in `crates/nodus` — NL-23: a workflow may restart its entire run from step 1, re-reading `@in`/`§config`, bounded by a declared ceiling with a visible carried count, requestable only from a run-boundary step, reconstructing fresh rather than inheriting the prior attempt's context. Opt-in via `§runtime: { restart_max: n }`; absent, behavior is byte-identical to today. Sequential tracks A (declaration surface) → B (control flow) + validation.

> **Spec inaccuracy to work around — `$restart` must stay writable.** The spec's §3 Invariant Compliance row (authoritative) puts **only `$restart_count`** in `RUNTIME_OWNED_VARIABLES`. Its §5 slice-3 line and the `INDEX.md` row loosely group `$restart` there as well — that is wrong and must **not** be implemented: `RUNTIME_OWNED_VARIABLES` membership makes a pipeline target an `E013` error, so a runtime-owned `$restart` would make the request unwritable and the whole feature unusable. Implement per §3: `$restart` is **reserved but writable** (like `$out`/`$draft`), `$restart_count` is **reserved and runtime-owned** (unforgeable). Flagged for correction on the next `/magic.spec` touch of this file.

> **Ordering note — the control-flow slice lands last.** T-18B02 is the only task that changes an existing run's control flow. Every guard it relies on (ceiling bound, error codes, reserved variables, boundary authority) is provable before it starts, so a failure there is unambiguous. Do not reorder it earlier for convenience.

## Atomic Checklist

- [x] [T-18A01] `restart_max` in `§runtime:` + bound check (1..=10)
- [x] [T-18A02] Error code — `RESTART_LIMIT` (Warn); `RESTART_SCOPE` redesigned as a bare validator code (E019), see notes
- [x] [T-18A03] `$restart` (writable) + `$restart_count` (runtime-owned) reserved variables
- [x] [T-18B01] `E019` static rejection of a nested `$restart` request (redesigned — no `Signal::Restart`, see notes)
- [x] [T-18B02] The bounded attempt loop around `execute_inner` + `RESTART_LIMIT`
- [x] [T-18T01] Validation suite — bound, authority, freshness, additivity, zero-dep

## Detailed Tracking

### [T-18A01] `restart_max` in `§runtime:` + bound check

- **Spec:** l2-nodus-restart.md §4.1 (declaring the ceiling) + §4.4 (bound check)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib parser::` → `parses_restart_max_when_declared` + `restart_max_absent_when_not_declared` pass. `cargo test -p nodus --lib validator::` → `e018_fires_on_unbounded_or_oversized_restart_max` + `no_e018_for_valid_restart_max` pass (n=0/11 raise E018; n=1/10 do not).
- **Handoff:** the declaration every later task reads; landed first.
- **Notes:** Added `restart_max: Option<u32>` to `RuntimeBlock` alongside `core`/`extends`/`agents`/`mode`; slotted into `parse_runtime_braces`'s existing `{ key: value }` scan (no new parse shape). Bound check `e018_restart_max_bounded` mirrors `e017_retry_bounded`'s exact shape. Extended the transpiler's compact runtime-block emitter so the key round-trips (verified in `tests/restart.rs::restart_max_survives_compact_round_trip`).

### [T-18A02] Error code(s)

- **Spec:** l2-nodus-restart.md §4.6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib vocab::` → `error_registry_lockstep` passes with 27 canonical codes (was 26); `error_meta(RESTART_LIMIT) == Some((Warn, Control))`.
- **Handoff:** fed T-18B02, which emits it.
- **Notes:** **Deviated from the spec as written — only `RESTART_LIMIT` became a `vocab.rs` `NODUS:*` code.** `RESTART_SCOPE` did **not** — see T-18B01's notes for why. `RESTART_LIMIT` follows the Phase-13 `CONFIG_INVALID` precedent exactly (constant + `error_meta` row + lockstep-array entry, all three together) and is `Warn`, mirroring `MAX_REACHED` — a bounded construct reaching its bound is a normal reported outcome.

### [T-18A03] `$restart` + `$restart_count` reserved variables

- **Spec:** l2-nodus-restart.md §3 (NL-8 row — authoritative) + §4.2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` → `e013_fires_when_pipeline_target_is_restart_count` + `no_e013_for_restart_request_target` pass. `cargo test -p nodus --lib vocab::` → `runtime_owned_is_subset_of_reserved` + `runtime_owned_excludes_writable_reserved` (pre-existing sanity tests) pass unchanged, confirming the split landed correctly with no new test needed for that invariant.
- **Handoff:** the request/exposure surface T-18B02 reads and writes.
- **Notes:** Implemented per the phase-file guardrail: `$restart` → `RESERVED_VARIABLES` only (writable, `$out`/`$draft` precedent); `$restart_count` → both `RESERVED_VARIABLES` and `RUNTIME_OWNED_VARIABLES` (unforgeable).

### [T-18B01] Nested-request rejection — redesigned from `Signal::Restart` to a validator-only static rule

- **Spec:** l2-nodus-restart.md §4.2 (signal) + §4.4 (validator) + §3 (NL-23(b) row)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` → `e019_fires_on_restart_request_nested_in_for_loop`, `e019_fires_on_restart_request_nested_in_switch_arm`, `no_e019_for_top_level_restart_request` pass. `cargo test -p nodus --test restart` → `nested_restart_request_rejected_by_validator` confirms `workflows::run` refuses the workflow (E019) before any execution is attempted.
- **Handoff:** the authority gate; landed before T-18B02's loop, though the loop no longer depends on it directly (see below).
- **Notes:** **`[DR]` Dropped `Signal::Restart` and moved rejection entirely into the validator, as a new bare code `E019` (not a `vocab.rs` `NODUS:*` constant).** Criterion: grounding surfaced that this codebase enforces every comparable structural rule (E010 `~UNTIL` max, E013 reserved pipeline target, E014 forward reference, E016 halt-requires-escalate, E017 retry bound) *validator-only*, with zero runtime-executor re-checks — `Executor::execute` called directly (a real, precedented pattern in this very crate's own test suite) always assumes a validated AST. Building `Signal`-based runtime interception would have been the *first* redundant-enforcement mechanism in the crate, and would have needed to cover ~FOR/~PARALLEL/?SWITCH bodies plus a genuinely ambiguous question the spec never resolves (whether `~MAP`'s aggregate `target` write counts as "inside the per-item stage" — it structurally can't, since `execute_map` always overwrites the inner command's own `pipeline_target` before running it, so `$restart` can never be that per-element command's target; only a nested container's *own* body/branches/arms can carry a real `$restart`-targeting command). The static walk (`e019_restart_scope` → `restart_scope_stmt` / `flag_restart_stmt`, mirroring `find_empty_switches_stmt`'s recursive-descent shape) covers `~FOR` bodies, `~PARALLEL` branches, and `?SWITCH` arms/default unconditionally (regardless of top-level-ness — a top-level `~FOR` is still a per-item context); `~UNTIL` is deliberately **not** walked (an explicit spec omission, left as-is rather than silently expanded) and `~MAP` needs no special case at all, per the structural argument above. Flagged both deviations (`RESTART_SCOPE` code registry, `Signal::Restart`) in `PLAN.md` Backlog for the next `/magic.spec` touch of `l2-nodus-restart.md`.

### [T-18B02] The bounded attempt loop

- **Spec:** l2-nodus-restart.md §4.3 (the attempt loop) + §3 (NL-23(a)/(c)/(d) rows)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test restart` → `ceiling_exhaustion_stops_after_max_plus_one_attempts_and_flags_restart_limit` (restart_max:2 → 3 manifests, `RESTART_LIMIT` flag, final `restart_count == 2`), `restart_count_progresses_and_context_is_fresh_each_attempt` (2 manifests, final `restart_count == 1`, prior-attempt-only variable absent from the final result), `no_restart_max_is_byte_identical_to_the_pre_nl23_path` (1 manifest, no `restart_count` key seeded at all).
- **Handoff:** completes NL-23; fed T-18T01.
- **Notes:** New `Executor::execute_with_restart` wraps **around** `execute_inner` — the four existing public callers (`execute`, `execute_with_params`, `execute_with_manifest_context`, `execute_for_environment`) now call it instead, with **no signature change** to any of them (verified by the full pre-existing suite passing unchanged). When `restart_max` is absent, it degrades to exactly one `execute_inner` call plus a check-and-flag (no `restart_count` seeding at all) — the additivity baseline. When present, `Self::seed_restart_count` merges `restart_count` into the `@in`-overlay input map fresh each attempt (LG-5 falls out of `execute_inner` building a new `ExecutionContext` every call, not from any explicit reset). `RESTART_LIMIT` is pushed to `result.flags` (not `result.errors`), matching how `MAX_REACHED` is recorded — confirmed by reading `execute_until` before writing this, not assumed. **A pre-existing, unrelated parser bug was found and worked around, not fixed:** `?IF cond → CMD(args) → $target` (the inline-conditional-action form) never actually attaches the trailing pipeline target — `try_parse_command_from_string` parses `NAME(args)` and silently discards anything after the closing paren, so every inline-`?IF` action's `pipeline_target` comes back `None` regardless of source text (confirmed by parsing a fixture and inspecting the AST directly). This predates NL-23 and affects any workflow using that syntax, not just restart requests. Out of scope to fix here; `tests/restart.rs`'s attempt-count-dependent scenario constructs its `WorkflowFile` directly (bypassing the parser, matching the Phase-16 precedent for an unrelated parser/validator gap) instead of relying on that broken syntax. Flagged in `PLAN.md` Backlog for a future fix.

### [T-18T01] Validation Task — bound, authority, freshness, additivity

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-restart.md` (NL-23(a)–(e)) and confirm NL-6/NL-7/NL-8 and LP-1 still hold.
- **Method:** New `crates/nodus/tests/restart.rs` (the one-file-per-cluster pattern established by `config.rs`/`dialog.rs`/`environment.rs`) — 5 tests covering ceiling exhaustion, nested-request rejection, `$restart_count` progression + fresh reconstruction, the no-`restart_max` additivity baseline, and the `restart_max` transpiler round-trip. Plus 9 new unit tests directly in `parser.rs`/`validator.rs`.
- **Status:** Done
- **Verify:** `cargo test -p nodus` → **387 passed** (was 373; +14: 9 lib unit tests + 5 `tests/restart.rs` integration tests), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean (fixed one `redundant_closure` finding: `.is_some_and(|v| Self::is_truthy(v))` → `.is_some_and(Self::is_truthy)`). `cargo fmt -p nodus -- --check` → clean (after one `cargo fmt` pass, no logic change). No `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` introduced in production code — the two new `.unwrap()` calls in `parser.rs` are both inside `#[cfg(test)] mod tests`, matching the file's existing convention (`parses_runtime_mode` uses the same style). `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; LP-1 zero-dep preserved.

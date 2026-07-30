---
phase: 18
name: "Bounded Whole-Run Self-Restart"
status: Todo
subsystem: "crates/nodus"
requires: [11, 13]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 18 Tasks — Bounded Whole-Run Self-Restart

**Phase:** 18
**Status:** Todo
**Strategic Goal:** Realize `l2-nodus-restart.md` (Stable v1.0.0) in `crates/nodus` — NL-23: a workflow may restart its entire run from step 1, re-reading `@in`/`§config`, bounded by a declared ceiling with a visible carried count, requestable only from a run-boundary step, reconstructing fresh rather than inheriting the prior attempt's context. Opt-in via `§runtime: { restart_max: n }`; absent, behavior is byte-identical to today. Sequential tracks A (declaration surface) → B (control flow) + validation.

> **Spec inaccuracy to work around — `$restart` must stay writable.** The spec's §3 Invariant Compliance row (authoritative) puts **only `$restart_count`** in `RUNTIME_OWNED_VARIABLES`. Its §5 slice-3 line and the `INDEX.md` row loosely group `$restart` there as well — that is wrong and must **not** be implemented: `RUNTIME_OWNED_VARIABLES` membership makes a pipeline target an `E013` error, so a runtime-owned `$restart` would make the request unwritable and the whole feature unusable. Implement per §3: `$restart` is **reserved but writable** (like `$out`/`$draft`), `$restart_count` is **reserved and runtime-owned** (unforgeable). Flagged for correction on the next `/magic.spec` touch of this file.

> **Ordering note — the control-flow slice lands last.** T-18B02 is the only task that changes an existing run's control flow. Every guard it relies on (ceiling bound, error codes, reserved variables, boundary authority) is provable before it starts, so a failure there is unambiguous. Do not reorder it earlier for convenience.

## Atomic Checklist

- [ ] [T-18A01] `restart_max` in `§runtime:` + bound check (1..=10)
- [ ] [T-18A02] Two error codes — `RESTART_LIMIT` (Warn) / `RESTART_SCOPE` (Error)
- [ ] [T-18A03] `$restart` (writable) + `$restart_count` (runtime-owned) reserved variables
- [ ] [T-18B01] `Signal::Restart` + top-level-only raise + `RESTART_SCOPE` static rule
- [ ] [T-18B02] The bounded attempt loop around `execute_inner` + `RESTART_LIMIT`
- [ ] [T-18T01] Validation suite — bound, authority, freshness, additivity, zero-dep

## Detailed Tracking

### [T-18A01] `restart_max` in `§runtime:` + bound check

- **Spec:** l2-nodus-restart.md §4.1 (declaring the ceiling) + §4.4 (bound check)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib parser::` — a `§runtime: { core: schema.nodus, restart_max: 3 }` block parses to `RuntimeBlock.restart_max == Some(3)`, and a block without the key parses to `None`. `cargo test -p nodus --lib validator::` — `restart_max: 0` and `restart_max: 11` each raise a `Severity::Error` diagnostic; `restart_max: 1` and `restart_max: 10` do not.
- **Handoff:** the declaration every later task reads; lands first.
- **Notes:** `RuntimeBlock` already carries `core`/`extends`/`agents`/`mode`, so add `restart_max: Option<u32>` alongside. `parse_runtime_braces` already scans `{ key: value, … }` populating known keys — `restart_max` slots into that existing loop, no new parse shape. Mirror `e017_retry_bounded`'s structure for the bound check (it filters `step.retry` the same way this filters the runtime block), so the run grain inherits the same `1..=10` sanity ceiling as the step grain. Also extend the transpiler's runtime-block emitter so the key round-trips (NL-6).

### [T-18A02] Two error codes

- **Spec:** l2-nodus-restart.md §4.6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib vocab::` — the lockstep test passes with both new codes registered; `error_meta(RESTART_LIMIT) == Some((Warn, Control))` and `error_meta(RESTART_SCOPE) == Some((Error, Control))`.
- **Handoff:** feeds T-18B01/B02, which emit them.
- **Notes:** Follow the Phase-13 `CONFIG_INVALID` precedent exactly: constant in `error_code`, row in `error_meta`, entry added to the `error_registry_lockstep` canonical array (the test fails if any canonical code lacks metadata, so all three edits land together). `RESTART_LIMIT` is deliberately `Warn` — a bounded construct reaching its bound is a normal reported outcome, mirroring `MAX_REACHED` — while `RESTART_SCOPE` is `Error` because a request from a per-item context is a structural mistake, not a graded outcome. Do not collapse them into one code; §6 records why a trace must distinguish a run-grain refusal from a `~UNTIL` exhaustion.

### [T-18A03] `$restart` + `$restart_count` reserved variables

- **Spec:** l2-nodus-restart.md §3 (NL-8 row — authoritative) + §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` — a workflow with `→ $restart_count` as a pipeline target raises `E013`; a workflow with `→ $restart` does **not**. `cargo test -p nodus --lib vocab::` — both names are in `RESERVED_VARIABLES`; only `$restart_count` is in `RUNTIME_OWNED_VARIABLES`.
- **Handoff:** the request/exposure surface T-18B01 and T-18B02 read and write.
- **Notes:** **Read the phase-file guardrail above before implementing this.** The asymmetry is the whole point and is easy to get wrong: the workflow must be able to *write* the request (`$restart`) but must never be able to *forge* the count (`$restart_count`) that flow logic trusts to observe its chain position. `RESERVED_VARIABLES` (16 entries) and `RUNTIME_OWNED_VARIABLES` (9 entries) are separate lists precisely to express this — `$out` and `$draft` are the existing precedent for reserved-but-writable.

### [T-18B01] `Signal::Restart` + top-level-only raise + `RESTART_SCOPE`

- **Spec:** l2-nodus-restart.md §4.2 (signal) + §4.4 (validator) + §3 (NL-23(b) row)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib validator::` — a restart request written inside a `~FOR`, `~MAP`, `~PARALLEL` body or a `?SWITCH` arm raises `RESTART_SCOPE`; the same request at top level does not. `cargo test -p nodus --test restart` — a run whose nested step requests a restart returns the `RESTART_SCOPE` error and performs no restart.
- **Handoff:** the authority gate T-18B02's loop trusts; must land before the loop.
- **Notes:** `Signal` is a **private** enum (`Break`/`Skip`/`Pause`/`Halt`) — adding `Restart` changes no public API and no `Value`, so NL-7 holds. Static detection reuses the existing AST walk that already distinguishes top-level `WorkflowFile.steps` from nested bodies (the same structural distinction Phase 17's `collect_vars_stmt` work traversed). Static where provable, refused at run time where not: the validator cannot see host-provided values, so the executor must also refuse a signal originating below top level rather than assuming the validator caught every case.

### [T-18B02] The bounded attempt loop

- **Spec:** l2-nodus-restart.md §4.3 (the attempt loop) + §3 (NL-23(a)/(c)/(d) rows)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test restart` — (a) a workflow requesting one restart runs its steps twice and returns `Status::Ok`; (b) with `restart_max: 2`, a workflow that always requests a restart attempts exactly 3 times then returns with `RESTART_LIMIT` present in the result; (c) `$restart_count` reads 0 on the first attempt and increments per attempt; (d) a variable written during attempt N is absent at the start of attempt N+1 (fresh reconstruction); (e) a workflow with no `restart_max` and no request produces a result identical to the pre-phase baseline.
- **Handoff:** completes NL-23; feeds T-18T01.
- **Notes:** Wrap **around** `execute_inner`, never inside it. That is the load-bearing choice (§4.3): `execute_inner` constructs a fresh `ExecutionContext` on entry, so re-entering the function *is* the LG-5 fresh reconstruction — no state-clearing routine exists to drift out of sync as the context gains fields later. Note `execute_inner` returns `(RunResult, bool)` and takes nine parameters including `run_id`/`started_at`; each attempt re-derives `@in` and the accepted `§config` from the same inputs. Per §4.5, each attempt is its own event stream (own `correlation_id`, own dense `seq`, own manifest) — this falls out of calling `execute_inner` per attempt and must not be "optimized" into a shared stream, which would break HO-7's `event_count == highest seq + 1` per-manifest identity. Attempt-chain linking in the manifest is explicitly **out of scope** (an `l2-nodus-observability` concern).

### [T-18T01] Validation Task — bound, authority, freshness, additivity

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-restart.md` (NL-23(a)–(e)) and confirm NL-6/NL-7/NL-8 and LP-1 still hold.
- **Method:** New `crates/nodus/tests/restart.rs` (the one-file-per-cluster pattern established by `config.rs`/`dialog.rs`/`environment.rs`) covering: ceiling exhaustion → `RESTART_LIMIT`; nested-context request → `RESTART_SCOPE` with no restart; `$restart_count` progression; fresh-reconstruction (no variable leakage across attempts); `§config` re-read per attempt; no-`restart_max` additivity baseline; `→ $restart_count` rejected by `E013` while `→ $restart` is accepted.
- **Status:** Todo
- **Verify:** `cargo test -p nodus` — full suite green (baseline 373 + the new tests), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` clean. `cargo fmt -p nodus -- --check` clean. No `.unwrap()`/`panic!()`/`unreachable!()` on production paths. `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` empty (LP-1 zero-dep). Transpiler round-trip holds for a workflow declaring `restart_max` (NL-6).

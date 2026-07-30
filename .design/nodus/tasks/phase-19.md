---
phase: 19
name: "Compensation Seam"
status: Todo
subsystem: "crates/nodus"
requires: [11, 14]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 19 Tasks — Compensation Seam

**Phase:** 19
**Status:** Todo
**Strategic Goal:** Realize `l2-nodus-compensation.md` (Stable v1.0.0) in `crates/nodus` — NL-22: an effectful step may declare a host-supplied compensating action, and when the scope fails, is cancelled, or is explicitly compensated, the compensations of its **successfully-completed** steps run in reverse order of completion (LIFO). Purely additive: a workflow declaring no compensation, or a run that never fails, behaves exactly as today. Sequential tracks A (declaration surface) → B (ledger + unwind) + validation.

> **Scope note — three compositions are vacuous in core today; do not build them.** The spec records that NL-22's stated composition with the LP-11 `decide → effect → observe` gate, with NL-12 crash-resume, and with a declared sub-region scope all name machinery that **does not exist in this crate** (verified at spec time). Do **not** invent an LP-11 gate, an NL-12 resume path, or a `~SCOPE ... ~END` construct to satisfy them. Compensations route through the one existing `execute_command` effect path (so they inherit the LP-11 seam for free when it lands); at-least-once driving is per process run with the ledger as the host's replay artifact; the run *is* the compensation scope this phase.

> **Decision already made — a failed compensation continues the unwind.** NL-22 fixes the LIFO order and requires each failure surfaced, but does not say whether a failure aborts the rest. The spec resolves this (§4.5, alternative weighed in §6): **continue** to the next entry. Do not "improve" this into abort-on-first-failure — nodus cannot verify whether the failure invalidated earlier undos (host domain), and abandoning leaves more effects live with fewer attempts recorded.

## Atomic Checklist

- [ ] [T-19A01] `~COMPENSATE` declaration surface — token, AST field, parser clause, transpiler
- [ ] [T-19A02] `NODUS:COMPENSATION_FAILED` error code
- [ ] [T-19B01] Completed-effect ledger + parallel un-compensable record
- [ ] [T-19B02] Arming condition + LIFO drain + failure recording
- [ ] [T-19T01] Validation suite — order, completed-only, fallible, armed, honesty, zero-dep

## Detailed Tracking

### [T-19A01] `~COMPENSATE` declaration surface

- **Spec:** l2-nodus-compensation.md §4.1 (lexer) + §4.2 (AST) + §4.3 (parser) + §4.7 (transpiler)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib lexer::` — `~COMPENSATE` lexes as `TokenType::TildeCompensate`, **not** as a generic `Flag`. `cargo test -p nodus --lib parser::` — `3. PUBLISH($doc) → $url ~COMPENSATE: UNPUBLISH($url)` parses to `Step.compensation == Some(CommandCall{ name: "UNPUBLISH", .. })` with the pipeline target still `$url`; a step without the clause parses to `None`. `cargo test -p nodus --lib transpiler::` — compact round-trip preserves the clause (NL-6).
- **Handoff:** the declaration surface every later task reads; lands first.
- **Notes:** `Step` already carries `retry: Option<u32>` from `~RETRY:n` — `compensation: Option<CommandCall>` is its direct structural analog, so mirror that field's treatment throughout (parser, transpiler, any exhaustive matches). **Keyword ordering matters:** `~COMPENSATE` must be matched before the generic `~identifier` flag rule, exactly as `~MAP`/`~RETRY` are; getting this wrong reproduces the concrete mis-lex-as-`Flag` bug `l2-nodus-control-flow` §1 records. Parse as a **trailing same-line clause** after the pipeline target, terminating at end-of-line — not an indented sub-clause, because the lexer emits no indent/dedent tokens (the Phase-13 `§config` lesson).

### [T-19A02] `NODUS:COMPENSATION_FAILED` error code

- **Spec:** l2-nodus-compensation.md §4.8
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib vocab::` — the `error_registry_lockstep` test passes with the new code registered, and `error_meta(COMPENSATION_FAILED) == Some((Error, Runtime))`.
- **Handoff:** feeds T-19B02, which emits it.
- **Notes:** Phase-13 `CONFIG_INVALID` precedent: constant in `error_code`, row in `error_meta`, entry in the lockstep canonical array — all three together, since the lockstep test fails if any canonical code lacks metadata.

### [T-19B01] Completed-effect ledger + un-compensable record

- **Spec:** l2-nodus-compensation.md §4.4 + §3 (NL-22(a) row)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test compensation` — after a **clean** run of a 3-step workflow where 2 steps declare `~COMPENSATE`, the ledger holds exactly those 2 entries in completion order with outcome `Pending`, the un-compensable record holds the 1 remaining completed effect, and **no** compensation has run (`Status::Ok` ⇒ no unwind).
- **Handoff:** the record T-19B02 drains; must land before the drain.
- **Notes:** Two vectors on `ExecutionContext`, appended when a step's action completes **without** a runtime error: `compensations` when `Step.compensation` is `Some`, `uncompensable` when it is `None`. The second vector is what makes NL-22's honesty rule real — an un-compensable committed effect is *recorded*, not inferred from absence — so do not skip it as redundant. Only completed effects enter: a never-started, still-running, or already-failed step must not appear, which keeps cancellation (the ordinary `Signal` interrupt) structurally distinct from the unwind. `step_identity` is already computed per step from Phase 14 (HO-15) — reuse it rather than recomputing.

### [T-19B02] Arming condition + LIFO drain + failure recording

- **Spec:** l2-nodus-compensation.md §4.5 + §3 (NL-22(b)/(c)/(d) + CO-4 rows)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test compensation` — (a) a run failing after 3 completed compensable effects runs their compensations in exactly reverse completion order (assert the observed command sequence); (b) a `Status::Ok` run runs none; (c) when the 2nd of 3 compensations fails, `NODUS:COMPENSATION_FAILED` is present, that entry's outcome is `Failed`, the remaining compensations **still run**, and the failed entry's original effect is recorded live; (d) a `!!` rule violation still fails the run and the unwind covers only the effects completed before it.
- **Handoff:** completes NL-22; feeds T-19T01.
- **Notes:** Arm only on `Status::Failed` / `Status::Aborted` or an explicit compensate request — never on `Ok`/`Partial` (NL-22(d): armed, not automatic). Drain the ledger **back-to-front** by popping, not by re-reading step order: reverse order is a correctness contract because later effects were built on earlier ones. Each compensation runs through the same `execute_command` path as any command, so it inherits the existing rule checks and event emissions (no new `ExecutionEvent` variant — HO-6). **Read the continue-on-failure guardrail above before implementing (c).** Note the existing status resolution already ranks rule violations above other outcomes; the unwind runs after the step loop and must not change the resolved status — compensation never rescues a failure into success.

### [T-19T01] Validation Task — order, completed-only, fallible, armed, honesty

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-compensation.md` (NL-22(a)–(d) + CO-4) and confirm NL-2/NL-7/HO-6 and LP-1 still hold.
- **Method:** New `crates/nodus/tests/compensation.rs` (one-file-per-cluster pattern) covering: reverse-completion-order driving; completed-only (a failed step's own compensation never runs); fallible compensation surfaced with the original effect live and the unwind continuing; armed-not-automatic (clean run keeps effects); un-compensable committed effects recorded explicitly; `!!` bypass unchanged; observer neutrality (a run with an audit provider attached emits no new event variant).
- **Status:** Todo
- **Verify:** `cargo test -p nodus` — full suite green (baseline 373 + the new tests, plus Phase 18's if it landed first), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` clean. `cargo fmt -p nodus -- --check` clean. No `.unwrap()`/`panic!()`/`unreachable!()` on production paths. `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` empty (LP-1 zero-dep). A workflow declaring no `~COMPENSATE` produces a result identical to the pre-phase baseline (additivity).

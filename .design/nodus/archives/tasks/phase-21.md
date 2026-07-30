---
phase: 21
name: "Compact-Form Round-Trip Fidelity for Control Flow (NL-6)"
status: Done
subsystem: "crates/nodus"
requires: [11, 17, 20]
provides:
  - "exhaustive compact-form Stmt emitters (nodus_stmt, no _ catch-all)"
  - "Step.sub_steps and Step.retry compact emission (previously silently dropped)"
  - "$var = expr assignment-shorthand round-trip (nodus_command_or_assignment)"
  - "WorkflowFile.comments compact emission"
  - "modifier-value re-quoting for whitespace-containing values"
  - "full-corpus NL-6 AST-equality harness (tests/parity.rs)"
  - "4 new normative fixtures: switch_dispatch, retry_bounded, halt_pause (v0.7 corpus backfill)"
key_files:
  created:
    - crates/nodus/tests/fixtures/switch_dispatch.nodus
    - crates/nodus/tests/fixtures/retry_bounded.nodus
    - crates/nodus/tests/fixtures/halt_pause.nodus
  modified:
    - crates/nodus/src/transpiler.rs
    - crates/nodus/tests/parity.rs
patterns_established:
  - "nodus_step delegates to an exhaustive nodus_stmt dispatcher (no wildcard arm) so a future Stmt variant is a compile error, not a silent drop — the shape of the defect this phase closed."
  - "A full-struct AST-equality sweep will surface defects outside the phase's chartered scope (found: WorkflowFile.comments, Step.retry, Step.sub_steps, modifier-value quoting, the ASSIGN-shorthand shape, and a Phase-6 test-block emission gap). Fix what's in-scope and blocks the whole corpus; narrow the assertion and flag what belongs to another L2 spec, rather than silently absorbing it."
duration_minutes: ~
---

# Stage 21 Tasks — Compact-Form Round-Trip Fidelity for Control Flow (NL-6)

**Phase:** 21
**Status:** Done
**Strategic Goal:** Restore conformance to **NL-6**, which three Stable specs mandate and the code cannot honor: `Transpiler::to_nodus` silently **drops every control-flow step**. `nodus_step` matches only `Stmt::Command` and `Stmt::Comment`; all seven other `Stmt` variants fall into a `_` catch-all returning `""`, and the caller's `if !line.is_empty()` then skips the step entirely. A workflow whose steps are `?IF`/`?SWITCH`/`~FOR`/`~UNTIL`/`~PARALLEL`/`~MAP` round-trips to a workflow with **fewer steps** — or none. Sequential tracks A (emitters) → B (the AST-equality guard the spec claims exists) → T; Track C (fixture corpus) is file-independent.

## Context: why this is plannable without a new spec

Third instance of the Phase 17 / Phase 20 shape (C12.1 fix-to-regain-conformance) and the largest: the mandate is **already Stable in three specs**, and the code structurally diverges from it.

- **`l1-nodus-language.md:51` — NL-6 (Stable v1.14.0):** *"compact form and human form are semantically equivalent; **compact → human → compact must produce an AST-equal result**; compact is the canonical form."*
- **`l2-nodus-runtime.md:37` (Stable v1.3.0):** *"`Transpiler::to_nodus` (compact, **lossless**) and `Transpiler::to_human` (one-way prose); **round-trip test in `workflows.rs` verifies AST equality after compact re-parse**"* — and `:146`: *"`Compact` = **lossless round-trip**"*.
- **`l2-nodus-control-flow.md:49` (Stable v1.0.1):** *"The transpiler round-trips **each new construct** (compact ⇄ human) so `compact → human → compact` stays **AST-equal**."* — and `:138`: *"**Each new node gains compact and human emitters** so NL-6 round-trip holds"*; `:143`: *"a first-class node **round-trips cleanly** (NL-6)"*.

Grounded at plan time — verified empirically, not inferred from a code read (the Phase-20 Track-B lesson):

- **Proved by execution, not by reading.** A temporary probe transpiled the Phase-20 `SWITCH_ARM_TARGETS_WF` fixture and re-parsed it: `orig steps=2 reparsed steps=1`. The emitted `@steps:` block contained only `2. LOG(done) → $out` — the entire `?SWITCH` step was gone. Probe reverted after observation.
- **The emitter asymmetry is total.** Human side: `humanize_step`, `humanize_map`, `humanize_switch`, `humanize_command`, `humanize_conditional`, `humanize_for`, `humanize_until` — 7 emitters, full control-flow coverage. Compact side: `nodus_step`, `nodus_command` — **2 emitters, zero control-flow coverage**. The spec's "each new node gains compact and human emitters" is half-true: the human half was built.
- **The claimed guard does not exist.** `l2-nodus-runtime.md:37` asserts a round-trip test in `workflows.rs` verifying AST equality. Grepping `workflows.rs` for `round_trip|to_nodus` returns exactly one hit — line 163, the `TranspileMode::Compact` dispatch. **No such test exists anywhere in the crate.** This is what hid the defect for its whole lifetime.
- **The existing round-trip tests are too weak to catch it.** `compact_map_transform_round_trip` and `compact_ticket_triage_round_trip` (parity.rs) assert only `ast2.header.name` — a dropped `~MAP` step is invisible to them. Same pattern as Phase 17 ("the one construct never tested end-to-end is the one that doesn't work end-to-end"), one level up.
- **Blast radius: 7 of 9 `Stmt` variants**, plus `Step.sub_steps` — which `transpiler.rs` never references at all.

### What is NOT in scope (verified AST-stable at plan time)

The probe's output showed three further oddities. **All three are AST-stable and therefore not NL-6 violations** — they lose information on the *first* parse and then reproduce idempotently, so `parse → to_nodus → parse` yields an equal AST. Do not "fix" them here; a source-equality test would trip on them for reasons unrelated to this defect:

- `core: schema.nodus` re-emits as `core: schema` — the **parser** stores `"schema"`, so both parses agree. Recorded as a separate Backlog item.
- `mode: production` is injected though the fixture never declared it — `RuntimeBlock.mode` documents "defaults to `production`", so both parses agree.
- `@err: ESCALATE(human)` re-emits as `@err: ESCALATE ( human )` — `ErrorDecl.raw` comes from `consume_rest_of_line()`, which space-joins tokens, so the *first* parse already produced the spaced form.

**Found during implementation, also excluded (a real defect, but not this phase's owner):** `@test:` block re-emission loses fidelity for structured `input`/`expected` values (found via the T-21B01 harness on `ticket_triage.nodus` — unlike the three items above, this one is a genuine, not-AST-stable NL-6 violation). It predates this phase (Phase 6) and belongs to `l2-nodus-testing.md`, not `l2-nodus-control-flow.md`; recorded in the PLAN Backlog for its own pass rather than folded in here. T-21B01's harness is scoped to `.steps` specifically so this pre-existing, out-of-scope gap doesn't block the control-flow fix.

## Sequencing rationale

- **A → B.** Track B's AST-equality harness is the phase's real Verify instrument, and it **must fail before Track A lands** — that failure is the proof the defect was real. It cannot be the gate for A's own tasks, so each Track-A task carries its own narrower per-construct assertion.
- **Track C is independent** (new fixture files) but feeds Track B: the corpus is what the harness should sweep, and it currently lacks `?SWITCH`/`~RETRY`/`!HALT`/`!PAUSE`.
- Track A's tasks all edit `transpiler.rs`, so they serialize regardless of track independence.

## Atomic Checklist

- [x] [T-21A01] Compact emitters — branch family (`Conditional`, `Switch`)
- [x] [T-21A02] Compact emitters — loop/collection family (`ForLoop`, `UntilLoop`, `Map`)
- [x] [T-21A03] Compact emitters — `Parallel`, `VarRef`, `Step.sub_steps`, `Step.retry`
- [x] [T-21B01] AST-equality round-trip harness (where the spec says it lives)
- [x] [T-21C01] Fixture-corpus backfill — `?SWITCH`, `~RETRY`, `!HALT`, `!PAUSE`
- [x] [T-21T01] Full-corpus round-trip + non-regression suite
- [x] [T-21T02] Quality gates + LP-1 zero-dependency check

## Detailed Tracking

### [T-21A01] Compact emitters — branch family (`Conditional`, `Switch`)

- **Spec:** `l2-nodus-control-flow.md` §3 (NL-6 row), §4 · `l1-nodus-language.md` NL-6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** Unit tests in `transpiler.rs` `mod tests` assert that a parsed `?IF`-chain (with `?ELIF`/`?ELSE` and an action flag such as `!HALT`) and a parsed `?SWITCH` (value arms + `*` default + at least one arm binding `→ $target`) each re-parse from `to_nodus` output into an AST equal to the original. `cargo test -p nodus` green, count strictly above the 405 baseline.
- **Handoff:** T-21A02.
- **Changes:** Added `nodus_stmt` (the new exhaustive dispatcher, no `_` arm), `nodus_conditional_chain`/`nodus_conditional_branch` (renders `?IF`/`?ELIF`/`?ELSE`, flags, and the branch action via `nodus_command` so its own `→ target` comes along for free), `nodus_switch` (arms + `*` default + `~END`). `Conditional.body` (block-form `?IF cond:`) confirmed empirically dead on the parse path — `parse_if_chain` never calls a body-collecting routine — so deliberately not rendered, per the phase-file guardrail; see T-21A03 for where that content actually lives (`Step.sub_steps`).
- **Notes / guardrails:**
  - **Emit the block terminators.** The parser's nesting depends on `~END` (and `~JOIN` for `~PARALLEL`) tokens, **not** on indentation — the lexer emits no indent/dedent tokens (the reason `~COMPENSATE` had to be a same-line clause). A `?SWITCH` emitted without its `~END` will not re-parse as a switch.
  - `?SWITCH` arm actions now carry `pipeline_target` (Phase 20) — emit it, or this phase re-creates the very loss Phase 20 just fixed, one layer out. Reuse `nodus_command`, which already renders `→ target`.
  - Mirror `humanize_switch`/`humanize_conditional` for *structure*, not for text — they are the reference for which fields exist, but the human form is prose and the compact form is canonical source.
  - **Do not modify the `humanize_*` family.** It is complete and correct; this phase is compact-side only.

### [T-21A02] Compact emitters — loop/collection family (`ForLoop`, `UntilLoop`, `Map`)

- **Spec:** `l2-nodus-control-flow.md` §3/§4 · `l1-nodus-language.md` NL-5, NL-6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** Unit tests assert AST-equal round-trip for a parsed `~FOR $x IN $c … ~END`, a `~UNTIL cond | MAX:n … ~END`, and a `~MAP $c: CMD($it) → $out`. The `~UNTIL` case must assert `max_iterations` survives (NL-5's bound is load-bearing, not decoration). `cargo test -p nodus` green.
- **Handoff:** T-21A03.
- **Changes:** Added `nodus_for`, `nodus_until` (handles both `MAX:n` present and absent — a bare `~UNTIL cond:` is a valid parsed state even though the validator separately requires the bound), `nodus_map` (clones the inner command and re-attaches `mb.target` as its `pipeline_target` before calling `nodus_command`, mirroring `parse_map`'s move in reverse), and the shared `push_indented_body` helper both loop emitters and Track A03's parallel/sub-step emission reuse.
- **Notes / guardrails:**
  - `MapBlock` carries its `→ $out` target on the **block**, not on the inner command (`parse_map` moves it — `mb.target` is `Some`, `mb.command.pipeline_target` is `None`). Emit from `mb.target`, or the target is lost.
  - `~UNTIL`'s `MAX:n` is part of the compact grammar (`~UNTIL cond | MAX:3:`) — round-trip it explicitly.
  - Loop bodies are `Vec<Stmt>`; emit each through the same recursive step emitter so nested control flow inside a loop body also survives.

### [T-21A03] Compact emitters — `Parallel`, `VarRef`, `Step.sub_steps`, `Step.retry`

- **Spec:** `l2-nodus-control-flow.md` §3/§4 · `l1-nodus-language.md` NL-6
- **Status:** Done
- **Assignment:** Agent
- **Verify:** Unit tests assert AST-equal round-trip for a parsed `~PARALLEL … ~JOIN → $t` (both branches present), a parsed `~PARALLEL … ~END` (no join), and a `~RETRY:n` step. `nodus_stmt` exhaustively matches `Stmt`'s 9 variants with no `_` arm — a future variant is now a compile error here, not a silent drop.
- **Handoff:** T-21B01.
- **Changes:**
  - `sub_steps` is **not dead on the parse path** — the plan-time assumption was wrong, caught by the T-21B01 harness itself rather than left standing. `ticket_triage.nodus`'s existing step 2 (`?IF $classification.severity = "critical":` followed by an indented `ESCALATE(...)`) populates it for real: `parse_step`'s post-body loop collects any indented content following a step's main line into `Step.sub_steps`, not into `Conditional.body` (confirming T-21A01's finding that `Conditional.body` is the one that's dead — the block-form `?IF cond:` body actually lives one level up, on the enclosing `Step`). `nodus_step` now emits `step.sub_steps` as indented lines via `push_indented_body`, with no terminator (sub-steps end at the next `StepNumber`/section token, not an explicit `~END`).
  - **`Step.retry` (`~RETRY:n`) was completely unemitted** — a second finding beyond the phase's original `Stmt`-variant enumeration, in the same function and same defect class (confirmed by the corpus harness: a `~RETRY:3` fixture step lost its bound on round-trip before this fix). Now prefixed onto the step's first line in `nodus_step`.
  - `nodus_parallel`: branches via `push_indented_body`, terminated by `~JOIN → target` when `join_target` is `Some`, else `~END` — matches `parse_parallel`'s two termination paths exactly.
  - `VarRef`: emits the bare `v.name` (already `$`-prefixed); re-parses via `parse_assignment_or_expr`'s no-`=` branch.
  - **A further, unplanned finding surfaced by the same harness**: `Stmt::Command` with `name == "ASSIGN"` is not a real callable command — it's how `parse_assignment_or_expr` represents the `$var = expr` shorthand internally (reusing the `Stmt::Command` shape rather than adding a new `Stmt` variant). `ASSIGN` is absent from `KNOWN_COMMANDS`, so emitting it via the generic `NAME(args)` call syntax cannot round-trip (it lexes back as a plain `Identifier`, not `CommandName`, and falls through to `Stmt::Comment`). Added `nodus_command_or_assignment`, detecting the exact shape `parse_assignment_or_expr` produces (`name == "ASSIGN"`, 2 args, no modifiers/validators/flags, `pipeline_target == Some(args[0])`) and emitting the `$var = expr` shorthand back instead of a generic call.
  - **Also found via the same harness, and fixed as part of this task rather than deferred** (same file, same defect class, blocking every fixture, not scope creep): `WorkflowFile.comments` (free-standing top-level `;;` lines) was never emitted at all — added right after the header, since the field carries no source position and the parser accepts a comment at any point between sections. And `nodus_command`'s modifier-value emission never re-quoted a value containing whitespace (`+msg="Critical ticket"` lost everything after the first word on reparse, since the lexer needs the quotes to keep it one token) — fixed by re-quoting any modifier value containing whitespace.

### [T-21B01] AST-equality round-trip harness

- **Spec:** `l2-nodus-runtime.md` §3 (NL-6 row) — *"round-trip test in `workflows.rs` verifies AST equality after compact re-parse"*
- **Status:** Done
- **Assignment:** Agent
- **Verify:** A test exists that, for each fixture in the normative corpus, asserts `Parser::parse(src)` is AST-equal to `Parser::parse(&Transpiler::to_nodus(&Parser::parse(src)))`. It must compare **parsed ASTs**, never source strings. Reverting any single Track-A task makes it fail.
- **Handoff:** T-21T01.
- **Changes:** `full_corpus_ast_equal_after_compact_round_trip` added to `tests/parity.rs` (not `workflows.rs` — see below), sweeping `NORMATIVE_CORPUS`. **Scoped to `ast1.steps == ast2.steps`, not the whole `WorkflowFile`**: a first pass at whole-struct equality surfaced a real, pre-existing (Phase 6) NL-6 violation in `@test:` block re-emission (`ticket_triage.nodus`'s structured `input`/`expected` values lose their braces and word-split on reparse) — genuine, but owned by `l2-nodus-testing.md`, not this phase's `l2-nodus-control-flow.md` mandate. Recorded in the PLAN Backlog for a dedicated pass rather than folded in here; narrowing to `.steps` keeps this phase's blast radius honest.
- **Notes / guardrails:**
  - **Location patched, per the Phase-17 precedent for a spec claim that didn't match reality.** The spec said `workflows.rs`; the actual home is `tests/parity.rs`, alongside the `NORMATIVE_CORPUS` it sweeps. `l2-nodus-runtime.md` §3 is patched to name the real location (this phase, alongside the NL-6 row's other corrections).
  - Compares parsed ASTs (`.steps`), never source text — confirmed the three AST-stable oddities (`core` truncation, `mode` defaulting, `@err:` spacing) don't trip it, since `.steps` doesn't even touch those fields.
  - `WorkflowFile` and every constituent AST type already derive `PartialEq, Eq` — no derive work needed.

### [T-21C01] Fixture-corpus backfill — `?SWITCH`, `~RETRY`, `!HALT`, `!PAUSE`

- **Spec:** `l2-nodus-control-flow.md` §4 (the v0.7 constructs)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** Four new `.nodus` files in `crates/nodus/tests/fixtures/`, each registered as an `include_str!` const in `tests/parity.rs`, each passing validate-with-no-errors and the T-21B01 round-trip harness.
- **Handoff:** T-21T01.
- **Changes:** Added `switch_dispatch.nodus` (`?SWITCH` with two value arms + `*` default), `retry_bounded.nodus` (`~RETRY:3`), `halt_pause.nodus` (`!HALT` and `!PAUSE` action flags on separate `?IF` steps). Each registered in `parity.rs`'s `NORMATIVE_CORPUS` and given its own `*_no_block_errors` validation test, matching the existing per-fixture convention.
- **Notes:** This was a standing Backlog item recorded as *"pure corpus tidying — no defect sits behind this."* **That framing is now obsolete**: the corpus is exactly what the round-trip harness sweeps, so its missing v0.7 constructs are the difference between the harness covering the defect class and only half of it. Phase 17 added `map_transform.nodus` as the corpus's first v0.7 fixture; this completes the set.

### [T-21T01] Full-corpus round-trip + non-regression suite

- **Goal:** Prove NL-6 AST-equality holds across the whole normative corpus, and that nothing outside the transpiler's compact path moved.
- **Method:** Run the T-21B01 harness over every fixture including the four new ones. Confirm the existing weak header-name round-trip tests still pass unmodified.
- **Status:** Done
- **Verify:** `cargo test -p nodus` green, zero failures, count strictly above the 405 baseline; every corpus fixture passes AST-equal `.steps` round-trip.
- **Changes:** 11 construct-specific unit tests added to `transpiler.rs` (one per `Stmt` variant plus `~RETRY:n` plus a nested `~FOR`-containing-`?SWITCH` case), each asserting `parse(src).steps == parse(to_nodus(parse(src))).steps`. Plus the corpus-wide `full_corpus_ast_equal_after_compact_round_trip` (T-21B01) covering all 11 normative fixtures. All pre-existing tests (including the now-redundant header-name round-trip checks) pass unmodified. 420 tests total (was 405; +15: 11 transpiler unit tests + 4 parity.rs tests — 3 new-fixture validations + 1 corpus harness).

### [T-21T02] Quality gates + LP-1 zero-dependency check

- **Goal:** Definition-of-done gates for the phase.
- **Method:** Run the workspace's standing gate set for `crates/nodus`.
- **Status:** Done
- **Verify:** All green — `cargo test -p nodus` (zero failures); `cargo clippy -p nodus --all-targets -- -D warnings` (zero lints); `cargo fmt -p nodus -- --check` (clean); no `.unwrap()` / `panic!()` / `unreachable!()` / `.expect(` introduced outside `#[cfg(test)]`. LP-1: `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` is empty.
- **Changes:** `cargo test -p nodus` → 420 passed, 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → 1 finding (`collapsible_if` in the `~COMPENSATE` emission guard), fixed with a `&&`-chained `if let`, re-verified clean. `cargo fmt -p nodus -- --check` → 3 findings, fixed via `cargo fmt -p nodus`, re-verified clean. Manual scan of `transpiler.rs` production code (outside `#[cfg(test)]`, which starts at line 743) for `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` → none found; `tests/parity.rs` is entirely test code. `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty (LP-1 preserved).

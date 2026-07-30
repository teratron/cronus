---
phase: 21
name: "Compact-Form Round-Trip Fidelity for Control Flow (NL-6)"
status: Todo
subsystem: "crates/nodus"
requires: [11, 17, 20]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 21 Tasks — Compact-Form Round-Trip Fidelity for Control Flow (NL-6)

**Phase:** 21
**Status:** Todo
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

## Sequencing rationale

- **A → B.** Track B's AST-equality harness is the phase's real Verify instrument, and it **must fail before Track A lands** — that failure is the proof the defect was real. It cannot be the gate for A's own tasks, so each Track-A task carries its own narrower per-construct assertion.
- **Track C is independent** (new fixture files) but feeds Track B: the corpus is what the harness should sweep, and it currently lacks `?SWITCH`/`~RETRY`/`!HALT`/`!PAUSE`.
- Track A's tasks all edit `transpiler.rs`, so they serialize regardless of track independence.

## Atomic Checklist

- [ ] [T-21A01] Compact emitters — branch family (`Conditional`, `Switch`)
- [ ] [T-21A02] Compact emitters — loop/collection family (`ForLoop`, `UntilLoop`, `Map`)
- [ ] [T-21A03] Compact emitters — `Parallel`, `VarRef`, and `Step.sub_steps`
- [ ] [T-21B01] AST-equality round-trip harness (where the spec says it lives)
- [ ] [T-21C01] Fixture-corpus backfill — `?SWITCH`, `~RETRY`, `!HALT`, `!PAUSE`
- [ ] [T-21T01] Full-corpus round-trip + non-regression suite
- [ ] [T-21T02] Quality gates + LP-1 zero-dependency check

## Detailed Tracking

### [T-21A01] Compact emitters — branch family (`Conditional`, `Switch`)

- **Spec:** `l2-nodus-control-flow.md` §3 (NL-6 row), §4 · `l1-nodus-language.md` NL-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** Unit tests in `transpiler.rs` `mod tests` assert that a parsed `?IF`-chain (with `?ELIF`/`?ELSE` and an action flag such as `!HALT`) and a parsed `?SWITCH` (value arms + `*` default + at least one arm binding `→ $target`) each re-parse from `to_nodus` output into an AST equal to the original. `cargo test -p nodus` green, count strictly above the 405 baseline.
- **Handoff:** T-21A02.
- **Notes / guardrails:**
  - **Emit the block terminators.** The parser's nesting depends on `~END` (and `~JOIN` for `~PARALLEL`) tokens, **not** on indentation — the lexer emits no indent/dedent tokens (the reason `~COMPENSATE` had to be a same-line clause). A `?SWITCH` emitted without its `~END` will not re-parse as a switch.
  - `?SWITCH` arm actions now carry `pipeline_target` (Phase 20) — emit it, or this phase re-creates the very loss Phase 20 just fixed, one layer out. Reuse `nodus_command`, which already renders `→ target`.
  - Mirror `humanize_switch`/`humanize_conditional` for *structure*, not for text — they are the reference for which fields exist, but the human form is prose and the compact form is canonical source.
  - **Do not modify the `humanize_*` family.** It is complete and correct; this phase is compact-side only.

### [T-21A02] Compact emitters — loop/collection family (`ForLoop`, `UntilLoop`, `Map`)

- **Spec:** `l2-nodus-control-flow.md` §3/§4 · `l1-nodus-language.md` NL-5, NL-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** Unit tests assert AST-equal round-trip for a parsed `~FOR $x IN $c … ~END`, a `~UNTIL cond | MAX:n … ~END`, and a `~MAP $c: CMD($it) → $out`. The `~UNTIL` case must assert `max_iterations` survives (NL-5's bound is load-bearing, not decoration). `cargo test -p nodus` green.
- **Handoff:** T-21A03.
- **Notes / guardrails:**
  - `MapBlock` carries its `→ $out` target on the **block**, not on the inner command (`parse_map` moves it — `mb.target` is `Some`, `mb.command.pipeline_target` is `None`). Emit from `mb.target`, or the target is lost.
  - `~UNTIL`'s `MAX:n` is part of the compact grammar (`~UNTIL cond | MAX:3:`) — round-trip it explicitly.
  - Loop bodies are `Vec<Stmt>`; emit each through the same recursive step emitter so nested control flow inside a loop body also survives.

### [T-21A03] Compact emitters — `Parallel`, `VarRef`, and `Step.sub_steps`

- **Spec:** `l2-nodus-control-flow.md` §3/§4 · `l1-nodus-language.md` NL-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** Unit tests assert AST-equal round-trip for a parsed `~PARALLEL … ~JOIN → $t` (both branches present) and for a step carrying `sub_steps`. After this task, **no `Stmt` variant reaches the `_` catch-all** — assert that by exhaustively matching the enum rather than leaving a wildcard arm.
- **Handoff:** T-21B01.
- **Notes / guardrails:**
  - **Check whether `sub_steps` is ever populated by the parser before writing an emitter for it.** `transpiler.rs` never references the field today; if no parsed workflow produces sub-steps, emitting a shape the parser cannot read back would be worse than the omission. Establish this empirically (parse a fixture, inspect the AST) and record the finding either way — if it is genuinely dead on the parse path, say so and skip it deliberately rather than inventing syntax.
  - Replace the `_` catch-all in `nodus_step` with explicit arms so any future `Stmt` variant is a **compile error** here, not a silent drop. That structural change is the durable fix — it is what would have prevented this defect.
  - `VarRef` is a bare variable reference line; keep the emission minimal and verify it re-parses as the same variant (it may be simpler to confirm the parser never emits a top-level `VarRef` step, in which case record that and handle the arm explicitly anyway).

### [T-21B01] AST-equality round-trip harness

- **Spec:** `l2-nodus-runtime.md` §3 (NL-6 row) — *"round-trip test in `workflows.rs` verifies AST equality after compact re-parse"*
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** A test exists that, for each fixture in the normative corpus, asserts `Parser::parse(src)` is AST-equal to `Parser::parse(&Transpiler::to_nodus(&Parser::parse(src)))`. It must compare **parsed ASTs**, never source strings. Reverting any single Track-A task makes it fail.
- **Handoff:** T-21T01.
- **Notes / guardrails:**
  - **Put it where the spec says it is — `workflows.rs`.** The spec's claim is currently false; building the test there makes the spec true without editing it. If there is a real reason it belongs elsewhere (e.g. the corpus `include_str!`s live in `tests/parity.rs`), then **patch `l2-nodus-runtime.md` §3 deliberately to name the real location** and record why — the Phase-17 precedent, where the phase checked where the corpus actually lives before writing tests and synced the spec to reality rather than overclaiming.
  - Compare ASTs, not text. Three known source-level differences are AST-stable and expected (`core` truncation, `mode` defaulting, `@err:` raw spacing — see "What is NOT in scope"); a string-equality test fails on all three for reasons unrelated to NL-6.
  - `WorkflowFile` needs `PartialEq` for a direct `assert_eq!`. If it is not already derived, prefer deriving it over hand-writing a comparison — but check first whether any field would make derived equality wrong.

### [T-21C01] Fixture-corpus backfill — `?SWITCH`, `~RETRY`, `!HALT`, `!PAUSE`

- **Spec:** `l2-nodus-control-flow.md` §4 (the v0.7 constructs)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** Four new `.nodus` files in `crates/nodus/tests/fixtures/`, each registered as an `include_str!` const in `tests/parity.rs`, each passing validate-with-no-errors and the T-21B01 round-trip harness.
- **Handoff:** T-21T01.
- **Notes:** This was a standing Backlog item recorded as *"pure corpus tidying — no defect sits behind this."* **That framing is now obsolete**: the corpus is exactly what the round-trip harness sweeps, so its missing v0.7 constructs are the difference between the harness covering the defect class and only half of it. Phase 17 added `map_transform.nodus` as the corpus's first v0.7 fixture; this completes the set. Keep each fixture minimal and valid (a `?SWITCH` fixture needs an `@in` default to seed its scrutinee — see `tests/control_flow.rs`); a fixture that trips an unrelated lint adds noise, not coverage.

### [T-21T01] Full-corpus round-trip + non-regression suite

- **Goal:** Prove NL-6 AST-equality holds across the whole normative corpus, and that nothing outside the transpiler's compact path moved.
- **Method:** Run the T-21B01 harness over every fixture including the four new ones. Add a targeted assertion that the Phase-20 `?SWITCH` arm `pipeline_target` survives the round-trip specifically (the regression this phase is most likely to silently re-introduce). Confirm the existing weak header-name round-trip tests still pass unmodified — they are now redundant but should not break.
- **Status:** Todo
- **Verify:** `cargo test -p nodus` green, zero failures, count strictly above the 405 baseline; every corpus fixture passes AST-equal round-trip.

### [T-21T02] Quality gates + LP-1 zero-dependency check

- **Goal:** Definition-of-done gates for the phase.
- **Method:** Run the workspace's standing gate set for `crates/nodus`.
- **Status:** Todo
- **Verify:** All green — `cargo test -p nodus` (zero failures); `cargo clippy -p nodus --all-targets -- -D warnings` (zero lints); `cargo fmt -p nodus -- --check` (clean); no `.unwrap()` / `panic!()` / `unreachable!()` / `.expect(` introduced outside `#[cfg(test)]`. LP-1: `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` is empty.

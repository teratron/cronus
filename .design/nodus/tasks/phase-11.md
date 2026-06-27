---
phase: 11
name: "Control-Flow Constructs (l2-nodus-control-flow)"
status: Todo
subsystem: "crates/nodus"
requires:
  - phase-10
provides: []
key_files: []
patterns_established: []
duration_minutes: 0
---

# Phase 11 — Control-Flow Constructs (l2-nodus-control-flow)

Implement the v0.7 control constructs from `l1-nodus-language.md` §4.6 in
`crates/nodus`, per `l2-nodus-control-flow.md`. Built as per-construct vertical
slices so each track leaves the crate green. Reuses the existing
`Status::Failed`/`Status::Paused`/`Signal::Pause` and the `SWITCH_NO_MATCH`/
`PAUSED` codes.

**Specs covered**: `l2-nodus-control-flow.md` (Stable).

## Track A — `!HALT` / `!PAUSE` action flags (Slice 1)

The smallest slice — action flags mirroring `!BREAK`, reusing `Signal`/`Status`.

- [ ] **T-11A01** — Lexer: tokenize `!HALT` and `!PAUSE` (new `BangHalt`/`BangPause` token types, matched like `!BREAK`)
  - **Verify**: lexer unit test — `!HALT` / `!PAUSE` produce the new tokens
- [ ] **T-11A02** — AST + parser: add `halt_flag` / `pause_flag` to `Conditional` (alongside `break_flag`); parse them on a conditional action
  - **Verify**: parser unit test — `?IF … → CMD !HALT` sets `halt_flag`; `!PAUSE` sets `pause_flag`
- [ ] **T-11A03** — Executor: `!HALT` → `Status::Failed` (push error, stop); `!PAUSE` → `Signal::Pause` → `Status::Paused` + `ResumeDescriptor`
  - **Verify**: integration test — a workflow whose taken branch carries `!HALT` ends `Status::Failed`; one with `!PAUSE` ends `Status::Paused` with a resume descriptor and no later step run
- [ ] **T-11A04** — Validator + transpiler: `!HALT` without an `ESCALATE()` in the same step → error; round-trip `!HALT`/`!PAUSE` in compact + human forms
  - **Verify**: validator unit test (halt-without-escalate error); transpiler round-trip test

## Track B — `?SWITCH` multi-branch dispatch (Slice 2)

- [ ] **T-11B01** — Lexer/AST/parser: `?SWITCH $v:` with `value → action` arms + optional `* → action` (`QSwitch` token, `SwitchBlock` node)
  - **Verify**: parser unit test — arms and default parsed into `SwitchBlock`
- [ ] **T-11B02** — Executor + validator + transpiler: first-match-wins dispatch; no match + no `*` → `NODUS:SWITCH_NO_MATCH` (warn, continue); empty-arms warning; round-trip
  - **Verify**: integration test — matching arm runs; unmatched-with-no-default surfaces `SWITCH_NO_MATCH` and continues

## Track C — `~MAP` collection transform (Slice 3)

- [ ] **T-11C01** — Lexer/AST/parser: `~MAP $coll: CMD($it) → $out` (`TildeMap` token before the generic `Flag` rule; `MapBlock` node; implicit `$it`)
  - **Verify**: parser unit test — `MapBlock` populated; `~MAP` no longer mis-lexes as `Flag`
- [ ] **T-11C02** — Executor + transpiler: map over a `Value::List` binding `$it`, collect into a list; empty/non-list ⇒ empty list; round-trip
  - **Verify**: integration test — `~MAP` over a 3-element list yields a 3-element result; empty collection yields `[]`

## Track D — `~RETRY:n` bounded step retry (Slice 4)

- [ ] **T-11D01** — Lexer/AST/parser: `~RETRY:n` step modifier (`TildeRetry` carrying `:n`, parsed like `MAX:n`)
  - **Verify**: parser unit test — retry bound captured on the step
- [ ] **T-11D02** — Executor + validator: re-run the step up to `n` on runtime error; on exhaustion route to `@err`; validator rejects missing `:n` or `n > 10` (NL-5)
  - **Verify**: integration test — a flaky step succeeds within `n` retries; validator unit test rejects `~RETRY` without a bound

## Track T — Gates

- [ ] **T-11T01** — Quality gates after each landed slice
  - **Verify**: `cargo test -p nodus` full suite green; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt` clean; `cargo doc --no-deps` no new warnings; SDD §6 clean

## Status

**Status:** Todo

## Notes

Slices are independent vertical features (A→B→C→D), each landing lexer + AST +
parser + executor + transpiler + validator + tests and leaving the crate green —
so the phase may complete incrementally across sessions. Risk concentrates in the
parser/lexer additions (new tokens must precede the generic `~`-flag / `!`-flag
rules). `Status::Paused`/`Signal::Pause` (Phase 10) and `SWITCH_NO_MATCH`/`PAUSED`
(Phase 8) already exist; this phase wires them to syntax.

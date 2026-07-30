---
phase: 22
name: "Whole-File NL-6 Round-Trip Closure"
status: Todo
subsystem: "crates/nodus"
requires: [6, 21]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 22 Tasks — Whole-File NL-6 Round-Trip Closure

**Phase:** 22
**Status:** Todo
**Strategic Goal:** Close the last three `WorkflowFile` fields that do not survive a compact round-trip, so NL-6's AST-equality mandate holds for the **whole file** rather than for `.steps` alone. Phase 21 closed eleven of fourteen fields; the corpus harness it built was deliberately scoped to `.steps` because the remainder were out of that phase's charter. This phase removes that scoping — widening the harness assertion from `.steps` to the entire `WorkflowFile` is the phase's single decisive acceptance signal (`l2-nodus-testing.md` §10.5). Sequential A → B → T.

## Context: the three remaining fields

`to_nodus` emission status per `WorkflowFile` field, established by direct inspection at plan time:

| Field | Round-trips? |
| --- | --- |
| `header`, `runtime`, `triggers`, `rules`, `preferences`, `input_decl`, `output_decl`, `context_decl`, `error_decl` | ✅ pre-existing |
| `steps`, `comments` | ✅ Phase 21 |
| **`tests`** | ⚠️ **partial** — emits, but not AST-equally (specified: `l2-nodus-testing.md` §10) |
| **`macros`** | ❌ **never emitted** — `transpiler.rs` contains zero references to `macros`/`MacroBlock` |
| **`human_mode`** | ❌ **never emitted** — zero references |

All three are violations of the same already-Stable mandate — `l1-nodus-language.md` NL-6, *"compact → human → compact must produce an AST-equal result"* — so all three are C12.1 fix-to-regain-conformance work needing **no new spec**. `tests` additionally has its Rust shape specified this cycle in `l2-nodus-testing.md` §10 (v1.2.0); `macros` and `human_mode` need no separate spec because NL-6 covers the whole AST and both reuse mechanisms this phase already builds.

**Why `macros`/`human_mode` are in scope rather than deferred:** the stated acceptance signal is whole-`WorkflowFile` equality. `macro_expand.nodus` is in the normative corpus and carries an `@macro: greet` block, so leaving `macros` unemitted makes that signal unreachable no matter how well `tests` is fixed — the phase would end unable to demonstrate its own goal. This is the Phase-21 precedent for `WorkflowFile.comments` and modifier-value quoting: both were outside the original enumeration but blocked every corpus fixture, so they landed with the work they blocked.

## Sequencing rationale

- **A → B → T.** Tracks A and B are logically independent (different emitter regions) but edit the same function region in `transpiler.rs`, so they serialize regardless.
- **Track T is the proof, not a formality.** Widening the harness is what demonstrates the goal; it cannot pass until A and B both land, and it should be attempted only after them.
- Track A's two tasks split emitter from validator: the `W015` diagnostic (A02) is independent of the emission fix (A01) and could land either order, but A01 first keeps the round-trip work contiguous.

## Atomic Checklist

- [ ] [T-22A01] `@test:` emission — invert branch to prefer `raw_lines`, add value re-quoting
- [ ] [T-22A02] `W015` — diagnose non-conforming `input:`/`expected:` pair separators
- [ ] [T-22B01] `@macro:` block emission
- [ ] [T-22B02] `human_mode` block emission
- [ ] [T-22T01] Widen the corpus harness to the whole `WorkflowFile`
- [ ] [T-22T02] Quality gates + LP-1 zero-dependency check

## Detailed Tracking

### [T-22A01] `@test:` emission — prefer `raw_lines`, re-quote values

- **Spec:** `l2-nodus-testing.md` §10.4 (both parts) · `l1-nodus-language.md` NL-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** A unit test asserts `parse(src).tests == parse(to_nodus(parse(src))).tests` for (a) a canonical line-per-pair body, (b) an inline-brace body, and (c) a body whose values contain whitespace and a token-splitting character (`"When is my invoice due?"`, `"T-001"` — the two shapes the Phase-21 harness observed corrupting). `cargo test -p nodus` green, count above the 420 baseline.
- **Handoff:** T-22A02.
- **Notes / guardrails:**
  - **The fix to §10.4(a) is a branch-order inversion, not a new mechanism.** `to_nodus` already has a `raw_lines` emission path; it is guarded by `if has_structured { … } else if !raw_lines.is_empty() { … }`, and since the structured fields are *derived from* `raw_lines` they are non-empty whenever it is — so that path is unreachable for every parsed block. Invert to prefer `raw_lines` when non-empty; keep structured emission as the fallback for programmatically-constructed `TestBlock`s.
  - **Emitting from `raw_lines` produces a single flat body line — that is correct, do not "improve" it.** `collect_braced_raw_lines` drops `Newline` tokens, so line structure is not recoverable. A multi-line re-render would change `raw_lines` on re-parse and break the very equality this task establishes. Readability is not the goal; the compact form is machine-canonical.
  - **Spec wording flagged, not blocking:** §10.4(a) says `raw_lines` "reproduces the body in the form the author wrote it". Strictly it reproduces the *token sequence*, not the line breaks (see above). The AST-equality conclusion is unaffected. Flag for a patch-level correction on the next `/magic.spec` touch of that file — the Phase-18 `$restart` precedent — rather than HALTing or silently implementing against inaccurate text.
  - **Quote values, never structural tokens.** `raw_lines` holds token *values* with no type information, so the emitter cannot distinguish a separator from a value that happens to look like one. Skip the structural set (`:`, `{`, `}`, `[`, `]`, `,`) and re-quote only the remaining elements whose bare form would not re-lex to a single token of equal value. A value whose literal text *is* a structural token is a known ambiguity the current parser also cannot represent — out of scope, do not attempt to solve it here.

### [T-22A02] `W015` — non-conforming pair separator

- **Spec:** `l2-nodus-testing.md` §7, §10.3 · `l1-nodus-testing.md` NT-9
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `Validator::validate` emits `W015` for a `@test:` block containing `expected: { status = SUCCESS }`, and does **not** emit it for a conforming `expected: { status: SUCCESS }`. A test asserts `ticket_triage.nodus` — the corpus fixture whose assertion has silently never executed — now reports `W015`.
- **Handoff:** T-22B01.
- **Notes / guardrails:**
  - **Warning severity, not error** (§7). An error would convert files that parse today into hard validate-before-run failures; the intent is to surface silently-ignored assertions, not to break the corpus.
  - Expect `W015` and `W009` to co-fire on a block whose only `expected:` pairs are all non-conforming — the section exists in source but is empty in the AST. §7 records that pairing as the intended signal, not a duplicate report; do not suppress either.
  - This does **not** make the dropped assertion execute. Teaching `parse_test_body` to accept `=` would be a grammar change, and `l1-nodus-testing.md` §4.1 admits only `:` — so the correct behavior is to keep rejecting it and say so. Fixing `ticket_triage.nodus`'s fixture text is likewise out of scope here (it would mask the diagnostic this task adds); flag it instead.

### [T-22B01] `@macro:` block emission

- **Spec:** `l1-nodus-language.md` NL-6 (whole-AST round-trip)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `parse(src).macros == parse(to_nodus(parse(src))).macros` for `macro_expand.nodus`; before this task that assertion fails with `macros: []` on the right-hand side.
- **Handoff:** T-22B02.
- **Notes / guardrails:**
  - `MacroBlock { name, raw_lines }` is structurally the same shape as the `TestBlock` raw-lines case, and `parse_macro_block` uses the **same** `collect_braced_raw_lines` helper — so T-22A01's emission and quoting rule applies directly. Reuse it rather than writing a second, divergent renderer.
  - Emit in the `@macro:{name} { … }` form the parser reads back; verify against `parse_macro_block` rather than assuming the header shape matches `@test:`'s.

### [T-22B02] `human_mode` block emission

- **Spec:** `l1-nodus-language.md` NL-6 (whole-AST round-trip)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `parse(src).human_mode == parse(to_nodus(parse(src))).human_mode` for a fixture carrying a `;; HUMAN MODE` block (`ticket_triage.nodus` has one); before this task the right-hand side is `None`.
- **Handoff:** T-22T01.
- **Notes / guardrails:**
  - **Emission order is load-bearing: emit `human_mode` last.** The parser routes a Comment token containing `HUMAN MODE` into `collect_comment_block()`, which then **greedily consumes every following Comment token**. Any free-standing `;;` comment emitted after the HUMAN MODE block is therefore absorbed into `human_mode` on re-parse — silently corrupting both it and `comments`, the field Phase 21 just fixed. Emitting it last avoids the interaction entirely.
  - `human_mode` holds the comment lines already `;;`-prefixed and `\n`-joined (`collect_comment_block` joins raw token values). Emit verbatim; do not re-prefix.

### [T-22T01] Widen the corpus harness to the whole `WorkflowFile`

- **Goal:** Demonstrate the phase's actual goal — NL-6 AST-equality for the entire file, not one field.
- **Method:** Change `full_corpus_ast_equal_after_compact_round_trip` (`tests/parity.rs`) from asserting `ast1.steps == ast2.steps` to `ast1 == ast2`, and update its doc comment — which currently explains the `.steps` scoping as a deliberate accommodation of exactly the gaps this phase closes.
- **Status:** Todo
- **Verify:** The widened assertion passes for all 11 normative corpus fixtures. Reverting any of T-22A01 / T-22B01 / T-22B02 individually makes it fail — that per-task sensitivity is what proves each one is load-bearing rather than incidental.
- **Notes:** If a field outside this phase's three turns out to also diverge, **do not silently narrow the assertion again**. Establish which field, whether it is a real NL-6 violation or an AST-stable artifact (the `core`/`mode`/`@err:` class Phase 21 characterized), and either fix it or record it explicitly — a second silent narrowing would hide the next gap the same way the first one hid these.

### [T-22T02] Quality gates + LP-1 zero-dependency check

- **Goal:** Definition-of-done gates for the phase.
- **Method:** Run the workspace's standing gate set for `crates/nodus`.
- **Status:** Todo
- **Verify:** All green — `cargo test -p nodus` (zero failures, count above the 420 baseline); `cargo clippy -p nodus --all-targets -- -D warnings` (zero lints); `cargo fmt -p nodus -- --check` (clean); no `.unwrap()` / `panic!()` / `unreachable!()` / `.expect(` introduced outside `#[cfg(test)]`. LP-1: `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` is empty.

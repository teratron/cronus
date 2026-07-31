---
phase: 25
name: "Effect Risk-Class Declaration"
status: Done
subsystem: "crates/nodus/src/executor.rs"
requires: []
provides:
  - "LP-11 context literal in execute_command carries optional reversible/external/value descriptors, read off cmd.modifiers when present (LP-16)"
  - "l2-nodus-portability 1.6.1: LP-16 fully reconciled to as-built"
key_files:
  created: []
  modified:
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/tests/portability.rs"
    - ".design/nodus/specifications/l2-nodus-portability.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "A consequence-descriptor seam can ride an existing generic decoration surface (+modifier=value) instead of new DSL grammar — check for a reusable existing surface before assuming a new one is needed"
  - "Omit-not-default for optional context keys: absence in a Value::Map, never a sentinel (Null/false/\"none\"), keeps 'undeclared' distinguishable from 'declared false' for the host to interpret"
duration_minutes: ~
---

# Stage 25 Tasks — Effect Risk-Class Declaration

**Phase:** 25
**Status:** Done
**Strategic Goal:** Extend the LP-11 gate's `context` with optional `reversible`/`external`/`value` consequence descriptors, exactly as `l2-nodus-portability` §4.10 specifies — no new DSL grammar, no new AST field, no `PolicyProvider` signature change.

## Scope note (read before starting)

`l2-nodus-portability` §4.10 (v1.6.0) is the authoritative design for this phase — every
task below cites the subsection it realizes. The descriptors ride the *existing*
`+modifier=value` grammar (`CommandCall.modifiers: Vec<(String, String)>`, already parsed,
transpiled, and NL-6 round-trip-safe since Phase 21) — there is no lexer, parser, AST, or
transpiler work in this phase. If implementation surfaces a gap the spec pass missed,
record it in Track C rather than silently improvising a different shape.

**Explicitly out of scope**, per §4.10.5:

- Closed-vocabulary validation of `value=` against `none | money | creds | perms` (an
  advisory `W`-code). No modifier name has a registry today (`vocab.rs` has no
  `KNOWN_MODIFIERS`); adding one for just these three would be an unmotivated asymmetry.
- LP-17 and LP-20. Each specializes the LP-11 gate differently (a `settlement` shape; an
  `obligation`/`discharge` shape) and needs its own core vocabulary before either has
  anything to carry through the gate — do not generalize this phase's mechanism to them.
- Any change to `PolicyProvider`'s trait signature, `evaluate`'s parameters, or a new
  `Tier`/`Friction` type. The host's own `evaluate` implementation decides tier-then-friction
  internally over the richer `context` — nodus adds no new vocabulary for that decision.

## Guardrails (from the §4.10 design)

1. **Real call site, not the spec's illustrative `build_context` helper.** The as-built
   `context` in `executor.rs::execute_command` is an inline `Value::Map(vec![...])` literal
   (around the existing `effect_class_of` gate, after `check_rules`, before the
   `ASK`/`CONFIRM` dispatch) — §4.9.2/§4.10.2's `[REFERENCE]` pseudocode names a separate
   `build_context` function for clarity, but there is no such function in the crate. Extend
   the real literal in place; do not introduce a new function unless refactoring the
   existing literal into one is the natural minimal-diff shape once you're looking at it.
2. **Omit, don't default.** A descriptor absent from `cmd.modifiers` must be *absent* from
   the resulting `context` map — never inserted as `Value::Null`/`Value::Bool(false)`/
   `Value::Text("none")`. An "unclassified" effect is silence in `context`; the host's own
   `evaluate` body applies L1 §4.12's "unclassified → most cautious" policy, not nodus.
3. **Modifier keys carry their `+` prefix.** `CommandCall.modifiers` stores keys as
   `"+reversible"`, `"+external"`, `"+value"` (confirmed in `ast.rs`'s own test:
   `modifiers: vec![("+tone".to_string(), "warm".to_string())]`) — match on the prefixed
   form; strip it only when inserting into `context` (§4.10.2's pseudocode already shows
   this via `key.trim_start_matches('+')`).
4. **No new `Value` kind, no new error code.** Descriptor values pass through as
   `Value::Text` (raw modifier strings, same as `args`). Denial still emits the existing
   `NODUS:POLICY_DENIED` — this phase adds no new failure mode, only a richer `context`.
5. **Scoped to the same gate, no wider.** Only commands where `effect_class_of(&cmd.name)`
   is `Some` ever reach this code path — a step decorating a non-gated command (e.g.
   `LOG +external=true`) is inert by construction; do not add a second check or a new gate
   for it.

## Atomic Checklist

- [x] [T-25A01] Extend the LP-11 `context` literal with `reversible`/`external`/`value`
- [x] [T-25C01] Reconcile `l2-nodus-portability` to the as-built result
- [x] [T-25T01] Descriptor visibility, absence, and no-op-regression coverage

## Detailed Tracking

### [T-25A01] Extend the LP-11 `context` literal with `reversible`/`external`/`value`

- **Spec:** l2-nodus-portability.md §4.10.1, §4.10.2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` clean; the Track T test confirms that a `GEN` step
  carrying `+reversible=true +external=true +value=money` reaches a test `PolicyProvider`
  whose `evaluate` observes a `context` map containing `("reversible", Value::Text("true"))`,
  `("external", Value::Text("true"))`, and `("value", Value::Text("money"))` alongside the
  existing `command`/`args` keys; an undecorated `GEN` step reaches `evaluate` with a
  `context` containing only `command`/`args` (Guardrail 2).
  **Satisfied**: `risk_descriptors_reach_context_when_declared` and
  `undeclared_risk_descriptors_are_absent_from_context_not_defaulted` (T-25T01) both pass;
  `cargo check -p nodus` clean.
- **Handoff:** T-25C01 reconciles the spec once this is real; T-25T01 adds the full coverage
  matrix.
- **Changes:** `execute_command`'s `context` construction (`executor.rs`) changed from a
  fixed two-key `Value::Map(vec![...])` literal to a mutable `context_pairs` vec that gains
  a loop over `["+reversible", "+external", "+value"]`, pushing the stripped-prefix key and
  `Value::Text(value.clone())` only when `cmd.modifiers` contains that key. No new type, no
  new field, no new error code.
- **Notes:** In `execute_command`, extend the existing gate's `context` construction —

  ```
  let context = Value::Map(vec![
      ("command".to_string(), Value::Text(cmd.name.clone())),
      ("args".to_string(), Value::List(cmd.args.iter().cloned().map(Value::Text).collect())),
      // + new: for each of "+reversible", "+external", "+value", if cmd.modifiers
      //   contains that key, push (key-without-'+', Value::Text(value.clone()))
  ]);
  ```

  A small loop over `["+reversible", "+external", "+value"]` finding each in
  `cmd.modifiers` (a `Vec<(String, String)>`, so `.iter().find(|(k, _)| k == key)`) and
  pushing the stripped-prefix pair only when found is the shape §4.10.2 specifies — follow
  it rather than inventing a different lookup mechanism. This is the only production-code
  change in this phase; it touches one function in one file.

### [T-25C01] Reconcile `l2-nodus-portability` to the as-built result

- **Spec:** l2-nodus-portability.md §4.10 (all subsections), §3.1 (LP-16 row), §5 item 7
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `node .magic/scripts/executor.js check-prerequisites --json --require-specs --verify-headers --workspace=nodus`
  reports no `VERSION_DRIFT` and the file header matches its `INDEX.md` row; `grep -n "not
  yet implemented" .design/nodus/specifications/l2-nodus-portability.md` no longer matches
  the LP-16 row.
  **Satisfied**: pre-flight `ok: true`, only the expected post-bump `SYNC_GAP`; the LP-16 row
  (§3.1) and §5 item 7 no longer say "not yet implemented" / "task-authorable" (checked with
  the same grep after the edit — zero remaining hits outside Document History, which is
  correctly append-only).
- **Handoff:** Track T.
- **Changes:** `l2-nodus-portability` 1.6.0 → 1.6.1 — §4.10 header, §4.10.2's `[REFERENCE]`
  pseudocode (corrected to the real inline literal — no separate `build_context` function
  exists, matching §4.9.3's own precedent of showing the as-built call site), §3.1's LP-16
  row, its Leverage paragraph's invariant count (two now Implemented: LP-11, LP-16), and §5
  item 7 all updated to Implemented/Done. `INDEX.md` row + top-level version synced
  (1.0.73 → 1.0.74).
- **Notes:** Patch or minor bump depending on how much §4.10's `[REFERENCE]` pseudocode
  diverges from the real, compiled shape (rename this task's own scope note if the actual
  loop/lookup shape differs from what §4.10.2 sketched — spec pseudocode is not binding on
  exact identifier names, but should be corrected to match once real code exists, following
  the Phase-17/20/23/24 Track-C precedent). Update: §3.1's LP-16 row (from "Designed, not
  yet implemented" to **Implemented**), its Leverage paragraph's invariant count, and §5
  item 7 (drop from the "Order of implementation" list or mark done, matching how Phase 24
  closed out LP-11's item 2 in the same section). Self-review before declaring this task
  done — the LP-11/LP-15 reconciliations found live stale references the first pass missed
  each time; check `l2-nodus-portability.md` for any remaining "not yet implemented" /
  "task-authorable" language beyond the sections listed above before finishing.

### [T-25T01] Descriptor visibility, absence, and no-op-regression coverage

- **Spec:** l2-nodus-portability.md §4.10.2, §4.10.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase covering: (a) a
  `GEN` step decorated with `+reversible=true +external=true +value=money` run through
  `run_with_policy` against a `PolicyProvider` test double that captures the `context` it
  receives — assert all three keys present with the exact expected `Value::Text` values;
  (b) an undecorated `GEN` step run the same way — assert `context` contains no
  `reversible`/`external`/`value` key at all (not `Value::Null`, not `Value::Bool(false)` —
  genuinely absent from the `Value::Map`'s pairs); (c) a plain `run`/`run_with_provider` call
  (no policy) with a decorated step behaves byte-for-byte as before this phase — the
  modifiers are inert without a `PolicyProvider` consulting them (Guardrail 5's shape,
  mirrored from Phase 24's own regression precedent).
  **Satisfied**: 447 tests pass (was 444, +3) — `risk_descriptors_reach_context_when_declared`,
  `undeclared_risk_descriptors_are_absent_from_context_not_defaulted`,
  `risk_descriptors_are_inert_without_a_policy_provider`, all passing on first run (no
  empirical surprise this phase, unlike Phase 24's reserved-variable-seeding catch).
  `cargo clippy -p nodus --all-targets -- -D warnings` clean; `cargo fmt -p nodus -- --check`
  found one line-wrap violation (no logic change), fixed; `git diff --stat` on
  `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved); no `unwrap`/`expect`/`panic!` added to
  `executor.rs`'s production path.
- **Handoff:** none — last task in the phase.
- **Changes:** Added `RISK_DECORATED_WF` fixture (a `GEN` step carrying all three
  descriptors), `CapturingPolicy` test double (`Arc<Mutex<Option<Value>>>`-backed, records
  the last `context` it was asked to evaluate, always permits), and 3 integration tests to
  `tests/portability.rs`, placed beside `AllowAllPolicy`/`DenyAllPolicy` under the existing
  LP-11 section marker.
- **Notes:** Add a new fixture (neither `MANIFEST_WF` nor `DEFERRED_WF` in
  `tests/portability.rs` carries any `+reversible`/`+external`/`+value` modifier — checked
  directly rather than assumed reusable) with a `GEN` step carrying all three modifiers. A
  capturing test double needs interior mutability to record what `evaluate` observed (e.g.
  `RefCell<Option<Value>>` or `Cell`-based capture of the `context` argument, since
  `PolicyProvider::evaluate` takes `&self`) — follow whatever capture idiom already exists
  in the crate's test doubles if one does; otherwise a `RefCell` is the zero-dependency
  default (LP-1). Place the new fixture and test double beside `AllowAllPolicy`/
  `DenyAllPolicy` under the existing `// ─── LP-11 per-effect authorization gate ───`
  section marker, not a new one — this is the same gate, richer input.

---
phase: 23
name: "Built-in Durable-State Conformance"
status: Todo
subsystem: "crates/nodus/src/portability.rs"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 23 Tasks — Built-in Durable-State Conformance

**Phase:** 23
**Status:** Todo
**Strategic Goal:** Make the shipped `StorageProvider` built-in satisfy its own L1 contract — an in-memory store that round-trips within an invocation — without touching the executor wiring that LP-3 still gates.

## Scope note (read before starting)

Two Stable L1 statements mandate this change:

- `l1-nodus-portability` §4.1, Storage row (amended v1.8.0 by LP-15): the built-in is an
  **in-memory** store, "non-durable; state discarded between invocations".
- **LP-2**: "The library ships exactly one built-in implementation per interface,
  **sufficient for in-process testing without I/O**."

The crate ships `NoopStorageProvider`: `store` discards, `load` always returns `None`. It
cannot round-trip even within one invocation, so it does not satisfy either statement. The
L1's distinction is deliberate — a discarding **audit** built-in is fine (audit is
write-only, `NoopAuditProvider`), but a discarding **store** breaks the read-back that makes
a store a store.

**Explicitly out of scope — executor wiring.** `store`/`load` have zero call sites and this
phase adds none. Wiring is LP-3-gated by three statements in `l2-nodus-portability` (§3's
LP-3 row, §2's Constraints, §4.4's heading) plus a standing PLAN Backlog line. LP-3 needs a
`/magic.spec` amendment documenting two independent host-usage contexts — governance, not
code. This phase changes only the built-in's internal behaviour, which §2's deferral
("hook points, run-parameter variants") does not cover.

## Guardrails (from plan-time grounding)

1. **The existing Phase-5 test encodes the old contract on purpose.**
   `tests/portability.rs::noop_storage_and_policy_compile` (T-5T02) asserts
   `storage.load("key").is_none()` with a comment calling it the LP-2 no-op. It must be
   **updated deliberately** as part of this change, not discovered as a mystery failure.
2. **No `unwrap()` on the lock.** `&self` methods need interior mutability; `std::sync::Mutex`
   is the zero-dep choice, and `lock()` returns a `Result`. The project forbids `unwrap()` /
   `panic!()` on production paths — recover the poisoned guard
   (`unwrap_or_else(|poisoned| poisoned.into_inner())`) so a poisoned lock degrades rather
   than aborts.
3. **Exactly one built-in per interface (LP-2).** Do **not** add an in-memory provider
   *alongside* the no-op. This is a replacement.
4. **Zero new dependencies (LP-1).** `std` only.
5. **Renaming is a public-API break.** The crate is `0.2.0` (pre-1.0) and the
   `cargo semver-checks` gate lands at 1.0.0 (`l2-nodus-portability` §5), so the rename is
   acceptable now, but record it — `NoopStorageProvider` is re-exported from `lib.rs`.

## Atomic Checklist

- [ ] [T-23A01] Replace `NoopStorageProvider` with an in-memory built-in
- [ ] [T-23A02] Update re-exports and the stale module/registry documentation
- [ ] [T-23C01] Reconcile `l2-nodus-portability` to the as-built built-in
- [~] [T-23C02] ~~Correct `l2-nodus-portability` §5 item 2~~ — **Cancelled (superseded)**, see below
- [ ] [T-23T01] Update the Phase-5 contract test and add round-trip coverage
- [ ] [T-23T02] Run the full gate set and confirm zero-dep

## Detailed Tracking

### [T-23A01] Replace `NoopStorageProvider` with an in-memory built-in

- **Spec:** l1-nodus-portability.md §4.1 (Storage row) / §4.11 · LP-2 · l2-nodus-portability.md §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` — a new test stores a `Value` under a key and reads the
  equal `Value` back via `load`; `cargo tree -p nodus` (or an unchanged
  `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock`) shows no new dependency.
- **Handoff:** T-23A02 fixes the re-export and documentation surface the rename breaks.
- **Notes:** Name it `InMemoryStorageProvider` — the crate's convention is
  `<Behaviour><Role>Provider` (`NoopAuditProvider`, `BuiltinSchemaProvider`,
  `DefaultConfigProvider`), while L1 §4.11's `[REFERENCE]` names the concept `InMemoryStore`;
  this honours the L1's intent in the crate's idiom. Storage: `std::sync::Mutex` around a
  `Vec<(String, Value)>` or `BTreeMap<String, Value>` — `Value` is `Clone` (verified,
  `executor.rs:32`) so `load` returns an owned clone. Guardrail 2 applies to every `lock()`.
  `store` on an existing key overwrites. Keep the trait signature unchanged
  (`store(&self, key: &str, value: &Value)` / `load(&self, key: &str) -> Option<Value>`) —
  the L1 §4.11 `put`/`get`/`delete` naming is a separate L1↔L2 divergence, **not** in scope,
  and changing it here would be a second breaking change for no conformance gain.

### [T-23A02] Update re-exports and the stale module/registry documentation

- **Spec:** l2-nodus-portability.md §4.2 · LP-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo doc -p nodus --no-deps` both clean; `grep -rn "NoopStorageProvider" crates/nodus/src crates/nodus/tests` returns no stale references.
- **Handoff:** Track C records the same change spec-side.
- **Notes:** `lib.rs` re-exports `NoopStorageProvider` by name — update it. `portability.rs`'s
  module doc says storage/policy are "pending LP-3 graduation"; that stays true of the
  **wiring** but is now misleading about the built-in, so scope the sentence to wiring rather
  than deleting it. Two comments elsewhere reference the no-op as precedent
  (`environment.rs:164`, `executor.rs:597`) — those describe the *interface-declared,
  wiring-pending* pattern, which is still accurate; leave them unless they name the type.

### [T-23C01] Reconcile `l2-nodus-portability` to the as-built built-in

- **Spec:** l2-nodus-portability.md §3.1 (LP-15 row), §4.2, §4.3, Overview, §5 item 3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `grep -n "NoopStorageProvider" .design/nodus/specifications/l2-nodus-portability.md`
  returns only historical Document History rows; `node .magic/scripts/executor.js check-prerequisites --json --require-specs --verify-headers --workspace=nodus`
  reports no `VERSION_DRIFT` and the file header matches its `INDEX.md` row.
- **Handoff:** Track T. (T-23C02, previously the follow-on in this file, is Cancelled — superseded.)
- **Notes:** Patch-level bump (1.2.0 → 1.2.1) with an `INDEX.md` row sync — the behaviour
  now matches what the L1 already mandated, so this records reality rather than changing a
  contract. Update: §4.2's Built-in cell, §4.3's body and heading, §3.1's LP-15 row (the
  divergence half is resolved; the **zero-call-sites** half is not and must remain), the
  Overview sentence, and §5 item 3, which currently splits LP-15 into "built-in divergence"
  and "executor hook points" — only the first is done. Reconciling spec-to-as-built inside a
  run phase follows the Phase-17 and Phase-20 Track-C precedent.

### [T-23C02] ~~Correct `l2-nodus-portability` §5 item 2~~ — Cancelled (superseded)

- **Spec:** l2-nodus-portability.md §5 item 2 · §3 (LP-3 row) · §2 · §4.4
- **Status:** **Cancelled (superseded)** — do not execute.
- **Assignment:** —
- **Verify:** n/a. The condition this task specified is already met in
  `l2-nodus-portability` 1.3.0: §5 item 2 no longer lists the LP-11 call site as
  authorable without a further spec pass, and §5's closing sentence was rewritten to
  separate the three states it had collapsed.
- **Handoff:** Track C completes at T-23C01; hand to Track T.
- **Notes:** Superseded by the `l2-nodus-portability` 1.3.0 spec pass, which corrected
  §5 item 2 **from the opposite direction to the one planned here**. This task assumed the
  only available fix was to demote the claim — rewrite the call site as LP-3-gated and state
  what would open the gate. The spec pass instead *opened* the gate: `l1-nodus-portability`
  1.14.0 §4.14 made LP-3 falsifiable, and §4.8.1 recorded it **satisfied** for
  `PolicyProvider` on the strength of two divergent host decision shapes (the runtime tool
  guard and plugin-hook interception) whose divergence the host's own `l1-interception-model`
  was written to unify.
  Recorded **Cancelled rather than Done** deliberately: the outcome this task described was
  reached, but not by this phase, and marking it Done would credit Phase 23 with work another
  cycle performed. The substantive correction it aimed at survives and is now more accurate —
  the LP-11 call site is not *ungated*, it is **permitted but not yet designed**, which is an
  ordinary L2 gap rather than a governance block. That design work is recorded in the
  Backlog as the next spec target, including the real L1↔L2 contradiction it must settle
  (L2 §4.4's skip-with-`ConstraintHit` versus L1 §4.7's typed `NODUS:POLICY_DENIED`).

### [T-23T01] Update the Phase-5 contract test and add round-trip coverage

- **Spec:** l1-nodus-portability.md §4.1 · LP-2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` passes with a net test-count increase; the updated
  `noop_storage_and_policy_compile` no longer asserts `load(...).is_none()` for a
  previously-stored key.
- **Handoff:** T-23T02 runs the full gate set.
- **Notes:** Update the existing test per Guardrail 1 (rename it too — "noop" is no longer
  what it exercises for storage; it still covers `NoopPolicyProvider` and
  `BuiltinSchemaProvider`, so consider splitting the storage half into its own test). New
  coverage: (a) store → load returns the equal value; (b) storing twice on one key overwrites;
  (c) `load` on an absent key still returns `None`; (d) two separate provider instances do not
  share state. Assert (d) explicitly — it is the property that makes the built-in safe for
  in-process testing, which is LP-2's stated purpose for it.

### [T-23T02] Run the full gate set and confirm zero-dep

- **Goal:** Verify the phase against spec and the project's mandatory gates.
- **Method:** `cargo test -p nodus`; `cargo clippy -p nodus --all-targets -- -D warnings`;
  `cargo fmt -p nodus -- --check`; `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock`
  empty (LP-1); manual scan confirming no `unwrap()`/`panic!()`/`expect(` added outside
  `#[cfg(test)]` (Guardrail 2). Run cargo via PowerShell, not Git Bash.
- **Status:** Todo

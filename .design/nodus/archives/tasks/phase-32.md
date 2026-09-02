---
phase: 32
name: "NE-12/HO-20 Digest Unification"
status: Done
subsystem: "crates/nodus/src/executor.rs, crates/nodus/src/environment.rs, crates/nodus/tests/environment.rs"
requires: []
provides:
  - "digest_ast widened to pub(crate) (executor.rs) for cross-module reuse"
  - "EnvRunResult::candidate() (NE-12) hashes the parsed AST via digest_ast, falling back to digest_source only on an unparseable source"
  - "integration test: CandidateResult.workflow_digest agrees with the independently-built ReproRecipe.workflow_digest for the same run (tests/environment.rs, ManifestCapture)"
  - "unit test: the fallback path returns digest_source's value explicitly, pinning a case the pre-existing test left unasserted"
  - "l2-nodus-environment.md 1.2.0 -> 1.3.0 / l2-nodus-portability.md 1.11.0 -> 1.11.1: digest-collision finding closed end to end (designed and built)"
key_files:
  created: []
  modified:
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/src/environment.rs"
    - "crates/nodus/tests/environment.rs"
    - ".design/nodus/specifications/l2-nodus-environment.md"
    - ".design/nodus/specifications/l2-nodus-portability.md"
    - ".design/nodus/INDEX.md"
patterns_established:
  - "A cross-struct identity claim (two fields, two call sites, same expected value) is proven by an integration test capturing both independently, not by trusting that reusing one function guarantees agreement — the second instance of this pattern in as many phases (Phase 31's ResumeDescriptor/ReproRecipe cross-check, now CandidateResult/ReproRecipe)"
duration_minutes: 15
---

# Stage 32 Tasks — NE-12/HO-20 Digest Unification

**Phase:** 32
**Status:** Done
**Strategic Goal:** `CandidateResult.workflow_digest` (`digest_source`, raw-text hash) and
`ReproRecipe.workflow_digest` (`digest_ast`, AST hash) answered different questions under the
identical field name — the naming collision the v1.42.0 replan found and recorded but did not
fix. `l2-nodus-environment.md` §4.4 v1.2.0 designs the resolution: `EnvRunResult::candidate()`
now hashes the parsed AST via the same `digest_ast` function `ReproRecipe` already uses,
falling back to the old `digest_source` byte hash only if the given source fails to parse.
Smallest phase since Phase 25/31 — one call site, unchanged signature, no new mechanism.

## Scope note (read before starting)

`l2-nodus-environment.md` §4.4 is the design source and settles every question needed:
which function to reuse (`digest_ast`, not a new one), why AST-level is the correct grain
(content identity — a cosmetic edit should not read as a different candidate), and what
happens on the one edge case (fall back, do not panic or change the return type). Two facts
ground the implementation, checked at planning time:

**1. Exactly one production call site, with an unchanged signature.** `EnvRunResult::candidate()`
(`environment.rs`) is never called from inside the crate's own execution paths — it is a
host-facing archival convenience the host calls *after* a run, supplying the same
`workflow_source: &str` it originally ran. Grepped every `.candidate(` call in the crate:
one production definition, one unit test (`environment.rs`, passes the non-workflow string
`"source"` — exercises the fallback path, asserts nothing about the digest value), two
integration tests (`tests/environment.rs`, both pass the real `ENV_WF` fixture or a
`str::replace` variant of it — both parse successfully, both exercise the primary path, and
neither assertion depends on which algorithm produced the value: one checks determinism
(`c1 == c2` for repeated calls) and content-sensitivity (`c1 != c3` for a renamed workflow,
which differs in the AST too, not only the bytes), the other checks `token_measure` only).
**No test breaks from the algorithm change.**

**2. `digest_ast` needs widening, not moving or duplicating.** It is `fn digest_ast(ast: &WorkflowFile) -> String`,
private to `executor.rs`, already used once (for `ReproRecipe.workflow_digest`). Widen to
`pub(crate) fn digest_ast` and `use crate::executor::digest_ast;` from `environment.rs` — no
physical move needed, Rust visibility alone is sufficient across sibling modules in one crate.
`digest_source` stays exactly where it is, unchanged, as the fallback.

**What must not happen.** No signature change to `candidate()` (`workflow_source: &str` stays
`&str` — the host does not hold a parsed AST and the crate returns none from any `run*` entry
point). No new failure mode — a parse failure inside `candidate()` degrades to the old
behaviour, it does not propagate an error or panic. No touching `l2-nodus-observability.md`'s
own `ReproRecipe`/`digest_ast` — that side is already correct and unchanged.

## Atomic Checklist

- [x] [T-32A01] Widen `digest_ast` to `pub(crate)`; `candidate()` hashes the parsed AST with a `digest_source` fallback
- [x] [T-32C01] Spec reconciliation — digest-collision finding closed in both carriers
- [x] [T-32T01] Cross-check + fallback coverage: `CandidateResult`/`ReproRecipe` agree on a real run; fallback still returns the old value on an unparseable source

## Detailed Tracking

### [T-32A01] Widen `digest_ast` to `pub(crate)`; `candidate()` hashes the parsed AST with a `digest_source` fallback

- **Spec:** l2-nodus-environment.md §4.4 (the unification design)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus` and `cargo clippy -p nodus --all-targets -- -D warnings`
  clean. `digest_ast` (`executor.rs`) is `pub(crate)`. `EnvRunResult::candidate()`
  (`environment.rs`) parses `workflow_source` via `Parser::parse`; on `Ok(ast)` it hashes
  `digest_ast(&ast)`, on `Err(_)` it falls back to `digest_source(workflow_source)` exactly as
  before. Signature unchanged.
  **Satisfied**: both commands clean on the first pass; no other call site required changes,
  confirming the Scope note's grep-based prediction.
- **Handoff:** T-32T01 is the acceptance evidence; T-32C01 reconciles the spec once this lands.
- **Notes:** Used fully-qualified paths (`crate::parser::Parser::parse`,
  `crate::executor::digest_ast`) at the call site instead of adding `use` imports — smaller
  diff, equally correct across sibling modules in one crate.
- **Changes:** `executor.rs`: `fn digest_ast` → `pub(crate) fn digest_ast`, doc comment
  extended to name the new cross-module reuse. `environment.rs`:
  `EnvRunResult::candidate()`'s `workflow_digest` field changed from a direct
  `digest_source(workflow_source)` call to a `match Parser::parse(workflow_source) { Ok(ast)
  => digest_ast(&ast), Err(_) => digest_source(workflow_source) }`.

### [T-32C01] Spec reconciliation — digest-collision finding closed in both carriers

- **Spec:** l2-nodus-environment.md §4.4, l2-nodus-portability.md §3.2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `l2-nodus-environment.md`'s NE-12 row and §4.4 design paragraph updated from a
  design-time statement to an implemented one (`[IMPLEMENTED v1.2.0, Phase 32]` tag, new
  "Implemented [Phase 32]" closing paragraph citing both tests). `l2-nodus-portability.md`
  §3.2's "Finding resolved" paragraph's cross-reference updated from the design version
  (`l2-nodus-environment.md` v1.2.0) to the implemented one (v1.3.0). `check-prerequisites
  --verify-headers --workspace=nodus` reports no `VERSION_DRIFT`, and **both** touched files'
  `INDEX.md` cells were updated together this time (the Phase-31→32 spec pass missed one of
  three cells and had to fix it on a second pre-flight run — checked twice here before
  proceeding).
  **Satisfied**: no divergence from the design — implementation matched v1.2.0 exactly,
  so both reconciliations are patch-to-minor cross-reference updates, not mechanism
  corrections.
- **Handoff:** Closes the phase's spec-sync obligation.
- **Notes:** No divergence found, unlike Phase 29's NE-14. `l2-nodus-portability.md`'s
  document-history rows for v1.9.0/v1.11.0 (which describe the digest finding as "not fixed
  here" / "RESOLVED" at the time each was written) were left as historical point-in-time
  records, per this workspace's convention — only the *live* prose (§3.2's body text) was
  updated to reflect the current, implemented state.
- **Changes:** `l2-nodus-environment.md` 1.2.0 → 1.3.0 (NE-12 row tag, new "Implemented"
  paragraph, Document History row). `l2-nodus-portability.md` 1.11.0 → 1.11.1 (§3.2
  cross-reference updated to v1.3.0, Document History row). `INDEX.md`: both version cells +
  top-level version 1.0.96 → 1.0.97 + Meta Information entry.

### [T-32T01] Cross-check + fallback coverage: `CandidateResult`/`ReproRecipe` agree on a real run; fallback still returns the old value on an unparseable source

- **Spec:** l2-nodus-environment.md §4.4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` green with at least two new assertions. **Cross-check**
  (`tests/environment.rs`, reusing the `ENV_WF` fixture and `run_with_environment_and_audit`):
  for one run, `result.candidate(ENV_WF, ..).workflow_digest` equals the
  `ReproRecipe.workflow_digest` captured from the same run's `run_complete` manifest.
  **Fallback** (unit test in `environment.rs`'s existing `#[cfg(test)] mod tests`):
  `candidate("<unparseable>", ..)` produces `workflow_digest == digest_source("<unparseable>")`.
  **Satisfied**: `candidate_digest_agrees_with_repro_recipe_digest` (integration,
  `tests/environment.rs`, new `ManifestCapture` `AuditProvider` mirroring Phase 31's
  `tests/dialog.rs` precedent exactly) and
  `candidate_digest_falls_back_to_source_hash_when_unparseable` (unit, `environment.rs`) both
  pass. `cargo test -p nodus`: **486 passed, 0 failed** (was 484, +2); `cargo clippy -p nodus
  --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean; `git diff --stat`
  on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved).
- **Handoff:** Phase acceptance signal.
- **Notes:** `run_with_environment_and_audit` (`workflows.rs:814`) exists and was used directly
  — confirmed at planning time, no discovery needed during execution. One transient toolchain
  hiccup during test execution (`cargo test` briefly failed to resolve `std`/`core` in one
  shell invocation) resolved itself on retry — an environment artifact unrelated to the code
  change, not a real compile error (confirmed clean on the very next invocation and every one
  after).
- **Changes:** `tests/environment.rs`: 2 new imports (`AuditProvider`, `ExecutionEvent`,
  `RunManifest`), `ManifestCapture` helper, 1 new integration test
  (`candidate_digest_agrees_with_repro_recipe_digest`). `environment.rs` test module: 1 new
  unit test (`candidate_digest_falls_back_to_source_hash_when_unparseable`).

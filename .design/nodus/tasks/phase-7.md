---
phase: 7
name: "Capability Manifest (LP-8)"
status: Todo
subsystem: "crates/nodus"
requires:
  - phase-5
provides: []
key_files: []
patterns_established: []
duration_minutes: 0
---

# Phase 7 — Capability Manifest (LP-8)

Implement the LP-8 capability manifest and pre-run satisfiability validation in
`crates/nodus`. A workflow declares the extension-point roles, host commands, and named
capabilities it requires; the runtime validates that declaration against the active host
before the first step executes and rejects fail-fast (never starting a partially-capable
run) with the missing-capability set named. The same manifest is the machine-checkable
two-host portability contract (LP-3). Completing this phase restabilizes the L2 spec.

**Specs covered**: `l2-nodus-portability.md` (RFC → Stable on completion); contract from
`l1-nodus-portability.md` §4.6.

**Restabilization (C12.1)**: `l2-nodus-portability` is currently RFC because LP-8 is
specified but unimplemented. This phase exists to drive it back to Stable — it bypasses
C12 quarantine under the Stabilization Exception.

## Track A — Manifest & Host Capability Surface

Pure additive types in `portability.rs`. No behavior change to existing runs.

- [ ] **T-7A01** — Define `CapabilityManifest` + `ExtensionRole` in `portability.rs`
  - `ExtensionRole` enum: `Model`, `Audit`, `Storage`, `Policy`, `Vocabulary` (the §4.2 taxonomy)
  - `CapabilityManifest { roles: BTreeSet<ExtensionRole>, commands: BTreeSet<String>, capabilities: BTreeSet<String> }` mirroring the L1 §4.6 manifest shape; deterministic ordering for diagnostics
  - Provide an empty/`Default` manifest that any built-in host satisfies (manifest-free runs stay runnable)
  - **Verify**: `cargo test -p nodus` — unit test `manifest_default_is_empty` in `portability.rs #[cfg(test)]` asserts the default manifest carries no roles/commands/capabilities; `cargo check -p nodus` passes

- [ ] **T-7A02** — Define the `HostCapabilities` resolution surface
  - Aggregates what the active host provides: `provides(role: ExtensionRole) -> bool`, `has_command(cmd: &str) -> bool` (delegates to the active `Schema`), `satisfies(cap: &str) -> bool`
  - Construct from the wired providers (Model/Audit/Storage/Policy presence) plus the active `Schema` — no concrete host type named (LP-1)
  - Built-in in-process configuration reports `Model` available (StubProvider) and the builtin schema commands
  - **Verify**: unit test `host_caps_reports_wired_roles` in `portability.rs #[cfg(test)]` — a host with only the model provider reports `provides(Model)=true`, `provides(Storage)=false`

## Track B — Pre-Run Validation Gate

The fail-fast gate and its wiring into the executor boot sequence. Highest-risk track.

- [ ] **T-7B01** — Implement `validate_manifest(manifest, host) -> Vec<Missing>`
  - Port the L1 §4.6 algorithm: collect every unsatisfied role, command, and capability into a `missing` list (order-stable); empty list ⇒ satisfiable
  - `Missing` is a typed enum (`Role(ExtensionRole)` / `Command(String)` / `Capability(String)`) so diagnostics name the exact unmet item
  - Pure function, no side effects, no I/O
  - **Verify**: unit tests `validate_manifest_satisfiable_empty` and `validate_manifest_reports_exact_missing` in `portability.rs #[cfg(test)]` — a manifest requiring `Storage` against a storage-less host yields exactly `[Missing::Role(Storage)]`

- [ ] **T-7B02** — Wire the fail-fast gate into executor boot + expose manifest-aware API
  - Run `validate_manifest` as a new step **before** the first workflow step in the executor boot sequence; on non-empty `missing`, reject before any step runs — no variable mutation, no execution events
  - Add a `NODUS:*` capability-rejection diagnostic in `error.rs` carrying the missing-capability set (extends the existing taxonomy; reuse an existing code only if one fits)
  - Add `run_with_manifest` (and a `_and_audit` combinator consistent with the existing orthogonal `run_with_*` family — LP-5) to `workflows.rs`; re-export from `lib.rs`
  - **Verify**: integration test in `tests/portability.rs` — a manifest requiring `Storage` run against a host with no storage provider returns the rejection error, the missing set names `Storage`, and `RunResult` shows zero executed steps

## Track C — Manifest Sourcing

How a workflow's required roles are determined for validation. Depends on Track A types.

- [ ] **T-7C01** — Derive a workflow's required-role manifest from its AST
  - Map invoked command kinds to `ExtensionRole` (e.g., a model-invoking command ⇒ `Model`) by walking the parsed step list; pure-computation workflows yield the empty manifest the built-in host satisfies
  - Scope guard: manifest is **inferred from the AST** in this phase; explicit DSL declaration (an `@needs` section) is deferred to the upstream-parity backlog — do not add grammar here
  - **Verify**: unit tests `manifest_from_model_workflow_requires_model` and `manifest_from_pure_workflow_is_empty` in `portability.rs #[cfg(test)]`

## Track T — Validation

Integration tests proving the LP-8 + LP-3 contract, then quality gates.

- [ ] **T-7T01** — LP-3 two-host substitution integration test
  - Host A (provides every role the manifest needs) runs the workflow to completion; Host B (missing one capability) is rejected fail-fast with that capability named
  - Encodes the LP-3 reduction: portability ⇔ "does host B satisfy the same manifest host A satisfied?"
  - **Verify**: `cargo test -p nodus --test portability -- manifest_lp3_two_host_substitution` passes

- [ ] **T-7T02** — Pre-run fail-fast purity test
  - A rejected run emits **no** `ExecutionEvent` and mutates **no** variable — rejection precedes all side effects (ties to the observability observer-neutrality boundary)
  - **Verify**: `cargo test -p nodus --test portability -- manifest_rejects_before_side_effects` passes — audit sink recorded zero events on the rejected run

- [ ] **T-7T03** — Quality gates
  - `cargo clippy -p nodus --all-targets -- -D warnings` — zero lints
  - `cargo fmt -p nodus` — format clean
  - `cargo test -p nodus` — full suite green (existing 204 + new Phase 7 tests)
  - `cargo doc -p nodus --no-deps` — no new doc warnings beyond the pre-existing `test`-fn baseline
  - **Verify**: all four commands exit 0; new-warning count is zero

## Track D — Spec Sync & Restabilize

Document the realized Rust shape and return the L2 spec to Stable.

- [ ] **T-7D01** — Author `l2-nodus-portability.md` §4.7 and restabilize
  - Add §4.7 "Capability Manifest (Rust)": `CapabilityManifest`, `ExtensionRole`, `HostCapabilities`, `validate_manifest`, the boot-sequence hook point, and the rejection diagnostic — `[REFERENCE]` contracts only, no active code
  - Flip the §3 `LP-8` compliance row from Pending → Implemented with its enforcement mechanism
  - Version `1.0.0 → 1.1.0` (minor — new section); `Status: RFC → Stable`; append Document History row; sync `.design/nodus/INDEX.md` (status + version)
  - **Verify**: `node .magic/scripts/executor.js check-prerequisites --json --require-specs --verify-headers --workspace=nodus` reports `ok: true` with no `VERSION_DRIFT` and no remaining RFC for this file; INDEX entry reads `Stable | 2 | 1.1.0`

## Status

**Status:** Todo

## Notes

Execution order is **A → {B, C} → T → D**, not fully parallel: `CapabilityManifest`
(Track A) is the shared type both the gate (Track B) and the sourcing (Track C) depend on,
so Track A must pass `cargo check` before B and C start. Track T runs after all
implementation tracks; Track D restabilizes the spec only once LP-8 is genuinely
implemented (the RFC → Stable flip cannot precede a passing Track T). Risk concentrates in
T-7B02 (boot-sequence integration + error taxonomy extension) — if it slips, T01/T02 block
but A/C/D-design can still progress.

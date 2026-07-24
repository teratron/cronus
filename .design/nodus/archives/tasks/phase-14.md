---
phase: 14
name: "Run-Manifest Identity & Reproducibility"
status: Done
subsystem: "crates/nodus"
requires: [4, 8]
provides:
  - "ExecutionMode/SimFidelity/Determinism/ReproRecipe/FaultIdentity types + RunManifest.{execution_mode,exposure_switches,repro}"
  - "step_identity(step_number, command_name) — definition-derived, stable cross-run identity on StepStart/StepEnd/StepError"
  - "StepError.fault_identity — message-independent grouping input"
  - "nodus-computed ReproRecipe (workflow_digest via AST Debug hash, nodus_version, determinism stated from model-call presence)"
  - "Executor::execute_with_manifest_context — host-declared execution_mode/exposure_switches entry point"
key_files:
  created: []
  modified:
    - "crates/nodus/src/observability.rs"
    - "crates/nodus/src/executor.rs"
    - "crates/nodus/src/lib.rs"
    - "crates/nodus/tests/observability.rs"
patterns_established:
  - "step_identity(u32, &str) over step_identity(&Step) — derive identity from what emission call sites actually carry, not the spec draft's AST-reference shape"
  - "Digest the parsed AST's Debug representation when the raw source string is architecturally unavailable at the point of use (execute_inner never sees source — only workflows.rs does)"
  - "Extend a private, narrowly-called inner fn (execute_inner) with new parameters rather than changing widely-used public signatures; add one new public entry point for the host-declaring path"
duration_minutes: ~
---

# Stage 14 Tasks — Run-Manifest Identity & Reproducibility

**Phase:** 14
**Status:** Done
**Strategic Goal:** Realize the manifest/identity observability cluster (`l2-nodus-observability.md` §4.7 / HO-12, HO-15, HO-18, HO-19, HO-20) in `crates/nodus` — enrich `RunManifest` and `StepError` into a cross-run-comparable, arm-partitionable, re-executable record. All-additive and optional: a run declaring none is byte-for-byte v1.1.0 (HO-5/HO-6 preserved). Zero new dependency (LP-1). Sequential tracks A→B→C→T. The per-event-descriptor batch (HO-7…HO-11/13/14/16/17) is out of scope (deferred by the spec).

## Atomic Checklist

- [x] [T-14A01] Manifest types & fields — `ExecutionMode` / `ReproRecipe` / `RunManifest.{execution_mode, exposure_switches, repro}`
- [x] [T-14A02] Event types & fields — `FaultIdentity` + `step_identity` / `fault_identity` on events
- [x] [T-14B01] `step_identity(&Step)` derivation + emission wiring (HO-15)
- [x] [T-14B02] `StepError.fault_identity` population, message-independent (HO-19)
- [x] [T-14C01] nodus-computed `repro` population (digest / version / determinism) + host-field defaults (HO-20)
- [x] [T-14C02] Host-supplied `execution_mode` / `exposure_switches` entry path (HO-12/HO-18)
- [x] [T-14T01] Validation suite — cross-run identity, message-independence, repro honesty, neutrality, zero-dep

## Detailed Tracking

### [T-14A01] Manifest types & fields

- **Spec:** l2-nodus-observability.md §4.7 (HO-12, HO-18, HO-20)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --lib observability::` → 228 passed (part of the full lib suite); `ExecutionMode`/`SimFidelity`/`Determinism`/`ReproRecipe` construct and derive `Default`; both existing `RunManifest` test literals updated with the 3 new fields.
- **Handoff:** the field surface T-14C01/C02 populate.
- **Notes:** Added to `observability.rs`: `ExecutionMode { Real, Simulated { fidelity: SimFidelity } }` (`#[derive(Default)]` + `#[default] Real`, per clippy's `derivable_impls` — a manual `impl Default` was rejected) + `SimFidelity { Structural, Modeled, Shadow }`; `ReproRecipe { workflow_digest, capability_set, exposure_switches, execution_mode, nodus_version, needs_vocabulary: Option<Vec<String>>, determinism }` (derives `Default`) + `Determinism { Deterministic, ContainsModelCalls }` (`#[default] Deterministic`). Added `execution_mode`, `exposure_switches: Vec<(String, String)>`, `repro: ReproRecipe` to `RunManifest`.

### [T-14A02] Event types & fields

- **Spec:** l2-nodus-observability.md §4.7 (HO-15, HO-19)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo check -p nodus --all-targets` → clean; every construction site across `observability.rs` (2), `executor.rs` (5) updated with the new fields.
- **Handoff:** the fields T-14B01/B02 derive and populate.
- **Notes:** Added `FaultIdentity { step_identity: String, code: String, discriminator: Option<String> }` (derives `PartialEq, Eq`) to `observability.rs`. Added `step_identity: String` to `StepStart`/`StepEnd`/`StepError`; added `fault_identity: FaultIdentity` to `StepError`. Confirmed via grep exactly one `StepError` emission site exists in the whole executor (the rule-violation path) — the churn was 2 test literals in `observability.rs` + 5 production sites in `executor.rs`, all updated; `FieldDescriptor`/`EnvInteraction` untouched.

### [T-14B01] `step_identity` derivation + emission (HO-15)

- **Spec:** l2-nodus-observability.md §4.7 (HO-15)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability step_identity` → 2 passed (`step_identity_is_stable_across_runs`: two runs of the same source produce identical `step_identity` on every `StepStart`/`StepEnd`; `step_identity_differs_for_a_different_command`: a different command at the same position yields a different identity).
- **Handoff:** feeds T-14B02 (`fault_identity` embeds it).
- **Notes:** **[DR]** Implemented `step_identity(step_number: u32, command_name: &str) -> String` (`"{n}:{cmd}"`) rather than the draft's `step_identity(&Step)` — the executor's dispatch call sites (`execute_command`/`handle_dialog`) carry `step_num: u32` + `cmd: &CommandCall`, never a `&Step` reference, so deriving identity from the two definition pieces actually in scope avoids threading an AST reference through the hot path for no semantic gain; the contract ("derived from the definition, not per-run allocated") is identical either way. Homed in `observability.rs` (re-exported at crate root); wired onto all 5 `StepStart`/`StepEnd`/`StepError` emission sites in `executor.rs`.

### [T-14B02] `fault_identity` population (HO-19)

- **Spec:** l2-nodus-observability.md §4.7 (HO-19)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability fault_identity_is_message_independent` → passed. Two workflows with `!!NEVER: FETCH` vs. `!!NEVER: Fetch` (differing rule-text casing) both violate the same step; `error_detail` genuinely differs (asserted `assert_ne!` as a test-validity guard) while `fault_identity` is identical.
- **Handoff:** completes the error-side identity; feeds T-14T01.
- **Notes:** Populated `StepError.fault_identity` at the single rule-violation emission site from `step_identity` + the constant `NODUS:RULE_VIOLATION` code + `discriminator: None`. **Scope note:** no `.nodus` grammar declares a discriminator today (no such syntax exists in the DSL) — the carrier field exists ahead of the declaring syntax, an honest gap rather than a silent one, mirroring HO-20's `needs_vocabulary: None` pattern.

### [T-14C01] nodus-computed `repro` population (HO-20)

- **Spec:** l2-nodus-observability.md §4.7 (HO-20)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability repro_` → 4 passed: `repro_determinism_reflects_model_calls` (GEN-workflow → `ContainsModelCalls`; LOG-only workflow → `Deterministic`), `repro_needs_vocabulary_is_none_and_version_matches_crate`, `repro_workflow_digest_is_deterministic_and_distinguishing` (same source twice → same digest; different source → different digest).
- **Handoff:** the recipe surface; host-supplied parts land in T-14C02.
- **Notes:** **[DR]** `workflow_digest` hashes the parsed `WorkflowFile`'s `Debug` representation via `DefaultHasher`, not the raw source string — `execute_inner` receives only the AST (parsing happens earlier in `workflows.rs`; the raw source is discarded before the executor ever sees it), so hashing source text was not an option without threading it through every `execute_inner` call site. Two sources differing only in whitespace/comments now share a digest, which is arguably the *more* correct reproducibility notion (neither affects execution). `determinism` computed from whether `ctx.log` contains a `GEN`/`ANALYZE` entry (reused existing infra, no new `ExecutionContext` field). `capability_set: Vec::new()` on the plain `execute()`/`execute_with_params()` path — `Executor` holds no `CapabilityManifest` context at all (that gate lives entirely in `workflows.rs::run_with_manifest`); an empty set honestly reports "none was checked."

### [T-14C02] Host-supplied `execution_mode` / `exposure_switches` (HO-12/HO-18)

- **Spec:** l2-nodus-observability.md §4.7 (HO-12, HO-18)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus --test observability execution_mode_and_exposure_switches_round_trip` + `default_execution_context_is_real_with_no_switches` → both passed. A caller declaring `Simulated{Structural}` + a switch sees both reflected in the manifest and mirrored into `repro`; a caller going through `run_with_audit` (declaring nothing) gets `Real` + `[]`.
- **Handoff:** completes §4.7 realization; feeds T-14T01.
- **Notes:** **[DR] resolved:** chose (per the two options drafted) a middle path narrower than both — `execute_inner` (private, only 3 in-file callers) gained the two parameters directly, so `execute()`/`execute_with_params()` keep their exact public signatures (forwarding `ExecutionMode::default()`/`vec![]`); ONE new public method `Executor::execute_with_manifest_context(ast, input, run_id, started_at, execution_mode, exposure_switches)` is the host-declaring entry point. No `workflows.rs`-level `run_with_*` wrapper was added — out of scope for this phase's Verify criteria; a future pass can add one following the `run_with_manifest` precedent if a host needs it at that layer.

### [T-14T01] Validation Task — manifest-cluster contract suite

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-observability.md` §4.7 (HO-12/15/18/19/20) and confirm LP-1 zero-dep + HO-5 observer-neutrality.
- **Method:** Extended `tests/observability.rs` with 8 new integration tests (T-14T01 section): HO-15 cross-run step-identity stability + differs-by-definition, HO-19 message-independence, HO-12/HO-18 host-declared round-trip + default-is-Real-and-empty, HO-20 determinism-reflects-model-calls + needs_vocabulary-None-version-matches + digest-deterministic-and-distinguishing.
- **Status:** Done
- **Verify:** `cargo test -p nodus --test observability` → 12 passed (was 4; +8). Full-crate `cargo test -p nodus` → **343 passed** (was 335; +8), 0 failed. `cargo clippy -p nodus --all-targets -- -D warnings` → clean (fixed 2 `derivable_impls` findings on `ExecutionMode`/`Determinism`'s manual `Default` impls). `cargo fmt -p nodus -- --check` → clean (after one `cargo fmt` pass, no logic change). No `.unwrap()`/`panic!()`/`unreachable!()` introduced in production code (checked directly). `git diff --stat -- crates/nodus/Cargo.toml crates/nodus/Cargo.lock` → empty; LP-1 zero-dep preserved.

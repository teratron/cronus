---
phase: 27
name: "Invocable Registry & CLI Derivation"
status: Todo
subsystem: "crates/contract · crates/domain · crates/core · crates/cli"
requires: []
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 27 Tasks — Invocable Registry & CLI Derivation

**Phase:** 27
**Status:** Todo
**Strategic Goal:** Mint one runtime invocable registry in the core, dispatch every action through it once, and move the command line onto it as a **projection** — so command parity stops being a property that is asserted and becomes one that is structural, and a verb contributed by an extension becomes reachable at all.

## Phase Notes

**Behaviour-preservation is the acceptance property, not a nice-to-have.** Every task in tracks A–D must leave observable command output unchanged. Where a task uncovers behaviour that is wrong, it is recorded as a **residual at the owning invocable** and left working as-is; the corrections ship in Phase 29 as separate disclosed changes (SP-10). A diff that both unifies and corrects presents a regression to users as a refactor to reviewers, and neither audience can read it.

**Four residuals are already known and must be preserved through this phase:** a store failure reported as an empty listing with a success exit code; the output-format flag discarded in nine sites; structured output assembled by unescaped string formatting in five; and a redaction path constructed with an empty secret list on every surface. Preserve, record, do not fix here.

**Build environment.** Anything that triggers C compilation (`rusqlite` `bundled`) or a Tauri/`windres` step must run in **PowerShell**, not Git Bash — MSYS2 makes `cc1.exe` fail to load there, so a clean check from Bash that suddenly fails at a C step is an environment artifact, not a code defect.

**Track independence.** A → B → (C ∥ D) → T, with **one crossing edge**: T-27D03 registers the command line against the corpus and therefore needs T-27C01. So C and D are file-independent and may run in parallel once B lands, but D cannot *finish* before C01 does. Treat C01 as the earliest task in track C rather than assuming the tracks are fully independent.

**Critical path and cascade risk.** Track B is the single point every later task and both later phases depend on. If the registry or dispatch shape is wrong, tracks C and D and all of Phases 28–29 rework. Land B behind its own tests before opening C or D, and prefer discovering a shape problem in B's unit tests over discovering it in the command line's migration.

**Sizing warning (planner audit).** Track D is the largest and most optimistically sized work here: the command line is ~4000 lines across 29 groups, and T-27D01/T-27D02 each touch all of them. Expect to split them by group with `.N` sub-task IDs rather than attempting one sweeping edit — the behaviour-preservation gate (`cli_smoke.rs` unmodified) is far easier to keep green in small increments, and a failed sweep is expensive to bisect.

## Atomic Checklist

- [ ] [T-27A01] Invocable descriptor and catalog types in the ports tier
- [ ] [T-27A02] Invocation / Outcome / Rejection envelope with four located rejection modes
- [ ] [T-27A03] Optional serialization feature on the ports tier
- [ ] [T-27B01] Registry: one published registration door, qualified identity, declared collision rule
- [ ] [T-27B02] Dispatch: bind-before-invoke over declared binders, returning a structured outcome
- [ ] [T-27B03] Contribution safety: failure policy, grant-gated reach, attribution marker
- [ ] [T-27B04] Core invocables registered from the facade through the public door; redaction at the boundary
- [ ] [T-27C01] Conformance fixture library and harness with three assertion families
- [ ] [T-27C02] Finding inventory and the two one-way ledgers, seeded with F-1…F-8
- [ ] [T-27D01] Command-line parser generated from descriptors
- [ ] [T-27D02] One renderer over the structured outcome; uniform output-format handling
- [ ] [T-27D03] Shipped-surface honesty: the five unbound groups leave the default surface; corpus registration
- [ ] [T-27T01] Corpus first run — convert failures into findings before fixing anything
- [ ] [T-27T02] Behaviour-preservation proof and full quality gates

## Detailed Tracking

### [T-27A01] Invocable descriptor and catalog types in the ports tier

- **Spec:** l2-invocable-registry.md §4.1, §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-contract` green; `cargo tree -p cronus-contract --depth 1` lists no external dependency in the default build.
- **Handoff:** T-27A02 (envelope types name these).
- **Notes:** Descriptor carries id, name, summary, group, `locus`, ordered binders, `stability`. The descriptor is the **whole** advertised contract — a surface must be able to render help, completion, and grouping from it alone, because any fact a surface must hold privately is the first step of a fork. `locus` is `Semantic | ClientLocal | HostOnly`; `stability` is `Shipped | Retired { superseded_by }`. Types live in the ports tier because all three surfaces and the domain must name them without depending on each other.

### [T-27A02] Invocation / Outcome / Rejection envelope with four located rejection modes

- **Spec:** l2-invocable-registry.md §4.5, §4.6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-contract` green, including a test asserting that an empty result and an unavailable source are **distinct** `Outcome` values and cannot be constructed from one another.
- **Handoff:** T-27B02 (dispatch produces these).
- **Notes:** `Outcome` is `Value | Stream | Rejected` and carries **structured data, never rendered text** — that is what lets one dispatch serve a text renderer, a structured renderer, terminal widgets, and IPC without any of them re-deriving the others' content. A `Rejection` names one of absent / unreadable / malformed / ill-shaped plus its location; collapsing the four into one "invalid input" is a defect, because each implies a different corrective action.

### [T-27A03] Optional serialization feature on the ports tier

- **Spec:** l2-invocable-registry.md §4.1
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo check -p cronus-contract` (default, no serde in the dependency graph) and `cargo check -p cronus-contract --features serde` both succeed; `cargo test -p cronus-contract --features serde` round-trips a descriptor and each `Outcome` variant.
- **Handoff:** Phase 29 (the desktop shell enables it).
- **Notes:** The ports tier is dependency-free **by construction** and its manifest carries an empty dependency section — so serialization is a feature, off by default, never an unconditional dependency. Landing it now rather than when the desktop needs it is SP-9: extraction before the second consumer is the only cheap point on the ladder.

### [T-27B01] Registry: one published registration door, qualified identity, declared collision rule

- **Spec:** l2-invocable-registry.md §4.3, §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain` green, including tests that (a) a contribution cannot claim the reserved core identity, (b) a bare verb colliding with a core verb resolves to the core's by the declared rule while the contribution stays reachable by qualified name, and (c) two sources claiming one identity resolve by declared source order with the later refused.
- **Handoff:** T-27B02, T-27B04.
- **Notes:** Registry logic is pure and belongs in the domain tier — no I/O. The reserved core namespace is what guarantees a contribution cannot shadow a core name **whatever the core adds later**, which a first-registration-wins rule cannot promise. Ordering must be *declared*, never a function of load order.

### [T-27B02] Dispatch: bind-before-invoke over declared binders, returning a structured outcome

- **Spec:** l2-invocable-registry.md §4.5, §4.6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain` green, including a test that a binding failure leaves **no** side effect — the body did not run — and a test that each of the four rejection modes reports its location.
- **Handoff:** T-27B03, T-27D01.
- **Notes:** Every declared binder completes before the body begins, which is what makes *did not run* a structural fact rather than a self-report and makes the whole rejection class safely re-invocable. Optional means **absent, never invalid**: a value that is present and fails to bind still rejects, with mode and location intact. Mapping a malformed value onto "not provided" is the quiet failure that makes a unit do confidently wrong work.

### [T-27B03] Contribution safety: failure policy, grant-gated reach, attribution marker

- **Spec:** l2-invocable-registry.md §4.10
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain` green, including tests that (a) a contributed invocable that panics or exceeds its bound resolves to a rejection **attributed to the contribution** and does not unwind into the core, (b) registering an invocable without the declared manifest grant is refused, and (c) the attribution marker is produced by the projection and cannot be set or suppressed by the contribution.
- **Handoff:** T-27T02.
- **Notes:** These three are what make a contributed verb *safe to be reachable*; reachability without them is the plugin story with its failure modes unspecified. The attribution rule is security-relevant, not cosmetic: a contributed verb that can present itself as the product converts the user's trust in the product into a capability an extension holds.

### [T-27B04] Core invocables registered from the facade through the public door; redaction at the boundary

- **Spec:** l2-invocable-registry.md §4.3, §4.12
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus` green; a test asserts every core invocable was registered through the same published function used by contributions (no private registration path exists); a test asserts dispatch output passes the shared redaction path exactly once.
- **Handoff:** T-27D01.
- **Notes:** EP-12 has one purpose — a seam exercised only by third parties is one nobody notices has become insufficient. Core-supplied contributions may differ in exactly three sanctioned ways (eager loading where ordering demands, implicit trust, surviving third-party disable switches) and no others. Redaction moves here so INV-7 is satisfied once instead of remembered three times; **the empty-secret-list half stays a recorded residual and is not fixed in this phase.**

### [T-27C01] Conformance fixture library and harness with three assertion families

- **Spec:** l2-surface-conformance.md §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-conformance` green on the harness's own self-tests; the harness is callable from another crate's test target (proven by T-27D03's use of it).
- **Handoff:** T-27D03, and Phases 28–29 registrations.
- **Notes:** New crate `cronus-conformance`, added to the engine workspace. It must be reachable **by path** from the detached desktop-shell workspace (which already depends on the facade that way), because that shell registers against the same harness in Phase 29 — a corpus crate the shell cannot link is a corpus one surface can never join. A **library plus a harness, not a suite** — the desktop shell builds in a detached workspace and cannot be linked from an engine-workspace test, and SP-7 wants registration to be something a surface does as part of becoming a surface. Three families: surface set (exposed = shipped minus **declared** exclusions), advertised schema against declared binders, and semantic outcome including rejection mode and the empty-versus-unavailable distinction. Fixtures are written **from the divergence, not the feature** — empty, single, zero-count, renamed, oversized, absent, unavailable, secret-bearing, identity-colliding — because the happy path is where implementations already agree and therefore proves nothing.

### [T-27C02] Finding inventory and the two one-way ledgers, seeded with F-1…F-8

- **Spec:** l2-surface-conformance.md §4.1, §4.2, §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** The inventory file lists all eight seed findings with every field of the §4.1 record populated, including `class` for each; at least two entries are class *preemptive* (an inventory with none has given up on SP-9). Ledger direction is checked by a test or a gate that fails on a tombstone removal or a debt addition.
- **Handoff:** T-27T01.
- **Notes:** Keep tombstones boringly literal — a path or an owner-qualified symbol plus its finding. Cleverness here produces a check nobody trusts and everybody bypasses. Repayment requires **all four** SP-4 conditions; three of four is recorded as *open*, not as *mostly repaid*.

### [T-27D01] Command-line parser generated from descriptors

- **Spec:** l2-cli.md §4.1, §4.3, §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-cli` green; `cronus --help` lists exactly the registry's shipped groups; the statically declared command enum is **deleted** from the crate and its removal recorded as a tombstone.
- **Handoff:** T-27D02.
- **Notes:** Build the parser at startup from descriptors using the builder API rather than the derive macro — the derive form is fixed at build time and therefore cannot carry a contributed verb, which is the whole reason for this phase. The project command grammar (verb-first, explicit verbs, noun groups) becomes a property the **registry** validates at registration, so it holds for a contributed verb as well as a core one.

### [T-27D02] One renderer over the structured outcome; uniform output-format handling

- **Spec:** l2-cli.md §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-cli` green; a test drives every shipped invocable through both output formats and asserts the requested format is honoured in **every** case; existing `crates/cli/tests/cli_smoke.rs` passes **unmodified** (behaviour preservation).
- **Handoff:** T-27D03.
- **Notes:** Rendering becomes one function of `Outcome` plus the requested format, replacing per-command decisions. **Record, do not fix:** the nine sites that discarded the format flag and the five that built structured output by unescaped string formatting are residuals — the renderer must reproduce today's observable output, including its defects, and Phase 29 corrects them separately.

### [T-27D03] Shipped-surface honesty: the five unbound groups leave the default surface; corpus registration

- **Spec:** l2-cli.md §3 (INV-9), l2-surface-conformance.md §4.5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cronus --help` and completion list **no** verb that answers "not implemented"; `cargo test -p cronus-cli` includes the conformance harness invoked against this surface's real projection.
- **Handoff:** T-27T01.
- **Notes:** `goal`, `trigger`, `mission`, `research`, `change` are unbound — they have no descriptor and therefore cannot appear. Their **behaviour is out of scope** for this phase; only their surface treatment changes. Register against the corpus here, before the second projection exists — registration is an order of magnitude cheaper before a surface has behaviour to preserve.

### [T-27T01] Corpus first run — convert failures into findings before fixing anything

- **Goal:** Run the corpus against the migrated command-line surface and treat the output as the initial inventory.
- **Method:** Execute the harness; expect it to fail immediately and unflatteringly. **Convert every failure into a finding record first**, then decide what is repaid in this phase and what is carried. Fixing on sight produces repairs that are rediscovered later as duplicates.
- **Verify:** Every corpus failure is either a closed finding with all four SP-4 conditions met, or an entry in the shrink-only debt ledger with a stated reason. No failure is silently fixed and unrecorded.
- **Status:** Todo

### [T-27T02] Behaviour-preservation proof and full quality gates

- **Goal:** Prove the migration changed structure and not behaviour, and that the phase meets the project's definition of done.
- **Method:** Assert pre-existing command-line tests pass **unmodified**; confirm the four known residuals are recorded at their owning invocables and still behave as before; run the workspace gates.
- **Verify:** `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` (zero), `cargo test --workspace` green across consecutive full runs; `crates/cli/tests/cli_smoke.rs` unmodified and passing; no `unwrap()`/`panic!()` introduced on production paths.
- **Status:** Todo

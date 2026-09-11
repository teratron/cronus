---
phase: 30
name: "Usage-Simulation Harness"
status: Todo
subsystem: "crates/simulation · simulations"
requires: [3, 27]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 30 Tasks — Usage-Simulation Harness

**Phase:** 30
**Status:** Todo
**Strategic Goal:** Land the machinery that turns a free-route run into evidence — disposable worlds, an as-it-happens transcript, a validator that refuses obligations which cannot fail, the wrapper every product invocation passes through, and the replay lane that guards findings at no inference cost. One authored scenario ships to prove the lane runs; the corpus itself is later work.

## Phase Notes

**The harness, not the corpus.** Exactly one `short` scenario lands here. The point of this phase is that a run becomes *recordable* and *replayable*, not that the product is covered.

**Ordering is load-bearing** (the spec's own §5). `world`'s refusals precede its happy path, because the missing guard is destructive rather than merely wrong. `transcript` precedes anything that reasons over a run, because until invocations are recorded as they happen, every later piece reasons over a narrative. The replay lane is green with one hand-written case before any agent runs. The validator lands with its refusal set intact — a validator added after scenarios exist is a validator that gets weakened to admit them.

**Track C is independent of Track A.** Scenario parsing touches no world and no transcript, so it can start immediately and does not stall if the Windows teardown or the repository-containment guard proves fiddly.

**Nothing here is wired into a build, package, or release step.** `cronus-simulation` is a workspace crate whose tests run with the workspace's tests; the agent-driven lanes are invoked by a person. A task that adds a build-manifest hook has gone outside this phase.

## Atomic Checklist

- [ ] [T-30A01] The crate exists in the ports tier, and a world refuses to be built inside the repository
- [ ] [T-30A02] Every invocation is recorded as it happens, never reconstructed afterwards
- [ ] [T-30B01] Pinned routes run as ordinary tests, with no agent in the loop
- [ ] [T-30C01] A scenario parses only if its obligations can fail
- [ ] [T-30C02] The first `short` scenario ships, and it is not a smoke test
- [ ] [T-30D01] `cronus-sim` is the only path by which a run touches the product; findings carry their attribution
- [ ] [T-30D02] Coverage reports the complement against the catalog, never a percentage
- [ ] [T-30T01] Invariant compliance, containment, and full quality gates

## Detailed Tracking

### [T-30A01] The crate exists in the ports tier, and a world refuses to be built inside the repository

- **Spec:** l2-simulation-suite.md §4.1, §3 (USM-2 row)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** New workspace member `crates/simulation` (package `cronus-simulation`), depending on `cronus-contract` and nothing from the domain tier — the same rationale `crates/conformance/Cargo.toml` already records. `src/world.rs` provides `build` / `drop`: a fresh directory under `std::env::temp_dir()` named by scenario id + UTC stamp, made the working directory of every spawned process; all `CRONUS_*` variables plus the `HOME`/`USERPROFILE` values used for configuration discovery cleared from the child environment; product version and the repository's dirtiness digest stamped at build time; teardown removing the tree with bounded retries, because a held handle delays deletion on Windows. `build` **refuses** when the resolved root lies inside the repository working tree.
- **Verify:** `cargo test -p cronus-simulation` (PowerShell, per the project's native-build discipline) passes with three new tests: (a) `world::build` against a root resolving inside the repository returns an error naming the containment refusal, and no directory is created; (b) a built world's working directory has no ancestor equal to the repository root, and a `CRONUS_MISSION_MODE` value exported in the parent process is absent from the recorded child environment; (c) `drop` removes the tree, and a second `drop` on the same world is a no-op rather than an error. `cargo clippy -p cronus-simulation --all-targets -- -D warnings` is clean.
- **Handoff:** T-30A02, T-30B01, T-30D01.
- **Notes:** Worlds are never reused — a second run gets a new directory, so a stale tree left behind by an exhausted retry budget is identifiable by its name rather than silently inherited.

### [T-30A02] Every invocation is recorded as it happens, never reconstructed afterwards

- **Spec:** l2-simulation-suite.md §4.3, §3 (USM-5 row)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `src/transcript.rs` — an append-only record written at the moment each invocation completes, one entry per product invocation carrying argv, stdin, stdout, stderr, exit code, wall duration, and a digest of the world's state before and after. The transcript header carries the product version, the resolved environment, and the world id from T-30A01. Entries are addressable by index, because verdicts cite indices rather than restating output.
- **Verify:** `cargo test -p cronus-simulation` passes with: (a) a test spawning the real `cronus --help` through the recorder asserts the entry's stdout equals the process's actual stdout byte-for-byte and that `duration` is greater than zero; (b) a test spawning a command that exits non-zero asserts an entry is still written, with the non-zero code and the stderr captured; (c) a test asserts the before/after state digests differ for an invocation that writes into the world and are equal for one that does not.
- **Handoff:** T-30B01, T-30D01.
- **Notes:** Writing at completion — not at the end of the run — is the property being tested. A test that builds the whole transcript in memory and flushes once would pass a weaker assertion than the one this task exists for.

### [T-30B01] Pinned routes run as ordinary tests, with no agent in the loop

- **Spec:** l2-simulation-suite.md §4.4, §3 (USM-7 row)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `src/replay.rs` executes a pinned route: build a world, run the recorded argv sequence in order, assert the recorded outcome. `tests/replays.rs` discovers and runs every case under `tests/replays/`, and one hand-written case lands with it — `tests/replays/init-then-status.toml`, covering `cronus init` writing the skeleton under `.cronus/` and `status` then finding that workspace, which is behaviour the smoke suite already asserts and which therefore cannot be wrong here for reasons outside this phase.
- **Verify:** `cargo test -p cronus-simulation --test replays` runs the case and reports it by name. Positive control: a copy of the case with its expected exit code altered fails, and the failure message names the case and the divergence — added as a `#[test]` over an inline fixture, not by committing a broken case. `cargo test` at the workspace root includes this target.
- **Handoff:** T-30T01.
- **Notes:** The replay runner makes no judgements and needs no vantage. Everything interesting happened when the route was discovered; its only job is to notice when the route stops holding.

### [T-30C01] A scenario parses only if its obligations can fail

- **Spec:** l2-simulation-suite.md §4.2, §3 (USM-1 / USM-3 / USM-11 rows)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `src/scenario.rs` parses Markdown with TOML frontmatter into a typed scenario: `id`, `tier` (from directory placement), `surfaces`, `covers`, `roles`, `vantage` (closed enum), `world`, `bound` (steps / wall seconds / spend), `[[perturbation]]`, `[[obligation]]`. **Unknown keys are rejected**, so a route cannot be smuggled in as an extra field. The validator refuses at parse time: an obligation with no `evidence`; an activity-shaped `statement`; an expectation whose only pass signal is vocabulary failure output also produces; an absence claim with no declared `positive_control`; an empty or duplicate-identity obligation set; a missing `bound`. A refusal is an error, never a skip.
- **Verify:** `cargo test -p cronus-simulation` passes a table-driven test with one fixture per refusal class, each asserting a parse error whose message names that class, plus one valid scenario that parses into the expected typed value. A fixture carrying an unknown top-level key is refused. Scenarios with zero perturbations parse and are reported `smoke`.
- **Handoff:** T-30C02, T-30D01, T-30D02.
- **Notes:** Independent of Track A — this task needs no world and no transcript, and should start in parallel with T-30A01.

### [T-30C02] The first `short` scenario ships, and it is not a smoke test

- **Spec:** l2-simulation-suite.md §4.1, §4.2; l1-usage-simulation.md §4.4 (perturbation classes)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `simulations/README.md` (how to author one; the eleven perturbation classes; what a vantage is) and `simulations/short/first-run-three-boards.md` — persona new to the product, vantage `builtin-help`, a goal stated in the persona's vocabulary and not the product's, at least two perturbations (reversal and return), and falsifiable obligations with one absence claim carrying its positive control. No file under `simulations/` references any planning or specification artifact.
- **Verify:** `cargo test -p cronus-simulation` passes a test that parses every file under `simulations/`, asserting the corpus is non-empty, that the `short` tier holds at least one scenario, and that every `short` scenario declares at least one perturbation. A second assertion greps the corpus for the containment-forbidden reference classes (task ids, phase designators, `.design/` paths, SDD system filenames) and fails on any hit.
- **Handoff:** T-30D01, T-30T01.
- **Notes:** The goal must not name a product verb — the validator flags vocabulary leakage, and this scenario is the first thing that check runs against.

### [T-30D01] `cronus-sim` is the only path by which a run touches the product; findings carry their attribution

- **Spec:** l2-simulation-suite.md §4.3, §4.5, §3 (USM-3 / USM-5 / USM-9 / USM-12 rows)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `src/bin/cronus_sim.rs` — `world new`, `run -- <argv>`, `note`, `verdict`, `finish`, `pin`, `coverage`. `finish` computes exit status **from obligation verdicts alone**: pass, fail, or `incomplete` as a third status distinct from both, emitted when the scenario's `bound` is exhausted or the run is interrupted, naming the undecided obligations. Discoveries recorded by `note` can never fail a run. `finish` also writes the run report, and `src/findings.rs` supplies the attribution it records: **product defect** (pin, then fix), **scenario defect** (the obligation cited entries that do not support it, or it failed on this route and passed on an earlier route through the same scenario with no product change), **environment defect** (the wrapper failed, the world could not be built or torn down, or the repository digest changed during the run — the run is `void`, not failed). Product-binary resolution is explicit: an argument or environment variable names the binary, otherwise the workspace target directory's build is used, and **the resolved path is recorded in the transcript header** — `CARGO_BIN_EXE_cronus` is unavailable to a binary target, and silently falling back to an installed `cronus` would measure the wrong product.
- **Verify:** An integration test drives the built `cronus-sim` end-to-end over fixture scenarios and asserts four distinct outcomes: a run whose obligations all pass exits `0`; a run with one failing obligation exits non-zero; a run whose only negative signal is a `note` discovery still exits `0`; a bounded-out run exits with the `incomplete` status code, and its report names the undecided obligations. A fifth assertion: with no binary argument and no environment override, the resolved path recorded in the transcript lies inside the workspace target directory. A sixth: a run whose repository digest differs between world build and teardown is reported `contaminated` with its verdicts discarded, and a run whose world could not be torn down is attributed `environment` and reported `void` rather than `fail` — the two attributions that must never be filed as product findings.
- **Handoff:** T-30D02, T-30T01.
- **Notes:** Routing every invocation through `run` is not convenience — it is why the transcript exists whether or not the agent chooses to mention what happened, which is also what makes a silently abandoned attempt visible.

### [T-30D02] Coverage reports the complement against the catalog, never a percentage

- **Spec:** l2-simulation-suite.md §3 (USM-8 row), §4.1
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `src/coverage.rs` enumerates the product's action catalog, intersects it with the `surfaces` / `covers` / `roles` declarations of every parsed scenario, and prints **what nothing simulates**. The catalog is obtained through the product's own projected catalog surface via the wrapper, not by linking a live registry — the crate stays on `cronus-contract` shapes, so a surface that cannot link the domain tier could still join later.
- **Verify:** `cargo test -p cronus-simulation` passes a test supplying a stub catalog of three entries and a scenario covering one, asserting the printed complement names exactly the other two and nothing else. A second test asserts the output carries a count and the entry names, and contains no percentage. `cronus-sim coverage` against the real product runs and exits `0` with a non-empty complement (the corpus holds one scenario; a complement of zero at this point would mean the catalog was not read).
- **Handoff:** T-30T01.
- **Notes:** The complement being large is the correct result for this phase. It becomes a coverage claim only once the corpus grows, which is deliberately not this phase's work.

### [T-30T01] Validation Task — invariant compliance, containment, and full quality gates

- **Goal:** Verify the landed harness against `l2-simulation-suite`'s Invariant Compliance table and the project's definition of done.
- **Method:** (a) Workspace gates, run in PowerShell: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo fmt --all --check` — all green, with `cargo test` retried at `-j 2` before any failure is treated as a real defect, per the known rustc-under-full-parallelism crash on this host. (b) A written mapping from each of USM-1…USM-12 to the code or test that discharges it, recorded in the task's `Changes` field; any invariant with no landed discharge is named as such rather than omitted. (c) Structural containment: `Cargo.toml`, the workspace manifest, and any CI configuration contain **no** invocation of an engine script and no dependency introduced solely to run one — asserted by inspection and recorded. (d) Reference containment: no file under `crates/simulation/` or `simulations/` contains a task id, a phase designator, an SDD system filename, or a `.design/` path.
- **Verify:** All four sub-checks reported individually with their evidence; a partial pass is reported as partial, never aggregated into green.
- **Status:** Todo

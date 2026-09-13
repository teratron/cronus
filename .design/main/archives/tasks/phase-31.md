---
phase: 31
name: "Discovery Classification (USM-13)"
status: Done
subsystem: "crates/simulation"
requires: [30]
provides:
  - "a closed five-name `DiscoveryClass` vocabulary in findings.rs (defect / friction / inefficiency / optimization-opportunity / improvement-idea), reusing the improvement-loop's own taxonomy verbatim, with parse/name/all_names_joined helpers and a serde form that matches its parsed spelling exactly"
  - "`RunState.notes` widened from Vec<String> to Vec<Discovery> (text + optional class + optional remedy, both new fields #[serde(default)]), with add_note taking both as optional parameters"
  - "cronus-sim note --class <name> --remedy <text>, flags accepted anywhere in the trailing arguments; an unrecognized class is a usage error (exit 5) naming all five valid classes"
  - "finish's report renders a discovery's class and remedy when present, the remedy explicitly labelled a proposal (not applied); an unclassified, remedy-free discovery renders exactly as before"
key_files:
  created: []
  modified:
    - "crates/simulation/src/findings.rs (DiscoveryClass, Discovery, FinishReport.notes retyped, Display extended, 9 new unit tests)"
    - "crates/simulation/src/run_state.rs (notes: Vec<Discovery>, add_note(text, class, remedy), 3 new unit tests)"
    - "crates/simulation/src/bin/cronus_sim.rs (cmd_note accepts --class/--remedy, doc comment updated)"
    - "crates/simulation/tests/cronus_sim_e2e.rs (3 new end-to-end tests)"
patterns_established:
  - "a closed vocabulary shared between a persisted enum and a CLI flag keeps one source of truth for its spelling: DiscoveryClass::name() is asserted equal to its own serde form by a unit test, so the CLI parser and the JSON schema cannot silently drift apart"
  - "the library stays the source of truth for a usage-error message's content (DiscoveryClass::all_names_joined()) rather than the binary hand-rolling the list a second time, per this project's CLI-is-a-thin-binding convention"
duration_minutes: 45
---

# Stage 31 Tasks — Discovery Classification (USM-13)

**Phase:** 31
**Status:** Done
**Strategic Goal:** Close the one row `l2-simulation-suite` §3 has carried as **Pending** since the harness shipped — a discovery may carry a class from the improvement-loop taxonomy and an optional proposed remedy, neither affecting what a run decides.

## Phase Notes

**Classification only — no authority, nothing automatic.** `l1-remedy-authority` grants permissions *against* these classes and has no `Implements:` L2, so it is unbuildable by construction right now. This phase adds two optional fields to a record. A classified discovery still cannot fail a run (USM-3), a proposed remedy is still a claim for a human to weigh, and the run still repairs nothing (USM-12).

**Ordering held as planned.** The vocabulary (A01) landed before the record that embeds it (A02), and the record before the wrapper flags that write it (B01) — a surface accepting a flag for a field with nowhere to go would have produced a parser that discarded what it parsed. B01 and B02 turned out to touch the same commit (the render logic lives in the same `Display` impl the vocabulary required), so they are recorded together rather than split into an artificial two-step diff.

## Atomic Checklist

- [x] [T-31A01] A closed five-name discovery vocabulary, parseable and round-trip-safe
- [x] [T-31A02] The record carries an optional class and remedy, and persists them
- [x] [T-31B01] `cronus-sim note` accepts `--class` / `--remedy`, and rejects an unknown class by name
- [x] [T-31B02] `finish` renders class and remedy as a proposal, unclassified notes unchanged
- [x] [T-31T01] Full-cycle validation and quality gates

## Detailed Tracking

### [T-31A01] A closed five-name discovery vocabulary, parseable and round-trip-safe

- **Spec:** l1-usage-simulation.md USM-13; l1-improvement-loop.md IMP-1 (the reused taxonomy)
- **Status:** Done
- **Assignment:** Agent
- **Scope:** `DiscoveryClass` enum in `crates/simulation/src/findings.rs` — `Defect | Friction | Inefficiency | OptimizationOpportunity | ImprovementIdea`, `#[serde(rename_all = "kebab-case")]`. `parse(&str) -> Option<Self>` rejects anything outside the five (no coercion to a nearest guess); `name()` renders the same spelling `parse` accepts; `all_names_joined()` for usage-error messages.
- **Verify:** `cargo test -p cronus-simulation --lib findings` — `all_five_class_names_parse_back_to_the_variant_that_named_them`, `an_unrecognized_class_name_is_rejected_rather_than_coerced` (`"bug"`, `""`, and the case-sensitive `"Defect"` all rejected), `every_class_survives_a_json_round_trip_under_its_own_name`, `all_names_joined_names_every_one_of_the_five_classes` — 4/4 pass.
- **Handoff:** T-31A02 (the record embeds this enum).
- **Changes:** Added `DiscoveryClass` (5 variants, closed) to `findings.rs` with `parse`/`name`/`all_names_joined`/`Display`, plus 4 unit tests proving the parse↔render↔serde triangle agrees.
- **Notes:** Deliberately reuses IMP-1's five names rather than inventing a sixth vocabulary, per USM-13's own closing sentence.

### [T-31A02] The record carries an optional class and remedy, and persists them

- **Spec:** l1-usage-simulation.md USM-13
- **Status:** Done
- **Assignment:** Agent
- **Scope:** New `Discovery { text: String, class: Option<DiscoveryClass>, remedy: Option<String> }` in `findings.rs`, both new fields `#[serde(default, skip_serializing_if = "Option::is_none")]`. `RunState.notes: Vec<String>` → `Vec<Discovery>` in `run_state.rs`; `add_note(text, class, remedy)`; `FinishReport.notes` retyped to match (no conversion — the same value carries straight through).
- **Verify:** `cargo test -p cronus-simulation --lib` — findings.rs: `a_plain_discovery_carries_no_class_or_remedy`, `a_discovery_json_object_lacking_class_and_remedy_keys_still_deserializes`, `a_fully_classified_discovery_round_trips_through_json_intact`; run_state.rs: `discoveries_of_every_shape_persist_and_reload_through_run_state_json` (fully-classified, text-only, and classified-with-no-remedy all round-trip through a real `save()`/`load()` cycle), `a_run_state_json_with_no_notes_key_at_all_still_loads_with_an_empty_discovery_list` (the outer field-level default, distinct from the inner one). 36/36 lib tests pass.
- **Handoff:** T-31B01 (the wrapper flag needs a field to write into).
- **Changes:** `Discovery` struct added to `findings.rs`; `RunState.notes` retyped and `add_note` signature extended in `run_state.rs`; `FinishReport.notes` retyped. 6 new unit tests across the two files.
- **Notes:** A `run-state.json` shaped like the pre-USM-13 `Vec<String>` schema is not a supported load target — that is a type change, not a missing-field default, and no scenario in this project holds a world open across a binary upgrade. What *is* guaranteed (and tested) is that a `Discovery` object missing its two new keys, and a `RunState` document missing `notes` entirely, both still load.

### [T-31B01] `cronus-sim note` accepts `--class` / `--remedy`, and rejects an unknown class by name

- **Spec:** l2-simulation-suite.md §4.3 (wrapper surface); l1-usage-simulation.md USM-13
- **Status:** Done
- **Assignment:** Agent
- **Scope:** `cmd_note` in `crates/simulation/src/bin/cronus_sim.rs` scans its trailing arguments for `--class <name>` and `--remedy <text>` (either position, either both, or neither), joining what remains as the discovery text. An unrecognized class name is `CliError::Usage`, which `main()` already turns into a stderr message and exit 5; the message is built from `DiscoveryClass::all_names_joined()` so the library stays the one source of truth for the vocabulary the binary is thin over.
- **Verify:** `cargo test -p cronus-simulation --test cronus_sim_e2e` — `an_unrecognized_discovery_class_is_a_usage_error_naming_all_five_valid_classes` (exit 5; stderr contains all five class names by name, not by count).
- **Handoff:** T-31B02 (the report needs to render what this stores).
- **Changes:** `cmd_note` rewritten to a flag-scanning loop; doc comment at the top of `cronus_sim.rs` extended with the new usage line.
- **Notes:** None.

### [T-31B02] `finish` renders class and remedy as a proposal, unclassified notes unchanged

- **Spec:** l1-usage-simulation.md USM-13 ("a proposed remedy is a claim for a human to weigh, never an act the run performs")
- **Status:** Done
- **Assignment:** Agent
- **Scope:** `FinishReport`'s `Display` impl in `findings.rs` (the one place the report is rendered, regardless of which binary calls it) prints `class: <name>` and `proposed remedy (not applied): <text>` per discovery only when present; a discovery with neither prints exactly the single `- {text}` line (two leading spaces) it always has.
- **Verify:** `cargo test -p cronus-simulation --test cronus_sim_e2e` — `a_classified_discovery_with_a_proposed_remedy_survives_into_the_finish_report` (text, class, and remedy all present in the report; outcome stays `pass`, proving USM-3 holds); `an_unclassified_note_renders_exactly_as_it_did_before_usm_13` (no `class:` or `proposed remedy` string appears when neither field is set); the pre-existing `a_note_only_discovery_does_not_affect_a_passing_outcome` still passes unmodified.
- **Handoff:** T-31T01.
- **Changes:** `Display for FinishReport` extended with the two conditional lines described above.
- **Notes:** None.

### [T-31T01] Full-cycle validation and quality gates

- **Goal:** Prove a classified discovery with a proposed remedy survives a real `world new → run → note --class → verdict → finish` cycle end to end, with the run's outcome unaffected by the classification, and that nothing in this phase's diff regressed the existing suite.
- **Method:** Full crate test suite plus workspace-standard quality gates.
- **Status:** Done
- **Verify:**
  - `cargo test -j 2 -p cronus-simulation` (PowerShell) — 36 lib tests, 14 e2e tests (11 pre-existing + 3 new), 1 replay-lane test, all green.
  - `cargo clippy -p cronus-simulation --all-targets -- -D warnings` (PowerShell) — clean.
  - `cargo fmt --all -- --check` (PowerShell) — clean (one formatting pass applied and re-verified during this task).
  - Production-path scan: every `unwrap()`/`panic!()`/`.expect()` introduced by this phase's diff resolves to `#[cfg(test)]`-gated code; the one production-path `.expect()` this scan surfaced (`cmd_run`'s `state.entry(idx).expect(...)`) predates this phase and was not touched.
  - Containment scan: no task-ID, phase-designator, or `.design/` path leaked into any touched source or test file.

## Provenance

Executed via `/magic.run main`. See the phase frontmatter above for `provides`/`key_files`/`patterns_established`.

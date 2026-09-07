//! The finding inventory: one mapping, discovered written by hand three
//! times, tracked here until every copy is actually gone. A finding
//! recorded and later fixed from memory is a finding that returns; this
//! module is what makes the class shrink monotonically instead of
//! oscillating.
//!
//! Two one-way ledgers travel with it. [`tombstones`] only grows — a
//! location named here must never come back. [`accepted_debt`] is meant to
//! shrink — a finding knowingly left open, with its reason. Both ledgers
//! are checked against a hand-maintained baseline (see the `ledger_baseline`
//! test module below): a location disappearing from the live tombstone list
//! or drifting out of the live/baseline debt agreement fails the build,
//! precisely so either reversal is visible as a deliberate edit to the
//! baseline rather than arriving as a quiet accident.

/// Whether a finding was discovered already existing (a duplicate found in
/// production or review) or recorded before a second copy could form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingClass {
    Observed,
    Preemptive,
}

/// Every condition a finding must satisfy before it is repaid. Three of
/// four is not partial credit — it is recorded as open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Repayment {
    pub copies_deleted: bool,
    pub deletion_pinned: bool,
    pub fixture_landed: bool,
    pub consumer_registered: bool,
}

impl Repayment {
    pub const fn open() -> Self {
        Repayment {
            copies_deleted: false,
            deletion_pinned: false,
            fixture_landed: false,
            consumer_registered: false,
        }
    }

    pub fn is_repaid(&self) -> bool {
        self.copies_deleted
            && self.deletion_pinned
            && self.fixture_landed
            && self.consumer_registered
    }
}

/// One tracked divergence: where it lives, what disagrees, and how it gets
/// fixed. Text fields are prose on purpose — a finding is read by a person
/// deciding what to do next, not machine-matched.
#[derive(Debug, Clone)]
pub struct Finding {
    /// Stable identity — referenced by tombstones and debt entries below,
    /// never recomputed from position.
    pub id: &'static str,
    pub sites: &'static str,
    pub divergence: &'static str,
    pub class: FindingClass,
    pub primitive: &'static str,
    pub repayment: Repayment,
    /// A known-incorrect behavior the extraction preserved, and why —
    /// `None` when there is nothing carried, which is the common case.
    pub residual: Option<&'static str>,
}

/// The eight findings from the audit that produced the invocable registry
/// and this corpus. Repayment reflects the state as of this task, not an
/// aspiration — most read `open` here because the primitive they need
/// (the registry, the corpus) has just been built; the migrations that
/// actually delete each copy are later tasks and phases.
pub fn seed_inventory() -> Vec<Finding> {
    vec![
        Finding {
            id: "F-1",
            sites: "the command line's parser enum, the terminal UI's slash catalog, and the desktop's per-capability IPC list — no single authoritative site, three peers",
            divergence: "the three describe different verb sets: twenty-nine command-line groups, twenty-one terminal-UI verbs, four desktop capabilities",
            class: FindingClass::Observed,
            primitive: "the invocable registry",
            repayment: Repayment {
                // The registry and its dispatch exist; the corpus's
                // surface-set family is exactly this finding's fixture.
                fixture_landed: true,
                ..Repayment::open()
            },
            residual: None,
        },
        Finding {
            id: "F-2",
            sites: "the terminal UI's hand-copied verb mirror",
            divergence: "a parity check compared a stale copy of the thing under test against itself, and stayed green while the surfaces differed by eight verbs — a check that cannot fail",
            class: FindingClass::Observed,
            primitive: "a corpus driven through real projections",
            repayment: Repayment {
                fixture_landed: true,
                ..Repayment::open()
            },
            residual: None,
        },
        Finding {
            id: "F-3",
            sites: "the command line's own hand-maintained command-parity documentation table",
            divergence: "the table listed verbs the product did not have and omitted every group it did",
            class: FindingClass::Observed,
            primitive: "a catalog projection, never a maintained table",
            // `[CLOSED]` The command line — this finding's one named site —
            // registered against the corpus, driving its real projection
            // through the harness with a zero-report surface-set/schema
            // result (the only reports that run produced were the two
            // already-accepted F-5 outcome-family fixtures, unrelated to
            // this finding). All four SP-4 conditions now hold: the table
            // was already deleted (`copies_deleted`) and pinned
            // (`deletion_pinned`) before this phase's code work began; the
            // surface-set family is its fixture (`fixture_landed`); and the
            // one surface this finding names has now registered and run it
            // (`consumer_registered`). The first finding this corpus
            // actually closes, not merely tracks.
            repayment: Repayment {
                copies_deleted: true,
                deletion_pinned: true,
                fixture_landed: true,
                consumer_registered: true,
            },
            residual: None,
        },
        Finding {
            id: "F-4",
            sites: "the command line's own board-path derivation",
            divergence: "a frontend computes a domain fact (where the board lives on disk); a second surface reaching parity would have to re-derive the identical fact rather than receive it",
            class: FindingClass::Observed,
            primitive: "a core-owned invocable that returns the fact instead of the frontend deriving it",
            repayment: Repayment::open(),
            residual: Some(
                "not yet covered by a corpus fixture — this divergence is a specific domain fact, not one of the dispatch-mechanics shapes the seed fixture set exercises. Needs its own fixture once the command line's migration (a later track) actually confronts the derivation.",
            ),
        },
        Finding {
            id: "F-5",
            sites: "the terminal UI's own redaction call, the desktop bridge's own redaction call, and the command line, which redacted nothing",
            divergence: "two implementations and one omission of one security property, each one call site's worth of drift from the others",
            class: FindingClass::Observed,
            primitive: "redaction at the single dispatch boundary",
            repayment: Repayment {
                // The boundary exists and is exercised by real tests; the
                // corpus's secret-bearing and boundary-crossing outcome
                // fixtures are this finding's fixture.
                fixture_landed: true,
                ..Repayment::open()
            },
            residual: Some(
                "the boundary is fed an empty secret list on every surface today, so masking is presently inert wherever it runs. The corpus's own secret-bearing and boundary-crossing fixtures are expected to fail for exactly this reason until that separate obligation is discharged — the failure is the finding staying honest, not a defect in the corpus.",
            ),
        },
        Finding {
            id: "F-6",
            sites: "per-command output rendering across the command line",
            divergence: "the requested output format is discarded in nine sites; structured output is hand-built and unescaped in five; two surfaces that render the same value do it by separate code",
            class: FindingClass::Observed,
            primitive: "one renderer per surface, driven by the structured outcome",
            repayment: Repayment::open(),
            residual: Some(
                "this is rendering fidelity, not semantic outcome — the corpus's outcome family deliberately does not police bytes rendered, only meaning. This finding's fixture lives at the renderer's own level (a later task), not in the cross-surface corpus.",
            ),
        },
        Finding {
            id: "F-7",
            sites: "the desktop WebView's client (not yet a copy — the risk is that it becomes one)",
            divergence: "a hand-written client mirroring the IPC command list would be a fourth restatement of the same mapping, arriving the moment anyone hand-adds a capability to it",
            class: FindingClass::Preemptive,
            primitive: "a client generated from the catalog",
            repayment: Repayment::open(),
            residual: None,
        },
        Finding {
            id: "F-8",
            sites: "any future surface's help or completion listing",
            divergence: "a new surface with no shared catalog to render from would restate the verb set by hand to build its own discovery",
            class: FindingClass::Preemptive,
            primitive: "the catalog projection this corpus's registering surfaces already use",
            repayment: Repayment {
                // The mechanism a new surface needs already exists; nobody
                // has been the concrete "future surface" yet.
                fixture_landed: true,
                ..Repayment::open()
            },
            residual: None,
        },
    ]
}

/// One append-only entry: a location that must never come back, and the
/// finding it closes out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tombstone {
    pub finding: &'static str,
    pub location: &'static str,
}

/// Append-only (SP-5). A location named here must never reappear — kept
/// boringly literal on purpose, since cleverness here produces a check
/// nobody trusts and everybody bypasses.
pub fn tombstones() -> Vec<Tombstone> {
    vec![Tombstone {
        finding: "F-3",
        location: "the command line's own hand-maintained command-parity documentation table",
    }]
}

/// One shrink-only entry: a finding knowingly left open, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedDebt {
    pub finding: &'static str,
    pub reason: &'static str,
}

/// Shrink-only (SP-5). An entry leaving this list is ordinary progress; an
/// entry arriving is a change that must say so — enforced by the baseline
/// comparison in this module's own tests, which requires an entry's arrival
/// or departure to be a deliberate edit to the checked-in baseline, not an
/// incidental one.
pub fn accepted_debt() -> Vec<AcceptedDebt> {
    vec![
        AcceptedDebt {
            finding: "F-5",
            reason: "unmasked output is preserved through convergence and corrected as a separate, disclosed change (SP-10) — the empty secret list is deliberately not yet populated",
        },
        AcceptedDebt {
            finding: "F-6",
            reason: "the output-format flag and unescaped structured output are known-wrong behaviors preserved through convergence and corrected separately (SP-10), not fixed inside the migration that would hide them as a refactor",
        },
    ]
}

/// The hand-maintained "last accepted" ledger state — a **separate, literal
/// copy**, never derived from [`tombstones`]/[`accepted_debt`] by calling
/// back into them. Deriving it from the live functions would make every
/// comparison below vacuously true, since the baseline would always equal
/// whatever the live data currently says; the entire mechanism depends on
/// this module being edited by a human, on purpose, only when a ledger
/// change is being deliberately accepted.
///
/// **To accept a ledger change**: edit this module's literals to match
/// [`tombstones`]/[`accepted_debt`] in the same commit that changes them.
/// That edit is the "must say so" SP-5 requires — visible in the diff,
/// never incidental.
#[cfg(test)]
mod ledger_baseline {
    use super::{AcceptedDebt, Tombstone};

    pub fn tombstones() -> Vec<Tombstone> {
        vec![Tombstone {
            finding: "F-3",
            location: "the command line's own hand-maintained command-parity documentation table",
        }]
    }

    pub fn accepted_debt() -> Vec<AcceptedDebt> {
        vec![
            AcceptedDebt {
                finding: "F-5",
                reason: "unmasked output is preserved through convergence and corrected as a separate, disclosed change (SP-10) — the empty secret list is deliberately not yet populated",
            },
            AcceptedDebt {
                finding: "F-6",
                reason: "the output-format flag and unescaped structured output are known-wrong behaviors preserved through convergence and corrected separately (SP-10), not fixed inside the migration that would hide them as a refactor",
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_seed_finding_has_every_anatomy_field_populated() {
        for finding in seed_inventory() {
            assert!(!finding.id.is_empty());
            assert!(
                !finding.sites.is_empty(),
                "{}: sites must be named",
                finding.id
            );
            assert!(
                !finding.divergence.is_empty(),
                "{}: divergence must be stated in user-visible terms",
                finding.id
            );
            assert!(
                !finding.primitive.is_empty(),
                "{}: the replacing primitive must be named",
                finding.id
            );
        }
    }

    #[test]
    fn the_seed_inventory_holds_exactly_the_eight_findings_from_the_audit() {
        let ids: Vec<&str> = seed_inventory().iter().map(|f| f.id).collect();
        assert_eq!(
            ids,
            vec!["F-1", "F-2", "F-3", "F-4", "F-5", "F-6", "F-7", "F-8"]
        );
    }

    #[test]
    fn at_least_two_findings_are_preemptive() {
        // An inventory with no preemptive entries has given up on SP-9:
        // extracting before the second implementation exists is the only
        // cheap point on the ladder, and it only shows up as a class here.
        let preemptive = seed_inventory()
            .iter()
            .filter(|finding| finding.class == FindingClass::Preemptive)
            .count();
        assert!(
            preemptive >= 2,
            "expected at least 2 preemptive findings, found {preemptive}"
        );
    }

    #[test]
    fn exactly_f3_reads_as_repaid_after_the_command_lines_registration() {
        // Repayment requires all four SP-4 conditions. F-3's one named site
        // is the command line's own table, deleted and pinned before this
        // phase's code work began — its remaining condition
        // (`consumer_registered`) closed the moment the command line ran
        // the corpus for real. Every other finding names at least one site
        // this project has not built or registered yet (the terminal UI, the
        // desktop shell, or a still-open residual), so none of them should
        // read as repaid. A finding flipping to "repaid" is a real event
        // this test exists to make visible, not something that happens
        // silently — this assertion is the visible record of the one that
        // already has, and a guard against any other flipping unnoticed.
        for finding in seed_inventory() {
            let expected_repaid = finding.id == "F-3";
            assert_eq!(
                finding.repayment.is_repaid(),
                expected_repaid,
                "{} reads as repaid={}, expected={expected_repaid} — is that actually true?",
                finding.id,
                finding.repayment.is_repaid(),
            );
        }
    }

    #[test]
    fn every_tombstone_names_a_finding_that_actually_exists() {
        let known_ids: Vec<&str> = seed_inventory().iter().map(|f| f.id).collect();
        for tombstone in tombstones() {
            assert!(
                known_ids.contains(&tombstone.finding),
                "tombstone names unknown finding {}",
                tombstone.finding
            );
        }
    }

    #[test]
    fn every_accepted_debt_entry_names_a_finding_that_is_genuinely_still_open() {
        let inventory = seed_inventory();
        for debt in accepted_debt() {
            let finding = inventory
                .iter()
                .find(|f| f.id == debt.finding)
                .unwrap_or_else(|| panic!("accepted debt names unknown finding {}", debt.finding));
            assert!(
                !finding.repayment.is_repaid(),
                "{} is listed as accepted debt but reads as fully repaid — remove it from the debt ledger",
                finding.id
            );
            assert!(
                !debt.reason.is_empty(),
                "{}: debt must state its reason",
                debt.finding
            );
        }
    }

    #[test]
    fn no_tombstoned_location_has_disappeared_from_the_baseline() {
        // Append-only, enforced: every location the baseline already
        // accepted must still be present in the live ledger. Removing one
        // here — accidentally or not — fails this test; only editing
        // `ledger_baseline` (this module) alongside `tombstones()` records
        // the change as the deliberate act it must be.
        let live = tombstones();
        for accepted in ledger_baseline::tombstones() {
            assert!(
                live.contains(&accepted),
                "tombstone for {} at {:?} was removed from the live ledger",
                accepted.finding,
                accepted.location
            );
        }
    }

    #[test]
    fn accepted_debt_matches_the_baseline_exactly() {
        // Shrink-only, enforced as exact agreement: any drift — an entry
        // added or removed — requires editing `ledger_baseline` (this
        // module) in the same change, which is what makes either direction
        // a visible, reviewable edit rather than a quiet one.
        let live = accepted_debt();
        let baseline = ledger_baseline::accepted_debt();
        for entry in &live {
            assert!(
                baseline.contains(entry),
                "debt entry for {} is new and not yet reflected in the baseline — is this addition disclosed?",
                entry.finding
            );
        }
        for entry in &baseline {
            assert!(
                live.contains(entry),
                "debt entry for {} disappeared without the baseline being updated to match",
                entry.finding
            );
        }
    }
}

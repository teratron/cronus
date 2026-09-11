//! What a finished run actually means: the outcome computed from obligation
//! verdicts alone (USM-3 — a discovery can never fail a run), and the
//! attribution that keeps a harness or environment defect from being filed
//! as a product finding.

use std::fmt;

/// The five outcomes a `finish` can produce. Deliberately more than
/// pass/fail: a run that never finished deciding, or that turned out to be
/// measuring something other than the product, is a different kind of
/// result and must be reported as one rather than folded into either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    /// Every declared obligation was decided, and all passed.
    Pass,
    /// Every declared obligation was decided, and at least one failed.
    Fail,
    /// At least one declared obligation was never decided — the run's
    /// bound was exhausted, or it was interrupted, before the actor
    /// finished judging everything (USM-9).
    Incomplete,
    /// The repository's own tracked-file dirtiness changed between world
    /// build and `finish` — something wrote outside the world. Verdicts
    /// are discarded regardless of what they said (USM-12).
    Contaminated,
    /// The harness or its environment failed (a world could not be torn
    /// down, for example) — not the product. Reported `void`, and must
    /// never be filed as a product finding.
    Environment,
}

impl RunOutcome {
    /// The process exit code `cronus-sim finish` reports for this outcome.
    /// A stable, small, disjoint set: a caller scripting against this
    /// binary can branch on it without parsing the report text.
    pub fn exit_code(self) -> i32 {
        match self {
            RunOutcome::Pass => 0,
            RunOutcome::Fail => 1,
            RunOutcome::Incomplete => 2,
            RunOutcome::Contaminated => 3,
            RunOutcome::Environment => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RunOutcome::Pass => "pass",
            RunOutcome::Fail => "fail",
            RunOutcome::Incomplete => "incomplete",
            RunOutcome::Contaminated => "contaminated",
            RunOutcome::Environment => "void",
        }
    }
}

/// The report `cronus-sim finish` prints and tears its world down after
/// producing.
#[derive(Debug, Clone)]
pub struct FinishReport {
    pub world_id: String,
    pub outcome: RunOutcome,
    /// Obligation ids left undecided — always empty unless
    /// `outcome == Incomplete`.
    pub undecided_obligations: Vec<String>,
    /// Discoveries recorded by `note`, carried through for the record.
    /// Never consulted to compute `outcome` (USM-3).
    pub notes: Vec<String>,
}

impl fmt::Display for FinishReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "world: {}", self.world_id)?;
        writeln!(f, "outcome: {}", self.outcome.label())?;
        if !self.undecided_obligations.is_empty() {
            writeln!(
                f,
                "undecided obligations: {}",
                self.undecided_obligations.join(", ")
            )?;
        }
        if !self.notes.is_empty() {
            writeln!(f, "notes:")?;
            for note in &self.notes {
                writeln!(f, "  - {note}")?;
            }
        }
        Ok(())
    }
}

//! Development workflow — the five-stage agent-assisted delivery pipeline.
//!
//! Design → Plan → Execute → Review → Deliver, with a two-stage quality gate
//! (implementer pass + reviewer pass) guarding entry to Deliver, human checkpoints
//! gating stage advance where the cost of error is high, and a durable append-only
//! progress ledger recording every transition.
//!
//! The bundled skill catalog and the implementer/reviewer agent dispatch are seams;
//! the stage machine, gate, checkpoint, and ledger algebra are implemented here.

/// A pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Design,
    Plan,
    Execute,
    Review,
    Deliver,
    Delivered,
}

impl Stage {
    /// The next stage in the linear pipeline, if any.
    fn next(self) -> Option<Stage> {
        match self {
            Stage::Design => Some(Stage::Plan),
            Stage::Plan => Some(Stage::Execute),
            Stage::Execute => Some(Stage::Review),
            Stage::Review => Some(Stage::Deliver),
            Stage::Deliver => Some(Stage::Delivered),
            Stage::Delivered => None,
        }
    }
}

/// The two-stage quality gate result (implementer then reviewer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityGate {
    pub implementer_pass: bool,
    pub reviewer_pass: bool,
}

impl QualityGate {
    /// Both stages must pass for the gate to be green.
    pub fn passed(self) -> bool {
        self.implementer_pass && self.reviewer_pass
    }
}

/// Why a stage advance was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvanceError {
    /// The pipeline is already at the terminal stage.
    AlreadyDelivered,
    /// The two-stage quality gate did not pass (guards entry to Deliver).
    QualityGateFailed,
    /// A required human checkpoint was not approved.
    CheckpointNotApproved,
}

impl std::fmt::Display for AdvanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AdvanceError::AlreadyDelivered => "pipeline already delivered",
            AdvanceError::QualityGateFailed => "two-stage quality gate failed",
            AdvanceError::CheckpointNotApproved => "human checkpoint not approved",
        };
        f.write_str(s)
    }
}

impl std::error::Error for AdvanceError {}

/// One durable ledger entry recording a stage transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub from: Stage,
    pub to: Stage,
    pub at: u64,
}

/// The delivery pipeline. Holds the current stage and an append-only ledger.
#[derive(Debug)]
pub struct Pipeline {
    stage: Stage,
    ledger: Vec<LedgerEntry>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Pipeline::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            stage: Stage::Design,
            ledger: Vec::new(),
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn ledger(&self) -> &[LedgerEntry] {
        &self.ledger
    }

    /// Whether advancing from the current stage requires a human checkpoint. The
    /// high-cost transition into Deliver (shipping) gates on human approval.
    pub fn requires_checkpoint(&self) -> bool {
        self.stage == Stage::Deliver
    }

    /// Whether advancing from the current stage requires the quality gate. Entry to
    /// Deliver (Review → Deliver) is gated on the two-stage quality gate.
    pub fn requires_quality_gate(&self) -> bool {
        self.stage == Stage::Review
    }

    /// Advance to the next stage, enforcing the quality gate and human checkpoint
    /// where required. Records the transition in the ledger on success.
    pub fn advance(
        &mut self,
        gate: QualityGate,
        human_approved: bool,
        at: u64,
    ) -> Result<Stage, AdvanceError> {
        let next = self.stage.next().ok_or(AdvanceError::AlreadyDelivered)?;

        if self.requires_quality_gate() && !gate.passed() {
            return Err(AdvanceError::QualityGateFailed);
        }
        if self.requires_checkpoint() && !human_approved {
            return Err(AdvanceError::CheckpointNotApproved);
        }

        self.ledger.push(LedgerEntry {
            from: self.stage,
            to: next,
            at,
        });
        self.stage = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: QualityGate = QualityGate {
        implementer_pass: true,
        reviewer_pass: true,
    };

    #[test]
    fn two_stage_gate_requires_both_passes() {
        assert!(PASS.passed());
        assert!(
            !QualityGate {
                implementer_pass: true,
                reviewer_pass: false
            }
            .passed()
        );
        assert!(
            !QualityGate {
                implementer_pass: false,
                reviewer_pass: true
            }
            .passed()
        );
    }

    #[test]
    fn pipeline_advances_through_all_stages() {
        let mut p = Pipeline::new();
        assert_eq!(p.stage(), Stage::Design);
        assert_eq!(p.advance(PASS, false, 1).unwrap(), Stage::Plan);
        assert_eq!(p.advance(PASS, false, 2).unwrap(), Stage::Execute);
        assert_eq!(p.advance(PASS, false, 3).unwrap(), Stage::Review);
        // Review -> Deliver needs the quality gate.
        assert_eq!(p.advance(PASS, false, 4).unwrap(), Stage::Deliver);
        // Deliver -> Delivered needs a human checkpoint.
        assert_eq!(p.advance(PASS, true, 5).unwrap(), Stage::Delivered);
        assert_eq!(p.ledger().len(), 5);
    }

    #[test]
    fn quality_gate_blocks_entry_to_deliver() {
        let mut p = Pipeline::new();
        p.advance(PASS, false, 1).unwrap(); // Plan
        p.advance(PASS, false, 2).unwrap(); // Execute
        p.advance(PASS, false, 3).unwrap(); // Review
        assert!(p.requires_quality_gate());
        let failing = QualityGate {
            implementer_pass: true,
            reviewer_pass: false,
        };
        assert_eq!(
            p.advance(failing, false, 4),
            Err(AdvanceError::QualityGateFailed)
        );
        assert_eq!(p.stage(), Stage::Review); // did not advance
    }

    #[test]
    fn human_checkpoint_blocks_delivery() {
        let mut p = Pipeline::new();
        for t in 1..=4 {
            p.advance(PASS, false, t).unwrap();
        }
        assert_eq!(p.stage(), Stage::Deliver);
        assert!(p.requires_checkpoint());
        // Without human approval, delivery is refused.
        assert_eq!(
            p.advance(PASS, false, 5),
            Err(AdvanceError::CheckpointNotApproved)
        );
        assert_eq!(p.stage(), Stage::Deliver);
    }

    #[test]
    fn cannot_advance_past_delivered() {
        let mut p = Pipeline::new();
        for t in 1..=4 {
            p.advance(PASS, false, t).unwrap();
        }
        p.advance(PASS, true, 5).unwrap(); // Delivered
        assert_eq!(
            p.advance(PASS, true, 6),
            Err(AdvanceError::AlreadyDelivered)
        );
    }

    #[test]
    fn ledger_is_append_only_record() {
        let mut p = Pipeline::new();
        p.advance(PASS, false, 10).unwrap();
        p.advance(PASS, false, 20).unwrap();
        assert_eq!(p.ledger()[0].from, Stage::Design);
        assert_eq!(p.ledger()[0].to, Stage::Plan);
        assert_eq!(p.ledger()[1].at, 20);
    }
}

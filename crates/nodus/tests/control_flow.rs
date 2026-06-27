//! Integration tests for the v0.7 control-flow constructs.
//!
//! Covers the `!HALT` (fatal stop) and `!PAUSE` (suspension) action flags on a
//! conditional branch: a taken `!HALT` ends the run failed and stops later
//! steps; a taken `!PAUSE` suspends with a resume descriptor and runs no later
//! step. The not-taken case proves neither flag fires spuriously.

use nodus::{executor::Status, workflows};

// A taken branch carrying `!HALT` alongside the required escalation.
const HALT_WF: &str = r#"§wf:guard_halt v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?IF 1 > 0 → ESCALATE(human) !HALT
  2. GEN(go) → $out
"#;

// A branch whose `!HALT` is never reached (condition false).
const HALT_NOT_TAKEN_WF: &str = r#"§wf:guard_skip v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?IF 0 > 1 → ESCALATE(human) !HALT
  2. LOG(done) → $out
"#;

// A taken branch carrying `!PAUSE`; the action itself does not suspend and
// does not lock the output, so a `Paused` status proves the flag fired.
const PAUSE_WF: &str = r#"§wf:guard_pause v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?IF 1 > 0 → ANALYZE(state) !PAUSE
  2. GEN(after) → $scratch
"#;

#[test]
fn halt_branch_ends_failed_and_stops_run() {
    let result = workflows::run(HALT_WF, "guard_halt.nodus", None).expect("run");
    assert_eq!(
        result.status,
        Status::Failed,
        "a taken !HALT branch must fail the run; errors: {:?}",
        result.errors
    );
    assert!(
        !result.log.iter().any(|e| e.command == "GEN"),
        "no step after !HALT may run; log: {:?}",
        result.log
    );
}

#[test]
fn halt_not_taken_runs_to_completion() {
    let result = workflows::run(HALT_NOT_TAKEN_WF, "guard_skip.nodus", None).expect("run");
    assert_eq!(
        result.status,
        Status::Ok,
        "an untaken !HALT branch must not fail the run; errors: {:?}",
        result.errors
    );
}

#[test]
fn pause_branch_suspends_with_resume() {
    let result = workflows::run(PAUSE_WF, "guard_pause.nodus", None).expect("run");
    assert_eq!(
        result.status,
        Status::Paused,
        "a taken !PAUSE branch must suspend the run; errors: {:?}",
        result.errors
    );
    let resume = result
        .resume
        .expect("a paused run carries a resume descriptor");
    assert_eq!(resume.workflow, "wf:guard_pause");
    assert!(
        !result.log.iter().any(|e| e.command == "GEN"),
        "no step after !PAUSE may run; log: {:?}",
        result.log
    );
}

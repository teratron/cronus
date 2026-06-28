//! Integration tests for the v0.7 control-flow constructs.
//!
//! Covers the `!HALT` (fatal stop) and `!PAUSE` (suspension) action flags on a
//! conditional branch: a taken `!HALT` ends the run failed and stops later
//! steps; a taken `!PAUSE` suspends with a resume descriptor and runs no later
//! step. The not-taken case proves neither flag fires spuriously.

use nodus::{
    executor::{DialogOutcome, DialogProvider, Status},
    workflows,
};

// A dialog backend that always times out — a deterministic, repeatable per-step
// runtime error used to exercise `~RETRY:n` exhaustion.
struct TimeoutProvider;
impl DialogProvider for TimeoutProvider {
    fn ask(&self, _p: &str, _m: &[(String, String)]) -> DialogOutcome {
        DialogOutcome::Timeout
    }
    fn confirm(&self, _c: &str, _m: &[(String, String)]) -> DialogOutcome {
        DialogOutcome::Timeout
    }
}

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

// ?SWITCH fixtures — scrutinee seeded via an `@in` default.
const SWITCH_MATCH_WF: &str = r#"§wf:switch_match v1.0
§runtime: { core: schema.nodus }
@in: { category?=urgent }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $in.category:
    urgent → ANALYZE(crisis)
    spam → GEN(reply)
  ~END
"#;

const SWITCH_DEFAULT_WF: &str = r#"§wf:switch_default v1.0
§runtime: { core: schema.nodus }
@in: { category?=mystery }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $in.category:
    urgent → ANALYZE(crisis)
    * → GEN(reply)
  ~END
"#;

const SWITCH_NO_MATCH_WF: &str = r#"§wf:switch_nomatch v1.0
§runtime: { core: schema.nodus }
@in: { category?=mystery }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $in.category:
    urgent → ANALYZE(crisis)
  ~END
  2. LOG(after) → $out
"#;

#[test]
fn switch_runs_first_matching_arm() {
    let result = workflows::run(SWITCH_MATCH_WF, "switch_match.nodus", None).expect("run");
    assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);
    assert!(
        result.log.iter().any(|e| e.command == "ANALYZE"),
        "the matching arm must run; log: {:?}",
        result.log
    );
    assert!(
        !result.log.iter().any(|e| e.command == "GEN"),
        "a non-matching arm must not run; log: {:?}",
        result.log
    );
}

#[test]
fn switch_falls_through_to_default() {
    let result = workflows::run(SWITCH_DEFAULT_WF, "switch_default.nodus", None).expect("run");
    assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);
    assert!(
        result.log.iter().any(|e| e.command == "GEN"),
        "the default arm must run when nothing matches; log: {:?}",
        result.log
    );
    assert!(
        !result.log.iter().any(|e| e.command == "ANALYZE"),
        "no value arm matched, so none should run; log: {:?}",
        result.log
    );
}

#[test]
fn switch_no_match_warns_and_continues() {
    let result = workflows::run(SWITCH_NO_MATCH_WF, "switch_nomatch.nodus", None).expect("run");
    assert!(
        result.flags.iter().any(|f| f == "NODUS:SWITCH_NO_MATCH"),
        "an unmatched switch with no default must flag SWITCH_NO_MATCH; flags: {:?}",
        result.flags
    );
    assert_eq!(
        result.status,
        Status::Ok,
        "SWITCH_NO_MATCH is advisory; the run continues. errors: {:?}",
        result.errors
    );
    assert!(
        result.log.iter().any(|e| e.command == "LOG"),
        "the step after the switch must still run; log: {:?}",
        result.log
    );
}

// ~RETRY:n — a flaky step retried up to n times.
const RETRY_TIMEOUT_WF: &str = r#"§wf:retry_timeout v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ~RETRY:3 ASK(question) → $answer
  2. LOG(after) → $out
"#;

#[test]
fn retry_reruns_failing_step_up_to_bound() {
    let result = workflows::run_with_dialog(
        RETRY_TIMEOUT_WF,
        "retry_timeout.nodus",
        None,
        TimeoutProvider,
    )
    .expect("run");
    let timeouts = result
        .errors
        .iter()
        .filter(|e| e.code == "NODUS:DIALOG_TIMEOUT")
        .count();
    assert_eq!(
        timeouts, 3,
        "a ~RETRY:3 step that always fails is attempted 3 times; errors: {:?}",
        result.errors
    );
    // Exhausted retries surface as errors but do not abort the run.
    assert_eq!(
        result.status,
        Status::Partial,
        "errors: {:?}",
        result.errors
    );
    assert!(
        result.log.iter().any(|e| e.command == "LOG"),
        "the step after an exhausted retry still runs; log: {:?}",
        result.log
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

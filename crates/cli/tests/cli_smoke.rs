// End-to-end smoke tests: run the compiled `cronus` binary as a subprocess and
// check exit codes and stdout. Cargo injects CARGO_BIN_EXE_cronus at build time.
//
// Smoke coverage: one success-path test per command group.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cronus"))
}

#[test]
fn help_exits_0() {
    let status = bin()
        .arg("--help")
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "--help must exit 0");
}

#[test]
fn unknown_command_exits_2() {
    let status = bin()
        .arg("totally-unknown-cmd")
        .status()
        .expect("failed to spawn binary");
    assert_eq!(status.code(), Some(2), "unknown command must exit 2");
}

#[test]
fn workflow_validate_clean_exits_0() {
    let dir = std::env::temp_dir().join(format!("cronus-smoke-val-ok-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("smoke_ok.nodus");
    std::fs::write(
        &file,
        "§wf:smoke_ok v1.0\n\
         §runtime: { core: schema.nodus }\n\
         @in: { x }\n\
         @out: $out\n\
         @err: ESCALATE(human)\n\
         @steps:\n\
           1. GEN($in.x) → $out\n\
           2. LOG($out)\n",
    )
    .unwrap();

    let status = bin()
        .args(["workflow", "validate"])
        .arg(&file)
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "validate on valid workflow must exit 0");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn workflow_validate_error_exits_1() {
    let dir = std::env::temp_dir().join(format!("cronus-smoke-val-err-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("smoke_bad.nodus");
    std::fs::write(&file, "§wf:bad v1.0\n@steps:\n  1. GEN($in.x) → $out\n").unwrap();

    let status = bin()
        .args(["workflow", "validate"])
        .arg(&file)
        .status()
        .expect("failed to spawn binary");
    assert_eq!(
        status.code(),
        Some(1),
        "validate on invalid workflow must exit 1"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn workflow_run_clean_exits_0() {
    let dir = std::env::temp_dir().join(format!("cronus-smoke-run-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("smoke_run.nodus");
    std::fs::write(
        &file,
        "§wf:smoke_run v1.0\n\
         §runtime: { core: schema.nodus }\n\
         @in: { x }\n\
         @out: $out\n\
         @err: ESCALATE(human)\n\
         @steps:\n\
           1. GEN($in.x) → $out\n\
           2. LOG($out)\n",
    )
    .unwrap();

    let status = bin()
        .args(["workflow", "run"])
        .arg(&file)
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "run on valid workflow must exit 0");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn workflow_transpile_outputs_nonempty() {
    let dir = std::env::temp_dir().join(format!("cronus-smoke-transpile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("smoke_transpile.nodus");
    std::fs::write(
        &file,
        "§wf:smoke_transpile v1.0\n\
         §runtime: { core: schema.nodus }\n\
         @in: { x }\n\
         @out: $out\n\
         @err: ESCALATE(human)\n\
         @steps:\n\
           1. GEN($in.x) → $out\n\
           2. LOG($out)\n",
    )
    .unwrap();

    let output = bin()
        .args(["workflow", "transpile"])
        .arg(&file)
        .output()
        .expect("failed to spawn binary");
    assert!(output.status.success(), "transpile must exit 0");
    assert!(
        !output.stdout.is_empty(),
        "transpile must produce non-empty stdout"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Command smoke tests ───────────────────────────────────────────────────────

#[test]
fn role_list_presets_exits_0() {
    let status = bin()
        .args(["role", "list", "--presets"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "role list --presets must exit 0");
}

#[test]
fn board_list_exits_0() {
    let status = bin()
        .args(["board", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(
        status.success(),
        "board list must exit 0 (empty board is ok)"
    );
}

#[test]
fn schedule_list_exits_0() {
    let status = bin()
        .args(["schedule", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(
        status.success(),
        "schedule list must exit 0 (empty schedule is ok)"
    );
}

#[test]
fn budget_show_exits_0() {
    let status = bin()
        .args(["budget", "show"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "budget show must exit 0");
}

#[test]
fn exec_list_exits_0() {
    let status = bin()
        .args(["exec", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "exec list must exit 0 (empty list is ok)");
}

#[test]
fn check_run_exits_0() {
    let status = bin()
        .args(["check", "run", "smoke-card"])
        .status()
        .expect("failed to spawn binary");
    assert!(
        status.success(),
        "check run must exit 0 at the gate-runner seam"
    );
}

#[test]
fn ext_list_exits_0() {
    let status = bin()
        .args(["ext", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(
        status.success(),
        "ext list must exit 0 (empty registry is ok)"
    );
}

#[test]
fn learn_list_exits_0() {
    let status = bin()
        .args(["learn", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(
        status.success(),
        "learn list must exit 0 (no pending proposals is ok)"
    );
}

#[test]
fn registry_list_exits_0() {
    let output = bin()
        .args(["registry", "list"])
        .output()
        .expect("failed to spawn binary");
    assert!(output.status.success(), "registry list must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("work") || stdout.contains("code"),
        "registry list must include built-in agent names"
    );
}

// ── Additional smoke tests ───────────────────────────────────────────────────────

#[test]
fn goal_help_exits_0() {
    let status = bin()
        .args(["goal", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "goal --help must exit 0");
}

#[test]
fn goal_start_exits_0() {
    let status = bin()
        .args(["goal", "start", "complete the build system"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "goal start must exit 0");
}

#[test]
fn trigger_help_exits_0() {
    let status = bin()
        .args(["trigger", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "trigger --help must exit 0");
}

#[test]
fn trigger_list_exits_0() {
    let status = bin()
        .args(["trigger", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "trigger list must exit 0");
}

#[test]
fn mission_help_exits_0() {
    let status = bin()
        .args(["mission", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "mission --help must exit 0");
}

#[test]
fn mission_start_exits_0() {
    let status = bin()
        .args(["mission", "start", "build the feature"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "mission start must exit 0");
}

#[test]
fn mission_list_exits_0() {
    let status = bin()
        .args(["mission", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "mission list must exit 0");
}

#[test]
fn research_help_exits_0() {
    let status = bin()
        .args(["research", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "research --help must exit 0");
}

#[test]
fn research_start_exits_0() {
    let status = bin()
        .args(["research", "start", "What is Rust ownership?"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "research start must exit 0");
}

#[test]
fn research_list_exits_0() {
    let status = bin()
        .args(["research", "list"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "research list must exit 0");
}

#[test]
fn change_help_exits_0() {
    let status = bin()
        .args(["change", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "change --help must exit 0");
}

#[test]
fn change_graph_exits_0() {
    let status = bin()
        .args(["change", "graph"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "change graph must exit 0");
}

#[test]
fn change_next_exits_0() {
    let status = bin()
        .args(["change", "next"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "change next must exit 0");
}

#[test]
fn activation_help_exits_0() {
    let status = bin()
        .args(["activation", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "activation --help must exit 0");
}

#[test]
fn activation_enable_without_acknowledgement_refuses_when_noninteractive() {
    // `status`/`observe` reads the real OS (read-only, harmless); `enable`
    // mutates real activation state, so this test never lets it proceed —
    // stdin is explicitly nulled (deterministically non-interactive
    // regardless of how the test runner itself was invoked), so the BA-5
    // gate must refuse before `default_activation_registry()` is ever
    // touched.
    use std::process::Stdio;
    let output = bin()
        .args(["activation", "enable", "--mode", "login"])
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn binary");
    assert_eq!(
        output.status.code(),
        Some(1),
        "a non-interactive enable without the acknowledgement flag must refuse"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("acknowledge-unattended-execution"),
        "the refusal must name the flag that would unblock it"
    );
}

#[test]
fn loop_help_exits_0() {
    let status = bin()
        .args(["loop", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "loop --help must exit 0");
}

#[test]
fn loop_run_over_a_file_that_already_exists_reaches_done_and_its_ledger_is_inspectable() {
    // A real end-to-end execution loop: the target file is pre-created, so
    // the real FileExistsBackend's oracle reports done on the first
    // iteration. This proves the CLI -> facade -> domain wiring is real,
    // not mocked — the same compiled binary an operator would run.
    let dir = std::env::temp_dir().join(format!("cronus-smoke-loop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("marker");
    std::fs::write(&target, b"present").unwrap();

    let output = bin()
        .args(["loop", "run", "--file"])
        .arg(&target)
        .output()
        .expect("failed to spawn binary");
    assert!(
        output.status.success(),
        "a loop run over an already-existing file must reach Done"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("done"), "stdout: {stdout}");

    // Extract the run id the command printed, then prove `log`/`show` read
    // back the SAME persisted state through the CLI/library-shared facade
    // call (INV-3 parity) — not divergent logic.
    let run_id = stdout
        .trim()
        .strip_prefix("loop ")
        .and_then(|s| s.split(':').next())
        .expect("run output names the run id")
        .to_string();

    let log_output = bin()
        .args(["loop", "log", &run_id])
        .output()
        .expect("failed to spawn binary");
    assert!(log_output.status.success(), "loop log must exit 0");
    let log_text = String::from_utf8_lossy(&log_output.stdout);
    assert!(
        log_text.contains("OUTCOME") && log_text.contains("Done"),
        "log: {log_text}"
    );

    let show_output = bin()
        .args(["loop", "show", &run_id])
        .output()
        .expect("failed to spawn binary");
    assert!(show_output.status.success(), "loop show must exit 0");
    let show_text = String::from_utf8_lossy(&show_output.stdout);
    assert!(show_text.contains("Execution"), "show: {show_text}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn loop_run_over_a_file_that_never_appears_stops_at_the_ceiling() {
    let dir = std::env::temp_dir().join(format!("cronus-smoke-loop-stop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("never-created");

    let status = bin()
        .args(["loop", "run", "--file"])
        .arg(&target)
        .status()
        .expect("failed to spawn binary");
    assert!(
        !status.success(),
        "a loop run whose file never appears must stop, not silently succeed"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn loop_evolve_is_marked_unavailable_not_a_silent_success() {
    // INV-9 shipped-surface honesty: no harness registry exists yet, so
    // this must refuse clearly rather than fake a result.
    let output = bin()
        .args(["loop", "evolve", "some-harness"])
        .output()
        .expect("failed to spawn binary");
    assert!(
        !output.status.success(),
        "evolve must not report success for an unbound capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unavailable"), "stderr: {stderr}");
}

#[test]
fn archetype_help_exits_0() {
    let status = bin()
        .args(["archetype", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "archetype --help must exit 0");
}

#[test]
fn archetype_list_and_info_show_the_shipped_and_blocked_archetypes() {
    // A real end-to-end read through the domain catalog via the compiled
    // binary — the same the operator runs.
    let list = bin()
        .args(["archetype", "list", "--catalog"])
        .output()
        .expect("failed to spawn binary");
    assert!(list.status.success(), "archetype list must exit 0");
    let list_out = String::from_utf8_lossy(&list.stdout);
    assert!(
        list_out.contains("software-engineering"),
        "list: {list_out}"
    );
    assert!(list_out.contains("advertising-agency"), "list: {list_out}");

    let info = bin()
        .args(["archetype", "info", "software-engineering"])
        .output()
        .expect("failed to spawn binary");
    assert!(info.status.success(), "archetype info must exit 0");
    let info_out = String::from_utf8_lossy(&info.stdout);
    assert!(info_out.contains("pool (18)"), "info: {info_out}");
    assert!(info_out.contains("seed: 0 role"), "info: {info_out}");

    // A blocked archetype reports blocked + its missing roles.
    let blocked = bin()
        .args(["archetype", "info", "finance-department"])
        .output()
        .expect("failed to spawn binary");
    assert!(
        blocked.status.success(),
        "info on a blocked archetype exits 0"
    );
    let blocked_out = String::from_utf8_lossy(&blocked.stdout);
    assert!(blocked_out.contains("BLOCKED"), "blocked: {blocked_out}");
    assert!(blocked_out.contains("controller"), "blocked: {blocked_out}");
}

#[test]
fn archetype_set_then_clear_both_exit_0() {
    let set = bin()
        .args(["archetype", "set", "software-engineering"])
        .status()
        .expect("failed to spawn binary");
    assert!(set.success(), "archetype set must exit 0");

    let clear = bin()
        .args(["archetype", "set", "--clear"])
        .status()
        .expect("failed to spawn binary");
    assert!(clear.success(), "archetype set --clear must exit 0");
}

#[test]
fn there_is_no_archetype_hire_subcommand() {
    // OA / §4.8: hiring belongs to `role` and the manager, never to an
    // archetype command — an `archetype hire` verb would put the prior on the
    // wrong side of the decision boundary. clap must reject it.
    let output = bin()
        .args(["archetype", "hire", "architect"])
        .output()
        .expect("failed to spawn binary");
    assert!(
        !output.status.success(),
        "there must be no `archetype hire` subcommand"
    );
}

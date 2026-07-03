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

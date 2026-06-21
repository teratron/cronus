// End-to-end smoke tests: run the compiled `cronus` binary as a subprocess and
// check exit codes and stdout. Cargo injects CARGO_BIN_EXE_cronus at build time.

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

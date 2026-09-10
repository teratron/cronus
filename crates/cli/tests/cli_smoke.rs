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

/// A command group named with no verb is a *usage* failure — clap answers it
/// with the group's help and exit code 2, exactly like an unknown verb — never
/// an application failure exit 1, and never the "error: internal: …" message
/// that both halves of the launcher used to print. Covers a semantic group, a
/// multi-verb installation group, and the one sub-nested installation group.
#[test]
fn a_group_with_no_verb_is_a_usage_failure_not_an_internal_error() {
    for args in [
        &["memory"][..],
        &["board"][..],
        &["workspace"][..],
        &["registry"][..],
        &["ext"][..],
        &["ext", "skill"][..],
    ] {
        let output = bin().args(args).output().expect("failed to spawn binary");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(2),
            "`cronus {}` must exit 2 (usage), got {:?} — stderr: {stderr}",
            args.join(" "),
            output.status.code()
        );
        assert!(
            !stderr.contains("error: internal:"),
            "`cronus {}` must not print an internal error: {stderr}",
            args.join(" ")
        );
        assert!(
            stderr.contains("Usage:"),
            "`cronus {}` must print a usage line: {stderr}",
            args.join(" ")
        );
    }
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

/// `workflow scaffold` writes the file the caller named. Passing a `.nodus`
/// path used to append a second `.nodus`, so the reported path and a later
/// `workflow validate <path>` disagreed. Now `scaffold X.nodus` writes exactly
/// `X.nodus` and that file validates clean.
#[test]
fn workflow_scaffold_writes_the_named_path_and_it_validates() {
    let dir = std::env::temp_dir().join(format!("cronus-smoke-scaffold-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("my_flow.nodus");

    let scaffold = bin()
        .args(["workflow", "scaffold"])
        .arg(&file)
        .output()
        .expect("failed to spawn binary");
    assert!(scaffold.status.success(), "scaffold must exit 0");
    assert!(
        file.is_file(),
        "scaffold must write exactly the named file, not <name>.nodus.nodus"
    );
    assert!(
        !dir.join("my_flow.nodus.nodus").exists(),
        "scaffold must not double-append the extension"
    );

    let validate = bin()
        .args(["workflow", "validate"])
        .arg(&file)
        .status()
        .expect("failed to spawn binary");
    assert!(
        validate.success(),
        "the scaffolded file must validate clean at its own path"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// `board move` names the valid states when the one given is not recognised,
/// so the vocabulary is discoverable without reading source.
#[test]
fn board_move_with_an_unknown_state_lists_the_valid_ones() {
    let output = bin()
        .args(["board", "move", "no-such-card", "sideways"])
        .output()
        .expect("failed to spawn binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("valid:") && stderr.contains("triage") && stderr.contains("done"),
        "the error must enumerate the valid states: {stderr}"
    );
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
fn activation_help_exits_0() {
    let status = bin()
        .args(["activation", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "activation --help must exit 0");
}

/// `tui` is answerable pre-composition like every other installation verb
/// (LH-1/LH-5) — `--help` never actually launches the interactive session,
/// which would block a subprocess test on a raw-mode terminal it does not
/// have.
#[test]
fn tui_help_exits_0() {
    let status = bin()
        .args(["tui", "--help"])
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "tui --help must exit 0");
}

/// The retired standalone `cronus-tui` executable's replacement is
/// discoverable from the one binary's own top-level listing (l2-tui.md
/// §4.4).
#[test]
fn top_level_help_lists_tui() {
    let output = bin()
        .arg("--help")
        .output()
        .expect("failed to spawn binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("tui"),
        "the top-level --help listing must name the tui verb: {stdout}"
    );
}

/// Every flat installation verb (its own id-tail equal to its group name)
/// must reach its real handler through `dispatch`, never fall into
/// `dispatch`'s "matched with no verb subcommand" internal-error branch —
/// a real, latent bug this task's own manual verification of `cronus tui`
/// caught: `dispatch` used to unconditionally expect a nested subcommand,
/// which no flat group's own matches ever have, so every one of them
/// (`status`, `doctor`, `tui`, …) failed the moment it was actually run
/// rather than merely `--help`'d. Deliberately real subprocess calls with
/// no `--fix`/mutating flag, so this is safe to run against whatever real
/// state this host happens to have — the property checked is independent
/// of that state.
#[test]
fn every_flat_installation_verb_reaches_its_real_handler_not_an_internal_routing_error() {
    for verb in ["status", "doctor"] {
        let output = bin().arg(verb).output().expect("failed to spawn binary");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("matched with no verb subcommand"),
            "{verb}: a flat group's own matches must resolve directly, not assume a nested \
             subcommand exists: {stderr}"
        );
    }
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

#[test]
fn workspace_create_refuses_the_reserved_dev_office_id() {
    // DVO-1: `WorkspaceKind::Developer` is not creatable through the
    // ordinary project-creation flow — the reserved id is refused before
    // any real workspace row is written.
    let output = bin()
        .args(["workspace", "create", "dev-office"])
        .output()
        .expect("failed to spawn binary");
    assert!(
        !output.status.success(),
        "creating the reserved 'dev-office' workspace id must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("reserved"), "stderr: {stderr}");
}

#[test]
fn dev_status_admit_revoke_round_trip_through_the_real_gate() {
    // Real end-to-end smoke: this test binary's own cwd IS a genuine
    // checkout of the canonical repo (the workspace this test runs from),
    // so `repo_authenticity` resolves `Genuine` for real — no fixture. The
    // sequence ends on `revoke`, restoring the on-disk admission record to
    // its default `absent` state (the `knowledge` smoke test's own
    // clean-up-what-you-touch precedent for real `%APPDATA%` side effects).
    let status_before = bin()
        .args(["dev", "status"])
        .output()
        .expect("failed to spawn binary");
    assert!(status_before.status.success(), "dev status must exit 0");

    let admit = bin()
        .args(["dev", "admit"])
        .output()
        .expect("failed to spawn binary");
    assert!(admit.status.success(), "dev admit must exit 0");

    let status_elevated = bin()
        .args(["dev", "status"])
        .output()
        .expect("failed to spawn binary");
    assert!(status_elevated.status.success());
    let stdout = String::from_utf8_lossy(&status_elevated.stdout);
    assert!(
        stdout.contains("elevated"),
        "genuine repo + admitted must resolve elevated, got: {stdout}"
    );

    let revoke = bin()
        .args(["dev", "revoke"])
        .output()
        .expect("failed to spawn binary");
    assert!(revoke.status.success(), "dev revoke must exit 0");

    let status_after = bin()
        .args(["dev", "status"])
        .output()
        .expect("failed to spawn binary");
    assert!(status_after.status.success());
    let stdout = String::from_utf8_lossy(&status_after.stdout);
    assert!(
        stdout.contains("absent"),
        "revoked admission must resolve absent again, got: {stdout}"
    );
}

#[test]
fn workspace_delete_refuses_the_reserved_dev_office_id() {
    // DVO-1: non-deletable through the ordinary flow, symmetric with create.
    let output = bin()
        .args(["workspace", "delete", "dev-office"])
        .output()
        .expect("failed to spawn binary");
    assert!(
        !output.status.success(),
        "deleting the reserved 'dev-office' workspace id must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("reserved"), "stderr: {stderr}");
}

/// `--format json` is honoured across the installation half too — several
/// verbs used to print prose regardless. Each output line here must parse as
/// JSON (checked structurally: starts with `{` or `[`, balanced, no bare
/// backslash outside a `\` escape).
#[test]
fn installation_verbs_emit_valid_json_for_the_json_format() {
    fn looks_like_json(s: &str) -> bool {
        let t = s.trim();
        if !(t.starts_with('{') || t.starts_with('[')) {
            return false;
        }
        // every backslash must be part of a recognised escape
        let bytes: Vec<char> = t.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == '\\' {
                match bytes.get(i + 1) {
                    Some('"') | Some('\\') | Some('/') | Some('n') | Some('r') | Some('t')
                    | Some('b') | Some('f') | Some('u') => i += 2,
                    _ => return false,
                }
            } else {
                i += 1;
            }
        }
        true
    }

    for args in [
        &["archetype", "list", "--format", "json"][..],
        &["archetype", "list", "--active", "--format", "json"][..],
        &["backup", "list", "--format", "json"][..],
        &["dev", "status", "--format", "json"][..],
    ] {
        let output = bin().args(args).output().expect("failed to spawn binary");
        assert!(
            output.status.success(),
            "`cronus {}` must exit 0",
            args.join(" ")
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
            assert!(
                looks_like_json(line),
                "`cronus {}` produced a non-JSON / badly-escaped line: {line:?}",
                args.join(" ")
            );
        }
    }
}

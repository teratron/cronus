//! `cronus-sim` — the only path by which a simulated run touches the
//! product. Every subcommand is a separate process invocation, on purpose:
//! an agent drives this the way it drives any other shell command, one
//! call at a time, and the record exists in [`cronus_simulation::run_state`]
//! whether or not the agent chooses to narrate what happened.
//!
//! ```text
//! cronus-sim world new [--bin <path>] <scenario-file>   -> prints a world id
//! cronus-sim run <world-id> -- <argv...>                 -> spawns the product
//! cronus-sim note <world-id> <text...>                   -> records a discovery
//! cronus-sim spend <world-id> <usd>                      -> reports the driving
//!                                                            agent's own incremental
//!                                                            cost since its last report
//! cronus-sim verdict <world-id> <obligation-id> <pass|fail> [--cite i,j,...]
//! cronus-sim finish <world-id>                           -> pass/fail/incomplete/
//!                                                            contaminated/void
//! cronus-sim pin <world-id> <name>                       -> distils recorded
//!                                                            entries into a
//!                                                            checked-in replay case
//! cronus-sim coverage [--bin <path>]                     -> catalog complement
//!                                                            against simulations/
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

use cronus_simulation::coverage;
use cronus_simulation::product;
use cronus_simulation::run_state::{RunState, RunStateError};
use cronus_simulation::scenario;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match dispatch(&args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("cronus-sim: {err}");
            5
        }
    };
    std::process::exit(code);
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(msg) => write!(f, "usage error: {msg}"),
            CliError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<RunStateError> for CliError {
    fn from(err: RunStateError) -> Self {
        CliError::Other(err.to_string())
    }
}

fn dispatch(args: &[String]) -> Result<i32, CliError> {
    match args.first().map(String::as_str) {
        Some("world") => cmd_world(&args[1..]),
        Some("run") => cmd_run(&args[1..]),
        Some("note") => cmd_note(&args[1..]),
        Some("spend") => cmd_spend(&args[1..]),
        Some("verdict") => cmd_verdict(&args[1..]),
        Some("finish") => cmd_finish(&args[1..]),
        Some("pin") => cmd_pin(&args[1..]),
        Some("coverage") => cmd_coverage(&args[1..]),
        Some(other) => Err(CliError::Usage(format!("unknown subcommand `{other}`"))),
        None => Err(CliError::Usage(
            "a subcommand is required: world | run | note | spend | verdict | finish | pin | coverage"
                .to_string(),
        )),
    }
}

fn cmd_world(args: &[String]) -> Result<i32, CliError> {
    if args.first().map(String::as_str) != Some("new") {
        return Err(CliError::Usage(
            "usage: world new [--bin <path>] <scenario-file>".to_string(),
        ));
    }
    let rest = &args[1..];
    let mut bin_override: Option<PathBuf> = None;
    let mut positional = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--bin" {
            i += 1;
            let path = rest
                .get(i)
                .ok_or_else(|| CliError::Usage("--bin requires a path".to_string()))?;
            bin_override = Some(PathBuf::from(path));
        } else {
            positional.push(rest[i].clone());
        }
        i += 1;
    }

    let scenario_path = positional
        .first()
        .ok_or_else(|| CliError::Usage("world new requires a scenario file path".to_string()))?;
    let text = std::fs::read_to_string(scenario_path)
        .map_err(|err| CliError::Other(format!("failed to read {scenario_path}: {err}")))?;
    let scenario = scenario::parse(&text)
        .map_err(|err| CliError::Other(format!("failed to parse scenario: {err}")))?;

    let resolved_binary = match bin_override {
        Some(path) => path,
        None => product::resolve_binary("cronus")
            .ok_or_else(|| CliError::Other("could not resolve the `cronus` binary".to_string()))?,
    };

    let state = RunState::create(resolved_binary, &scenario)?;
    println!("{}", state.world_id);
    Ok(0)
}

fn cmd_run(args: &[String]) -> Result<i32, CliError> {
    let world_id = args
        .first()
        .ok_or_else(|| CliError::Usage("usage: run <world-id> -- <argv...>".to_string()))?;
    let sep = args.iter().position(|a| a == "--").ok_or_else(|| {
        CliError::Usage("run requires a `--` separator before the product argv".to_string())
    })?;
    let argv: Vec<String> = args[sep + 1..].to_vec();

    let mut state = RunState::load(world_id)?;
    match state.record_run(&argv) {
        Ok(idx) => {
            let entry = state.entry(idx).expect("just-recorded entry must exist");
            print!("{}", entry.stdout);
            eprint!("{}", entry.stderr);
            eprintln!("cronus-sim: entry={idx} exit={:?}", entry.exit_code);
            Ok(entry.exit_code.unwrap_or(-1))
        }
        Err(RunStateError::BoundExhausted { .. }) => {
            eprintln!("cronus-sim: run refused — this world's bound is exhausted; call finish");
            Ok(6)
        }
        Err(other) => Err(CliError::Other(other.to_string())),
    }
}

fn cmd_note(args: &[String]) -> Result<i32, CliError> {
    let world_id = args
        .first()
        .ok_or_else(|| CliError::Usage("usage: note <world-id> <text...>".to_string()))?;
    let text = args[1..].join(" ");
    if text.trim().is_empty() {
        return Err(CliError::Usage("note requires non-empty text".to_string()));
    }
    let mut state = RunState::load(world_id)?;
    state.add_note(text)?;
    Ok(0)
}

fn cmd_spend(args: &[String]) -> Result<i32, CliError> {
    let world_id = args
        .first()
        .ok_or_else(|| CliError::Usage("usage: spend <world-id> <usd>".to_string()))?;
    let usd_str = args
        .get(1)
        .ok_or_else(|| CliError::Usage("spend requires a dollar amount".to_string()))?;
    let usd: f64 = usd_str
        .parse()
        .map_err(|_| CliError::Usage(format!("spend amount must be a number, got `{usd_str}`")))?;
    if usd < 0.0 {
        return Err(CliError::Usage(
            "spend amount must not be negative".to_string(),
        ));
    }

    let mut state = RunState::load(world_id)?;
    state.record_spend(usd)?;
    Ok(0)
}

fn cmd_verdict(args: &[String]) -> Result<i32, CliError> {
    let world_id = args.first().ok_or_else(|| {
        CliError::Usage(
            "usage: verdict <world-id> <obligation-id> <pass|fail> [--cite i,j,...]".to_string(),
        )
    })?;
    let obligation_id = args
        .get(1)
        .ok_or_else(|| CliError::Usage("verdict requires an obligation id".to_string()))?;
    let verdict_word = args
        .get(2)
        .ok_or_else(|| CliError::Usage("verdict requires `pass` or `fail`".to_string()))?;
    let passed = match verdict_word.as_str() {
        "pass" => true,
        "fail" => false,
        other => {
            return Err(CliError::Usage(format!(
                "verdict must be `pass` or `fail`, got `{other}`"
            )));
        }
    };

    let mut cites = Vec::new();
    if let Some(pos) = args.iter().position(|a| a == "--cite")
        && let Some(list) = args.get(pos + 1)
    {
        for part in list.split(',') {
            let idx: usize = part.trim().parse().map_err(|_| {
                CliError::Usage(format!("--cite entries must be integers, got `{part}`"))
            })?;
            cites.push(idx);
        }
    }

    let mut state = RunState::load(world_id)?;
    state.record_verdict(obligation_id, passed, cites)?;
    Ok(0)
}

fn cmd_finish(args: &[String]) -> Result<i32, CliError> {
    let world_id = args
        .first()
        .ok_or_else(|| CliError::Usage("usage: finish <world-id>".to_string()))?;
    let state = RunState::load(world_id)?;
    let report = state.finish()?;
    print!("{report}");
    Ok(report.outcome.exit_code())
}

fn cmd_pin(args: &[String]) -> Result<i32, CliError> {
    let world_id = args
        .first()
        .ok_or_else(|| CliError::Usage("usage: pin <world-id> <name>".to_string()))?;
    let name = args
        .get(1)
        .ok_or_else(|| CliError::Usage("pin requires a name".to_string()))?;
    let state = RunState::load(world_id)?;

    let dest = pin_to(&state, name)?;
    println!("{}", dest.display());
    Ok(0)
}

/// Distil `state`'s recorded entries into a checked-in replay case at
/// `<this crate>/tests/replays/<name>.toml` — the same directory
/// `tests/replays.rs` already scans, so a pinned finding joins the
/// always-on guard on the next `cargo test`. Exposed as a free function
/// (rather than inlined in `cmd_pin`) so it is unit-testable without a
/// subprocess.
fn pin_to(state: &RunState, name: &str) -> Result<PathBuf, CliError> {
    let mut toml_text = format!("id = \"{}\"\n\n", escape(name));
    for entry in &state.entries {
        let argv_toml = entry
            .argv
            .iter()
            .map(|a| format!("\"{}\"", escape(a)))
            .collect::<Vec<_>>()
            .join(", ");
        toml_text += &format!(
            "[[step]]\nargv = [{argv_toml}]\nexpect_exit = {}\n\n",
            entry.exit_code.unwrap_or(-1)
        );
    }

    let dest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/replays");
    std::fs::create_dir_all(&dest_dir)
        .map_err(|err| CliError::Other(format!("failed to create {dest_dir:?}: {err}")))?;
    let dest = dest_dir.join(format!("{name}.toml"));
    std::fs::write(&dest, toml_text)
        .map_err(|err| CliError::Other(format!("failed to write {dest:?}: {err}")))?;
    Ok(dest)
}

fn cmd_coverage(args: &[String]) -> Result<i32, CliError> {
    let mut bin_override: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--bin" {
            i += 1;
            let path = args
                .get(i)
                .ok_or_else(|| CliError::Usage("--bin requires a path".to_string()))?;
            bin_override = Some(PathBuf::from(path));
        }
        i += 1;
    }

    let resolved_binary = match bin_override {
        Some(path) => path,
        None => product::resolve_binary("cronus")
            .ok_or_else(|| CliError::Other("could not resolve the `cronus` binary".to_string()))?,
    };

    let catalog = coverage::catalog_from_completion(&resolved_binary)
        .map_err(|err| CliError::Other(err.to_string()))?;
    let dir = coverage::default_simulations_dir();
    let scenarios = coverage::load_corpus(&dir).map_err(|err| CliError::Other(err.to_string()))?;

    let covers: Vec<&str> = scenarios
        .iter()
        .flat_map(|s| s.covers.iter().map(String::as_str))
        .collect();
    let report = coverage::CoverageReport::compute(&catalog, covers);
    print!("{report}");
    Ok(0)
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cronus_simulation::run_state::EntryState;
    use std::collections::BTreeMap;

    fn sample_state() -> RunState {
        RunState {
            world_id: "pin-test-world".to_string(),
            root: std::env::temp_dir().join("cronus-sim-pin-test-world"),
            resolved_binary: PathBuf::from("cronus"),
            product_version: "0.0.0".to_string(),
            repo_dirty_digest_at_build: None,
            bound_steps: 10,
            bound_wall_secs: 60,
            bound_spend_usd: 5.0,
            created_at_unix_ms: 0,
            obligation_ids: Vec::new(),
            entries: vec![
                EntryState {
                    argv: vec!["--help".to_string()],
                    stdout: "help text".to_string(),
                    stderr: String::new(),
                    exit_code: Some(0),
                    duration_ms: 1,
                    state_digest_before: 1,
                    state_digest_after: 1,
                },
                EntryState {
                    argv: vec!["totally-unknown-cmd".to_string()],
                    stdout: String::new(),
                    stderr: "error".to_string(),
                    exit_code: Some(2),
                    duration_ms: 1,
                    state_digest_before: 1,
                    state_digest_after: 1,
                },
            ],
            verdicts: BTreeMap::new(),
            notes: Vec::new(),
            spend_usd_used: 0.0,
        }
    }

    #[test]
    fn pin_writes_a_replay_case_that_parses_and_matches_the_recorded_exits() {
        let state = sample_state();
        let name = "pin-unit-test-do-not-commit";
        let dest = pin_to(&state, name).expect("pin must succeed");

        let text = std::fs::read_to_string(&dest).expect("pinned file must be readable");
        let case = cronus_simulation::replay::parse(&text).expect("pinned file must parse");

        assert_eq!(case.id, name);
        assert_eq!(case.steps.len(), 2);
        assert_eq!(case.steps[0].argv, vec!["--help"]);
        assert_eq!(case.steps[0].expect_exit, 0);
        assert_eq!(case.steps[1].argv, vec!["totally-unknown-cmd"]);
        assert_eq!(case.steps[1].expect_exit, 2);

        // Clean up: this is a test artifact, not a real finding, and must
        // not join the always-on replay lane it just proved it could join.
        let _ = std::fs::remove_file(&dest);
    }
}

//! Cross-process state for one `cronus-sim` run.
//!
//! `world`, `run`, `note`, `verdict`, and `finish` are separate CLI
//! invocations — that is the entire point of the wrapper (§4.3 of
//! `l2-simulation-suite`): an agent issues them one at a time, the way it
//! issues any other shell command. Everything they need to share therefore
//! lives in one JSON file inside the world's own root, so it disappears
//! with the world on teardown rather than lingering.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::findings::{FinishReport, RunOutcome};
use crate::scenario::Scenario;
use crate::world::World;

const STATE_FILE_NAME: &str = "run-state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryState {
    pub argv: Vec<String>,
    /// Lossy UTF-8 — the cross-process persisted form trades exact-byte
    /// fidelity (which [`crate::transcript::Transcript`] keeps, in-process)
    /// for a value that survives a plain JSON round trip. Every scenario
    /// this harness drives emits well-formed text; a future binary output
    /// class would need a different field, not a silent truncation here.
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub state_digest_before: u64,
    pub state_digest_after: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictState {
    pub passed: bool,
    pub cites: Vec<usize>,
}

/// Persisted run state — one JSON file per world.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunState {
    pub world_id: String,
    pub root: PathBuf,
    pub resolved_binary: PathBuf,
    pub product_version: String,
    /// Captured once, at `create()` time. The baseline `finish` compares a
    /// fresh recomputation against to detect contamination (USM-12).
    pub repo_dirty_digest_at_build: Option<String>,
    pub bound_steps: u32,
    pub bound_wall_secs: u32,
    pub created_at_unix_ms: u128,
    pub obligation_ids: Vec<String>,
    #[serde(default)]
    pub entries: Vec<EntryState>,
    #[serde(default)]
    pub verdicts: BTreeMap<String, VerdictState>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug)]
pub enum RunStateError {
    World(crate::world::WorldError),
    Io(std::io::Error),
    Serde(String),
    UnknownWorld(String),
    UnknownObligation(String),
    UnknownEntry(usize),
    BoundExhausted {
        entries_used: u32,
        bound_steps: u32,
        wall_elapsed_secs: u64,
        bound_wall_secs: u32,
    },
}

impl fmt::Display for RunStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunStateError::World(err) => write!(f, "{err}"),
            RunStateError::Io(err) => write!(f, "run-state I/O error: {err}"),
            RunStateError::Serde(msg) => write!(f, "run-state serialization error: {msg}"),
            RunStateError::UnknownWorld(id) => {
                write!(f, "no such world `{id}` — was it built with `world new`?")
            }
            RunStateError::UnknownObligation(id) => {
                write!(
                    f,
                    "obligation `{id}` was not declared by this world's scenario"
                )
            }
            RunStateError::UnknownEntry(idx) => {
                write!(f, "cited transcript entry {idx} does not exist")
            }
            RunStateError::BoundExhausted {
                entries_used,
                bound_steps,
                wall_elapsed_secs,
                bound_wall_secs,
            } => write!(
                f,
                "this world's bound is exhausted ({entries_used}/{bound_steps} steps, \
                 {wall_elapsed_secs}/{bound_wall_secs}s elapsed) — call finish"
            ),
        }
    }
}

impl std::error::Error for RunStateError {}

impl RunState {
    fn state_path(root: &Path) -> PathBuf {
        root.join(STATE_FILE_NAME)
    }

    /// Build a fresh world for `scenario` and persist its initial state.
    pub fn create(resolved_binary: PathBuf, scenario: &Scenario) -> Result<Self, RunStateError> {
        let world = World::build(&scenario.id).map_err(RunStateError::World)?;
        let state = RunState {
            world_id: world.id().to_string(),
            root: world.root().to_path_buf(),
            resolved_binary,
            product_version: world.product_version.to_string(),
            repo_dirty_digest_at_build: world.repo_dirty_digest.clone(),
            bound_steps: scenario.bound.steps,
            bound_wall_secs: scenario.bound.wall_secs,
            created_at_unix_ms: now_unix_ms(),
            obligation_ids: scenario.obligations.iter().map(|o| o.id.clone()).collect(),
            entries: Vec::new(),
            verdicts: BTreeMap::new(),
            notes: Vec::new(),
        };
        // `world` owns the directory it just created; letting it fall out
        // of scope here does NOT delete anything — teardown is an explicit
        // consuming method, never a Drop impl. The directory must survive
        // this call, since the whole point is a later, separate process
        // finding it again by id.
        state.save()?;
        Ok(state)
    }

    pub fn load(world_id: &str) -> Result<Self, RunStateError> {
        let root = World::root_for_id(world_id);
        let path = Self::state_path(&root);
        let text = std::fs::read_to_string(&path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                RunStateError::UnknownWorld(world_id.to_string())
            } else {
                RunStateError::Io(err)
            }
        })?;
        serde_json::from_str(&text).map_err(|err| RunStateError::Serde(err.to_string()))
    }

    fn save(&self) -> Result<(), RunStateError> {
        let text = serde_json::to_string_pretty(self)
            .map_err(|err| RunStateError::Serde(err.to_string()))?;
        std::fs::write(Self::state_path(&self.root), text).map_err(RunStateError::Io)
    }

    fn attached(&self) -> World {
        World::attach(self.root.clone())
    }

    fn wall_elapsed_secs(&self) -> u64 {
        let now = now_unix_ms();
        (now.saturating_sub(self.created_at_unix_ms) / 1000) as u64
    }

    /// Spawn `argv` inside this world and append the resulting entry.
    /// Refuses (without spawning anything) once the scenario's declared
    /// bound is exhausted — the mechanism that stops a stuck actor from
    /// spending without limit on a goal the product cannot satisfy.
    pub fn record_run(&mut self, argv: &[String]) -> Result<usize, RunStateError> {
        if self.entries.len() as u32 >= self.bound_steps
            || self.wall_elapsed_secs() >= u64::from(self.bound_wall_secs)
        {
            return Err(RunStateError::BoundExhausted {
                entries_used: self.entries.len() as u32,
                bound_steps: self.bound_steps,
                wall_elapsed_secs: self.wall_elapsed_secs(),
                bound_wall_secs: self.bound_wall_secs,
            });
        }

        let world = self.attached();
        let digest_before = world.state_digest().map_err(RunStateError::Io)?;
        let started = Instant::now();

        let child = Command::new(&self.resolved_binary)
            .args(argv)
            .current_dir(world.cwd())
            .env_clear()
            .envs(world.child_env(std::env::vars()))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(RunStateError::Io)?;
        let output = child.wait_with_output().map_err(RunStateError::Io)?;
        let duration_ms = started.elapsed().as_millis() as u64;
        let digest_after = world.state_digest().map_err(RunStateError::Io)?;

        let entry = EntryState {
            argv: argv.to_vec(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
            duration_ms,
            state_digest_before: digest_before,
            state_digest_after: digest_after,
        };
        self.entries.push(entry);
        let idx = self.entries.len() - 1;
        self.save()?;
        Ok(idx)
    }

    pub fn entry(&self, index: usize) -> Option<&EntryState> {
        self.entries.get(index)
    }

    /// Append a discovery. Never consulted when computing an outcome
    /// (USM-3) — a discovery is information, not an obligation.
    pub fn add_note(&mut self, text: String) -> Result<(), RunStateError> {
        self.notes.push(text);
        self.save()
    }

    pub fn record_verdict(
        &mut self,
        obligation_id: &str,
        passed: bool,
        cites: Vec<usize>,
    ) -> Result<(), RunStateError> {
        if !self.obligation_ids.iter().any(|id| id == obligation_id) {
            return Err(RunStateError::UnknownObligation(obligation_id.to_string()));
        }
        for &idx in &cites {
            if idx >= self.entries.len() {
                return Err(RunStateError::UnknownEntry(idx));
            }
        }
        self.verdicts
            .insert(obligation_id.to_string(), VerdictState { passed, cites });
        self.save()
    }

    /// Compute the outcome from obligation verdicts alone, tear the world
    /// down, and report. Order matters: contamination is checked first
    /// because it invalidates every verdict regardless of what they said;
    /// undecided obligations are checked next because an incomplete run
    /// has not actually produced a pass or a fail yet.
    pub fn finish(self) -> Result<FinishReport, RunStateError> {
        let world = self.attached();
        let current_digest = world.current_repo_dirty_digest();

        let contaminated = matches!(
            (&self.repo_dirty_digest_at_build, &current_digest),
            (Some(before), Some(after)) if before != after
        );

        let undecided: Vec<String> = self
            .obligation_ids
            .iter()
            .filter(|id| !self.verdicts.contains_key(id.as_str()))
            .cloned()
            .collect();

        let outcome = if contaminated {
            RunOutcome::Contaminated
        } else if !undecided.is_empty() {
            RunOutcome::Incomplete
        } else if self.verdicts.values().any(|v| !v.passed) {
            RunOutcome::Fail
        } else {
            RunOutcome::Pass
        };

        let mut report = FinishReport {
            world_id: self.world_id.clone(),
            outcome,
            undecided_obligations: undecided,
            notes: self.notes.clone(),
        };

        // A world that cannot be torn down is an environment failure, not
        // a product one — it must never override an already-computed
        // Contaminated verdict (both point at the environment, and
        // Contaminated is the more specific, already-established finding).
        if world.teardown().is_err() && report.outcome != RunOutcome::Contaminated {
            report.outcome = RunOutcome::Environment;
        }

        Ok(report)
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

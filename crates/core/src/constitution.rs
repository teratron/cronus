//! Agent constitution — five identity files, bootstrap ritual, 3-file TOML merge,
//! and agentic readiness checklist.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ── Identity file names ───────────────────────────────────────────────────────

pub const SOUL_FILE: &str = "SOUL.md";
pub const PROFILE_FILE: &str = "PROFILE.md";
pub const MEMORY_FILE: &str = "MEMORY.md";
pub const HEARTBEAT_FILE: &str = "HEARTBEAT.md";
pub const BOOTSTRAP_FILE: &str = "BOOTSTRAP.md";

pub const IDENTITY_FILES: [&str; 5] = [
    SOUL_FILE,
    PROFILE_FILE,
    MEMORY_FILE,
    HEARTBEAT_FILE,
    BOOTSTRAP_FILE,
];

// ── Embedded templates ────────────────────────────────────────────────────────

const SOUL_TEMPLATE: &str = "# Soul\n\n## Core values\n\n- Honesty\n- Helpfulness\n- Harmlessness\n\n## Does\n\n- Assist with technical tasks\n\n## Does not do\n\n- Generate harmful content\n";
const PROFILE_TEMPLATE: &str = "# Profile\n\n## Name\n\nCronus Agent\n\n## Voice\n\nClear and direct\n\n## Focus\n\nSoftware engineering\n";
const MEMORY_TEMPLATE: &str = "# Memory\n\n<!-- Quick-recall facts (≤200 lines, ≤150 chars/line, ≤25 KB) -->\n";
const HEARTBEAT_TEMPLATE: &str = "# Heartbeat\n\nstatus: active\nlast_seen: ~\n";
const BOOTSTRAP_TEMPLATE: &str = "# Bootstrap\n\n- [ ] Identity files created\n- [ ] Memory initialised\n- [ ] Profile configured\n";

fn template_for(filename: &str) -> &'static str {
    match filename {
        SOUL_FILE => SOUL_TEMPLATE,
        PROFILE_FILE => PROFILE_TEMPLATE,
        MEMORY_FILE => MEMORY_TEMPLATE,
        HEARTBEAT_FILE => HEARTBEAT_TEMPLATE,
        BOOTSTRAP_FILE => BOOTSTRAP_TEMPLATE,
        _ => "",
    }
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────

/// Bootstrap identity files under `workspace_dir/.cronus/`.
///
/// Idempotent — existing files are left unchanged.
pub fn bootstrap(workspace_dir: &Path) -> ConstitutionResult<Vec<PathBuf>> {
    let identity_dir = workspace_dir.join(".cronus");
    fs::create_dir_all(&identity_dir)?;

    let mut created = Vec::new();
    for filename in &IDENTITY_FILES {
        let path = identity_dir.join(filename);
        if !path.exists() {
            fs::write(&path, template_for(filename))?;
            created.push(path);
        }
    }
    Ok(created)
}

/// Returns paths of all five identity files (regardless of existence).
pub fn identity_paths(workspace_dir: &Path) -> [PathBuf; 5] {
    let dir = workspace_dir.join(".cronus");
    [
        dir.join(SOUL_FILE),
        dir.join(PROFILE_FILE),
        dir.join(MEMORY_FILE),
        dir.join(HEARTBEAT_FILE),
        dir.join(BOOTSTRAP_FILE),
    ]
}

// ── TOML value ────────────────────────────────────────────────────────────────

/// Minimal TOML-like value tree for the 3-file merge (no external dep).
#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Bool(bool),
    Array(Vec<TomlValue>),
    Table(HashMap<String, TomlValue>),
    KeyedArray(Vec<HashMap<String, TomlValue>>), // arrays of tables with a `name` key
}

// ── 3-file TOML merge ─────────────────────────────────────────────────────────

/// Merge three config layers: `user > team > base`.
///
/// Rules:
/// - Scalars: user wins if set, else team, else base.
/// - Tables: recursive key union (same rules applied recursively).
/// - KeyedArray (arrays of `name`-keyed tables): user entry for same `name`
///   replaces the base entry; new names from user are appended; team entries
///   fill gaps left by base.
pub fn merge_toml(base: TomlValue, team: TomlValue, user: TomlValue) -> TomlValue {
    match (base, team, user) {
        (TomlValue::Table(b), TomlValue::Table(t), TomlValue::Table(u)) => {
            merge_tables(b, t, u)
        }
        (TomlValue::KeyedArray(b), TomlValue::KeyedArray(t), TomlValue::KeyedArray(u)) => {
            merge_keyed_arrays(b, t, u)
        }
        // Scalar: user wins if non-empty-string, else team, else base.
        (b, t, TomlValue::String(ref us)) if us.is_empty() => {
            // user not set — try team
            match t {
                TomlValue::String(ref ts) if ts.is_empty() => b,
                other_t => other_t,
            }
        }
        (_, _, u) => u,
    }
}

fn merge_tables(
    base: HashMap<String, TomlValue>,
    team: HashMap<String, TomlValue>,
    user: HashMap<String, TomlValue>,
) -> TomlValue {
    let mut result: HashMap<String, TomlValue> = HashMap::new();
    let all_keys: std::collections::HashSet<String> = base
        .keys()
        .chain(team.keys())
        .chain(user.keys())
        .cloned()
        .collect();

    for key in all_keys {
        let b = base.get(&key).cloned().unwrap_or(TomlValue::String(String::new()));
        let t = team.get(&key).cloned().unwrap_or(TomlValue::String(String::new()));
        let u = user.get(&key).cloned().unwrap_or(TomlValue::String(String::new()));
        result.insert(key, merge_toml(b, t, u));
    }
    TomlValue::Table(result)
}

fn merge_keyed_arrays(
    base: Vec<HashMap<String, TomlValue>>,
    team: Vec<HashMap<String, TomlValue>>,
    user: Vec<HashMap<String, TomlValue>>,
) -> TomlValue {
    let name_of = |entry: &HashMap<String, TomlValue>| -> Option<String> {
        if let Some(TomlValue::String(n)) = entry.get("name") {
            Some(n.clone())
        } else {
            None
        }
    };

    // Start with base; apply team gaps; override with user.
    let mut result: Vec<HashMap<String, TomlValue>> = base;

    // Merge team: add entries not already present by name.
    for t_entry in team {
        if let Some(name) = name_of(&t_entry) {
            if !result.iter().any(|e| name_of(e).as_deref() == Some(&name)) {
                result.push(t_entry);
            }
        } else {
            result.push(t_entry);
        }
    }

    // Override with user: replace same-name, append new.
    for u_entry in user {
        if let Some(name) = name_of(&u_entry) {
            if let Some(slot) = result.iter_mut().find(|e| name_of(e).as_deref() == Some(&name)) {
                *slot = u_entry;
            } else {
                result.push(u_entry);
            }
        } else {
            result.push(u_entry);
        }
    }

    TomlValue::KeyedArray(result)
}

// ── Agent activation sequence ─────────────────────────────────────────────────

/// Steps in the 8-step agent activation sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationStep {
    PrependSteps,
    AdoptPersona,
    PersistentFacts,
    Config,
    Greet,
    AppendSteps,
    Menu,
    Done,
}

impl ActivationStep {
    pub const SEQUENCE: [ActivationStep; 8] = [
        ActivationStep::PrependSteps,
        ActivationStep::AdoptPersona,
        ActivationStep::PersistentFacts,
        ActivationStep::Config,
        ActivationStep::Greet,
        ActivationStep::AppendSteps,
        ActivationStep::Menu,
        ActivationStep::Done,
    ];
}

/// Execute the 8-step activation sequence (no-op seam at Phase 4).
pub fn activate(steps: &[ActivationStep]) -> Vec<(ActivationStep, Result<(), String>)> {
    steps.iter().copied().map(|s| (s, Ok(()))).collect()
}

// ── Agentic readiness checklist ───────────────────────────────────────────────

/// The eight readiness signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadinessSignal {
    ContextFile,
    Skills,
    Agents,
    Templates,
    Hooks,
    IsolatedRuntime,
    Mcp,
    Freshness,
}

/// Weight of each readiness signal in the score.
fn signal_weight(s: ReadinessSignal) -> u32 {
    match s {
        ReadinessSignal::ContextFile => 20,
        ReadinessSignal::Skills => 12,
        ReadinessSignal::Agents => 12,
        ReadinessSignal::Templates => 12,
        ReadinessSignal::Hooks => 6,
        ReadinessSignal::IsolatedRuntime => 20,
        ReadinessSignal::Mcp => 12,
        ReadinessSignal::Freshness => 6,
    }
}

/// Readiness tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessTier {
    Ready,
    Partial,
    NotReady,
}

/// Score a set of present signals and return (score, tier).
///
/// `score ≥ 80 → Ready`, `≥ 50 → Partial`, `< 50 → NotReady`.
pub fn readiness_score(present: &[ReadinessSignal]) -> (u32, ReadinessTier) {
    let score: u32 = present.iter().map(|&s| signal_weight(s)).sum();
    let tier = if score >= 80 {
        ReadinessTier::Ready
    } else if score >= 50 {
        ReadinessTier::Partial
    } else {
        ReadinessTier::NotReady
    };
    (score, tier)
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ConstitutionError {
    Io(std::io::Error),
}

impl std::fmt::Display for ConstitutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstitutionError::Io(e) => write!(f, "constitution I/O error: {e}"),
        }
    }
}

impl std::error::Error for ConstitutionError {}

impl From<std::io::Error> for ConstitutionError {
    fn from(e: std::io::Error) -> Self {
        ConstitutionError::Io(e)
    }
}

pub type ConstitutionResult<T> = Result<T, ConstitutionError>;

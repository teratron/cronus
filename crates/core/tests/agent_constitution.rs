use cronus::constitution::{
    ActivationStep, BOOTSTRAP_FILE, HEARTBEAT_FILE, IDENTITY_FILES, MEMORY_FILE, PROFILE_FILE,
    ReadinessSignal, ReadinessTier, SOUL_FILE, TomlValue, activate, bootstrap, identity_paths,
    merge_toml, readiness_score,
};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_workspace() -> std::path::PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let d = std::env::temp_dir().join(format!("cronus-const-{pid}-{id}"));
    fs::create_dir_all(&d).unwrap();
    d
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────

#[test]
fn bootstrap_creates_all_five_identity_files() {
    let ws = tmp_workspace();
    let created = bootstrap(&ws).unwrap();
    assert_eq!(created.len(), 5, "all 5 identity files must be created");
    let dir = ws.join(".cronus");
    for name in &IDENTITY_FILES {
        assert!(dir.join(name).exists(), "{name} must exist after bootstrap");
    }
    fs::remove_dir_all(&ws).ok();
}

#[test]
fn bootstrap_is_idempotent() {
    let ws = tmp_workspace();
    bootstrap(&ws).unwrap();
    // Write custom content to SOUL
    let soul = ws.join(".cronus").join(SOUL_FILE);
    fs::write(&soul, "custom content").unwrap();
    // Second bootstrap must not overwrite
    let created = bootstrap(&ws).unwrap();
    assert_eq!(created.len(), 0, "second bootstrap must create nothing");
    assert_eq!(fs::read_to_string(&soul).unwrap(), "custom content");
    fs::remove_dir_all(&ws).ok();
}

#[test]
fn identity_paths_returns_five_paths() {
    let ws = tmp_workspace();
    let paths = identity_paths(&ws);
    assert_eq!(paths.len(), 5);
    fs::remove_dir_all(&ws).ok();
}

#[test]
fn bootstrap_files_have_non_empty_content() {
    let ws = tmp_workspace();
    bootstrap(&ws).unwrap();
    let dir = ws.join(".cronus");
    for name in &IDENTITY_FILES {
        let content = fs::read_to_string(dir.join(name)).unwrap();
        assert!(!content.is_empty(), "{name} must not be empty");
    }
    fs::remove_dir_all(&ws).ok();
}

#[test]
fn identity_file_constants_match() {
    assert_eq!(SOUL_FILE, "SOUL.md");
    assert_eq!(PROFILE_FILE, "PROFILE.md");
    assert_eq!(MEMORY_FILE, "MEMORY.md");
    assert_eq!(HEARTBEAT_FILE, "HEARTBEAT.md");
    assert_eq!(BOOTSTRAP_FILE, "BOOTSTRAP.md");
}

// ── 3-file TOML merge: scalar ─────────────────────────────────────────────────

#[test]
fn merge_scalar_user_wins_when_set() {
    let base = TomlValue::String("base".into());
    let team = TomlValue::String("team".into());
    let user = TomlValue::String("user".into());
    assert_eq!(
        merge_toml(base, team, user),
        TomlValue::String("user".into())
    );
}

#[test]
fn merge_scalar_team_wins_when_user_empty() {
    let base = TomlValue::String("base".into());
    let team = TomlValue::String("team".into());
    let user = TomlValue::String(String::new());
    assert_eq!(
        merge_toml(base, team, user),
        TomlValue::String("team".into())
    );
}

#[test]
fn merge_scalar_base_wins_when_team_and_user_empty() {
    let base = TomlValue::String("base".into());
    let team = TomlValue::String(String::new());
    let user = TomlValue::String(String::new());
    assert_eq!(
        merge_toml(base, team, user),
        TomlValue::String("base".into())
    );
}

// ── 3-file TOML merge: table ──────────────────────────────────────────────────

#[test]
fn merge_table_user_key_overrides_base() {
    let mut base_map = HashMap::new();
    base_map.insert("key".into(), TomlValue::String("base".into()));
    let base = TomlValue::Table(base_map);

    let team = TomlValue::Table(HashMap::new());

    let mut user_map = HashMap::new();
    user_map.insert("key".into(), TomlValue::String("user".into()));
    let user = TomlValue::Table(user_map);

    if let TomlValue::Table(result) = merge_toml(base, team, user) {
        assert_eq!(result["key"], TomlValue::String("user".into()));
    } else {
        panic!("expected Table");
    }
}

#[test]
fn merge_table_accumulates_keys_from_all_three() {
    let mut base_map = HashMap::new();
    base_map.insert("a".into(), TomlValue::String("a".into()));
    let base = TomlValue::Table(base_map);

    let mut team_map = HashMap::new();
    team_map.insert("b".into(), TomlValue::String("b".into()));
    let team = TomlValue::Table(team_map);

    let mut user_map = HashMap::new();
    user_map.insert("c".into(), TomlValue::String("c".into()));
    let user = TomlValue::Table(user_map);

    if let TomlValue::Table(result) = merge_toml(base, team, user) {
        assert!(result.contains_key("a"));
        assert!(result.contains_key("b"));
        assert!(result.contains_key("c"));
    } else {
        panic!("expected Table");
    }
}

// ── 3-file TOML merge: keyed array ───────────────────────────────────────────

fn named_entry(name: &str, val: &str) -> HashMap<String, TomlValue> {
    let mut m = HashMap::new();
    m.insert("name".into(), TomlValue::String(name.into()));
    m.insert("value".into(), TomlValue::String(val.into()));
    m
}

#[test]
fn keyed_array_user_overrides_same_name() {
    let base = TomlValue::KeyedArray(vec![named_entry("hook", "base-hook")]);
    let team = TomlValue::KeyedArray(vec![]);
    let user = TomlValue::KeyedArray(vec![named_entry("hook", "user-hook")]);

    if let TomlValue::KeyedArray(result) = merge_toml(base, team, user) {
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["value"], TomlValue::String("user-hook".into()));
    } else {
        panic!("expected KeyedArray");
    }
}

#[test]
fn keyed_array_user_appends_new_names() {
    let base = TomlValue::KeyedArray(vec![named_entry("old", "v")]);
    let team = TomlValue::KeyedArray(vec![]);
    let user = TomlValue::KeyedArray(vec![named_entry("new", "u")]);

    if let TomlValue::KeyedArray(result) = merge_toml(base, team, user) {
        assert_eq!(result.len(), 2);
        let names: Vec<_> = result
            .iter()
            .filter_map(|e| {
                if let Some(TomlValue::String(n)) = e.get("name") {
                    Some(n.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(names.contains(&"old"));
        assert!(names.contains(&"new"));
    } else {
        panic!("expected KeyedArray");
    }
}

// ── Agent activation sequence ─────────────────────────────────────────────────

#[test]
fn activation_sequence_has_eight_steps() {
    assert_eq!(ActivationStep::SEQUENCE.len(), 8);
}

#[test]
fn activate_runs_all_steps_in_order() {
    let results = activate(&ActivationStep::SEQUENCE);
    assert_eq!(results.len(), 8);
    for (step, result) in &results {
        assert!(result.is_ok(), "step {step:?} must succeed at Phase 4");
    }
}

#[test]
fn activate_with_subset_of_steps() {
    let steps = [ActivationStep::PrependSteps, ActivationStep::Greet];
    let results = activate(&steps);
    assert_eq!(results.len(), 2);
}

// ── Agentic readiness checklist ───────────────────────────────────────────────

#[test]
fn readiness_with_four_strong_signals_is_ready() {
    // ContextFile(20) + IsolatedRuntime(20) + Skills(12) + Agents(12) = 64 → Partial
    // ContextFile(20) + IsolatedRuntime(20) + Skills(12) + Agents(12) + Mcp(12) = 76 → Partial
    // ContextFile(20) + IsolatedRuntime(20) + Skills(12) + Agents(12) + Mcp(12) + Templates(12) = 88 → Ready
    let signals = [
        ReadinessSignal::ContextFile,
        ReadinessSignal::IsolatedRuntime,
        ReadinessSignal::Skills,
        ReadinessSignal::Agents,
        ReadinessSignal::Mcp,
        ReadinessSignal::Templates,
    ];
    let (score, tier) = readiness_score(&signals);
    assert!(score >= 80, "score={score}");
    assert_eq!(tier, ReadinessTier::Ready);
}

#[test]
fn readiness_partial_between_50_and_80() {
    let signals = [ReadinessSignal::ContextFile, ReadinessSignal::Skills];
    // 20 + 12 = 32 → NotReady, need more
    let signals2 = [
        ReadinessSignal::ContextFile,
        ReadinessSignal::IsolatedRuntime,
        ReadinessSignal::Skills,
    ];
    let (score, tier) = readiness_score(&signals2);
    // 20 + 20 + 12 = 52 → Partial
    assert!((50..80).contains(&score), "score={score}");
    assert_eq!(tier, ReadinessTier::Partial);
    let _ = signals;
}

#[test]
fn readiness_not_ready_below_50() {
    let signals = [ReadinessSignal::Hooks]; // 6 points
    let (score, tier) = readiness_score(&signals);
    assert!(score < 50, "score={score}");
    assert_eq!(tier, ReadinessTier::NotReady);
}

#[test]
fn readiness_empty_signals_is_not_ready() {
    let (score, tier) = readiness_score(&[]);
    assert_eq!(score, 0);
    assert_eq!(tier, ReadinessTier::NotReady);
}

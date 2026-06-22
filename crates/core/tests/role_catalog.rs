use std::sync::atomic::{AtomicU64, Ordering};

use cronus::roles::{
    AgentConfigRevision, HiredFrom,
    RevisionSource, RoleCategory, RoleError, RoleManager, PRESET_CATALOG,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir(label: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("cronus-role-test-{pid}-{id}-{label}"))
}

fn make_manager(label: &str) -> (RoleManager, std::path::PathBuf, std::path::PathBuf) {
    let program_dir = unique_dir(&format!("{label}-program"));
    let state_dir = unique_dir(&format!("{label}-state"));
    std::fs::create_dir_all(&program_dir).unwrap();
    std::fs::create_dir_all(&state_dir).unwrap();
    let mgr = RoleManager::new(program_dir.clone(), state_dir.clone());
    (mgr, program_dir, state_dir)
}

// ── Catalog tests ─────────────────────────────────────────────────────────────

#[test]
fn preset_catalog_has_25_roles() {
    assert_eq!(PRESET_CATALOG.len(), 25, "catalog should have exactly 25 preset roles");
}

#[test]
fn preset_catalog_covers_all_categories() {
    let has_engineering = PRESET_CATALOG.iter().any(|r| r.category == RoleCategory::Engineering);
    let has_quality = PRESET_CATALOG.iter().any(|r| r.category == RoleCategory::Quality);
    let has_ops = PRESET_CATALOG.iter().any(|r| r.category == RoleCategory::OpsAndDocs);
    let has_memory = PRESET_CATALOG.iter().any(|r| r.category == RoleCategory::Memory);
    let has_business = PRESET_CATALOG.iter().any(|r| r.category == RoleCategory::Business);
    assert!(has_engineering && has_quality && has_ops && has_memory && has_business);
}

#[test]
fn preset_catalog_ids_are_unique() {
    let mut ids = std::collections::HashSet::new();
    for role in PRESET_CATALOG {
        assert!(ids.insert(role.id), "duplicate preset ID: {}", role.id);
    }
}

#[test]
fn list_presets_returns_full_catalog() {
    let (mgr, _, _) = make_manager("list-presets");
    let presets = mgr.list_presets();
    assert_eq!(presets.len(), 25);
}

// ── Hire / fire tests ─────────────────────────────────────────────────────────

#[test]
fn hire_copies_blueprint_to_state() {
    let (mgr, _, state_dir) = make_manager("hire");
    let instance = mgr.hire("architect", None).unwrap();
    let instance_dir = state_dir.join("employees").join(&instance.id);
    assert!(instance_dir.exists(), "instance directory should be created");
    assert!(instance_dir.join("memory").exists());
    assert!(instance_dir.join("skills").exists());
    assert!(instance_dir.join("skins").exists());
    assert!(instance_dir.join("config.json").exists());
}

#[test]
fn hire_records_hired_from_preset() {
    let (mgr, _, _) = make_manager("hire-from");
    let instance = mgr.hire("backend-engineer", None).unwrap();
    assert_eq!(instance.hired_from, HiredFrom::Preset("backend-engineer".to_string()));
}

#[test]
fn hire_with_custom_name() {
    let (mgr, _, state_dir) = make_manager("hire-named");
    let instance = mgr.hire("code-reviewer", Some("my-reviewer")).unwrap();
    assert_eq!(instance.id, "my-reviewer");
    assert!(state_dir.join("employees").join("my-reviewer").exists());
}

#[test]
fn hire_unknown_preset_returns_not_found() {
    let (mgr, _, _) = make_manager("hire-unknown");
    let err = mgr.hire("non-existent-role", None).unwrap_err();
    assert!(matches!(err, RoleError::NotFound(_)));
}

#[test]
fn fire_archives_memory_and_removes_from_roster() {
    let (mgr, _, state_dir) = make_manager("fire");
    let _instance = mgr.hire("doc-writer", Some("writer-1")).unwrap();
    let instance_dir = state_dir.join("employees").join("writer-1");
    std::fs::write(instance_dir.join("memory").join("note.txt"), "recall").unwrap();

    mgr.fire("writer-1").unwrap();

    // Instance dir should be renamed/moved
    assert!(!instance_dir.exists(), "instance dir should be gone after fire");
    // Fired dir exists
    let fired_dir = state_dir.join("employees").join("writer-1-fired");
    assert!(fired_dir.exists(), "fired dir should contain the archived instance");
}

#[test]
fn fire_unknown_instance_returns_not_found() {
    let (mgr, _, _) = make_manager("fire-unknown");
    let err = mgr.fire("no-such-instance").unwrap_err();
    assert!(matches!(err, RoleError::NotFound(_)));
}

#[test]
fn list_hired_returns_all_instances() {
    let (mgr, _, _) = make_manager("list-hired");
    mgr.hire("architect", Some("arch-1")).unwrap();
    mgr.hire("debugger", Some("debug-1")).unwrap();
    let hired = mgr.list_hired().unwrap();
    assert_eq!(hired.len(), 2);
}

// ── Custom role tests ─────────────────────────────────────────────────────────

#[test]
fn create_custom_role_writes_custom_hired_from() {
    let (mgr, _, _) = make_manager("custom");
    let instance = mgr.create_custom("my-role", "My Custom Role").unwrap();
    assert_eq!(instance.hired_from, HiredFrom::Custom);
    assert_eq!(instance.category, RoleCategory::Custom);
}

#[test]
fn create_custom_derived_from_preset_writes_derived_from() {
    let (mgr, _, _) = make_manager("derived");
    let instance = mgr
        .create_from_preset("my-reviewer", "My Reviewer", "code-reviewer")
        .unwrap();
    assert_eq!(
        instance.hired_from,
        HiredFrom::DerivedFrom("code-reviewer".to_string())
    );
}

// ── Preset read-only guard ────────────────────────────────────────────────────

#[test]
fn mutating_preset_returns_read_only_error() {
    let (mgr, _, _) = make_manager("readonly");
    let err = mgr.mutate_preset("architect").unwrap_err();
    assert!(matches!(err, RoleError::PresetReadOnly));
}

// ── Config revision tests ─────────────────────────────────────────────────────

#[test]
fn config_revision_saves_and_lists() {
    let dir = unique_dir("revisions");
    std::fs::create_dir_all(&dir).unwrap();
    let rev = AgentConfigRevision::new_patch(
        "arch-1",
        1,
        r#"{"model":"claude-3"}"#.to_string(),
        r#"{"model":"claude-4"}"#.to_string(),
        vec!["model".to_string()],
        1_000_000,
    );
    rev.save_to(&dir).unwrap();

    let numbers = AgentConfigRevision::list_revision_numbers(&dir).unwrap();
    assert_eq!(numbers, vec![1u32]);
}

#[test]
fn config_revision_numbers_are_monotonically_increasing() {
    let dir = unique_dir("revisions-mono");
    std::fs::create_dir_all(&dir).unwrap();
    for n in [1u32, 2, 3] {
        let rev = AgentConfigRevision::new_patch(
            "agent",
            n,
            format!("before-{n}"),
            format!("after-{n}"),
            vec![],
            n as u64 * 1000,
        );
        rev.save_to(&dir).unwrap();
    }
    let numbers = AgentConfigRevision::list_revision_numbers(&dir).unwrap();
    assert_eq!(numbers, vec![1, 2, 3]);
}

#[test]
fn rollback_revision_records_source_and_original() {
    let rev = AgentConfigRevision::rollback_from(
        "agent-x",
        5,
        r#"{"model":"new"}"#.to_string(),
        r#"{"model":"old"}"#.to_string(),
        6,
        9_999_999,
    );
    assert_eq!(rev.source, RevisionSource::Rollback);
    assert_eq!(rev.rolled_back_from, Some(5));
    assert_eq!(rev.revision_number, 6);
}

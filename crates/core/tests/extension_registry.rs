use cronus::extensions::{
    ExtensionKind, ExtensionManifest, ExtensionPermissions, ExtensionRegistry, ExtensionSource,
    ExtensionState, validate_manifest,
};

fn make_manifest(id: &str, kind: ExtensionKind) -> ExtensionManifest {
    ExtensionManifest {
        id: id.to_string(),
        kind,
        name: format!("{id}-name"),
        version: "1.0.0".to_string(),
        source: ExtensionSource::Custom,
        capabilities: vec![],
        permissions: ExtensionPermissions::default(),
    }
}

// ── validate_manifest ────────────────────────────────────────────────────────

#[test]
fn valid_manifest_passes_validation() {
    let m = make_manifest("my-ext", ExtensionKind::Plugin);
    assert!(validate_manifest(&m).is_ok());
}

#[test]
fn manifest_with_empty_id_fails() {
    let mut m = make_manifest("ok", ExtensionKind::Skill);
    m.id = String::new();
    assert!(validate_manifest(&m).is_err());
}

#[test]
fn manifest_with_empty_name_fails() {
    let mut m = make_manifest("ok", ExtensionKind::Skill);
    m.name = String::new();
    assert!(validate_manifest(&m).is_err());
}

#[test]
fn manifest_with_empty_version_fails() {
    let mut m = make_manifest("ok", ExtensionKind::Skill);
    m.version = String::new();
    assert!(validate_manifest(&m).is_err());
}

// ── ExtensionRegistry lifecycle ───────────────────────────────────────────────

#[test]
fn register_sets_discovered_state() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("ext-1", ExtensionKind::Skill))
        .unwrap();
    assert_eq!(reg.state("ext-1"), Some(ExtensionState::Discovered));
}

#[test]
fn state_returns_none_for_unknown_id() {
    let reg = ExtensionRegistry::new();
    assert_eq!(reg.state("unknown"), None);
}

#[test]
fn transition_discovered_to_permitted_succeeds() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::McpServer))
        .unwrap();
    assert!(reg.transition("e", ExtensionState::Permitted).is_ok());
    assert_eq!(reg.state("e"), Some(ExtensionState::Permitted));
}

#[test]
fn transition_permitted_to_active_succeeds() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::Plugin))
        .unwrap();
    reg.transition("e", ExtensionState::Permitted).unwrap();
    reg.transition("e", ExtensionState::Active).unwrap();
    assert_eq!(reg.state("e"), Some(ExtensionState::Active));
}

#[test]
fn transition_active_to_inactive_succeeds() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::Skill))
        .unwrap();
    reg.transition("e", ExtensionState::Permitted).unwrap();
    reg.transition("e", ExtensionState::Active).unwrap();
    reg.transition("e", ExtensionState::Inactive).unwrap();
    assert_eq!(reg.state("e"), Some(ExtensionState::Inactive));
}

#[test]
fn transition_inactive_back_to_active_succeeds() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::Skill))
        .unwrap();
    reg.transition("e", ExtensionState::Permitted).unwrap();
    reg.transition("e", ExtensionState::Active).unwrap();
    reg.transition("e", ExtensionState::Inactive).unwrap();
    reg.transition("e", ExtensionState::Active).unwrap();
    assert_eq!(reg.state("e"), Some(ExtensionState::Active));
}

#[test]
fn invalid_transition_returns_error() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::Skill))
        .unwrap();
    // cannot go directly from Discovered to Active
    let result = reg.transition("e", ExtensionState::Active);
    assert!(result.is_err());
}

#[test]
fn require_active_returns_manifest_when_active() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::Plugin))
        .unwrap();
    reg.transition("e", ExtensionState::Permitted).unwrap();
    reg.transition("e", ExtensionState::Active).unwrap();

    let m = reg.require_active("e").unwrap();
    assert_eq!(m.id, "e");
}

#[test]
fn require_active_fails_when_not_active() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e", ExtensionKind::Plugin))
        .unwrap();
    assert!(reg.require_active("e").is_err());
}

#[test]
fn list_returns_all_registered_extensions() {
    let mut reg = ExtensionRegistry::new();
    reg.register(make_manifest("e1", ExtensionKind::Skill))
        .unwrap();
    reg.register(make_manifest("e2", ExtensionKind::McpServer))
        .unwrap();
    assert_eq!(reg.list().len(), 2);
}

// ── ExtensionKind::as_str ────────────────────────────────────────────────────

#[test]
fn extension_kind_as_str_values() {
    assert_eq!(ExtensionKind::Skill.as_str(), "skill");
    assert_eq!(ExtensionKind::McpServer.as_str(), "mcp-server");
    assert_eq!(ExtensionKind::Plugin.as_str(), "plugin");
}

// ── ExtensionState::as_str ───────────────────────────────────────────────────

#[test]
fn extension_state_as_str_values() {
    assert_eq!(ExtensionState::Discovered.as_str(), "discovered");
    assert_eq!(ExtensionState::Active.as_str(), "active");
    assert_eq!(ExtensionState::Inactive.as_str(), "inactive");
}

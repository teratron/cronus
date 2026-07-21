//! Developer-office workspace registration and elevated-action confinement/
//! audit wiring (DVO-1, DVO-6, DVO-7). Composition over already-shipped
//! subsystems: the `store-local` workspace registry, `l2-tool-security`'s
//! authority gate, and its append-only audit log — no new authority
//! mechanism here, only a thin binding scoped to the dev office.
//!
//! **Disclosed scope boundary:** the elevated-action audit entries here ride
//! `tool_security::AuditEntry` — a real, already-shipped, append-only
//! logged-audit-trail mechanism — not a MAC-signed, tamper-evident,
//! model-unforgeable "tool receipt". That stronger primitive has no
//! implementation anywhere in this project yet; building it is a
//! substantial, security-sensitive subsystem in its own right and belongs
//! in its own spec-driven phase, not improvised inline here. What's wired
//! is the real, composable piece that exists today.

use std::path::Path;

use cronus_domain::tool_security::{AuditEntry, ToolPermitResult, ToolPolicy, append_audit_entry, now_ms};
use cronus_store_local::workspace::{
    Workspace, WorkspaceError, WorkspaceId, WorkspaceManager, WorkspaceTemplate,
};

/// The reserved, well-known id of the sole `WorkspaceKind::Developer`
/// instance (DVO-1). Never accepted by the ordinary `workspace create`/
/// `delete` CLI flow (guarded there, the `RESERVED_USERNAMES` precedent) —
/// only [`register_dev_workspace`], called solely on an `Elevated` gate
/// resolution, ever writes this row.
pub const DEV_OFFICE_WORKSPACE_ID: &str = "dev-office";

/// Whether `id` is the reserved developer-workspace id — the check the
/// ordinary workspace create/delete CLI flow guards against, so
/// `WorkspaceKind::Developer` is not creatable or deletable through the
/// ordinary project-creation flow (DVO-1).
pub fn is_reserved_dev_workspace_id(id: &str) -> bool {
    id == DEV_OFFICE_WORKSPACE_ID
}

/// Register (or re-affirm) the sole developer workspace, scoped to
/// `bound_repo`. Cardinality (0..1) falls out of the registry's own
/// existing semantics: a second registration attempt at the same reserved
/// id returns [`WorkspaceError::AlreadyExists`], not a duplicate row — no
/// new uniqueness logic needed here.
pub fn register_dev_workspace(
    mgr: &WorkspaceManager,
    bound_repo: &Path,
) -> Result<Workspace, WorkspaceError> {
    let id = WorkspaceId::new(DEV_OFFICE_WORKSPACE_ID)
        .expect("DEV_OFFICE_WORKSPACE_ID is a valid kebab-case constant");
    mgr.create(&id, "Developer Office", bound_repo, WorkspaceTemplate::Empty)
}

/// Run one elevated dev-office action through the confinement/audit chain
/// (DVO-7): the `l2-tool-security` authority gate first, an append-only
/// audit entry always second — allowed *and* blocked attempts are both
/// recorded, because an audit trail that only remembers successes isn't one.
/// If the audit write itself fails, the action is refused (fail-closed):
/// an unaudited elevated action is never allowed to proceed silently.
///
/// Nothing here reaches the workspace registry at all — an elevated action
/// has no parameter or capability that could touch another workspace's
/// store (the INV-8 boundary assertion), by plain absence, not a runtime
/// check that could be bypassed.
pub fn run_elevated_action(
    policy: &ToolPolicy,
    audit_path: &Path,
    action_name: &str,
) -> Result<(), String> {
    let permit = policy.is_permitted(action_name);
    let outcome: &'static str = if matches!(permit, ToolPermitResult::Allowed) {
        "allowed"
    } else {
        "blocked"
    };
    let entry = AuditEntry {
        timestamp: now_ms(),
        layer: "dev-office",
        tool_name: Some(action_name.to_string()),
        finding_id: format!("dev-office:{action_name}"),
        category: "elevated-action".to_string(),
        severity: "info".to_string(),
        outcome,
    };
    append_audit_entry(audit_path, &entry).map_err(|e| format!("audit logging failed: {e}"))?;
    match permit {
        ToolPermitResult::Allowed => Ok(()),
        ToolPermitResult::Blocked(reason) => Err(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cronus-dev-office-workspace-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn dev_workspace_registers_once_and_refuses_a_second_registration() {
        let mgr = WorkspaceManager::open_in_memory().unwrap();
        let bound_repo = temp_dir("cardinality");

        let first = register_dev_workspace(&mgr, &bound_repo);
        assert!(first.is_ok());

        let second = register_dev_workspace(&mgr, &bound_repo);
        assert!(matches!(second, Err(WorkspaceError::AlreadyExists(_))));
    }

    #[test]
    fn dev_workspace_is_scoped_to_its_own_bound_path_only() {
        let mgr = WorkspaceManager::open_in_memory().unwrap();
        let dev_repo = temp_dir("dev-scope");
        let user_repo = temp_dir("user-scope");

        register_dev_workspace(&mgr, &dev_repo).unwrap();
        let user_id = WorkspaceId::new("acme-project").unwrap();
        mgr.create(&user_id, "Acme", &user_repo, WorkspaceTemplate::Default)
            .unwrap();

        let dev_ws = mgr
            .get(&WorkspaceId::new(DEV_OFFICE_WORKSPACE_ID).unwrap())
            .unwrap()
            .expect("dev workspace must exist");
        assert_eq!(dev_ws.path, dev_repo);
        assert_ne!(dev_ws.path, user_repo);

        // Registering the dev workspace does not disturb an unrelated user
        // workspace's own row — no shared/cross-referencing state.
        let user_ws = mgr.get(&user_id).unwrap().expect("user workspace exists");
        assert_eq!(user_ws.path, user_repo);
    }

    #[test]
    fn reserved_dev_workspace_id_is_recognized() {
        assert!(is_reserved_dev_workspace_id(DEV_OFFICE_WORKSPACE_ID));
        assert!(!is_reserved_dev_workspace_id("acme-project"));
    }

    #[test]
    fn elevated_action_allowed_by_policy_is_audited() {
        let dir = temp_dir("audit-allowed");
        let audit_path = dir.join("audit.jsonl");
        let policy = ToolPolicy::default();

        let result = run_elevated_action(&policy, &audit_path, "dev.self_edit");
        assert!(result.is_ok());

        let logged = std::fs::read_to_string(&audit_path).unwrap();
        assert!(logged.contains("dev-office"));
        assert!(logged.contains("dev.self_edit"));
        assert!(logged.contains("\"outcome\":\"allowed\""));
    }

    #[test]
    fn elevated_action_blocked_by_policy_is_still_audited_and_refused() {
        let dir = temp_dir("audit-blocked");
        let audit_path = dir.join("audit.jsonl");
        let mut policy = ToolPolicy::default();
        policy.disabled_tools.push("dev.self_edit".to_string());

        let result = run_elevated_action(&policy, &audit_path, "dev.self_edit");
        assert!(result.is_err());

        let logged = std::fs::read_to_string(&audit_path).unwrap();
        assert!(
            logged.contains("\"outcome\":\"blocked\""),
            "a refused elevated action must still be audited, not silently dropped"
        );
    }
}

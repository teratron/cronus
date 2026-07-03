//! Resource sharing — the uniform access-grant model shared by every resource type.
//!
//! One grant primitive over all resource kinds (RS-1); three principal types
//! user/group/public (RS-2); read/write with write-implies-read (RS-3); absence of
//! any grant means private (RS-4); the owner always wins, checked first (RS-5);
//! grants are additive with no precedence (RS-6); every grant change emits an audit
//! event (RS-7); the resource kind is a compile-time enum (RS-8).
//!
//! In-memory here (SQLite-backed `access_grant` table in production); the DB layer
//! and the audit-log transport are seams. The resolution algebra is implemented and
//! tested here.

/// A resource kind — the compile-time enumeration of shareable entities (RS-8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Knowledge,
    Skill,
    Tool,
    Note,
    Channel,
    File,
    Prompt,
}

/// A principal type (RS-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalKind {
    User,
    Group,
    /// The `public` principal; its id is the sentinel `*`.
    Public,
}

/// A permission level (RS-3). `Write` implies `Read` via the `Ord` derivation
/// (Read < Write), so a `Write` grant satisfies a `Read` requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Read,
    Write,
}

/// A single access grant row (RS-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessGrant {
    pub resource_type: ResourceKind,
    pub resource_id: String,
    pub principal_type: PrincipalKind,
    /// user_id | group_id | "*" for public.
    pub principal_id: String,
    pub permission: Permission,
}

/// An audit event emitted on every grant change (RS-7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantAudit {
    Added(AccessGrant),
    Removed {
        resource_type: ResourceKind,
        resource_id: String,
    },
}

/// The public sentinel principal id.
pub const PUBLIC_ID: &str = "*";

/// The access-grant store. In-memory stand-in for the `access_grant` SQLite table;
/// records audit events for every change (RS-7).
#[derive(Debug, Default)]
pub struct GrantStore {
    grants: Vec<AccessGrant>,
    audit: Vec<GrantAudit>,
}

impl GrantStore {
    pub fn new() -> Self {
        GrantStore::default()
    }

    /// Add a grant (idempotent on the full uniqueness tuple). Emits an audit event.
    pub fn add(&mut self, grant: AccessGrant) {
        if !self.grants.contains(&grant) {
            self.audit.push(GrantAudit::Added(grant.clone()));
            self.grants.push(grant);
        }
    }

    /// Replace the complete grant set for a resource (delete-all + insert-new),
    /// emitting audit events. Used by `set_grants` replace-all semantics.
    pub fn set_grants(
        &mut self,
        resource_type: ResourceKind,
        resource_id: &str,
        grants: Vec<AccessGrant>,
    ) {
        self.delete_for_resource(resource_type, resource_id);
        for g in grants {
            self.add(g);
        }
    }

    /// Remove all grants for a resource (call on resource deletion).
    pub fn delete_for_resource(&mut self, resource_type: ResourceKind, resource_id: &str) {
        let before = self.grants.len();
        self.grants
            .retain(|g| !(g.resource_type == resource_type && g.resource_id == resource_id));
        if self.grants.len() != before {
            self.audit.push(GrantAudit::Removed {
                resource_type,
                resource_id: resource_id.to_string(),
            });
        }
    }

    /// All grants for a single resource.
    pub fn get_grants(&self, resource_type: ResourceKind, resource_id: &str) -> Vec<&AccessGrant> {
        self.grants
            .iter()
            .filter(|g| g.resource_type == resource_type && g.resource_id == resource_id)
            .collect()
    }

    pub fn audit_log(&self) -> &[GrantAudit] {
        &self.audit
    }

    /// Whether `user_id` holds at least `required` permission on the resource.
    /// Resolution order: owner → direct user grant → group grant → public grant
    /// (RS-5 first, then additive RS-6). Absence of any grant = private (RS-4).
    pub fn has_access(
        &self,
        user_id: &str,
        is_owner: bool,
        resource_type: ResourceKind,
        resource_id: &str,
        required: Permission,
        groups: &[String],
    ) -> bool {
        // RS-5: owner always wins, checked before any grant lookup.
        if is_owner {
            return true;
        }
        self.get_grants(resource_type, resource_id)
            .into_iter()
            .any(|g| {
                // RS-3: write implies read (Permission: Read < Write).
                if g.permission < required {
                    return false;
                }
                match g.principal_type {
                    PrincipalKind::User => g.principal_id == user_id,
                    PrincipalKind::Group => groups.iter().any(|gr| gr == &g.principal_id),
                    PrincipalKind::Public => g.principal_id == PUBLIC_ID,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant(pt: PrincipalKind, pid: &str, perm: Permission) -> AccessGrant {
        AccessGrant {
            resource_type: ResourceKind::Note,
            resource_id: "note-1".into(),
            principal_type: pt,
            principal_id: pid.into(),
            permission: perm,
        }
    }

    #[test]
    fn absence_of_grant_is_private() {
        // RS-4.
        let store = GrantStore::new();
        assert!(!store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &[]
        ));
    }

    #[test]
    fn owner_always_wins_before_grant_lookup() {
        // RS-5.
        let store = GrantStore::new(); // no grants at all
        assert!(store.has_access(
            "owner",
            true,
            ResourceKind::Note,
            "note-1",
            Permission::Write,
            &[]
        ));
    }

    #[test]
    fn direct_user_grant_authorizes() {
        let mut store = GrantStore::new();
        store.add(grant(PrincipalKind::User, "u1", Permission::Read));
        assert!(store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &[]
        ));
        assert!(!store.has_access(
            "u2",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &[]
        ));
    }

    #[test]
    fn write_implies_read() {
        // RS-3: a write grant satisfies a read requirement, not vice versa.
        let mut store = GrantStore::new();
        store.add(grant(PrincipalKind::User, "u1", Permission::Write));
        assert!(store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &[]
        ));
        assert!(store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Write,
            &[]
        ));

        let mut ro = GrantStore::new();
        ro.add(grant(PrincipalKind::User, "u1", Permission::Read));
        assert!(!ro.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Write,
            &[]
        ));
    }

    #[test]
    fn group_and_public_grants_are_additive() {
        // RS-2 + RS-6: group and public grants each authorize independently.
        let mut store = GrantStore::new();
        store.add(grant(PrincipalKind::Group, "g-eng", Permission::Read));
        assert!(store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &["g-eng".into()]
        ));
        assert!(!store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &["g-sales".into()]
        ));

        store.add(grant(PrincipalKind::Public, PUBLIC_ID, Permission::Read));
        // Now anyone reads via the public grant.
        assert!(store.has_access(
            "stranger",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &[]
        ));
    }

    #[test]
    fn grant_changes_emit_audit_events() {
        // RS-7.
        let mut store = GrantStore::new();
        store.add(grant(PrincipalKind::User, "u1", Permission::Read));
        store.delete_for_resource(ResourceKind::Note, "note-1");
        assert_eq!(store.audit_log().len(), 2);
        assert!(matches!(store.audit_log()[0], GrantAudit::Added(_)));
        assert!(matches!(store.audit_log()[1], GrantAudit::Removed { .. }));
    }

    #[test]
    fn set_grants_replaces_all() {
        let mut store = GrantStore::new();
        store.add(grant(PrincipalKind::User, "u1", Permission::Read));
        store.set_grants(
            ResourceKind::Note,
            "note-1",
            vec![grant(PrincipalKind::User, "u2", Permission::Write)],
        );
        assert!(!store.has_access(
            "u1",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Read,
            &[]
        ));
        assert!(store.has_access(
            "u2",
            false,
            ResourceKind::Note,
            "note-1",
            Permission::Write,
            &[]
        ));
    }

    #[test]
    fn grants_are_kind_scoped() {
        // RS-1/RS-8: a grant on a Note does not authorize the same id as a File.
        let mut store = GrantStore::new();
        store.add(grant(PrincipalKind::User, "u1", Permission::Read));
        assert!(!store.has_access(
            "u1",
            false,
            ResourceKind::File,
            "note-1",
            Permission::Read,
            &[]
        ));
    }
}

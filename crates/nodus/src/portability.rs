//! Portability and extension-point traits for host integration.
//!
//! Provides the vocabulary-extension seam ([`SchemaProvider`]),
//! and interface-only contracts for storage ([`StorageProvider`]) and
//! policy evaluation ([`PolicyProvider`]) that are pending LP-3 graduation.
//! Each trait ships with a built-in no-op implementation that satisfies the
//! interface without I/O, matching the LP-2 pattern established by
//! [`crate::executor::StubProvider`] and [`crate::observability::NoopAuditProvider`].

use crate::executor::Value;

// ─── SchemaProvider ───────────────────────────────────────────────────────────

/// Vocabulary-extension seam for host-supplied command and variable names.
///
/// Return non-empty slices to extend the builtin vocabulary; return `&[]`
/// to leave it unchanged. The extensions are merged with the builtin baseline
/// by [`crate::vocab::Schema::with_provider`] — collisions with builtin names
/// are silently deduplicated.
pub trait SchemaProvider {
    /// Host-declared command names that extend the builtin vocabulary.
    fn host_commands(&self) -> &[&str];

    /// Additional reserved variable names beyond the builtin set.
    fn host_reserved_variables(&self) -> &[&str];
}

/// Built-in provider: no extensions; pure builtin vocabulary.
pub struct BuiltinSchemaProvider;

impl SchemaProvider for BuiltinSchemaProvider {
    fn host_commands(&self) -> &[&str] {
        &[]
    }

    fn host_reserved_variables(&self) -> &[&str] {
        &[]
    }
}

// ─── StorageProvider (pending LP-3) ──────────────────────────────────────────

/// Durable key/value store for cross-invocation state.
///
/// This interface is specified but executor integration is deferred until LP-3
/// is satisfied (two independent hosts require durable cross-invocation state).
pub trait StorageProvider {
    /// Persist a named value. `key` is host-defined; the runtime treats it as
    /// opaque.
    fn store(&self, key: &str, value: &Value);

    /// Retrieve a named value. Returns `None` if the key is absent.
    fn load(&self, key: &str) -> Option<Value>;
}

/// No-op storage: `store` discards all values; `load` always returns `None`.
pub struct NoopStorageProvider;

impl StorageProvider for NoopStorageProvider {
    fn store(&self, _key: &str, _value: &Value) {}

    fn load(&self, _key: &str) -> Option<Value> {
        None
    }
}

// ─── PolicyProvider (pending LP-3) ───────────────────────────────────────────

/// Runtime policy evaluation for host-defined gates.
///
/// This interface is specified but executor integration is deferred until LP-3
/// is satisfied (two independent hosts require policy evaluation beyond
/// `!!`-rules). The `evaluate` contract is boolean permit/deny only; spend
/// tracking and approval workflows are host-side concerns.
pub trait PolicyProvider {
    /// Evaluate a named policy gate.
    ///
    /// `gate` is the host-defined policy identifier. `context` is the current
    /// variable environment snapshot. Returns `true` to permit the action,
    /// `false` to deny it.
    fn evaluate(&self, gate: &str, context: &Value) -> bool;
}

/// No-op policy: permits every action unconditionally.
pub struct NoopPolicyProvider;

impl PolicyProvider for NoopPolicyProvider {
    fn evaluate(&self, _gate: &str, _context: &Value) -> bool {
        true
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_schema_provider_empty_commands() {
        let p = BuiltinSchemaProvider;
        assert!(p.host_commands().is_empty());
    }

    #[test]
    fn builtin_schema_provider_empty_reserved() {
        let p = BuiltinSchemaProvider;
        assert!(p.host_reserved_variables().is_empty());
    }

    #[test]
    fn noop_storage_load_returns_none() {
        let s = NoopStorageProvider;
        assert!(s.load("any_key").is_none());
    }

    #[test]
    fn noop_storage_store_no_panic() {
        let s = NoopStorageProvider;
        s.store("k", &Value::Null);
    }

    #[test]
    fn noop_policy_permits_all() {
        let p = NoopPolicyProvider;
        assert!(p.evaluate("any_gate", &Value::Null));
    }
}

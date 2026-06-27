//! Portability and extension-point traits for host integration.
//!
//! Provides the vocabulary-extension seam ([`SchemaProvider`]),
//! and interface-only contracts for storage ([`StorageProvider`]) and
//! policy evaluation ([`PolicyProvider`]) that are pending LP-3 graduation.
//! Each trait ships with a built-in no-op implementation that satisfies the
//! interface without I/O, matching the LP-2 pattern established by
//! [`crate::executor::StubProvider`] and [`crate::observability::NoopAuditProvider`].
//!
//! It also defines the LP-8 capability manifest ([`CapabilityManifest`]) and the
//! pre-run satisfiability gate ([`validate_manifest`]): a workflow declares the
//! extension roles, host commands, and named capabilities it needs, and the
//! runtime rejects fail-fast — before any step runs — when the active host
//! cannot satisfy them. The same manifest is the machine-checkable two-host
//! portability contract (LP-3).

use crate::ast::{CommandCall, Conditional, Stmt, WorkflowFile};
use crate::executor::Value;
use crate::vocab;
use std::collections::BTreeSet;

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

// ─── Capability Manifest (LP-8) ──────────────────────────────────────────────

/// Model-backed commands — those the executor dispatches to its
/// [`crate::executor::ModelProvider`]. A workflow invoking any of them requires
/// the [`ExtensionRole::Model`] role from its host.
const MODEL_COMMANDS: &[&str] = &["GEN", "ANALYZE"];

/// Dialog commands — those the executor dispatches to its [`crate::executor::DialogProvider`].
/// A workflow invoking one without a `+default` requires the [`ExtensionRole::Dialog`] role.
const DIALOG_COMMANDS: &[&str] = &["ASK", "CONFIRM"];

/// An LP-2 extension-point role a workflow may require from its host.
///
/// Roles name *capabilities*, never concrete host types (LP-1). They mirror the
/// extension-point taxonomy: model inference, audit tracing, durable storage,
/// policy evaluation, and host vocabulary extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionRole {
    /// Model inference backend ([`crate::executor::ModelProvider`]).
    Model,
    /// Execution-event audit sink ([`crate::observability::AuditProvider`]).
    Audit,
    /// Durable cross-invocation storage ([`StorageProvider`]).
    Storage,
    /// Runtime policy evaluation ([`PolicyProvider`]).
    Policy,
    /// Host vocabulary extension ([`SchemaProvider`]).
    Vocabulary,
    /// Human-in-the-loop dialog backend ([`crate::executor::DialogProvider`]).
    Dialog,
}

/// What a workflow declares it needs from its host to execute (LP-8).
///
/// Expressed only in terms of the extension-point taxonomy ([`ExtensionRole`])
/// and named schema capabilities — never a concrete host type (LP-1). An empty
/// manifest is satisfied by any host, so manifest-free and model-only workflows
/// stay runnable against the built-in in-process host without host wiring.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityManifest {
    roles: BTreeSet<ExtensionRole>,
    commands: BTreeSet<String>,
    capabilities: BTreeSet<String>,
}

impl CapabilityManifest {
    /// An empty manifest — satisfied by every host.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require an extension-point role.
    pub fn require_role(mut self, role: ExtensionRole) -> Self {
        self.roles.insert(role);
        self
    }

    /// Require a host-schema command by name.
    pub fn require_command(mut self, command: impl Into<String>) -> Self {
        self.commands.insert(command.into());
        self
    }

    /// Require a named capability.
    pub fn require_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.insert(capability.into());
        self
    }

    /// The required extension roles.
    pub fn roles(&self) -> &BTreeSet<ExtensionRole> {
        &self.roles
    }

    /// The required host-schema commands.
    pub fn commands(&self) -> &BTreeSet<String> {
        &self.commands
    }

    /// The required named capabilities.
    pub fn capabilities(&self) -> &BTreeSet<String> {
        &self.capabilities
    }

    /// Whether the manifest requires nothing (satisfied by any host).
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty() && self.commands.is_empty() && self.capabilities.is_empty()
    }

    /// Derive the manifest a workflow requires by walking its AST.
    ///
    /// A model-backed command (`GEN`/`ANALYZE`) requires [`ExtensionRole::Model`].
    /// A command outside the builtin vocabulary is a host-extension command: it
    /// requires [`ExtensionRole::Vocabulary`] and is recorded as a required
    /// command name. Builtin non-model commands need nothing — they are always
    /// available. Explicit DSL declaration (an `@needs` section) is a later
    /// refinement; this derives the manifest from invoked commands alone.
    pub fn from_workflow(ast: &WorkflowFile) -> Self {
        let mut calls: Vec<&CommandCall> = Vec::new();
        for step in &ast.steps {
            if let Some(body) = &step.body {
                collect_command_calls(body, &mut calls);
            }
            for sub in &step.sub_steps {
                collect_command_calls(sub, &mut calls);
            }
        }

        let mut manifest = Self::new();
        for cmd in calls {
            let name = cmd.name.as_str();
            if MODEL_COMMANDS.contains(&name) {
                manifest.roles.insert(ExtensionRole::Model);
            }
            // A dialog with a `+default` is resolved by the built-in synchronous
            // provider, so it needs no host dialog backend.
            if DIALOG_COMMANDS.contains(&name)
                && !cmd.modifiers.iter().any(|(k, _)| k == "+default")
            {
                manifest.roles.insert(ExtensionRole::Dialog);
            }
            if !vocab::is_known_command(name) {
                manifest.roles.insert(ExtensionRole::Vocabulary);
                manifest.commands.insert(cmd.name.clone());
            }
        }
        manifest
    }
}

/// Collect every command invocation reachable from a statement, descending into
/// conditionals, loops, and parallel branches.
fn collect_command_calls<'a>(stmt: &'a Stmt, out: &mut Vec<&'a CommandCall>) {
    match stmt {
        Stmt::Command(cmd) => out.push(cmd),
        Stmt::Conditional(cond) => collect_from_conditional(cond, out),
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                collect_command_calls(child, out);
            }
        }
        Stmt::UntilLoop(ul) => {
            for child in &ul.body {
                collect_command_calls(child, out);
            }
        }
        Stmt::Parallel(pb) => {
            for branch in &pb.branches {
                collect_command_calls(branch, out);
            }
        }
        Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
}

/// Collect command invocations from a conditional chain: inline action, nested
/// body, every `?ELIF` branch, and the trailing `?ELSE`.
fn collect_from_conditional<'a>(cond: &'a Conditional, out: &mut Vec<&'a CommandCall>) {
    if let Some(action) = &cond.action {
        out.push(action);
    }
    for child in &cond.body {
        collect_command_calls(child, out);
    }
    for elif in &cond.elif_branches {
        collect_from_conditional(elif, out);
    }
    if let Some(else_branch) = &cond.else_branch {
        collect_from_conditional(else_branch, out);
    }
}

/// What a host actually provides — the resolution surface a manifest is checked
/// against (LP-8). Hosts are built explicitly so the same struct serves both the
/// built-in in-process configuration and host-substitution tests (LP-3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostCapabilities {
    roles: BTreeSet<ExtensionRole>,
    commands: BTreeSet<String>,
    capabilities: BTreeSet<String>,
}

impl HostCapabilities {
    /// A host that provides nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// The built-in in-process host: it provides [`ExtensionRole::Model`] (the
    /// [`crate::executor::StubProvider`]), [`ExtensionRole::Audit`] (a sink is
    /// always wired), and [`ExtensionRole::Vocabulary`] (the builtin schema). It
    /// declares no host-extension commands and no named capabilities.
    pub fn builtin() -> Self {
        let mut host = Self::new();
        host.roles.insert(ExtensionRole::Model);
        host.roles.insert(ExtensionRole::Audit);
        host.roles.insert(ExtensionRole::Vocabulary);
        host
    }

    /// Declare that this host provides a role.
    pub fn with_role(mut self, role: ExtensionRole) -> Self {
        self.roles.insert(role);
        self
    }

    /// Declare that this host provides a host-schema command.
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.commands.insert(command.into());
        self
    }

    /// Declare that this host satisfies a named capability.
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.insert(capability.into());
        self
    }

    /// Does the host provide `role`?
    pub fn provides(&self, role: ExtensionRole) -> bool {
        self.roles.contains(&role)
    }

    /// Does the host provide the host-schema command `command`?
    pub fn has_command(&self, command: &str) -> bool {
        self.commands.contains(command)
    }

    /// Does the host satisfy the named capability `capability`?
    pub fn satisfies(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }
}

/// A single capability a host failed to provide, named precisely for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Missing {
    /// An extension-point role the host does not provide.
    Role(ExtensionRole),
    /// A host-schema command the host does not provide.
    Command(String),
    /// A named capability the host does not satisfy.
    Capability(String),
}

/// Resolve a manifest against a host: return every capability the host fails to
/// provide (LP-8). An empty result means the manifest is fully satisfiable and
/// the workflow may run; a non-empty result is the fail-fast rejection set. The
/// order is stable (roles, then commands, then capabilities, each sorted).
pub fn validate_manifest(manifest: &CapabilityManifest, host: &HostCapabilities) -> Vec<Missing> {
    let mut missing = Vec::new();
    for &role in &manifest.roles {
        if !host.provides(role) {
            missing.push(Missing::Role(role));
        }
    }
    for command in &manifest.commands {
        if !host.has_command(command) {
            missing.push(Missing::Command(command.clone()));
        }
    }
    for capability in &manifest.capabilities {
        if !host.satisfies(capability) {
            missing.push(Missing::Capability(capability.clone()));
        }
    }
    missing
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

    // ── LP-8 capability manifest ────────────────────────────────────────────

    #[test]
    fn manifest_default_is_empty() {
        let m = CapabilityManifest::new();
        assert!(m.is_empty());
        assert!(m.roles().is_empty());
        assert!(m.commands().is_empty());
        assert!(m.capabilities().is_empty());
    }

    #[test]
    fn host_caps_reports_wired_roles() {
        let host = HostCapabilities::new().with_role(ExtensionRole::Model);
        assert!(host.provides(ExtensionRole::Model));
        assert!(!host.provides(ExtensionRole::Storage));
    }

    #[test]
    fn builtin_host_provides_model_audit_vocabulary() {
        let host = HostCapabilities::builtin();
        assert!(host.provides(ExtensionRole::Model));
        assert!(host.provides(ExtensionRole::Audit));
        assert!(host.provides(ExtensionRole::Vocabulary));
        assert!(!host.provides(ExtensionRole::Storage));
        assert!(!host.provides(ExtensionRole::Policy));
    }

    #[test]
    fn validate_manifest_satisfiable_empty() {
        let manifest = CapabilityManifest::new().require_role(ExtensionRole::Model);
        let host = HostCapabilities::builtin();
        assert!(validate_manifest(&manifest, &host).is_empty());
    }

    #[test]
    fn validate_manifest_reports_exact_missing() {
        let manifest = CapabilityManifest::new().require_role(ExtensionRole::Storage);
        let host = HostCapabilities::builtin(); // builtin host provides no Storage
        let missing = validate_manifest(&manifest, &host);
        assert_eq!(missing, vec![Missing::Role(ExtensionRole::Storage)]);
    }

    #[test]
    fn validate_manifest_reports_missing_command_and_capability() {
        let manifest = CapabilityManifest::new()
            .require_command("HOST_CMD")
            .require_capability("vision");
        let host = HostCapabilities::builtin();
        let missing = validate_manifest(&manifest, &host);
        assert!(missing.contains(&Missing::Command("HOST_CMD".to_string())));
        assert!(missing.contains(&Missing::Capability("vision".to_string())));
    }

    #[test]
    fn manifest_from_model_workflow_requires_model() {
        let src = "\
§wf:m v1.0
@in: { query }
@out: $out
@steps:
  1. GEN($in.query) → $out
";
        let ast = crate::parser::Parser::parse(src).expect("parse");
        let manifest = CapabilityManifest::from_workflow(&ast);
        assert!(
            manifest.roles().contains(&ExtensionRole::Model),
            "GEN workflow must require the Model role: {manifest:?}"
        );
    }

    #[test]
    fn manifest_from_pure_workflow_is_empty() {
        // LOG is a builtin, non-model command → no roles required.
        let src = "\
§wf:p v1.0
@out: $out
@steps:
  1. LOG($out)
";
        let ast = crate::parser::Parser::parse(src).expect("parse");
        let manifest = CapabilityManifest::from_workflow(&ast);
        assert!(
            manifest.is_empty(),
            "pure builtin workflow needs nothing: {manifest:?}"
        );
    }

    #[test]
    fn manifest_from_host_command_requires_vocabulary() {
        // A command outside the builtin vocabulary requires Vocabulary + the command.
        // The host command is recognized only through schema-aware parsing.
        struct HostSchema;
        impl SchemaProvider for HostSchema {
            fn host_commands(&self) -> &[&str] {
                &["CUSTOM_CMD"]
            }
            fn host_reserved_variables(&self) -> &[&str] {
                &[]
            }
        }
        let schema = crate::vocab::Schema::with_provider(&HostSchema);
        let src = "\
§wf:h v1.0
@out: $out
@steps:
  1. CUSTOM_CMD($out) → $out
";
        let ast = crate::parser::Parser::parse_with_schema(src, &schema).expect("parse");
        let manifest = CapabilityManifest::from_workflow(&ast);
        assert!(manifest.roles().contains(&ExtensionRole::Vocabulary));
        assert!(manifest.commands().contains("CUSTOM_CMD"));
    }
}

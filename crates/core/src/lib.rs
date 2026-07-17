//! Cronus core engine library — the facade and composition root (§4.1, §4.2).
//!
//! Domain logic lives in `cronus-domain` (no I/O); the SQLite-backed default
//! for the user-data plane lives in `cronus-store-local`; the default for
//! the auth/identity planes lives in `cronus-auth-local`. This crate wires
//! them together and re-exports every module under its historical path, so
//! `cronus::memory::MemoryEntry`, `cronus::orchestration::…`, and every other
//! existing call site are unaffected by the split. Frontends (CLI, TUI, app)
//! depend on this crate; it has no presentation dependencies of its own.

pub use cronus_domain::{
    Capabilities, Engine, acp, agent_migration, agent_registry, automation, autonomy, backup,
    budget, checkpoint, config_hotreload, constitution, context_mgmt, context_router, deliberation,
    development_workflow, doctor, egress, error_reporting, exec_workspace, extensions, file_store,
    global_orch, hooks, inner_monologue, kanban, learning, lookahead, memory_capture,
    memory_intelligence, mission, notes, office_control, orchestration, paths, quality, redact,
    research, resource_sharing, roles, router, sandbox_policy, scheduler, secrets,
    self_improvement, session, skills, state, store, telemetry, tool_security, trigger_triage,
    version_control, voice, wiki_access, wiki_regen,
};

// The facade-wiring modules whose default implementation reaches into an
// adapter crate (`cronus-store-local` / `cronus-auth-local` /
// `cronus-model-local`) stay defined here — the tier model has no edge from
// `cronus-domain` to any adapter, so these shims cannot live there.
pub mod auth;
pub mod context_compaction;
pub mod inbox;
pub mod memory;
pub mod model_bridge;
pub mod workspace;

pub use context_compaction::TransportCompactor;
/// The local model-transport adapter, re-exported so a host can construct a
/// `contract::InferenceBackend` (an `EndpointProfile`) and hand it to the
/// engine — the wired transport surface (l1-model-runtime §4.1).
pub use cronus_model_local as model;
pub use model_bridge::NodusModelBridge;

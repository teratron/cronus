//! Loop runner — the single governed place every autonomous loop runs.
//!
//! An execution loop re-attempts a fixed task until an oracle says it is
//! done; an evolution loop deliberately changes the artifacts it works from
//! between iterations to improve. Both declare a class, a mutation manifest,
//! an oracle, and a ceiling before they run — the governor is the one place
//! that enforces all four.

pub mod escalate;
pub mod evolution;
pub mod execution;
pub mod governor;
pub mod objective;
pub mod oracle;
pub mod spec;

pub use escalate::{
    EscalationOutcome, HeldOutMeasurement, NoveltySource, RefusalReason, escalate,
    is_separated_oracle,
};
pub use evolution::{
    EvolutionBackend, EvolutionReport, GenerationRecord, GenerationVerdict, run_evolution,
    score_generation,
};
pub use execution::{ExecutionBackend, ExecutionReport, run_execution};
pub use governor::{
    ControlFlow, IllegalMutation, Mutation, StopReason, TurnResult, check_ceiling, guard_writes,
    judge,
};
pub use objective::{objective_context_entry, re_project_objective, update_progress};
pub use oracle::{
    ApprovalContext, HumanFallback, resolve_human_oracle, select_judge_binding, select_oracle,
};
pub use spec::{
    Ceiling, LedgerEntry, LoopClass, LoopOutcome, LoopSpec, MutableArtifact, MutationManifest,
    ObjectiveSlot, Oracle, Verdict, WorkspaceKind,
};

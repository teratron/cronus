//! Long-term memory subsystem.
//!
//! Memories are bi-temporal (valid_at / created_at), trust-scored,
//! full-text searchable via FTS5, and linked into session chains.
//! HRR (holographic reduced representation) encoding is a seam —
//! the stub returns a zeroed vector; the real encoder ships with
//! sqlite-vec in a later phase.
//!
//! The SQLite-backed store, its Bellman-propagation/trust-scoring helpers,
//! and at-rest encryption moved to `cronus-store-local` (§4.2, §4.6) — the
//! adapter implementing the `MemorySearch`/`UserDataStore` seam. Re-exported
//! here so every existing call site (`crate::memory::MemoryStore`,
//! `cronus::memory::MemoryStore`, …) is unaffected. `consolidation` has no
//! infrastructure dependency and stays here, in the domain tier.

pub mod consolidation;

pub use cronus_store_local::memory::{
    CodeChangeType, MemoryError, MemoryStore, Result, SuggestedAction, TrustUpdate, chain,
    encryption, store, trust,
};

// ── Shared with the ports tier ───────────────────────────────────────────────
//
// MemoryId/MemoryKind/MemorySource/VerificationState/MemoryEntry moved to
// `cronus-contract` (§4.2) — `MemorySearch`/`UserDataStore`
// carry `MemoryEntry` across the domain/adapter boundary, so its shape lives
// where both sides can see it. Re-exported here so every existing call site
// (`crate::memory::MemoryEntry`, `cronus::memory::MemoryEntry`, …) is unaffected.
pub use cronus_contract::{MemoryEntry, MemoryId, MemoryKind, MemorySource, VerificationState};

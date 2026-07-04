//! Notes — persistent structured artifacts with CRDT concurrent merge, append-only
//! version history, and non-destructive soft deletion.
//!
//! The content is modeled as a set of insertion operations keyed by a globally
//! unique, totally-ordered op id `(site, counter)`. Merge is set union — commutative,
//! associative, and idempotent — so replicas that apply disjoint edits converge to
//! the same rendered state regardless of merge order (the CRDT property). Rendering
//! walks the ops in op-id order.
//!
//! Production storage is a ProseMirror JSON content tree + a Yjs binary update log
//! (a seam); here the convergence, history, and soft-delete algebra are implemented
//! and tested with a minimal insertion CRDT.

use std::collections::BTreeMap;

/// A globally unique, totally-ordered operation id: `(site, counter)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpId {
    pub site: u64,
    pub counter: u64,
}

/// One immutable version snapshot in the append-only history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub rendered: String,
    pub at: u64,
}

/// A CRDT-backed note.
#[derive(Debug, Clone, Default)]
pub struct Note {
    /// Insertion ops, ordered by op id (the convergence key).
    ops: BTreeMap<OpId, String>,
    deleted: bool,
    history: Vec<Version>,
}

impl Note {
    pub fn new() -> Self {
        Note::default()
    }

    /// Apply a local insertion op. Idempotent on the op id (re-applying is a no-op),
    /// which is what makes merge idempotent.
    pub fn insert(&mut self, id: OpId, fragment: &str) {
        self.ops.entry(id).or_insert_with(|| fragment.to_string());
    }

    /// Merge another replica's ops into this one (CRDT union). Commutative,
    /// associative, idempotent — order-independent convergence.
    pub fn merge(&mut self, other: &Note) {
        for (id, frag) in &other.ops {
            self.ops.entry(*id).or_insert_with(|| frag.clone());
        }
    }

    /// Render the note by concatenating fragments in op-id order.
    pub fn render(&self) -> String {
        self.ops.values().cloned().collect()
    }

    /// Whether the note is soft-deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    /// Soft-delete: non-destructive: the content and history are retained (VI-style
    /// recoverability). Rendering is unaffected — deletion is a flag, not erasure.
    pub fn soft_delete(&mut self) {
        self.deleted = true;
    }

    /// Restore a soft-deleted note.
    pub fn restore(&mut self) {
        self.deleted = false;
    }

    /// Snapshot the current rendered state into the append-only history.
    pub fn snapshot(&mut self, at: u64) {
        let rendered = self.render();
        self.history.push(Version { rendered, at });
    }

    pub fn history(&self) -> &[Version] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(site: u64, counter: u64) -> OpId {
        OpId { site, counter }
    }

    #[test]
    fn concurrent_merges_converge_regardless_of_order() {
        // CRDT convergence: two replicas make disjoint edits; merging in either
        // direction yields the identical rendered state.
        let mut a = Note::new();
        a.insert(op(1, 0), "Hello ");
        let mut b = Note::new();
        b.insert(op(2, 0), "World");

        let mut a_then_b = a.clone();
        a_then_b.merge(&b);
        let mut b_then_a = b.clone();
        b_then_a.merge(&a);

        assert_eq!(a_then_b.render(), b_then_a.render());
        // Op-id order (site 1 before site 2) gives a deterministic render.
        assert_eq!(a_then_b.render(), "Hello World");
    }

    #[test]
    fn merge_is_idempotent() {
        let mut a = Note::new();
        a.insert(op(1, 0), "x");
        let mut b = Note::new();
        b.insert(op(2, 0), "y");
        a.merge(&b);
        let once = a.render();
        a.merge(&b); // merging again changes nothing
        a.merge(&b);
        assert_eq!(a.render(), once);
    }

    #[test]
    fn duplicate_op_id_is_a_noop() {
        let mut n = Note::new();
        n.insert(op(1, 0), "first");
        n.insert(op(1, 0), "second"); // same op id -> ignored
        assert_eq!(n.render(), "first");
    }

    #[test]
    fn soft_delete_is_non_destructive_and_recoverable() {
        let mut n = Note::new();
        n.insert(op(1, 0), "content");
        n.soft_delete();
        assert!(n.is_deleted());
        // Content survives a soft delete.
        assert_eq!(n.render(), "content");
        n.restore();
        assert!(!n.is_deleted());
    }

    #[test]
    fn history_is_append_only_snapshots() {
        let mut n = Note::new();
        n.insert(op(1, 0), "v1");
        n.snapshot(100);
        n.insert(op(1, 1), " v2");
        n.snapshot(200);
        assert_eq!(n.history().len(), 2);
        assert_eq!(n.history()[0].rendered, "v1");
        assert_eq!(n.history()[1].rendered, "v1 v2");
    }
}

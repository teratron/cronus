---
phase: 11
name: "Content, Sharing & Dev-Workflow Subsystems"
status: In Progress
subsystem: "crates/core (resource_sharing, notes, file_store, development_workflow)"
requires:
  - "Phase 4: memory-store, agent-session"
  - "Phase 5: extension-registry"
provides:
  - "resource_sharing: uniform access-grant model — has_access resolution (owner→user→group→public), write-implies-read, additive grants, audit events (RS-1…RS-8)"
key_files:
  created:
    - crates/core/src/resource_sharing.rs
  modified:
    - crates/core/src/lib.rs
patterns_established:
  - "Access resolution as pure in-memory algebra; SQLite access_grant table is the deferred seam"
duration_minutes: ~
---

# Stage 11 Tasks — Content, Sharing & Dev-Workflow Subsystems

**Phase:** 11
**Status:** In Progress
**Strategic Goal:** The access-controlled content stores plus the bundled dev-workflow skill catalog. Natural build order: resource-sharing (the access layer) before notes/file-store; development-workflow is independent. All L2 specs are already Stable. Domain-logic-first per the Phase-9/10 precedent — each subsystem's algorithm is implemented and tested against in-memory state; SQLite/CRDT/storage-backend integration is deferred as documented seams.

## Atomic Checklist

- [x] [T-11A01] Resource Sharing — access-grant model + has_access resolution + audit
- [x] [T-11AT01] Validation — RS-1…RS-8
- [ ] [T-11B01] File Store — content-addressed dedup + reference-tracking GC + immutable blobs
- [ ] [T-11BT01] Validation — content addressing + GC
- [ ] [T-11C01] Notes — schema + CRDT update log + version history + soft-delete
- [ ] [T-11CT01] Validation — CRDT merge convergence + soft-delete
- [ ] [T-11D01] Development Workflow — skill catalog + implementer/reviewer dispatch + progress ledger
- [ ] [T-11DT01] Validation — DW-1…DW-10

## Detailed Tracking

### Track A — Resource Sharing (`crates/core`, DONE)

The access foundation for notes/files/knowledge; `Implements: l1-resource-sharing.md`.

#### [T-11A01] Access-grant model + has_access resolution + audit

- **Spec:** l2-resource-sharing.md §4.2, §4.3, §4.4 (RS-1…RS-8)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus --lib resource_sharing::` → 8 passed / 0 failed; clippy `-D warnings` clean; fmt clean.
- **Notes:** In-memory `GrantStore` (SQLite `access_grant` table is the seam). `has_access` resolves owner→user→group→public (RS-5 first, RS-6 additive); `Permission: Read < Write` gives write-implies-read (RS-3); absence = private (RS-4); every change emits a `GrantAudit` event (RS-7); `ResourceKind` compile-time enum (RS-8).
- **Changes:** `crates/core/src/resource_sharing.rs` (new); `lib.rs` `pub mod resource_sharing`. 8 unit tests.

#### [T-11AT01] Validation — Resource Sharing

- **Goal:** Verify RS-1…RS-8. **Method:** `cargo test -p cronus --lib resource_sharing`. **Status:** Done

### Track B — File Store (`crates/core`, pending)

Content-addressed blob store; `Implements: l1-file-management.md`; depends on resource-sharing.

#### [T-11B01] Content-addressed dedup + reference-tracking GC + immutable blobs

- **Spec:** l2-file-store.md
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus --lib file_store::` green — identical content dedups to one blob (SHA-256 addressing, hashing a seam); reference count tracks holders; a blob with zero references is GC-eligible; blobs are immutable.
- **Notes:** Metadata decoupled from blobs; access-controlled download via resource-sharing.

#### [T-11BT01] Validation — File Store

- **Goal:** Content addressing + GC. **Method:** `cargo test -p cronus --lib file_store`. **Status:** Todo

### Track C — Notes (`crates/core`, pending)

CRDT-backed notes; `Implements: l1-notes.md`; depends on resource-sharing.

#### [T-11C01] Schema + CRDT update log + version history + soft-delete

- **Spec:** l2-notes.md
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus --lib notes::` green — concurrent CRDT updates converge to one state (Yjs semantics modeled); version history append-only; soft-delete is non-destructive + recoverable.
- **Notes:** ProseMirror JSON content tree + Yjs update log are seams; the merge-convergence + soft-delete algebra is implemented here.

#### [T-11CT01] Validation — Notes

- **Goal:** CRDT merge convergence + soft-delete. **Method:** `cargo test -p cronus --lib notes`. **Status:** Todo

### Track D — Development Workflow (`crates/core`, pending)

Bundled skill catalog; `Implements: l1-development-workflow.md`; depends on extension-registry + agent-session.

#### [T-11D01] Skill catalog + implementer/reviewer dispatch + progress ledger

- **Spec:** l2-development-workflow.md
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus --lib development_workflow::` green — five-stage pipeline (Design→Plan→Execute→Review→Deliver) with two-stage quality gate; progress ledger is durable; human checkpoints gate stage advance (DW-1…DW-10).
- **Notes:** Independent of Tracks A–C.

#### [T-11DT01] Validation — Development Workflow

- **Goal:** DW-1…DW-10. **Method:** `cargo test -p cronus --lib development_workflow`. **Status:** Todo

## Notes

- **Execution mode**: Parallel (C3). Track A (Done) is the access foundation; B and C depend on it; D is independent.
- **Domain-logic-first**: SQLite schemas, Yjs CRDT binary format, and storage backends are deferred seams; the resolution/merge/GC/pipeline algebra is what these tasks implement and test.

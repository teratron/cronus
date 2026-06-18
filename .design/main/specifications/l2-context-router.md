# Context Router

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-routing.md

## Overview

The concrete realization of the three context routers under one roof: **memory routing** (which memories to recall, where to write), **rules routing** (which rules apply to a context), and **session routing** (continue vs new, retire stale). Each applies the smart-router pattern with most-specific-first resolution.

## Related Specifications

- [l1-routing.md](l1-routing.md) - The router pattern this implements.
- [l2-memory-store.md](l2-memory-store.md) - Memory routing refines the store's recall/write path.
- [l1-storage-model.md](l1-storage-model.md) - Scope levels these routers resolve over.
- [l2-cli.md](l2-cli.md) - Command grammar standard.

## 1. Motivation

Memory, rules, and sessions all need "pick the right scope/target for this context." Consolidating them keeps the scope-resolution logic consistent (most-specific-first) and avoids three near-identical thin specs.

## 2. Constraints & Assumptions

- All three run on the hot path and resolve quickly.
- Scope resolution is most-specific-first (employee/role -> workspace -> global).
- Routers read existing state; they introduce no new authoritative store.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| RTG-1 Multi-signal | Memory routing fuses similarity + tags + utility; session routing uses recency + topic match. |
| RTG-2 Fallback | If no specific match, fall back to the next-broader scope (then a safe default). |
| RTG-3 Short-circuit | Recently-resolved context (same query/scope) may be reused within a turn. |
| RTG-4 Most-specific-first | Memory and rules resolve role -> workspace -> global; specific overrides general. |
| RTG-5 Configurable | Thresholds (similarity floor, staleness window) live in config. |
| RTG-6 Privacy | Routers operate on local state; no client data leaves the device. |
| RTG-7 Bounded & traceable | Recall is token-budgeted; routing choices are recorded. |
| RTG-8 Lifecycle | Session routing decides continue/new and retires stale sessions (MEM-5). |

## 4. Detailed Design

### 4.1 Memory routing

Refines the memory store: on **recall**, fuse semantic + lexical + tags across scopes and resolve most-specific-first (role -> workspace -> global), token-budgeted. On **write**, classify the fact's scope and route it to the owning store. (Defers to `l2-memory-store.md` for the store mechanics.)

### 4.2 Rules routing

Given a context (which office, which role, which task), select the applicable rules by scope, most-specific-first: role rules override workspace rules override global rules. Conflicts resolve by specificity; equal-scope conflicts are surfaced, not guessed. <!-- TBD: tie-break policy for equal-scope rule conflicts -->

### 4.3 Session routing

```mermaid
graph TD
    REQ[incoming work/turn] --> MATCH{recent + topically related session?}
    MATCH -->|yes| CONT[continue that session]
    MATCH -->|no| NEW[start a new session]
    BG[periodic] --> STALE{session stale?}
    STALE -->|yes| RETIRE[retire/prune session]
```

Continue a session when it is recent and on-topic; otherwise start fresh; retire stale sessions (consistent with MEM-5 prune and OFF "удаление ненужных сессий").

### 4.4 Command surface

Context routing is mostly automatic (no dedicated client commands); its behavior is observable via memory and status commands. Routing decisions are recorded for tracing (RTG-7).

## 5. Drawbacks & Alternatives

- **Wrong session continuation:** a bad topic match resumes the wrong thread; mitigated by a conservative match threshold (favor new on doubt).
- **Rule conflict ambiguity:** equal-scope conflicts need a tie-break (see §4.2 TBD).
- **Alternative — separate specs per router:** rejected for v0.1.0 to avoid fragmentation; can be split later if any router grows large.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ROUTING]` | `.design/main/specifications/l1-routing.md` | Invariants this implements |
| `[MEMORY]` | `.design/main/specifications/l2-memory-store.md` | Memory store mechanics this routes over |
| `[STORAGE]` | `.design/main/specifications/l1-storage-model.md` | Scope levels resolved |

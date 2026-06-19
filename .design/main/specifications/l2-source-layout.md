# Source Layout (Monorepo)

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-architecture.md

## Overview

The development-time organization of the Cronus repository: a polyglot monorepo with a Rust workspace for the core and binaries, an apps layer for the desktop/mobile shell, and a JS/TS package layer for the UI. It maps the architecture's layers (core library + CLI/TUI/GUI frontends) onto concrete workspace members and clarifies that the workflow runtime is an external crate dependency.

> Scope: this is the **developer/source** layout. The **user/install** layout (program vs state tiers) is specified separately in [l2-filesystem-layout.md](l2-filesystem-layout.md).

## Related Specifications

- [l1-architecture.md](l1-architecture.md) - The layer model (core + frontends) realized here.
- [l2-technology-stack.md](l2-technology-stack.md) - Monorepo tooling (moon/Nx) + Rust workspace + Tauri + React.
- [l2-workflow-runtime.md](l2-workflow-runtime.md) - The workflow runtime is an external crate the core depends on.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - The complementary user-install layout.

## 1. Motivation

The architecture separates a reusable core from thin frontends; the source tree should make that boundary obvious and enforce dependency direction. A polyglot monorepo also has to host Rust and JS side by side without one tool pretending to own the other (see the stack spec's monorepo verdict).

## 2. Constraints & Assumptions

- Dependency direction is inward: frontends/apps depend on the core; the core depends on nothing in this product (INV-1/INV-2).
- Rust members live in a Cargo workspace; JS members in a pnpm workspace; the polyglot runner (moon/Nx) sequences both.
- The workflow runtime is an **external** Rust crate (its own repository), consumed as a dependency — not vendored into the tree (per the adopted decision).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| INV-1 Embeddable core | `crates/core` is a library crate with no frontend dependencies; apps/bins depend on it. |
| INV-2 Logic in core only | UI (`packages/ui`) and shells (`apps/`, `crates/cli`, `crates/tui`) hold no domain logic. |
| INV-3 Frontend interchangeability | CLI, TUI, and the app are separate members over the same `core`. |
| INV-4 Hub-and-spoke | `apps/desktop` can host the always-on engine; the same shell builds the mobile thin client. |

## 4. Detailed Design

### 4.1 Repository layout

```plaintext
cronus/
├── crates/                 # Rust workspace (Cargo)
│   ├── core/               # engine library: orchestration, memory, scheduler, routers, quality, board, office projection
│   ├── cli/                # `cronus` binary (depends on core)
│   └── tui/                # `cronus-tui` binary (depends on core)
├── apps/
│   └── desktop/            # Tauri v2 shell; src-tauri depends on core (desktop + mobile targets)
├── packages/               # JS/TS workspace (pnpm)
│   └── ui/                 # React 19 + Vite frontend (office view, kanban board, dashboard, editor)
├── .design/                # SDD artifacts (engine-managed; excluded from product releases)
└── (build config: Cargo workspace, pnpm-workspace, moon/Nx)
```

External dependency (not in-tree): the **workflow-runtime crate** (its own repository), referenced by `crates/core` in its `Cargo.toml`.

### 4.2 Dependency direction

```mermaid
graph TD
    UI[packages/ui] --> DESKTOP[apps/desktop]
    DESKTOP --> CORE[crates/core]
    CLI[crates/cli] --> CORE
    TUI[crates/tui] --> CORE
    CORE --> WFL[(external workflow-runtime crate)]
    CORE --> DEPS[(sqlite-vec, llama.cpp FFI, ...)]
```

Arrows point inward to `core`; `core` points only outward to libraries, never to a frontend (INV-1/INV-2).

### 4.3 Tooling split (polyglot)

Cargo owns Rust builds/caching; pnpm + the polyglot runner (moon/Nx) own JS and sequence the Tauri build; the runner does not try to cache Rust output (delegated to Cargo/sccache). See [l2-technology-stack.md](l2-technology-stack.md) §monorepo.

### 4.4 Migration from the initial flat layout

The initial placeholder `src/{app,cli,core,dashboard,kanban,office,tui}` mixed Rust modules with UI views. It is superseded by this layout: domain logic → `crates/core`; CLI/TUI → `crates/{cli,tui}`; shell → `apps/desktop`; `dashboard`/`office`/`kanban` were **UI views** → `packages/ui`. <!-- TBD: confirm crate granularity — single `core` crate vs split (engine/memory/scheduler) sub-crates -->

## 5. Drawbacks & Alternatives

- **Root-level crates/apps/packages vs everything under src/:** root-level is the polyglot-monorepo norm and keeps Rust/JS workspaces clean; chosen over a single `src/`.
- **External workflow-runtime crate:** adds a cross-repo dependency to manage, but preserves the runtime's reuse beyond Cronus (its design goal).
- **Alternative — vendor the runtime in `crates/`:** available as a fallback if cross-repo coordination becomes painful.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ARCH]` | `.design/main/specifications/l1-architecture.md` | Layer model realized here |
| `[STACK]` | `.design/main/specifications/l2-technology-stack.md` | Monorepo tooling + Rust workspace |
| `[USER-LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | Complementary user-install layout |

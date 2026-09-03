# UI (React 19 + Vite)

The graphical frontend: office view, Kanban board, dashboard, chat/briefings, editor.
Presentation only — calls `core` over the Tauri IPC bridge. No domain logic.

## Module tiers

`src/` is partitioned into four tiers. A module may import only from a tier
**below** its own — never sideways within the surfaces tier, and never upward.

| Tier | Holds | May import from |
| --- | --- | --- |
| **composition root** (`index.ts`, `App.tsx`, `styles.css`) | the package's public entry point and the top-level app component | shell, surfaces, shared |
| **shell** (`shell/`) | the application frame — building frame, floor tabs, sidebar, docks, overlays, command palette, surface router, workbench composer | surfaces, shared |
| **surfaces** (`surfaces/`) | renderable destinations — dashboard, office view, and each surface the navigation catalog names | shared |
| **shared** (`shared/`) | the core bridge client, theme and token resolution, scheme manifests, localization, the navigation and surface catalogs, geometry helpers | shared (must stay acyclic) |

The shared tier is a leaf: it names no surface, shell, or root module in an
import or a type position. Exactly one shared module — `shared/bridge.ts` —
performs IPC invocation; every other module receives core data as props or
through that module's typed client.

Grouping is by surface, never by file kind: there is no package-level
`components/`, `hooks/`, or `types/` folder. A surface stays a single file
until it acquires a **second module of its own** (a panel, a sub-component, a
local hook) — at that point it becomes a folder with an `index.ts` that
declares its public API: the mount component, its props, and the core
projection types it consumes, and nothing else. A folder is minted for bounded
scope, never for size.

A co-located test takes the tier of the module it sits with. The rules are
checked by the repo's structural analyzer against `.fallowrc.jsonc`; run
`pnpm exec fallow guard <file>` to see which tier a file is in and what it may
import before editing it.

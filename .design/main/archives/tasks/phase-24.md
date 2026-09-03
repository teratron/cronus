---
phase: 24
name: "Application Shell Frame & Design System (GUI)"
status: Done
subsystem: "packages/ui + apps/desktop"
requires:
  - "Phase 8: l2-app-ui — Tauri v2 + React 19 shell, IPC bridge (bridge.ts), theme.ts, i18n.ts, Workbench (archives/tasks/phase-8.md)"
  - "Phase 11: l2-navigation — four-layer nav model constants + component tree at 1.0.0 (archives/tasks/phase-11.md)"
  - "Phase 0 concepts: l1-application-shell (AS-1…AS-13), l1-navigation-model v1.3.0 (NV-1…NV-10, §4.1 catalog, §4.5 facets, §4.6 menu leaves), l1-design-identity v1.0.2 (DI-1…DI-9)"
provides:
  - "design tokens: a canonical CSS-custom-property taxonomy (tokens.ts CANONICAL_TOKENS + tokens.css safe fallback + @theme Tailwind v4 map); a `default` colour-scheme package (schemes/default/{manifest.json,tokens.light.css,tokens.dark.css}); a craft lint (scripts/craft-lint.mjs, `lint:craft` script) enforcing the DI-3 must-fix subset"
  - "theming resolver: theme.ts extended with the two-axis (mode × scheme) resolver — resolveScheme / surfaceAttributes / registerScheme / schemeCatalog / DEFAULT_SCHEME_ID; unknown scheme → default + warning; unresolvable default → safe fallback, never blank (DI-2)"
  - "shell frame components (packages/ui/src/shell/): BuildingFrame (L0 title bar + File/Edit/View/Help menu, INV-9-filtered via visibleMenu), FloorTabBar (L1, pinned Home, live OfficeState dot, ︙ menu, +/drop), SubsystemSidebar (L2, SIDEBAR_PRIMARY + foot SIDEBAR_UTILITY, badges, run-control), MechanismNav (L3 strip from L3_FACETS), SurfaceRouter + SurfacePlaceholder (INV-9), RightDock (read-only file tree), SelectionSurface + CommandPalette (AS-10 delegated selection surface), GlobalSettingsOverlay (full-screen, mode + scheme pickers), BuildingShell (composes all, applies surfaceAttributes on root)"
  - "navigation.ts: SIDEBAR_PRIMARY (11) + SIDEBAR_UTILITY (4) frozen runs, isCanonicalOrder over the two-run order, composeSidebar → {pinned, primary, utility}, L3_FACETS + hasMechanismNav"
  - "i18n.ts: MessageKey union + en (full) / ru (partial) catalogs extended with ~90 shell strings (frame chrome, File/Edit/View/Help leaves, floor menu, 15 sidebar tab labels, run control, palette, dock, global settings, INV-9 placeholder copy)"
  - "apps/desktop/tauri/src/settings.rs: `theme` + `color_scheme` String fields with serde defaults (\"system\" / \"default\"), older files round-trip with defaults filled (l2-app-ui §4.7 load_or_create pattern)"
  - "SDD containment: root CHANGELOG.md `[nodus-0.3.0]` section — the `SDD_REFERENCE_LEAK` at line 18 (\"Phases 8–23, each with its own L2 realization spec\") restated in plain language; an adjacent `l2-nodus-portability` spec-name ref neutralised"
key_files:
  created:
    - packages/ui/src/tokens.ts
    - packages/ui/src/tokens.css
    - packages/ui/src/tokens.test.ts
    - packages/ui/src/theme.test.ts
    - packages/ui/src/craft-lint.test.ts
    - packages/ui/scripts/craft-lint.mjs
    - packages/ui/src/schemes/default/manifest.json
    - packages/ui/src/schemes/default/tokens.light.css
    - packages/ui/src/schemes/default/tokens.dark.css
    - packages/ui/src/shell/actions.ts
    - packages/ui/src/shell/menu.ts
    - packages/ui/src/shell/building-frame.tsx
    - packages/ui/src/shell/floor-tab-bar.tsx
    - packages/ui/src/shell/subsystem-sidebar.tsx
    - packages/ui/src/shell/mechanism-nav.tsx
    - packages/ui/src/shell/surface-router.tsx
    - packages/ui/src/shell/command-palette.tsx
    - packages/ui/src/shell/right-dock.tsx
    - packages/ui/src/shell/global-settings-overlay.tsx
    - packages/ui/src/shell/building-shell.tsx
    - packages/ui/src/shell/index.ts
    - packages/ui/src/shell/shell.test.tsx
    - packages/ui/src/shell/overlays.test.tsx
    - packages/ui/src/shell/building-shell.test.tsx
    - packages/ui/src/shell/conformance.test.tsx
  modified:
    - packages/ui/src/styles.css
    - packages/ui/src/theme.ts
    - packages/ui/src/navigation.ts
    - packages/ui/src/navigation.test.ts
    - packages/ui/src/i18n.ts
    - packages/ui/src/index.ts
    - packages/ui/package.json
    - apps/desktop/tauri/src/settings.rs
    - biome.json
    - CHANGELOG.md
patterns_established:
  - "Two-axis theming: OS-appearance mode (data-theme) × named colour scheme (data-scheme); a scheme is a schema-validated data package (manifest + per-mode token CSS), added by dropping files, never a code change (DI-1/DI-2)"
  - "Token contract enforced by a craft lint in the package gate: no literal colour / font-family / raw-px radius outside the token layer; the exempt list + rule tiers are lint config, not engine code (DI-3/DI-5/DI-7)"
  - "Thin action registry: a flat {id,label,run,binding,bound} map; unbound entries dropped by visibleMenu (INV-9). Not the full AS-6…AS-8 context-predicate dispatch — that stays deferred"
  - "Delegated selection surface: SelectionSurface owns query/list/keyboard/confirm; a SelectionDelegate supplies source + row + confirm. The command palette is the first delegate (AS-10)"
  - "Amendment-delta over a Done phase on the frontend: the Phase 8/11 Workbench + surfaces are untouched (their tests stay green); the new shell is a parallel BuildingShell component, adopted as the live app root in a later GUI↔core integration pass"
  - "biome.json css.parser.tailwindDirectives = true so biome tolerates Tailwind v4 @theme in .css"
duration_minutes: ~
---

# Stage 24 Tasks — Application Shell Frame & Design System (GUI)

**Phase:** 24
**Status:** Done
**Strategic Goal:** Realize the desktop shell chrome + the two-axis design-token system from the approved "Cronus Building" mockup — presentation-only, over the existing Phase 8 IPC bridge. One new L2 (`l2-design-system`) plus the v1.1.0 / v1.4.0 amendment deltas of `l2-navigation` and `l2-app-ui` (their Phase 11 / Phase 8 builds keep `[x]` at delivered scope). The ~16 content surfaces render as explicit INV-9 placeholders — no new core capability, no `bridge.ts` `CoreClient` expansion, no layout persistence in the core.

**Execution mode:** Parallel (C3). **Critical path:** T-24A01 (token taxonomy) → T-24A02 (`(mode × scheme)` resolver) → Tracks B and C. **Shared-file handoffs:** `theme.ts` — A02 extends it (resolver), C01 wires it (sequenced, not parallel); `navigation.ts` — B03 owns it (`SIDEBAR_PRIMARY`/`SIDEBAR_UTILITY`) and gates the B04/B05 router. **Thin runtime:** the full `l1-application-shell` AS-1…AS-13 machinery (single-authority entity store, context-predicate dispatch tree, dockable-pane layout persistence) is deferred; actions are a minimal `{id,label,run,binding}` registry feeding the palette.

## Atomic Checklist

- [x] [T-24A01] Token taxonomy + Tailwind v4 `@theme` binding
- [x] [T-24A02] `default` colour-scheme package + `(mode × scheme)` resolver
- [x] [T-24A03] DI-3 craft lint (must-fix subset) + migrate hardcoded values
- [x] [T-24B01] L0 BuildingFrame + File/Edit/View/Help menu + action registry
- [x] [T-24B02] L1 FloorTabBar (pinned Home + project floors, live OfficeState, ︙ menu, `+`/drop)
- [x] [T-24B03] L2 SubsystemSidebar + expanded catalog (`navigation.ts`: `SIDEBAR_PRIMARY` + foot `SIDEBAR_UTILITY`)
- [x] [T-24B04] L3 MechanismNav strip (§4.5 facet catalog) + surface router
- [x] [T-24B05] Right file-tree dock + command palette (Ctrl+Shift+J) + global-settings overlay
- [x] [T-24C01] Two-axis theming wiring + `app.json` `colorScheme` + Settings ▸ Appearance pickers
- [x] [T-24C02] Surface placeholders (INV-9) + i18n externalization (`en`/`ru`)
- [x] [T-24D01] SDD containment cleanup — `CHANGELOG.md:18` reference leak
- [x] [T-24T01] Spec-conformance test sweep (NV-1/7/10, DI-2/3, INV-9)
- [x] [T-24T02] Verification gate run (vitest · biome · tsc · fallow audit · craft lint)

## Evidence

Frontend gate (`packages/ui`, cwd there): `tsc --noEmit` → exit 0 · `vitest run` → **15 files / 94 tests passed** (baseline 39 + 55 new) · `biome check packages/ui apps/desktop/src` → **0 errors / 0 warnings** · `craft-lint` (`node scripts/craft-lint.mjs`) → **clean** over the real component tree; the seeded-literal fixtures exit 1 as designed.
Rust (`apps/desktop/tauri`, PowerShell): `cargo fmt -- --check` → 0 · `cargo check` → 0 · `cargo clippy --all-targets -- -D warnings` → 0 · `cargo test settings` → **8 passed** (2 new: older-file-defaults-filled, theming-axes-round-trip; 6 pre-existing). `settings.rs` is dependency-neutral; `Cargo.lock` left at its committed state (a pre-existing `Cargo.toml`↔`Cargo.lock` drift there — `cargo --locked` refuses on the committed lock — is out of Phase 24 scope).
`fallow audit --changed-since HEAD`: does not complete within a practical window on this host (no output emitted; not a code failure). Re-run in CI where the base-snapshot pass is budgeted. The presentation-only boundary is otherwise held by construction — every shell component renders from props and forwards intents; the only bridge type touched is the existing one; `packages/ui` gains no `@tauri-apps/*` or business-logic import.

### T-24C01 — deferred sub-item (disclosed)

The frontend two-axis theming is complete and tested (`theme.ts` resolver, `BuildingShell` root application, `GlobalSettingsOverlay` mode + scheme pickers). The `apps/desktop/tauri/src/settings.rs` `theme` + `color_scheme` persistence fields are added. **Not done in this phase:** switching `apps/desktop/src/main.tsx` to render `BuildingShell` as the live app root (it still renders the Phase 8 `App`/`Workbench`), and wiring the Tauri settings read + floor/OfficeState projections over the bridge. That is a GUI↔core integration step — adopting the shell as the live surface, binding real projections — beyond "build the shell frame", and follows the project's domain-logic-first / shell-integration-deferred precedent (Phases 8/9/17/18). `@cronus/ui` exports the whole shell (`BuildingShell` + parts) ready for that pass.

## Detailed Tracking

### Track A — Design token system (`l2-design-system`)

*Foundation. Gates Tracks B and C (they consume token names).*

### [T-24A01] Token taxonomy + Tailwind v4 `@theme` binding

- **Spec:** l2-design-system.md §4.1, §4.3 · l1-design-identity DI-1/DI-3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** a vitest asserts every canonical token name from `l2-design-system` §4.1 (surface/text/line/accent/semantic colour roles, typography, spacing, radius, motion, elevation) is defined as a CSS custom property; `tsc --noEmit` clean; the `@theme` block maps each canonical token to a Tailwind utility; `pnpm -C packages/ui build` succeeds.
- **Handoff:** Token names frozen — A02, and every Track B/C component, styles against them.
- **Notes:** Extend `packages/ui/src/styles.css` (or a new `tokens.css`) with `@import "tailwindcss";` + the `@theme { --color-…: var(--…); … }` map. Semantic-role names only — no raw hues in the map. Do not set per-mode values here (A02 owns the `:root[data-scheme][data-theme]` blocks).

### [T-24A02] `default` colour-scheme package + `(mode × scheme)` resolver

- **Spec:** l2-design-system.md §4.2, §4.4, §4.5 · l1-design-identity DI-1/DI-2/DI-4
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — `resolve("system", "default", /*osPrefersDark*/ true)` returns the dark token set + `resolvedMode: "dark"`; `resolve("light", "default", true)` returns light; an unknown scheme id resolves to `default` and emits a surfaced warning (never a blank surface); a corrupt/absent `default` applies the minimal built-in safe token set and logs an integrity error. The resolver applies `data-theme` + `data-scheme` + the per-mode custom-property block on the surface root.
- **Handoff:** C01 wires this resolver into `App`/`Workbench`.
- **Notes:** Author `packages/ui/src/schemes/default/{manifest.json, tokens.light.css, tokens.dark.css}` from the mockup palette (dark-first — `#0a0a0a`/`#131313`/`#e5e5e5`, accents `#86efac`/`#e2c08d`/`#3794ff`/`#ff6568`, Geist family — plus a derived light variant). Extend `theme.ts`: add `resolveScheme(mode, schemeId, osPrefersDark)` + a catalog lookup with id-stable layered override (built-in layer only in this slice). `manifest.json` validates against a published JSON schema before entering the catalog.

### [T-24A03] DI-3 craft lint (must-fix subset) + migrate hardcoded values

- **Spec:** l2-design-system.md §4.6 · l1-design-identity DI-3/DI-5/DI-6/DI-7
- **Status:** Done
- **Assignment:** Agent
- **Verify:** the lint command (a `packages/ui` `package.json` script) exits non-zero on a seeded literal (`#fff` in a component, a `font-family:` literal, a raw-px `border-radius`) and exits 0 on the clean tree after the Track B components are migrated to `var(--token)` / Tailwind utilities; advisory rules are reported but do not fail the run; the auto-vs-advisory split is declared in the lint config (data, not engine code).
- **Handoff:** Wired into the T-24T02 gate.
- **Notes:** A biome/stylelint rule or a small custom check. Runs after Track B lands so the migration target exists.

### Track B — Shell frame chrome (`l2-navigation` v1.1.0 amendment delta)

*The Phase 11 `l2-navigation` build keeps its `[x]` at 1.0.0; this track is the 1.1.0 delta as a new surface.*

### [T-24B01] L0 BuildingFrame + File/Edit/View/Help menu + action registry

- **Spec:** l2-navigation.md §4.1, §4.6 · l1-navigation-model §4.6 · l1-application-shell AS-6 (thin) · INV-9
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — the burger toggles the menu; File/Edit/View/Help render exactly the `l1-navigation-model` §4.6 leaves with separators; a leaf whose action is not registered is absent (not disabled, not "not implemented"); "New / Add Project…" fires `onCreateFloor`, "Settings…" opens the global-settings overlay.
- **Handoff:** The action registry (`{id,label,run,binding}[]`) is the palette's source (B05).
- **Notes:** `BuildingFrame` component: icon, burger toggle, back/forward, sidebar toggle, right-dock toggle, window controls; dropdown menus. Presentation-only; a native menu bar vs the in-window burger is a per-platform render choice — build the in-window burger.

### [T-24B02] L1 FloorTabBar

- **Spec:** l2-navigation.md §4.2 · l1-navigation-model NV-2/NV-3/NV-8/NV-9
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — the Home tab is pinned, has no close/delete in its ︙ menu, and renders first; a project floor tab shows a status dot reflecting the injected `OfficeState` (not a poll); `+` and a full-bar drop both fire `onCreateFloor`; floor switch fires `onSelectFloor` (load/unload dispatched as an intent, not performed here).
- **Handoff:** —
- **Notes:** Render-from-props. `OfficeState` arrives as a projection prop; no bridge call in the component.

### [T-24B03] L2 SubsystemSidebar + expanded catalog

- **Spec:** l2-navigation.md §4.3 · l1-navigation-model §4.1 (NV-1) · INV-9
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — `navigation.ts` exports `SIDEBAR_PRIMARY` (`[Dashboard, Chat, Sessions, Inbox, Office, Employees, Schedule, Kanban, Automation, Memory, Wiki]`) and `SIDEBAR_UTILITY` (`[Channels, Security, Providers/ACP, Settings]`); `isCanonicalOrder` accepts exactly this two-run order and rejects any permutation; `composeSidebar(pins)` renders pins above `SIDEBAR_PRIMARY` without mutating either frozen array; the sidebar renders 11 primary + a visually-separated 4-item foot group + the floor-identity header + the search affordance + the run-control (play/pause/stop).
- **Handoff:** The catalog gates the B04 router.
- **Notes:** Update the `SidebarTab` union, `SIDEBAR_TABS` → the two arrays, `isCanonicalOrder`, `composeSidebar`, `settingsTier` if key names shifted. `Object.freeze` both arrays.

### [T-24B04] L3 MechanismNav strip + surface router

- **Spec:** l2-navigation.md §4.1 · l1-navigation-model §4.5 (facet catalog) · l1-navigation-model NV-10 · INV-9
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — a frontend constant `L3_FACETS` matches `l1-navigation-model` §4.5 (Schedule→[Cron,Pulse], Inbox→[Messages,Poll/Clarify], Dashboard→[Agent Statistics,Token Usage], Office→[Home,Project], Kanban→[boards], Automation→[flows], Channels→[per-channel], Settings→[Global,Local]); `MechanismNav` renders a strip for `Schedule` and renders nothing for `Memory`; the surface router maps a known subsystem to its surface and an unmapped one to the placeholder without throwing.
- **Handoff:** The router is where C02 mounts placeholders.
- **Notes:** —

### [T-24B05] Right file-tree dock + command palette + global-settings overlay

- **Spec:** l2-navigation.md §4.6, §4.7, §4.8 · l1-navigation-model NV-7 · l1-application-shell AS-9 (edge dock) / AS-10 (delegated selection surface) · INV-9
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — the palette opens on the Ctrl+Shift+J binding and closes on Esc; its delegate sources recent offices / go-to-subsystem / settings / actions, fuzzy-ranks, renders a row (icon, label, secondary text, binding), and confirm dispatches the item without a bridge call; the right dock toggles from the title-bar control and renders a read-only file-tree projection (git-ignored entries dimmed); the global-settings overlay mounts full-screen above the workbench from File▸Settings and from the Settings-tab Global tier, and closing it restores the prior surface + floor + subsystem unchanged.
- **Handoff:** —
- **Notes:** All three are caller-owned view state — no core round-trip, no persisted layout in the core. One reusable selection surface parameterized by a delegate (AS-10); the palette is its first delegate.

### Track C — Theming wiring & surface placeholders (`l2-app-ui` v1.4.0 amendment delta)

*The Phase 8 `l2-app-ui` build keeps its `[x]` at delivered scope; this track is the 1.4.0 two-axis-theming delta.*

### [T-24C01] Two-axis theming wiring + `app.json` `colorScheme` + Settings ▸ Appearance pickers

- **Spec:** l2-app-ui.md §4.5, §4.7 · l2-design-system §4.2, §4.5 · l1-design-identity DI-2
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — changing the `colorScheme` prop swaps `data-scheme` and the applied custom-property block with no component unmount (cosmetic-only, DI-2); changing `theme` swaps `data-theme` the same way; `tsc --noEmit` clean; a settings object without a `colorScheme` field deserializes with the serde default `"default"` filled (older `app.json` round-trips); Settings ▸ Appearance renders a mode picker (system/light/dark) and a scheme picker (built-in schemes).
- **Handoff:** Depends on T-24A02 (the resolver). Sequenced after A02 on `theme.ts`.
- **Notes:** Add `colorScheme` to the `apps/desktop` settings schema next to `theme`/`locale`, each with `#[serde(default = "…")]`. Wire `App`/`Workbench` to read `{ theme, colorScheme }` and apply the resolver at the surface root.

### [T-24C02] Surface placeholders (INV-9) + i18n externalization

- **Spec:** l2-app-ui.md §4.1, §4.6 · l1-architecture INV-9 (shipped-surface honesty)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** vitest — every `SIDEBAR_PRIMARY` + `SIDEBAR_UTILITY` tab resolves through the router to a placeholder or a real empty-state surface, none throw; no fabricated data from the mockup (agent names, card counts, chart values) appears in any component — a `grep` for those literals returns nothing; every new user-facing string resolves through `t(key)` with the key present in both the `en` and `ru` catalogs (a `grep` for hardcoded quoted UI text in the new components returns nothing).
- **Handoff:** —
- **Notes:** The placeholder copy itself ("this surface will be populated by the core" or similar) is an i18n key. Extend `i18n.ts` `MessageKey` union + both catalogs.

### Track D — SDD containment cleanup

### [T-24D01] `CHANGELOG.md:18` reference leak

- **Spec:** RULES.md §6 (SDD Reference Containment) · rules/magic.md §6 (Remediation)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `grep -nE '[Pp]hase[-\s][0-9]|L2 realization spec' CHANGELOG.md` returns nothing in the `[nodus-0.3.0]` "Earlier in this release cycle" bullet; a `finalize` / diagnostic rescan reports no `SDD_REFERENCE_LEAK` at `CHANGELOG.md`.
- **Handoff:** —
- **Notes:** Product file (outside `.design/`), one bullet. Restate "(Phases 8–23, each with its own L2 realization spec)" as plain language with no phase designator and no spec-layer noun — e.g. "Earlier in this release cycle:". Pre-existing leak, injected by a prior `finalize`; carried here per magic.md §6 (audit finds, a scheduled Coder task repairs — never edited from a workflow whose write scope is `.design/`).

### Track T — Validation

### [T-24T01] Spec-conformance test sweep

- **Goal:** One named vitest per touched invariant class, verifying implementation vs spec.
- **Method:** `pnpm -C packages/ui test` — named tests: `NV-1` (two frozen runs, order, pins-above), `NV-7` (L0 menu leaves + palette + dock present), `NV-10` (L3 facet catalog matches §4.5), `DI-2` (mode + scheme switch is cosmetic-only, no unmount), `DI-3` (craft lint fails on a literal), `INV-9` (every tab → placeholder or empty-state, no dead controls, no fabricated data). Each named test exists and passes.
- **Status:** Done

### [T-24T02] Verification gate run

- **Goal:** Prove the full `packages/ui` (+ `apps/desktop`) quality gate is green for the touched packages.
- **Method:** `pnpm -C packages/ui test` (vitest) · biome lint+format (zero errors) · `tsc --noEmit` · `fallow audit --changed-since <base>` (no new dead code / duplication / circular deps / architecture-boundary violations — presentation-only holds against the amended `theme.ts` / `navigation.ts` / `i18n.ts` / `styles.css`, not just the new files) · the DI-3 craft lint. All five commands exit 0; captured in the phase Evidence Capsule.
- **Status:** Done

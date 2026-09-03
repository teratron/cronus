/**
 * Navigation model — the four-layer "building" navigation as pure presentation
 * logic. Presentation only: the canonical catalog and layer structure are frontend
 * constants; live floor/state data arrives from the core over the bridge. No
 * business logic lives here — these are the render/selection rules the shell obeys.
 *
 * Maps the navigation-model invariants: canonical sidebar order and its two runs
 * (a primary function run + a foot utility group), lazy floor loading, two-tier
 * settings, strict layer nesting, pinned home floor, and the per-subsystem
 * mechanism (L3) facet catalog.
 */

/** The four nested navigation layers, outermost to innermost. */
export type NavLayer = "building" | "floor" | "subsystem" | "mechanism";

export const NAV_LAYERS: NavLayer[] = [
  "building",
  "floor",
  "subsystem",
  "mechanism",
];

/** A canonical subsystem sidebar tab. */
export type SidebarTab =
  // primary run
  | "dashboard"
  | "chat"
  | "sessions"
  | "inbox"
  | "office"
  | "employees"
  | "schedule"
  | "kanban"
  | "automation"
  | "memory"
  | "wiki"
  // foot utility group
  | "channels"
  | "security"
  | "providers"
  | "settings";

/**
 * The primary function run, in fixed canonical order. Frozen: no tab may be
 * hidden, reordered, or removed at the application level.
 */
export const SIDEBAR_PRIMARY: readonly SidebarTab[] = Object.freeze([
  "dashboard",
  "chat",
  "sessions",
  "inbox",
  "office",
  "employees",
  "schedule",
  "kanban",
  "automation",
  "memory",
  "wiki",
] as const);

/**
 * The foot utility group — visually separated, pinned to the sidebar foot,
 * fixed order. Frozen for the same reason as the primary run.
 */
export const SIDEBAR_UTILITY: readonly SidebarTab[] = Object.freeze([
  "channels",
  "security",
  "providers",
  "settings",
] as const);

/** The canonical catalog, both runs concatenated in render order. */
export const SIDEBAR_TABS: readonly SidebarTab[] = Object.freeze([
  ...SIDEBAR_PRIMARY,
  ...SIDEBAR_UTILITY,
]);

/** Whether a candidate ordering matches the canonical two-run order exactly. */
export function isCanonicalOrder(tabs: readonly SidebarTab[]): boolean {
  return tabs.length === SIDEBAR_TABS.length && tabs.every((tab, i) => tab === SIDEBAR_TABS[i]);
}

/**
 * Compose the rendered sidebar: user-pinned shortcut tabs render above the
 * primary run; the two canonical runs stay intact and in order below. Pins
 * never mutate either frozen array.
 */
export function composeSidebar(pinnedShortcuts: readonly SidebarTab[]): {
  pinned: readonly SidebarTab[];
  primary: readonly SidebarTab[];
  utility: readonly SidebarTab[];
} {
  return {
    pinned: pinnedShortcuts,
    primary: SIDEBAR_PRIMARY,
    utility: SIDEBAR_UTILITY,
  };
}

/** Whether `inner` is a valid direct child layer of `outer`. */
export function isChildLayer(outer: NavLayer, inner: NavLayer): boolean {
  const oi = NAV_LAYERS.indexOf(outer);
  const ii = NAV_LAYERS.indexOf(inner);
  return oi >= 0 && ii === oi + 1;
}

/**
 * The per-subsystem mechanism (L3) facet catalog. A subsystem listed here grows
 * a sub-navigation strip of these facets; one absent from the map is flat and
 * renders no L3 layer. Depth is earned per-subsystem, never imposed uniformly.
 */
export const L3_FACETS: Partial<Record<SidebarTab, readonly string[]>> = Object.freeze({
  dashboard: [
    "agent-statistics",
    "token-usage",
  ],
  inbox: [
    "messages",
    "poll-clarify",
  ],
  schedule: [
    "cron",
    "pulse",
  ],
  office: [
    "home",
    "project",
  ],
  kanban: [
    "boards",
  ],
  automation: [
    "flows",
  ],
  channels: [
    "detail",
  ],
  settings: [
    "global",
    "local",
  ],
});

/** Whether a subsystem carries an L3 mechanism sub-navigation. */
export function hasMechanismNav(tab: SidebarTab): boolean {
  return (L3_FACETS[tab]?.length ?? 0) > 0;
}

export type FloorKind = "home" | "project";

export interface Floor {
  id: string;
  name: string;
  kind: FloorKind;
  /** Whether the floor is currently loaded in memory. */
  loaded: boolean;
  /** Whether the floor has a running task that requires monitoring. */
  hasRunningTask: boolean;
}

/** The home floor is pinned, non-closable, and always loaded. */
export function isClosable(floor: Floor): boolean {
  return floor.kind !== "home";
}

/**
 * Whether a floor should be loaded into memory: the home floor always, the
 * active floor, or any floor holding a running task. An inactive project floor
 * with no running task must not consume foreground resources.
 */
export function shouldLoad(floor: Floor, activeFloorId: string): boolean {
  if (floor.kind === "home") return true;
  if (floor.id === activeFloorId) return true;
  return floor.hasRunningTask;
}

/**
 * Whether a floor is eligible for unloading: a closed, inactive project floor
 * with no running task. The home floor is never a candidate.
 */
export function isUnloadable(floor: Floor, activeFloorId: string): boolean {
  return floor.kind === "project" && floor.id !== activeFloorId && !floor.hasRunningTask;
}

/** The two settings tiers. */
export type SettingsTier = "global" | "local";

/** Resolve which tier owns a setting key. Global affects the whole app; local
 * travels with the active office. */
export function settingsTier(key: string): SettingsTier {
  const globalKeys = new Set([
    "appearance",
    "models",
    "security",
    "notifications",
    "updates",
    "configuredIde",
  ]);
  return globalKeys.has(key) ? "global" : "local";
}

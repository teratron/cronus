/**
 * Navigation model — the four-layer "building" navigation as pure presentation
 * logic. Presentation only: the canonical catalog and layer structure are frontend
 * constants; live floor/state data arrives from the core over the bridge. No
 * business logic lives here — these are the render/selection rules the shell obeys.
 *
 * Maps the navigation model invariants NV-1 (canonical sidebar order), NV-2 (floor
 * lazy loading), NV-4 (two-tier settings), NV-6 (strict layer nesting), NV-9
 * (pinned home floor).
 */

/** The four nested navigation layers, outermost to innermost (NV-6). */
export type NavLayer = "building" | "floor" | "subsystem" | "mechanism";

export const NAV_LAYERS: NavLayer[] = [
  "building",
  "floor",
  "subsystem",
  "mechanism",
];

/** A canonical subsystem sidebar tab. */
export type SidebarTab =
  | "chat"
  | "inbox"
  | "channels"
  | "sessions"
  | "schedule"
  | "pulse"
  | "memory"
  | "office"
  | "kanban"
  | "security"
  | "wiki"
  | "settings";

/**
 * The canonical sidebar catalog in fixed order (NV-1). Frozen: no tab may be
 * hidden, reordered, or removed at the application level.
 */
export const SIDEBAR_TABS: readonly SidebarTab[] = Object.freeze([
  "chat",
  "inbox",
  "channels",
  "sessions",
  "schedule",
  "pulse",
  "memory",
  "office",
  "kanban",
  "security",
  "wiki",
  "settings",
]);

/** Whether a candidate ordering matches the canonical order exactly (NV-1). */
export function isCanonicalOrder(tabs: readonly SidebarTab[]): boolean {
  return (
    tabs.length === SIDEBAR_TABS.length &&
    tabs.every((tab, i) => tab === SIDEBAR_TABS[i])
  );
}

/**
 * Compose the rendered sidebar: user-pinned shortcut tabs render above the
 * canonical set, which stays intact below (NV-1). Pins never mutate the catalog.
 */
export function composeSidebar(pinnedShortcuts: readonly SidebarTab[]): {
  pinned: readonly SidebarTab[];
  canonical: readonly SidebarTab[];
} {
  return { pinned: pinnedShortcuts, canonical: SIDEBAR_TABS };
}

/** Whether `inner` is a valid direct child layer of `outer` (NV-6). */
export function isChildLayer(outer: NavLayer, inner: NavLayer): boolean {
  const oi = NAV_LAYERS.indexOf(outer);
  const ii = NAV_LAYERS.indexOf(inner);
  return oi >= 0 && ii === oi + 1;
}

export type FloorKind = "home" | "project";

export interface Floor {
  id: string;
  name: string;
  kind: FloorKind;
  /** Whether the floor is currently loaded in memory (NV-2). */
  loaded: boolean;
  /** Whether the floor has a running task that requires monitoring (NV-2). */
  hasRunningTask: boolean;
}

/** The home floor is pinned, non-closable, and always loaded (NV-9). */
export function isClosable(floor: Floor): boolean {
  return floor.kind !== "home";
}

/**
 * Whether a floor should be loaded into memory (NV-2): the home floor always,
 * the active floor, or any floor holding a running task. An inactive project
 * floor with no running task must not consume foreground resources.
 */
export function shouldLoad(floor: Floor, activeFloorId: string): boolean {
  if (floor.kind === "home") return true; // NV-9: home is always loaded
  if (floor.id === activeFloorId) return true;
  return floor.hasRunningTask;
}

/**
 * Whether a floor is eligible for unloading (NV-2): a closed, inactive project
 * floor with no running task. The home floor is never a candidate (NV-9).
 */
export function isUnloadable(floor: Floor, activeFloorId: string): boolean {
  return (
    floor.kind === "project" &&
    floor.id !== activeFloorId &&
    !floor.hasRunningTask
  );
}

/** The two settings tiers (NV-4). */
export type SettingsTier = "global" | "local";

export const SETTINGS_TIERS: readonly SettingsTier[] = Object.freeze([
  "global",
  "local",
]);

/** Resolve which tier owns a setting key (NV-4). Global affects the whole app;
 * local travels with the active office. */
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

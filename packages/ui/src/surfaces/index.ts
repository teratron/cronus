/**
 * Surfaces tier — the declaration.
 *
 * Each surface is a renderable destination the shell routes to. A surface knows
 * only the shared tier; it never imports a sibling surface. This barrel
 * publishes each surface's mount component, its props, and the core projection
 * types it consumes — never a sub-component or a private helper.
 */

export type {
  BuildingStats,
  DashboardProjection,
  DashboardProps,
  OfficeStats,
} from "./dashboard";
export { DashboardPanel } from "./dashboard";
export type {
  OfficeAgent,
  OfficeProjection,
  OfficeRenderMode,
  OfficeTask,
  OfficeViewProps,
} from "./office-view";
export { OfficeViewPanel } from "./office-view";

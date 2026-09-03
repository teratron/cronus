/** The application shell frame — L0…L3 chrome, overlays, and composers; presentation only. */

export {
  type ActionRegistry,
  createActionRegistry,
  isBound,
  type ShellAction,
} from "./actions";
export { BuildingFrame, type BuildingFrameProps } from "./building-frame";
export { BuildingShell, type BuildingShellProps } from "./building-shell";
export {
  CommandPalette,
  type CommandPaletteProps,
  commandPaletteDelegate,
  type SelectionDelegate,
  type SelectionItem,
  SelectionSurface,
} from "./command-palette";
export {
  type FloorTab,
  FloorTabBar,
  type FloorTabBarProps,
  type OfficeState,
} from "./floor-tab-bar";
export {
  GlobalSettingsOverlay,
  type GlobalSettingsOverlayProps,
} from "./global-settings-overlay";
export { MechanismNav, type MechanismNavProps } from "./mechanism-nav";
export { MENU, type MenuGroup, type MenuGroupId, visibleMenu } from "./menu";
export { type FileNode, RightDock, type RightDockProps } from "./right-dock";
export {
  type RunControlState,
  SubsystemSidebar,
  type SubsystemSidebarProps,
} from "./subsystem-sidebar";
export {
  SurfacePlaceholder,
  SurfaceRouter,
  type SurfaceRouterProps,
} from "./surface-router";

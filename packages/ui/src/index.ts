export type { AppProps } from "./App";
export { App } from "./App";
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
export type { CoreClient, InvokeFn } from "./shared/bridge";
export { createCoreClient } from "./shared/bridge";
export type { Locale, MessageKey } from "./shared/i18n";
export { DEFAULT_LOCALE, t, translator } from "./shared/i18n";
export type {
  Floor,
  FloorKind,
  NavLayer,
  SettingsTier,
  SidebarTab,
} from "./shared/navigation";
export {
  composeSidebar,
  hasMechanismNav,
  isCanonicalOrder,
  isChildLayer,
  isClosable,
  isUnloadable,
  L3_FACETS,
  NAV_LAYERS,
  SIDEBAR_PRIMARY,
  SIDEBAR_TABS,
  SIDEBAR_UTILITY,
  settingsTier,
  shouldLoad,
} from "./shared/navigation";
export type {
  ResolvedSurface,
  ResolvedTheme,
  SchemeManifest,
  Theme,
} from "./shared/theme";
export {
  DEFAULT_SCHEME_ID,
  registerScheme,
  resolveScheme,
  resolveTheme,
  schemeCatalog,
  surfaceAttributes,
  themeAttributes,
} from "./shared/theme";
export { CANONICAL_TOKENS, type TokenName } from "./shared/tokens";
export * from "./shell";
export type { SurfaceId, WorkbenchProps } from "./surfaces";
export { SURFACES, Workbench } from "./surfaces";

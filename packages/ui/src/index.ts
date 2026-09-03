export type { AppProps } from "./App";
export { App } from "./App";
export type { CoreClient, InvokeFn } from "./bridge";
export { createCoreClient } from "./bridge";
export type {
  BuildingStats,
  DashboardProjection,
  DashboardProps,
  OfficeStats,
} from "./dashboard";
export { DashboardPanel } from "./dashboard";
export type { Locale, MessageKey } from "./i18n";
export { DEFAULT_LOCALE, t, translator } from "./i18n";
export type {
  Floor,
  FloorKind,
  NavLayer,
  SettingsTier,
  SidebarTab,
} from "./navigation";
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
} from "./navigation";
export type {
  OfficeAgent,
  OfficeProjection,
  OfficeRenderMode,
  OfficeTask,
  OfficeViewProps,
} from "./office-view";
export { OfficeViewPanel } from "./office-view";
export * from "./shell";
export type { SurfaceId, WorkbenchProps } from "./surfaces";
export { SURFACES, Workbench } from "./surfaces";
export type {
  ResolvedSurface,
  ResolvedTheme,
  SchemeManifest,
  Theme,
} from "./theme";
export {
  DEFAULT_SCHEME_ID,
  registerScheme,
  resolveScheme,
  resolveTheme,
  schemeCatalog,
  surfaceAttributes,
  themeAttributes,
} from "./theme";
export { CANONICAL_TOKENS, type TokenName } from "./tokens";

export type { ChannelEvent, CoreClient, InvokeFn, ListenFn } from "./shared/bridge";
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
export type {
  BuildingStats,
  DashboardProjection,
  DashboardProps,
  OfficeAgent,
  OfficeProjection,
  OfficeRenderMode,
  OfficeStats,
  OfficeTask,
  OfficeViewProps,
} from "./surfaces";
export { DashboardPanel, OfficeViewPanel } from "./surfaces";

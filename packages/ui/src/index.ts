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
  OfficeAgent,
  OfficeProjection,
  OfficeRenderMode,
  OfficeTask,
  OfficeViewProps,
} from "./office-view";
export { OfficeViewPanel } from "./office-view";
export type { SurfaceId, WorkbenchProps } from "./surfaces";
export { SURFACES, Workbench } from "./surfaces";
export type { ResolvedTheme, Theme } from "./theme";
export { resolveTheme, themeAttributes } from "./theme";

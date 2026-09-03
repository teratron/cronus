/**
 * Workbench — the original surface composer.
 *
 * Presentation only: renders whatever state it is given and forwards surface
 * selection as an intent callback; it never mutates domain state. It is a
 * composer (a nav strip plus a routed surface panel), so it lives in the shell
 * tier — it is allowed to know surfaces, and a surface must never know it.
 *
 * `BuildingShell` is its successor. This composer stays until the desktop entry
 * point adopts that shell (a UI-to-core integration step); it is kept and
 * relocated here rather than retired.
 */

import { type Locale, translator } from "../shared/i18n";
import { SURFACE_LABEL, SURFACES, type SurfaceId } from "../shared/surface-catalog";
import { resolveTheme, type Theme, themeAttributes } from "../shared/theme";
import {
  DashboardPanel,
  type DashboardProjection,
  type OfficeProjection,
  type OfficeRenderMode,
  OfficeViewPanel,
} from "../surfaces";

export interface WorkbenchProps {
  /** Active surface — owned by the caller (render-from-state). */
  active: SurfaceId;
  /** Surface-selection intent; the workbench never switches itself. */
  onSelect?: (surface: SurfaceId) => void;
  /** Core status line, shown in the footer. */
  status?: string;
  locale?: Locale;
  theme?: Theme;
  /** OS dark-mode preference, injected by the shell (no direct OS reads). */
  systemPrefersDark?: boolean;
  /** Office projection from the core; the office surface renders it. */
  office?: OfficeProjection;
  /** Office render mode — caller-owned view state. */
  officeMode?: OfficeRenderMode;
  /** Inspect intent for an office node. */
  onInspect?: (nodeId: string) => void;
  /** Dashboard projection from the core; the dashboard surface renders it. */
  dashboard?: DashboardProjection;
}

export function Workbench({
  active,
  onSelect,
  status,
  locale = "en",
  theme = "system",
  systemPrefersDark = true,
  office,
  officeMode = "graph",
  onInspect,
  dashboard,
}: WorkbenchProps) {
  const msg = translator(locale);
  const resolved = resolveTheme(theme, systemPrefersDark);
  const attrs = themeAttributes(resolved);

  return (
    <div
      data-theme={attrs["data-theme"]}
      data-testid="workbench"
      className={`flex h-screen flex-col bg-neutral-950 text-neutral-100 ${attrs.className}`}
    >
      <nav className="flex gap-2 border-b border-neutral-800 p-2" aria-label={msg("app.title")}>
        {SURFACES.map((surface) => (
          <button
            key={surface}
            type="button"
            data-testid={`nav-${surface}`}
            aria-current={surface === active ? "page" : undefined}
            className="rounded px-3 py-1 text-sm aria-[current=page]:bg-neutral-800"
            onClick={() => onSelect?.(surface)}
          >
            {msg(SURFACE_LABEL[surface])}
          </button>
        ))}
      </nav>
      <main className="flex-1 p-4" data-testid={`surface-${active}`}>
        <h2 className="text-lg font-semibold">{msg(SURFACE_LABEL[active])}</h2>
        {active === "office" && office ? (
          <OfficeViewPanel
            projection={office}
            mode={officeMode}
            onInspect={onInspect}
            locale={locale}
          />
        ) : active === "dashboard" && dashboard ? (
          <DashboardPanel projection={dashboard} locale={locale} />
        ) : (
          <p className="text-sm text-neutral-400">{msg("surface.empty")}</p>
        )}
      </main>
      <footer className="border-t border-neutral-800 p-2 text-xs text-neutral-400">
        <span data-testid="status">{status ?? msg("status.connecting")}</span>
      </footer>
    </div>
  );
}

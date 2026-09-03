/**
 * Surface router — maps the active subsystem tab to a surface.
 *
 * In this slice every subsystem resolves to an explicit placeholder (INV-9:
 * a control renders real content only when bound to a shipped capability; until
 * then it is a named placeholder, never fabricated data). Two Phase 8 panels
 * (Office, Dashboard) are wired to render their real empty state from an
 * injected projection when the caller supplies one; everything else is a
 * placeholder. As a capability ships, its case moves from placeholder to panel.
 */

import type { JSX } from "react";
import { DashboardPanel, type DashboardProjection } from "../dashboard";
import { type OfficeProjection, OfficeViewPanel } from "../office-view";
import { type Locale, translator } from "../shared/i18n";
import type { SidebarTab } from "../shared/navigation";

export interface SurfaceRouterProps {
  active: SidebarTab;
  /** Optional real projections for the two Phase 8 panels. */
  office?: OfficeProjection;
  dashboard?: DashboardProjection;
  locale?: Locale;
}

/** The explicit "not yet bound" surface (INV-9). Never shows fabricated data. */
export function SurfacePlaceholder({ tab, locale = "en" }: { tab: SidebarTab; locale?: Locale }) {
  const msg = translator(locale);
  return (
    <div
      data-testid={`surface-${tab}`}
      data-placeholder="true"
      className="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center"
    >
      <p className="text-sm text-text-secondary">{msg("surface.placeholder")}</p>
      <p className="text-xs text-text-muted">{msg("surface.placeholder.hint")}</p>
    </div>
  );
}

export function SurfaceRouter({
  active,
  office,
  dashboard,
  locale = "en",
}: SurfaceRouterProps): JSX.Element {
  if (active === "office" && office) {
    return (
      <div data-testid="surface-office" className="flex-1 overflow-y-auto p-4">
        <OfficeViewPanel projection={office} mode="graph" locale={locale} />
      </div>
    );
  }
  if (active === "dashboard" && dashboard) {
    return (
      <div data-testid="surface-dashboard" className="flex-1 overflow-y-auto p-4">
        <DashboardPanel projection={dashboard} locale={locale} />
      </div>
    );
  }
  return <SurfacePlaceholder tab={active} locale={locale} />;
}

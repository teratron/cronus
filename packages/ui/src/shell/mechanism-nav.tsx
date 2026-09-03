/**
 * L3 · Mechanism sub-navigation — a sub-tab strip for subsystems with facets.
 *
 * Presentation only: renders the facet list for the active subsystem from the
 * frozen `L3_FACETS` catalog, or nothing for a flat subsystem. Selection is
 * forwarded as an intent; the strip is scoped to exactly one subsystem and never
 * addresses a sibling's facets.
 */

import type { Locale } from "../i18n";
import { L3_FACETS, type SidebarTab } from "../navigation";

export interface MechanismNavProps {
  subsystem: SidebarTab;
  activeFacet?: string;
  onSelectFacet?: (facet: string) => void;
  locale?: Locale;
}

/** Facet id → display label. Kept minimal — a facet-label i18n pass follows the
 *  per-surface builds; for now the id is title-cased. */
function facetLabel(id: string): string {
  return id
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export function MechanismNav({ subsystem, activeFacet, onSelectFacet }: MechanismNavProps) {
  const facets = L3_FACETS[subsystem];
  if (!facets || facets.length === 0) return null;

  return (
    <div
      data-testid="mechanism-nav"
      data-subsystem={subsystem}
      className="flex items-center gap-0.5 rounded-md border border-border-subtle bg-surface-2 p-0.5"
    >
      {facets.map((facet) => (
        <button
          key={facet}
          type="button"
          data-testid={`facet-${facet}`}
          aria-current={facet === activeFacet ? "page" : undefined}
          className="rounded-sm px-3 py-1 text-xs text-text-secondary hover:text-text-primary aria-[current=page]:bg-surface-3 aria-[current=page]:text-text-primary"
          onClick={() => onSelectFacet?.(facet)}
        >
          {facetLabel(facet)}
        </button>
      ))}
    </div>
  );
}

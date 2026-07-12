/**
 * Dashboard panel: live read-only statistics projection.
 *
 * Renders per-office statistics plus the building-level aggregate from one
 * injected projection. Aggregation happens in the core — the panel displays
 * whatever it is given and never derives authoritative numbers itself.
 */

import { type Locale, translator } from "./i18n";

/** Statistics for one office, as projected by the core. */
export interface OfficeStats {
  id: string;
  name: string;
  activeAgents: number;
  /** Card counts keyed by pipeline state (e.g. running, blocked, done). */
  cardsByState: Record<string, number>;
}

/** Building-level aggregate, computed by the core (read-only here). */
export interface BuildingStats {
  offices: number;
  activeAgents: number;
  totalCards: number;
}

/** The dashboard projection both sections render from. */
export interface DashboardProjection {
  offices: OfficeStats[];
  building?: BuildingStats;
}

export interface DashboardProps {
  projection: DashboardProjection;
  locale?: Locale;
}

export function DashboardPanel({ projection, locale = "en" }: DashboardProps) {
  const msg = translator(locale);
  return (
    <div data-testid="dashboard">
      {projection.building ? (
        <section data-testid="dashboard-building">
          <h3>{msg("dashboard.building")}</h3>
          <dl>
            <dt>{msg("dashboard.offices")}</dt>
            <dd data-testid="building-offices">{projection.building.offices}</dd>
            <dt>{msg("dashboard.active-agents")}</dt>
            <dd data-testid="building-active">{projection.building.activeAgents}</dd>
            <dt>{msg("dashboard.cards")}</dt>
            <dd data-testid="building-cards">{projection.building.totalCards}</dd>
          </dl>
        </section>
      ) : null}
      {projection.offices.map((office) => (
        <section key={office.id} data-testid={`dashboard-office-${office.id}`}>
          <h3>{office.name}</h3>
          <p data-testid={`office-active-${office.id}`}>
            {msg("dashboard.active-agents")}: {office.activeAgents}
          </p>
          <ul>
            {Object.entries(office.cardsByState).map(([state, count]) => (
              <li key={state} data-testid={`cards-${office.id}-${state}`}>
                {state}: {count}
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

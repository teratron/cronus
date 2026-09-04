/**
 * Dashboard panel: live read-only statistics projection.
 *
 * A stat-tile row over a grid of two-series trend charts, plus the per-office
 * and building-level rollups. Aggregation happens in the core — the panel
 * displays whatever it is given and never derives an authoritative number.
 *
 * The one thing it does compute is the bar *scale*: heights are a fraction of
 * the largest value in that chart's own series, so a chart is never drawn
 * against a magic constant that silently lies once the data grows.
 */

import { type Locale, translator } from "../shared/i18n";

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

/** One headline figure. `value` is pre-formatted by the core, unit and all. */
export interface StatCard {
  id: string;
  value: string;
  label: string;
}

/** Which categorical series colour a trend's half uses. */
export type ChartTone = 1 | 2 | 3 | 4 | 5;

const TONE_BG: Record<ChartTone, string> = {
  1: "bg-chart-1",
  2: "bg-chart-2",
  3: "bg-chart-3",
  4: "bg-chart-4",
  5: "bg-chart-5",
};

/** One bucket of a trend: two comparable values sharing an x label. */
export interface TrendPoint {
  label: string;
  a: number;
  b: number;
}

/** A two-series bar trend. */
export interface TrendSeries {
  id: string;
  title: string;
  a: {
    label: string;
    tone: ChartTone;
  };
  b: {
    label: string;
    tone: ChartTone;
  };
  points: TrendPoint[];
}

/** The dashboard projection every section renders from. */
export interface DashboardProjection {
  /** Human-readable window the figures cover, e.g. "Last 7 days". */
  range?: string;
  /** Headline tiles. */
  cards?: StatCard[];
  /** Trend charts, rendered two per row. */
  trends?: TrendSeries[];
  offices?: OfficeStats[];
  building?: BuildingStats;
}

export interface DashboardProps {
  projection: DashboardProjection;
  /** Shown beside the section title — whose figures these are. */
  scopeLabel?: string;
  locale?: Locale;
}

function TrendChart({ series }: { series: TrendSeries }) {
  // Scale against this chart's own peak; a flat-zero series stays flat rather
  // than dividing by zero.
  const peak = Math.max(
    1,
    ...series.points.flatMap((p) => [
      p.a,
      p.b,
    ]),
  );
  const pct = (v: number) => `${Math.max(0, Math.min(100, (v / peak) * 100))}%`;

  return (
    <div
      data-testid={`trend-${series.id}`}
      className="rounded-lg border border-border-subtle bg-surface-3 px-4.5 py-4"
    >
      <div className="font-semibold text-md text-text-primary">{series.title}</div>
      <div className="mt-2.5 mb-3.5 flex gap-4">
        {[
          series.a,
          series.b,
        ].map((s) => (
          <span key={s.label} className="flex items-center gap-1.5 text-text-secondary text-xs">
            <span className={`h-2.25 w-2.25 rounded-xs ${TONE_BG[s.tone]}`} />
            {s.label}
          </span>
        ))}
      </div>
      <div className="flex h-35 items-end gap-1 border-border-subtle border-b px-0.5">
        {series.points.map((p) => (
          <div key={p.label} className="flex h-full flex-1 items-end justify-center gap-0.75">
            <div
              className={`w-2.25 rounded-t-xs ${TONE_BG[series.a.tone]}`}
              style={{
                height: pct(p.a),
              }}
            />
            <div
              className={`w-2.25 rounded-t-xs ${TONE_BG[series.b.tone]}`}
              style={{
                height: pct(p.b),
              }}
            />
          </div>
        ))}
      </div>
      <div className="flex gap-1 px-0.5 pt-1.5">
        {series.points.map((p) => (
          <span key={p.label} className="flex-1 text-center font-mono text-2xs text-text-muted">
            {p.label}
          </span>
        ))}
      </div>
    </div>
  );
}

export function DashboardPanel({ projection, scopeLabel, locale = "en" }: DashboardProps) {
  const msg = translator(locale);
  const { range, cards = [], trends = [], offices = [], building } = projection;

  return (
    <div data-testid="dashboard" className="mx-auto max-w-290">
      {cards.length > 0 || trends.length > 0 ? (
        <div className="mb-4 flex items-center gap-3">
          <span className="font-semibold text-text-primary text-xl">
            {msg("dashboard.statistics")}
          </span>
          {scopeLabel ? <span className="text-sm text-text-muted">{scopeLabel}</span> : null}
          {range ? (
            <span className="ml-auto rounded-md border border-border-strong bg-surface-3 px-2.75 py-1.5 font-mono text-sm text-text-strong">
              {range}
            </span>
          ) : null}
        </div>
      ) : null}

      {cards.length > 0 ? (
        <div data-testid="dashboard-cards" className="mb-3 grid grid-cols-6 gap-2.5">
          {cards.map((c) => (
            <div
              key={c.id}
              data-testid={`stat-${c.id}`}
              className="rounded-lg border border-border-subtle bg-surface-3 px-3 py-4.5 text-center"
            >
              <div className="-tracking-[0.01em] font-semibold text-2xl text-text-primary">
                {c.value}
              </div>
              <div className="mt-1.75 text-text-secondary text-xs leading-tight">{c.label}</div>
            </div>
          ))}
        </div>
      ) : null}

      {trends.length > 0 ? (
        <div className="grid grid-cols-2 gap-3">
          {trends.map((t) => (
            <TrendChart key={t.id} series={t} />
          ))}
        </div>
      ) : null}

      {building ? (
        <section
          data-testid="dashboard-building"
          className="mt-3 rounded-lg border border-border-subtle bg-surface-3 px-4.5 py-4"
        >
          <h3 className="font-semibold text-md text-text-primary">{msg("dashboard.building")}</h3>
          <dl className="mt-3 flex gap-8">
            <div>
              <dt className="text-text-secondary text-xs">{msg("dashboard.offices")}</dt>
              <dd
                data-testid="building-offices"
                className="font-semibold text-text-primary text-xl"
              >
                {building.offices}
              </dd>
            </div>
            <div>
              <dt className="text-text-secondary text-xs">{msg("dashboard.active-agents")}</dt>
              <dd data-testid="building-active" className="font-semibold text-text-primary text-xl">
                {building.activeAgents}
              </dd>
            </div>
            <div>
              <dt className="text-text-secondary text-xs">{msg("dashboard.cards")}</dt>
              <dd data-testid="building-cards" className="font-semibold text-text-primary text-xl">
                {building.totalCards}
              </dd>
            </div>
          </dl>
        </section>
      ) : null}

      {offices.map((office) => (
        <section
          key={office.id}
          data-testid={`dashboard-office-${office.id}`}
          className="mt-3 rounded-lg border border-border-subtle bg-surface-3 px-4.5 py-4"
        >
          <h3 className="font-semibold text-md text-text-primary">{office.name}</h3>
          <p
            data-testid={`office-active-${office.id}`}
            className="mt-1 text-sm text-text-secondary"
          >
            {msg("dashboard.active-agents")}: {office.activeAgents}
          </p>
          <ul className="mt-2 flex flex-wrap gap-2">
            {Object.entries(office.cardsByState).map(([state, count]) => (
              <li
                key={state}
                data-testid={`cards-${office.id}-${state}`}
                className="rounded-sm bg-surface-4 px-2 py-0.5 text-text-secondary text-xs"
              >
                {state}: {count}
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

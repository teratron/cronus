/**
 * L2 · Subsystem sidebar — the canonical catalog in two fixed runs.
 *
 * Presentation only: a primary function run, then a visually-separated foot
 * utility group. Pinned shortcut tabs render above the primary run and never
 * mutate the frozen arrays. A floor-identity header, a search affordance, and a
 * run-control sit around the catalog. Selection is forwarded as an intent;
 * badge counts come from injected per-subsystem signals.
 */

import { type Locale, type MessageKey, translator } from "../i18n";
import { composeSidebar, type SidebarTab } from "../navigation";

const TAB_LABEL: Record<SidebarTab, MessageKey> = {
  dashboard: "nav.dashboard",
  chat: "nav.chat",
  sessions: "nav.sessions",
  inbox: "nav.inbox",
  office: "nav.office",
  employees: "nav.employees",
  schedule: "nav.schedule",
  kanban: "nav.kanban",
  automation: "nav.automation",
  memory: "nav.memory",
  wiki: "nav.wiki",
  channels: "nav.channels",
  security: "nav.security",
  providers: "nav.providers",
  settings: "nav.settings",
};

export type RunControlState = "running" | "paused" | "stopped";

export interface SubsystemSidebarProps {
  active: SidebarTab;
  onSelect?: (tab: SidebarTab) => void;
  /** User-pinned shortcut tabs, rendered above the primary run. */
  pinned?: readonly SidebarTab[];
  /** Live pending-item counts keyed by tab (absent → no badge). */
  badges?: Partial<Record<SidebarTab, number>>;
  /** Active floor identity for the header. */
  floorName?: string;
  floorSlug?: string;
  onOpenSearch?: () => void;
  runState?: RunControlState;
  onRun?: (next: RunControlState) => void;
  locale?: Locale;
}

export function SubsystemSidebar({
  active,
  onSelect,
  pinned = [],
  badges = {},
  floorName,
  floorSlug,
  onOpenSearch,
  runState = "stopped",
  onRun,
  locale = "en",
}: SubsystemSidebarProps) {
  const msg = translator(locale);
  const { pinned: pins, primary, utility } = composeSidebar(pinned);

  const tabButton = (tab: SidebarTab) => {
    const badge = badges[tab];
    return (
      <button
        key={tab}
        type="button"
        data-testid={`sidebar-${tab}`}
        aria-current={tab === active ? "page" : undefined}
        className="flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-sm text-text-secondary hover:text-text-primary aria-[current=page]:bg-surface-2 aria-[current=page]:text-text-primary"
        onClick={() => onSelect?.(tab)}
      >
        {msg(TAB_LABEL[tab])}
        {typeof badge === "number" ? (
          <span
            data-testid={`sidebar-badge-${tab}`}
            className="ml-auto rounded-pill bg-info px-1.5 text-xs font-semibold text-text-inverse"
          >
            {badge}
          </span>
        ) : null}
      </button>
    );
  };

  return (
    <div
      data-testid="subsystem-sidebar"
      className="flex w-52.5 flex-none flex-col border-r border-border-subtle bg-surface-1"
    >
      <div className="flex items-center gap-2.5 border-b border-border-subtle p-3">
        <span className="flex h-7 w-7 flex-none items-center justify-center rounded-md bg-surface-3 text-xs font-semibold text-text-secondary">
          {(floorName ?? "•").slice(0, 2).toUpperCase()}
        </span>
        <span className="min-w-0">
          <span className="block truncate text-sm font-semibold text-text-primary">
            {floorName ?? msg("floor.home")}
          </span>
          {floorSlug ? (
            <span className="block truncate text-xs text-text-muted">{floorSlug}</span>
          ) : null}
        </span>
      </div>

      <div className="px-2.5 pt-3 pb-1.5">
        <button
          type="button"
          data-testid="sidebar-search"
          className="flex w-full items-center gap-2 rounded-md border border-border-subtle bg-surface-2 px-2.5 py-1.5 text-left text-sm text-text-muted hover:border-border-strong hover:text-text-secondary"
          onClick={() => onOpenSearch?.()}
        >
          {msg("frame.search")}
          <span className="ml-auto text-xs">Ctrl Shift J</span>
        </button>
      </div>

      <div className="flex flex-1 flex-col gap-px overflow-y-auto p-2">
        {pins.length > 0 ? (
          <div data-testid="sidebar-pins" className="flex flex-col gap-px pb-1.5">
            {pins.map(tabButton)}
          </div>
        ) : null}
        <div data-testid="sidebar-primary" className="flex flex-col gap-px">
          {primary.map(tabButton)}
        </div>
      </div>

      <div
        data-testid="sidebar-utility"
        className="flex flex-col gap-px border-t border-border-subtle p-2"
      >
        {utility.map(tabButton)}
      </div>

      <div className="flex items-center gap-1 border-t border-border-subtle px-2.5 py-2">
        <div className="flex-1" />
        <div
          data-testid="run-control"
          data-state={runState}
          className="flex items-center gap-px rounded-md border border-border-subtle bg-surface-2 p-0.5"
          title={msg("run.play")}
        >
          <button
            type="button"
            data-testid="run-play"
            aria-pressed={runState === "running"}
            title={msg("run.play")}
            className="flex h-5.5 w-6 items-center justify-center rounded-sm border-none bg-transparent text-text-secondary hover:text-text-primary aria-pressed:text-success"
            onClick={() => onRun?.("running")}
          >
            ▶
          </button>
          <button
            type="button"
            data-testid="run-pause"
            aria-pressed={runState === "paused"}
            title={msg("run.pause")}
            className="flex h-5.5 w-6 items-center justify-center rounded-sm border-none bg-transparent text-text-secondary hover:text-text-primary aria-pressed:text-warning"
            onClick={() => onRun?.("paused")}
          >
            ⏸
          </button>
          <button
            type="button"
            data-testid="run-stop"
            aria-pressed={runState === "stopped"}
            title={msg("run.stop")}
            className="flex h-5.5 w-6 items-center justify-center rounded-sm border-none bg-transparent text-text-secondary hover:text-text-primary aria-pressed:text-danger"
            onClick={() => onRun?.("stopped")}
          >
            ⏹
          </button>
        </div>
      </div>
    </div>
  );
}

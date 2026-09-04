/**
 * L2 · Subsystem sidebar — the canonical catalog in two fixed runs.
 *
 * Presentation only: a floor-identity header, a search affordance, the primary
 * function run, then a visually-separated foot utility group, and a footer
 * carrying the settings/help entries beside the project run control. Pinned
 * shortcut tabs render above the primary run and never mutate the frozen
 * arrays. Selection is forwarded as an intent; badge counts come from injected
 * per-subsystem signals — nothing here polls.
 *
 * A badge is one of two kinds, and the difference is meaning, not styling: an
 * **alert** count (unread, failing) is a filled pill in its semantic colour; a
 * **tally** (how many of a thing exist) is plain muted text. Rendering a tally
 * as an alert is how a sidebar starts crying wolf.
 */

import { type Locale, type MessageKey, translator } from "../shared/i18n";
import { composeSidebar, type SidebarTab } from "../shared/navigation";
import { Icon, type IconName } from "./icons";

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

const TAB_ICON: Record<SidebarTab, IconName> = {
  dashboard: "dashboard",
  chat: "chat",
  sessions: "activity",
  inbox: "inbox",
  office: "office",
  employees: "employees",
  schedule: "schedule",
  kanban: "kanban",
  automation: "automation",
  memory: "memory",
  wiki: "wiki",
  channels: "channels",
  security: "security",
  providers: "automation",
  settings: "settings",
};

/** Which tabs report an alert count rather than a plain tally. */
const ALERT_TONE: Partial<Record<SidebarTab, string>> = {
  chat: "bg-warning",
  sessions: "bg-success",
  inbox: "bg-info",
  kanban: "bg-warning",
  security: "bg-danger",
};

export type RunControlState = "running" | "paused" | "stopped";

export interface SubsystemSidebarProps {
  active: SidebarTab;
  onSelect?: (tab: SidebarTab) => void;
  /** User-pinned shortcut tabs, rendered above the primary run. */
  pinned?: readonly SidebarTab[];
  /** Live pending-item counts keyed by tab (absent → no badge). */
  badges?: Partial<Record<SidebarTab, number>>;
  /** Tabs whose count is a tally, not an alert — rendered as muted text. */
  tallies?: Partial<Record<SidebarTab, string | number>>;
  /** Tabs carrying an unread marker with no count (a dot). */
  markers?: readonly SidebarTab[];
  /** Active floor identity for the header. */
  floorName?: string;
  floorSlug?: string;
  /** Initials for the floor's manager avatar. */
  floorInitials?: string;
  onOpenSearch?: () => void;
  onOpenSettings?: () => void;
  onOpenHelp?: () => void;
  runState?: RunControlState;
  onRun?: (next: RunControlState) => void;
  locale?: Locale;
}

const ROW =
  "flex w-full items-center gap-2.5 rounded-sm px-2.25 py-1.75 text-left text-base transition-colors";

export function SubsystemSidebar({
  active,
  onSelect,
  pinned = [],
  badges = {},
  tallies = {},
  markers = [],
  floorName,
  floorSlug,
  floorInitials,
  onOpenSearch,
  onOpenSettings,
  onOpenHelp,
  runState = "stopped",
  onRun,
  locale = "en",
}: SubsystemSidebarProps) {
  const msg = translator(locale);
  const { pinned: pins, primary, utility } = composeSidebar(pinned);
  const initials = floorInitials ?? (floorName ?? "").slice(0, 2).toUpperCase();

  const tabButton = (tab: SidebarTab) => {
    const alert = badges[tab];
    const tally = tallies[tab];
    const isActive = tab === active;
    return (
      <button
        key={tab}
        type="button"
        data-testid={`sidebar-${tab}`}
        aria-current={isActive ? "page" : undefined}
        className={`${ROW} ${
          isActive
            ? "bg-surface-4 text-text-primary"
            : "bg-transparent text-text-secondary hover:bg-surface-3 hover:text-text-primary"
        }`}
        onClick={() => onSelect?.(tab)}
      >
        <Icon name={TAB_ICON[tab]} />
        {msg(TAB_LABEL[tab])}
        {alert !== undefined ? (
          <span
            data-testid={`sidebar-badge-${tab}`}
            className={`ml-auto rounded-pill px-1.5 font-semibold text-2xs text-text-inverse ${
              ALERT_TONE[tab] ?? "bg-info"
            }`}
          >
            {alert}
          </span>
        ) : tally !== undefined ? (
          <span data-testid={`sidebar-tally-${tab}`} className="ml-auto text-text-muted text-xs">
            {tally}
          </span>
        ) : markers.includes(tab) ? (
          <span
            data-testid={`sidebar-marker-${tab}`}
            className="ml-auto h-1.5 w-1.5 rounded-pill bg-info"
          />
        ) : null}
      </button>
    );
  };

  const runButton = (state: RunControlState, icon: IconName, label: MessageKey) => (
    <button
      type="button"
      data-testid={`run-${state === "running" ? "play" : state === "paused" ? "pause" : "stop"}`}
      title={msg(label)}
      aria-pressed={runState === state}
      className={`flex h-5.5 w-6 items-center justify-center rounded-sm transition-colors ${
        runState === state
          ? "bg-surface-active text-text-primary"
          : "bg-transparent text-text-muted hover:text-text-primary"
      }`}
      onClick={() => onRun?.(state)}
    >
      <Icon name={icon} size={12} />
    </button>
  );

  return (
    <div
      data-testid="subsystem-sidebar"
      className="flex w-52.5 flex-none flex-col border-border-subtle border-r bg-surface-2"
    >
      {/* floor identity */}
      <div className="flex items-center gap-2.25 border-border-subtle border-b px-3 py-2.75">
        <span className="flex h-7 w-7 flex-none items-center justify-center rounded-md bg-surface-4 font-semibold text-text-primary text-xs">
          {initials}
        </span>
        <span className="min-w-0">
          <span className="block truncate font-semibold text-base text-text-primary">
            {floorName}
          </span>
          <span className="block truncate text-text-secondary text-2xs">{floorSlug}</span>
        </span>
      </div>

      {/* search */}
      <div className="px-2.5 pt-3 pb-1.5">
        <button
          type="button"
          data-testid="sidebar-search"
          className="flex w-full items-center gap-2 rounded-md border border-border-subtle bg-surface-5 py-1.75 pr-2 pl-2.5 text-left text-text-muted transition-colors hover:border-border-strong hover:text-text-secondary"
          onClick={onOpenSearch}
        >
          <Icon name="search" size={14} className="flex-none" />
          <span className="flex-1 text-base">{msg("sidebar.search")}</span>
          <span className="flex gap-0.75">
            {[
              "Ctrl",
              "Shift",
              "J",
            ].map((k) => (
              <span
                key={k}
                className="rounded-xs border border-border-strong px-1.25 py-px text-2xs text-text-muted"
              >
                {k}
              </span>
            ))}
          </span>
        </button>
      </div>

      {/* primary run */}
      <div className="flex flex-1 flex-col gap-px overflow-y-auto px-2 pt-2 pb-1">
        {pins.length > 0 ? (
          <div data-testid="sidebar-pins" className="flex flex-col gap-px pb-1">
            {pins.map(tabButton)}
          </div>
        ) : null}
        {primary.map(tabButton)}
      </div>

      {/* foot utility group */}
      <div
        data-testid="sidebar-utility"
        className="flex flex-col gap-px border-border-subtle border-t px-2 pt-1.5 pb-2"
      >
        {utility.filter((t) => t !== "settings").map(tabButton)}
      </div>

      {/* footer: global entries + run control */}
      <div className="flex items-center gap-1 border-border-subtle border-t px-2.5 py-2">
        <button
          type="button"
          data-testid="sidebar-settings"
          title={msg("nav.settings")}
          className="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-transparent text-text-secondary transition-colors hover:bg-surface-active hover:text-text-primary"
          onClick={onOpenSettings}
        >
          <Icon name="settings" />
        </button>
        <button
          type="button"
          data-testid="sidebar-help"
          title={msg("sidebar.help")}
          className="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-transparent text-text-secondary transition-colors hover:bg-surface-active hover:text-text-primary"
          onClick={onOpenHelp}
        >
          <Icon name="help" />
        </button>
        <div className="flex-1" />
        <div
          data-testid="run-control"
          data-state={runState}
          title={msg("sidebar.run-control")}
          className="flex items-center gap-px rounded-md border border-border-subtle bg-surface-4 p-0.5"
        >
          {runButton("running", "play", "sidebar.run.play")}
          {runButton("paused", "pause", "sidebar.run.pause")}
          {runButton("stopped", "stop", "sidebar.run.stop")}
        </div>
      </div>
    </div>
  );
}

/**
 * L1 · Floor tab bar — one tab per workspace (office).
 *
 * Presentation only: renders from an injected floor list and forwards selection,
 * creation, and per-floor menu intents. The pinned home floor renders first and
 * offers no close/delete. Each tab's status dot reflects the injected
 * `OfficeState` — the caller subscribes to the core's state stream and re-renders;
 * this component never polls.
 */

import { type Locale, translator } from "../shared/i18n";
import type { FloorKind } from "../shared/navigation";

/** Live office lifecycle state, mirrored from the core taxonomy. */
export type OfficeState = "active" | "idle" | "paused" | "hibernating" | "error" | "offline";

const STATE_DOT: Record<OfficeState, string> = {
  active: "bg-success",
  idle: "bg-text-muted",
  paused: "bg-warning",
  hibernating: "bg-info",
  error: "bg-danger",
  offline: "bg-border-strong",
};

/** One floor tab, as projected by the caller. */
export interface FloorTab {
  id: string;
  name: string;
  kind: FloorKind;
  state: OfficeState;
}

export interface FloorTabBarProps {
  floors: readonly FloorTab[];
  activeFloorId: string;
  onSelectFloor?: (id: string) => void;
  /** The "+" control and a full-bar folder drop both resolve to floor creation. */
  onCreateFloor?: () => void;
  /** Open the per-floor actions menu (rename / open-in-IDE / pause / close / delete). */
  onFloorMenu?: (id: string) => void;
  locale?: Locale;
}

export function FloorTabBar({
  floors,
  activeFloorId,
  onSelectFloor,
  onCreateFloor,
  onFloorMenu,
  locale = "en",
}: FloorTabBarProps) {
  const msg = translator(locale);

  return (
    <div
      data-testid="floor-tab-bar"
      // a folder dropped anywhere on the bar creates a floor bound to it (NV-8)
      role="tablist"
      aria-label={msg("floor.add")}
      className="flex h-10 flex-none select-none items-center gap-0.5 border-b border-border-subtle bg-surface-0 px-1.5"
      onDrop={(e) => {
        e.preventDefault();
        onCreateFloor?.();
      }}
      onDragOver={(e) => e.preventDefault()}
    >
      {floors.map((floor) => {
        const isActive = floor.id === activeFloorId;
        const isHome = floor.kind === "home";
        return (
          <div
            key={floor.id}
            data-testid={`floor-${floor.id}`}
            data-home={isHome || undefined}
            className={`flex h-7.5 items-center gap-1 rounded-md px-2.5 ${
              isActive ? "bg-surface-2" : "hover:bg-surface-1"
            }`}
          >
            <button
              type="button"
              data-testid={`floor-select-${floor.id}`}
              aria-current={isActive ? "page" : undefined}
              className="flex items-center gap-2 border-none bg-transparent p-0 text-sm text-text-secondary hover:text-text-primary"
              onClick={() => onSelectFloor?.(floor.id)}
            >
              <span
                data-testid={`floor-state-${floor.id}`}
                data-state={floor.state}
                className={`h-1.75 w-1.75 flex-none rounded-pill ${STATE_DOT[floor.state]}`}
                title={floor.state}
              />
              {isHome ? msg("floor.home") : floor.name}
            </button>
            {!isHome ? (
              <button
                type="button"
                data-testid={`floor-menu-${floor.id}`}
                title={msg("floor.rename")}
                className="flex h-5 w-5 items-center justify-center rounded-sm border-none bg-transparent text-text-muted hover:bg-surface-3 hover:text-text-primary"
                onClick={() => onFloorMenu?.(floor.id)}
              >
                ⋮
              </button>
            ) : null}
          </div>
        );
      })}
      <button
        type="button"
        data-testid="floor-add"
        title={msg("floor.add")}
        className="ml-0.5 flex h-7 w-7 flex-none items-center justify-center rounded-md border-none bg-transparent text-text-secondary hover:bg-surface-1 hover:text-text-primary"
        onClick={() => onCreateFloor?.()}
      >
        +
      </button>
      <div className="flex-1" />
    </div>
  );
}

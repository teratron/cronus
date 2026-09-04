/**
 * L1 · Floor tab bar — one tab per workspace (office).
 *
 * Presentation only: renders from an injected floor list and forwards selection,
 * creation, and per-floor menu intents. The pinned home floor renders first,
 * separated from the project floors by a rule, and offers no close/delete or
 * overflow menu. Each project tab carries a status dot reflecting the injected
 * `OfficeState` — the caller subscribes to the core's state stream and
 * re-renders; this component never polls.
 */

import { type Locale, translator } from "../shared/i18n";
import type { FloorKind } from "../shared/navigation";
import { Icon } from "./icons";

/** Live office lifecycle state, mirrored from the core taxonomy. */
export type OfficeState = "active" | "idle" | "paused" | "hibernating" | "error" | "offline";

const STATE_DOT: Record<OfficeState, string> = {
  active: "bg-success",
  idle: "bg-text-muted",
  paused: "bg-warning",
  hibernating: "bg-state-dormant",
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
  const home = floors.find((f) => f.kind === "home");
  const projects = floors.filter((f) => f.kind !== "home");

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: a folder dropped anywhere on the bar creates a floor; the "+" button is the keyboard-reachable equivalent
    <div
      data-testid="floor-tab-bar"
      className="flex h-10 flex-none select-none items-center gap-0.5 border-b border-border-subtle bg-surface-1 px-1.5"
      onDrop={(e) => {
        e.preventDefault();
        onCreateFloor?.();
      }}
      onDragOver={(e) => e.preventDefault()}
    >
      {home ? (
        <button
          type="button"
          data-testid="floor-home"
          data-home="true"
          aria-current={home.id === activeFloorId ? "page" : undefined}
          title={msg("floor.home")}
          className={`flex h-7.5 items-center gap-2 rounded-md px-2.75 text-base transition-colors ${
            home.id === activeFloorId
              ? "bg-surface-3 text-text-primary"
              : "bg-transparent text-text-secondary hover:bg-surface-2 hover:text-text-primary"
          }`}
          onClick={() => onSelectFloor?.(home.id)}
        >
          <Icon name="home" size={14} />
          {home.name}
        </button>
      ) : null}

      {home && projects.length > 0 ? (
        <span className="mx-1 h-4 w-px flex-none bg-border-subtle" />
      ) : null}

      {projects.map((floor) => {
        const active = floor.id === activeFloorId;
        return (
          <div
            key={floor.id}
            data-testid={`floor-${floor.id}`}
            className={`group flex h-7.5 items-center gap-0.5 rounded-md py-0 pr-1 pl-2.75 transition-colors ${
              active ? "bg-surface-3" : "bg-transparent hover:bg-surface-2"
            }`}
          >
            <button
              type="button"
              data-testid={`floor-select-${floor.id}`}
              aria-current={active ? "page" : undefined}
              className={`flex items-center gap-2 border-none bg-transparent p-0 text-base transition-colors hover:text-text-primary ${
                active ? "text-text-primary" : "text-text-secondary"
              } ${floor.state === "hibernating" ? "opacity-85" : ""}`}
              onClick={() => onSelectFloor?.(floor.id)}
            >
              <span
                data-testid={`floor-state-${floor.id}`}
                data-state={floor.state}
                title={msg(`office.state.${floor.state}` as const)}
                className={`h-1.75 w-1.75 flex-none rounded-pill ${STATE_DOT[floor.state]}`}
              />
              {floor.name}
            </button>
            <button
              type="button"
              data-testid={`floor-menu-${floor.id}`}
              title={msg("floor.actions")}
              className="flex h-5 w-5 items-center justify-center rounded-sm bg-transparent text-text-muted transition-colors hover:bg-surface-active hover:text-text-primary"
              onClick={() => onFloorMenu?.(floor.id)}
            >
              <Icon name="dots" size={14} />
            </button>
          </div>
        );
      })}

      <button
        type="button"
        data-testid="floor-add"
        title={msg("floor.add")}
        className="ml-0.5 flex h-7 w-7 flex-none items-center justify-center rounded-sm bg-transparent text-text-secondary transition-colors hover:bg-surface-hover hover:text-text-primary"
        onClick={() => onCreateFloor?.()}
      >
        <Icon name="plus" />
      </button>

      <div className="flex-1" />
    </div>
  );
}

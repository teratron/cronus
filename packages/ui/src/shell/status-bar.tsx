/**
 * Status bar — the shell's bottom rail.
 *
 * A read-only strip: which floor is live and in what run state, the two budget
 * meters, the quality-gate signal, and the process-monitor entry. Presentation
 * only — every value is injected, and a meter renders nothing rather than a
 * guess when its value is absent.
 */

import { type Locale, translator } from "../shared/i18n";
import { Icon, type IconName } from "./icons";
import type { RunControlState } from "./subsystem-sidebar";

const RUN_TONE: Record<RunControlState, string> = {
  running: "text-success",
  paused: "text-warning",
  stopped: "text-text-muted",
};

const RUN_DOT: Record<RunControlState, string> = {
  running: "bg-success",
  paused: "bg-warning",
  stopped: "bg-text-muted",
};

const RUN_ICON: Record<RunControlState, IconName> = {
  running: "play",
  paused: "pause",
  stopped: "stop",
};

/** One budget meter: a filled fraction plus the label the user actually reads. */
export interface BudgetMeter {
  /** 0–100. Clamped on render; a bar never overflows its track. */
  percent: number;
  label: string;
  /** `true` paints the fill in the danger tone — a budget in trouble. */
  critical?: boolean;
}

export interface StatusBarProps {
  floorName?: string;
  runState?: RunControlState;
  /** Per-session budget consumption. */
  session?: BudgetMeter;
  /** Rolling-week budget consumption. */
  weekly?: BudgetMeter;
  /** Whether the quality gates are currently green (absent → not reported). */
  gatesGreen?: boolean;
  /** Resident-set summary for the process monitor entry. */
  memoryLabel?: string;
  onOpenProcessMonitor?: () => void;
  locale?: Locale;
}

function Meter({ meter, title }: { meter: BudgetMeter; title: string }) {
  const pct = Math.max(0, Math.min(100, meter.percent));
  return (
    <span className="flex items-center gap-1.75" title={title}>
      <span className="h-1 w-13 overflow-hidden rounded-xs bg-surface-active">
        <span
          className={`block h-full rounded-xs ${meter.critical ? "bg-danger" : "bg-success"}`}
          style={{
            width: `${pct}%`,
          }}
        />
      </span>
      {meter.label}
    </span>
  );
}

export function StatusBar({
  floorName,
  runState = "stopped",
  session,
  weekly,
  gatesGreen,
  memoryLabel,
  onOpenProcessMonitor,
  locale = "en",
}: StatusBarProps) {
  const msg = translator(locale);
  const runLabel = msg(
    runState === "running"
      ? "status.run.running"
      : runState === "paused"
        ? "status.run.paused"
        : "status.run.stopped",
  );

  return (
    <div
      data-testid="status-bar"
      className="flex h-6.5 flex-none select-none items-center gap-3.5 border-border-subtle border-t bg-surface-1 px-3 text-text-secondary text-xs"
    >
      {floorName ? (
        <span className="flex items-center gap-1.75">
          <span className={`h-1.75 w-1.75 rounded-pill ${RUN_DOT[runState]}`} />
          {floorName}
        </span>
      ) : null}

      {session ? <Meter meter={session} title={msg("status.session-budget")} /> : null}
      {weekly ? <Meter meter={weekly} title={msg("status.weekly-budget")} /> : null}

      {gatesGreen ? (
        <span className="flex items-center gap-1.5">
          <Icon name="check" size={11} className="text-success" />
          {msg("status.gates-green")}
        </span>
      ) : null}

      <span
        data-testid="status-run-state"
        data-state={runState}
        title={msg("status.run-state")}
        className={`flex items-center gap-1.5 font-medium ${RUN_TONE[runState]}`}
      >
        <Icon name={RUN_ICON[runState]} size={11} />
        {runLabel}
      </span>

      <div className="flex-1" />

      {memoryLabel ? (
        <button
          type="button"
          data-testid="status-process-monitor"
          title={msg("status.memory")}
          className="flex h-6.5 items-center gap-1.5 bg-transparent px-2 text-text-secondary text-xs transition-colors hover:bg-surface-hover hover:text-text-strong"
          onClick={onOpenProcessMonitor}
        >
          <Icon name="activity" size={12} />
          {memoryLabel}
        </button>
      ) : null}
    </div>
  );
}

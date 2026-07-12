/**
 * Office View panel: graph + spatial-floor projections of one office.
 *
 * Both render modes consume the same injected `OfficeProjection` — the panel
 * is a pure projection renderer (no data fetching, no office mutation). Node
 * selection is forwarded as an intent; the render mode is owned by the
 * caller, like every other piece of view state.
 */

import { type Locale, translator } from "./i18n";

/** One agent node of the office projection. */
export interface OfficeAgent {
  id: string;
  name: string;
  role: string;
  /** Reporting edge target (manager agent id). */
  reportsTo?: string;
  /** Whether a live session is running for this agent. */
  active: boolean;
  /** Cosmetic floor placement; agents without a room share the open space. */
  room?: string;
}

/** One task node (board card) of the office projection. */
export interface OfficeTask {
  id: string;
  title: string;
  /** Assignment edge target (agent id). */
  assignee?: string;
}

/** The projected office model both render modes consume. */
export interface OfficeProjection {
  agents: OfficeAgent[];
  tasks: OfficeTask[];
}

/** The two representations of the same model. */
export type OfficeRenderMode = "graph" | "floor";

export interface OfficeViewProps {
  projection: OfficeProjection;
  /** Render mode — caller-owned view state. */
  mode: OfficeRenderMode;
  /** Inspect intent for a node (agent or task); the panel never drills itself. */
  onInspect?: (nodeId: string) => void;
  locale?: Locale;
}

export function OfficeViewPanel({ projection, mode, onInspect, locale = "en" }: OfficeViewProps) {
  const msg = translator(locale);
  if (projection.agents.length === 0 && projection.tasks.length === 0) {
    return <p data-testid="office-empty">{msg("office.empty")}</p>;
  }
  return mode === "graph" ? (
    <GraphRender projection={projection} onInspect={onInspect} />
  ) : (
    <FloorRender projection={projection} onInspect={onInspect} />
  );
}

function GraphRender({
  projection,
  onInspect,
}: {
  projection: OfficeProjection;
  onInspect?: (nodeId: string) => void;
}) {
  const reportingEdges = projection.agents.filter((agent) => agent.reportsTo);
  const assignmentEdges = projection.tasks.filter((task) => task.assignee);
  return (
    <div data-testid="office-graph">
      <ul>
        {projection.agents.map((agent) => (
          <li key={agent.id} data-testid={`node-agent-${agent.id}`}>
            <button type="button" onClick={() => onInspect?.(agent.id)}>
              {agent.name} · {agent.role}
              {agent.active ? <span data-testid={`active-${agent.id}`}> ●</span> : null}
            </button>
          </li>
        ))}
        {projection.tasks.map((task) => (
          <li key={task.id} data-testid={`node-task-${task.id}`}>
            <button type="button" onClick={() => onInspect?.(task.id)}>
              {task.title}
            </button>
          </li>
        ))}
      </ul>
      <ul>
        {reportingEdges.map((agent) => (
          <li key={agent.id} data-testid={`edge-reports-${agent.id}-${agent.reportsTo}`} />
        ))}
        {assignmentEdges.map((task) => (
          <li key={task.id} data-testid={`edge-assigned-${task.id}-${task.assignee}`} />
        ))}
      </ul>
    </div>
  );
}

function FloorRender({
  projection,
  onInspect,
}: {
  projection: OfficeProjection;
  onInspect?: (nodeId: string) => void;
}) {
  const rooms = new Map<string, OfficeAgent[]>();
  for (const agent of projection.agents) {
    const room = agent.room ?? "open-space";
    const seated = rooms.get(room) ?? [];
    seated.push(agent);
    rooms.set(room, seated);
  }
  return (
    <div data-testid="office-floor">
      {[
        ...rooms.entries(),
      ].map(([room, seated]) => (
        <section key={room} data-testid={`room-${room}`}>
          <h3>{room}</h3>
          <ul>
            {seated.map((agent) => (
              <li key={agent.id} data-testid={`seat-${agent.id}`}>
                <button type="button" onClick={() => onInspect?.(agent.id)}>
                  {agent.name}
                </button>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { type OfficeProjection, OfficeViewPanel } from "./office-view";
import { Workbench } from "./surfaces";

const projection: OfficeProjection = {
  agents: [
    {
      id: "mgr",
      name: "Manager",
      role: "orchestrator",
      active: true,
      room: "hq",
    },
    {
      id: "eng",
      name: "Backend Eng",
      role: "backend-engineer",
      reportsTo: "mgr",
      active: false,
    },
  ],
  tasks: [
    {
      id: "t1",
      title: "Migrate endpoints",
      assignee: "eng",
    },
  ],
};

describe("OfficeViewPanel", () => {
  it("graph mode renders agent/task nodes with reporting and assignment edges", () => {
    render(<OfficeViewPanel projection={projection} mode="graph" />);

    expect(screen.getByTestId("node-agent-mgr")).toBeInTheDocument();
    expect(screen.getByTestId("node-agent-eng")).toBeInTheDocument();
    expect(screen.getByTestId("node-task-t1")).toBeInTheDocument();
    expect(screen.getByTestId("edge-reports-eng-mgr")).toBeInTheDocument();
    expect(screen.getByTestId("edge-assigned-t1-eng")).toBeInTheDocument();
    expect(screen.getByTestId("active-mgr")).toBeInTheDocument();
    expect(screen.queryByTestId("active-eng")).not.toBeInTheDocument();
  });

  it("floor mode seats the same projection into rooms", () => {
    render(<OfficeViewPanel projection={projection} mode="floor" />);

    expect(screen.getByTestId("room-hq")).toBeInTheDocument();
    expect(screen.getByTestId("room-open-space")).toBeInTheDocument();
    expect(screen.getByTestId("seat-mgr")).toBeInTheDocument();
    expect(screen.getByTestId("seat-eng")).toBeInTheDocument();
  });

  it("re-renders when the next projection arrives (projection, not source)", () => {
    const { rerender } = render(<OfficeViewPanel projection={projection} mode="graph" />);
    expect(screen.queryByTestId("node-agent-new")).not.toBeInTheDocument();

    const next: OfficeProjection = {
      agents: [
        ...projection.agents,
        {
          id: "new",
          name: "Reviewer",
          role: "code-reviewer",
          active: false,
        },
      ],
      tasks: projection.tasks,
    };
    rerender(<OfficeViewPanel projection={next} mode="graph" />);
    expect(screen.getByTestId("node-agent-new")).toBeInTheDocument();
  });

  it("forwards inspect intents and renders an empty state", () => {
    const onInspect = vi.fn();
    render(<OfficeViewPanel projection={projection} mode="graph" onInspect={onInspect} />);
    fireEvent.click(screen.getByTestId("node-agent-eng").querySelector("button") as HTMLElement);
    expect(onInspect).toHaveBeenCalledWith("eng");

    render(
      <OfficeViewPanel
        projection={{
          agents: [],
          tasks: [],
        }}
        mode="graph"
      />,
    );
    expect(screen.getByTestId("office-empty")).toBeInTheDocument();
  });

  it("the office surface hosts the panel when a projection is supplied", () => {
    render(<Workbench active="office" office={projection} officeMode="graph" />);
    expect(screen.getByTestId("office-graph")).toBeInTheDocument();
  });
});

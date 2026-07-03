import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { DashboardPanel, type DashboardProjection } from "./dashboard";
import { Workbench } from "./surfaces";

const projection: DashboardProjection = {
  offices: [
    {
      id: "acme",
      name: "Acme Project",
      activeAgents: 2,
      cardsByState: { running: 1, blocked: 2, done: 7 },
    },
  ],
  building: { offices: 3, activeAgents: 5, totalCards: 42 },
};

describe("DashboardPanel", () => {
  it("renders per-office and building-aggregate statistics from a projection", () => {
    render(<DashboardPanel projection={projection} />);

    expect(screen.getByTestId("dashboard-office-acme")).toBeInTheDocument();
    expect(screen.getByTestId("office-active-acme")).toHaveTextContent("2");
    expect(screen.getByTestId("cards-acme-blocked")).toHaveTextContent(
      "blocked: 2",
    );
    expect(screen.getByTestId("building-offices")).toHaveTextContent("3");
    expect(screen.getByTestId("building-active")).toHaveTextContent("5");
    expect(screen.getByTestId("building-cards")).toHaveTextContent("42");
  });

  it("updates when the next projection arrives", () => {
    const { rerender } = render(<DashboardPanel projection={projection} />);
    rerender(
      <DashboardPanel
        projection={{
          ...projection,
          building: { offices: 3, activeAgents: 6, totalCards: 43 },
        }}
      />,
    );
    expect(screen.getByTestId("building-active")).toHaveTextContent("6");
  });

  it("omits the building section when the projection has none (per-office view)", () => {
    render(<DashboardPanel projection={{ offices: projection.offices }} />);
    expect(screen.queryByTestId("dashboard-building")).not.toBeInTheDocument();
  });

  it("the dashboard surface hosts the panel when a projection is supplied", () => {
    render(<Workbench active="dashboard" dashboard={projection} />);
    expect(screen.getByTestId("dashboard")).toBeInTheDocument();
  });
});

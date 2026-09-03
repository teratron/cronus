import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../shared/i18n";
import { SURFACES } from "../shared/surface-catalog";
import { resolveTheme, themeAttributes } from "../shared/theme";
import type { DashboardProjection, OfficeProjection } from "../surfaces";
import { Workbench } from "./workbench";

describe("workbench (render-from-state)", () => {
  it("renders the active surface from injected state without mutating it", () => {
    const onSelect = vi.fn();
    render(<Workbench active="dashboard" onSelect={onSelect} status="ready" />);

    expect(screen.getByTestId("surface-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("status")).toHaveTextContent("ready");

    // Clicking a surface forwards an intent — the workbench does not switch itself.
    fireEvent.click(screen.getByTestId("nav-office"));
    expect(onSelect).toHaveBeenCalledWith("office");
    expect(screen.getByTestId("surface-dashboard")).toBeInTheDocument();
  });

  it("exposes all five surfaces in the navigation", () => {
    render(<Workbench active="office" />);
    for (const surface of SURFACES) {
      expect(screen.getByTestId(`nav-${surface}`)).toBeInTheDocument();
    }
  });
});

describe("workbench surface hosting", () => {
  const dashboard: DashboardProjection = {
    offices: [
      {
        id: "acme",
        name: "Acme Project",
        activeAgents: 2,
        cardsByState: {
          running: 1,
          blocked: 2,
          done: 7,
        },
      },
    ],
    building: {
      offices: 3,
      activeAgents: 5,
      totalCards: 42,
    },
  };

  const office: OfficeProjection = {
    agents: [
      {
        id: "mgr",
        name: "Manager",
        role: "orchestrator",
        active: true,
        room: "hq",
      },
    ],
    tasks: [
      {
        id: "t1",
        title: "Migrate endpoints",
        assignee: "mgr",
      },
    ],
  };

  it("the dashboard surface hosts the panel when a projection is supplied", () => {
    render(<Workbench active="dashboard" dashboard={dashboard} />);
    expect(screen.getByTestId("dashboard")).toBeInTheDocument();
  });

  it("the office surface hosts the panel when a projection is supplied", () => {
    render(<Workbench active="office" office={office} officeMode="graph" />);
    expect(screen.getByTestId("office-graph")).toBeInTheDocument();
  });
});

describe("theming", () => {
  it("resolves the system theme against the OS preference", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });

  it("switching theme swaps the token attributes on the root", () => {
    const { rerender } = render(<Workbench active="office" theme="dark" />);
    expect(screen.getByTestId("workbench")).toHaveAttribute("data-theme", "dark");
    expect(screen.getByTestId("workbench").className).toContain("dark");

    rerender(<Workbench active="office" theme="light" />);
    expect(screen.getByTestId("workbench")).toHaveAttribute("data-theme", "light");
    expect(screen.getByTestId("workbench").className).not.toContain("dark");
  });

  it("token attributes are derived, never literal colors", () => {
    expect(themeAttributes("dark")).toEqual({
      "data-theme": "dark",
      className: "dark",
    });
    expect(themeAttributes("light")).toEqual({
      "data-theme": "light",
      className: "",
    });
  });
});

describe("localization", () => {
  it("switching locale swaps every visible navigation string", () => {
    const { rerender } = render(<Workbench active="dashboard" locale="en" />);
    expect(screen.getByTestId("nav-dashboard")).toHaveTextContent(t("en", "surface.dashboard"));

    rerender(<Workbench active="dashboard" locale="ru" />);
    for (const surface of SURFACES) {
      expect(screen.getByTestId(`nav-${surface}`)).toHaveTextContent(t("ru", `surface.${surface}`));
    }
  });

  it("the connecting placeholder is localized, not hardcoded", () => {
    render(<Workbench active="office" locale="ru" />);
    expect(screen.getByTestId("status")).toHaveTextContent(t("ru", "status.connecting"));
  });

  it("a key missing from a locale falls back to English", () => {
    // Simulate a partial catalog: the ru catalog is Partial<Catalog> by type;
    // resolution must fall back to the English value rather than render blank.
    expect(t("ru", "app.title")).toBe(t("en", "app.title"));
    expect(t("ru", "surface.office")).not.toHaveLength(0);
  });
});

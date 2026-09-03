import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Projection } from "../shared/projection";
import type { OfficeProjection } from "../surfaces";
import { SurfaceRouter } from "./surface-router";

const emptyOffice: OfficeProjection = {
  agents: [],
  tasks: [],
};

describe("SurfaceRouter — four-state projection rendering", () => {
  it("a non-projected tab is always an explicit placeholder", () => {
    render(<SurfaceRouter active="wiki" />);
    const surface = screen.getByTestId("surface-wiki");
    expect(surface).toHaveAttribute("data-placeholder", "true");
    expect(surface).not.toHaveAttribute("data-state");
  });

  it("office: absent and an explicit unrequested both render the placeholder, marked", () => {
    const { rerender } = render(<SurfaceRouter active="office" />);
    expect(screen.getByTestId("surface-office")).toHaveAttribute("data-state", "unrequested");
    rerender(
      <SurfaceRouter
        active="office"
        office={{
          kind: "unrequested",
        }}
      />,
    );
    expect(screen.getByTestId("surface-office")).toHaveAttribute("data-state", "unrequested");
  });

  it("office pending renders the placeholder marked pending, not the panel", () => {
    render(
      <SurfaceRouter
        active="office"
        office={{
          kind: "pending",
        }}
      />,
    );
    const surface = screen.getByTestId("surface-office");
    expect(surface).toHaveAttribute("data-placeholder", "true");
    expect(surface).toHaveAttribute("data-state", "pending");
  });

  it("office unavailable carries the reason, never a panel", () => {
    const p: Projection<OfficeProjection> = {
      kind: "unavailable",
      reason: "core channel closed",
    };
    render(<SurfaceRouter active="office" office={p} />);
    const surface = screen.getByTestId("surface-office");
    expect(surface).toHaveAttribute("data-state", "unavailable");
    expect(surface).toHaveAttribute("data-reason", "core channel closed");
  });

  it("office loaded — even with empty data — renders the panel, not a placeholder", () => {
    render(
      <SurfaceRouter
        active="office"
        office={{
          kind: "loaded",
          data: emptyOffice,
        }}
      />,
    );
    const surface = screen.getByTestId("surface-office");
    expect(surface).not.toHaveAttribute("data-placeholder");
    expect(surface).not.toHaveAttribute("data-state");
  });
});

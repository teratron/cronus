import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { BuildingShell } from "./building-shell";
import type { FloorTab } from "./floor-tab-bar";

const HOME: FloorTab = {
  id: "home",
  name: "Home",
  kind: "home",
  state: "idle",
};

const base = {
  floors: [
    HOME,
  ],
  activeFloorId: "home",
} as const;

describe("store compliance", () => {
  it("render-from-state: the same props render the same output", () => {
    const props = {
      ...base,
      theme: "dark" as const,
      locale: "en" as const,
    };
    const first = render(<BuildingShell {...props} />);
    const firstHtml = first.container.innerHTML;
    first.unmount();

    const second = render(<BuildingShell {...props} />);
    expect(second.container.innerHTML).toBe(firstHtml);
  });

  it("themed root carries token attributes, never an inline style literal", () => {
    render(<BuildingShell {...base} theme="dark" />);
    const root = screen.getByTestId("building-shell");
    expect(root.getAttribute("style")).toBeNull();
    expect(root).toHaveAttribute("data-theme", "dark");
    expect(root).toHaveAttribute("data-scheme", "default");
  });

  it("every visible string resolves through i18n (a locale swap leaves no stale text)", () => {
    const { rerender } = render(<BuildingShell {...base} locale="en" />);
    const english = screen.getByTestId("building-shell").textContent ?? "";

    rerender(<BuildingShell {...base} locale="ru" />);
    const russian = screen.getByTestId("building-shell").textContent ?? "";

    expect(russian).not.toBe(english);
  });
});

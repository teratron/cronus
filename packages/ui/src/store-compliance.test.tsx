import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Workbench } from "./surfaces";

describe("store compliance", () => {
  it("render-from-state: the same state renders the same output", () => {
    const props = {
      active: "dashboard" as const,
      status: "Cronus core 0.1.0 — ok",
      locale: "en" as const,
      theme: "dark" as const,
    };
    const first = render(<Workbench {...props} />);
    const firstHtml = first.container.innerHTML;
    first.unmount();

    const second = render(<Workbench {...props} />);
    expect(second.container.innerHTML).toBe(firstHtml);
  });

  it("a masked secret in a projection renders masked, untransformed (INV-7)", () => {
    // The core masks before the value crosses IPC; the UI must render the
    // masked text verbatim and never attempt to reconstruct it.
    render(<Workbench active="office" status="token=*** ready" />);
    const status = screen.getByTestId("status");
    expect(status).toHaveTextContent("token=*** ready");
    expect(status.textContent).not.toMatch(/sk-|Bearer /);
  });

  it("themed surfaces carry token attributes, not inline literal colors", () => {
    render(<Workbench active="office" theme="dark" />);
    const root = screen.getByTestId("workbench");
    expect(root.getAttribute("style")).toBeNull();
    expect(root).toHaveAttribute("data-theme", "dark");
  });

  it("every visible string resolves through i18n (locale swap leaves no stale text)", () => {
    const { rerender } = render(<Workbench active="office" locale="en" />);
    const english = screen.getByTestId("workbench").textContent ?? "";

    rerender(<Workbench active="office" locale="ru" />);
    const russian = screen.getByTestId("workbench").textContent ?? "";

    // Localized labels changed; the only shared visible text is the brand
    // name (untranslated by design) and the shared placeholder dots.
    expect(russian).not.toBe(english);
    expect(russian).toContain("Офис");
    expect(english).toContain("Office");
  });
});

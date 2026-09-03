import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { MessageKey } from "../shared/i18n";
import { type BindingLayer, mergeKeymap } from "../shared/keymap";
import { KeymapSurface } from "./keymap-surface";

const base: BindingLayer = {
  name: "base",
  bindings: [
    {
      actionId: "file.settings",
      sequence: [
        "Ctrl+,",
      ],
    },
    {
      actionId: "view.command-palette",
      sequence: [
        "Ctrl+Shift+P",
      ],
    },
  ],
};
const user: BindingLayer = {
  name: "user",
  bindings: [
    {
      actionId: "view.command-palette",
      sequence: [
        "Ctrl+Shift+J",
      ],
    },
  ],
};

const labelFor = (id: string): MessageKey | undefined =>
  id === "file.settings" ? "menu.file.settings" : undefined;

describe("KeymapSurface", () => {
  it("renders each action with its effective binding and originating layer", () => {
    render(
      <KeymapSurface
        bindings={mergeKeymap([
          base,
          user,
        ])}
        labelFor={labelFor}
      />,
    );

    const settings = screen.getByTestId("keymap-row-file.settings");
    expect(settings).toHaveAttribute("data-layer", "base");
    expect(settings).toHaveTextContent("Ctrl+,");

    const palette = screen.getByTestId("keymap-row-view.command-palette");
    // the user layer overrode it — the surface shows that
    expect(palette).toHaveAttribute("data-layer", "user");
    expect(palette).toHaveTextContent("Ctrl+Shift+J");
    expect(screen.getByTestId("keymap-origin-view.command-palette")).toHaveTextContent("user");
  });

  it("falls back to the raw action id when no label is known", () => {
    render(
      <KeymapSurface
        bindings={mergeKeymap([
          base,
        ])}
        labelFor={() => undefined}
      />,
    );
    expect(screen.getByTestId("keymap-row-view.command-palette")).toHaveTextContent(
      "view.command-palette",
    );
  });

  it("renders nothing for an empty table", () => {
    render(<KeymapSurface bindings={[]} labelFor={labelFor} />);
    expect(screen.getByTestId("keymap-surface")).toBeEmptyDOMElement();
  });
});

import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { App } from "./App";
import { createCoreClient, type InvokeFn } from "./shared/bridge";

describe("App", () => {
  it("renders the supplied core status (render-from-state)", () => {
    render(<App status="Cronus core 0.1.0" />);
    expect(screen.getByTestId("status")).toHaveTextContent("Cronus core 0.1.0");
  });

  it("shows a connecting placeholder when no status is provided", () => {
    render(<App />);
    expect(screen.getByTestId("status")).toHaveTextContent("connecting…");
  });

  it("owns surface selection as view state", () => {
    render(<App status="ok" />);
    expect(screen.getByTestId("surface-office")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("nav-board"));
    expect(screen.getByTestId("surface-board")).toBeInTheDocument();
  });

  it("round-trips: a bridged status value renders in the App surface", async () => {
    const invoke = vi.fn().mockResolvedValue("Cronus core 0.1.0 — bridged");
    const client = createCoreClient(invoke as InvokeFn);

    const status = await client.status();
    render(<App status={status} />);

    expect(screen.getByTestId("status")).toHaveTextContent("Cronus core 0.1.0 — bridged");
  });
});

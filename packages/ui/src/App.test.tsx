import { render, screen } from "@testing-library/react";
import { App } from "./App";

describe("App", () => {
  it("renders the supplied core status (render-from-state)", () => {
    render(<App status="Cronus core 0.1.0" />);
    expect(screen.getByTestId("status")).toHaveTextContent("Cronus core 0.1.0");
  });

  it("shows a connecting placeholder when no status is provided", () => {
    render(<App />);
    expect(screen.getByTestId("status")).toHaveTextContent("connecting…");
  });
});

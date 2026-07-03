import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { createCoreClient, type InvokeFn } from "./bridge";

describe("core bridge client", () => {
  it("marshals status to the capability_status IPC command", async () => {
    const invoke = vi.fn().mockResolvedValue("Cronus core 0.1.0 — ok");
    const client = createCoreClient(invoke as InvokeFn);

    await expect(client.status()).resolves.toBe("Cronus core 0.1.0 — ok");
    expect(invoke).toHaveBeenCalledWith("capability_status");
  });

  it("marshals version to the capability_version IPC command", async () => {
    const invoke = vi.fn().mockResolvedValue("0.1.0");
    const client = createCoreClient(invoke as InvokeFn);

    await expect(client.version()).resolves.toBe("0.1.0");
    expect(invoke).toHaveBeenCalledWith("capability_version");
  });

  it("round-trips: a bridged status value renders in the App surface", async () => {
    const invoke = vi.fn().mockResolvedValue("Cronus core 0.1.0 — bridged");
    const client = createCoreClient(invoke as InvokeFn);

    const status = await client.status();
    render(<App status={status} />);

    expect(screen.getByTestId("status")).toHaveTextContent(
      "Cronus core 0.1.0 — bridged",
    );
  });
});

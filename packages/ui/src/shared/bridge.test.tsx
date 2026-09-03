import { describe, expect, it, vi } from "vitest";
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
});

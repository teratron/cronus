import { describe, expect, it } from "vitest";
import {
  composeSidebar,
  type Floor,
  isCanonicalOrder,
  isChildLayer,
  isClosable,
  isUnloadable,
  NAV_LAYERS,
  SIDEBAR_TABS,
  settingsTier,
  shouldLoad,
} from "./navigation";

const homeFloor: Floor = {
  id: "home",
  name: "Home",
  kind: "home",
  loaded: true,
  hasRunningTask: false,
};

const projectFloor: Floor = {
  id: "proj-1",
  name: "Project 1",
  kind: "project",
  loaded: false,
  hasRunningTask: false,
};

describe("navigation model", () => {
  it("exposes the canonical sidebar order and rejects reordering (NV-1)", () => {
    expect(SIDEBAR_TABS[0]).toBe("chat");
    expect(SIDEBAR_TABS[SIDEBAR_TABS.length - 1]).toBe("settings");
    expect(isCanonicalOrder(SIDEBAR_TABS)).toBe(true);
    const reordered = [...SIDEBAR_TABS].reverse();
    expect(isCanonicalOrder(reordered)).toBe(false);
  });

  it("keeps the canonical set intact below pinned shortcuts (NV-1)", () => {
    const { pinned, canonical } = composeSidebar(["kanban", "memory"]);
    expect(pinned).toEqual(["kanban", "memory"]);
    expect(isCanonicalOrder(canonical)).toBe(true);
  });

  it("enforces strict four-layer nesting (NV-6)", () => {
    expect(NAV_LAYERS).toEqual(["building", "floor", "subsystem", "mechanism"]);
    expect(isChildLayer("building", "floor")).toBe(true);
    expect(isChildLayer("floor", "subsystem")).toBe(true);
    // Non-adjacent or reversed layers are not parent-child.
    expect(isChildLayer("building", "subsystem")).toBe(false);
    expect(isChildLayer("subsystem", "floor")).toBe(false);
  });

  it("pins the home floor as non-closable and always loaded (NV-9)", () => {
    expect(isClosable(homeFloor)).toBe(false);
    expect(isClosable(projectFloor)).toBe(true);
    expect(shouldLoad(homeFloor, "proj-1")).toBe(true); // always, even inactive
  });

  it("lazy-loads project floors only when active or running (NV-2)", () => {
    expect(shouldLoad(projectFloor, "home")).toBe(false); // inactive, idle
    expect(shouldLoad(projectFloor, "proj-1")).toBe(true); // active
    const running = { ...projectFloor, hasRunningTask: true };
    expect(shouldLoad(running, "home")).toBe(true); // running task monitored
  });

  it("marks inactive idle project floors unloadable but never home (NV-2/NV-9)", () => {
    expect(isUnloadable(projectFloor, "home")).toBe(true);
    expect(isUnloadable(projectFloor, "proj-1")).toBe(false); // active
    expect(isUnloadable(homeFloor, "proj-1")).toBe(false); // home never unloads
  });

  it("routes settings keys to the correct tier (NV-4)", () => {
    expect(settingsTier("appearance")).toBe("global");
    expect(settingsTier("models")).toBe("global");
    expect(settingsTier("office-identity")).toBe("local");
    expect(settingsTier("git")).toBe("local");
  });
});

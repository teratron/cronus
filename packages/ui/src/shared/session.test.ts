import { describe, expect, it } from "vitest";
import type { Projection } from "./projection";
import {
  allSettled,
  createSessionStore,
  type RequestStatus,
  statusOf,
  unavailableIds,
} from "./session";

const loaded: Projection<number> = {
  kind: "loaded",
  data: 0,
};
const pending: Projection<number> = {
  kind: "pending",
};
const unavailable: Projection<number> = {
  kind: "unavailable",
  reason: "x",
};
const unrequested: Projection<number> = {
  kind: "unrequested",
};

describe("session domain — derived per-projection request status", () => {
  it("maps each projection kind onto a request status", () => {
    const cases: [
      Projection<unknown>,
      RequestStatus,
    ][] = [
      [
        unrequested,
        "idle",
      ],
      [
        loaded,
        "idle",
      ],
      [
        pending,
        "pending",
      ],
      [
        unavailable,
        "failed",
      ],
    ];
    for (const [p, expected] of cases) {
      expect(statusOf(p)).toBe(expected);
    }
  });

  it("observe indexes a projection's status; an unchanged status is a no-op", () => {
    const store = createSessionStore();
    store.dispatch({
      type: "observe",
      id: "floors",
      projection: pending,
    });
    expect(store.snapshot().status).toEqual({
      floors: "pending",
    });

    const settled = store.snapshot();
    store.dispatch({
      type: "observe",
      id: "floors",
      projection: pending,
    });
    expect(store.snapshot()).toBe(settled);

    store.dispatch({
      type: "observe",
      id: "floors",
      projection: loaded,
    });
    expect(store.snapshot().status).toEqual({
      floors: "idle",
    });
  });

  it("distinguishes loading from unavailable across several projections", () => {
    const store = createSessionStore();
    store.dispatch({
      type: "observe",
      id: "floors",
      projection: loaded,
    });
    store.dispatch({
      type: "observe",
      id: "badges",
      projection: pending,
    });
    store.dispatch({
      type: "observe",
      id: "tree",
      projection: unavailable,
    });

    expect(allSettled(store.snapshot())).toBe(false);
    expect(unavailableIds(store.snapshot())).toEqual([
      "tree",
    ]);
  });

  it("allSettled is true only when nothing is in flight or failed", () => {
    const store = createSessionStore();
    store.dispatch({
      type: "observe",
      id: "a",
      projection: loaded,
    });
    store.dispatch({
      type: "observe",
      id: "b",
      projection: unrequested,
    });
    expect(allSettled(store.snapshot())).toBe(true);
  });

  it("forget drops an id; forgetting an absent id is a no-op", () => {
    const store = createSessionStore();
    store.dispatch({
      type: "observe",
      id: "a",
      projection: pending,
    });
    const before = store.snapshot();
    store.dispatch({
      type: "forget",
      id: "missing",
    });
    expect(store.snapshot()).toBe(before);
    store.dispatch({
      type: "forget",
      id: "a",
    });
    expect(store.snapshot().status).toEqual({});
  });
});

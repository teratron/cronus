import { describe, expect, it } from "vitest";
import {
  createProjectionStore,
  isLoaded,
  isUnavailable,
  type Projection,
  projectionReducer,
} from "./projection";

describe("Projection — the four-state cache", () => {
  it("a fresh store is unrequested, never a default-empty value", () => {
    const store = createProjectionStore<string[]>();
    const snap = store.snapshot();
    expect(snap.kind).toBe("unrequested");
    // there is no `.data` to mistake for a real result
    expect("data" in snap).toBe(false);
  });

  it("request -> pending -> loaded carries data only in the loaded state", () => {
    const store = createProjectionStore<string[]>();
    store.dispatch({
      type: "request",
    });
    expect(store.snapshot().kind).toBe("pending");
    store.dispatch({
      type: "fulfill",
      data: [],
    });
    const snap = store.snapshot();
    expect(snap.kind).toBe("loaded");
    expect(isLoaded(snap) && snap.data).toEqual([]);
  });

  it("loaded-empty and unavailable stay separately observable — the regression this guards", () => {
    const empty = projectionReducer<string[]>(
      {
        kind: "unrequested",
      },
      {
        type: "fulfill",
        data: [],
      },
    );
    const failed = projectionReducer<string[]>(
      {
        kind: "unrequested",
      },
      {
        type: "fail",
        reason: "channel closed",
      },
    );
    expect(empty.kind).toBe("loaded");
    expect(failed.kind).toBe("unavailable");
    expect(empty.kind).not.toBe(failed.kind);
    // a consumer that only asks "is there data" cannot tell them apart; the kind can
    expect(isLoaded(empty)).toBe(true);
    expect(isLoaded(failed)).toBe(false);
    expect(isUnavailable(failed) && failed.reason).toBe("channel closed");
  });

  it("fail always carries a reason; reset returns to unrequested", () => {
    const store = createProjectionStore<number>({
      kind: "loaded",
      data: 3,
    });
    store.dispatch({
      type: "fail",
      reason: "core offline",
    });
    expect(store.snapshot()).toEqual({
      kind: "unavailable",
      reason: "core offline",
    });
    store.dispatch({
      type: "reset",
    });
    expect(store.snapshot().kind).toBe("unrequested");
  });

  it("a redundant request is a snapshot-identity no-op", () => {
    const store = createProjectionStore<number>();
    store.dispatch({
      type: "request",
    });
    const pending = store.snapshot();
    store.dispatch({
      type: "request",
    });
    expect(store.snapshot()).toBe(pending);
  });

  it("re-fulfilling replaces the data and notifies", () => {
    const store = createProjectionStore<number>();
    let notifications = 0;
    store.subscribe(() => {
      notifications += 1;
    });
    store.dispatch({
      type: "fulfill",
      data: 1,
    });
    store.dispatch({
      type: "fulfill",
      data: 2,
    });
    const snap = store.snapshot() as Extract<
      Projection<number>,
      {
        kind: "loaded";
      }
    >;
    expect(snap.data).toBe(2);
    expect(notifications).toBe(2);
  });
});

// The per-entity autosave queue invariants (PR-02): serialization with
// latest-value coalescing, no settling while newer work is queued,
// error recovery, deleted-entity blocking, and generation invalidation
// before workflow deletion.

import { describe, expect, it } from "vitest";

import { createAutosave, type SaveStatus } from "./autosave";

function deferred() {
  let resolve!: () => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<void>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

/** Lets settled promise callbacks run. */
function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function harness() {
  const statuses: Array<{ key: string; status: SaveStatus }> = [];
  const autosave = createAutosave((key, status) => {
    statuses.push({ key, status });
  });
  const last = () => statuses[statuses.length - 1];
  return { autosave, statuses, last };
}

describe("autosave queue", () => {
  it("serializes per key and coalesces queued edits to the latest", async () => {
    const { autosave, last } = harness();
    const first = deferred();
    const calls: string[] = [];
    autosave.schedule("step:1", () => {
      calls.push("a");
      return first.promise;
    });
    autosave.schedule("step:1", async () => {
      calls.push("b");
    });
    autosave.schedule("step:1", async () => {
      calls.push("c");
    });
    expect(calls, "only one request in flight").toEqual(["a"]);

    first.resolve();
    await flush();
    // The middle edit was replaced by the latest one.
    expect(calls).toEqual(["a", "c"]);
    expect(last()?.status).toEqual({ state: "idle" });
  });

  it("does not settle while a newer edit is still queued", async () => {
    const { autosave, last } = harness();
    const first = deferred();
    const second = deferred();
    autosave.schedule("step:1", () => first.promise);
    autosave.schedule("step:1", () => second.promise);

    first.resolve();
    await flush();
    // The older completion launched the newer save; the key is still
    // saving, so a stale "idle" can never mask a pending edit.
    expect(last()?.status).toEqual({ state: "saving" });

    second.resolve();
    await flush();
    expect(last()?.status).toEqual({ state: "idle" });
  });

  it("surfaces a failure and retries the failed save", async () => {
    const { autosave, last } = harness();
    const first = deferred();
    let calls = 0;
    autosave.schedule("step:1", () => {
      calls += 1;
      return calls === 1 ? first.promise : Promise.resolve();
    });
    first.reject(new Error("disk full"));
    await flush();
    const failed = last()?.status;
    expect(failed?.state).toBe("error");
    expect(failed?.state === "error" && failed.message).toContain("disk full");

    autosave.retry("step:1");
    await flush();
    expect(calls).toBe(2);
    expect(last()?.status).toEqual({ state: "idle" });
  });

  it("blocks a deleted entity: queued work is dropped and the in-flight completion is ignored", async () => {
    const { autosave, statuses } = harness();
    const first = deferred();
    const calls: string[] = [];
    autosave.schedule("step:1", () => {
      calls.push("in-flight");
      return first.promise;
    });
    autosave.schedule("step:1", async () => {
      calls.push("queued");
    });
    autosave.block("step:1");
    const before = statuses.length;

    first.resolve();
    await flush();
    expect(calls, "the queued update never fires").toEqual(["in-flight"]);
    expect(statuses.length, "no status settles after blocking").toBe(before);

    // Later schedules for the blocked entity are ignored too.
    autosave.schedule("step:1", async () => {
      calls.push("late");
    });
    await flush();
    expect(calls).toEqual(["in-flight"]);
  });

  it("invalidate drops queued work and ignores every stale completion", async () => {
    const { autosave, statuses } = harness();
    const ok = deferred();
    const failing = deferred();
    const calls: string[] = [];
    autosave.schedule("step:1", () => {
      calls.push("one");
      return ok.promise;
    });
    autosave.schedule("step:1", async () => {
      calls.push("one-queued");
    });
    autosave.schedule("workflow", () => {
      calls.push("rename");
      return failing.promise;
    });
    autosave.invalidate();
    const before = statuses.length;

    ok.resolve();
    failing.reject(new Error("gone"));
    await flush();
    expect(calls).toEqual(["one", "rename"]);
    expect(
      statuses.length,
      "stale completions neither settle nor error after invalidation",
    ).toBe(before);
  });
});

// Per-entity serialized autosave queues for the detail view (PR-02).
//
// Invariants:
// - At most one request is in flight per entity key; newer edits for the
//   same key coalesce into one queued save that always carries the
//   latest value.
// - An older completed request never overwrites a newer edit: the local
//   state stays the authority, and the key's status settles to "idle"
//   only when its queue is drained.
// - A blocked key (deleted step) drops its queued save and ignores the
//   completion of any request still in flight.
// - `invalidate()` (called before workflow deletion) bumps the
//   generation: every queued save is dropped and every in-flight
//   completion — success or failure — is ignored. The key itself stays
//   serialized: a save scheduled AFTER the invalidation queues behind a
//   still-in-flight stale request and launches only when it settles, so
//   a replayed newer value can never race the orphaned older write.
// - `discard(key)` drops the key's queued and failed work (without
//   blocking future schedules) and settles a non-in-flight key to idle;
//   the detail view uses it when the local value becomes invalid.

export type SaveStatus =
  | { state: "idle" }
  | { state: "saving" }
  | { state: "error"; message: string };

/** A save operation carrying the latest local value for its entity. */
type SaveOp = () => Promise<void>;

interface Entry {
  inFlight: boolean;
  /** The coalesced next save; always the latest scheduled one. */
  queued: SaveOp | null;
  /** The failed save kept for an explicit retry. */
  failed: SaveOp | null;
  blocked: boolean;
}

export interface Autosave {
  /** Schedules the latest save for the key, replacing any queued one. */
  schedule(key: string, save: SaveOp): void;
  /** Re-runs the key's failed save, if any. */
  retry(key: string): void;
  /** Drops the key's pending work; later completions are ignored. */
  block(key: string): void;
  /** Drops the key's queued and failed work without blocking the key. */
  discard(key: string): void;
  /** Drops everything and ignores all in-flight completions. */
  invalidate(): void;
}

export function createAutosave(
  onStatus: (key: string, status: SaveStatus) => void,
): Autosave {
  const entries = new Map<string, Entry>();
  let generation = 0;

  function entryFor(key: string): Entry {
    let entry = entries.get(key);
    if (entry === undefined) {
      entry = { inFlight: false, queued: null, failed: null, blocked: false };
      entries.set(key, entry);
    }
    return entry;
  }

  function launch(key: string, entry: Entry, save: SaveOp): void {
    entry.inFlight = true;
    entry.failed = null;
    const launchedIn = generation;
    onStatus(key, { state: "saving" });
    save().then(
      () => settle(key, entry, launchedIn, save, null),
      (caught: unknown) => settle(key, entry, launchedIn, save, String(caught)),
    );
  }

  function settle(
    key: string,
    entry: Entry,
    launchedIn: number,
    save: SaveOp,
    failure: string | null,
  ): void {
    // A stale completion from before `invalidate()` settles no status
    // and installs no retry, but it must still release the key and
    // launch work queued AFTER the invalidation: a replayed save waits
    // here instead of racing the orphaned in-flight write.
    if (launchedIn !== generation) {
      entry.inFlight = false;
      if (entry.blocked) {
        return;
      }
      const replay = entry.queued;
      if (replay !== null) {
        entry.queued = null;
        launch(key, entry, replay);
      }
      return;
    }
    entry.inFlight = false;
    if (entry.blocked) {
      return;
    }
    const next = entry.queued;
    if (next !== null) {
      // A newer edit supersedes this outcome either way: send it.
      entry.queued = null;
      launch(key, entry, next);
      return;
    }
    if (failure !== null) {
      entry.failed = save;
      onStatus(key, { state: "error", message: failure });
      return;
    }
    onStatus(key, { state: "idle" });
  }

  return {
    schedule(key, save) {
      const entry = entryFor(key);
      if (entry.blocked) {
        return;
      }
      if (entry.inFlight) {
        entry.queued = save;
        return;
      }
      launch(key, entry, save);
    },
    retry(key) {
      const entry = entries.get(key);
      if (entry === undefined || entry.blocked || entry.inFlight) {
        return;
      }
      const failed = entry.failed;
      if (failed !== null) {
        launch(key, entry, failed);
      }
    },
    block(key) {
      const entry = entryFor(key);
      entry.blocked = true;
      entry.queued = null;
      entry.failed = null;
    },
    discard(key) {
      const entry = entries.get(key);
      if (entry === undefined || entry.blocked) {
        return;
      }
      entry.queued = null;
      entry.failed = null;
      if (!entry.inFlight) {
        onStatus(key, { state: "idle" });
      }
    },
    invalidate() {
      // Entries stay in the map so their in-flight flags keep the keys
      // serialized; only their pending work is dropped.
      generation += 1;
      for (const entry of entries.values()) {
        entry.queued = null;
        entry.failed = null;
      }
    },
  };
}

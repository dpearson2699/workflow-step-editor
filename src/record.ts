// The record-flow session controller (DEC-002, DEC-009, AC-004): one
// object per Record click that owns the live channel subscription and
// the start/stop/terminal ordering, outside the React tree so renders
// and StrictMode effect replays can never double-start a recording.
//
// Ordering invariants (the plan's edge cases):
// - Rows stream in ordered and deduplicated by step id.
// - Steps arriving before the start promise resolves are kept.
// - A terminal received while start is pending wins over plain
//   recording mode once start settles.
// - A Stop click during startup latches and issues once start resolves.
// - Stop issues at most one stop command; a second click is a no-op.
// - Draft entry is driven by the terminal envelope or, because channel
//   delivery is documented best-effort, by a successful stop result —
//   whichever settles first wins, so both orders of stop-command
//   resolution versus terminal-envelope arrival converge and a lost
//   terminal can never strand the live view. A synthesized entry stays
//   supersedable by the genuine channel terminal exactly once, so a
//   fail-stop that resolved the stop command Ok keeps its banner.
// - A stop rejected with `no active recording` after start published
//   the workflow id means an autonomous fail-stop finished and its
//   `failed` terminal was lost; the session resolves through DEC-009's
//   load-decides-reviewability path instead of re-arming a dead Stop.
// - A disposed session ignores every late channel message and promise
//   settlement (stale-session suppression).
//
// A `failed` terminal decides reviewability by loading the workflow
// (DEC-009): when it still loads, the flow enters draft review behind a
// failure banner; when it does not, the flow exits to the landing page
// with the error.

import type { ApiClient, Classification, LiveEnvelope } from "./api/client";

/** One live step row: index is the array position. */
export interface LiveRow {
  id: string;
  classification: Classification;
  title: string;
  /** RFC 3339 event timestamp from the envelope (DEC-009). */
  ts: string;
}

export type RecordState =
  | {
      phase: "live";
      rows: readonly LiveRow[];
      /** True from a Stop click (latched or issued) until it settles. */
      stopPending: boolean;
      /** A failed stop command; Stop is re-armed for another attempt. */
      stopError: string | null;
    }
  | { phase: "draft"; workflowId: string; failure: string | null }
  | { phase: "exited"; error: string | null };

export interface RecordSession {
  getState(): RecordState;
  /** Notifies on every state change; returns the unsubscribe. */
  subscribe(listener: () => void): () => void;
  /** The Stop Recording banner action. */
  stop(): void;
  /** Ignore every further channel message and promise settlement. */
  dispose(): void;
}

type Terminal = Extract<LiveEnvelope, { type: "stopped" | "failed" }>;

/** Starts a recording immediately and returns its session controller. */
export function startRecordSession(api: ApiClient): RecordSession {
  let disposed = false;
  let startSettled = false;
  let stopRequested = false;
  let terminalHandled = false;
  // True while the handled terminal was synthesized (from a stop result
  // or a `no active recording` rejection) rather than received on the
  // channel: the genuine terminal still supersedes it, so a fail-stop
  // queued ahead of the stop keeps its failure banner.
  let syntheticTerminal = false;
  // The workflow id published by the resolved start command; resolves a
  // fail-stopped session whose `failed` terminal was lost.
  let startedWorkflowId: string | null = null;
  let pendingTerminal: Terminal | null = null;
  const seenStepIds = new Set<string>();
  let rows: LiveRow[] = [];

  let state: RecordState = {
    phase: "live",
    rows,
    stopPending: false,
    stopError: null,
  };
  const listeners = new Set<() => void>();

  function setState(next: RecordState): void {
    state = next;
    for (const listener of listeners) {
      listener();
    }
  }

  function liveState(stopError: string | null = null): RecordState {
    return { phase: "live", rows, stopPending: stopRequested, stopError };
  }

  function handleTerminal(terminal: Terminal): void {
    terminalHandled = true;
    if (terminal.type === "stopped") {
      setState({
        phase: "draft",
        workflowId: terminal.workflow_id,
        failure: null,
      });
      return;
    }
    // Reviewability is decided by loading the workflow, never by extra
    // envelope fields (DEC-009).
    api.getWorkflow(terminal.workflow_id).then(
      () => {
        if (disposed) {
          return;
        }
        setState({
          phase: "draft",
          workflowId: terminal.workflow_id,
          failure: terminal.error,
        });
      },
      (caught: unknown) => {
        if (disposed) {
          return;
        }
        setState({
          phase: "exited",
          error:
            `Recording failed: ${terminal.error}. ` +
            `The captured workflow could not be loaded: ${String(caught)}`,
        });
      },
    );
  }

  function issueStop(): void {
    setState(liveState());
    api.stopRecording().then(
      (workflowId: string) => {
        if (disposed || terminalHandled) {
          return;
        }
        // The terminal envelope normally drives draft entry, but the
        // channel is documented best-effort while the stop command
        // resolves only after finalization saved the manifest. Its
        // workflow id is the reliable fallback into draft review; a
        // terminal that already landed keeps precedence. The synthesis
        // stays supersedable: a fail-stop queued ahead of this stop can
        // resolve it Ok while the worker emits `failed`, and that
        // genuine terminal must still deliver its failure banner.
        syntheticTerminal = true;
        handleTerminal({ type: "stopped", workflow_id: workflowId });
      },
      (caught: unknown) => {
        if (disposed || terminalHandled) {
          return;
        }
        const message = String(caught);
        if (message.includes("no active recording") && startedWorkflowId !== null) {
          // The recording ended behind this session: an autonomous
          // fail-stop finished (the backend is Idle again) and its
          // `failed` terminal was lost on the best-effort channel.
          // Resolve the session through DEC-009's load-decides-
          // reviewability path instead of re-arming a dead Stop.
          syntheticTerminal = true;
          handleTerminal({
            type: "failed",
            workflow_id: startedWorkflowId,
            error: "the recording ended unexpectedly before this stop",
          });
          return;
        }
        // Re-arm Stop: the recording may still be active. When the
        // rejection raced a fail-stop, the imminent terminal envelope
        // supersedes this state.
        stopRequested = false;
        setState(liveState(message));
      },
    );
  }

  function onEnvelope(envelope: LiveEnvelope): void {
    if (disposed) {
      return;
    }
    if (terminalHandled) {
      // The genuine channel terminal supersedes a synthesized one
      // exactly once, so a trailing `failed` envelope still lands its
      // failure banner over a fallback-entered draft.
      if (envelope.type !== "step" && syntheticTerminal) {
        syntheticTerminal = false;
        handleTerminal(envelope);
      }
      return;
    }
    if (envelope.type === "step") {
      if (seenStepIds.has(envelope.step.id)) {
        return;
      }
      seenStepIds.add(envelope.step.id);
      rows = [
        ...rows,
        {
          id: envelope.step.id,
          classification: envelope.step.classification,
          title: envelope.step.title,
          ts: envelope.ts,
        },
      ];
      setState(liveState(state.phase === "live" ? state.stopError : null));
      return;
    }
    if (!startSettled) {
      // Processed once start settles: the terminal wins over recording
      // mode, and a start rejection defers to it.
      pendingTerminal = envelope;
      return;
    }
    handleTerminal(envelope);
  }

  api.startRecording(onEnvelope).then(
    (workflowId: string) => {
      startSettled = true;
      startedWorkflowId = workflowId;
      if (disposed || terminalHandled) {
        return;
      }
      if (pendingTerminal !== null) {
        handleTerminal(pendingTerminal);
        return;
      }
      if (stopRequested) {
        // The latched startup Stop click issues now.
        issueStop();
      }
    },
    (caught: unknown) => {
      startSettled = true;
      if (disposed || terminalHandled) {
        return;
      }
      if (pendingTerminal !== null) {
        // The session ran far enough to finalize; its terminal carries
        // the more complete outcome.
        handleTerminal(pendingTerminal);
        return;
      }
      setState({
        phase: "exited",
        error: `Could not start recording: ${String(caught)}`,
      });
    },
  );

  return {
    getState() {
      return state;
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    stop() {
      if (
        disposed ||
        terminalHandled ||
        stopRequested ||
        state.phase !== "live"
      ) {
        return;
      }
      stopRequested = true;
      if (!startSettled) {
        // Latched: issued from the start continuation above.
        setState(liveState());
        return;
      }
      issueStop();
    },
    dispose() {
      disposed = true;
    },
  };
}

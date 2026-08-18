// Component tests for the record flow (AC-004, DEC-002, DEC-009),
// driven through the product shell: start/stop gating, startup Stop
// latching, early step and terminal arrival while start is pending,
// stale-session suppression, both orders of stop-command resolution
// versus terminal-envelope arrival, the failed-recording draft path,
// and the load-failure landing error path.

import { describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";

import type {
  ApiClient,
  LiveEnvelope,
  LoadedWorkflow,
} from "./api/client";
import { AppShell } from "./App";

function deferred<T = void>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const WORKFLOW_ID = "2026-08-16-223105-9f3a";
/** The backend's default timestamp name already in the manifest. */
const DEFAULT_NAME = "2026-08-16 22:31:05";

// Coherent fixture as of 2026-08-16T22:31:08Z: a freshly stopped
// two-step recording still under its default manifest name.
function draftWorkflow(): LoadedWorkflow {
  return {
    manifest: {
      schema_version: 1,
      id: WORKFLOW_ID,
      name: DEFAULT_NAME,
      created_at: "2026-08-16T22:31:05Z",
      steps: [
        {
          id: "step_0001",
          event_ids: ["evt_0001"],
          classification: "click",
          title: 'Click "OK" — TextEdit',
          description: "",
        },
        {
          id: "step_0002",
          event_ids: ["evt_0002"],
          classification: "type",
          title: "Press Cmd+S — TextEdit",
          description: "",
        },
      ],
    },
    events: [
      {
        id: "evt_0001",
        ts: "2026-08-16T22:31:05.123Z",
        kind: "click",
        display_id: 1,
        pos: { x: 512, y: 384 },
        button: "left",
        key: null,
        window: {
          app: "TextEdit",
          title: "Untitled",
          pid: 871,
          bounds: { x: 100, y: 50, w: 800, h: 600 },
        },
        element: {
          role: "AXButton",
          title: "OK",
          frame: { x: 480, y: 360, w: 80, h: 32 },
          source: "ax",
        },
      },
      {
        id: "evt_0002",
        ts: "2026-08-16T22:31:06.456Z",
        kind: "key_down",
        display_id: 1,
        pos: { x: 512, y: 384 },
        button: null,
        key: { key_code: 1, chars: "s", modifiers: ["command"] },
        window: {
          app: "TextEdit",
          title: "Untitled",
          pid: 871,
          bounds: { x: 100, y: 50, w: 800, h: 600 },
        },
        element: {
          role: "AXTextArea",
          title: null,
          frame: { x: 120, y: 80, w: 760, h: 540 },
          source: "ax",
        },
      },
    ],
  };
}

function stepEnvelope(n: number, title?: string): LiveEnvelope {
  const id = String(n).padStart(4, "0");
  return {
    type: "step",
    step: {
      id: `step_${id}`,
      event_ids: [`evt_${id}`],
      classification: "click",
      title: title ?? `Click ${n}`,
      description: "",
    },
    ts: `2026-08-16T22:31:0${n}.000Z`,
  };
}

/** A controllable recorder: captured channels, deferred commands. */
function recorder() {
  const sinks: Array<(envelope: LiveEnvelope) => void> = [];
  const starts: Array<ReturnType<typeof deferred<string>>> = [];
  const stops: Array<ReturnType<typeof deferred<string>>> = [];
  return {
    sinks,
    starts,
    stops,
    startRecording: vi.fn((onEnvelope: (envelope: LiveEnvelope) => void) => {
      sinks.push(onEnvelope);
      const gate = deferred<string>();
      starts.push(gate);
      return gate.promise;
    }),
    stopRecording: vi.fn(() => {
      const gate = deferred<string>();
      stops.push(gate);
      return gate.promise;
    }),
  };
}

function apiWith(overrides: Partial<ApiClient> = {}): ApiClient {
  return {
    checkPermissions: async () => ({
      input_monitoring: "granted",
      accessibility: "granted",
      screen_recording: "granted",
    }),
    requestPermission: async () => "granted",
    listWorkflows: async () => [],
    getWorkflow: async () => draftWorkflow(),
    revealWorkflow: async () => {},
    readScreenshot: async () => {
      throw new Error("no screenshot in this test");
    },
    updateStep: async () => {},
    deleteStep: async () => {},
    renameWorkflow: async () => {},
    deleteWorkflow: async () => {},
    startRecording: async () => WORKFLOW_ID,
    stopRecording: async () => WORKFLOW_ID,
    ...overrides,
  };
}

async function renderShell(overrides: Partial<ApiClient> = {}) {
  const api = apiWith(overrides);
  render(<AppShell api={api} />);
  // The landing settles: permissions granted, Record enabled.
  await waitFor(() => {
    const button = screen.getByRole("button", {
      name: "● Record New Workflow",
    }) as HTMLButtonElement;
    expect(button.disabled).toBe(false);
  });
  return api;
}

function recordButton(): HTMLButtonElement {
  return screen.getByRole("button", {
    name: "● Record New Workflow",
  }) as HTMLButtonElement;
}

function stopButton(): HTMLButtonElement {
  return screen.getByRole("button", {
    name: "■ Stop Recording",
  }) as HTMLButtonElement;
}

describe("record flow", () => {
  it("starts once for a double Record click and keeps steps that arrive before start resolves", async () => {
    const capture = recorder();
    await renderShell(capture);

    const record = recordButton();
    fireEvent.click(record);
    fireEvent.click(record);
    expect(capture.startRecording).toHaveBeenCalledTimes(1);

    // The Stop banner is the sole visible action in the capture view.
    expect(stopButton().disabled).toBe(false);
    expect(screen.queryByRole("button", { name: "‹ Workflows" })).toBeNull();
    expect(screen.getByText(/0 steps captured/)).toBeTruthy();

    // Early arrival: the channel outruns the start promise (DEC-009
    // rows: index, dot, title, event time).
    await act(async () => {
      capture.sinks[0](stepEnvelope(1, 'Click "OK" — TextEdit'));
    });
    expect(screen.getByText('Click "OK" — TextEdit')).toBeTruthy();
    expect(screen.getByText(/1 step captured/)).toBeTruthy();

    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });
    // Ordered and deduplicated by step id.
    await act(async () => {
      capture.sinks[0](stepEnvelope(2));
      capture.sinks[0](stepEnvelope(2));
    });
    expect(screen.getByText(/2 steps captured/)).toBeTruthy();
  });

  it("issues one stop for repeated clicks and falls back to the stop result when the terminal is lost", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    fireEvent.click(stopButton());
    expect(stopButton().disabled).toBe(true);
    fireEvent.click(stopButton());
    expect(capture.stopRecording).toHaveBeenCalledTimes(1);

    // Order A: the stop command resolves and no terminal envelope ever
    // arrives (channel delivery is best-effort). The successful stop
    // result drives draft entry.
    await act(async () => {
      capture.stops[0].resolve(WORKFLOW_ID);
    });
    expect(screen.getByText("draft")).toBeTruthy();
    // Draft review is the full detail view under the manifest's
    // default name, with Discard and Save… in the header.
    await screen.findByText(DEFAULT_NAME);
    expect(screen.getByRole("button", { name: "Discard" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Save…" })).toBeTruthy();
    expect(screen.getByText('Click "OK" — TextEdit')).toBeTruthy();

    // A terminal that trails the fallback is ignored: draft is entered
    // exactly once and no error surfaces.
    await act(async () => {
      capture.sinks[0]({ type: "stopped", workflow_id: WORKFLOW_ID });
    });
    expect(screen.getAllByText("draft")).toHaveLength(1);
  });

  it("enters draft exactly once when the terminal envelope outruns the stop result", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    fireEvent.click(stopButton());
    // Order B: the terminal envelope lands before the stop command
    // resolves.
    await act(async () => {
      capture.sinks[0]({ type: "stopped", workflow_id: WORKFLOW_ID });
    });
    expect(screen.getByText("draft")).toBeTruthy();
    await act(async () => {
      capture.stops[0].resolve(WORKFLOW_ID);
    });
    expect(screen.getAllByText("draft")).toHaveLength(1);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("latches a Stop click during startup and issues it once start resolves", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());

    fireEvent.click(stopButton());
    expect(capture.stopRecording).not.toHaveBeenCalled();
    expect(stopButton().disabled).toBe(true);

    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });
    expect(capture.stopRecording).toHaveBeenCalledTimes(1);

    await act(async () => {
      capture.sinks[0]({ type: "stopped", workflow_id: WORKFLOW_ID });
      capture.stops[0].resolve(WORKFLOW_ID);
    });
    expect(screen.getByText("draft")).toBeTruthy();
  });

  it("lets a terminal received while start is pending win and shows the failure banner over draft review", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());

    await act(async () => {
      capture.sinks[0](stepEnvelope(1));
      capture.sinks[0]({
        type: "failed",
        workflow_id: WORKFLOW_ID,
        error: "event tap disabled",
      });
    });
    // Still pending: the flow waits for start to settle.
    expect(screen.getByText(/Stop Recording/)).toBeTruthy();

    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });
    // The workflow still loads, so the failed recording lands in draft
    // review behind the error banner (DEC-009).
    await screen.findByText("draft");
    const banner = screen
      .getAllByRole("alert")
      .find((alert) =>
        alert.textContent?.includes("Recording failed and may be incomplete"),
      );
    expect(banner?.textContent).toContain("event tap disabled");
    expect(capture.stopRecording).not.toHaveBeenCalled();
  });

  it("surfaces a failed recording whose workflow no longer loads on the landing page", async () => {
    const capture = recorder();
    await renderShell({
      ...capture,
      getWorkflow: async () => {
        throw new Error("workflow not found: " + WORKFLOW_ID);
      },
    });
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    await act(async () => {
      capture.sinks[0]({
        type: "failed",
        workflow_id: WORKFLOW_ID,
        error: "event tap disabled",
      });
    });

    // Back on the landing page with the failure surfaced.
    await screen.findByRole("button", { name: "● Record New Workflow" });
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Recording failed: event tap disabled");
    expect(alert.textContent).toContain("could not be loaded");
    expect(alert.textContent).toContain("workflow not found");
  });

  it("surfaces a rejected start on the landing page", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].reject(new Error("a recording is already active"));
    });

    await screen.findByRole("button", { name: "● Record New Workflow" });
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Could not start recording",
    );
  });

  it("re-arms Stop and surfaces the error when the stop command fails without a terminal", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].reject(new Error("recording task failed: panic"));
    });
    expect(screen.getByRole("alert").textContent).toContain(
      "Could not stop the recording",
    );
    expect(stopButton().disabled).toBe(false);

    fireEvent.click(stopButton());
    expect(capture.stopRecording).toHaveBeenCalledTimes(2);
    await act(async () => {
      capture.stops[1].resolve(WORKFLOW_ID);
      capture.sinks[0]({ type: "stopped", workflow_id: WORKFLOW_ID });
    });
    expect(screen.getByText("draft")).toBeTruthy();
  });

  it("lets a trailing failed terminal supersede the stop-result draft with its failure banner", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    // A pipeline failure queued ahead of the Stop resolves the stop
    // command Ok while the worker emits `failed`; the command result
    // can beat the channel, so the fallback enters draft first.
    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].resolve(WORKFLOW_ID);
    });
    await screen.findByText("draft");
    expect(
      screen.queryByText(/Recording failed and may be incomplete/),
    ).toBeNull();

    // The genuine failed terminal supersedes the synthesized draft and
    // lands its failure banner, still exactly one draft.
    await act(async () => {
      capture.sinks[0]({
        type: "failed",
        workflow_id: WORKFLOW_ID,
        error: "tap disabled",
      });
    });
    const banner = await screen.findByText(
      /Recording failed and may be incomplete/,
    );
    expect(banner.textContent).toContain("tap disabled");
    expect(screen.getAllByText("draft")).toHaveLength(1);
  });

  it("resolves a fail-stopped session whose lost terminal left nothing to stop", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    // The pipeline fail-stopped on its own and the `failed` terminal
    // was lost on the best-effort channel: the backend is Idle, so the
    // user's Stop rejects with the defined NotRecording error. The
    // session resolves through the load-check path instead of
    // re-arming a dead Stop.
    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].reject(new Error("no active recording"));
    });
    await screen.findByText("draft");
    const banner = screen
      .getAllByRole("alert")
      .find((alert) =>
        alert.textContent?.includes("Recording failed and may be incomplete"),
      );
    expect(banner?.textContent).toContain("ended unexpectedly");
  });

  it("surfaces a late failed terminal even after the draft was already saved", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    // The stop result enters draft; the user completes the save
    // ceremony before the genuine terminal ever arrives.
    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].resolve(WORKFLOW_ID);
    });
    await screen.findByText("draft");
    fireEvent.click(screen.getByRole("button", { name: "Save…" }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => {
      expect(screen.queryByText("draft")).toBeNull();
    });

    // The trailing genuine failed terminal must still disclose the
    // incomplete recording instead of presenting it as an ordinary
    // saved workflow.
    await act(async () => {
      capture.sinks[0]({
        type: "failed",
        workflow_id: WORKFLOW_ID,
        error: "tap disabled",
      });
    });
    const banner = await screen.findByText(
      /Recording failed and may be incomplete/,
    );
    expect(banner.textContent).toContain("tap disabled");
  });

  it("resolves a finalization-failed stop through draft review without a second click", async () => {
    const capture = recorder();
    await renderShell(capture);
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    // The recording ended but the manifest save failed; the matching
    // failed terminal was lost. The defined rejection resolves the
    // session through the load-check path instead of re-arming Stop.
    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].reject(
        new Error("recording finalization failed: manifest save failed: disk full"),
      );
    });
    await screen.findByText("draft");
    const banner = screen
      .getAllByRole("alert")
      .find((alert) =>
        alert.textContent?.includes("Recording failed and may be incomplete"),
      );
    expect(banner?.textContent).toContain("manifest save failed: disk full");
    expect(capture.stopRecording).toHaveBeenCalledTimes(1);
  });

  it("ignores a stale synthetic load check once the genuine terminal superseded it", async () => {
    const capture = recorder();
    const loads: Array<ReturnType<typeof deferred<LoadedWorkflow>>> = [];
    await renderShell({
      ...capture,
      getWorkflow: () => {
        const gate = deferred<LoadedWorkflow>();
        loads.push(gate);
        return gate.promise;
      },
    });
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
    });

    // A finalization-failed stop synthesizes a failed terminal whose
    // load check stays in flight...
    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].reject(
        new Error("recording finalization failed: manifest save failed: disk full"),
      );
    });
    expect(loads).toHaveLength(1);

    // ...then the genuine failed terminal supersedes it with its own
    // load check.
    await act(async () => {
      capture.sinks[0]({
        type: "failed",
        workflow_id: WORKFLOW_ID,
        error: "tap disabled",
      });
    });
    expect(loads).toHaveLength(2);

    // The genuine check resolves into draft with the genuine error...
    await act(async () => {
      loads[1].resolve(draftWorkflow());
    });
    const banner = await screen.findByText(
      /Recording failed and may be incomplete/,
    );
    expect(banner.textContent).toContain("tap disabled");

    // ...and the stale synthetic check settling last (even as a
    // rejection) must neither overwrite the banner nor bounce the flow
    // to the landing page.
    await act(async () => {
      loads[0].reject(new Error("workflow not found"));
    });
    expect(screen.getAllByText("draft")).toHaveLength(1);
    const lateBanner = screen
      .getAllByRole("alert")
      .find((alert) =>
        alert.textContent?.includes("Recording failed and may be incomplete"),
      );
    expect(lateBanner?.textContent).toContain("tap disabled");
    expect(
      screen.queryByRole("button", { name: "● Record New Workflow" }),
    ).toBeNull();
  });

  it("ignores messages from a stale session after a new recording starts", async () => {
    const capture = recorder();
    const deleteWorkflow = vi.fn(async () => {});
    await renderShell({ ...capture, deleteWorkflow });

    // Session 1 runs to draft review, then is discarded.
    fireEvent.click(recordButton());
    await act(async () => {
      capture.starts[0].resolve(WORKFLOW_ID);
      capture.sinks[0](stepEnvelope(1));
    });
    fireEvent.click(stopButton());
    await act(async () => {
      capture.stops[0].resolve(WORKFLOW_ID);
      capture.sinks[0]({ type: "stopped", workflow_id: WORKFLOW_ID });
    });
    await screen.findByText("draft");
    fireEvent.click(screen.getByRole("button", { name: "Discard" }));
    fireEvent.click(screen.getByRole("button", { name: "Discard Recording" }));
    // The remounted landing re-checks permissions before enabling
    // Record; wait for the enable gate, not just the button's presence.
    await waitFor(() => {
      expect(recordButton().disabled).toBe(false);
    });
    expect(deleteWorkflow).toHaveBeenCalledWith(WORKFLOW_ID);

    // Session 2 starts; the stale session-1 channel keeps talking.
    fireEvent.click(recordButton());
    expect(capture.startRecording).toHaveBeenCalledTimes(2);
    await act(async () => {
      capture.starts[1].resolve("wf-2");
      capture.sinks[0](stepEnvelope(7, "Stale step"));
      capture.sinks[0]({ type: "stopped", workflow_id: WORKFLOW_ID });
    });
    // The new session is untouched: no stale row, no draft entry.
    expect(screen.queryByText("Stale step")).toBeNull();
    expect(screen.queryByText("draft")).toBeNull();
    expect(screen.getByText(/0 steps captured/)).toBeTruthy();
  });
});

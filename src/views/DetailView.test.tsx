// Component tests for the detail view (AC-003, AC-005): edit
// persistence with autosave ordering and recovery, header rename, step
// deletion with stale-update suppression, click-to-swap, the metadata
// grid for both element sources, and the Delete… confirmation flow.

import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import type {
  ApiClient,
  LoadedWorkflow,
  WorkflowEvent,
} from "../api/client";
import { formatEventTime } from "../lib/format";
import { DetailView } from "./DetailView";

// jsdom has no blob URL support; the scoped-read cache needs both.
beforeAll(() => {
  let counter = 0;
  Object.assign(URL, {
    createObjectURL: () => `blob:test-${(counter += 1)}`,
    revokeObjectURL: () => {},
  });
});
afterAll(() => {
  const url = URL as unknown as Record<string, unknown>;
  delete url.createObjectURL;
  delete url.revokeObjectURL;
});

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

function baseEvent(id: string, ts: string): WorkflowEvent {
  return {
    id,
    ts,
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
  };
}

// Coherent fixture as of 2026-08-16T22:31:08Z: a three-event recording
// in TextEdit finishing with an unresolved-window click (DEC-011).
function workflowFixture(): LoadedWorkflow {
  const clickEvent = baseEvent("evt_0001", "2026-08-16T22:31:05.123Z");
  const keyEvent: WorkflowEvent = {
    ...baseEvent("evt_0002", "2026-08-16T22:31:06.456Z"),
    kind: "key_down",
    button: null,
    key: { key_code: 1, chars: "s", modifiers: ["command"] },
    element: {
      role: "AXTextArea",
      title: null,
      frame: { x: 120, y: 80, w: 760, h: 540 },
      source: "ax",
    },
  };
  const fallbackEvent: WorkflowEvent = {
    ...baseEvent("evt_0003", "2026-08-16T22:31:07.000Z"),
    window: null,
    element: {
      role: null,
      title: null,
      frame: { x: 412, y: 284, w: 200, h: 200 },
      source: "fallback",
    },
  };
  return {
    manifest: {
      schema_version: 1,
      id: WORKFLOW_ID,
      name: "Approve invoice",
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
        {
          id: "step_0003",
          event_ids: ["evt_0003"],
          classification: "click",
          title: "Click at (512, 384)",
          description: "",
        },
      ],
    },
    events: [clickEvent, keyEvent, fallbackEvent],
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
    getWorkflow: async () => workflowFixture(),
    revealWorkflow: async () => {},
    readScreenshot: async () => {
      throw new Error("no screenshot in this test");
    },
    updateStep: async () => {},
    deleteStep: async () => {},
    renameWorkflow: async () => {},
    deleteWorkflow: async () => {},
    ...overrides,
  };
}

async function renderDetail(overrides: Partial<ApiClient> = {}, handlers?: {
  onBack?: () => void;
  onDeleted?: () => void;
}) {
  const api = apiWith(overrides);
  render(
    <DetailView
      api={api}
      workflowId={WORKFLOW_ID}
      initialName="Approve invoice"
      onBack={handlers?.onBack ?? (() => {})}
      onDeleted={handlers?.onDeleted ?? (() => {})}
    />,
  );
  await screen.findByText('Click "OK" — TextEdit');
  return api;
}

describe("detail view rendering", () => {
  it("renders the compact step list beside the selected step's triple and metadata", async () => {
    await renderDetail();

    const rows = within(
      screen.getByRole("list", { name: "Steps" }),
    ).getAllByRole("listitem");
    expect(rows.length).toBe(3);
    expect(rows[0].textContent).toContain("1");
    expect(rows[0].textContent).toContain('Click "OK" — TextEdit');
    expect(rows[0].textContent).toContain(
      formatEventTime("2026-08-16T22:31:05.123Z"),
    );
    expect(rows[1].textContent).toContain("Press Cmd+S — TextEdit");

    // All three screenshots stay visible as labeled placeholders when
    // no image is available.
    const pane = screen.getByRole("region", { name: "Step detail" });
    expect(within(pane).getByText("No full screenshot")).toBeTruthy();
    expect(within(pane).getByText("No window image")).toBeTruthy();
    expect(within(pane).getByText("No element image")).toBeTruthy();

    // The metadata grid for the selected (first) step.
    expect(within(pane).getByText("TextEdit — Untitled")).toBeTruthy();
    expect(within(pane).getByText("(512, 384)")).toBeTruthy();
    expect(within(pane).getByText('AXButton "OK" · ax')).toBeTruthy();
    expect(
      within(pane).getByText(formatEventTime("2026-08-16T22:31:05.123Z")),
    ).toBeTruthy();
  });

  it("swaps the large screenshot when a labeled thumbnail is clicked", async () => {
    await renderDetail({
      readScreenshot: async () => new Uint8Array([137, 80, 78, 71]),
    });

    const big = await screen.findByAltText("full screenshot");
    expect(big.getAttribute("src")).toContain("blob:");

    fireEvent.click(
      screen.getByRole("button", { name: "Show window screenshot" }),
    );
    expect(await screen.findByAltText("window screenshot")).toBeTruthy();
    expect(screen.queryByAltText("full screenshot")).toBeNull();

    fireEvent.click(
      screen.getByRole("button", { name: "Show element screenshot" }),
    );
    expect(await screen.findByAltText("element screenshot")).toBeTruthy();
  });

  it("retries a transiently failed screenshot read on reselection", async () => {
    let fail = true;
    const readScreenshot = vi.fn(async () => {
      if (fail) {
        throw new Error("transient read failure");
      }
      return new Uint8Array([1]);
    });
    await renderDetail({ readScreenshot });
    await waitFor(() => expect(readScreenshot).toHaveBeenCalledTimes(3));
    expect(screen.getByText("No full screenshot")).toBeTruthy();

    // The failed keys were un-cached: selecting another step and
    // returning retries the reads instead of pinning the placeholder.
    fail = false;
    fireEvent.click(screen.getByText("Press Cmd+S — TextEdit"));
    fireEvent.click(screen.getByText('Click "OK" — TextEdit'));
    await waitFor(() =>
      expect(screen.queryByText("No full screenshot")).toBeNull(),
    );
    expect(screen.getByAltText("full screenshot")).toBeTruthy();
  });

  it("renders the metadata grid for both element sources", async () => {
    await renderDetail();
    const pane = screen.getByRole("region", { name: "Step detail" });

    // The key step: ax source without an element title, chord key.
    fireEvent.click(screen.getByText("Press Cmd+S — TextEdit"));
    expect(within(pane).getByText("Cmd+s")).toBeTruthy();
    expect(within(pane).getByText("AXTextArea · ax")).toBeTruthy();

    // The DEC-011 fallback step: no window, no role, fallback source.
    fireEvent.click(screen.getByText("Click at (512, 384)"));
    expect(within(pane).getByText("— · fallback")).toBeTruthy();
    const definitions = within(pane)
      .getAllByRole("definition")
      .map((node) => node.textContent);
    expect(definitions).toContain("—");
  });
});

describe("edit persistence", () => {
  it("autosaves the latest full trio for a step edit and settles to saved", async () => {
    const update = deferred();
    const updateStep = vi.fn(() => update.promise);
    await renderDetail({ updateStep });

    fireEvent.change(screen.getByRole("textbox", { name: "Step title" }), {
      target: { value: "Approve the invoice" },
    });
    expect(updateStep).toHaveBeenCalledWith(WORKFLOW_ID, "step_0001", {
      title: "Approve the invoice",
      description: "",
      classification: "click",
    });
    expect(screen.getByRole("status").textContent).toContain("Saving…");

    await act(async () => {
      update.resolve();
    });
    await waitFor(() => {
      expect(screen.queryByRole("status")).toBeNull();
    });
    const title = screen.getByRole("textbox", {
      name: "Step title",
    }) as HTMLInputElement;
    expect(title.value).toBe("Approve the invoice");
  });

  it("persists a classification change through the four-value dropdown", async () => {
    const updateStep = vi.fn(async () => {});
    await renderDetail({ updateStep });

    const dropdown = screen.getByRole("combobox", {
      name: "Classification",
    }) as HTMLSelectElement;
    expect(
      Array.from(dropdown.options).map((option) => option.value),
    ).toEqual(["click", "type", "wait", "assert"]);

    fireEvent.change(dropdown, { target: { value: "assert" } });
    expect(updateStep).toHaveBeenCalledWith(WORKFLOW_ID, "step_0001", {
      title: 'Click "OK" — TextEdit',
      description: "",
      classification: "assert",
    });
  });

  it("never lets an older completed request overwrite a newer edit", async () => {
    const first = deferred();
    const second = deferred();
    const updateStep = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    await renderDetail({ updateStep });

    const title = screen.getByRole("textbox", { name: "Step title" });
    fireEvent.change(title, { target: { value: "One" } });
    fireEvent.change(title, { target: { value: "One two" } });
    expect(updateStep).toHaveBeenCalledTimes(1);

    // The older request completes while a newer edit is queued: the
    // input keeps the newer value and the queue sends it next.
    await act(async () => {
      first.resolve();
    });
    await waitFor(() => {
      expect(updateStep).toHaveBeenCalledTimes(2);
    });
    expect(updateStep).toHaveBeenLastCalledWith(WORKFLOW_ID, "step_0001", {
      title: "One two",
      description: "",
      classification: "click",
    });
    expect((title as HTMLInputElement).value).toBe("One two");
    expect(screen.getByRole("status").textContent).toContain("Saving…");

    await act(async () => {
      second.resolve();
    });
    await waitFor(() => {
      expect(screen.queryByRole("status")).toBeNull();
    });
  });

  it("surfaces a failed autosave and recovers through Retry", async () => {
    const updateStep = vi
      .fn()
      .mockRejectedValueOnce(new Error("storage error"))
      .mockResolvedValueOnce(undefined);
    await renderDetail({ updateStep });

    fireEvent.change(
      screen.getByRole("textbox", { name: "Step description" }),
      { target: { value: "double-check the total" } },
    );
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Save failed");
    expect(alert.textContent).toContain("storage error");

    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() => {
      expect(screen.queryByRole("alert")).toBeNull();
    });
    expect(updateStep).toHaveBeenCalledTimes(2);
    expect(updateStep).toHaveBeenLastCalledWith(WORKFLOW_ID, "step_0001", {
      title: 'Click "OK" — TextEdit',
      description: "double-check the total",
      classification: "click",
    });
  });
});

describe("header rename", () => {
  it("persists the trimmed name through rename_workflow", async () => {
    const renameWorkflow = vi.fn(async () => {});
    await renderDetail({ renameWorkflow });

    fireEvent.change(screen.getByRole("textbox", { name: "Workflow name" }), {
      target: { value: "  Approve vendor invoice  " },
    });
    expect(renameWorkflow).toHaveBeenCalledWith(
      WORKFLOW_ID,
      "Approve vendor invoice",
    );
    await waitFor(() => {
      expect(screen.queryByRole("status")).toBeNull();
    });
  });

  it("clears a stale rename retry when the name becomes invalid", async () => {
    const renameWorkflow = vi
      .fn<(id: string, name: string) => Promise<void>>()
      .mockRejectedValueOnce(new Error("rename failed"))
      .mockResolvedValue(undefined);
    await renderDetail({ renameWorkflow });

    const input = screen.getByRole("textbox", { name: "Workflow name" });
    fireEvent.change(input, { target: { value: "Doomed name" } });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy(),
    );

    // Clearing the name shows the validation error and removes the
    // stale Retry, so the previous value can never be resent over a
    // blank input.
    fireEvent.change(input, { target: { value: "" } });
    expect(screen.getByText("Name cannot be empty")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Retry" })).toBeNull();
    expect(renameWorkflow).toHaveBeenCalledTimes(1);

    // A valid value schedules a fresh save as usual.
    fireEvent.change(input, { target: { value: "Recovered name" } });
    await waitFor(() =>
      expect(renameWorkflow).toHaveBeenLastCalledWith(
        WORKFLOW_ID,
        "Recovered name",
      ),
    );
  });

  it("keeps a name typed while the initial load is in flight", async () => {
    const load = deferred<LoadedWorkflow>();
    const renameWorkflow = vi.fn(async () => {});
    const api = apiWith({ getWorkflow: () => load.promise, renameWorkflow });
    render(
      <DetailView
        api={api}
        workflowId={WORKFLOW_ID}
        initialName="Approve invoice"
        onBack={() => {}}
        onDeleted={() => {}}
      />,
    );

    const input = screen.getByRole("textbox", {
      name: "Workflow name",
    }) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Renamed early" } });

    await act(async () => {
      load.resolve(workflowFixture());
    });
    // The older manifest name must not clobber the newer local edit.
    expect(input.value).toBe("Renamed early");
    expect(renameWorkflow).toHaveBeenCalledWith(WORKFLOW_ID, "Renamed early");
    // The rest of the load still landed.
    expect(screen.getByText('Click "OK" — TextEdit')).toBeTruthy();
  });

  it("surfaces a rename failure and blocks an empty name locally", async () => {
    const renameWorkflow = vi
      .fn()
      .mockRejectedValue(new Error("storage error"));
    await renderDetail({ renameWorkflow });

    const name = screen.getByRole("textbox", { name: "Workflow name" });
    fireEvent.change(name, { target: { value: "New name" } });
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Save failed");

    fireEvent.change(name, { target: { value: "   " } });
    expect(screen.getByText("Name cannot be empty")).toBeTruthy();
    // The empty value never reaches the backend.
    expect(renameWorkflow).toHaveBeenCalledTimes(1);
  });
});

describe("step deletion", () => {
  it("removes the selected step only after backend success and moves the selection", async () => {
    const deletion = deferred();
    const deleteStep = vi.fn(() => deletion.promise);
    await renderDetail({ deleteStep });

    fireEvent.click(screen.getByRole("button", { name: "Delete step 1" }));
    expect(deleteStep).toHaveBeenCalledWith(WORKFLOW_ID, "step_0001");
    // The row stays until the backend confirms.
    expect(screen.getByText('Click "OK" — TextEdit')).toBeTruthy();

    await act(async () => {
      deletion.resolve();
    });
    await waitFor(() => {
      expect(screen.queryByText('Click "OK" — TextEdit')).toBeNull();
    });
    // The next step is selected; its events stay resolvable by id.
    const title = screen.getByRole("textbox", {
      name: "Step title",
    }) as HTMLInputElement;
    expect(title.value).toBe("Press Cmd+S — TextEdit");
    const pane = screen.getByRole("region", { name: "Step detail" });
    expect(within(pane).getByText("Cmd+s")).toBeTruthy();
  });

  it("surfaces a failed step deletion and keeps the row", async () => {
    const deleteStep = vi.fn(async () => {
      throw new Error("storage error");
    });
    await renderDetail({ deleteStep });

    fireEvent.click(screen.getByRole("button", { name: "Delete step 2" }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Could not delete step");
    expect(screen.getByText("Press Cmd+S — TextEdit")).toBeTruthy();
  });

  it("keeps a selection made while a step deletion is in flight", async () => {
    const removal = deferred();
    const deleteStep = vi.fn(() => removal.promise);
    await renderDetail({ deleteStep });

    // step_0001 is selected; delete it, then select step_0003 before
    // the backend responds.
    fireEvent.click(screen.getByRole("button", { name: "Delete step 1" }));
    fireEvent.click(screen.getByText("Click at (512, 384)"));
    await act(async () => {
      removal.resolve();
    });

    // The newer selection survives the completed deletion; the pane is
    // not stranded on the removed step.
    const title = screen.getByRole("textbox", {
      name: "Step title",
    }) as HTMLInputElement;
    expect(title.value).toBe("Click at (512, 384)");
  });

  it("ignores a second delete click for the same step while one is in flight", async () => {
    const removal = deferred();
    const deleteStep = vi.fn(() => removal.promise);
    await renderDetail({ deleteStep });

    const control = screen.getByRole("button", { name: "Delete step 1" });
    fireEvent.click(control);
    fireEvent.click(control);
    await act(async () => {
      removal.resolve();
    });

    expect(deleteStep).toHaveBeenCalledTimes(1);
    expect(screen.queryByText(/Could not delete step/)).toBeNull();
  });

  it("blocks stale queued updates for a deleted step", async () => {
    const firstUpdate = deferred();
    const updateStep = vi.fn(() => firstUpdate.promise);
    await renderDetail({ updateStep });

    const title = screen.getByRole("textbox", { name: "Step title" });
    fireEvent.change(title, { target: { value: "One" } });
    fireEvent.change(title, { target: { value: "One two" } });
    expect(updateStep).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: "Delete step 1" }));
    await waitFor(() => {
      expect(screen.queryByText("One two")).toBeNull();
    });

    // The in-flight completion for the deleted step must not launch the
    // queued update.
    await act(async () => {
      firstUpdate.resolve();
    });
    expect(updateStep).toHaveBeenCalledTimes(1);
  });
});

describe("workflow deletion", () => {
  it("confirms destructively with Cancel as the default and names the keystroke data", async () => {
    const deleteWorkflow = vi.fn(async () => {});
    await renderDetail({ deleteWorkflow });

    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("keystroke data");
    expect(dialog.textContent).toContain("cannot be undone");

    const cancel = within(dialog).getByRole("button", { name: "Cancel" });
    expect(document.activeElement).toBe(cancel);

    fireEvent.click(cancel);
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(deleteWorkflow).not.toHaveBeenCalled();
  });

  it("navigates only after backend success and ignores stale autosave completions", async () => {
    const staleUpdate = deferred();
    const removal = deferred();
    const updateStep = vi.fn(() => staleUpdate.promise);
    const deleteWorkflow = vi.fn(() => removal.promise);
    const onDeleted = vi.fn();
    await renderDetail({ updateStep, deleteWorkflow }, { onDeleted });

    // An edit is still in flight when the deletion is confirmed.
    fireEvent.change(screen.getByRole("textbox", { name: "Step title" }), {
      target: { value: "Mid-flight edit" },
    });

    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Workflow" }));
    expect(deleteWorkflow).toHaveBeenCalledWith(WORKFLOW_ID);
    expect(onDeleted).not.toHaveBeenCalled();

    await act(async () => {
      removal.resolve();
    });
    expect(onDeleted).toHaveBeenCalledTimes(1);

    // The generation was invalidated before deletion: the stale
    // completion neither errors nor sends anything further.
    await act(async () => {
      staleUpdate.reject(new Error("workflow not found"));
    });
    expect(updateStep).toHaveBeenCalledTimes(1);
    expect(screen.queryByText(/workflow not found/)).toBeNull();
  });

  it("ignores Escape while the deletion runs and still surfaces its failure", async () => {
    const removal = deferred();
    const deleteWorkflow = vi.fn(() => removal.promise);
    await renderDetail({ deleteWorkflow });

    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Workflow" }));
    const dialog = screen.getByRole("alertdialog");
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(screen.getByRole("alertdialog")).toBeTruthy();

    await act(async () => {
      removal.reject(
        new Error("storage error: could not access the workflow data"),
      );
    });
    expect(
      within(screen.getByRole("alertdialog")).getByRole("alert").textContent,
    ).toContain("could not access the workflow data");

    // Once the deletion settled, Escape cancels again.
    fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("re-sends dropped autosaves when the deletion fails", async () => {
    const firstSave = deferred();
    const updateStep = vi.fn(() => firstSave.promise);
    const removal = deferred();
    const deleteWorkflow = vi.fn(() => removal.promise);
    await renderDetail({ updateStep, deleteWorkflow });

    const title = screen.getByRole("textbox", { name: "Step title" });
    fireEvent.change(title, { target: { value: "First" } });
    fireEvent.change(title, { target: { value: "Final title" } });

    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Workflow" }));
    await act(async () => {
      removal.reject(new Error("workflow is recording or stopping"));
    });

    // The replay is scheduled but must wait for the orphaned in-flight
    // request: launching concurrently would leave the backend write
    // order unspecified.
    expect(updateStep).toHaveBeenCalledTimes(1);
    await act(async () => {
      firstSave.resolve();
    });
    // The stale settle released the key and launched the replay with
    // the latest local trio.
    expect(updateStep).toHaveBeenCalledTimes(2);
    expect(updateStep).toHaveBeenLastCalledWith(WORKFLOW_ID, "step_0001", {
      title: "Final title",
      description: "",
      classification: "click",
    });
    await act(async () => {});
    // No indicator sticks on "Saving…" after the re-sent save settles.
    expect(screen.queryByText("Saving…")).toBeNull();
  });

  it("keeps the workflow and surfaces the error when deletion fails", async () => {
    const deleteWorkflow = vi.fn(async () => {
      throw new Error("storage error: could not access the workflow data");
    });
    const onDeleted = vi.fn();
    await renderDetail({ deleteWorkflow }, { onDeleted });

    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Workflow" }));

    const dialog = screen.getByRole("alertdialog");
    await waitFor(() => {
      expect(
        within(dialog).getByRole("alert").textContent,
      ).toContain("could not access the workflow data");
    });
    expect(onDeleted).not.toHaveBeenCalled();
    expect(screen.getByText('Click "OK" — TextEdit')).toBeTruthy();
  });
});

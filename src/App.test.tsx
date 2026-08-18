// The landing container keeps mount-time load errors visible across
// later user actions: a permission request must not clear the
// workflow-load error while the list is still missing.

import { describe, expect, it } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";

import type { ApiClient, PermissionReport } from "./api/client";
import { LandingContainer } from "./App";

const oneMissing: PermissionReport = {
  input_monitoring: "not_requested",
  accessibility: "granted",
  screen_recording: "granted",
};

function apiWith(overrides: Partial<ApiClient>): ApiClient {
  return {
    checkPermissions: async () => oneMissing,
    requestPermission: async () => "granted",
    listWorkflows: async () => [],
    revealWorkflow: async () => {},
    readScreenshot: async () => new Uint8Array(),
    ...overrides,
  };
}

describe("landing container error channels", () => {
  it("keeps the workflow-load error visible after a permission request", async () => {
    let granted = false;
    const api = apiWith({
      listWorkflows: async () => {
        throw new Error("store unavailable");
      },
      requestPermission: async () => {
        granted = true;
        return "granted";
      },
      checkPermissions: async () =>
        granted ? { ...oneMissing, input_monitoring: "granted" } : oneMissing,
    });
    render(<LandingContainer api={api} onOpenWorkflow={() => {}} />);

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Could not load workflows");

    fireEvent.click(
      screen.getByRole("button", { name: /Input Monitoring/ }),
    );
    // The request round-trip settles (pill re-renders as granted) and
    // the load error is still on screen.
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /✓ Input Monitoring/ }),
      ).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toContain(
      "Could not load workflows",
    );
  });
});

// Component tests for the landing view (AC-002): row rendering from
// summary fixtures including placeholder fallbacks, Record gating on
// permission states with its hint, and reveal-versus-navigate behavior.

import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";

import type { PermissionReport, WorkflowSummary } from "../api/client";
import { formatSummaryMeta } from "../lib/format";
import { LandingView, type LandingViewProps } from "./LandingView";

const granted: PermissionReport = {
  input_monitoring: "granted",
  accessibility: "granted",
  screen_recording: "granted",
};

// Coherent fixture as of 2026-08-16T22:31:41Z: a recording finished at
// 22:31:23 with ten manifest steps over an 18.25s event span.
const approveInvoice: WorkflowSummary = {
  id: "2026-08-16-223105-9f3a",
  name: "Approve invoice",
  created_at: "2026-08-16T22:31:05Z",
  step_count: 10,
  duration_ms: 18_250,
  thumbnail_event_id: "evt_0001",
};

// A workflow whose event log is damaged: placeholder duration and
// thumbnail (DEC-006), one manifest step.
const damagedLog: WorkflowSummary = {
  id: "2026-08-15-140200-11ab",
  name: "Export payroll report",
  created_at: "2026-08-15T14:02:00Z",
  step_count: 1,
  duration_ms: null,
  thumbnail_event_id: null,
};

function renderLanding(overrides: Partial<LandingViewProps> = {}) {
  const props: LandingViewProps = {
    workflows: [approveInvoice, damagedLog],
    error: null,
    permissions: granted,
    thumbnails: new Map([[approveInvoice.id, "blob:mock-thumb-1"]]),
    onRequestPermission: vi.fn(),
    onOpenWorkflow: vi.fn(),
    onRevealWorkflow: vi.fn(),
    onRecord: vi.fn(),
    ...overrides,
  };
  render(<LandingView {...props} />);
  return props;
}

describe("workflow rows", () => {
  it("renders one row per summary with name and date · steps · duration", () => {
    renderLanding();

    const rows = screen.getAllByRole("listitem");
    expect(rows).toHaveLength(2);

    const first = within(rows[0]);
    expect(first.getByText("Approve invoice")).toBeTruthy();
    expect(first.getByText(formatSummaryMeta(approveInvoice))).toBeTruthy();
    expect(formatSummaryMeta(approveInvoice)).toContain("10 steps · 18s");

    const thumb = first.getByRole("img");
    expect(thumb.getAttribute("src")).toBe("blob:mock-thumb-1");
  });

  it("falls back to labeled placeholders for a damaged event log", () => {
    renderLanding();

    const row = screen.getAllByRole("listitem")[1];
    const scoped = within(row);
    expect(scoped.queryByRole("img")).toBeNull();
    expect(scoped.getByText("No preview")).toBeTruthy();
    const meta = formatSummaryMeta(damagedLog);
    expect(meta).toContain("1 step · —");
    expect(scoped.getByText(meta)).toBeTruthy();
  });

  it("shows the empty state when no workflows exist", () => {
    renderLanding({ workflows: [], thumbnails: new Map() });
    expect(screen.queryByRole("list")).toBeNull();
    expect(screen.getByText(/No workflows yet/)).toBeTruthy();
  });
});

describe("record gating", () => {
  it("enables Record without a hint when all three permissions are granted", () => {
    renderLanding();
    const record = screen.getByRole("button", { name: /Record New Workflow/ });
    expect((record as HTMLButtonElement).disabled).toBe(false);
    expect(screen.queryByText(/Grant the missing permission/)).toBeNull();
  });

  it.each([
    ["input_monitoring", "not_requested"],
    ["accessibility", "blocked_by_prerequisite"],
    ["screen_recording", "denied"],
  ] as const)(
    "disables Record and explains when %s is %s",
    (kind, status) => {
      renderLanding({ permissions: { ...granted, [kind]: status } });
      const record = screen.getByRole("button", { name: /Record New Workflow/ });
      expect((record as HTMLButtonElement).disabled).toBe(true);
      expect(
        screen.getByText("Grant the missing permission above to enable recording."),
      ).toBeTruthy();
    },
  );

  it("requests the clicked permission from its pill", () => {
    const props = renderLanding({
      permissions: { ...granted, screen_recording: "denied" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Screen Recording/ }));
    expect(props.onRequestPermission).toHaveBeenCalledWith("screen_recording");
  });
});

describe("reveal versus navigate", () => {
  it("opens the workflow when the row body is clicked", () => {
    const props = renderLanding();
    fireEvent.click(screen.getByRole("button", { name: /Approve invoice/ }));
    expect(props.onOpenWorkflow).toHaveBeenCalledWith(approveInvoice);
    expect(props.onRevealWorkflow).not.toHaveBeenCalled();
  });

  it("reveals without navigating when the reveal control is clicked", () => {
    const props = renderLanding();
    const row = screen.getAllByRole("listitem")[0];
    fireEvent.click(within(row).getByRole("button", { name: "⌘ Reveal" }));
    expect(props.onRevealWorkflow).toHaveBeenCalledWith(approveInvoice.id);
    expect(props.onOpenWorkflow).not.toHaveBeenCalled();
  });

  it("lays out Reveal and the chevron as separate row siblings", () => {
    // The pinned prototype Home row: open target, Reveal, chevron in
    // flow order, so the hover-revealed control cannot overlap the
    // chevron (final-gate finding, revision epoch 2).
    renderLanding();
    const row = screen.getAllByRole("listitem")[0];
    const open = within(row).getByRole("button", { name: /Approve invoice/ });
    const reveal = within(row).getByRole("button", { name: "⌘ Reveal" });
    const chevron = within(row).getByText("›");
    expect(chevron.parentElement).toBe(row);
    expect(open.contains(chevron)).toBe(false);
    expect(reveal.compareDocumentPosition(chevron)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(open.compareDocumentPosition(reveal)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
  });
});

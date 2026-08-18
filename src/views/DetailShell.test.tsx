// The detail shell shows the workflow name and navigates back to the
// landing page (AC-002); PR-02 fills the review pane.

import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

import { DetailShell } from "./DetailShell";

describe("detail shell", () => {
  it("shows the workflow name and a back control that navigates", () => {
    const onBack = vi.fn();
    render(<DetailShell workflowName="Approve invoice" onBack={onBack} />);

    expect(
      screen.getByRole("heading", { name: "Approve invoice" }),
    ).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "‹ Workflows" }));
    expect(onBack).toHaveBeenCalledTimes(1);
  });
});

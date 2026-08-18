// The discriminated view reducer: landing <-> detail navigation.

import { describe, expect, it } from "vitest";

import { initialView, viewReducer } from "./view";

describe("view reducer", () => {
  it("navigates into a workflow and back", () => {
    const detail = viewReducer(initialView, {
      kind: "open_workflow",
      workflowId: "2026-08-16-223105-9f3a",
      workflowName: "Approve invoice",
    });
    expect(detail).toEqual({
      kind: "detail",
      workflowId: "2026-08-16-223105-9f3a",
      workflowName: "Approve invoice",
    });

    expect(viewReducer(detail, { kind: "back_to_landing" })).toEqual({
      kind: "landing",
    });
  });
});

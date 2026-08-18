// The discriminated view reducer: landing <-> detail navigation and
// the record-flow entry and exits.

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
      error: null,
    });
  });

  it("enters the record flow and exits with or without an error", () => {
    const record = viewReducer(initialView, { kind: "start_record" });
    expect(record).toEqual({ kind: "record" });

    expect(
      viewReducer(record, { kind: "exit_record", error: null }),
    ).toEqual({ kind: "landing", error: null });
    expect(
      viewReducer(record, {
        kind: "exit_record",
        error: "Recording failed: tap disabled",
      }),
    ).toEqual({ kind: "landing", error: "Recording failed: tap disabled" });
  });
});

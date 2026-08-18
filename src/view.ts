// The discriminated view state of the product shell and its reducer:
// the landing page, the detail view for a saved workflow, and the
// record flow (live capture through draft review, PR-03). The landing
// error carries a record-flow exit failure onto the landing page.

export type View =
  | { kind: "landing"; error: string | null }
  | { kind: "detail"; workflowId: string; workflowName: string }
  | { kind: "record" };

export type ViewAction =
  | { kind: "open_workflow"; workflowId: string; workflowName: string }
  | { kind: "back_to_landing" }
  | { kind: "start_record" }
  | { kind: "exit_record"; error: string | null };

export const initialView: View = { kind: "landing", error: null };

export function viewReducer(_view: View, action: ViewAction): View {
  switch (action.kind) {
    case "open_workflow":
      return {
        kind: "detail",
        workflowId: action.workflowId,
        workflowName: action.workflowName,
      };
    case "back_to_landing":
      return { kind: "landing", error: null };
    case "start_record":
      return { kind: "record" };
    case "exit_record":
      return { kind: "landing", error: action.error };
    default: {
      const exhausted: never = action;
      return exhausted;
    }
  }
}

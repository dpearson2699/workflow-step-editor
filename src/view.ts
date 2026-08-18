// The discriminated view state of the product shell and its reducer.
// PR-01 ships the landing view and the detail shell; later slices add
// the recording and draft-review states.

export type View =
  | { kind: "landing" }
  | { kind: "detail"; workflowId: string; workflowName: string };

export type ViewAction =
  | { kind: "open_workflow"; workflowId: string; workflowName: string }
  | { kind: "back_to_landing" };

export const initialView: View = { kind: "landing" };

export function viewReducer(_view: View, action: ViewAction): View {
  switch (action.kind) {
    case "open_workflow":
      return {
        kind: "detail",
        workflowId: action.workflowId,
        workflowName: action.workflowName,
      };
    case "back_to_landing":
      return { kind: "landing" };
    default: {
      const exhausted: never = action;
      return exhausted;
    }
  }
}

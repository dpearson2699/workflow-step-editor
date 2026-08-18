// The typed frontend API client: the only module that touches the Tauri
// IPC surface (`invoke`/`Channel`). Views receive data and callbacks;
// they never import from @tauri-apps/api directly.

import { invoke } from "@tauri-apps/api/core";

export type PermissionStatus =
  | "granted"
  | "denied"
  | "not_requested"
  | "blocked_by_prerequisite";

export type PermissionKind =
  | "input_monitoring"
  | "accessibility"
  | "screen_recording";

export interface PermissionReport {
  input_monitoring: PermissionStatus;
  accessibility: PermissionStatus;
  screen_recording: PermissionStatus;
}

/** One row of `list_workflows` (backend `WorkflowSummary`). */
export interface WorkflowSummary {
  id: string;
  name: string;
  /** RFC 3339 UTC timestamp with second precision. */
  created_at: string;
  /** Number of manifest steps (not raw events). */
  step_count: number;
  /**
   * Milliseconds from the first to the last event timestamp; zero with
   * fewer than two events; null when the event log is unreadable.
   */
  duration_ms: number | null;
  /**
   * Event id whose window crop is the row thumbnail; null when there is
   * no step or the event log is unreadable (DEC-006).
   */
  thumbnail_event_id: string | null;
}

/** The DEC-007 screenshot variant allowlist. */
export type ShotVariant = "full" | "window" | "element";

/** The IPC surface this slice of the product uses. */
export interface ApiClient {
  checkPermissions(): Promise<PermissionReport>;
  requestPermission(kind: PermissionKind): Promise<PermissionStatus>;
  listWorkflows(): Promise<WorkflowSummary[]>;
  /** Reveals the workflow folder in Finder (backend-resolved path). */
  revealWorkflow(id: string): Promise<void>;
  /** The scoped screenshot read: raw PNG bytes by ids, never paths. */
  readScreenshot(
    workflowId: string,
    eventId: string,
    variant: ShotVariant,
  ): Promise<Uint8Array>;
}

export function createTauriClient(): ApiClient {
  return {
    checkPermissions() {
      return invoke<PermissionReport>("check_permissions");
    },
    requestPermission(kind) {
      return invoke<PermissionStatus>("request_permission", { kind });
    },
    listWorkflows() {
      return invoke<WorkflowSummary[]>("list_workflows");
    },
    revealWorkflow(id) {
      return invoke<void>("reveal_workflow", { id });
    },
    async readScreenshot(workflowId, eventId, variant) {
      const bytes = await invoke<ArrayBuffer>("read_screenshot", {
        workflowId,
        eventId,
        variant,
      });
      return new Uint8Array(bytes);
    },
  };
}

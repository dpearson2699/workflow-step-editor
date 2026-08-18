// The typed frontend API client: the only module that touches the Tauri
// IPC surface (`invoke`/`Channel`). Views receive data and callbacks;
// they never import from @tauri-apps/api directly.

import { Channel, invoke } from "@tauri-apps/api/core";

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

/** The four-value step classification enum (schema v1). */
export type Classification = "click" | "type" | "wait" | "assert";

/** One reviewable manifest step. */
export interface Step {
  id: string;
  /** The raw events this step was parsed from; resolve by id, never
   *  by array index. */
  event_ids: string[];
  classification: Classification;
  title: string;
  description: string;
}

/** The editable `workflow.json` manifest. */
export interface Manifest {
  schema_version: number;
  id: string;
  name: string;
  created_at: string;
  steps: Step[];
}

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** The `key` object of a key-down event (null on clicks). */
export interface KeyInfo {
  key_code: number;
  chars: string;
  /** snake_case modifier names in capture order. */
  modifiers: string[];
}

/** The window the event hit (null when none resolved, DEC-011). */
export interface WindowInfo {
  app: string;
  title: string;
  pid: number;
  bounds: Rect;
}

/** Where the element metadata came from. */
export type ElementSource = "ax" | "fallback";

export interface ElementInfo {
  role: string | null;
  title: string | null;
  frame: Rect;
  source: ElementSource;
}

/** One line of `events.jsonl`: a raw captured event. */
export interface WorkflowEvent {
  id: string;
  /** RFC 3339 UTC timestamp with millisecond precision. */
  ts: string;
  kind: "click" | "key_down";
  display_id: number;
  pos: { x: number; y: number };
  button: "left" | "right" | "middle" | null;
  key: KeyInfo | null;
  window: WindowInfo | null;
  element: ElementInfo;
}

/** The result of `get_workflow`: manifest plus the raw event log. */
export interface LoadedWorkflow {
  manifest: Manifest;
  events: WorkflowEvent[];
}

/**
 * One tagged item on the live capture channel (backend `LiveEnvelope`).
 * `step` items stream one per captured event, followed by exactly one
 * terminal `stopped` or `failed` item (terminal-last). The `ts` on a
 * step item is the source event's RFC 3339 timestamp — a transient
 * envelope field for the live rows (DEC-009), not a schema field.
 */
export type LiveEnvelope =
  | { type: "step"; step: Step; ts: string }
  | { type: "stopped"; workflow_id: string }
  | { type: "failed"; workflow_id: string; error: string };

/** A transient step patch: only supplied fields change (DEC-004). */
export interface StepPatch {
  title?: string;
  description?: string;
  classification?: Classification;
}

/** The IPC surface this slice of the product uses. */
export interface ApiClient {
  checkPermissions(): Promise<PermissionReport>;
  requestPermission(kind: PermissionKind): Promise<PermissionStatus>;
  listWorkflows(): Promise<WorkflowSummary[]>;
  getWorkflow(id: string): Promise<LoadedWorkflow>;
  /** Reveals the workflow folder in Finder (backend-resolved path). */
  revealWorkflow(id: string): Promise<void>;
  /** The scoped screenshot read: raw PNG bytes by ids, never paths. */
  readScreenshot(
    workflowId: string,
    eventId: string,
    variant: ShotVariant,
  ): Promise<Uint8Array>;
  updateStep(workflowId: string, stepId: string, patch: StepPatch): Promise<void>;
  deleteStep(workflowId: string, stepId: string): Promise<void>;
  /** Manifest name only, trimmed and non-empty; folder and id never change. */
  renameWorkflow(id: string, name: string): Promise<void>;
  /** ADR 0003 hard delete: success means the folder is absent. */
  deleteWorkflow(id: string): Promise<void>;
  /**
   * Starts a recording under the manifest's default timestamp name.
   * Envelopes stream to `onEnvelope` until the terminal item; envelopes
   * may arrive before the returned promise (the workflow id) resolves.
   */
  startRecording(onEnvelope: (envelope: LiveEnvelope) => void): Promise<string>;
  /** Stops the active recording; resolves with the workflow id after
   *  finalization. The terminal envelope arrives on the channel. */
  stopRecording(): Promise<string>;
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
    getWorkflow(id) {
      return invoke<LoadedWorkflow>("get_workflow", { id });
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
    updateStep(workflowId, stepId, patch) {
      return invoke<void>("update_step", { workflowId, stepId, patch });
    },
    deleteStep(workflowId, stepId) {
      return invoke<void>("delete_step", { workflowId, stepId });
    },
    renameWorkflow(id, name) {
      return invoke<void>("rename_workflow", { id, name });
    },
    deleteWorkflow(id) {
      return invoke<void>("delete_workflow", { id });
    },
    startRecording(onEnvelope) {
      const channel = new Channel<LiveEnvelope>();
      channel.onmessage = onEnvelope;
      // No frontend name: the backend names the manifest with its
      // timestamp default, which the naming dialog later pre-selects.
      return invoke<string>("start_recording", { name: null, channel });
    },
    stopRecording() {
      return invoke<string>("stop_recording");
    },
  };
}

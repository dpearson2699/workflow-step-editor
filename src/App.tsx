// The product shell (issue #13, DEC-001): a discriminated view reducer
// over the landing page and the detail shell. Data flows through the
// typed API client; views stay presentational.

import { useEffect, useReducer, useRef, useState } from "react";

import type {
  ApiClient,
  PermissionKind,
  PermissionReport,
  WorkflowSummary,
} from "./api/client";
import { createTauriClient } from "./api/client";
import { initialView, viewReducer } from "./view";
import { DetailShell } from "./views/DetailShell";
import { LandingView } from "./views/LandingView";
import "./App.css";

const client = createTauriClient();

/**
 * Loads row thumbnails through the scoped screenshot read and caches
 * the blob URLs for this landing view. Every URL is revoked when the
 * view unmounts or a cache entry is replaced (DEC-007).
 */
function useThumbnails(
  api: ApiClient,
  workflows: WorkflowSummary[] | null,
): ReadonlyMap<string, string> {
  const [urls, setUrls] = useState<ReadonlyMap<string, string>>(new Map());
  const cache = useRef(new Map<string, string>());
  const requested = useRef(new Set<string>());
  const disposed = useRef(false);

  useEffect(() => {
    disposed.current = false;
    return () => {
      disposed.current = true;
      for (const url of cache.current.values()) {
        URL.revokeObjectURL(url);
      }
      cache.current.clear();
      requested.current.clear();
    };
  }, []);

  useEffect(() => {
    if (workflows === null) {
      return;
    }
    for (const workflow of workflows) {
      const eventId = workflow.thumbnail_event_id;
      if (eventId === null || requested.current.has(workflow.id)) {
        continue;
      }
      requested.current.add(workflow.id);
      api
        .readScreenshot(workflow.id, eventId, "window")
        .then((bytes) => {
          const blob = new Blob([bytes as BlobPart], { type: "image/png" });
          const url = URL.createObjectURL(blob);
          if (disposed.current) {
            URL.revokeObjectURL(url);
            return;
          }
          const replaced = cache.current.get(workflow.id);
          if (replaced !== undefined) {
            URL.revokeObjectURL(replaced);
          }
          cache.current.set(workflow.id, url);
          setUrls(new Map(cache.current));
        })
        .catch(() => {
          // The row keeps its labeled placeholder (DEC-006).
        });
    }
  }, [api, workflows]);

  return urls;
}

/** Exported for the container component test only. */
export function LandingContainer(props: {
  api: ApiClient;
  onOpenWorkflow: (workflow: WorkflowSummary) => void;
}) {
  const { api } = props;
  const [permissions, setPermissions] = useState<PermissionReport | null>(null);
  const [workflows, setWorkflows] = useState<WorkflowSummary[] | null>(null);
  // Mount-time load failures persist; user-action failures reset per
  // action. Separate channels keep a permission request or reveal from
  // clearing the workflow-load error while the list is still missing.
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const thumbnails = useThumbnails(api, workflows);

  useEffect(() => {
    let stale = false;
    api
      .checkPermissions()
      .then((report) => {
        if (!stale) {
          setPermissions(report);
        }
      })
      .catch((caught) => {
        if (!stale) {
          setLoadError(String(caught));
        }
      });
    api
      .listWorkflows()
      .then((list) => {
        if (!stale) {
          setWorkflows(list);
        }
      })
      .catch((caught) => {
        if (!stale) {
          setLoadError(`Could not load workflows: ${String(caught)}`);
        }
      });
    return () => {
      stale = true;
    };
  }, [api]);

  async function requestPermission(kind: PermissionKind) {
    setActionError(null);
    try {
      await api.requestPermission(kind);
      setPermissions(await api.checkPermissions());
    } catch (caught) {
      setActionError(String(caught));
    }
  }

  async function revealWorkflow(id: string) {
    setActionError(null);
    try {
      await api.revealWorkflow(id);
    } catch (caught) {
      setActionError(String(caught));
    }
  }

  return (
    <LandingView
      workflows={workflows}
      error={actionError ?? loadError}
      permissions={permissions}
      thumbnails={thumbnails}
      onRequestPermission={(kind) => void requestPermission(kind)}
      onOpenWorkflow={props.onOpenWorkflow}
      onRevealWorkflow={(id) => void revealWorkflow(id)}
      onRecord={() => {
        // The record flow ships in PR-03 (AC-004); the gate and hint
        // are this slice's contract (AC-002).
      }}
    />
  );
}

function App() {
  const [view, dispatch] = useReducer(viewReducer, initialView);

  if (view.kind === "detail") {
    return (
      <DetailShell
        workflowName={view.workflowName}
        onBack={() => dispatch({ kind: "back_to_landing" })}
      />
    );
  }
  return (
    <LandingContainer
      api={client}
      onOpenWorkflow={(workflow) =>
        dispatch({
          kind: "open_workflow",
          workflowId: workflow.id,
          workflowName: workflow.name,
        })
      }
    />
  );
}

export default App;

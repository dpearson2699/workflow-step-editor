// The product shell (issue #13, DEC-001): a discriminated view reducer
// over the landing page, the detail shell, and the record flow. Data
// flows through the typed API client; views stay presentational. The
// record session lives in a shell-owned ref (never a mount effect), so
// StrictMode effect replays and re-renders cannot double-start a
// recording.

import {
  useEffect,
  useReducer,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";

import type {
  ApiClient,
  PermissionKind,
  PermissionReport,
  WorkflowSummary,
} from "./api/client";
import { createTauriClient } from "./api/client";
import { startRecordSession, type RecordSession } from "./record";
import { initialView, viewReducer } from "./view";
import { DetailView } from "./views/DetailView";
import { LandingView } from "./views/LandingView";
import { RecordingView } from "./views/RecordingView";
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
  onRecord?: () => void;
  /** A record-flow exit error surfaced on the landing page (AC-004). */
  initialError?: string | null;
}) {
  const { api } = props;
  const [permissions, setPermissions] = useState<PermissionReport | null>(null);
  const [workflows, setWorkflows] = useState<WorkflowSummary[] | null>(null);
  // Mount-time load failures persist; user-action failures reset per
  // action. Separate channels keep a permission request or reveal from
  // clearing the workflow-load error while the list is still missing.
  const [loadError, setLoadError] = useState<string | null>(
    props.initialError ?? null,
  );
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
      onRecord={() => props.onRecord?.()}
    />
  );
}

/**
 * Renders one record session (AC-004): the live capture view while the
 * session streams, then the detail view in draft mode once the terminal
 * envelope lands. Exported for the record-flow component tests.
 */
export function RecordFlowContainer(props: {
  api: ApiClient;
  session: RecordSession;
  /** Leaves the flow for the landing page; `error` shows there. */
  onExit: (error: string | null) => void;
}) {
  const { session, onExit } = props;
  const state = useSyncExternalStore(session.subscribe, session.getState);

  const exitedError = state.phase === "exited" ? state.error : null;
  const exited = state.phase === "exited";
  useEffect(() => {
    if (exited) {
      onExit(exitedError);
    }
  }, [exited, exitedError, onExit]);

  if (state.phase === "draft") {
    return (
      <DetailView
        api={props.api}
        workflowId={state.workflowId}
        initialName=""
        draft={{ failure: state.failure }}
        onBack={() => onExit(null)}
        onDeleted={() => onExit(null)}
      />
    );
  }
  if (state.phase === "exited") {
    return null;
  }
  return (
    <RecordingView
      rows={state.rows}
      stopPending={state.stopPending}
      stopError={state.stopError}
      onStop={() => session.stop()}
    />
  );
}

/** The shell over an injected API client; exported for component tests. */
export function AppShell(props: { api: ApiClient }) {
  const { api } = props;
  const [view, dispatch] = useReducer(viewReducer, initialView);
  // The active record session. Owned here (not in a mount effect) so a
  // double Record click or a StrictMode replay never starts a second
  // recording; cleared on every exit from the record view.
  const sessionRef = useRef<RecordSession | null>(null);

  function record() {
    if (sessionRef.current !== null) {
      return;
    }
    sessionRef.current = startRecordSession(api);
    dispatch({ kind: "start_record" });
  }

  function exitRecord(error: string | null) {
    sessionRef.current?.dispose();
    sessionRef.current = null;
    dispatch({ kind: "exit_record", error });
  }

  if (view.kind === "detail") {
    return (
      <DetailView
        api={api}
        workflowId={view.workflowId}
        initialName={view.workflowName}
        onBack={() => dispatch({ kind: "back_to_landing" })}
        onDeleted={() => dispatch({ kind: "back_to_landing" })}
      />
    );
  }
  if (view.kind === "record" && sessionRef.current !== null) {
    return (
      <RecordFlowContainer
        api={api}
        session={sessionRef.current}
        onExit={exitRecord}
      />
    );
  }
  return (
    <LandingContainer
      api={api}
      initialError={view.kind === "landing" ? view.error : null}
      onOpenWorkflow={(workflow) =>
        dispatch({
          kind: "open_workflow",
          workflowId: workflow.id,
          workflowName: workflow.name,
        })
      }
      onRecord={record}
    />
  );
}

function App() {
  return <AppShell api={client} />;
}

export default App;

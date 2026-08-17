import { useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";

// The tagged live-channel envelope the Rust `LiveEnvelope` serializes to.
type LiveEnvelope =
  | { type: "step"; step: Step }
  | { type: "stopped"; workflow_id: string }
  | { type: "failed"; workflow_id: string; error: string };

interface Step {
  id: string;
  event_ids: string[];
  classification: string;
  title: string;
  description: string;
}

type PermissionStatus =
  | "granted"
  | "denied"
  | "not_requested"
  | "blocked_by_prerequisite";

interface PermissionReport {
  input_monitoring: PermissionStatus;
  accessibility: PermissionStatus;
  screen_recording: PermissionStatus;
}

// This is the bare dev-only capture trigger (DEC-010). It is developer
// scaffolding, not the product review UI (issue #13): start/stop plus
// minimal live output (latest step title and received count) to prove
// the capture channel end to end during the proven gate.
function App() {
  const [recording, setRecording] = useState(false);
  const [workflowId, setWorkflowId] = useState<string | null>(null);
  const [latestStep, setLatestStep] = useState<string>("—");
  const [count, setCount] = useState(0);
  const [terminal, setTerminal] = useState<string>("");
  const [permissions, setPermissions] = useState<PermissionReport | null>(null);
  const [error, setError] = useState<string>("");

  async function checkPermissions() {
    setError("");
    try {
      setPermissions(await invoke<PermissionReport>("check_permissions"));
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function requestPermission(kind: string) {
    setError("");
    try {
      await invoke("request_permission", { kind });
      await checkPermissions();
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function startRecording() {
    setError("");
    setLatestStep("—");
    setCount(0);
    setTerminal("");

    const channel = new Channel<LiveEnvelope>();
    channel.onmessage = (message) => {
      if (message.type === "step") {
        setLatestStep(message.step.title);
        setCount((current) => current + 1);
      } else if (message.type === "stopped") {
        setTerminal(`stopped (${message.workflow_id})`);
        setRecording(false);
      } else {
        setTerminal(`failed: ${message.error}`);
        setRecording(false);
      }
    };

    try {
      const id = await invoke<string>("start_recording", {
        name: null,
        channel,
      });
      setWorkflowId(id);
      setRecording(true);
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function stopRecording() {
    setError("");
    try {
      await invoke<string>("stop_recording");
    } catch (caught) {
      setError(String(caught));
    }
  }

  return (
    <main style={{ fontFamily: "system-ui", padding: "1.5rem", maxWidth: 640 }}>
      <h1>Workflow Step Editor</h1>
      <p style={{ color: "#666" }}>
        Dev-only capture trigger. The product review UI arrives in a later
        capability.
      </p>

      <section style={{ marginBottom: "1rem" }}>
        <button type="button" onClick={checkPermissions}>
          Check permissions
        </button>
        {permissions && (
          <ul>
            <li>
              Input Monitoring: {permissions.input_monitoring}{" "}
              <button
                type="button"
                onClick={() => requestPermission("input_monitoring")}
              >
                Request
              </button>
            </li>
            <li>
              Accessibility: {permissions.accessibility}{" "}
              <button
                type="button"
                onClick={() => requestPermission("accessibility")}
              >
                Request
              </button>
            </li>
            <li>
              Screen Recording: {permissions.screen_recording}{" "}
              <button
                type="button"
                onClick={() => requestPermission("screen_recording")}
              >
                Request
              </button>
            </li>
          </ul>
        )}
      </section>

      <section style={{ marginBottom: "1rem" }}>
        <button type="button" onClick={startRecording} disabled={recording}>
          Start recording
        </button>{" "}
        <button type="button" onClick={stopRecording} disabled={!recording}>
          Stop recording
        </button>
      </section>

      <section>
        <p>
          <strong>Status:</strong>{" "}
          {recording ? "recording" : "idle"}
          {workflowId ? ` · workflow ${workflowId}` : ""}
        </p>
        <p>
          <strong>Steps received:</strong> {count}
        </p>
        <p>
          <strong>Latest step:</strong> {latestStep}
        </p>
        {terminal && (
          <p>
            <strong>Terminal:</strong> {terminal}
          </p>
        )}
        {error && (
          <p style={{ color: "#b00" }}>
            <strong>Error:</strong> {error}
          </p>
        )}
      </section>
    </main>
  );
}

export default App;

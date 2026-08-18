//! Thin Tauri command wiring for the capture lifecycle (issue #7
//! decision 4). All behavior lives in the application services; commands
//! translate arguments and map errors to strings.

use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::recording::channel::{LiveEnvelope, StepSink};
use crate::recording::coordinator::RecordingCoordinator;
use crate::recording::store::{LoadedWorkflow, ShotVariant, WorkflowSummary};

/// Managed capture-lifecycle service.
pub struct RecorderState(pub Arc<RecordingCoordinator>);

/// The production [`StepSink`]: the live IPC channel handed to
/// `start_recording`. Send failures surface as sink errors, which the
/// recording worker ignores so persistence continues.
struct ChannelSink(Channel<LiveEnvelope>);

impl StepSink for ChannelSink {
    fn emit(&self, item: LiveEnvelope) -> Result<(), String> {
        self.0.send(item).map_err(|error| error.to_string())
    }
}

#[tauri::command]
pub async fn start_recording(
    state: State<'_, RecorderState>,
    name: Option<String>,
    channel: Channel<LiveEnvelope>,
) -> Result<String, String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        coordinator
            .start_recording(name.as_deref(), Box::new(ChannelSink(channel)))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("recording task failed: {error}"))?
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, RecorderState>) -> Result<String, String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        coordinator
            .stop_recording()
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("recording task failed: {error}"))?
}

#[tauri::command]
pub fn list_workflows(state: State<'_, RecorderState>) -> Result<Vec<WorkflowSummary>, String> {
    state.0.list_workflows().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_workflow(
    state: State<'_, RecorderState>,
    id: String,
) -> Result<LoadedWorkflow, String> {
    state.0.get_workflow(&id).map_err(|error| error.to_string())
}

/// Reveals the workflow's folder in Finder. The backend resolves and
/// validates the folder through the store; no filesystem path crosses
/// IPC in either direction (DEC-007).
#[tauri::command]
pub async fn reveal_workflow(state: State<'_, RecorderState>, id: String) -> Result<(), String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = coordinator
            .workflow_path(&id)
            .map_err(|error| error.to_string())?;
        let status = std::process::Command::new("/usr/bin/open")
            .arg("-R")
            .arg(&path)
            .status()
            .map_err(|error| format!("could not reveal workflow in Finder: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "could not reveal workflow in Finder: open exited with {status}",
            ))
        }
    })
    .await
    .map_err(|error| format!("reveal task failed: {error}"))?
}

/// The scoped screenshot read (DEC-007): workflow id + event id +
/// allowlisted variant resolve to the canonical PNG, returned as raw
/// bytes. Paths are never accepted or exposed.
#[tauri::command]
pub async fn read_screenshot(
    state: State<'_, RecorderState>,
    workflow_id: String,
    event_id: String,
    variant: String,
) -> Result<tauri::ipc::Response, String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let variant: ShotVariant = variant.parse().map_err(
            |error: crate::recording::store::InvalidShotVariant| error.to_string(),
        )?;
        coordinator
            .read_screenshot(&workflow_id, &event_id, variant)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("screenshot read task failed: {error}"))?
    .map(tauri::ipc::Response::new)
}

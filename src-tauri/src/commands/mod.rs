//! Thin Tauri command wiring for the capture lifecycle (issue #7
//! decision 4). All behavior lives in the application services; commands
//! translate arguments and map errors to strings.

use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::recording::channel::{LiveEnvelope, StepSink};
use crate::recording::coordinator::RecordingCoordinator;
use crate::recording::store::{LoadedWorkflow, WorkflowSummary};

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

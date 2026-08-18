//! Thin Tauri command wiring for the capture lifecycle (issue #7
//! decision 4). All behavior lives in the application services; commands
//! translate arguments and map errors to strings.

use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::recording::channel::{LiveEnvelope, StepSink};
use crate::recording::coordinator::{RecordingCoordinator, RecordingError, StepPatch};
use crate::recording::store::{LoadedWorkflow, ShotVariant, StoreError, WorkflowSummary};

/// Renders a DEC-007 command failure for IPC without any filesystem
/// path: id-carrying validation and not-found variants pass through
/// unchanged, and the path-carrying storage variants collapse to
/// stable path-free descriptions.
fn path_free_error(error: &RecordingError) -> String {
    match error {
        RecordingError::Store(StoreError::SymlinkRejected(_)) => {
            "storage error: refused a symlinked location".to_owned()
        }
        RecordingError::Store(StoreError::Io { .. }) => {
            "storage error: could not access the workflow data".to_owned()
        }
        other => other.to_string(),
    }
}

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
            .map_err(|error| path_free_error(&error))?;
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
            .map_err(|error| path_free_error(&error))
    })
    .await
    .map_err(|error| format!("screenshot read task failed: {error}"))?
    .map(tauri::ipc::Response::new)
}

/// Applies a transient step patch (title, description, classification)
/// through the coordinator's DEC-008-serialized manifest mutation path.
#[tauri::command]
pub async fn update_step(
    state: State<'_, RecorderState>,
    workflow_id: String,
    step_id: String,
    patch: StepPatch,
) -> Result<(), String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        coordinator
            .update_step(&workflow_id, &step_id, patch)
            .map_err(|error| path_free_error(&error))
    })
    .await
    .map_err(|error| format!("step update task failed: {error}"))?
}

/// Removes one step entry from the manifest; its raw events and
/// screenshots stay byte-identical.
#[tauri::command]
pub async fn delete_step(
    state: State<'_, RecorderState>,
    workflow_id: String,
    step_id: String,
) -> Result<(), String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        coordinator
            .delete_step(&workflow_id, &step_id)
            .map_err(|error| path_free_error(&error))
    })
    .await
    .map_err(|error| format!("step deletion task failed: {error}"))?
}

/// Renames the workflow: manifest name only, trimmed and non-empty; the
/// folder and id never change.
#[tauri::command]
pub async fn rename_workflow(
    state: State<'_, RecorderState>,
    id: String,
    name: String,
) -> Result<(), String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        coordinator
            .rename_workflow(&id, &name)
            .map_err(|error| path_free_error(&error))
    })
    .await
    .map_err(|error| format!("rename task failed: {error}"))?
}

/// Hard-deletes a saved workflow (ADR 0003): the backend resolves and
/// validates the folder inside the workflow root and removes it whole.
/// Success means the directory is absent; an already-missing directory
/// counts as deleted.
#[tauri::command]
pub async fn delete_workflow(state: State<'_, RecorderState>, id: String) -> Result<(), String> {
    let coordinator = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        coordinator
            .delete_workflow(&id)
            .map_err(|error| path_free_error(&error))
    })
    .await
    .map_err(|error| format!("workflow deletion task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DEC-007: no filesystem path crosses IPC in either direction —
    /// including the error channel of the two scoped commands.
    #[test]
    fn dec_007_ipc_error_strings_never_carry_a_filesystem_path() {
        let path_bearing = [
            RecordingError::Store(StoreError::SymlinkRejected(
                "/private/store/wf_1/shots".into(),
            )),
            RecordingError::Store(StoreError::Io {
                context: "open /private/store/wf_1/shots/evt_9999.window.png".to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            }),
        ];
        for error in &path_bearing {
            let message = path_free_error(error);
            assert!(
                !message.contains('/'),
                "filesystem path leaked into the IPC error string: {message}",
            );
        }
    }

    /// Id-carrying variants keep their diagnostic value unchanged.
    #[test]
    fn path_free_mapping_passes_id_only_variants_through() {
        for error in [
            RecordingError::Store(StoreError::NotFound("wf_1".to_owned())),
            RecordingError::Store(StoreError::InvalidWorkflowId("../x".to_owned())),
            RecordingError::Store(StoreError::InvalidEventId("a/b".to_owned())),
        ] {
            assert_eq!(path_free_error(&error), error.to_string());
        }
    }
}

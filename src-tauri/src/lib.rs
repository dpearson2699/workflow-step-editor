pub mod capture;
mod commands;
pub mod domain;
mod permissions;
pub mod recording;

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use tauri::{Manager, State};

use permissions::macos::MacosPermissionSource;
use permissions::{PermissionKind, PermissionReport, PermissionService, PermissionStatus};
use recording::clock::{Clock, SystemClock};
use recording::coordinator::{PermissionGate, PipelineFactory, RecordingCoordinator};
use recording::store::JsonWorkflowStore;

/// Managed permission state. The mutex serializes all permission
/// operations so concurrent commands cannot violate the request order;
/// the recording coordinator gates start on the same shared service.
struct PermissionState(Arc<Mutex<PermissionService<MacosPermissionSource>>>);

#[tauri::command]
fn check_permissions(state: State<'_, PermissionState>) -> PermissionReport {
    state
        .0
        .lock()
        .expect("permission service mutex poisoned")
        .check_all()
}

#[tauri::command]
fn request_permission(
    state: State<'_, PermissionState>,
    kind: String,
) -> Result<PermissionStatus, String> {
    let kind = PermissionKind::from_str(&kind).map_err(|error| error.to_string())?;
    Ok(state
        .0
        .lock()
        .expect("permission service mutex poisoned")
        .request(kind))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let permission_service = Arc::new(Mutex::new(PermissionService::new(MacosPermissionSource)));
    let permission_gate: Arc<dyn PermissionGate> = permission_service.clone();
    tauri::Builder::default()
        .manage(PermissionState(permission_service))
        .setup(move |app| {
            let workflows_root = app.path().app_data_dir()?.join("workflows");
            let clock: Arc<dyn Clock> = Arc::new(SystemClock);
            let store = Arc::new(JsonWorkflowStore::new(workflows_root, clock.clone()));
            // The real macOS capture adapter: a ListenOnly event tap,
            // pre-buffered per-display streams, AX resolution, and the
            // bounded screenshot queue behind the one trait boundary.
            let factory: PipelineFactory =
                Box::new(|| Box::new(capture::MacosCapturePipeline::new()));
            let coordinator = Arc::new(RecordingCoordinator::new(
                store,
                permission_gate.clone(),
                factory,
                clock,
            ));
            app.manage(commands::RecorderState(coordinator));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_permissions,
            request_permission,
            commands::start_recording,
            commands::stop_recording,
            commands::list_workflows,
            commands::get_workflow,
            commands::reveal_workflow,
            commands::read_screenshot,
            commands::update_step,
            commands::delete_step,
            commands::rename_workflow,
            commands::delete_workflow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

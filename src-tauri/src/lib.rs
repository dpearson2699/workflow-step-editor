mod permissions;

use std::str::FromStr;
use std::sync::Mutex;

use tauri::State;

use permissions::macos::MacosPermissionSource;
use permissions::{PermissionKind, PermissionReport, PermissionService, PermissionStatus};

/// Managed permission state. The mutex serializes all permission
/// operations so concurrent commands cannot violate the request order.
struct PermissionState(Mutex<PermissionService<MacosPermissionSource>>);

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
    tauri::Builder::default()
        .manage(PermissionState(Mutex::new(PermissionService::new(
            MacosPermissionSource,
        ))))
        .invoke_handler(tauri::generate_handler![
            check_permissions,
            request_permission
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

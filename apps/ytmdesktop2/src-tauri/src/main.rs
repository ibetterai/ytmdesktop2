#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod global_shortcut;
mod media_session;
mod notification_tray;
mod plugin_bridge;
mod settings;

use serde::{Deserialize, Serialize};

const CONTRACT_VERSION: u8 = 1;
const SHELL_ID: &str = "ytmdesktop2-tauri-feasibility";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TauriShellInfo {
    contract_version: u8,
    shell_id: &'static str,
    shell_version: &'static str,
}

impl TauriShellInfo {
    const fn current() -> Self {
        Self {
            contract_version: CONTRACT_VERSION,
            shell_id: SHELL_ID,
            shell_version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[tauri::command]
fn tauri_shell_info() -> TauriShellInfo {
    TauriShellInfo::current()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TauriMainWindowState {
    is_maximized: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TauriMainWindowStateError {
    code: TauriMainWindowStateErrorCode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum TauriMainWindowStateErrorCode {
    WindowStateUnavailable,
}

#[tauri::command]
fn tauri_main_window_state(
    window: tauri::WebviewWindow,
) -> Result<TauriMainWindowState, TauriMainWindowStateError> {
    window
        .is_maximized()
        .map(|is_maximized| TauriMainWindowState { is_maximized })
        .map_err(|_| TauriMainWindowStateError {
            code: TauriMainWindowStateErrorCode::WindowStateUnavailable,
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TauriWindowControlRequest {
    action: TauriWindowControlAction,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum TauriWindowControlAction {
    Minimize,
    ToggleMaximize,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "camelCase")]
enum TauriWindowControlResponse {
    Minimize,
    ToggleMaximize {
        #[serde(rename = "isMaximized")]
        is_maximized: bool,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TauriWindowControlError {
    code: TauriWindowControlErrorCode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum TauriWindowControlErrorCode {
    WindowControlFailed,
}

fn toggle_maximize_response(is_maximized: bool) -> TauriWindowControlResponse {
    TauriWindowControlResponse::ToggleMaximize {
        is_maximized: !is_maximized,
    }
}

#[tauri::command]
fn tauri_window_control(
    window: tauri::WebviewWindow,
    request: TauriWindowControlRequest,
) -> Result<TauriWindowControlResponse, TauriWindowControlError> {
    match request.action {
        TauriWindowControlAction::Minimize => window
            .minimize()
            .map(|_| TauriWindowControlResponse::Minimize)
            .map_err(|_| TauriWindowControlError {
                code: TauriWindowControlErrorCode::WindowControlFailed,
            }),
        TauriWindowControlAction::ToggleMaximize => {
            let is_maximized = window.is_maximized().map_err(|_| TauriWindowControlError {
                code: TauriWindowControlErrorCode::WindowControlFailed,
            })?;
            let response = toggle_maximize_response(is_maximized);

            if is_maximized {
                window.unmaximize()
            } else {
                window.maximize()
            }
            .map(|_| response)
            .map_err(|_| TauriWindowControlError {
                code: TauriWindowControlErrorCode::WindowControlFailed,
            })
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tauri_shell_info,
            tauri_main_window_state,
            tauri_window_control,
            media_session::tauri_media_session_update,
            global_shortcut::tauri_global_shortcut_set_registration,
            notification_tray::tauri_notification_present,
            notification_tray::tauri_tray_set_registration,
            plugin_bridge::tauri_plugin_bridge
        ])
        .manage(settings::SettingsSnapshotState::feasibility_defaults())
        .run(tauri::generate_context!())
        .expect("error while running Tauri feasibility shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_info_has_the_expected_values() {
        let info = tauri_shell_info();

        assert_eq!(info.contract_version, 1);
        assert_eq!(info.shell_id, "ytmdesktop2-tauri-feasibility");
        assert_eq!(info.shell_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn shell_info_serializes_to_the_camel_case_contract() {
        let serialized =
            serde_json::to_value(tauri_shell_info()).expect("shell info is serializable");

        assert_eq!(
            serialized,
            serde_json::json!({
                "contractVersion": 1,
                "shellId": "ytmdesktop2-tauri-feasibility",
                "shellVersion": env!("CARGO_PKG_VERSION"),
            })
        );
    }

    #[test]
    fn main_window_state_serializes_to_the_camel_case_contract() {
        let state = serde_json::to_value(TauriMainWindowState { is_maximized: true })
            .expect("main window state is serializable");

        assert_eq!(state, serde_json::json!({ "isMaximized": true }));
    }

    #[test]
    fn main_window_state_error_serializes_without_native_details() {
        let error = serde_json::to_value(TauriMainWindowStateError {
            code: TauriMainWindowStateErrorCode::WindowStateUnavailable,
        })
        .expect("main window state error is serializable");

        assert_eq!(
            error,
            serde_json::json!({ "code": "windowStateUnavailable" })
        );
    }

    #[test]
    fn window_control_success_variants_serialize_to_the_contract() {
        let minimize = serde_json::to_value(TauriWindowControlResponse::Minimize)
            .expect("minimize response is serializable");
        let toggle =
            serde_json::to_value(TauriWindowControlResponse::ToggleMaximize { is_maximized: true })
                .expect("toggle response is serializable");

        assert_eq!(minimize, serde_json::json!({ "action": "minimize" }));
        assert_eq!(
            toggle,
            serde_json::json!({ "action": "toggleMaximize", "isMaximized": true })
        );
    }

    #[test]
    fn window_control_error_serializes_without_native_details() {
        let error = serde_json::to_value(TauriWindowControlError {
            code: TauriWindowControlErrorCode::WindowControlFailed,
        })
        .expect("window control error is serializable");

        assert_eq!(error, serde_json::json!({ "code": "windowControlFailed" }));
    }

    #[test]
    fn toggle_maximize_response_inverts_the_current_state() {
        assert_eq!(
            toggle_maximize_response(false),
            TauriWindowControlResponse::ToggleMaximize { is_maximized: true }
        );
        assert_eq!(
            toggle_maximize_response(true),
            TauriWindowControlResponse::ToggleMaximize {
                is_maximized: false
            }
        );
    }
}

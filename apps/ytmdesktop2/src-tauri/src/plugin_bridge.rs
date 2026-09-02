use crate::settings::{self, SettingsErrorCode, SettingsSnapshot, SettingsSnapshotState};
use serde::Serialize;
use serde_json::Value;

const BRIDGE_CONTRACT_VERSION: u8 = 1;
const MAIN_WEBVIEW_LABEL: &str = "main";
const MAIN_WEBVIEW_ORIGIN: &str = "tauri://localhost";
const SETTINGS_SNAPSHOT_COMMAND: &str = "settings.snapshot";

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginBridgeError {
    code: PluginBridgeErrorCode,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum PluginBridgeErrorCode {
    UnsupportedCommand,
    UnauthorizedCaller,
    InvalidRequest,
    PluginFailed,
}

#[derive(Debug, PartialEq)]
enum PluginBridgeCommand {
    SettingsSnapshot,
}

#[derive(Debug, PartialEq)]
struct PluginBridgeRequest {
    command: PluginBridgeCommand,
    payload: Value,
}

fn bridge_error(code: PluginBridgeErrorCode) -> PluginBridgeError {
    PluginBridgeError { code }
}

fn decode_request(request: Value) -> Result<PluginBridgeRequest, PluginBridgeError> {
    let object = request
        .as_object()
        .ok_or_else(|| bridge_error(PluginBridgeErrorCode::InvalidRequest))?;

    if object.len() != 3
        || object.get("contractVersion") != Some(&Value::from(BRIDGE_CONTRACT_VERSION))
    {
        return Err(bridge_error(PluginBridgeErrorCode::InvalidRequest));
    }

    let command = match object.get("command").and_then(Value::as_str) {
        Some(SETTINGS_SNAPSHOT_COMMAND) => PluginBridgeCommand::SettingsSnapshot,
        Some(_) => return Err(bridge_error(PluginBridgeErrorCode::UnsupportedCommand)),
        None => return Err(bridge_error(PluginBridgeErrorCode::InvalidRequest)),
    };
    let payload = object
        .get("request")
        .filter(|value| value.is_object())
        .cloned()
        .ok_or_else(|| bridge_error(PluginBridgeErrorCode::InvalidRequest))?;

    Ok(PluginBridgeRequest { command, payload })
}

fn is_authorized_caller(label: &str, origin: &str) -> bool {
    label == MAIN_WEBVIEW_LABEL && origin == MAIN_WEBVIEW_ORIGIN
}

fn dispatch_request(
    caller_label: &str,
    caller_origin: &str,
    state: &SettingsSnapshotState,
    request: Value,
) -> Result<SettingsSnapshot, PluginBridgeError> {
    if !is_authorized_caller(caller_label, caller_origin) {
        return Err(bridge_error(PluginBridgeErrorCode::UnauthorizedCaller));
    }

    let request = decode_request(request)?;
    match request.command {
        PluginBridgeCommand::SettingsSnapshot => {
            settings::snapshot_from_request(state, request.payload).map_err(|error| {
                match error.code {
                    SettingsErrorCode::InvalidKey => {
                        bridge_error(PluginBridgeErrorCode::InvalidRequest)
                    }
                    SettingsErrorCode::SettingsUnavailable => {
                        bridge_error(PluginBridgeErrorCode::PluginFailed)
                    }
                }
            })
        }
    }
}

fn webview_origin(window: &tauri::WebviewWindow) -> Option<String> {
    window.url().ok().and_then(|url| {
        url.host_str()
            .map(|host| format!("{}://{}", url.scheme(), host))
    })
}

/// The only Tauri command available to the inactive plugin bridge.
///
/// Caller identity comes from Tauri's injected webview, never from the request.
#[tauri::command]
pub fn tauri_plugin_bridge(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SettingsSnapshotState>,
    request: Value,
) -> Result<SettingsSnapshot, PluginBridgeError> {
    let origin = webview_origin(&window).unwrap_or_default();
    dispatch_request(window.label(), &origin, &state, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> Value {
        serde_json::json!({
            "contractVersion": 1,
            "command": SETTINGS_SNAPSHOT_COMMAND,
            "request": {
                "contractVersion": 1,
                "keys": ["app.zoomFactor"],
            },
        })
    }

    #[test]
    fn shared_manifest_matches_the_backend_caller_and_command_contract() {
        let manifest: Value = serde_json::from_str(include_str!("../bridge-manifest.json"))
            .expect("bridge manifest is valid JSON");

        assert_eq!(manifest["contractVersion"], 1);
        assert_eq!(
            manifest["callers"],
            serde_json::json!([{
                "webview": MAIN_WEBVIEW_LABEL,
                "origin": MAIN_WEBVIEW_ORIGIN,
                "capability": "allow-tauri-plugin-bridge",
            }])
        );
        assert_eq!(
            manifest["methods"],
            serde_json::json!([{
                "name": SETTINGS_SNAPSHOT_COMMAND,
                "request": "SettingsSnapshotRequest",
                "response": "SettingsSnapshot",
                "errors": ["unsupportedCommand", "unauthorizedCaller", "invalidRequest", "pluginFailed"],
            }])
        );
    }

    #[test]
    fn dispatches_the_only_allow_listed_bridge_command() {
        let response = dispatch_request(
            "main",
            "tauri://localhost",
            &SettingsSnapshotState::feasibility_defaults(),
            valid_request(),
        )
        .expect("allow-listed command succeeds");

        assert_eq!(
            serde_json::to_value(response).expect("response is serializable"),
            serde_json::json!({
                "contractVersion": 1,
                "values": { "app": { "zoomFactor": 1.0 } },
                "missing": [],
            })
        );
    }

    #[test]
    fn rejects_unknown_malformed_and_unauthorized_requests_with_redacted_codes() {
        let state = SettingsSnapshotState::feasibility_defaults();
        let cases = [
            (
                "main",
                "tauri://localhost",
                serde_json::json!({
                    "contractVersion": 1,
                    "command": "plugins.runAnything",
                    "request": {},
                }),
                PluginBridgeErrorCode::UnsupportedCommand,
            ),
            (
                "main",
                "tauri://localhost",
                serde_json::json!({ "contractVersion": 1, "command": "settings.snapshot" }),
                PluginBridgeErrorCode::InvalidRequest,
            ),
            (
                "remote-youtube",
                "https://music.youtube.com",
                valid_request(),
                PluginBridgeErrorCode::UnauthorizedCaller,
            ),
        ];

        for (label, origin, request, expected_code) in cases {
            let error =
                dispatch_request(label, origin, &state, request).expect_err("request is rejected");
            assert_eq!(error.code, expected_code);
            assert_eq!(
                serde_json::to_value(error).expect("error is serializable"),
                serde_json::json!({ "code": expected_code }),
            );
        }
    }

    #[test]
    fn maps_allow_list_validation_failure_to_invalid_request() {
        let error = dispatch_request(
            "main",
            "tauri://localhost",
            &SettingsSnapshotState::feasibility_defaults(),
            serde_json::json!({
                "contractVersion": 1,
                "command": "settings.snapshot",
                "request": { "contractVersion": 1, "keys": ["plugins.secret"] },
            }),
        )
        .expect_err("unknown setting is rejected");

        assert_eq!(
            serde_json::to_value(error).expect("error is serializable"),
            serde_json::json!({ "code": "invalidRequest" }),
        );
    }
}

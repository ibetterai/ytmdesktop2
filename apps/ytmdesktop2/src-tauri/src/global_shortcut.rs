use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GLOBAL_SHORTCUT_CONTRACT_VERSION: u8 = 1;

const MAIN_WEBVIEW_LABEL: &str = "main";
// Tauri 2.11.5 serves bundled assets through the custom `tauri` protocol on most
// platforms and through these Wry-compatible localhost origins on Windows.
const MAIN_WEBVIEW_ORIGINS: [&str; 3] = [
    "tauri://localhost",
    "http://tauri.localhost",
    "https://tauri.localhost",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GlobalShortcutRegistrationRequest {
    contract_version: u8,
    enabled: bool,
}

/// Fixed defaults match the legacy provider, but callers select no accelerator or action.
/// That keeps arbitrary operating-system shortcut registration outside this bridge.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum GlobalShortcutId {
    PreviousTrack,
    NextTrack,
    TogglePlayback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GlobalShortcutDefinition {
    id: GlobalShortcutId,
    accelerator: &'static str,
}

const DEFAULT_SHORTCUTS: [GlobalShortcutDefinition; 3] = [
    GlobalShortcutDefinition {
        id: GlobalShortcutId::PreviousTrack,
        accelerator: "Shift+Alt+Left",
    },
    GlobalShortcutDefinition {
        id: GlobalShortcutId::NextTrack,
        accelerator: "Shift+Alt+Right",
    },
    GlobalShortcutDefinition {
        id: GlobalShortcutId::TogglePlayback,
        accelerator: "Shift+Alt+Space",
    },
];

/// The event a future native registration backend may emit to the sole host webview.
/// It contains a typed action only; it does not disclose a native shortcut handle.
#[derive(Debug, PartialEq, Serialize)]
#[allow(dead_code)] // Declared now for the native backend intentionally absent from this slice.
#[serde(rename_all = "camelCase")]
pub struct GlobalShortcutTriggeredEvent {
    contract_version: u8,
    shortcut: GlobalShortcutId,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GlobalShortcutRegistrationResult {
    Ok {
        contract_version: u8,
        enabled: bool,
    },
    Error {
        contract_version: u8,
        code: GlobalShortcutErrorCode,
    },
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GlobalShortcutErrorCode {
    InvalidRequest,
    UnauthorizedCaller,
    Unsupported,
    Unavailable,
    OperationFailed,
}

#[derive(Debug)]
#[allow(dead_code)] // Only `Unavailable` is reachable while the feasibility shell has no backend.
enum GlobalShortcutBackendError {
    Unsupported,
    Unavailable,
    OperationFailed,
}

fn error(code: GlobalShortcutErrorCode) -> GlobalShortcutRegistrationResult {
    GlobalShortcutRegistrationResult::Error {
        contract_version: GLOBAL_SHORTCUT_CONTRACT_VERSION,
        code,
    }
}

fn is_authorized_caller(label: &str, origin: &str) -> bool {
    label == MAIN_WEBVIEW_LABEL && MAIN_WEBVIEW_ORIGINS.contains(&origin)
}

fn decode_request(
    request: Value,
) -> Result<GlobalShortcutRegistrationRequest, GlobalShortcutErrorCode> {
    let request: GlobalShortcutRegistrationRequest =
        serde_json::from_value(request).map_err(|_| GlobalShortcutErrorCode::InvalidRequest)?;

    if request.contract_version != GLOBAL_SHORTCUT_CONTRACT_VERSION {
        return Err(GlobalShortcutErrorCode::InvalidRequest);
    }

    Ok(request)
}

fn map_backend_error(
    backend_error: GlobalShortcutBackendError,
) -> GlobalShortcutRegistrationResult {
    let code = match backend_error {
        GlobalShortcutBackendError::Unsupported => GlobalShortcutErrorCode::Unsupported,
        GlobalShortcutBackendError::Unavailable => GlobalShortcutErrorCode::Unavailable,
        GlobalShortcutBackendError::OperationFailed => GlobalShortcutErrorCode::OperationFailed,
    };
    error(code)
}

/// The feasibility shell deliberately has no global-shortcut plugin or OS registration path.
/// The inactive result proves the registration contract without creating a production path.
fn set_platform_registration(_enabled: bool) -> Result<(), GlobalShortcutBackendError> {
    let _fixed_defaults = DEFAULT_SHORTCUTS;
    Err(GlobalShortcutBackendError::Unavailable)
}

fn dispatch_registration(
    caller_label: &str,
    caller_origin: &str,
    request: Value,
) -> GlobalShortcutRegistrationResult {
    if !is_authorized_caller(caller_label, caller_origin) {
        return error(GlobalShortcutErrorCode::UnauthorizedCaller);
    }

    let request = match decode_request(request) {
        Ok(request) => request,
        Err(code) => return error(code),
    };

    set_platform_registration(request.enabled)
        .map(|_| GlobalShortcutRegistrationResult::Ok {
            contract_version: GLOBAL_SHORTCUT_CONTRACT_VERSION,
            enabled: request.enabled,
        })
        .unwrap_or_else(map_backend_error)
}

fn origin_from_url(url: &tauri::Url) -> Option<String> {
    let host = url.host_str()?;
    let origin = match url.port() {
        Some(port) => format!("{}://{host}:{port}", url.scheme()),
        None => format!("{}://{host}", url.scheme()),
    };
    Some(origin)
}

fn webview_origin(window: &tauri::WebviewWindow) -> Option<String> {
    window.url().ok().and_then(|url| origin_from_url(&url))
}

/// The sole registration-state command in the inactive feasibility shell.
/// Caller identity is derived from Tauri's injected webview, never request data.
#[tauri::command]
pub fn tauri_global_shortcut_set_registration(
    window: tauri::WebviewWindow,
    request: Value,
) -> GlobalShortcutRegistrationResult {
    let origin = webview_origin(&window).unwrap_or_default();
    dispatch_registration(window.label(), &origin, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request(enabled: bool) -> Value {
        serde_json::json!({ "contractVersion": 1, "enabled": enabled })
    }

    #[test]
    fn manifest_declares_the_versioned_registration_command_event_and_only_authorized_caller() {
        let manifest: Value =
            serde_json::from_str(include_str!("../global-shortcut-manifest.json"))
                .expect("global shortcut manifest is valid JSON");

        assert_eq!(manifest["contractVersion"], 1);
        assert_eq!(
            manifest["callers"],
            serde_json::json!([{
                "webview": MAIN_WEBVIEW_LABEL,
                "origins": MAIN_WEBVIEW_ORIGINS,
                "capability": "allow-tauri-global-shortcut-set-registration",
            }])
        );
        assert_eq!(
            manifest["methods"],
            serde_json::json!([{
                "name": "globalShortcut.setRegistration",
                "request": "GlobalShortcutRegistrationRequest",
                "response": "GlobalShortcutRegistrationResult",
                "errors": ["invalidRequest", "unauthorizedCaller", "unsupported", "unavailable", "operationFailed"],
            }])
        );
        assert_eq!(
            manifest["events"],
            serde_json::json!([{
                "name": "globalShortcut.triggered",
                "payload": "GlobalShortcutTriggeredEvent",
                "callers": ["main"],
            }])
        );
    }

    #[test]
    fn maps_only_the_legacy_fixed_shortcuts_and_serializes_typed_events() {
        assert_eq!(
            DEFAULT_SHORTCUTS,
            [
                GlobalShortcutDefinition {
                    id: GlobalShortcutId::PreviousTrack,
                    accelerator: "Shift+Alt+Left",
                },
                GlobalShortcutDefinition {
                    id: GlobalShortcutId::NextTrack,
                    accelerator: "Shift+Alt+Right",
                },
                GlobalShortcutDefinition {
                    id: GlobalShortcutId::TogglePlayback,
                    accelerator: "Shift+Alt+Space",
                },
            ]
        );
        let event = GlobalShortcutTriggeredEvent {
            contract_version: 1,
            shortcut: GlobalShortcutId::NextTrack,
        };
        assert_eq!(
            serde_json::to_value(event).expect("event is serializable"),
            serde_json::json!({ "contractVersion": 1, "shortcut": "nextTrack" })
        );
    }

    #[test]
    fn accepts_only_the_versioned_enabled_flag_without_accelerator_input() {
        assert!(decode_request(valid_request(true)).is_ok());

        for request in [
            serde_json::json!({ "contractVersion": 2, "enabled": true }),
            serde_json::json!({ "contractVersion": 1, "enabled": "true" }),
            serde_json::json!({
                "contractVersion": 1,
                "enabled": true,
                "accelerator": "CmdOrCtrl+Shift+P"
            }),
        ] {
            assert!(matches!(
                decode_request(request),
                Err(GlobalShortcutErrorCode::InvalidRequest)
            ));
        }
    }

    #[test]
    fn rejects_untrusted_callers_and_returns_only_stable_error_codes() {
        assert_eq!(
            dispatch_registration(
                "remote-youtube",
                "https://music.youtube.com",
                valid_request(true)
            ),
            error(GlobalShortcutErrorCode::UnauthorizedCaller)
        );
        assert_eq!(
            dispatch_registration("main", "tauri://localhost", serde_json::json!({})),
            error(GlobalShortcutErrorCode::InvalidRequest)
        );
    }

    #[test]
    fn accepts_each_platform_bundled_origin_and_keeps_the_backend_inactive() {
        for origin in MAIN_WEBVIEW_ORIGINS {
            assert_eq!(
                dispatch_registration("main", origin, valid_request(true)),
                error(GlobalShortcutErrorCode::Unavailable),
                "{origin} must be accepted only for the bundled main webview",
            );
        }
    }

    #[test]
    fn rejects_lookalikes_ports_malformed_origins_and_non_main_callers() {
        for origin in [
            "https://tauri.localhost.evil.example",
            "https://evil.tauri.localhost",
            "http://tauri.localhost:8443",
            "https://tauri.localhost:8443",
            "tauri://localhost.evil",
            "not an origin",
            "",
        ] {
            assert_eq!(
                dispatch_registration("main", origin, valid_request(true)),
                error(GlobalShortcutErrorCode::UnauthorizedCaller),
                "{origin} must not be trusted as a bundled origin",
            );
        }

        for origin in MAIN_WEBVIEW_ORIGINS {
            assert_eq!(
                dispatch_registration("settings", origin, valid_request(true)),
                error(GlobalShortcutErrorCode::UnauthorizedCaller),
                "{origin} must not be accepted from a non-main webview",
            );
        }

        let remote_lookalike = tauri::Url::parse("http://tauri.localhost:8443/index.html")
            .expect("lookalike test URL is valid");
        assert_eq!(
            origin_from_url(&remote_lookalike).as_deref(),
            Some("http://tauri.localhost:8443"),
        );
    }

    #[test]
    fn maps_backend_failures_without_exposing_platform_errors() {
        for (backend_error, code) in [
            (
                GlobalShortcutBackendError::Unsupported,
                GlobalShortcutErrorCode::Unsupported,
            ),
            (
                GlobalShortcutBackendError::Unavailable,
                GlobalShortcutErrorCode::Unavailable,
            ),
            (
                GlobalShortcutBackendError::OperationFailed,
                GlobalShortcutErrorCode::OperationFailed,
            ),
        ] {
            assert_eq!(map_backend_error(backend_error), error(code));
        }

        assert_eq!(
            dispatch_registration("main", "tauri://localhost", valid_request(false)),
            error(GlobalShortcutErrorCode::Unavailable)
        );
    }
}

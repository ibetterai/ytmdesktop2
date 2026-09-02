use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const NOTIFICATION_TRAY_CONTRACT_VERSION: u8 = 1;

const MAIN_WEBVIEW_LABEL: &str = "main";
const MAIN_WEBVIEW_ORIGIN: &str = "tauri://localhost";
const MAX_NOTIFICATION_MESSAGE_LENGTH: usize = 512;
const TRAY_TOOLTIP: &str = "YouTube Music for Desktop";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NotificationPresentRequest {
    contract_version: u8,
    notification: NotificationPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NotificationPayload {
    #[serde(rename = "type")]
    notification_type: NotificationKind,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum NotificationKind {
    Success,
    Info,
    Error,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TrayRegistrationRequest {
    contract_version: u8,
    enabled: bool,
}

/// Fixed tray actions map legacy left-click and “show app” behavior without exposing menu IDs.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum TrayAction {
    TogglePopover,
    ShowMainWindow,
}

/// The event a future native tray backend may emit to the sole host webview.
/// It contains a typed action only and no native tray/menu handle.
#[derive(Debug, PartialEq, Serialize)]
#[allow(dead_code)] // Declared now for the native backend intentionally absent from this slice.
#[serde(rename_all = "camelCase")]
pub struct TrayActivatedEvent {
    contract_version: u8,
    action: TrayAction,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum NotificationTrayResult {
    Ok {
        contract_version: u8,
    },
    Error {
        contract_version: u8,
        code: NotificationTrayErrorCode,
    },
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationTrayErrorCode {
    InvalidRequest,
    UnauthorizedCaller,
    Unsupported,
    Unavailable,
    OperationFailed,
}

#[derive(Debug)]
#[allow(dead_code)] // Only `Unavailable` is reachable while the feasibility shell has no backend.
enum NotificationTrayBackendError {
    Unsupported,
    Unavailable,
    OperationFailed,
}

fn error(code: NotificationTrayErrorCode) -> NotificationTrayResult {
    NotificationTrayResult::Error {
        contract_version: NOTIFICATION_TRAY_CONTRACT_VERSION,
        code,
    }
}

fn is_authorized_caller(label: &str, origin: &str) -> bool {
    label == MAIN_WEBVIEW_LABEL && origin == MAIN_WEBVIEW_ORIGIN
}

fn decode_notification_request(
    request: Value,
) -> Result<NotificationPresentRequest, NotificationTrayErrorCode> {
    let request: NotificationPresentRequest =
        serde_json::from_value(request).map_err(|_| NotificationTrayErrorCode::InvalidRequest)?;

    if request.contract_version != NOTIFICATION_TRAY_CONTRACT_VERSION
        || request.notification.message.is_empty()
        || request.notification.message.chars().count() > MAX_NOTIFICATION_MESSAGE_LENGTH
    {
        return Err(NotificationTrayErrorCode::InvalidRequest);
    }

    Ok(request)
}

fn decode_tray_request(
    request: Value,
) -> Result<TrayRegistrationRequest, NotificationTrayErrorCode> {
    let request: TrayRegistrationRequest =
        serde_json::from_value(request).map_err(|_| NotificationTrayErrorCode::InvalidRequest)?;

    if request.contract_version != NOTIFICATION_TRAY_CONTRACT_VERSION {
        return Err(NotificationTrayErrorCode::InvalidRequest);
    }

    Ok(request)
}

fn map_backend_error(backend_error: NotificationTrayBackendError) -> NotificationTrayResult {
    let code = match backend_error {
        NotificationTrayBackendError::Unsupported => NotificationTrayErrorCode::Unsupported,
        NotificationTrayBackendError::Unavailable => NotificationTrayErrorCode::Unavailable,
        NotificationTrayBackendError::OperationFailed => NotificationTrayErrorCode::OperationFailed,
    };
    error(code)
}

/// The feasibility shell deliberately has no native notification backend.
fn present_platform_notification(
    notification: NotificationPayload,
) -> Result<(), NotificationTrayBackendError> {
    let _notification_type = notification.notification_type;
    Err(NotificationTrayBackendError::Unavailable)
}

/// The feasibility shell deliberately has no native tray backend or context menu.
fn set_platform_tray_registration(_enabled: bool) -> Result<(), NotificationTrayBackendError> {
    let _fixed_tooltip = TRAY_TOOLTIP;
    let _fixed_actions = [TrayAction::TogglePopover, TrayAction::ShowMainWindow];
    Err(NotificationTrayBackendError::Unavailable)
}

fn dispatch_notification(
    caller_label: &str,
    caller_origin: &str,
    request: Value,
) -> NotificationTrayResult {
    if !is_authorized_caller(caller_label, caller_origin) {
        return error(NotificationTrayErrorCode::UnauthorizedCaller);
    }

    let request = match decode_notification_request(request) {
        Ok(request) => request,
        Err(code) => return error(code),
    };

    present_platform_notification(request.notification)
        .map(|_| NotificationTrayResult::Ok {
            contract_version: NOTIFICATION_TRAY_CONTRACT_VERSION,
        })
        .unwrap_or_else(map_backend_error)
}

fn dispatch_tray_registration(
    caller_label: &str,
    caller_origin: &str,
    request: Value,
) -> NotificationTrayResult {
    if !is_authorized_caller(caller_label, caller_origin) {
        return error(NotificationTrayErrorCode::UnauthorizedCaller);
    }

    let request = match decode_tray_request(request) {
        Ok(request) => request,
        Err(code) => return error(code),
    };

    set_platform_tray_registration(request.enabled)
        .map(|_| NotificationTrayResult::Ok {
            contract_version: NOTIFICATION_TRAY_CONTRACT_VERSION,
        })
        .unwrap_or_else(map_backend_error)
}

fn webview_origin(window: &tauri::WebviewWindow) -> Option<String> {
    window.url().ok().and_then(|url| {
        url.host_str()
            .map(|host| format!("{}://{}", url.scheme(), host))
    })
}

/// The sole notification command in the inactive feasibility shell.
/// Caller identity is derived from Tauri's injected webview, never request data.
#[tauri::command]
pub fn tauri_notification_present(
    window: tauri::WebviewWindow,
    request: Value,
) -> NotificationTrayResult {
    let origin = webview_origin(&window).unwrap_or_default();
    dispatch_notification(window.label(), &origin, request)
}

/// The sole tray-registration command in the inactive feasibility shell.
/// The request cannot select a tray icon, menu item, action, or window label.
#[tauri::command]
pub fn tauri_tray_set_registration(
    window: tauri::WebviewWindow,
    request: Value,
) -> NotificationTrayResult {
    let origin = webview_origin(&window).unwrap_or_default();
    dispatch_tray_registration(window.label(), &origin, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_notification_request() -> Value {
        serde_json::json!({
            "contractVersion": 1,
            "notification": { "type": "success", "message": "Added to queue" }
        })
    }

    fn valid_tray_request(enabled: bool) -> Value {
        serde_json::json!({ "contractVersion": 1, "enabled": enabled })
    }

    #[test]
    fn manifest_declares_versioned_commands_event_and_only_authorized_caller() {
        let manifest: Value =
            serde_json::from_str(include_str!("../notification-tray-manifest.json"))
                .expect("notification and tray manifest is valid JSON");

        assert_eq!(manifest["contractVersion"], 1);
        assert_eq!(
            manifest["callers"],
            serde_json::json!([{
                "webview": MAIN_WEBVIEW_LABEL,
                "origin": MAIN_WEBVIEW_ORIGIN,
                "capabilities": [
                    "allow-tauri-notification-present",
                    "allow-tauri-tray-set-registration",
                ],
            }])
        );
        assert_eq!(
            manifest["methods"],
            serde_json::json!([
                {
                    "name": "notification.present",
                    "request": "NotificationPresentRequest",
                    "response": "NotificationTrayResult",
                    "errors": ["invalidRequest", "unauthorizedCaller", "unsupported", "unavailable", "operationFailed"],
                },
                {
                    "name": "tray.setRegistration",
                    "request": "TrayRegistrationRequest",
                    "response": "NotificationTrayResult",
                    "errors": ["invalidRequest", "unauthorizedCaller", "unsupported", "unavailable", "operationFailed"],
                },
            ])
        );
        assert_eq!(
            manifest["events"],
            serde_json::json!([{
                "name": "tray.activated",
                "payload": "TrayActivatedEvent",
                "callers": ["main"],
            }])
        );
    }

    #[test]
    fn preserves_the_bounded_toast_shape_and_serializes_fixed_tray_actions() {
        assert!(decode_notification_request(valid_notification_request()).is_ok());
        assert_eq!(TRAY_TOOLTIP, "YouTube Music for Desktop");
        let event = TrayActivatedEvent {
            contract_version: 1,
            action: TrayAction::TogglePopover,
        };
        assert_eq!(
            serde_json::to_value(event).expect("event is serializable"),
            serde_json::json!({ "contractVersion": 1, "action": "togglePopover" })
        );
    }

    #[test]
    fn rejects_unbounded_or_untrusted_notification_and_tray_input() {
        for request in [
            serde_json::json!({
                "contractVersion": 1,
                "notification": { "type": "info", "message": "" }
            }),
            serde_json::json!({
                "contractVersion": 1,
                "notification": { "type": "error", "message": "message", "url": "https://example.test" }
            }),
        ] {
            assert!(matches!(
                decode_notification_request(request),
                Err(NotificationTrayErrorCode::InvalidRequest)
            ));
        }
        let oversized = serde_json::json!({
            "contractVersion": 1,
            "notification": { "type": "info", "message": "x".repeat(MAX_NOTIFICATION_MESSAGE_LENGTH + 1) }
        });
        assert!(matches!(
            decode_notification_request(oversized),
            Err(NotificationTrayErrorCode::InvalidRequest)
        ));
        for request in [
            serde_json::json!({ "contractVersion": 2, "enabled": true }),
            serde_json::json!({ "contractVersion": 1, "enabled": true, "window": "other" }),
        ] {
            assert!(matches!(
                decode_tray_request(request),
                Err(NotificationTrayErrorCode::InvalidRequest)
            ));
        }
    }

    #[test]
    fn rejects_untrusted_callers_and_returns_only_stable_error_codes() {
        assert_eq!(
            dispatch_notification(
                "remote-youtube",
                "https://music.youtube.com",
                valid_notification_request()
            ),
            error(NotificationTrayErrorCode::UnauthorizedCaller)
        );
        assert_eq!(
            dispatch_tray_registration("main", "tauri://localhost", serde_json::json!({})),
            error(NotificationTrayErrorCode::InvalidRequest)
        );
    }

    #[test]
    fn maps_backend_failures_without_exposing_platform_errors() {
        for (backend_error, code) in [
            (
                NotificationTrayBackendError::Unsupported,
                NotificationTrayErrorCode::Unsupported,
            ),
            (
                NotificationTrayBackendError::Unavailable,
                NotificationTrayErrorCode::Unavailable,
            ),
            (
                NotificationTrayBackendError::OperationFailed,
                NotificationTrayErrorCode::OperationFailed,
            ),
        ] {
            assert_eq!(map_backend_error(backend_error), error(code));
        }
        assert_eq!(
            dispatch_notification("main", "tauri://localhost", valid_notification_request()),
            error(NotificationTrayErrorCode::Unavailable)
        );
        assert_eq!(
            dispatch_tray_registration("main", "tauri://localhost", valid_tray_request(true)),
            error(NotificationTrayErrorCode::Unavailable)
        );
    }
}

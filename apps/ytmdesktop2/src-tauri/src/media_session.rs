use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MEDIA_SESSION_CONTRACT_VERSION: u8 = 1;

const MAIN_WEBVIEW_LABEL: &str = "main";
// Tauri 2.11.5 serves bundled assets through the custom `tauri` protocol on most
// platforms and through these Wry-compatible localhost origins on Windows/Android.
const MAIN_WEBVIEW_ORIGINS: [&str; 3] = [
    "tauri://localhost",
    "http://tauri.localhost",
    "https://tauri.localhost",
];
const MAX_TEXT_LENGTH: usize = 512;
const MAX_TRACK_ID_LENGTH: usize = 128;
const MAX_POSITION_MILLIS: u64 = 86_400_000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MediaSessionStateRequest {
    contract_version: u8,
    state: MediaSessionState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MediaSessionState {
    playback: MediaSessionPlayback,
    track: Option<MediaSessionTrack>,
    position_millis: Option<u64>,
    duration_millis: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum MediaSessionPlayback {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MediaSessionTrack {
    id: String,
    title: String,
    artist: Option<String>,
    album: Option<String>,
}

/// The typed event a future native media backend may emit to the sole host webview.
/// It contains a media action only; no native handle, URL, or platform error crosses the boundary.
#[derive(Debug, PartialEq, Serialize)]
#[allow(dead_code)] // Declared now for the native backend that is intentionally absent from this slice.
#[serde(rename_all = "camelCase")]
pub struct MediaSessionCommandEvent {
    contract_version: u8,
    command: MediaSessionCommand,
}

#[derive(Debug, PartialEq, Serialize)]
#[allow(dead_code)] // The manifest reserves every supported platform action before backend activation.
#[serde(tag = "action", rename_all = "camelCase")]
pub enum MediaSessionCommand {
    TogglePlayback,
    Play,
    Pause,
    Next,
    Previous,
    SeekAbsolute {
        #[serde(rename = "positionMillis")]
        position_millis: u64,
    },
    SeekRelative {
        #[serde(rename = "offsetMillis")]
        offset_millis: i64,
    },
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MediaSessionUpdateResult {
    Ok {
        contract_version: u8,
    },
    Error {
        contract_version: u8,
        code: MediaSessionErrorCode,
    },
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaSessionErrorCode {
    InvalidRequest,
    UnauthorizedCaller,
    Unsupported,
    Unavailable,
    OperationFailed,
}

#[derive(Debug)]
#[allow(dead_code)] // Only `Unavailable` is reachable while this feasibility seam has no backend.
enum MediaSessionBackendError {
    Unsupported,
    Unavailable,
    OperationFailed,
}

fn error(code: MediaSessionErrorCode) -> MediaSessionUpdateResult {
    MediaSessionUpdateResult::Error {
        contract_version: MEDIA_SESSION_CONTRACT_VERSION,
        code,
    }
}

fn is_authorized_caller(label: &str, origin: &str) -> bool {
    label == MAIN_WEBVIEW_LABEL && MAIN_WEBVIEW_ORIGINS.contains(&origin)
}

fn is_bounded_text(value: &str, max_length: usize) -> bool {
    !value.is_empty() && value.chars().count() <= max_length
}

fn validate_state(state: &MediaSessionState) -> bool {
    let track_is_valid = state.track.as_ref().is_none_or(|track| {
        is_bounded_text(&track.id, MAX_TRACK_ID_LENGTH)
            && is_bounded_text(&track.title, MAX_TEXT_LENGTH)
            && track
                .artist
                .as_deref()
                .is_none_or(|artist| is_bounded_text(artist, MAX_TEXT_LENGTH))
            && track
                .album
                .as_deref()
                .is_none_or(|album| is_bounded_text(album, MAX_TEXT_LENGTH))
    });
    let timeline_is_valid = state
        .position_millis
        .is_none_or(|position| position <= MAX_POSITION_MILLIS)
        && state
            .duration_millis
            .is_none_or(|duration| duration <= MAX_POSITION_MILLIS)
        && state
            .position_millis
            .zip(state.duration_millis)
            .is_none_or(|(position, duration)| position <= duration);
    let playback_is_valid =
        matches!(state.playback, MediaSessionPlayback::Stopped) || state.track.is_some();

    track_is_valid && timeline_is_valid && playback_is_valid
}

fn decode_request(request: Value) -> Result<MediaSessionStateRequest, MediaSessionErrorCode> {
    let request: MediaSessionStateRequest =
        serde_json::from_value(request).map_err(|_| MediaSessionErrorCode::InvalidRequest)?;

    if request.contract_version != MEDIA_SESSION_CONTRACT_VERSION || !validate_state(&request.state)
    {
        return Err(MediaSessionErrorCode::InvalidRequest);
    }

    Ok(request)
}

fn map_backend_error(backend_error: MediaSessionBackendError) -> MediaSessionUpdateResult {
    let code = match backend_error {
        MediaSessionBackendError::Unsupported => MediaSessionErrorCode::Unsupported,
        MediaSessionBackendError::Unavailable => MediaSessionErrorCode::Unavailable,
        MediaSessionBackendError::OperationFailed => MediaSessionErrorCode::OperationFailed,
    };
    error(code)
}

/// The feasibility shell deliberately has no platform media backend. Keeping the adapter
/// unavailable proves the contract and capability without enabling a Tauri production path.
fn update_platform_media_session(
    _state: MediaSessionState,
) -> Result<(), MediaSessionBackendError> {
    Err(MediaSessionBackendError::Unavailable)
}

fn dispatch_update(
    caller_label: &str,
    caller_origin: &str,
    request: Value,
) -> MediaSessionUpdateResult {
    if !is_authorized_caller(caller_label, caller_origin) {
        return error(MediaSessionErrorCode::UnauthorizedCaller);
    }

    let request = match decode_request(request) {
        Ok(request) => request,
        Err(code) => return error(code),
    };

    update_platform_media_session(request.state)
        .map(|_| MediaSessionUpdateResult::Ok {
            contract_version: MEDIA_SESSION_CONTRACT_VERSION,
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

/// The sole state-changing media-session command in the inactive feasibility shell.
/// Caller identity is derived from Tauri's injected webview, never request data.
#[tauri::command]
pub fn tauri_media_session_update(
    window: tauri::WebviewWindow,
    request: Value,
) -> MediaSessionUpdateResult {
    let origin = webview_origin(&window).unwrap_or_default();
    dispatch_update(window.label(), &origin, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> Value {
        serde_json::json!({
            "contractVersion": 1,
            "state": {
                "playback": "playing",
                "track": {
                    "id": "dQw4w9WgXcQ",
                    "title": "Never Gonna Give You Up",
                    "artist": "Rick Astley",
                    "album": "Whenever You Need Somebody"
                },
                "positionMillis": 1_000,
                "durationMillis": 213_000
            }
        })
    }

    #[test]
    fn manifest_declares_the_versioned_command_event_and_only_authorized_caller() {
        let manifest: Value = serde_json::from_str(include_str!("../media-session-manifest.json"))
            .expect("media session manifest is valid JSON");

        assert_eq!(manifest["contractVersion"], 1);
        assert_eq!(
            manifest["callers"],
            serde_json::json!([{
                "webview": MAIN_WEBVIEW_LABEL,
                "origins": MAIN_WEBVIEW_ORIGINS,
                "capability": "allow-tauri-media-session-update",
            }])
        );
        assert_eq!(
            manifest["methods"],
            serde_json::json!([{
                "name": "mediaSession.updateState",
                "request": "MediaSessionStateRequest",
                "response": "MediaSessionUpdateResult",
                "errors": ["invalidRequest", "unauthorizedCaller", "unsupported", "unavailable", "operationFailed"],
            }])
        );
        assert_eq!(
            manifest["events"],
            serde_json::json!([{
                "name": "mediaSession.command",
                "payload": "MediaSessionCommandEvent",
                "callers": ["main"],
            }])
        );
    }

    #[test]
    fn serializes_media_commands_without_native_handles() {
        let event = MediaSessionCommandEvent {
            contract_version: 1,
            command: MediaSessionCommand::SeekRelative {
                offset_millis: -15_000,
            },
        };

        assert_eq!(
            serde_json::to_value(event).expect("event is serializable"),
            serde_json::json!({
                "contractVersion": 1,
                "command": { "action": "seekRelative", "offsetMillis": -15_000 },
            })
        );
    }

    #[test]
    fn accepts_only_bounded_and_versioned_media_state() {
        assert!(decode_request(valid_request()).is_ok());

        for request in [
            serde_json::json!({ "contractVersion": 2, "state": { "playback": "stopped" } }),
            serde_json::json!({
                "contractVersion": 1,
                "state": { "playback": "playing", "track": null }
            }),
            serde_json::json!({
                "contractVersion": 1,
                "state": {
                    "playback": "paused",
                    "track": { "id": "id", "title": "title" },
                    "positionMillis": 2,
                    "durationMillis": 1
                }
            }),
        ] {
            assert!(matches!(
                decode_request(request),
                Err(MediaSessionErrorCode::InvalidRequest)
            ));
        }
    }

    #[test]
    fn rejects_untrusted_callers_and_returns_only_stable_error_codes() {
        assert_eq!(
            dispatch_update(
                "remote-youtube",
                "https://music.youtube.com",
                valid_request()
            ),
            error(MediaSessionErrorCode::UnauthorizedCaller)
        );
        assert_eq!(
            dispatch_update("main", "tauri://localhost", serde_json::json!({})),
            error(MediaSessionErrorCode::InvalidRequest)
        );
    }

    #[test]
    fn accepts_each_platform_bundled_origin_and_keeps_the_backend_inactive() {
        for origin in [
            "tauri://localhost",
            "http://tauri.localhost",
            "https://tauri.localhost",
        ] {
            assert_eq!(
                dispatch_update("main", origin, valid_request()),
                error(MediaSessionErrorCode::Unavailable),
                "{origin} must be accepted only for the bundled main webview",
            );
        }
    }

    #[test]
    fn rejects_remote_lookalikes_of_the_bundled_tauri_origin() {
        for origin in [
            "https://tauri.localhost.evil.example",
            "https://evil.tauri.localhost",
            "http://tauri.localhost:8443",
            "tauri://localhost.evil",
        ] {
            assert_eq!(
                dispatch_update("main", origin, valid_request()),
                error(MediaSessionErrorCode::UnauthorizedCaller),
                "{origin} must not be trusted as a bundled origin",
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
                MediaSessionBackendError::Unsupported,
                MediaSessionErrorCode::Unsupported,
            ),
            (
                MediaSessionBackendError::Unavailable,
                MediaSessionErrorCode::Unavailable,
            ),
            (
                MediaSessionBackendError::OperationFailed,
                MediaSessionErrorCode::OperationFailed,
            ),
        ] {
            assert_eq!(map_backend_error(backend_error), error(code));
        }

        assert_eq!(
            dispatch_update("main", "tauri://localhost", valid_request()),
            error(MediaSessionErrorCode::Unavailable)
        );
    }
}

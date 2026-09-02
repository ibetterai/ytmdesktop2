use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;

pub const SETTINGS_CONTRACT_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum SettingsKey {
    #[serde(rename = "app.zoomFactor")]
    AppZoomFactor,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SettingsSnapshotRequest {
    keys: Vec<SettingsKey>,
    contract_version: u8,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSnapshot {
    contract_version: u8,
    values: SettingsSnapshotValues,
    missing: Vec<SettingsKey>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SettingsSnapshotValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<SettingsSnapshotAppValues>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSnapshotAppValues {
    zoom_factor: f64,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsError {
    pub code: SettingsErrorCode,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SettingsErrorCode {
    InvalidKey,
    SettingsUnavailable,
}

#[derive(Clone, Debug)]
struct SettingsSnapshotStore {
    zoom_factor: Option<f64>,
}

impl SettingsSnapshotStore {
    const fn feasibility_defaults() -> Self {
        Self {
            zoom_factor: Some(1.0),
        }
    }
}

pub struct SettingsSnapshotState {
    store: Mutex<SettingsSnapshotStore>,
}

impl SettingsSnapshotState {
    pub const fn feasibility_defaults() -> Self {
        Self {
            store: Mutex::new(SettingsSnapshotStore::feasibility_defaults()),
        }
    }
}

fn invalid_key() -> SettingsError {
    SettingsError {
        code: SettingsErrorCode::InvalidKey,
    }
}

fn settings_unavailable() -> SettingsError {
    SettingsError {
        code: SettingsErrorCode::SettingsUnavailable,
    }
}

fn decode_request(request: Value) -> Result<SettingsSnapshotRequest, SettingsError> {
    let request: SettingsSnapshotRequest =
        serde_json::from_value(request).map_err(|_| invalid_key())?;

    if request.contract_version != SETTINGS_CONTRACT_VERSION
        || request.keys.windows(2).any(|keys| keys[0] == keys[1])
    {
        return Err(invalid_key());
    }

    Ok(request)
}

fn snapshot(store: &SettingsSnapshotStore, request: SettingsSnapshotRequest) -> SettingsSnapshot {
    let has_zoom_factor = request.keys.contains(&SettingsKey::AppZoomFactor);
    let zoom_factor = has_zoom_factor.then_some(store.zoom_factor).flatten();
    let missing = has_zoom_factor
        .then_some(store.zoom_factor.is_none())
        .filter(|missing| *missing)
        .map(|_| vec![SettingsKey::AppZoomFactor])
        .unwrap_or_default();

    SettingsSnapshot {
        contract_version: SETTINGS_CONTRACT_VERSION,
        values: SettingsSnapshotValues {
            app: zoom_factor.map(|zoom_factor| SettingsSnapshotAppValues { zoom_factor }),
        },
        missing,
    }
}

fn read_store(state: &SettingsSnapshotState) -> Result<SettingsSnapshotStore, SettingsError> {
    state
        .store
        .lock()
        .map(|store| store.clone())
        .map_err(|_| settings_unavailable())
}

pub fn snapshot_from_request(
    state: &SettingsSnapshotState,
    request: Value,
) -> Result<SettingsSnapshot, SettingsError> {
    let request = decode_request(request)?;
    let store = read_store(state)?;

    Ok(snapshot(&store, request))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshots_only_the_requested_allow_listed_key() {
        let request = decode_request(serde_json::json!({
            "contractVersion": 1,
            "keys": ["app.zoomFactor"],
        }))
        .expect("allow-listed request is valid");

        assert_eq!(
            snapshot(&SettingsSnapshotStore::feasibility_defaults(), request),
            SettingsSnapshot {
                contract_version: 1,
                values: SettingsSnapshotValues {
                    app: Some(SettingsSnapshotAppValues { zoom_factor: 1.0 }),
                },
                missing: vec![],
            }
        );
    }

    #[test]
    fn rejects_unknown_or_malformed_keys_with_the_redacted_error() {
        for request in [
            serde_json::json!({ "contractVersion": 1, "keys": ["api.authRequired"] }),
            serde_json::json!({ "contractVersion": 1, "keys": "app.zoomFactor" }),
            serde_json::json!({ "contractVersion": 2, "keys": ["app.zoomFactor"] }),
        ] {
            assert!(matches!(decode_request(request), Err(error) if error == invalid_key()));
        }
    }

    #[test]
    fn snapshot_serializes_to_the_versioned_contract() {
        let response = snapshot(
            &SettingsSnapshotStore::feasibility_defaults(),
            decode_request(serde_json::json!({
                "contractVersion": 1,
                "keys": ["app.zoomFactor"],
            }))
            .expect("allow-listed request is valid"),
        );

        assert_eq!(
            serde_json::to_value(response).expect("snapshot is serializable"),
            serde_json::json!({
                "contractVersion": 1,
                "values": { "app": { "zoomFactor": 1.0 } },
                "missing": [],
            })
        );
    }

    #[test]
    fn reports_requested_but_unavailable_keys_as_missing() {
        let response = snapshot(
            &SettingsSnapshotStore { zoom_factor: None },
            decode_request(serde_json::json!({
                "contractVersion": 1,
                "keys": ["app.zoomFactor"],
            }))
            .expect("allow-listed request is valid"),
        );

        assert_eq!(
            serde_json::to_value(response).expect("snapshot is serializable"),
            serde_json::json!({
                "contractVersion": 1,
                "values": {},
                "missing": ["app.zoomFactor"],
            })
        );
    }

    #[test]
    fn storage_failures_are_redacted_as_settings_unavailable() {
        let state = SettingsSnapshotState::feasibility_defaults();
        let _ = std::panic::catch_unwind(|| {
            let _store = state.store.lock().expect("lock starts usable");
            panic!("simulate an unavailable settings store");
        });

        assert_eq!(
            serde_json::to_value(read_store(&state).expect_err("poisoned store is unavailable"))
                .expect("error is serializable"),
            serde_json::json!({ "code": "settingsUnavailable" })
        );
    }
}

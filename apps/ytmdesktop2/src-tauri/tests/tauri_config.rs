use std::{fs, path::PathBuf};

#[test]
fn defines_a_bundled_mac_app_with_no_static_local_window() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = fs::read_to_string(manifest_dir.join("tauri.conf.json"))
        .expect("Tauri app has a configuration");
    let config: serde_json::Value =
        serde_json::from_str(&config).expect("Tauri configuration is valid JSON");

    assert_eq!(config["build"]["frontendDist"], "ui");
    assert_eq!(config["productName"], "YouTube Music Tauri");
    assert_eq!(config["identifier"], "com.ibetterai.ytmdesktop2.tauri");
    assert_eq!(config["bundle"]["active"], true);
    assert_eq!(config["bundle"]["targets"], serde_json::json!(["app"]));

    let windows = config["app"]["windows"]
        .as_array()
        .expect("Tauri configuration declares windows");
    assert!(windows.is_empty());
}

#[test]
fn gives_the_remote_webview_no_native_permissions() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let capability = fs::read_to_string(manifest_dir.join("capabilities/main.json"))
        .expect("Tauri app has a main-window capability");
    let capability: serde_json::Value =
        serde_json::from_str(&capability).expect("capability configuration is valid JSON");
    assert_eq!(capability["windows"], serde_json::json!(["main"]));
    assert_eq!(capability["permissions"], serde_json::json!([]));
}

#[test]
fn global_shortcut_manifest_scopes_windows_bundled_origins_to_main_only() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(manifest_dir.join("global-shortcut-manifest.json"))
        .expect("global shortcut manifest exists");
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest).expect("global shortcut manifest is valid JSON");

    assert_eq!(
        manifest["callers"],
        serde_json::json!([{
            "webview": "main",
            "origins": [
                "tauri://localhost",
                "http://tauri.localhost",
                "https://tauri.localhost",
            ],
            "capability": "allow-tauri-global-shortcut-set-registration",
        }])
    );
}

#[test]
fn notification_tray_manifest_scopes_windows_bundled_origins_to_main_only() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(manifest_dir.join("notification-tray-manifest.json"))
        .expect("notification and tray manifest exists");
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest).expect("notification and tray manifest is valid JSON");

    assert_eq!(
        manifest["callers"],
        serde_json::json!([{
            "webview": "main",
            "origins": [
                "tauri://localhost",
                "http://tauri.localhost",
                "https://tauri.localhost",
            ],
            "capabilities": [
                "allow-tauri-notification-present",
                "allow-tauri-tray-set-registration",
            ],
        }])
    );
}

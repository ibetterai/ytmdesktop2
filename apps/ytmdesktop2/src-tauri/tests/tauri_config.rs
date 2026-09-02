use std::{fs, path::PathBuf};

#[test]
fn defines_an_unsigned_blank_host_window() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = fs::read_to_string(manifest_dir.join("tauri.conf.json"))
        .expect("feasibility seam has a Tauri configuration");
    let config: serde_json::Value =
        serde_json::from_str(&config).expect("Tauri configuration is valid JSON");

    assert_eq!(config["build"]["frontendDist"], "ui");
    assert_eq!(config["bundle"]["active"], false);

    let windows = config["app"]["windows"]
        .as_array()
        .expect("Tauri configuration declares windows");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0]["label"], "main");
    assert_eq!(windows[0]["title"], "YTMDesktop2 Tauri Feasibility");
}

#[test]
fn grants_only_the_tauri_commands_to_the_main_window() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let capability = fs::read_to_string(manifest_dir.join("capabilities/main.json"))
        .expect("feasibility shell has a main-window capability");
    let capability: serde_json::Value =
        serde_json::from_str(&capability).expect("capability configuration is valid JSON");
    let bridge_manifest = fs::read_to_string(manifest_dir.join("bridge-manifest.json"))
        .expect("bridge manifest exists");
    let bridge_manifest: serde_json::Value =
        serde_json::from_str(&bridge_manifest).expect("bridge manifest is valid JSON");

    assert_eq!(capability["windows"], serde_json::json!(["main"]));
    assert_eq!(
        capability["permissions"],
        serde_json::json!([
            "allow-tauri-shell-info",
            "allow-tauri-window-control",
            "allow-tauri-main-window-state",
            "allow-tauri-media-session-update",
            "allow-tauri-global-shortcut-set-registration",
            "allow-tauri-notification-present",
            "allow-tauri-tray-set-registration",
            "allow-tauri-plugin-bridge"
        ])
    );
    assert!(capability["permissions"]
        .as_array()
        .expect("capability permissions are an array")
        .contains(&bridge_manifest["callers"][0]["capability"]));
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

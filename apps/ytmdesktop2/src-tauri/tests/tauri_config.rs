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
fn grants_only_shell_info_and_window_control_to_the_main_window() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let capability = fs::read_to_string(manifest_dir.join("capabilities/main.json"))
        .expect("feasibility shell has a main-window capability");
    let capability: serde_json::Value =
        serde_json::from_str(&capability).expect("capability configuration is valid JSON");

    assert_eq!(capability["windows"], serde_json::json!(["main"]));
    assert_eq!(
        capability["permissions"],
        serde_json::json!(["allow-tauri-shell-info", "allow-tauri-window-control"])
    );
}

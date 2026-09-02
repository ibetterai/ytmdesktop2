fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "tauri_shell_info",
            "tauri_main_window_state",
            "tauri_window_control",
            "tauri_media_session_update",
            "tauri_plugin_bridge",
        ]),
    ))
    .expect("failed to build Tauri feasibility shell permissions");
}

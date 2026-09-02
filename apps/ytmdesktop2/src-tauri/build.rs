fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["tauri_shell_info"])),
    )
    .expect("failed to build Tauri feasibility shell permissions");
}

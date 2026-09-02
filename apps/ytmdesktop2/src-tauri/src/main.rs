#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

const CONTRACT_VERSION: u8 = 1;
const SHELL_ID: &str = "ytmdesktop2-tauri-feasibility";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TauriShellInfo {
    contract_version: u8,
    shell_id: &'static str,
    shell_version: &'static str,
}

impl TauriShellInfo {
    const fn current() -> Self {
        Self {
            contract_version: CONTRACT_VERSION,
            shell_id: SHELL_ID,
            shell_version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[tauri::command]
fn tauri_shell_info() -> TauriShellInfo {
    TauriShellInfo::current()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![tauri_shell_info])
        .run(tauri::generate_context!())
        .expect("error while running Tauri feasibility shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_info_has_the_expected_values() {
        let info = tauri_shell_info();

        assert_eq!(info.contract_version, 1);
        assert_eq!(info.shell_id, "ytmdesktop2-tauri-feasibility");
        assert_eq!(info.shell_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn shell_info_serializes_to_the_camel_case_contract() {
        let serialized =
            serde_json::to_value(tauri_shell_info()).expect("shell info is serializable");

        assert_eq!(
            serialized,
            serde_json::json!({
                "contractVersion": 1,
                "shellId": "ytmdesktop2-tauri-feasibility",
                "shellVersion": env!("CARGO_PKG_VERSION"),
            })
        );
    }
}

//! Tauri command registration and application setup.

mod commands;
mod core;

pub use self::core::types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::probe::probe_system,
            commands::probe::scan_file,
            commands::convert::convert_files,
            commands::convert::check_existing_files,
            commands::youtube::scan_playlist,
            commands::youtube::download_youtube_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

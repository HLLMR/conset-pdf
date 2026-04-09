//! Tauri v2 application builder.
//!
//! This is the library entry point. `main.rs` calls `run()` as a thin shim
//! so the binary stays clean and the app logic is testable.

pub mod backend_process;
pub mod commands;
pub mod setup;

use backend_process::AppState;
use tauri::{Emitter, Manager, RunEvent, WindowEvent};

use commands::{
    cmd_apply_addendum, cmd_apply_sheet_addendum, cmd_extract, cmd_extract_submittal,
    cmd_index_drawing, cmd_index_submittal, cmd_open_file_dialog, cmd_save_file_dialog,
    cmd_segment, cmd_validate_manifest, cmd_visualize,
};

/// Application entry point — called by `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ----- Tauri v2 plugins -----
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        // ----- Managed state -----
        .manage(AppState::new())
        // ----- Startup probe -----
        .setup(|app| {
            let cli_path = backend_process::backend_cli_path(app.handle());
            if let Err(e) = setup::probe_environment(&cli_path) {
                // Show a blocking dialog before the main window is interactive.
                // The user must acknowledge and quit; the app does not proceed.
                let msg = format!(
                    "Conset PDF could not start:\n\n{e}\n\nThe application will now exit."
                );
                eprintln!("[setup] probe failed: {msg}");
                // In production this would use tauri-plugin-dialog for a native modal.
                // For now we log and continue so development without a release binary
                // is not blocked.
            }
            Ok(())
        })
        // ----- Command handlers -----
        .invoke_handler(tauri::generate_handler![
            cmd_extract,
            cmd_segment,
            cmd_index_drawing,
            cmd_index_submittal,
            cmd_apply_addendum,
            cmd_apply_sheet_addendum,
            cmd_extract_submittal,
            cmd_open_file_dialog,
            cmd_save_file_dialog,
            cmd_validate_manifest,
            cmd_visualize,
        ])
        // ----- Window event: subprocess lifecycle -----
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::WindowEvent {
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                let state = app_handle.state::<AppState>();
                let has_active = {
                    let guard = state.active_child.lock().expect("mutex poisoned");
                    guard.is_some()
                };

                if has_active {
                    // Prevent immediate close; frontend will show confirmation dialog.
                    api.prevent_close();
                    // Emit an event so the frontend can display the confirmation dialog.
                    let _ = app_handle.emit("close-requested-while-processing", ());
                }
                // If no active child, allow close — Tauri exits normally.
            }
        });
}

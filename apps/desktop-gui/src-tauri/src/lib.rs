//! Tauri v2 application builder.
//!
//! This is the library entry point. `main.rs` calls `run()` as a thin shim
//! so the binary stays clean and the app logic is testable.

pub mod backend_process;
pub mod commands;
pub mod setup;

use backend_process::AppState;
use tauri::{Emitter, Manager, RunEvent, WindowEvent};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};

use commands::{
    cmd_apply_addendum, cmd_apply_sheet_addendum, cmd_extract, cmd_extract_submittal,
    cmd_index_drawing, cmd_index_submittal, cmd_open_file_dialog, cmd_save_file_dialog,
    cmd_segment, cmd_validate_manifest, cmd_visualize,
};

/// Builds the typed specta command collection.
///
/// Only commands with fully typed (specta-compatible) return types are included here.
/// Commands returning raw `serde_json::Value` are wired via `generate_handler!` below.
///
/// Shared between the live app and the `gen-bindings` binary.
pub fn specta_builder() -> SpectaBuilder<tauri::Wry> {
    SpectaBuilder::<tauri::Wry>::new().commands(collect_commands![
        cmd_open_file_dialog,
        cmd_save_file_dialog,
        cmd_validate_manifest,
    ])
}

/// Application entry point — called by `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // In dev builds, auto-export TypeScript bindings on boot.
    #[cfg(debug_assertions)]
    {
        let builder = specta_builder();
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("src")
            .join("bindings.ts");
        if let Err(e) = builder.export(specta_typescript::Typescript::default(), &out) {
            eprintln!("[specta] failed to write bindings: {e}");
        }
    }

    tauri::Builder::default()
        // ----- Tauri v2 plugins -----
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        // ----- All command handlers (single handler required by Tauri) -----
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
        // ----- Managed state -----
        .manage(AppState::new())
        // ----- Startup probe -----
        .setup(|app| {
            let cli_path = backend_process::backend_cli_path(app.handle());
            if let Err(e) = setup::probe_environment(&cli_path) {
                let msg = format!(
                    "Conset PDF could not start:\n\n{e}\n\nThe application will now exit."
                );
                eprintln!("[setup] probe failed: {msg}");
            }
            Ok(())
        })
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
                    api.prevent_close();
                    let _ = app_handle.emit("close-requested-while-processing", ());
                }
            }
        });
}

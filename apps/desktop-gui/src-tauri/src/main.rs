// Tauri v2 application entry point.
// All app logic lives in lib.rs; this file is the binary shim only.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    conset_pdf_desktop_gui_tauri::run();
}

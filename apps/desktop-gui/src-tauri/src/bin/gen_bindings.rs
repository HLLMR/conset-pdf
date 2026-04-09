//! Standalone TypeScript bindings generator.
//!
//! Run with:
//!   cargo run -p conset-pdf-desktop-gui-tauri --bin gen-bindings
//!
//! This writes `apps/desktop-gui/src/bindings.ts`, which is the canonical
//! TypeScript type source for all session and contract types. The file is
//! .gitignored and must be regenerated whenever session types change.
//!
//! The `run()` entry-point also auto-generates in `#[cfg(debug_assertions)]`
//! mode, so this binary is mainly a convenience for CI / pre-commit hooks.

fn main() {
    let builder = conset_pdf_desktop_gui_tauri::specta_builder();

    // One level up from src-tauri → apps/desktop-gui/src/bindings.ts
    let out_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("bindings.ts");

    builder
        .export(specta_typescript::Typescript::default(), &out_path)
        .expect("Failed to export TypeScript bindings");

    println!("Wrote TypeScript bindings to {}", out_path.display());
}

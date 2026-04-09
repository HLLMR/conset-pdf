//! Backend process management — spawn `backend-cli` subprocess and capture output.
//!
//! # Contract
//! - Only this module interacts with `std::process::Command`.
//! - `PDFIUM_LIB_PATH` is injected as an env var on every subprocess spawn.
//! - The bundled `backend-cli` binary is resolved via the Tauri resource directory.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::Value;
use tauri::{AppHandle, Manager};

// ---------------------------------------------------------------------------
// AppState — holds the active child process handle for lifecycle management
// ---------------------------------------------------------------------------

/// Shared Tauri application state.
///
/// `active_child` is `Some` while a backend-cli subprocess is running.
/// Protected by a `Mutex` so the `CloseRequested` handler and command completions
/// can safely coordinate without data races.
pub struct AppState {
    pub active_child: Mutex<Option<std::process::Child>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_child: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// Returns the path to the bundled `backend-cli` binary.
///
/// In development (`cargo tauri dev`): falls back to the workspace target/release
/// or target/debug binary alongside the project root.
/// In production: resolves from the Tauri resource directory.
pub fn backend_cli_path(app: &AppHandle) -> PathBuf {
    let binary_name = if cfg!(windows) {
        "backend-cli.exe"
    } else {
        "backend-cli"
    };

    // Production path: bundled resource
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join(binary_name);
        if candidate.exists() {
            return candidate;
        }
    }

    // Development fallback: workspace target/release then target/debug
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir)
        .parent() // src-tauri → desktop-gui
        .and_then(Path::parent) // desktop-gui → apps
        .and_then(Path::parent) // apps → root
        .map(Path::to_owned)
        .unwrap_or_else(|| PathBuf::from("."));

    let release = workspace_root
        .join("target")
        .join("release")
        .join(binary_name);
    if release.exists() {
        return release;
    }

    workspace_root.join("target").join("debug").join(binary_name)
}

/// Returns the resource directory, used to set `PDFIUM_LIB_PATH`.
fn pdfium_lib_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().resource_dir().ok()
}

// ---------------------------------------------------------------------------
// Subprocess execution
// ---------------------------------------------------------------------------

/// Spawns `backend-cli` with the given arguments and returns stdout parsed as JSON.
///
/// Sets `PDFIUM_LIB_PATH` to the resource directory so the bundled PDFium
/// library is found at runtime without requiring PATH manipulation by the user.
///
/// # Errors
/// Returns an `Err(String)` if:
/// - The binary cannot be found or executed
/// - The process exits with a non-zero status
/// - Stdout cannot be parsed as valid JSON
pub fn run_backend(app: &AppHandle, args: &[&str]) -> Result<Value, String> {
    let cli_path = backend_cli_path(app);

    let mut cmd = std::process::Command::new(&cli_path);
    cmd.args(args);

    // Inject PDFium library path
    if let Some(lib_dir) = pdfium_lib_dir(app) {
        cmd.env("PDFIUM_LIB_PATH", lib_dir);
    }

    // Capture both stdout and stderr
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = cmd.output().map_err(|e| {
        format!(
            "Failed to spawn backend-cli at '{}': {e}",
            cli_path.display()
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "backend-cli exited with {}: {stderr}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim())
        .map_err(|e| format!("backend-cli output is not valid JSON: {e}\nOutput: {stdout}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_initialises_with_no_active_child() {
        let state = AppState::new();
        let guard = state.active_child.lock().expect("mutex poisoned");
        assert!(guard.is_none());
    }
}

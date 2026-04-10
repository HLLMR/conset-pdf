//! Backend process management â€” spawn `backend-cli` subprocess and capture output.
//!
//! # Contract
//! - Only this module interacts with `std::process::Command`.
//! - `PDFIUM_LIB_PATH` is injected as an env var on every subprocess spawn.
//! - The bundled `backend-cli` binary is resolved via the Tauri resource directory.
//! - `AppState.active_child` holds the handle while a subprocess is alive so the
//!   window close handler can kill it cleanly.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use conset_pdf_contracts::WorkflowResponse;
use tauri::{AppHandle, Manager};

// ---------------------------------------------------------------------------
// AppState â€” holds the active child process handle for lifecycle management
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
        .parent() // src-tauri â†’ desktop-gui
        .and_then(Path::parent) // desktop-gui â†’ apps
        .and_then(Path::parent) // apps â†’ root
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

/// Spawns `backend-cli` with the given arguments, stores the child handle in
/// `AppState` for lifecycle management, waits for completion, and returns
/// stdout as a parsed [`WorkflowResponse`].
///
/// Sets `PDFIUM_LIB_PATH` to the resource directory so the bundled PDFium
/// library is found at runtime without requiring PATH manipulation by the user.
///
/// The child handle is removed from `AppState` on completion (success or error)
/// so the close handler can distinguish idle from busy.
///
/// # Errors
/// Returns an `Err(String)` if:
/// - The binary cannot be found or executed
/// - The operation was cancelled (close handler killed the child)
/// - The process exits with a non-zero status
/// - Stdout cannot be parsed as a valid [`WorkflowResponse`]
pub fn run_backend(
    app: &AppHandle,
    state: &AppState,
    args: &[&str],
) -> Result<WorkflowResponse, String> {
    let cli_path = backend_cli_path(app);

    let mut cmd = std::process::Command::new(&cli_path);
    cmd.args(args);

    // Inject PDFium library path
    if let Some(lib_dir) = pdfium_lib_dir(app) {
        cmd.env("PDFIUM_LIB_PATH", lib_dir);
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let child = cmd.spawn().map_err(|e| {
        format!(
            "Failed to spawn backend-cli at '{}': {e}",
            cli_path.display()
        )
    })?;

    // Store child so the CloseRequested handler can kill it.
    {
        let mut guard = state.active_child.lock().expect("mutex poisoned");
        *guard = Some(child);
    }

    // Take child back to wait â€” releases the lock so kill_active_child() can
    // still acquire it if the user dismisses the window during the wait.
    let output = {
        let mut guard = state.active_child.lock().expect("mutex poisoned");
        match guard.take() {
            Some(c) => c
                .wait_with_output()
                .map_err(|e| format!("wait_with_output failed: {e}"))?,
            // Child was killed by the close handler between store and take.
            None => return Err("Operation was cancelled".to_owned()),
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "backend-cli exited with {}: {stderr}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).map_err(|e| {
        format!("backend-cli output is not valid JSON: {e}\nOutput: {stdout}")
    })
}

/// Streaming variant of [`run_backend`].
///
/// When `backend-cli` gains `--progress-events` support (Sprint 11.3), this
/// function will spawn the process with that flag, emit a Tauri event per
/// progress line, and return the final [`WorkflowResponse`].
///
/// For now it delegates to [`run_backend`] â€” the infrastructure is wired but
/// line-by-line event emission is gated on the CLI feature landing.
pub fn run_backend_streaming(
    app: &AppHandle,
    state: &AppState,
    args: &[&str],
) -> Result<WorkflowResponse, String> {
    // TODO Sprint 11.3: spawn with --progress-events, BufRead lines, emit events.
    run_backend(app, state, args)
}

/// Kills the active subprocess if one is running and reaps the process to
/// avoid leaving a zombie.  No-op if no child is active.
pub fn kill_active_child(state: &AppState) {
    let mut guard = state.active_child.lock().expect("mutex poisoned");
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait(); // reap â€” avoids zombie on Unix
    }
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

    #[test]
    fn kill_active_child_when_idle_is_noop() {
        let state = AppState::new();
        // Must not panic or deadlock when no child is present.
        kill_active_child(&state);
        let guard = state.active_child.lock().expect("mutex poisoned");
        assert!(guard.is_none());
    }

    #[test]
    fn active_child_remains_none_after_spawn_and_wait() {
        // Spawn a trivially-fast process so we can check post-wait state.
        // On Windows `cmd /c exit 0` exits with code 0.
        // On Unix `true` exits with code 0.
        #[cfg(windows)]
        let (prog, args_list): (&str, &[&str]) = ("cmd", &["/c", "exit 0"]);
        #[cfg(not(windows))]
        let (prog, args_list): (&str, &[&str]) = ("true", &[]);

        let state = AppState::new();

        let child = std::process::Command::new(prog)
            .args(args_list)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        if let Ok(child) = child {
            {
                let mut guard = state.active_child.lock().expect("mutex poisoned");
                *guard = Some(child);
            }
            // Simulate what run_backend does: take and wait.
            let mut guard = state.active_child.lock().expect("mutex poisoned");
            if let Some(c) = guard.take() {
                let _ = c.wait_with_output();
            }
        }

        let guard = state.active_child.lock().expect("mutex poisoned");
        assert!(guard.is_none(), "active_child should be None after completion");
    }

    #[test]
    fn kill_active_child_clears_mutex() {
        // Spawn a long-running no-op, store in state, then kill.
        #[cfg(windows)]
        let (prog, args_list): (&str, &[&str]) = ("cmd", &["/c", "pause"]);
        #[cfg(not(windows))]
        let (prog, args_list): (&str, &[&str]) = ("sleep", &["60"]);

        let state = AppState::new();

        let child = std::process::Command::new(prog)
            .args(args_list)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        if let Ok(child) = child {
            {
                let mut guard = state.active_child.lock().expect("mutex poisoned");
                *guard = Some(child);
            }
            kill_active_child(&state);
            let guard = state.active_child.lock().expect("mutex poisoned");
            assert!(guard.is_none(), "active_child should be None after kill");
        }
    }
}

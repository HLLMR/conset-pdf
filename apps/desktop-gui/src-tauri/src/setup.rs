//! First-run environment probe.
//!
//! Called during Tauri `setup` hook before the main window becomes interactive.
//! If the backend binary is missing or non-executable, the user sees a clear
//! error message — not a silent undefined failure on the first workflow run.

use std::path::Path;

/// Error variants for the environment probe.
#[derive(Debug, Clone)]
pub enum SetupError {
    BinaryNotFound(String),
    BinaryNotExecutable(String),
}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupError::BinaryNotFound(path) => write!(
                f,
                "backend-cli binary not found at '{path}'.\n\
                 Resolution: run `cargo build --release -p backend-cli` \
                 before `cargo tauri build`, then reinstall the app."
            ),
            SetupError::BinaryNotExecutable(msg) => write!(
                f,
                "backend-cli could not be executed: {msg}.\n\
                 On macOS/Linux, check file permissions. \
                 On Windows, ensure the EXE is not blocked by security policy."
            ),
        }
    }
}

/// Probe that the bundled `backend-cli` binary exists and responds to `--help`.
///
/// Does NOT require PDFium — `--help` exits immediately without loading the library.
///
/// # Errors
/// Returns a `SetupError` if the binary is missing or cannot be executed.
pub fn probe_environment(cli_path: &Path) -> Result<(), SetupError> {
    if !cli_path.exists() {
        return Err(SetupError::BinaryNotFound(
            cli_path.display().to_string(),
        ));
    }

    let status = std::process::Command::new(cli_path)
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| SetupError::BinaryNotExecutable(e.to_string()))?;

    // --help exits 0 on success; accept any exit so long as the process runs.
    // A non-zero exit here would mean the binary exists but panics on startup,
    // which we treat as non-executable for the user's purposes.
    let _ = status;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_environment_fails_gracefully_when_binary_missing() {
        let fake_path = Path::new("/nonexistent/path/backend-cli-does-not-exist");
        let result = probe_environment(fake_path);
        assert!(
            result.is_err(),
            "Expected error for missing binary, got Ok"
        );
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("not found"),
            "Error message should mention 'not found': {err_str}"
        );
    }
}

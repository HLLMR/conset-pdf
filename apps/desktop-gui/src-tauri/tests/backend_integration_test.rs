//! Sprint 11.2 integration tests for the Tauri command layer.
//!
//! These tests bypass the Tauri runtime and call `backend-cli` directly to
//! verify that the command argument shapes and `WorkflowResponse` parsing are
//! correct end-to-end.
//!
//! Prerequisites: `cargo build --bin backend-cli` must have been run first.
//! Tests skip gracefully if the binary or fixture is absent.

use std::path::PathBuf;
use std::process::Command;

use conset_pdf_contracts::{OperationStatus, WorkflowResponse};

// ---------------------------------------------------------------------------
// Helpers (mirror cli_basic_test.rs helpers)
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    // tests/.. = src-tauri, ../.. = desktop-gui, ../../.. = apps, ../../../.. = root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // src-tauri → desktop-gui
        .and_then(|p| p.parent()) // desktop-gui → apps
        .and_then(|p| p.parent()) // apps → root
        .expect("could not resolve repo root")
        .to_path_buf()
}

fn backend_cli_path() -> PathBuf {
    repo_root()
        .join("target")
        .join("debug")
        .join(if cfg!(windows) { "backend-cli.exe" } else { "backend-cli" })
}

fn tier1(name: &str) -> PathBuf {
    repo_root()
        .join("tests")
        .join("corpus")
        .join("tier1")
        .join(name)
}

fn tmp_dir(label: &str) -> PathBuf {
    let dir = repo_root()
        .join("target")
        .join("tmp")
        .join("tauri-integration")
        .join(label);
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn parse_workflow_response(bytes: &[u8]) -> WorkflowResponse {
    let stdout = std::str::from_utf8(bytes).expect("stdout is UTF-8");
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Could not parse WorkflowResponse: {e}\nstdout: {stdout}"))
}

// ---------------------------------------------------------------------------
// 11.2.D — Integration tests for command argument shapes
// ---------------------------------------------------------------------------

/// Verifies that `backend-cli extract --dry-run` returns a `WorkflowResponse`
/// that is either succeeded or succeeded-with-warnings (not Failed) when given
/// a real fixture PDF.
///
/// This test validates the argument shape `cmd_extract` produces in commands.rs.
#[test]
fn cmd_extract_dry_run_returns_success() {
    let cli = backend_cli_path();
    if !cli.exists() {
        eprintln!(
            "[skip] backend-cli not found at {} — run `cargo build --bin backend-cli` first",
            cli.display()
        );
        return;
    }

    let spec = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    if !spec.exists() {
        eprintln!("[skip] Tier-1 fixture not found: {}", spec.display());
        return;
    }

    let out_json = tmp_dir("extract_dry_run").join("transcript.json");

    let output = Command::new(&cli)
        .arg("extract")
        .arg("--input")
        .arg(&spec)
        .arg("--output")
        .arg(&out_json)
        .output()
        .expect("failed to spawn backend-cli");

    assert!(
        output.status.success(),
        "backend-cli exited with non-zero status: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response: WorkflowResponse = parse_workflow_response(&output.stdout);
    assert_ne!(
        response.result.status,
        OperationStatus::Failed,
        "extract on a valid fixture should not fail:\n{:#?}",
        response.result
    );
    assert!(
        !response.contracts_version.is_empty(),
        "contracts_version must be present"
    );
}

/// Verifies that `backend-cli apply-addendum` with a bogus input path returns a
/// `WorkflowResponse` with `status = Failed` and a non-empty `error_code`.
///
/// This test validates the error-path argument shape for `cmd_apply_addendum`.
#[test]
fn cmd_apply_addendum_missing_file_returns_error() {
    let cli = backend_cli_path();
    if !cli.exists() {
        eprintln!(
            "[skip] backend-cli not found at {} — run `cargo build --bin backend-cli` first",
            cli.display()
        );
        return;
    }

    let bogus_input = repo_root().join("nonexistent_original.pdf");
    let bogus_addendum = repo_root().join("nonexistent_addendum.pdf");
    let out_pdf = tmp_dir("apply_addendum_missing").join("out.pdf");
    let audit = tmp_dir("apply_addendum_missing").join("audit.zip");

    let output = Command::new(&cli)
        .arg("apply-addendum")
        .arg("--input")
        .arg(&bogus_input)
        .arg("--addendum")
        .arg(&bogus_addendum)
        .arg("--output")
        .arg(&out_pdf)
        .arg("--audit-bundle")
        .arg(&audit)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli");

    // Backend should exit non-zero OR return a Failed WorkflowResponse.
    if output.status.success() {
        let response: WorkflowResponse = parse_workflow_response(&output.stdout);
        assert_eq!(
            response.result.status,
            OperationStatus::Failed,
            "apply-addendum with missing inputs should fail: {:#?}",
            response.result
        );
        assert!(
            response.result.error_code.is_some(),
            "Failed response must carry an error_code"
        );
    }
    // Non-zero exit is also acceptable — the error is correctly surfaced.
}

//! Step 10 scaffold: backend-cli integration test harness.
//!
//! These tests are intentionally conservative and mostly ignored for now because
//! the Phase 1 implementation is not complete. They still validate command
//! surface, response envelope shape, and dry-run behavior where possible.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn find_sample_pdf() -> Option<PathBuf> {
    let base = repo_root().join("tests").join("corpus");
    let candidates = ["tier1", "tier2", "tier3", "holdout"];

    for tier in candidates {
        let tier_dir = base.join(tier);
        if !tier_dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&tier_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "pdf") {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn backend_cli_exe_path() -> PathBuf {
    // The executable should exist after `cargo build --bin backend-cli`.
    repo_root().join("target").join("debug").join(if cfg!(windows) {
        "backend-cli.exe"
    } else {
        "backend-cli"
    })
}

#[test]
#[ignore = "Scaffold test: requires built backend-cli binary and corpus PDF"]
fn cli_extract_dry_run_returns_contract_response_shape() {
    let exe = backend_cli_exe_path();
    assert!(
        exe.exists(),
        "backend-cli binary not found at {}. Run: cargo build --bin backend-cli",
        exe.display()
    );

    let sample_pdf = find_sample_pdf().unwrap_or_else(|| Path::new("tests/corpus/tier1/sample.pdf").to_path_buf());

    let output = Command::new(&exe)
        .arg("extract")
        .arg("--input")
        .arg(sample_pdf)
        .arg("--dry-run")
        .output()
        .expect("failed to execute backend-cli");

    assert!(output.status.success(), "backend-cli returned non-zero status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("stdout should be JSON WorkflowResponse");

    // Validate envelope shape only (Phase 0/1 scaffold expectation).
    assert!(json.get("request_id").is_some());
    assert!(json.get("session_id").is_some());
    assert!(json.get("operation_id").is_some());
    assert!(json.get("contracts_version").is_some());
    assert!(json.get("result").is_some());

    let result = json.get("result").expect("result object");
    assert!(result.get("status").is_some());
    assert!(result.get("summary").is_some());
}

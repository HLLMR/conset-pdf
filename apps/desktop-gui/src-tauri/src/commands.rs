//! Tauri command handlers — the typed IPC boundary between TypeScript and Rust.
//!
//! # Contract
//! - These functions may ONLY depend on `conset-pdf-contracts` (never on engine internals).
//! - All heavy work is delegated to `backend-cli` subprocess via `backend_process::run_backend`.
//! - Return types serialize cleanly to JSON for the TypeScript caller.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::backend_process;

// ---------------------------------------------------------------------------
// Supplementary types for commands not covered by WorkflowRequest/Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub sections_targeted: usize,
}

// ---------------------------------------------------------------------------
// Workflow execution commands
// ---------------------------------------------------------------------------

/// Run `backend-cli extract` on a single PDF.
#[tauri::command]
pub fn cmd_extract(
    app: AppHandle,
    input: String,
    output: String,
) -> Result<serde_json::Value, String> {
    backend_process::run_backend(
        &app,
        &["extract", "--input", &input, "--output", &output],
    )
}

/// Run `backend-cli segment` on an extracted transcript.
#[tauri::command]
pub fn cmd_segment(
    app: AppHandle,
    input: String,
    output: String,
) -> Result<serde_json::Value, String> {
    backend_process::run_backend(
        &app,
        &["segment", "--input", &input, "--output", &output],
    )
}

/// Run `backend-cli index-drawing` on a drawing transcript.
#[tauri::command]
pub fn cmd_index_drawing(
    app: AppHandle,
    input: String,
    output: String,
) -> Result<serde_json::Value, String> {
    backend_process::run_backend(
        &app,
        &["index-drawing", "--input", &input, "--output", &output],
    )
}

/// Run `backend-cli index-submittal` on a submittal transcript.
#[tauri::command]
pub fn cmd_index_submittal(
    app: AppHandle,
    input: String,
    output: String,
) -> Result<serde_json::Value, String> {
    backend_process::run_backend(
        &app,
        &["index-submittal", "--input", &input, "--output", &output],
    )
}

/// Run the full `backend-cli apply-addendum` pipeline (extract + edit + stitch internally).
#[tauri::command]
pub fn cmd_apply_addendum(
    app: AppHandle,
    original: String,
    addendum: String,
    output: String,
    audit_bundle: String,
    dry_run: bool,
    progress_events: bool,
) -> Result<serde_json::Value, String> {
    let mut args = vec![
        "apply-addendum",
        "--input",
        &original,
        "--addendum",
        &addendum,
        "--output",
        &output,
        "--audit-bundle",
        &audit_bundle,
    ];
    if dry_run {
        args.push("--dry-run");
    }
    if progress_events {
        args.push("--progress-events");
    }
    backend_process::run_backend(&app, &args)
}

/// Run the full `backend-cli apply-sheet-addendum` pipeline (extract + index + stitch internally).
#[tauri::command]
pub fn cmd_apply_sheet_addendum(
    app: AppHandle,
    manifest: String,
    output: String,
    audit_bundle: String,
    dry_run: bool,
    progress_events: bool,
) -> Result<serde_json::Value, String> {
    let mut args = vec![
        "apply-sheet-addendum",
        "--manifest",
        &manifest,
        "--output",
        &output,
        "--audit-bundle",
        &audit_bundle,
    ];
    if dry_run {
        args.push("--dry-run");
    }
    if progress_events {
        args.push("--progress-events");
    }
    backend_process::run_backend(&app, &args)
}

/// Run `backend-cli extract-submittal` to export structured submittal data.
#[tauri::command]
pub fn cmd_extract_submittal(
    app: AppHandle,
    input: String,
    index: String,
    output: String,
    format: String,
    audit_bundle: String,
    dry_run: bool,
) -> Result<serde_json::Value, String> {
    let mut args = vec![
        "extract-submittal",
        "--input",
        &input,
        "--index",
        &index,
        "--output",
        &output,
        "--format",
        &format,
        "--audit-bundle",
        &audit_bundle,
    ];
    if dry_run {
        args.push("--dry-run");
    }
    backend_process::run_backend(&app, &args)
}

/// Run `backend-cli visualize` to generate per-page span-detection PNG overlays.
#[tauri::command]
pub fn cmd_visualize(
    app: AppHandle,
    input: String,
    output_dir: String,
) -> Result<serde_json::Value, String> {
    backend_process::run_backend(
        &app,
        &["visualize", "--input", &input, "--output", &output_dir],
    )
}

// ---------------------------------------------------------------------------
// Dialog commands (thin wrappers; real dialog logic uses tauri-plugin-dialog)
// ---------------------------------------------------------------------------

/// Stub — open-file-dialog; real implementation wires tauri-plugin-dialog on the TS side.
/// This command exists so the TypeScript invoke surface is complete and typed.
#[tauri::command]
pub fn cmd_open_file_dialog(
    _title: String,
    _filters: Vec<FileFilter>,
) -> Result<Option<Vec<String>>, String> {
    // In production, the frontend calls @tauri-apps/plugin-dialog directly.
    // This Rust stub satisfies the typed binding generator.
    Ok(None)
}

/// Stub — save-file-dialog; real implementation wires tauri-plugin-dialog on the TS side.
#[tauri::command]
pub fn cmd_save_file_dialog(
    _title: String,
    _default_name: String,
) -> Result<Option<String>, String> {
    Ok(None)
}

// ---------------------------------------------------------------------------
// Manifest validation
// ---------------------------------------------------------------------------

/// Validate an `AddendumManifest` JSON against a `SegmentIndex` JSON.
///
/// Checks that all `section_id` values exist in the detected segment index and
/// that operation types / required fields are present and well-formed.
/// Lives here (not in engine) because it only parses JSON — no PDFium needed.
#[tauri::command]
pub fn cmd_validate_manifest(
    manifest_json: String,
    segment_index_json: String,
) -> Result<ManifestValidationResult, String> {
    validate_manifest_inner(&manifest_json, &segment_index_json)
        .map_err(|e| e.to_string())
}

fn validate_manifest_inner(
    manifest_json: &str,
    segment_index_json: &str,
) -> Result<ManifestValidationResult, serde_json::Error> {
    use serde_json::Value;

    let manifest: Value = serde_json::from_str(manifest_json)?;
    let index: Value = serde_json::from_str(segment_index_json)?;

    let mut errors: Vec<String> = Vec::new();
    let mut sections_targeted: usize = 0;

    // Collect known section IDs from the segment index.
    let known_ids: std::collections::HashSet<String> = index
        .get("sections")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.get("id").and_then(Value::as_str).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    // Validate each section_edit_spec in the manifest.
    let empty: Vec<Value> = Vec::new();
    let specs: &[Value] = manifest
        .get("section_edits")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(empty.as_slice());

    for (i, spec) in specs.iter().enumerate() {
        let section_id = spec
            .get("section_id")
            .and_then(Value::as_str)
            .unwrap_or("");

        if section_id.is_empty() {
            errors.push(format!("Section edit [{i}]: missing `section_id`"));
        } else if !known_ids.contains(section_id) {
            errors.push(format!(
                "Section '{section_id}' not found in document segments"
            ));
        } else {
            sections_targeted += 1;
        }

        // Require at least one operation.
        let ops = spec.get("ops").and_then(Value::as_array);
        match ops {
            None => errors.push(format!("Section '{section_id}': missing `ops` array")),
            Some(o) if o.is_empty() => {
                errors.push(format!("Section '{section_id}': `ops` array is empty"));
            }
            _ => {}
        }
    }

    if specs.is_empty() {
        errors.push("Manifest contains no section edits".to_owned());
    }

    Ok(ManifestValidationResult {
        valid: errors.is_empty(),
        errors,
        sections_targeted,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_manifest_valid_section_produces_success() {
        let manifest = r#"{
            "section_edits": [
                { "section_id": "23 82 16", "ops": [{ "type": "Replace", "node_path": "P.1", "content": "x" }] }
            ]
        }"#;
        let index = r#"{ "sections": [{ "id": "23 82 16", "title": "Test Section" }] }"#;
        let result = validate_manifest_inner(manifest, index).unwrap();
        assert!(result.valid);
        assert_eq!(result.sections_targeted, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validate_manifest_unknown_section_reports_error() {
        let manifest = r#"{
            "section_edits": [
                { "section_id": "23 99 99", "ops": [{ "type": "Replace", "node_path": "P.1", "content": "x" }] }
            ]
        }"#;
        let index = r#"{ "sections": [{ "id": "23 82 16", "title": "Test Section" }] }"#;
        let result = validate_manifest_inner(manifest, index).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("23 99 99")));
    }

    #[test]
    fn validate_manifest_empty_section_edits_reports_error() {
        let manifest = r#"{ "section_edits": [] }"#;
        let index = r#"{ "sections": [] }"#;
        let result = validate_manifest_inner(manifest, index).unwrap();
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("no section edits")));
    }
}

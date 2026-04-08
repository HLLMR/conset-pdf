//! ApplySheetAddendum operation handler (Phase 9.2).
//!
//! Loads a [`DrawingAddendumManifest`] JSON, delegates to
//! [`DrawingsPatchOrchestrator`] for extraction → sheet indexing → page
//! extraction → stitch, and writes the resulting [`DrawingPatchResult`] to
//! stdout via the normal `WorkflowResponse` serialisation path.  Audit
//! artifacts (`change-report.json`, `metrics.json`) are written to
//! `--audit-bundle` when provided.
//!
//! # Metadata keys
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `manifest_path` | yes | Path to the [`DrawingAddendumManifest`] JSON file |
//! | `audit_bundle_dir` | no | Directory for audit artifacts |

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::DrawingsPatchOrchestrator;
use conset_pdf_ir::DrawingAddendumManifest;

/// Run the apply-sheet-addendum operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::ApplySheetAddendum,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    let meta = &req.options.metadata;

    // ── Require manifest_path ─────────────────────────────────────────────────
    let manifest_path = match meta.iter().find(|kv| kv.key == "manifest_path") {
        Some(kv) => kv.value.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--manifest <JSON> (manifest_path key) is required".to_owned(),
                vec![],
                Some("MISSING_MANIFEST_PATH".to_owned()),
            );
        }
    };

    // ── Load and validate DrawingAddendumManifest ─────────────────────────────
    let manifest: DrawingAddendumManifest = match std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("cannot read manifest '{}': {e}", manifest_path))
        .and_then(|text| {
            serde_json::from_str(&text).map_err(|e| {
                format!(
                    "invalid DrawingAddendumManifest JSON at '{}': {e}",
                    manifest_path
                )
            })
        }) {
        Ok(m) => m,
        Err(msg) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                msg,
                vec![],
                Some("MANIFEST_READ_ERROR".to_owned()),
            );
        }
    };

    if manifest.sheets.is_empty() {
        record_ended(bundle, req, &started_at, OperationStatus::Failed);
        return make_response(
            req,
            OperationStatus::Failed,
            "DrawingAddendumManifest.sheets is empty — nothing to do".to_owned(),
            vec![],
            Some("EMPTY_MANIFEST".to_owned()),
        );
    }

    // ── Create audit bundle dir if requested ──────────────────────────────────
    if let Some(kv) = meta.iter().find(|kv| kv.key == "audit_bundle_dir") {
        if let Err(e) = std::fs::create_dir_all(&kv.value) {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!(
                    "cannot create audit bundle directory '{}': {e}",
                    kv.value
                ),
                vec![],
                Some("AUDIT_DIR_CREATE_ERROR".to_owned()),
            );
        }
    }

    // ── Propagate --dry-run flag from CLI options into the manifest ───────────
    // The CLI --dry-run flag sets req.options.dry_run; the manifest field may
    // independently be false.  Either source should activate dry-run mode.
    let mut manifest = manifest;
    if req.options.dry_run {
        manifest.dry_run = true;
    }

    // ── Propagate --audit-bundle CLI option into manifest.audit_bundle_dir ───
    // If the caller passed --audit-bundle <dir> the path lands in metadata; wire
    // it into the manifest so the orchestrator writes the bundle there.
    if manifest.audit_bundle_dir.is_none() {
        if let Some(kv) = meta.iter().find(|kv| kv.key == "audit_bundle_dir") {
            manifest.audit_bundle_dir = Some(kv.value.clone());
        }
    }

    // ── Delegate to DrawingsPatchOrchestrator ─────────────────────────────────
    let patch_result = match DrawingsPatchOrchestrator::run(&manifest) {
        Ok(r) => r,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                e,
                vec![],
                Some("ORCHESTRATOR_ERROR".to_owned()),
            );
        }
    };

    let replaced = patch_result
        .sheet_results
        .iter()
        .filter(|r| r.status == conset_pdf_ir::SheetPatchStatus::Replaced)
        .count();
    let total = patch_result.sheet_results.len();

    // Emit the result JSON as part of the response payload via warnings vec
    // (same pattern as apply-addendum result embedding).
    let mut warnings: Vec<String> = Vec::new();
    match serde_json::to_string_pretty(&patch_result) {
        Ok(json) => {
            // Write to --output file if provided.
            if let Some(ref out_path) = req.output_path {
                if let Err(e) = std::fs::write(out_path, &json) {
                    warnings.push(format!(
                        "failed to write patch result to '{out_path}': {e}"
                    ));
                }
            }
        }
        Err(e) => {
            warnings.push(format!("failed to serialise DrawingPatchResult: {e}"));
        }
    }

    let failed = patch_result
        .sheet_results
        .iter()
        .filter(|r| {
            matches!(
                r.status,
                conset_pdf_ir::SheetPatchStatus::Failed { .. }
            )
        })
        .count();

    let status = if failed > 0 && replaced == 0 {
        OperationStatus::Failed
    } else if failed > 0 || !warnings.is_empty() {
        OperationStatus::SucceededWithWarnings
    } else {
        OperationStatus::Succeeded
    };

    record_ended(bundle, req, &started_at, status.clone());
    make_response(
        req,
        status,
        format!(
            "Applied {replaced}/{total} sheet replacement(s){}",
            if manifest.dry_run { " (dry-run)" } else { "" }
        ),
        warnings,
        None,
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn record_ended(
    bundle: &mut AuditBundle,
    req: &WorkflowRequest,
    started_at: &chrono::DateTime<chrono::Utc>,
    result: OperationStatus,
) {
    let ended_at = Utc::now();
    let elapsed_ms = (ended_at - *started_at).num_milliseconds();
    let duration_ms: u64 = u64::try_from(elapsed_ms).unwrap_or(0);
    bundle.add_event(AuditEvent::new(AuditEventData::OperationEnded {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::ApplySheetAddendum,
        ended_at_utc: ended_at.to_rfc3339(),
        duration_ms,
        result,
    }));
}

fn make_response(
    req: &WorkflowRequest,
    status: OperationStatus,
    summary: String,
    warnings: Vec<String>,
    error_code: Option<String>,
) -> WorkflowResponse {
    WorkflowResponse::new(
        req.request_id.clone(),
        req.session_id.clone(),
        req.operation_id.clone(),
        OperationResult { status, summary, warnings, error_code, output_artifacts: vec![] },
        vec![],
    )
}

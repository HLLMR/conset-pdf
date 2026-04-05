//! VisualizeSegments operation handler.
//!
//! Reads a `SegmentIndex` JSON produced by the Segment operation, re-extracts
//! the source PDF identified in the index, and renders per-page PNG overlays
//! with color-coded furniture bands:
//!
//! - **Blue** — header band (top 15 %)
//! - **Red**  — footer band  (bottom 15 %)
//! - **Green** — body band
//!
//! Pages that begin a new CSI section are identified by filename:
//! `page-{N:04}-section-{id}.png`; others are `page-{N:04}.png`.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::Extractor;
use conset_pdf_ir::SegmentIndex;
use std::path::Path;

/// Run the visualize-segments operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::VisualizeSegments,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    if req.options.dry_run {
        record_ended(bundle, req, &started_at, OperationStatus::Succeeded);
        return make_response(
            req,
            OperationStatus::Succeeded,
            "dry_run: argument validation passed — no rendering performed".to_owned(),
            vec![],
            None,
        );
    }

    // Require --output for the output PNG directory.
    let output_dir = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <DIR> is required for the visualize-segments operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Read and deserialize segment index JSON.
    let segment_index = match read_segment_index(&req.input_path) {
        Ok(s) => s,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to read segment index '{}': {e}", req.input_path),
                vec![],
                Some("INVALID_SEGMENT_INDEX".to_owned()),
            );
        }
    };

    // Re-extract the source PDF to get the LayoutTranscript.
    let transcript = match Extractor::new().extract(&segment_index.source_path) {
        Ok(t) => t,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!(
                    "Failed to extract source PDF '{}': {e}",
                    segment_index.source_path
                ),
                vec![],
                Some("EXTRACTION_ERROR".to_owned()),
            );
        }
    };

    // Render color-coded overlays.
    match conset_pdf_engine::visualize::render_segment_overlays(
        &transcript,
        &segment_index,
        Path::new(&output_dir),
    ) {
        Ok(pages) => {
            record_ended(bundle, req, &started_at, OperationStatus::Succeeded);
            make_response(
                req,
                OperationStatus::Succeeded,
                format!("Rendered {pages} segment overlay(s) to \"{output_dir}\""),
                vec![],
                None,
            )
        }
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            make_response(
                req,
                OperationStatus::Failed,
                format!("Segment visualization failed: {e}"),
                vec![],
                Some("RENDER_ERROR".to_owned()),
            )
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_segment_index(path: &str) -> std::result::Result<SegmentIndex, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error reading segment index: {e}"))?;
    serde_json::from_str::<SegmentIndex>(&json)
        .map_err(|e| format!("JSON parse error: {e}"))
}

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
        operation: WorkflowOperation::VisualizeSegments,
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

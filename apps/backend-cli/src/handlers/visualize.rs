//! Visualize operation handler.
//!
//! Reads a `LayoutTranscript` JSON produced by the Extract operation, then
//! calls the engine's overlay renderer to write one PNG per page to the
//! requested output directory.
//!
//! The source PDF path is embedded in `transcript.metadata.source_path`;
//! no separate PDF argument is required.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_ir::LayoutTranscript;
use std::path::Path;

/// Run the visualize operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Visualize,
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

    // Require --output for the output directory.
    let output_dir = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <DIR> is required for the visualize operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Deserialize transcript JSON.
    let transcript = match read_transcript(&req.input_path) {
        Ok(t) => t,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to read transcript '{}': {e}", req.input_path),
                vec![],
                Some("INVALID_TRANSCRIPT".to_owned()),
            );
        }
    };

    // Render overlays.
    match conset_pdf_engine::visualize::render_transcript_overlays(
        &transcript,
        Path::new(&output_dir),
    ) {
        Ok(pages) => {
            record_ended(bundle, req, &started_at, OperationStatus::Succeeded);
            make_response(
                req,
                OperationStatus::Succeeded,
                format!("Rendered {pages} page overlay(s) to \"{output_dir}\""),
                vec![],
                None,
            )
        }
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            make_response(
                req,
                OperationStatus::Failed,
                format!("Visualization failed: {e}"),
                vec![],
                Some("RENDER_ERROR".to_owned()),
            )
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_transcript(path: &str) -> std::result::Result<LayoutTranscript, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error reading transcript: {e}"))?;
    serde_json::from_str::<LayoutTranscript>(&json)
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
        operation: WorkflowOperation::Visualize,
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

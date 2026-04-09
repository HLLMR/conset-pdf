//! Index-submittal operation handler.
//!
//! Reads a `LayoutTranscript` JSON produced by the Extract operation, runs the
//! submittal unit-boundary oracle, and writes the resulting `SubmittalIndex`
//! JSON to the requested output path.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::SubmittalSegmentEngine;
use conset_pdf_ir::LayoutTranscript;
use std::path::Path;

/// Run the index-submittal operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::IndexSubmittal,
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
            "dry_run: argument validation passed — no indexing performed".to_owned(),
            vec![],
            None,
        );
    }

    // Require --output for the submittal index JSON.
    let output_path = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <FILE> is required for the index-submittal operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Read and deserialize transcript JSON.
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

    // Derive packet name from the input path stem.
    let packet_name = packet_name_from_path(&req.input_path);

    // Run the submittal segmentation engine.
    let submittal_index = SubmittalSegmentEngine::build_index(&transcript, &packet_name);

    let unit_count = submittal_index.unit_count();
    let total_pages = submittal_index.total_pages();

    // Write SubmittalIndex JSON.
    let mut warnings: Vec<String> = Vec::new();
    match serde_json::to_string_pretty(&submittal_index) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&output_path, &json) {
                warnings.push(format!("Failed to write submittal index to '{output_path}': {e}"));
            }
        }
        Err(e) => {
            warnings.push(format!("Failed to serialise submittal index: {e}"));
        }
    }

    let status = if warnings.is_empty() {
        OperationStatus::Succeeded
    } else {
        OperationStatus::SucceededWithWarnings
    };

    record_ended(bundle, req, &started_at, status.clone());
    make_response(
        req,
        status,
        format!("Indexed {unit_count} unit(s) from {total_pages} pages"),
        warnings,
        None,
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_transcript(path: &str) -> std::result::Result<LayoutTranscript, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error reading transcript: {e}"))?;
    serde_json::from_str::<LayoutTranscript>(&json)
        .map_err(|e| format!("JSON parse error: {e}"))
}

/// Derive a packet name from an input path by using the file stem.
///
/// `"/tmp/sub/SUB_PESH_BERG_TRANE-AHU_transcript.json"` → `"SUB_PESH_BERG_TRANE-AHU_transcript"`
fn packet_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("UNKNOWN")
        .to_owned()
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
        operation: WorkflowOperation::IndexSubmittal,
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

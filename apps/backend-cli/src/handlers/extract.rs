//! Extract operation handler.
//!
//! This handler is the **contracts ↔ engine translation layer** for the Extract
//! operation, as deferred from Step 4 of the monorepo reorganisation plan:
//!
//! - Inbound: `WorkflowRequest` (contracts type) → `path: &str` (engine input)
//! - Engine call: `conset_pdf_engine::Extractor::extract` → `LayoutTranscript`
//! - Outbound: `LayoutTranscript` page count → `WorkflowResponse` (contracts type)
//!
//! The engine itself has no dependency on the `contracts` crate; translation
//! is confined to this handler module.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::Extractor;

/// Run the extract operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Extract,
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
            "dry_run: argument validation passed — no file was processed".to_owned(),
            vec![],
            None,
        );
    }

    // ── Contracts → engine ────────────────────────────────────────────────────
    let extract_result = Extractor::new().extract(&req.input_path);

    // ── Engine → contracts ────────────────────────────────────────────────────
    let (status, summary, warnings, error_code) = match &extract_result {
        Ok(transcript) => {
            let page_count = transcript.page_count();
            let mut w: Vec<String> = Vec::new();

            // Write transcript JSON to output file when --output is given.
            if let Some(out_path) = &req.output_path {
                match serde_json::to_string_pretty(transcript) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(out_path, &json) {
                            w.push(format!("Failed to write transcript to '{out_path}': {e}"));
                        }
                    }
                    Err(e) => {
                        w.push(format!("Failed to serialize transcript: {e}"));
                    }
                }
            }

            let status = if w.is_empty() {
                OperationStatus::Succeeded
            } else {
                OperationStatus::SucceededWithWarnings
            };
            (
                status,
                format!("Extracted {page_count} page(s) from \"{}\"", req.input_path),
                w,
                None,
            )
        }
        Err(e) => (
            OperationStatus::Failed,
            format!("Extraction failed: {e}"),
            vec![],
            Some("EXTRACTION_ERROR".to_owned()),
        ),
    };

    record_ended(bundle, req, &started_at, status.clone());
    make_response(req, status, summary, warnings, error_code)
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
        operation: WorkflowOperation::Extract,
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

//! VisualizeAst operation handler.
//!
//! Reads a `ParsedDocument` JSON produced by the Parse operation and renders
//! a self-contained HTML tree-view file for outline inspection.
//!
//! The output path (`--output`) is a single `.html` file.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_ir::ParsedDocument;
use std::path::Path;

/// Run the visualize-ast operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::VisualizeAst,
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
            "dry_run: argument validation passed — no HTML written".to_owned(),
            vec![],
            None,
        );
    }

    // --output is required (path to the output HTML file).
    let output_path = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <FILE.html> is required for the visualize-ast operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Read and deserialize ParsedDocument JSON.
    let parsed_doc = match read_parsed_doc(&req.input_path) {
        Ok(d) => d,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to read parsed document '{}': {e}", req.input_path),
                vec![],
                Some("INVALID_PARSED_DOCUMENT".to_owned()),
            );
        }
    };

    let section_count = parsed_doc.sections.len();

    // Render HTML.
    match conset_pdf_engine::visualize_ast::render_ast_html(&parsed_doc, Path::new(&output_path)) {
        Ok(()) => {
            record_ended(bundle, req, &started_at, OperationStatus::Succeeded);
            make_response(
                req,
                OperationStatus::Succeeded,
                format!(
                    "Rendered AST for {section_count} section(s) → \"{output_path}\""
                ),
                vec![],
                None,
            )
        }
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            make_response(
                req,
                OperationStatus::Failed,
                format!("AST visualisation failed: {e}"),
                vec![],
                Some("RENDER_ERROR".to_owned()),
            )
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_parsed_doc(path: &str) -> std::result::Result<ParsedDocument, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error reading parsed document: {e}"))?;
    serde_json::from_str::<ParsedDocument>(&json)
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
        operation: WorkflowOperation::VisualizeAst,
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

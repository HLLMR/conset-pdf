//! Edit operation handler.
//!
//! Accepts a [`ParsedDocument`] JSON as `--input` and an [`EditRequest`] JSON
//! as the `operations_path` metadata key, applies the edit operations via
//! [`SectionEditor`], and writes the modified [`ParsedDocument`] JSON to
//! `--output`.
//!
//! # Metadata keys
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `operations_path` | yes | Path to the [`EditRequest`] JSON file |

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::SectionEditor;
use conset_pdf_ir::{EditRequest, ParsedDocument};

/// Run the edit operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Edit,
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
            "dry_run: argument validation passed — no editing performed".to_owned(),
            vec![],
            None,
        );
    }

    // --output is required.
    let output_path = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <FILE> is required for the edit operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // `operations_path` metadata key: path to the EditRequest JSON.
    let operations_path = match req
        .options
        .metadata
        .iter()
        .find(|kv| kv.key == "operations_path")
        .map(|kv| kv.value.as_str())
    {
        Some(p) => p.to_owned(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--operations <FILE> (operations_path metadata key) is required for edit".to_owned(),
                vec![],
                Some("MISSING_OPERATIONS_PATH".to_owned()),
            );
        }
    };

    // ── Load ParsedDocument ───────────────────────────────────────────────────
    let doc: ParsedDocument = match std::fs::read_to_string(&req.input_path)
        .map_err(|e| format!("cannot read '{}': {e}", req.input_path))
        .and_then(|text| {
            serde_json::from_str(&text)
                .map_err(|e| format!("invalid ParsedDocument JSON at '{}': {e}", req.input_path))
        }) {
        Ok(d) => d,
        Err(msg) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                msg,
                vec![],
                Some("INPUT_READ_ERROR".to_owned()),
            );
        }
    };

    // ── Load EditRequest ──────────────────────────────────────────────────────
    let edit_request: EditRequest = match std::fs::read_to_string(&operations_path)
        .map_err(|e| format!("cannot read operations file '{}': {e}", operations_path))
        .and_then(|text| {
            serde_json::from_str(&text)
                .map_err(|e| format!("invalid EditRequest JSON at '{}': {e}", operations_path))
        }) {
        Ok(r) => r,
        Err(msg) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                msg,
                vec![],
                Some("OPERATIONS_READ_ERROR".to_owned()),
            );
        }
    };

    let op_count = edit_request.operations.len();

    // ── Apply edits ───────────────────────────────────────────────────────────
    let mut editor = SectionEditor::new(doc);
    let result = editor.apply(edit_request);
    let updated_doc = editor.into_document();

    if !result.success {
        let err_msg = result
            .error
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown error".to_string());
        record_ended(bundle, req, &started_at, OperationStatus::Failed);
        return make_response(
            req,
            OperationStatus::Failed,
            format!(
                "Edit failed after {}/{op_count} operation(s): {err_msg}",
                result.operations_applied
            ),
            result.warnings,
            Some("EDIT_ERROR".to_owned()),
        );
    }

    // ── Write output ──────────────────────────────────────────────────────────
    let json = match serde_json::to_string_pretty(&updated_doc) {
        Ok(j) => j,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to serialize updated document: {e}"),
                vec![],
                Some("SERIALIZATION_ERROR".to_owned()),
            );
        }
    };

    if let Err(e) = std::fs::write(&output_path, &json) {
        record_ended(bundle, req, &started_at, OperationStatus::Failed);
        return make_response(
            req,
            OperationStatus::Failed,
            format!("Failed to write output to '{}': {e}", output_path),
            vec![],
            Some("OUTPUT_WRITE_ERROR".to_owned()),
        );
    }

    let status = if result.warnings.is_empty() {
        OperationStatus::Succeeded
    } else {
        OperationStatus::SucceededWithWarnings
    };

    record_ended(bundle, req, &started_at, status.clone());
    make_response(
        req,
        status,
        format!(
            "Applied {}/{op_count} edit operation(s) to {} section(s)",
            result.operations_applied,
            updated_doc.sections.len(),
        ),
        result.warnings,
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
        operation: WorkflowOperation::Edit,
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

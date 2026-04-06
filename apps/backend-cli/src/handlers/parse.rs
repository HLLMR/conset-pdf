//! Parse operation handler.
//!
//! Accepts a source PDF path, runs the full extract → segment → parse pipeline
//! internally, and writes a [`ParsedDocument`] JSON to the output path.
//!
//! An optional `section_filter` key in `WorkflowOptions.metadata` limits
//! output to a single CSI section (e.g. `"23 82 16"`).

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::{segment::segment_transcript, Extractor};

/// Run the parse operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Parse,
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
            "dry_run: argument validation passed — no parsing performed".to_owned(),
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
                "--output <FILE> is required for the parse operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Optional section-ID filter from metadata.
    let section_filter: Option<String> = req
        .options
        .metadata
        .iter()
        .find(|kv| kv.key == "section_filter")
        .map(|kv| kv.value.clone());

    // ── Extract ───────────────────────────────────────────────────────────────
    let transcript = match Extractor::new().extract(&req.input_path) {
        Ok(t) => t,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Extraction failed for '{}': {e}", req.input_path),
                vec![],
                Some("EXTRACTION_ERROR".to_owned()),
            );
        }
    };

    // ── Segment ───────────────────────────────────────────────────────────────
    let segment_index = match segment_transcript(&transcript) {
        Ok(idx) => idx,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Segmentation failed: {e}"),
                vec![],
                Some("SEGMENTATION_ERROR".to_owned()),
            );
        }
    };

    // ── Parse ─────────────────────────────────────────────────────────────────
    let parsed_doc = match conset_pdf_engine::parse::parse_document(
        &transcript,
        &segment_index,
        section_filter.as_deref(),
    ) {
        Ok(d) => d,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Parse failed: {e}"),
                vec![],
                Some("PARSE_ERROR".to_owned()),
            );
        }
    };

    let section_count = parsed_doc.sections.len();
    let node_count: usize = parsed_doc.sections.iter().map(|s| count_nodes(&s.nodes)).sum();
    let mut warnings = parsed_doc.global_warnings.clone();

    // Write ParsedDocument JSON.
    match serde_json::to_string_pretty(&parsed_doc) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&output_path, &json) {
                warnings.push(format!("Failed to write AST to '{output_path}': {e}"));
            }
        }
        Err(e) => {
            warnings.push(format!("Failed to serialise AST: {e}"));
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
        format!("Parsed {section_count} section(s), {node_count} outline node(s)"),
        warnings,
        None,
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count_nodes(nodes: &[conset_pdf_ir::AstNode]) -> usize {
    nodes
        .iter()
        .map(|n| 1 + count_nodes(&n.children))
        .sum()
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
        operation: WorkflowOperation::Parse,
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


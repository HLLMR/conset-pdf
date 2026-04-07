//! Stitch operation handler.
//!
//! Reads a [`SegmentIndex`] JSON (`--segment-index`), an original PDF
//! (`--input`), and a replacement PDF (`replacement_path` metadata key), then
//! calls [`PdfStitcher::stitch`] to produce a new PDF with the target section's
//! pages replaced.
//!
//! # Metadata keys
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `segment_index_path` | yes | Path to the [`SegmentIndex`] JSON |
//! | `replacement_path` | yes | Path to the regenerated replacement PDF |
//! | `section_id` | yes | CSI section ID to replace (e.g. `"23 82 16"`) |

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::PdfStitcher;
use conset_pdf_ir::{SegmentIndex, StitchPlan};

/// Run the stitch operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Stitch,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    let output_path = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <FILE> is required for the stitch operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    let meta = &req.options.metadata;

    let segment_index_path = match meta.iter().find(|kv| kv.key == "segment_index_path") {
        Some(kv) => kv.value.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--segment-index <FILE> (segment_index_path key) is required".to_owned(),
                vec![],
                Some("MISSING_SEGMENT_INDEX_PATH".to_owned()),
            );
        }
    };

    let replacement_path = match meta.iter().find(|kv| kv.key == "replacement_path") {
        Some(kv) => kv.value.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--replacement <FILE> (replacement_path key) is required".to_owned(),
                vec![],
                Some("MISSING_REPLACEMENT_PATH".to_owned()),
            );
        }
    };

    let section_id = match meta.iter().find(|kv| kv.key == "section_id") {
        Some(kv) => kv.value.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--section <ID> (section_id key) is required".to_owned(),
                vec![],
                Some("MISSING_SECTION_ID".to_owned()),
            );
        }
    };

    // ── Load SegmentIndex ─────────────────────────────────────────────────────
    let segment_index: SegmentIndex =
        match std::fs::read_to_string(&segment_index_path)
            .map_err(|e| format!("cannot read '{}': {e}", segment_index_path))
            .and_then(|text| {
                serde_json::from_str(&text).map_err(|e| {
                    format!("invalid SegmentIndex JSON at '{}': {e}", segment_index_path)
                })
            }) {
            Ok(idx) => idx,
            Err(msg) => {
                record_ended(bundle, req, &started_at, OperationStatus::Failed);
                return make_response(
                    req,
                    OperationStatus::Failed,
                    msg,
                    vec![],
                    Some("SEGMENT_INDEX_READ_ERROR".to_owned()),
                );
            }
        };

    // ── Run stitch ────────────────────────────────────────────────────────────
    let plan = StitchPlan {
        original_path: req.input_path.clone(),
        section_id: section_id.clone(),
        segment_index,
        replacement_path,
        output_path: output_path.clone(),
        dry_run: req.options.dry_run,
    };

    match PdfStitcher::stitch(&plan) {
        Ok(result) => {
            let status = if result.warnings.is_empty() {
                OperationStatus::Succeeded
            } else {
                OperationStatus::SucceededWithWarnings
            };
            let summary = format!(
                "Stitched section '{}': removed {} page(s), inserted {} page(s); \
                 output has {} page(s) total",
                result.section_id,
                result.pages_removed,
                result.pages_inserted,
                result.total_pages_after,
            );
            record_ended(bundle, req, &started_at, status.clone());
            make_response(req, status, summary, result.warnings, None)
        }
        Err(err) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            make_response(
                req,
                OperationStatus::Failed,
                err.to_string(),
                vec![],
                Some(error_code_for(&err)),
            )
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn error_code_for(err: &conset_pdf_ir::StitchError) -> String {
    use conset_pdf_ir::StitchError;
    match err {
        StitchError::SectionNotFound(_) => "SECTION_NOT_FOUND",
        StitchError::OriginalNotFound(_) => "ORIGINAL_NOT_FOUND",
        StitchError::ReplacementNotFound(_) => "REPLACEMENT_NOT_FOUND",
        StitchError::WriteFailed(_) => "WRITE_FAILED",
        StitchError::PageRangeOutOfBounds(_) => "PAGE_RANGE_OUT_OF_BOUNDS",
        StitchError::PdfStructure(_) => "PDF_STRUCTURE_ERROR",
    }
    .to_owned()
}

fn record_ended(
    bundle: &mut AuditBundle,
    req: &WorkflowRequest,
    started_at: &chrono::DateTime<Utc>,
    status: OperationStatus,
) {
    let ended_at = Utc::now();
    let elapsed = (ended_at - *started_at).num_milliseconds();
    bundle.add_event(AuditEvent::new(AuditEventData::OperationEnded {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Stitch,
        ended_at_utc: ended_at.to_rfc3339(),
        duration_ms: u64::try_from(elapsed).unwrap_or(0),
        result: status,
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
        OperationResult {
            status,
            summary,
            warnings,
            error_code,
            output_artifacts: vec![],
        },
        vec![],
    )
}

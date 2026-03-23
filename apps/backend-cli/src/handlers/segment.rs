//! Segment operation handler (not implemented).
//!
//! Phase 1 will implement document section segmentation using the parsed
//! `LayoutTranscript` produced by the extract stage.

use conset_pdf_audit::AuditBundle;
use conset_pdf_contracts::{OperationResult, OperationStatus, WorkflowRequest, WorkflowResponse};

/// Run the segment operation for the given request.
pub fn run(req: &WorkflowRequest, _bundle: &mut AuditBundle) -> WorkflowResponse {
    WorkflowResponse::new(
        req.request_id.clone(),
        req.session_id.clone(),
        req.operation_id.clone(),
        OperationResult {
            status: OperationStatus::Failed,
            summary: "segment is not implemented in this CLI version".to_owned(),
            warnings: vec![],
            error_code: Some("NOT_IMPLEMENTED".to_owned()),
            output_artifacts: vec![],
        },
        vec![],
    )
}

//! Parse operation handler (not implemented).
//!
//! Phase 1 will implement document AST construction from the segmented
//! `LayoutTranscript` produced by the segment stage.

use conset_pdf_audit::AuditBundle;
use conset_pdf_contracts::{OperationResult, OperationStatus, WorkflowRequest, WorkflowResponse};

/// Run the parse operation for the given request.
pub fn run(req: &WorkflowRequest, _bundle: &mut AuditBundle) -> WorkflowResponse {
    WorkflowResponse::new(
        req.request_id.clone(),
        req.session_id.clone(),
        req.operation_id.clone(),
        OperationResult {
            status: OperationStatus::Failed,
            summary: "parse is not implemented in this CLI version".to_owned(),
            warnings: vec![],
            error_code: Some("NOT_IMPLEMENTED".to_owned()),
            output_artifacts: vec![],
        },
        vec![],
    )
}

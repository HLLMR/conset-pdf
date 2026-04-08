//! Operation handlers for backend-cli.
//!
//! Each handler module accepts a [`WorkflowRequest`] reference plus a mutable
//! [`AuditBundle`] for event recording, and returns a [`WorkflowResponse`].
//!
//! This layer is the **only** place in the codebase where `contracts` types are
//! translated to and from the engine's `LayoutTranscript`-typed API.

pub mod apply_addendum;
pub mod edit;
pub mod extract;
pub mod parse;
pub mod regenerate;
pub mod segment;
pub mod stitch;
pub mod visualize;
pub mod visualize_ast;
pub mod visualize_segments;

use conset_pdf_audit::AuditBundle;
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};

/// Dispatch a workflow request to the appropriate handler.
pub fn dispatch(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    match req.operation {
        WorkflowOperation::Extract => extract::run(req, bundle),
        WorkflowOperation::Segment => segment::run(req, bundle),
        WorkflowOperation::Parse => parse::run(req, bundle),
        WorkflowOperation::Edit => edit::run(req, bundle),
        WorkflowOperation::Regenerate => regenerate::run(req, bundle),
        WorkflowOperation::Stitch => stitch::run(req, bundle),
        WorkflowOperation::SpecsPatch => apply_addendum::run(req, bundle),
        WorkflowOperation::Visualize => visualize::run(req, bundle),
        WorkflowOperation::VisualizeSegments => visualize_segments::run(req, bundle),
        WorkflowOperation::VisualizeAst => visualize_ast::run(req, bundle),
        _ => not_implemented(req, bundle),
    }
}

fn not_implemented(req: &WorkflowRequest, _bundle: &mut AuditBundle) -> WorkflowResponse {
    WorkflowResponse::new(
        req.request_id.clone(),
        req.session_id.clone(),
        req.operation_id.clone(),
        OperationResult {
            status: OperationStatus::Failed,
            summary: format!(
                "Operation is not implemented in this CLI version: {:?}",
                req.operation
            ),
            warnings: vec![],
            error_code: Some("NOT_IMPLEMENTED".to_owned()),
            output_artifacts: vec![],
        },
        vec![],
    )
}

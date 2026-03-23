//! Workflow orchestration contracts.
//!
//! Ordering contract imported from Phase D M-002:
//! analyze -> applyCorrections -> execute

use conset_pdf_contracts::{OperationResult, WorkflowRequest, WorkflowResponse};
use serde::{Deserialize, Serialize};

pub mod assemble_set;
pub mod detect;
pub mod fix_bookmarks;
pub mod merge_addenda;
pub mod specs_patch;
pub mod split_set;

/// Execution phase defined by the canonical workflow ordering contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowPhase {
    Analyze,
    ApplyCorrections,
    Execute,
}

/// Shared context passed to workflow implementations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub session_id: String,
    pub operation_id: String,
    pub phase: WorkflowPhase,
    pub request: WorkflowRequest,
}

/// Result from a workflow run, including the response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub phase: WorkflowPhase,
    pub completed: bool,
    pub response: WorkflowResponse,
}

impl WorkflowResult {
    #[must_use]
    pub fn from_operation_result(
        phase: WorkflowPhase,
        completed: bool,
        request: &WorkflowRequest,
        operation_result: OperationResult,
    ) -> Self {
        let response = WorkflowResponse::new(
            request.request_id.clone(),
            request.session_id.clone(),
            request.operation_id.clone(),
            operation_result,
            Vec::new(),
        );
        Self { phase, completed, response }
    }
}

/// Trait implemented by each concrete workflow style.
pub trait Workflow {
    fn name(&self) -> &'static str;
    fn run(&self, context: WorkflowContext) -> WorkflowResult;
}

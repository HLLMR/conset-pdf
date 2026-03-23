use conset_pdf_contracts::{OperationResult, OperationStatus};

use crate::{Workflow, WorkflowContext, WorkflowPhase, WorkflowResult};

pub struct MergeAddendaWorkflow;

impl Workflow for MergeAddendaWorkflow {
    fn name(&self) -> &'static str {
        "merge_addenda"
    }

    fn run(&self, context: WorkflowContext) -> WorkflowResult {
        let result = OperationResult {
            status: OperationStatus::SucceededWithWarnings,
            summary: "merge_addenda workflow scaffold".to_owned(),
            warnings: vec!["Workflow not implemented yet".to_owned()],
            error_code: None,
            output_artifacts: Vec::new(),
        };
        WorkflowResult::from_operation_result(
            WorkflowPhase::Execute,
            false,
            &context.request,
            result,
        )
    }
}

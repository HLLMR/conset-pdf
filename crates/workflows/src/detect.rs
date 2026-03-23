use conset_pdf_contracts::{OperationResult, OperationStatus};

use crate::{Workflow, WorkflowContext, WorkflowPhase, WorkflowResult};

pub struct DetectWorkflow;

impl Workflow for DetectWorkflow {
    fn name(&self) -> &'static str {
        "detect"
    }

    fn run(&self, context: WorkflowContext) -> WorkflowResult {
        let result = OperationResult {
            status: OperationStatus::SucceededWithWarnings,
            summary: "detect workflow scaffold".to_owned(),
            warnings: vec!["Workflow not implemented yet".to_owned()],
            error_code: None,
            output_artifacts: Vec::new(),
        };
        WorkflowResult::from_operation_result(
            WorkflowPhase::Analyze,
            false,
            &context.request,
            result,
        )
    }
}

use conset_pdf_contracts::{OperationResult, OperationStatus};

use crate::{Workflow, WorkflowContext, WorkflowPhase, WorkflowResult};

pub struct SpecsPatchWorkflow;

impl Workflow for SpecsPatchWorkflow {
    fn name(&self) -> &'static str {
        "specs_patch"
    }

    fn run(&self, context: WorkflowContext) -> WorkflowResult {
        let result = OperationResult {
            status: OperationStatus::SucceededWithWarnings,
            summary: "specs_patch workflow scaffold".to_owned(),
            warnings: vec!["Workflow not implemented yet".to_owned()],
            error_code: None,
            output_artifacts: Vec::new(),
        };
        WorkflowResult::from_operation_result(
            WorkflowPhase::ApplyCorrections,
            false,
            &context.request,
            result,
        )
    }
}

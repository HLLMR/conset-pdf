//! Step 10 scaffold: GUI IPC command contract tests.
//!
//! These tests do not spin up a Tauri runtime yet. They mock the IPC boundary by
//! invoking desktop-gui command handlers directly with `WorkflowRequest` values.

use conset_pdf_contracts::{
    OperationStatus, WorkflowOperation, WorkflowOptions, WorkflowRequest,
};

#[test]
fn gui_parse_stub_returns_not_implemented_contract_response() {
    let request = WorkflowRequest {
        request_id: "req-gui-1".to_owned(),
        session_id: "session-gui-1".to_owned(),
        operation_id: "op-gui-parse-1".to_owned(),
        operation: WorkflowOperation::Parse,
        input_path: "tests/corpus/tier1/sample.pdf".to_owned(),
        output_path: None,
        options: WorkflowOptions::default(),
    };

    let response = conset_pdf_desktop_gui::cmd_parse(request);

    assert_eq!(response.result.status, OperationStatus::Failed);
    assert_eq!(response.result.error_code.as_deref(), Some("NOT_IMPLEMENTED"));
    assert!(!response.contracts_version.is_empty());
}

#[test]
fn gui_extract_stub_returns_not_implemented_contract_response() {
    let request = WorkflowRequest {
        request_id: "req-gui-2".to_owned(),
        session_id: "session-gui-1".to_owned(),
        operation_id: "op-gui-extract-1".to_owned(),
        operation: WorkflowOperation::Extract,
        input_path: "tests/corpus/tier1/sample.pdf".to_owned(),
        output_path: None,
        options: WorkflowOptions::default(),
    };

    let response = conset_pdf_desktop_gui::cmd_extract(request);

    assert_eq!(response.result.status, OperationStatus::Failed);
    assert_eq!(response.result.error_code.as_deref(), Some("NOT_IMPLEMENTED"));
    assert!(response.result.summary.contains("not implemented"));
}

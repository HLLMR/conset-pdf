//! Desktop GUI backend contracts and command stubs.
//!
//! Frontend (TypeScript/React) wiring is intentionally deferred. This crate
//! currently defines command handler signatures that mirror the shared
//! contracts types so the IPC boundary stays stable while implementation
//! details evolve.

use conset_pdf_contracts::{OperationResult, OperationStatus, WorkflowRequest, WorkflowResponse};
use serde_json::{Map, Number, Value};

/// Initializes GUI backend services.
///
/// This is a placeholder hook for future startup tasks such as loading user
/// profiles, opening local cache stores, and connecting Tauri command routes.
pub fn init_gui_backend() {
    // Phase 1: add concrete startup services.
}

/// Tauri command stub for extract operations.
#[must_use]
pub fn cmd_extract(request: WorkflowRequest) -> WorkflowResponse {
    not_implemented_response(request, "extract")
}

/// Tauri command stub for segment operations.
#[must_use]
pub fn cmd_segment(request: WorkflowRequest) -> WorkflowResponse {
    not_implemented_response(request, "segment")
}

/// Tauri command stub for parse operations.
#[must_use]
pub fn cmd_parse(request: WorkflowRequest) -> WorkflowResponse {
    not_implemented_response(request, "parse")
}

/// Placeholder window configuration for the future `src-tauri` runtime layer.
///
/// This captures the canonical shape expected by `tauri.conf.json` and gives a
/// typed Rust-side source for defaults that can be surfaced in settings UI.
#[must_use]
pub fn tauri_window_placeholder_config() -> Value {
    let mut config = Map::new();
    config.insert("label".to_owned(), Value::String("main".to_owned()));
    config.insert("title".to_owned(), Value::String("Conset PDF".to_owned()));
    config.insert("width".to_owned(), Value::Number(Number::from(1200)));
    config.insert("height".to_owned(), Value::Number(Number::from(800)));
    config.insert("resizable".to_owned(), Value::Bool(true));
    config.insert("fullscreen".to_owned(), Value::Bool(false));
    config.insert("decorations".to_owned(), Value::Bool(true));
    Value::Object(config)
}

fn not_implemented_response(request: WorkflowRequest, op_name: &str) -> WorkflowResponse {
    WorkflowResponse::new(
        request.request_id,
        request.session_id,
        request.operation_id,
        OperationResult {
            status: OperationStatus::Failed,
            summary: format!("GUI command not implemented yet: {op_name}"),
            warnings: vec!["Desktop frontend wiring is deferred to the GUI phase".to_owned()],
            error_code: Some("NOT_IMPLEMENTED".to_owned()),
            output_artifacts: vec![],
        },
        vec![],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_contracts::{WorkflowOperation, WorkflowOptions};

    #[test]
    fn parse_stub_returns_not_implemented() {
        let request = WorkflowRequest {
            request_id: "req-1".to_owned(),
            session_id: "session-1".to_owned(),
            operation_id: "op-1".to_owned(),
            operation: WorkflowOperation::Parse,
            input_path: "input.pdf".to_owned(),
            output_path: None,
            options: WorkflowOptions::default(),
            intake_bundle: None,
        };

        let response = cmd_parse(request);
        assert_eq!(response.result.status, OperationStatus::Failed);
        assert_eq!(response.result.error_code.as_deref(), Some("NOT_IMPLEMENTED"));
    }
}

//! ExtractSchedules operation handler (Phase 9.4.C).
//!
//! Reads a `LayoutTranscript` JSON produced by the Extract operation, builds a
//! `DrawingIndex` to identify schedule sheets, then calls
//! [`extract_tables_from_sheet`] on each schedule sheet page to produce a
//! JSON report of extracted tables.
//!
//! # Metadata keys
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `format` | no | Output format: `"json"` (default) |
//!
//! # Output path
//!
//! The output JSON is written to `--output <FILE>`.  If `--output` is omitted
//! the report is printed to stdout inside the normal `WorkflowResponse`.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::{extract_tables_from_sheet, DrawingSegmentEngine, ExtractedTable};
use conset_pdf_ir::LayoutTranscript;

// ── Entry point ───────────────────────────────────────────────────────────────

/// Run the extract-schedules operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::ExtractSchedules,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    if req.options.dry_run {
        record_ended(bundle, req, &started_at, OperationStatus::Succeeded);
        return make_response(
            req,
            OperationStatus::Succeeded,
            "dry_run: argument validation passed — no extraction performed".to_owned(),
            vec![],
            None,
        );
    }

    // ── Load transcript ───────────────────────────────────────────────────────
    let transcript = match read_transcript(&req.input_path) {
        Ok(t) => t,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to read transcript '{}': {e}", req.input_path),
                vec![],
                Some("INVALID_TRANSCRIPT".to_owned()),
            );
        }
    };

    // ── Build drawing index ───────────────────────────────────────────────────
    let drawing_index = DrawingSegmentEngine::build_index(&transcript);
    let sheet_count = drawing_index.sheet_count;

    // ── Extract tables from schedule sheets ───────────────────────────────────
    let pages = transcript.pages();
    let mut all_tables: Vec<ExtractedTable> = Vec::new();
    let mut schedule_sheet_count = 0usize;

    for sheet in &drawing_index.sheets {
        if !sheet.is_schedule_sheet {
            continue;
        }
        schedule_sheet_count += 1;

        // Iterate each page belonging to this sheet.
        for page_idx in sheet.start_page..=sheet.end_page {
            if let Some(page) = pages.get(page_idx) {
                let tables = extract_tables_from_sheet(page, sheet);
                all_tables.extend(tables);
            }
        }
    }

    let table_count = all_tables.len();

    // Serialise tables to a serde_json value first (they derive serde::Serialize).
    let tables_value = match serde_json::to_value(&all_tables) {
        Ok(v) => v,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to serialise tables: {e}"),
                vec![],
                Some("SERIALISE_ERROR".to_owned()),
            );
        }
    };

    let output_value = serde_json::json!({
        "schema_version": "1.0.0",
        "sheet_count": sheet_count,
        "schedule_sheet_count": schedule_sheet_count,
        "table_count": table_count,
        "tables": tables_value
    });

    // ── Serialize output ──────────────────────────────────────────────────────
    let output_json = match serde_json::to_string_pretty(&output_value) {
        Ok(j) => j,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to serialise output: {e}"),
                vec![],
                Some("SERIALISE_ERROR".to_owned()),
            );
        }
    };

    // ── Write to file or embed in response ────────────────────────────────────
    let mut warnings: Vec<String> = Vec::new();
    if let Some(out_path) = &req.output_path {
        if let Err(e) = std::fs::write(out_path, &output_json) {
            warnings.push(format!("Failed to write output to '{out_path}': {e}"));
        }
    }

    let status = if warnings.is_empty() {
        OperationStatus::Succeeded
    } else {
        OperationStatus::SucceededWithWarnings
    };

    record_ended(bundle, req, &started_at, status.clone());
    make_response(
        req,
        status,
        format!(
            "Extracted {table_count} table(s) from {schedule_sheet_count} schedule sheet(s) \
             (of {sheet_count} total)"
        ),
        warnings,
        None,
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_transcript(path: &str) -> Result<LayoutTranscript, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error reading transcript: {e}"))?;
    serde_json::from_str::<LayoutTranscript>(&json)
        .map_err(|e| format!("JSON parse error: {e}"))
}

fn record_ended(
    bundle: &mut AuditBundle,
    req: &WorkflowRequest,
    started_at: &chrono::DateTime<chrono::Utc>,
    result: OperationStatus,
) {
    let ended_at = Utc::now();
    let elapsed_ms = (ended_at - *started_at).num_milliseconds();
    let duration_ms: u64 = u64::try_from(elapsed_ms).unwrap_or(0);
    bundle.add_event(AuditEvent::new(AuditEventData::OperationEnded {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::ExtractSchedules,
        ended_at_utc: ended_at.to_rfc3339(),
        duration_ms,
        result,
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
        OperationResult { status, summary, warnings, error_code, output_artifacts: vec![] },
        vec![],
    )
}

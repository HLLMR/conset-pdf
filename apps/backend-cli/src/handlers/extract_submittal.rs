//! ExtractSubmittal operation handler (Sprint 10.4).
//!
//! Reads a `LayoutTranscript` JSON (produced by `extract`) and a
//! `SubmittalIndex` JSON (produced by `index-submittal`), extracts per-unit
//! KV pairs and performance tables, then writes the assembled
//! `EquipmentDataset` to the requested output path in JSON or CSV format.
//!
//! An optional audit bundle directory (`--audit-bundle`) receives:
//! - `unit-report.json` — per-unit record counts, confidence, and warnings
//! - `metrics.json` — totals and elapsed time
//!
//! # Metadata keys (in `WorkflowOptions.metadata`)
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `index_path` | **yes** | Path to the `SubmittalIndex` JSON |
//! | `format` | no | `"json"` (default) or `"csv"` |
//! | `audit_bundle_dir` | no | Directory for unit-report + metrics |

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::{
    build_equipment_dataset, dataset_to_csv, dataset_to_json, extract_kv_pairs, extract_unit_tables,
};
use conset_pdf_ir::{LayoutTranscript, SubmittalIndex};
use std::path::Path;

// ── Entry point ───────────────────────────────────────────────────────────────

/// Run the extract-submittal operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::ExtractSubmittal,
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

    // Require --output.
    let output_path = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <FILE> is required for the extract-submittal operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Resolve metadata fields.
    let index_path = meta_value(&req.options.metadata, "index_path");
    let format = meta_value(&req.options.metadata, "format");
    let format = if format.is_empty() { "json".to_owned() } else { format };
    let audit_bundle_dir = meta_value(&req.options.metadata, "audit_bundle_dir");

    if index_path.is_empty() {
        record_ended(bundle, req, &started_at, OperationStatus::Failed);
        return make_response(
            req,
            OperationStatus::Failed,
            "--index <FILE> is required for the extract-submittal operation".to_owned(),
            vec![],
            Some("MISSING_INDEX_PATH".to_owned()),
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

    // ── Load SubmittalIndex ───────────────────────────────────────────────────
    let submittal_index = match read_submittal_index(&index_path) {
        Ok(idx) => idx,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("Failed to read submittal index '{index_path}': {e}"),
                vec![],
                Some("INVALID_INDEX".to_owned()),
            );
        }
    };

    // ── Per-unit extraction ───────────────────────────────────────────────────
    let pages = transcript.pages();
    let total_pages = pages.len();

    let mut kv_by_unit: Vec<(usize, Vec<conset_pdf_ir::KvPair>)> = Vec::new();
    let mut tables_by_unit: Vec<(usize, Vec<conset_pdf_engine::ExtractedTable>)> = Vec::new();

    for (unit_idx, unit) in submittal_index.units.iter().enumerate() {
        if unit.is_cover {
            continue;
        }
        let start = unit.start_page;
        let end = unit.end_page.min(total_pages.saturating_sub(1));
        let unit_pages: Vec<&_> = pages[start..=end].iter().collect();

        let kv = extract_kv_pairs(&unit_pages);
        let tbls = extract_unit_tables(&unit_pages, unit);

        kv_by_unit.push((unit_idx, kv));
        tables_by_unit.push((unit_idx, tbls));
    }

    // ── Assemble dataset ──────────────────────────────────────────────────────
    let dataset = build_equipment_dataset(&submittal_index, &tables_by_unit, &kv_by_unit);
    let record_count = dataset.record_count;
    let unit_count = dataset.unit_count;

    // ── Serialise output ──────────────────────────────────────────────────────
    let serialised = if format == "csv" {
        dataset_to_csv(&dataset)
    } else {
        dataset_to_json(&dataset)
    };

    let mut warnings: Vec<String> = Vec::new();

    if let Err(e) = std::fs::write(&output_path, &serialised) {
        warnings.push(format!("Failed to write output to '{output_path}': {e}"));
    }

    // ── Audit bundle ──────────────────────────────────────────────────────────
    if !audit_bundle_dir.is_empty() {
        write_audit_bundle(&audit_bundle_dir, &dataset, &started_at, &mut warnings);
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
            "Extracted {record_count} record(s) from {unit_count} unit(s) \
             (format: {format})"
        ),
        warnings,
        None,
    )
}

// ── Audit bundle ──────────────────────────────────────────────────────────────

fn write_audit_bundle(
    dir: &str,
    dataset: &conset_pdf_ir::EquipmentDataset,
    started_at: &chrono::DateTime<chrono::Utc>,
    warnings: &mut Vec<String>,
) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        warnings.push(format!("Failed to create audit bundle dir '{dir}': {e}"));
        return;
    }

    // unit-report.json
    let unit_report = serde_json::json!({
        "schema_version": "1.0.0",
        "packet_name": dataset.packet_name,
        "unit_count": dataset.unit_count,
        "total_record_count": dataset.record_count,
        "units": dataset.unit_summaries.iter().map(|s| serde_json::json!({
            "unit_tag": s.unit_tag,
            "record_count": s.record_count,
            "avg_confidence": format!("{:.4}", s.avg_confidence),
            "table_record_count": s.table_record_count,
            "kv_record_count": s.kv_record_count,
            "warnings": s.warnings,
        })).collect::<Vec<_>>(),
    });
    let unit_report_path = Path::new(dir).join("unit-report.json");
    if let Err(e) =
        std::fs::write(&unit_report_path, serde_json::to_string_pretty(&unit_report).unwrap())
    {
        warnings.push(format!("Failed to write unit-report.json: {e}"));
    }

    // metrics.json
    let elapsed_ms = (Utc::now() - *started_at).num_milliseconds();
    let metrics = serde_json::json!({
        "schema": "metrics/v1",
        "packet_name": dataset.packet_name,
        "unit_count": dataset.unit_count,
        "total_record_count": dataset.record_count,
        "elapsed_ms": elapsed_ms,
    });
    let metrics_path = Path::new(dir).join("metrics.json");
    if let Err(e) =
        std::fs::write(&metrics_path, serde_json::to_string_pretty(&metrics).unwrap())
    {
        warnings.push(format!("Failed to write metrics.json: {e}"));
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_transcript(path: &str) -> Result<LayoutTranscript, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error: {e}"))?;
    serde_json::from_str::<LayoutTranscript>(&json).map_err(|e| format!("JSON parse error: {e}"))
}

fn read_submittal_index(path: &str) -> Result<SubmittalIndex, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("I/O error: {e}"))?;
    serde_json::from_str::<SubmittalIndex>(&json).map_err(|e| format!("JSON parse error: {e}"))
}

fn meta_value(
    metadata: &[conset_pdf_contracts::KeyValuePair],
    key: &str,
) -> String {
    metadata
        .iter()
        .find(|kv| kv.key == key)
        .map(|kv| kv.value.clone())
        .unwrap_or_default()
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
        operation: WorkflowOperation::ExtractSubmittal,
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

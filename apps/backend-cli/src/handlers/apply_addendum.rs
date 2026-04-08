//! ApplyAddendum operation handler (Phase 7).
//!
//! Loads an [`AddendumManifest`] JSON, delegates to [`SpecsPatchOrchestrator`]
//! for extraction → segmentation → parse → edit → render → stitch, and writes
//! the resulting [`AddendumResult`] to stdout via the normal `WorkflowResponse`
//! serialisation path.  Audit artifacts are written to `--audit-bundle` when
//! that option is provided.
//!
//! # Metadata keys
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `manifest_path` | yes | Path to the [`AddendumManifest`] JSON file |
//! | `original_path` | yes | Path to the source spec PDF (mirrors `input_path`) |
//! | `audit_bundle_dir` | no | Directory for audit artifacts |

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::SpecsPatchOrchestrator;
use conset_pdf_ir::{AddendumManifest, AddendumResult, DiagnosticEvent};

/// Run the apply-addendum operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::SpecsPatch,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    let meta = &req.options.metadata;

    // ── Require manifest_path ─────────────────────────────────────────────────
    let manifest_path = match meta.iter().find(|kv| kv.key == "manifest_path") {
        Some(kv) => kv.value.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--addendum <JSON> (manifest_path key) is required".to_owned(),
                vec![],
                Some("MISSING_MANIFEST_PATH".to_owned()),
            );
        }
    };

    // ── Load and validate AddendumManifest ────────────────────────────────────
    let manifest: AddendumManifest =
        match std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("cannot read manifest '{}': {e}", manifest_path))
            .and_then(|text| {
                serde_json::from_str(&text)
                    .map_err(|e| format!("invalid AddendumManifest JSON at '{}': {e}", manifest_path))
            }) {
            Ok(m) => m,
            Err(msg) => {
                record_ended(bundle, req, &started_at, OperationStatus::Failed);
                return make_response(
                    req,
                    OperationStatus::Failed,
                    msg,
                    vec![],
                    Some("MANIFEST_READ_ERROR".to_owned()),
                );
            }
        };

    if manifest.sections.is_empty() {
        record_ended(bundle, req, &started_at, OperationStatus::Failed);
        return make_response(
            req,
            OperationStatus::Failed,
            "AddendumManifest.sections is empty — nothing to do".to_owned(),
            vec![],
            Some("EMPTY_MANIFEST".to_owned()),
        );
    }

    // ── Build optional audit bundle dir ───────────────────────────────────────
    let audit_bundle_dir: Option<std::path::PathBuf> = meta
        .iter()
        .find(|kv| kv.key == "audit_bundle_dir")
        .map(|kv| std::path::PathBuf::from(&kv.value));

    if let Some(ref dir) = audit_bundle_dir {
        if let Err(e) = std::fs::create_dir_all(dir) {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("cannot create audit bundle directory '{}': {e}", dir.display()),
                vec![],
                Some("AUDIT_DIR_CREATE_ERROR".to_owned()),
            );
        }
    }

    // ── Delegate to SpecsPatchOrchestrator ────────────────────────────────────
    let result = match SpecsPatchOrchestrator::run(
        &req.input_path,
        manifest,
        req.output_path.as_deref(),
        req.options.dry_run,
    ) {
        Ok(r) => r,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                e,
                vec![],
                Some("ORCHESTRATOR_ERROR".to_owned()),
            );
        }
    };

    // ── Write audit bundle artifacts ───────────────────────────────────────────
    let mut warnings: Vec<String> = vec![];
    if let Some(ref dir) = audit_bundle_dir {
        if let Err(e) = write_audit_artifacts(dir, &result) {
            warnings.push(format!("audit bundle write error: {e}"));
        }
    }

    // ── Build response summary ────────────────────────────────────────────────
    let status = if result.failed == 0 {
        if warnings.is_empty() {
            OperationStatus::Succeeded
        } else {
            OperationStatus::SucceededWithWarnings
        }
    } else if result.succeeded == 0 {
        OperationStatus::Failed
    } else {
        OperationStatus::SucceededWithWarnings
    };

    let summary = build_summary(&result, req.options.dry_run);

    record_ended(bundle, req, &started_at, status.clone());
    make_response(req, status, summary, warnings, None)
}

// ── Audit bundle ─────────────────────────────────────────────────────────────

/// Write `change-report.json`, `diagnostics.jsonl`, and `metrics.json` to `dir`.
///
/// `diagnostics.jsonl` format:
/// - Line 1: schema header JSON object (not a `DiagnosticEvent`)
/// - Lines 2+: one serialized `DiagnosticEvent` per line
/// - If serialized size would exceed 8 MB, a sentinel truncation line is
///   appended and no further events are written.
///
/// `metrics.json` is an executive summary roll-up derived from `diagnostics`.
fn write_audit_artifacts(
    dir: &std::path::Path,
    result: &AddendumResult,
) -> Result<(), String> {
    let change_report = serde_json::to_string_pretty(result)
        .map_err(|e| format!("serialize change-report: {e}"))?;
    std::fs::write(dir.join("change-report.json"), change_report)
        .map_err(|e| format!("write change-report.json: {e}"))?;

    const MAX_BYTES: usize = 8 * 1024 * 1024; // 8 MB safety cap

    let header = format!(
        r#"{{"schema":"diagnostics/v1","pipeline_version":"{}","generated_at":"{}"}}"#,
        env!("CARGO_PKG_VERSION"),
        Utc::now().to_rfc3339(),
    );
    let mut lines: Vec<String> = vec![header];
    let mut running_bytes = lines[0].len() + 1; // +1 for the newline

    let mut events_written: usize = 0;
    let mut truncated = false;

    for event in &result.diagnostics {
        let serialized = serde_json::to_string(event)
            .map_err(|e| format!("serialize diagnostic event: {e}"))?;
        running_bytes += serialized.len() + 1;
        if running_bytes > MAX_BYTES {
            truncated = true;
            break;
        }
        events_written += 1;
        lines.push(serialized);
    }

    if truncated {
        lines.push(format!(
            r#"{{"truncated":true,"reason":"size_cap_exceeded","events_written":{events_written}}}"#
        ));
    }

    std::fs::write(dir.join("diagnostics.jsonl"), lines.join("\n"))
        .map_err(|e| format!("write diagnostics.jsonl: {e}"))?;

    // ── metrics.json ──────────────────────────────────────────────────────────
    let metrics = build_metrics(result);
    let metrics_json = serde_json::to_string_pretty(&metrics)
        .map_err(|e| format!("serialize metrics: {e}"))?;
    std::fs::write(dir.join("metrics.json"), metrics_json + "\n")
        .map_err(|e| format!("write metrics.json: {e}"))?;

    Ok(())
}

// ── Metrics roll-up ───────────────────────────────────────────────────────────

/// Derive the `metrics.json` executive summary JSON from `AddendumResult`.
///
/// All fields are derived from `result.diagnostics` so that `metrics.json` and
/// `diagnostics.jsonl` remain the single source of truth for post-run data.
///
/// Schema version: `"metrics/v1"`.
fn build_metrics(result: &AddendumResult) -> serde_json::Value {
    use std::collections::HashMap;

    let mut total_pages_input: usize = 0;
    let mut extraction_elapsed_ms: u64 = 0;
    let mut sections_detected: usize = 0;
    let mut section_coverage_ratio: f64 = 0.0;

    // section_id → render elapsed_ms
    let mut render_ms_map: HashMap<String, u64> = HashMap::new();
    // section_id → (elapsed_ms, pages_removed, pages_inserted)
    let mut stitch_map: HashMap<String, (u64, usize, usize)> = HashMap::new();
    // (section_id, node_count, unclassified_count) in emission order
    let mut parse_rows: Vec<(String, usize, usize)> = Vec::new();

    for event in &result.diagnostics {
        match event {
            DiagnosticEvent::Extraction(e) => {
                total_pages_input = e.page_count;
                extraction_elapsed_ms = e.elapsed_ms;
            }
            DiagnosticEvent::Segmentation(s) => {
                sections_detected = s.section_count;
                section_coverage_ratio = s.coverage_ratio;
            }
            DiagnosticEvent::Parse(p) => {
                parse_rows.push((
                    p.section_id.clone(),
                    p.node_count,
                    p.node_distribution.unclassified,
                ));
            }
            DiagnosticEvent::Render(r) => {
                render_ms_map.insert(r.section_id.clone(), r.elapsed_ms);
            }
            DiagnosticEvent::Stitch(s) => {
                stitch_map.insert(
                    s.section_id.clone(),
                    (s.elapsed_ms, s.pages_removed, s.pages_inserted),
                );
            }
            _ => {}
        }
    }

    let pages_removed_total: usize = stitch_map.values().map(|(_, r, _)| *r).sum();
    let pages_inserted_total: usize = stitch_map.values().map(|(_, _, i)| *i).sum();
    let total_pages_output = total_pages_input
        .saturating_sub(pages_removed_total)
        .saturating_add(pages_inserted_total);

    let render_elapsed_total: u64 = render_ms_map.values().sum();
    let stitch_elapsed_total: u64 = stitch_map.values().map(|(ms, _, _)| *ms).sum();
    let total_elapsed_ms = extraction_elapsed_ms + render_elapsed_total + stitch_elapsed_total;

    let sections_patched = parse_rows.len();

    let per_section: Vec<serde_json::Value> = parse_rows
        .into_iter()
        .map(|(id, node_count, unclassified_count)| {
            let unclassified_ratio = if node_count == 0 {
                0.0_f64
            } else {
                (unclassified_count as f64 / node_count as f64 * 1_000.0).round() / 1_000.0
            };
            let render_ms = render_ms_map.get(&id).copied();
            let stitch_ms = stitch_map.get(&id).map(|(ms, _, _)| *ms).unwrap_or(0);
            serde_json::json!({
                "section_id": id,
                "parse_node_count": node_count,
                "unclassified_count": unclassified_count,
                "unclassified_ratio": unclassified_ratio,
                "render_ms": render_ms,
                "stitch_ms": stitch_ms,
            })
        })
        .collect();

    serde_json::json!({
        "schema": "metrics/v1",
        "generated_at": Utc::now().to_rfc3339(),
        "total_pages_input": total_pages_input,
        "total_pages_output": total_pages_output,
        "sections_detected": sections_detected,
        "sections_patched": sections_patched,
        "section_coverage_ratio": section_coverage_ratio,
        "total_elapsed_ms": total_elapsed_ms,
        "per_section": per_section,
    })
}

// ── Summary text ─────────────────────────────────────────────────────────────

fn build_summary(result: &AddendumResult, dry_run: bool) -> String {
    let prefix = if dry_run { "dry_run: " } else { "" };
    let desc = result
        .manifest_description
        .as_deref()
        .map(|d| format!(" ({d})"))
        .unwrap_or_default();
    let output = result
        .output_path
        .as_deref()
        .map(|p| format!("; output: '{p}'"))
        .unwrap_or_default();
    format!(
        "{prefix}apply-addendum{desc}: {}/{} section(s) patched successfully{}",
        result.succeeded, result.total_sections, output
    )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn record_ended(
    bundle: &mut AuditBundle,
    req: &WorkflowRequest,
    started_at: &chrono::DateTime<Utc>,
    status: OperationStatus,
) {
    let ended_at = Utc::now();
    let elapsed = (ended_at - *started_at).num_milliseconds();
    bundle.add_event(AuditEvent::new(AuditEventData::OperationEnded {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::SpecsPatch,
        ended_at_utc: ended_at.to_rfc3339(),
        duration_ms: u64::try_from(elapsed).unwrap_or(0),
        result: status,
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

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
use conset_pdf_ir::{AddendumManifest, AddendumResult};

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

/// Write `change-report.json` (and later per-section artifacts) to `dir`.
fn write_audit_artifacts(
    dir: &std::path::Path,
    result: &AddendumResult,
) -> Result<(), String> {
    let change_report = serde_json::to_string_pretty(result)
        .map_err(|e| format!("serialize change-report: {e}"))?;
    std::fs::write(dir.join("change-report.json"), change_report)
        .map_err(|e| format!("write change-report.json: {e}"))?;
    Ok(())
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

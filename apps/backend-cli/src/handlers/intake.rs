//! Intake operation handler — Stage 0 triage: rotation detection and normalization.
//!
//! Translates the [`WorkflowRequest`] envelope into an [`IntakeBundle`], delegates
//! to [`Stage0Normalizer`], and wraps the per-file results in a
//! [`NormalizedIntakeBundle`] written (when `--output` is given) as JSON.

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    intake::{
        IntakeBundle, IntakeFile, IntakeRole, IssueSeverity, NormalizedIntakeBundle,
        DocumentClass, TriageConfidence, INTAKE_SCHEMA_VERSION,
    },
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
    CONTRACTS_VERSION,
};
use conset_pdf_engine::Stage0Normalizer;

/// Run Stage 0 intake triage for the given request.
///
/// If `req.intake_bundle` is `Some` that bundle is used as-is; otherwise a
/// single-file bundle is constructed from `req.input_path`.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Intake,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    // ── Build intake bundle ───────────────────────────────────────────────────
    let effective_bundle: IntakeBundle = req.intake_bundle.clone().unwrap_or_else(|| {
        IntakeBundle {
            files: vec![IntakeFile {
                path: req.input_path.clone(),
                role: IntakeRole::Unknown,
            }],
            declared_order: None,
        }
    });

    // ── Contracts → engine ────────────────────────────────────────────────────
    let norm = Stage0Normalizer::normalize(&effective_bundle, req.options.dry_run);

    // ── Engine → contracts ────────────────────────────────────────────────────
    let mut warnings: Vec<String> = Vec::new();
    let mut all_issues = Vec::new();
    let mut total_pages: u32 = 0;
    let mut total_normalized: usize = 0;
    let mut any_fatal = false;

    for file_result in &norm.files {
        total_pages = total_pages.saturating_add(file_result.page_count);
        total_normalized = total_normalized.saturating_add(file_result.rotations_normalized);
        for issue in &file_result.issues {
            if issue.severity == IssueSeverity::Fatal {
                any_fatal = true;
                warnings.push(format!(
                    "[{}] {} — {}",
                    issue.code, issue.description,
                    issue.suggested_action.as_deref().unwrap_or("")
                ));
            } else {
                all_issues.push(issue.clone());
            }
        }
    }

    let (status, summary, error_code) = if any_fatal {
        (
            OperationStatus::Failed,
            format!(
                "Intake triage failed for '{}': one or more files could not be processed",
                req.input_path
            ),
            Some("INTAKE_FATAL_ISSUE".to_owned()),
        )
    } else if req.options.dry_run {
        (
            OperationStatus::Succeeded,
            format!(
                "dry_run: detected {} rotated page(s) across {total_pages} page(s) in {} file(s)",
                all_issues.len(),
                effective_bundle.files.len()
            ),
            None,
        )
    } else {
        let status = if warnings.is_empty() {
            OperationStatus::Succeeded
        } else {
            OperationStatus::SucceededWithWarnings
        };
        (
            status,
            format!(
                "Intake triage complete: {total_pages} page(s), \
                 {total_normalized} rotation(s) normalized, {} issue(s) recorded",
                all_issues.len()
            ),
            None,
        )
    };

    // ── Build NormalizedIntakeBundle and optionally write JSON ────────────────
    if let Some(out_path) = &req.output_path {
        let normalized_bundle = NormalizedIntakeBundle {
            schema_version: INTAKE_SCHEMA_VERSION.to_owned(),
            bundle_id: format!("intake-{}", started_at.timestamp_millis()),
            document_class: DocumentClass::Unknown,
            triage_confidence: TriageConfidence::Low,
            source_path: req.input_path.clone(),
            page_count: Some(total_pages),
            detected_discipline: None,
            detected_spec_section: None,
            addenda_ordering: None,
            advisory_manifest: None,
            issues: all_issues,
            contracts_version: CONTRACTS_VERSION.to_owned(),
        };
        match serde_json::to_string_pretty(&normalized_bundle) {
            Ok(json) => {
                if let Err(e) = std::fs::write(out_path, &json) {
                    warnings.push(format!(
                        "Failed to write intake bundle JSON to '{out_path}': {e}"
                    ));
                }
            }
            Err(e) => {
                warnings.push(format!("Failed to serialize intake bundle: {e}"));
            }
        }
    }

    record_ended(bundle, req, &started_at, status.clone());
    make_response(req, status, summary, warnings, error_code)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
        operation: WorkflowOperation::Intake,
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

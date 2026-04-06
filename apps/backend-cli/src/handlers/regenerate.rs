//! Regenerate operation handler.
//!
//! Reads a [`ParsedDocument`] JSON and a [`SpecChromeMetadata`] JSON, selects a
//! section by ID (or the first section), renders it to PDF via
//! [`SectionRenderer`], and writes the PDF bytes to `--output`.
//!
//! # Metadata keys
//!
//! | Key | Required | Description |
//! |-----|----------|-------------|
//! | `chrome_metadata_path` | yes | Path to the [`SpecChromeMetadata`] JSON file |
//! | `section_filter` | no | CSI section ID to render; first section when absent |
//! | `font_family` | no | CSS font-family override |
//! | `font_size_pt` | no | Body font size in points (string representation of u8) |

use chrono::Utc;
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    OperationResult, OperationStatus, WorkflowOperation, WorkflowRequest, WorkflowResponse,
};
use conset_pdf_engine::SectionRenderer;
use conset_pdf_ir::{ParsedDocument, RenderConfig, SpecChromeMetadata};

/// Run the regenerate operation for the given request.
pub fn run(req: &WorkflowRequest, bundle: &mut AuditBundle) -> WorkflowResponse {
    let started_at = Utc::now();

    bundle.add_event(AuditEvent::new(AuditEventData::OperationStarted {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Regenerate,
        started_at_utc: started_at.to_rfc3339(),
        page_count: None,
        file_size_bucket: None,
        detection_source: None,
    }));

    // `output_path` is required.
    let output_path = match &req.output_path {
        Some(p) => p.clone(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--output <FILE> is required for the regenerate operation".to_owned(),
                vec![],
                Some("MISSING_OUTPUT_PATH".to_owned()),
            );
        }
    };

    // Resolve metadata keys.
    let meta = &req.options.metadata;

    let chrome_metadata_path = match meta
        .iter()
        .find(|kv| kv.key == "chrome_metadata_path")
        .map(|kv| kv.value.as_str())
    {
        Some(p) => p.to_owned(),
        None => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                "--chrome-metadata <FILE> (chrome_metadata_path key) is required".to_owned(),
                vec![],
                Some("MISSING_CHROME_METADATA_PATH".to_owned()),
            );
        }
    };

    let section_filter = meta.iter().find(|kv| kv.key == "section_filter").map(|kv| kv.value.clone());

    // ── Build RenderConfig from optional metadata overrides ───────────────────
    let mut render_config = RenderConfig::default();
    if let Some(font) = meta.iter().find(|kv| kv.key == "font_family") {
        render_config.font_family = font.value.clone();
    }
    if let Some(fs_kv) = meta.iter().find(|kv| kv.key == "font_size_pt") {
        if let Ok(fs) = fs_kv.value.parse::<u8>() {
            render_config.font_size_pt = fs;
        }
    }

    // ── Load ParsedDocument ───────────────────────────────────────────────────
    let doc: ParsedDocument = match std::fs::read_to_string(&req.input_path)
        .map_err(|e| format!("cannot read '{}': {e}", req.input_path))
        .and_then(|text| {
            serde_json::from_str(&text)
                .map_err(|e| format!("invalid ParsedDocument JSON at '{}': {e}", req.input_path))
        }) {
        Ok(d) => d,
        Err(msg) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                msg,
                vec![],
                Some("INPUT_READ_ERROR".to_owned()),
            );
        }
    };

    // ── Load SpecChromeMetadata ───────────────────────────────────────────────
    let chrome_meta: SpecChromeMetadata = match std::fs::read_to_string(&chrome_metadata_path)
        .map_err(|e| format!("cannot read chrome-metadata '{}': {e}", chrome_metadata_path))
        .and_then(|text| {
            serde_json::from_str(&text).map_err(|e| {
                format!("invalid SpecChromeMetadata JSON at '{}': {e}", chrome_metadata_path)
            })
        }) {
        Ok(m) => m,
        Err(msg) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                msg,
                vec![],
                Some("CHROME_METADATA_READ_ERROR".to_owned()),
            );
        }
    };

    // ── Select target section ─────────────────────────────────────────────────
    let section_ast = {
        let candidate = if let Some(ref id) = section_filter {
            doc.sections.iter().find(|s| &s.section_id == id)
        } else {
            doc.sections.first()
        };
        match candidate {
            Some(s) => s.clone(),
            None => {
                let msg = match section_filter {
                    Some(id) => format!("section '{}' not found in ParsedDocument", id),
                    None => "ParsedDocument contains no sections".to_owned(),
                };
                record_ended(bundle, req, &started_at, OperationStatus::Failed);
                return make_response(
                    req,
                    OperationStatus::Failed,
                    msg,
                    vec![],
                    Some("SECTION_NOT_FOUND".to_owned()),
                );
            }
        }
    };

    // ── Dry run: build HTML only, skip Chrome ─────────────────────────────────
    if req.options.dry_run {
        let renderer = SectionRenderer::new(render_config);
        let result = renderer.dry_run(&section_ast, &chrome_meta);
        let warning_summary = result.warnings.join("; ");
        record_ended(bundle, req, &started_at, OperationStatus::Succeeded);
        return make_response(
            req,
            OperationStatus::Succeeded,
            format!("dry_run: {warning_summary}"),
            result.warnings,
            None,
        );
    }

    // ── Render section to PDF ─────────────────────────────────────────────────
    let renderer = SectionRenderer::new(render_config);
    let render_result = match renderer.render(&section_ast, &chrome_meta) {
        Ok(r) => r,
        Err(e) => {
            record_ended(bundle, req, &started_at, OperationStatus::Failed);
            return make_response(
                req,
                OperationStatus::Failed,
                format!("render failed: {e}"),
                vec![],
                Some("RENDER_ERROR".to_owned()),
            );
        }
    };

    // ── Write PDF bytes ───────────────────────────────────────────────────────
    if let Err(e) = std::fs::write(&output_path, &render_result.pdf_bytes) {
        record_ended(bundle, req, &started_at, OperationStatus::Failed);
        return make_response(
            req,
            OperationStatus::Failed,
            format!("cannot write output PDF '{}': {e}", output_path),
            vec![],
            Some("OUTPUT_WRITE_ERROR".to_owned()),
        );
    }

    let (status, summary) = if render_result.warnings.is_empty() {
        (
            OperationStatus::Succeeded,
            format!(
                "rendered section '{}' → '{}' ({} pages estimated)",
                section_ast.section_id, output_path, render_result.page_count_estimate
            ),
        )
    } else {
        (
            OperationStatus::SucceededWithWarnings,
            format!(
                "rendered section '{}' → '{}' with {} warning(s)",
                section_ast.section_id,
                output_path,
                render_result.warnings.len()
            ),
        )
    };

    record_ended(bundle, req, &started_at, status.clone());
    make_response(req, status, summary, render_result.warnings, None)
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn record_ended(
    bundle: &mut AuditBundle,
    req: &WorkflowRequest,
    started_at: &chrono::DateTime<Utc>,
    result: OperationStatus,
) {
    let ended_at = Utc::now();
    let duration_ms =
        u64::try_from((ended_at - *started_at).num_milliseconds()).unwrap_or(0);
    bundle.add_event(AuditEvent::new(AuditEventData::OperationEnded {
        session_id: req.session_id.clone(),
        operation_id: req.operation_id.clone(),
        operation: WorkflowOperation::Regenerate,
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

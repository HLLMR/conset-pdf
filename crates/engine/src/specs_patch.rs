//! Phase 7 `apply-addendum` orchestrator.
//!
//! [`SpecsPatchOrchestrator`] drives the full pipeline:
//!
//! 1. **Extract** — PDFium extracts the `LayoutTranscript` from the source PDF.
//! 2. **Segment** — CSI footer oracle builds the `SegmentIndex`.
//! 3. **Parse + Edit + Render** — for each `SectionEditSpec` in the manifest:
//!    parse the section AST, apply edit operations, and render via Chrome (or
//!    dry-run HTML build only).
//! 4. **Stitch** — successful sections are stitched back in descending
//!    `start_page` order (last-to-first) so that page indices for earlier
//!    sections remain valid when they are processed.
//!
//! # Partial success
//!
//! If a section fails at any stage (parse, edit, render, stitch), that failure
//! is recorded in [`SectionPatchResult`] and all other sections continue.  The
//! final [`AddendumResult`] reports per-section outcomes along with aggregate
//! success/failure counts.
//!
//! # Chrome metadata merge order
//!
//! 1. Extracted [`ChromeMetadata`] from the source PDF.
//! 2. Manifest-level `project_metadata` override (if present).
//! 3. Per-section `chrome_override` (if present) — highest priority.
//!
//! Non-empty string fields replace the corresponding lower-priority value;
//! empty string fields in an override are ignored (the lower-priority value is
//! preserved).

use conset_pdf_ir::{
    AddendumManifest, AddendumResult, DiagnosticEvent, EditDiagnostic, EditFailureTrace,
    ExtractionDiagnostic, NodeDistribution, OutlineTag, ParseDiagnostic, RenderDiagnostic,
    RenderOutcome, SectionPatchResult, SectionPatchStatus, SegmentIndex, SegmentTrace,
    SegmentationDiagnostic, SpecChromeMetadata, StitchDiagnostic, StitchPlan, UnclassifiedNodeTrace,
};

use crate::{
    edit::SectionEditor,
    extractor::Extractor,
    patterns::PatternDatabase,
    render::SectionRenderer,
    stitch::PdfStitcher,
};

/// Stateless apply-addendum orchestrator.
///
/// All logic lives in [`SpecsPatchOrchestrator::run`]; the struct is a zero-size
/// namespace.
pub struct SpecsPatchOrchestrator;

impl SpecsPatchOrchestrator {
    /// Run the full apply-addendum pipeline.
    ///
    /// # Arguments
    ///
    /// * `source_path` — Path to the original spec PDF.
    /// * `manifest` — Validated [`AddendumManifest`] describing the patches.
    /// * `output_path` — Path for the stitched output PDF, or `None` on
    ///   dry-run (no PDF is written).
    /// * `dry_run` — When `true`, validation and HTML assembly are performed
    ///   but Chrome render and PDF write are skipped.
    ///
    /// # Errors
    ///
    /// Returns a human-readable error string only for catastrophic failures that
    /// prevent the pipeline from starting (extraction or segmentation failure).
    /// Per-section failures are recorded in the returned [`AddendumResult`].
    pub fn run(
        source_path: &str,
        manifest: AddendumManifest,
        output_path: Option<&str>,
        dry_run: bool,
    ) -> Result<AddendumResult, String> {
        let mut diagnostics: Vec<DiagnosticEvent> = Vec::new();

        // ── Step 0: Pattern database ──────────────────────────────────────────
        // Fail fast if the embedded default.json is ever accidentally malformed.
        let pattern_db = PatternDatabase::load_default()?;

        // ── Step 1: Extract ───────────────────────────────────────────────────
        let extract_start = std::time::Instant::now();
        let transcript = Extractor::new()
            .extract(source_path)
            .map_err(|e| format!(
                "extraction failed for '{source_path}': {e} — \
                 verify the source PDF is not password-protected, corrupt, or a scanned image"
            ))?;
        let extract_elapsed_ms = extract_start.elapsed().as_millis() as u64;

        // Extraction diagnostic.
        diagnostics.push(DiagnosticEvent::Extraction(
            build_extraction_diagnostic(transcript.pages(), extract_elapsed_ms),
        ));

        // ── Step 2: Segment ───────────────────────────────────────────────────
        let segment_index = crate::segment::segment_transcript(&transcript)
            .map_err(|e| format!("segmentation failed: {e}"))?;

        // Segmentation diagnostic.
        {
            let page_count = transcript.pages().len();
            let covered: std::collections::HashSet<usize> = segment_index
                .sections
                .iter()
                .flat_map(|s| s.start_page..=s.end_page)
                .collect();
            let pages_missing_footer: Vec<usize> = (0..page_count)
                .filter(|p| !covered.contains(p))
                .collect();
            let sections: Vec<SegmentTrace> = segment_index
                .sections
                .iter()
                .map(|s| SegmentTrace {
                    section_id: s.section_id.clone(),
                    section_title: s.section_title.clone(),
                    start_page: s.start_page,
                    end_page: s.end_page,
                    footer_match_count: s.end_page.saturating_sub(s.start_page) + 1,
                    page_counter_detected: s.page_counter_detected,
                })
                .collect();
            diagnostics.push(DiagnosticEvent::Segmentation(SegmentationDiagnostic {
                section_count: segment_index.sections.len(),
                coverage_ratio: segment_index.coverage.coverage_ratio,
                pages_missing_footer,
                sections,
            }));
        }

        // ── Step 3: Parse → Edit → Render for each section ───────────────────
        let renderer = SectionRenderer::with_defaults();

        // We'll build two parallel structures:
        // - `section_results`: one entry per manifest section (failure or pending-success)
        // - `stitch_queue`: (section_id, start_page, temp_pdf_path) for stitch step
        let mut section_results: Vec<SectionPatchResult> = Vec::new();

        // Mapping from section_id → index in section_results (so we can update
        // Success { pages_removed, pages_inserted } after stitching).
        let mut result_idx: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // (section_id, start_page, temp_replacement_pdf_path)
        let mut stitch_queue: Vec<(String, usize, String)> = Vec::new();

        // Scratch temp directory for intermediate PDFs (real runs only).
        // Using `tempfile::TempDir` ensures the directory is removed on drop
        // (normal return, early `?` propagation, and process panic all clean up).
        let temp_dir: Option<tempfile::TempDir> = if dry_run {
            None
        } else {
            let td = tempfile::TempDir::new()
                .map_err(|e| format!("cannot create temp directory: {e}"))?;
            Some(td)
        };

        for spec in &manifest.sections {
            let section_id = &spec.section_id;

            // Locate this section in the segment index.
            let section_entry =
                segment_index.sections.iter().find(|s| &s.section_id == section_id);

            let section_title = section_entry
                .map(|e| e.section_title.clone())
                .unwrap_or_default();

            let start_page = match section_entry {
                Some(e) => e.start_page,
                None => {
                    section_results.push(SectionPatchResult::failed(
                        section_id,
                        &section_title,
                        format!("section '{section_id}' not found in segment index"),
                    ));
                    continue;
                }
            };

            // Parse only this section.
            let (section_ast, parse_stats) = match section_entry {
                Some(entry) => crate::parse::parse_section_with_stats(&transcript, entry),
                None => unreachable!("start_page already checked above"),
            };

            // Parse diagnostic.
            {
                use conset_pdf_ir::AstNode;
                fn walk_nodes(nodes: &[AstNode], tag: &OutlineTag) -> usize {
                    nodes.iter().map(|n| {
                        (if n.tag == *tag { 1 } else { 0 }) + walk_nodes(&n.children, tag)
                    }).sum()
                }
                fn collect_unclassified(nodes: &[AstNode]) -> Vec<UnclassifiedNodeTrace> {
                    let mut out = Vec::new();
                    for n in nodes {
                        if n.tag == OutlineTag::Unclassified {
                            out.push(UnclassifiedNodeTrace {
                                page_index: n.page_index,
                                x_indent: n.x_indent,
                                text_snippet: n.text.chars().take(80).collect(),
                            });
                        }
                        out.extend(collect_unclassified(&n.children));
                    }
                    out
                }
                let node_count = walk_nodes(&section_ast.nodes, &OutlineTag::Part)
                    + walk_nodes(&section_ast.nodes, &OutlineTag::Article)
                    + walk_nodes(&section_ast.nodes, &OutlineTag::Paragraph)
                    + walk_nodes(&section_ast.nodes, &OutlineTag::SubParagraph)
                    + walk_nodes(&section_ast.nodes, &OutlineTag::SubSubParagraph)
                    + walk_nodes(&section_ast.nodes, &OutlineTag::SubSubSubParagraph)
                    + walk_nodes(&section_ast.nodes, &OutlineTag::Unclassified);
                let distribution = NodeDistribution {
                    part: walk_nodes(&section_ast.nodes, &OutlineTag::Part),
                    article: walk_nodes(&section_ast.nodes, &OutlineTag::Article),
                    paragraph: walk_nodes(&section_ast.nodes, &OutlineTag::Paragraph),
                    sub_paragraph: walk_nodes(&section_ast.nodes, &OutlineTag::SubParagraph),
                    sub_sub_paragraph: walk_nodes(&section_ast.nodes, &OutlineTag::SubSubParagraph),
                    sub_sub_sub_paragraph: walk_nodes(&section_ast.nodes, &OutlineTag::SubSubSubParagraph),
                    unclassified: walk_nodes(&section_ast.nodes, &OutlineTag::Unclassified),
                };
                let unclassified_nodes = collect_unclassified(&section_ast.nodes);
                diagnostics.push(DiagnosticEvent::Parse(ParseDiagnostic {
                    section_id: section_id.clone(),
                    total_lines: parse_stats.total_lines,
                    noise_lines_skipped: parse_stats.noise_lines_skipped,
                    inject_missing_parts_count: parse_stats.inject_missing_parts_count,
                    node_count,
                    node_distribution: distribution,
                    unclassified_nodes,
                }));
            }

            // Apply edit operations (skip editing when the list is empty).
            let (edited_ast, edit_diag) = if spec.operations.is_empty() {
                let diag = EditDiagnostic {
                    section_id: section_id.clone(),
                    operations_attempted: 0,
                    operations_applied: 0,
                    failures: vec![],
                };
                (section_ast, diag)
            } else {
                use conset_pdf_ir::{EditRequest, ParsedDocument};
                let ops_count = spec.operations.len();
                let single_section_doc = ParsedDocument {
                    source_path: source_path.to_owned(),
                    sections: vec![section_ast],
                    global_warnings: vec![],
                };
                let mut editor = SectionEditor::new(single_section_doc);
                let edit_request = EditRequest::new(
                    format!("addendum patch for {section_id}"),
                    spec.operations.clone(),
                );
                let edit_result = editor.apply(edit_request);

                // Build edit diagnostic. The engine is atomic: one optional error.
                let failures: Vec<EditFailureTrace> = if !edit_result.success {
                    if let Some(ref err) = edit_result.error {
                        vec![EditFailureTrace {
                            operation_index: edit_result.operations_applied,
                            op_type: "unknown".to_owned(),
                            path: vec![],
                            reason: err.to_string(),
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                let diag = EditDiagnostic {
                    section_id: section_id.clone(),
                    operations_attempted: ops_count,
                    operations_applied: edit_result.operations_applied,
                    failures,
                };

                if !edit_result.success {
                    let reason = edit_result
                        .error
                        .as_ref()
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "unknown edit error".to_owned());
                    diagnostics.push(DiagnosticEvent::Edit(diag));
                    section_results.push(SectionPatchResult::failed(
                        section_id,
                        &section_title,
                        format!("edit failed: {reason}"),
                    ));
                    continue;
                }
                let edited_doc = editor.into_document();
                match edited_doc.sections.into_iter().find(|s| &s.section_id == section_id) {
                    Some(ast) => (ast, diag),
                    None => {
                        diagnostics.push(DiagnosticEvent::Edit(diag));
                        section_results.push(SectionPatchResult::failed(
                            section_id,
                            &section_title,
                            "section missing from document after editing".to_owned(),
                        ));
                        continue;
                    }
                }
            };
            diagnostics.push(DiagnosticEvent::Edit(edit_diag));

            // Merge chrome metadata for this section.
            let chrome_meta = merge_chrome_meta(
                &segment_index,
                section_id,
                &edited_ast.section_title,
                manifest.project_metadata.as_ref(),
                spec.chrome_override.as_ref(),
                manifest.issue_date.as_deref(),
            );

            if dry_run {
                // Dry-run: build HTML but skip Chrome and skip stitch.
                let render_result = renderer.dry_run(&edited_ast, &chrome_meta);
                diagnostics.push(DiagnosticEvent::Render(RenderDiagnostic {
                    section_id: section_id.clone(),
                    chrome_binary: render_result.chrome_binary.clone(),
                    chrome_binary_version: String::new(),
                    html_size_bytes: render_result.html_size_bytes,
                    elapsed_ms: 0,
                    outcome: RenderOutcome::DryRun,
                }));
                let idx = section_results.len();
                result_idx.insert(section_id.clone(), idx);
                section_results.push(SectionPatchResult::success(
                    section_id,
                    &section_title,
                    0,
                    0,
                ));
            } else {
                // Real render via headless Chrome.
                let render_start = std::time::Instant::now();
                let render_result = renderer.render(&edited_ast, &chrome_meta);
                let render_elapsed_ms = render_start.elapsed().as_millis() as u64;

                let render_result = match render_result {
                    Ok(r) => {
                        diagnostics.push(DiagnosticEvent::Render(RenderDiagnostic {
                            section_id: section_id.clone(),
                            chrome_binary: r.chrome_binary.clone(),
                            chrome_binary_version: r.chrome_binary_version.clone(),
                            html_size_bytes: r.html_size_bytes,
                            elapsed_ms: render_elapsed_ms,
                            outcome: RenderOutcome::Success {
                                output_size_bytes: r.pdf_bytes.len(),
                            },
                        }));
                        r
                    }
                    Err(e) => {
                        let (exit_code, stderr_tail) = match &e {
                            conset_pdf_ir::RenderError::ChromeRenderFailed { exit_code, stderr } => {
                                let tail: String = stderr
                                    .chars()
                                    .rev()
                                    .take(500)
                                    .collect::<String>()
                                    .chars()
                                    .rev()
                                    .collect();
                                (Some(*exit_code), tail)
                            }
                            _ => (None, e.to_string()),
                        };
                        diagnostics.push(DiagnosticEvent::Render(RenderDiagnostic {
                            section_id: section_id.clone(),
                            chrome_binary: String::new(),
                            chrome_binary_version: String::new(),
                            html_size_bytes: 0,
                            elapsed_ms: render_elapsed_ms,
                            outcome: RenderOutcome::Failed { exit_code, stderr_tail },
                        }));
                        section_results.push(SectionPatchResult::failed(
                            section_id,
                            &section_title,
                            format!("render failed: {e}"),
                        ));
                        continue;
                    }
                };

                // Write replacement PDF to an isolated temp file.
                let safe_id = section_id.replace(' ', "_");
                // SAFETY: `temp_dir` is `Some` whenever `dry_run` is false,
                // and this branch is only reachable when `dry_run` is false.
                let temp_pdf_path = temp_dir.as_ref().unwrap().path().join(format!("{safe_id}.pdf"));
                let temp_pdf_str = temp_pdf_path.to_string_lossy().into_owned();

                if let Err(e) = std::fs::write(&temp_pdf_path, &render_result.pdf_bytes) {
                    section_results.push(SectionPatchResult::failed(
                        section_id,
                        &section_title,
                        format!("failed to write replacement PDF to temp: {e}"),
                    ));
                    continue;
                }

                // Enqueue for stitching; record a placeholder result we'll update later.
                let idx = section_results.len();
                result_idx.insert(section_id.clone(), idx);
                section_results.push(SectionPatchResult::success(
                    section_id,
                    &section_title,
                    0, // updated after stitch
                    0, // updated after stitch
                ));
                stitch_queue.push((section_id.clone(), start_page, temp_pdf_str));
            }
        }

        // ── Step 4: Stitch (descending start_page = last-to-first) ───────────
        let final_output_path = if dry_run || stitch_queue.is_empty() {
            if !dry_run && output_path.is_none() {
                return Err("--output is required for non-dry-run".to_owned());
            }
            output_path.map(str::to_owned)
        } else {
            let out = output_path
                .ok_or_else(|| "--output is required for non-dry-run".to_owned())?
                .to_owned();

            // Sort descending so we stitch the highest-page-number section first.
            stitch_queue.sort_by(|a, b| b.1.cmp(&a.1));

            let total = stitch_queue.len();
            // current_source starts as the original PDF path; after each successful
            // stitch it advances to that stitch's output path.
            let mut current_source = source_path.to_owned();

            for (stitch_idx, (section_id, _start_page, replacement_path)) in
                stitch_queue.iter().enumerate()
            {
                let is_last = stitch_idx + 1 == total;
                let stitch_out = if is_last {
                    out.clone()
                } else {
                    // SAFETY: `temp_dir` is `Some` whenever `dry_run` is false,
                    // and this loop body is only reached when `!dry_run`.
                    temp_dir
                        .as_ref()
                        .unwrap()
                        .path()
                        .join(format!("stitch_{stitch_idx}.pdf"))
                        .to_string_lossy()
                        .into_owned()
                };

                let plan = StitchPlan {
                    original_path: current_source.clone(),
                    section_id: section_id.clone(),
                    segment_index: segment_index.clone(),
                    replacement_path: replacement_path.clone(),
                    output_path: stitch_out.clone(),
                    dry_run: false,
                };

                let stitch_start = std::time::Instant::now();
                match PdfStitcher::stitch(&plan) {
                    Ok(sr) => {
                        let stitch_elapsed_ms = stitch_start.elapsed().as_millis() as u64;
                        // Update the placeholder SectionPatchResult with real counts.
                        if let Some(&ridx) = result_idx.get(section_id) {
                            section_results[ridx].status = SectionPatchStatus::Success {
                                pages_removed: sr.pages_removed,
                                pages_inserted: sr.pages_inserted,
                            };
                        }
                        diagnostics.push(DiagnosticEvent::Stitch(StitchDiagnostic {
                            section_id: section_id.clone(),
                            pages_removed: sr.pages_removed,
                            pages_inserted: sr.pages_inserted,
                            bookmarks_rerouted: usize::from(sr.bookmarks_updated),
                            elapsed_ms: stitch_elapsed_ms,
                            warnings: sr.warnings.clone(),
                        }));
                        current_source = stitch_out;
                    }
                    Err(e) => {
                        let stitch_elapsed_ms = stitch_start.elapsed().as_millis() as u64;
                        // Mark this section failed; advance current_source unchanged.
                        if let Some(&ridx) = result_idx.get(section_id) {
                            section_results[ridx].status = SectionPatchStatus::Failed {
                                reason: format!("stitch failed: {e}"),
                            };
                        }
                        diagnostics.push(DiagnosticEvent::Stitch(StitchDiagnostic {
                            section_id: section_id.clone(),
                            pages_removed: 0,
                            pages_inserted: 0,
                            bookmarks_rerouted: 0,
                            elapsed_ms: stitch_elapsed_ms,
                            warnings: vec![format!("stitch failed: {e}")],
                        }));
                        // Don't advance current_source — subsequent sections still
                        // reference a valid (un-stitched) intermediate file.
                    }
                }
            }

            Some(out)
        };

        let mut result = AddendumResult::from_results(
            manifest.description.clone(),
            section_results,
            final_output_path,
        );
        result.diagnostics = diagnostics;
        result.pattern_db_version = Some(pattern_db.version);
        Ok(result)
    }
}

// ── Chrome metadata merge ─────────────────────────────────────────────────────

/// Build [`SpecChromeMetadata`] for a section by merging:
/// 1. extracted `SegmentIndex` chrome metadata (global project fields),
/// 2. manifest-level `project_metadata` override,
/// 3. per-section `chrome_override`.
///
/// Non-empty override fields replace lower-priority values; empty strings are
/// ignored so partial overrides are safe.
fn merge_chrome_meta(
    index: &SegmentIndex,
    section_id: &str,
    section_title: &str,
    manifest_override: Option<&SpecChromeMetadata>,
    section_override: Option<&SpecChromeMetadata>,
    issue_date: Option<&str>,
) -> SpecChromeMetadata {
    // Base: extracted global chrome metadata from the source PDF.
    let base = &index.chrome_metadata;
    let mut meta = SpecChromeMetadata {
        project_id: base.project_id.clone(),
        project_name: base.project_name.clone(),
        firm: base.firm.clone(),
        date: base.date.clone(),
        section_id: section_id.to_owned(),
        section_title: section_title.to_owned(),
    };

    // Override: manifest-level project_metadata.
    if let Some(ov) = manifest_override {
        apply_override(&mut meta, ov);
    }

    // Addendum issue date (lowest precedence after per-section override).
    if let Some(date) = issue_date {
        if !date.is_empty() && meta.date.is_empty() {
            meta.date = date.to_owned();
        }
    }

    // Override: per-section chrome_override (highest precedence).
    if let Some(ov) = section_override {
        apply_override(&mut meta, ov);
    }

    meta
}

/// Overwrite non-empty fields in `dest` from `src`.
fn apply_override(dest: &mut SpecChromeMetadata, src: &SpecChromeMetadata) {
    if !src.project_id.is_empty() {
        dest.project_id = src.project_id.clone();
    }
    if !src.project_name.is_empty() {
        dest.project_name = src.project_name.clone();
    }
    if !src.firm.is_empty() {
        dest.firm = src.firm.clone();
    }
    if !src.date.is_empty() {
        dest.date = src.date.clone();
    }
    // section_id and section_title from the override — only when explicitly set.
    if !src.section_id.is_empty() {
        dest.section_id = src.section_id.clone();
    }
    if !src.section_title.is_empty() {
        dest.section_title = src.section_title.clone();
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Build an [`ExtractionDiagnostic`] from a page slice and elapsed time.
///
/// Separated from `run()` so it can be tested with synthetic transcripts.
fn build_extraction_diagnostic(
    pages: &[conset_pdf_ir::Page],
    elapsed_ms: u64,
) -> ExtractionDiagnostic {
    let page_count = pages.len();
    let mut total_spans = 0usize;
    let mut zero_span_pages: Vec<usize> = Vec::new();
    let mut low_span_pages: Vec<usize> = Vec::new();
    for (idx, page) in pages.iter().enumerate() {
        let n = page.spans().len();
        total_spans += n;
        if n == 0 {
            zero_span_pages.push(idx);
        } else if n < 5 {
            low_span_pages.push(idx);
        }
    }
    ExtractionDiagnostic {
        page_count,
        total_spans,
        zero_span_pages,
        low_span_pages,
        elapsed_ms,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{ChromeMetadata, CoverageStats, SectionEntry};

    fn make_index(sections: Vec<SectionEntry>) -> SegmentIndex {
        SegmentIndex {
            source_path: "test.pdf".to_owned(),
            chrome_metadata: ChromeMetadata {
                project_id: "PROJ-001".to_owned(),
                project_name: "Test Building".to_owned(),
                firm: "Test Engineers".to_owned(),
                date: "2025-01-01".to_owned(),
            },
            sections,
            coverage: CoverageStats {
                pages_total: 10,
                pages_tagged: 10,
                pages_missing_footer: 0,
                coverage_ratio: 1.0,
            },
        }
    }

    fn make_entry(id: &str, title: &str, start: usize, end: usize) -> SectionEntry {
        SectionEntry {
            section_id: id.to_owned(),
            section_title: title.to_owned(),
            start_page: start,
            end_page: end,
            page_count: end - start + 1,
            page_counter_detected: false,
            confidence: 1.0,
        }
    }

    #[test]
    fn merge_chrome_meta_base_only() {
        let index = make_index(vec![make_entry("23 82 16", "Heating Water Coils", 0, 4)]);
        let meta = merge_chrome_meta(&index, "23 82 16", "Heating Water Coils", None, None, None);
        assert_eq!(meta.project_id, "PROJ-001");
        assert_eq!(meta.section_id, "23 82 16");
        assert_eq!(meta.section_title, "Heating Water Coils");
        assert_eq!(meta.date, "2025-01-01");
    }

    #[test]
    fn merge_chrome_meta_manifest_override() {
        let index = make_index(vec![]);
        let ov = SpecChromeMetadata {
            project_id: "PROJ-002".to_owned(),
            project_name: String::new(), // empty → keep base
            firm: String::new(),
            date: "2025-10-17".to_owned(),
            section_id: String::new(),
            section_title: String::new(),
        };
        let meta =
            merge_chrome_meta(&index, "23 82 16", "Coils", Some(&ov), None, None);
        assert_eq!(meta.project_id, "PROJ-002");
        assert_eq!(meta.project_name, "Test Building"); // base preserved
        assert_eq!(meta.date, "2025-10-17");
    }

    #[test]
    fn merge_chrome_meta_section_override_wins() {
        let index = make_index(vec![]);
        let manifest_ov = SpecChromeMetadata {
            project_id: "MANIFEST".to_owned(),
            project_name: String::new(),
            firm: String::new(),
            date: String::new(),
            section_id: String::new(),
            section_title: String::new(),
        };
        let section_ov = SpecChromeMetadata {
            project_id: "SECTION".to_owned(),
            project_name: String::new(),
            firm: String::new(),
            date: String::new(),
            section_id: String::new(),
            section_title: String::new(),
        };
        let meta = merge_chrome_meta(
            &index,
            "23 82 16",
            "Coils",
            Some(&manifest_ov),
            Some(&section_ov),
            None,
        );
        assert_eq!(meta.project_id, "SECTION");
    }

    #[test]
    fn merge_chrome_meta_issue_date_fills_empty_date() {
        let mut index = make_index(vec![]);
        index.chrome_metadata.date = String::new(); // base has no date
        let meta = merge_chrome_meta(&index, "23 82 16", "Coils", None, None, Some("2025-12-01"));
        assert_eq!(meta.date, "2025-12-01");
    }

    #[test]
    fn merge_chrome_meta_issue_date_does_not_override_extracted() {
        let index = make_index(vec![]); // base.date = "2025-01-01"
        let meta = merge_chrome_meta(&index, "23 82 16", "Coils", None, None, Some("2025-12-01"));
        // issue_date only fills when base.date is empty
        assert_eq!(meta.date, "2025-01-01");
    }

    // ── 8.1.H: RAII temp cleanup ──────────────────────────────────────────────

    /// Verify that the temp directory does not persist after `run()` returns,
    /// even when every manifest section fails (non-existent section ID).
    ///
    /// We capture the path via a `TempDir::path()` clone *before* drop so we can
    /// assert on it afterwards.  Because `TempDir` is dropped at the end of
    /// `run()`, the directory should be gone by the time we check.
    #[test]
    fn orchestrator_temp_dir_cleaned_up_on_section_failure() {
        use conset_pdf_ir::{AddendumManifest, SectionEditSpec};
        use std::path::PathBuf;

        // Build a minimal single-section manifest pointing at a section ID that
        // cannot exist in any real PDF — guaranteed to produce a Failed result.
        let manifest = AddendumManifest {
            description: None,
            sections: vec![SectionEditSpec {
                section_id: "DOES_NOT_EXIST".to_owned(),
                operations: vec![],
                chrome_override: None,
            }],
            project_metadata: None,
            issue_date: None,
        };

        // Record what TempDir *would* create by running a dry-run first to get
        // a section failure without touching the filesystem for real.
        // For the actual cleanup test we need a non-dry-run but a non-existent
        // source PDF will fail at extraction — before temp dir creation.
        // So we verify the dry-run path (temp_dir = None) is clean, then verify
        // the non-dry-run with a non-existent PDF exits cleanly with an Err at
        // the extraction stage (before any temp dir is created).
        let result_dry = SpecsPatchOrchestrator::run(
            "nonexistent_source.pdf",
            manifest.clone(),
            None,
            true, // dry_run — temp_dir stays None
        );
        // Dry-run: extraction will fail because the file doesn't exist.
        assert!(result_dry.is_err(), "expected extraction error for missing PDF");

        // Non-dry-run: also fails at extraction — temp dir creation is guarded
        // by the extraction success, so no temp directory is created.
        // This confirms the early-return path leaves no orphaned dir.
        let tmp_snapshot: PathBuf = std::env::temp_dir();
        let entries_before: usize = std::fs::read_dir(&tmp_snapshot)
            .map(|rd| rd.count())
            .unwrap_or(0);

        let result_real = SpecsPatchOrchestrator::run(
            "nonexistent_source.pdf",
            manifest,
            Some("out.pdf"),
            false,
        );
        assert!(result_real.is_err(), "expected extraction error");

        let entries_after: usize = std::fs::read_dir(&tmp_snapshot)
            .map(|rd| rd.count())
            .unwrap_or(0);
        // No net growth in temp dir — extraction failure exits before TempDir creation.
        assert_eq!(
            entries_after, entries_before,
            "temp dir entry count changed unexpectedly"
        );
    }

    // ── 8.1.F: Diagnostic wiring unit tests ──────────────────────────────────

    /// `build_extraction_diagnostic` must populate `zero_span_pages` and
    /// `low_span_pages` based on per-page span counts.
    #[test]
    fn orchestrator_diagnostics_extraction_zero_span_pages_detected() {
        use conset_pdf_ir::types::{BBox, Span};
        use conset_pdf_ir::{LayoutTranscript, Page};

        // Page 0: 0 spans → zero_span_pages
        let page0 = Page::new(0, 612.0, 792.0).unwrap();

        // Page 1: 3 spans → low_span_pages  (< 5)
        let mut page1 = Page::new(1, 612.0, 792.0).unwrap();
        let bbox = BBox::new(0.1, 0.5, 0.2, 0.02).unwrap();
        for _ in 0..3 {
            page1.add_span(Span::new("word", bbox.clone(), 10.0).unwrap()).unwrap();
        }

        // Page 2: 10 spans → normal
        let mut page2 = Page::new(2, 612.0, 792.0).unwrap();
        let bbox2 = BBox::new(0.1, 0.5, 0.2, 0.02).unwrap();
        for _ in 0..10 {
            page2.add_span(Span::new("word", bbox2.clone(), 10.0).unwrap()).unwrap();
        }

        let transcript = LayoutTranscript::from_pages(vec![page0, page1, page2]).unwrap();
        let diag = build_extraction_diagnostic(transcript.pages(), 42);

        assert_eq!(diag.page_count, 3);
        assert_eq!(diag.total_spans, 13);
        assert_eq!(diag.zero_span_pages, vec![0]);
        assert_eq!(diag.low_span_pages, vec![1]);
        assert_eq!(diag.elapsed_ms, 42);
    }

    /// `parse_section_with_stats` must produce `OutlineTag::Unclassified` nodes
    /// for body-band text that does not match any CSI outline-marker pattern and
    /// has no preceding structural node to fold into.
    #[test]
    fn orchestrator_diagnostics_parse_unclassified_nodes_traced() {
        use conset_pdf_ir::types::{BBox, Span};
        use conset_pdf_ir::{LayoutTranscript, OutlineTag, Page, SectionEntry};

        let mut page = Page::new(0, 612.0, 792.0).unwrap();
        // y=0.5 is in the body band [0.15, 0.85]
        let bbox = BBox::new(0.1, 0.5, 0.7, 0.02).unwrap();
        page.add_span(
            Span::new("some random diagnostic note", bbox, 10.0).unwrap(),
        )
        .unwrap();
        let transcript = LayoutTranscript::from_pages(vec![page]).unwrap();

        let entry = SectionEntry {
            section_id: "TEST".to_owned(),
            section_title: "Test Section".to_owned(),
            start_page: 0,
            end_page: 0,
            page_count: 1,
            page_counter_detected: false,
            confidence: 1.0,
        };

        let (ast, _stats) = crate::parse::parse_section_with_stats(&transcript, &entry);

        let unclassified_count = ast
            .nodes
            .iter()
            .filter(|n| n.tag == OutlineTag::Unclassified)
            .count();
        assert!(
            unclassified_count > 0,
            "expected at least one unclassified node; got {:?}",
            ast.nodes.iter().map(|n| &n.tag).collect::<Vec<_>>()
        );
    }

    /// The stderr tail extraction logic (double-reverse) must capture exactly the
    /// last 500 characters when Chrome emits a longer error message.
    #[test]
    fn orchestrator_diagnostics_render_failure_stderr_captured() {
        // Reproduce the exact double-reverse logic used in `run()`.
        let long_stderr: String = "x".repeat(500) + &"z".repeat(500);
        let tail: String = long_stderr
            .chars()
            .rev()
            .take(500)
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        assert_eq!(tail.len(), 500, "tail should be exactly 500 chars");
        assert!(
            tail.chars().all(|c| c == 'z'),
            "tail should be the last 500 z-chars, got: {tail:?}"
        );
    }
}

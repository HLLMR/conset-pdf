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
    AddendumManifest, AddendumResult, EditRequest, ParsedDocument, SectionPatchResult,
    SectionPatchStatus, SegmentIndex, SpecChromeMetadata, StitchPlan,
};

use crate::{
    edit::SectionEditor,
    extractor::Extractor,
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
        // ── Step 1: Extract ───────────────────────────────────────────────────
        let transcript = Extractor::new()
            .extract(source_path)
            .map_err(|e| format!("extraction failed: {e}"))?;

        // ── Step 2: Segment ───────────────────────────────────────────────────
        let segment_index = crate::segment::segment_transcript(&transcript)
            .map_err(|e| format!("segmentation failed: {e}"))?;

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
        let temp_dir: Option<std::path::PathBuf> = if dry_run {
            None
        } else {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let td = std::env::temp_dir().join(format!("specs_patch_{ts}"));
            std::fs::create_dir_all(&td)
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
            let parsed_doc = match crate::parse::parse_document(
                &transcript,
                &segment_index,
                Some(section_id),
            ) {
                Ok(doc) => doc,
                Err(e) => {
                    section_results.push(SectionPatchResult::failed(
                        section_id,
                        &section_title,
                        format!("parse failed: {e}"),
                    ));
                    continue;
                }
            };

            let section_ast = match parsed_doc
                .sections
                .into_iter()
                .find(|s| &s.section_id == section_id)
            {
                Some(ast) => ast,
                None => {
                    section_results.push(SectionPatchResult::failed(
                        section_id,
                        &section_title,
                        "parser returned no AST for section".to_owned(),
                    ));
                    continue;
                }
            };

            // Apply edit operations (skip editing when the list is empty).
            let edited_ast = if spec.operations.is_empty() {
                section_ast
            } else {
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
                if !edit_result.success {
                    let reason = edit_result
                        .error
                        .as_ref()
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "unknown edit error".to_owned());
                    section_results.push(SectionPatchResult::failed(
                        section_id,
                        &section_title,
                        format!("edit failed: {reason}"),
                    ));
                    continue;
                }
                let edited_doc = editor.into_document();
                match edited_doc.sections.into_iter().find(|s| &s.section_id == section_id) {
                    Some(ast) => ast,
                    None => {
                        section_results.push(SectionPatchResult::failed(
                            section_id,
                            &section_title,
                            "section missing from document after editing".to_owned(),
                        ));
                        continue;
                    }
                }
            };

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
                renderer.dry_run(&edited_ast, &chrome_meta);
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
                let render_result = match renderer.render(&edited_ast, &chrome_meta) {
                    Ok(r) => r,
                    Err(e) => {
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
                let temp_pdf_path = temp_dir.as_ref().unwrap().join(format!("{safe_id}.pdf"));
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
                    temp_dir
                        .as_ref()
                        .unwrap()
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

                match PdfStitcher::stitch(&plan) {
                    Ok(sr) => {
                        // Update the placeholder SectionPatchResult with real counts.
                        if let Some(&ridx) = result_idx.get(section_id) {
                            section_results[ridx].status = SectionPatchStatus::Success {
                                pages_removed: sr.pages_removed,
                                pages_inserted: sr.pages_inserted,
                            };
                        }
                        current_source = stitch_out;
                    }
                    Err(e) => {
                        // Mark this section failed; advance current_source unchanged.
                        if let Some(&ridx) = result_idx.get(section_id) {
                            section_results[ridx].status = SectionPatchStatus::Failed {
                                reason: format!("stitch failed: {e}"),
                            };
                        }
                        // Don't advance current_source — subsequent sections still
                        // reference a valid (un-stitched) intermediate file.
                    }
                }
            }

            Some(out)
        };

        Ok(AddendumResult::from_results(
            manifest.description.clone(),
            section_results,
            final_output_path,
        ))
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
}

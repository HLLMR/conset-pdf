//! Transcript overlay visualization.
//!
//! Loads the source PDF identified in a [`LayoutTranscript`]'s metadata, renders
//! each page as a PNG at 1400 px wide, and draws coloured rectangles over every
//! extracted span bounding box.  Used for Phase 1 coordinate-system verification.
//!
//! The output directory is created if it does not already exist.  Files are named
//! `page-{4-digit-zero-padded-index}.png` (e.g. `page-0000.png`).

use crate::error::{EngineError, Result};
use conset_pdf_extraction::PdfiumExtractor;
use conset_pdf_ir::{BBox, LayoutTranscript, SegmentIndex};
use std::path::Path;

/// Renders each page of the source PDF with span-bbox overlays from `transcript`
/// and writes PNGs to `output_dir`.
///
/// The source PDF path is read from `transcript.metadata().source_path`.
///
/// Returns the number of pages rendered.
///
/// # Errors
///
/// Returns [`EngineError`] if PDFium cannot be initialised, the PDF cannot be
/// loaded, a page cannot be rendered, or writing a PNG fails.
pub fn render_transcript_overlays(transcript: &LayoutTranscript, output_dir: &Path) -> Result<u32> {
    let pdf_path = &transcript.metadata().source_path;

    let extractor = PdfiumExtractor::try_new()
        .map_err(|e| EngineError::other(format!("PDFium init failed: {e}")))?;

    std::fs::create_dir_all(output_dir).map_err(EngineError::Io)?;

    let mut rendered = 0u32;

    for page in transcript.pages() {
        let page_index = page.page_index();
        let bboxes: Vec<conset_pdf_ir::BBox> =
            page.spans().iter().map(|s| s.bbox.clone()).collect();

        let output_path = output_dir.join(format!("page-{page_index:04}.png"));

        extractor
            .render_page_with_spans(pdf_path, page_index, &bboxes, &output_path)
            .map_err(|e| {
                EngineError::other(format!("Failed to render page {page_index}: {e}"))
            })?;

        log::debug!("Rendered overlay: {}", output_path.display());
        rendered += 1;
    }

    Ok(rendered)
}

/// Renders each page of the source PDF with color-coded span overlays derived
/// from a [`SegmentIndex`] and its corresponding [`LayoutTranscript`].
///
/// Color coding:
/// - **Blue** `[30, 100, 220]` — header band (Y < 0.15)
/// - **Red**  `[220, 50, 30]`  — footer band  (Y > 0.85)
/// - **Green** `[0, 200, 0]`   — body band
///
/// Pages that start or end a section receive a section-boundary marker in the
/// filename: `page-{N:04}-section-{id}.png`.  All other pages are named
/// `page-{N:04}.png`.
///
/// Returns the number of pages rendered.
///
/// # Errors
///
/// Returns [`EngineError`] if PDFium cannot be initialised, the PDF cannot be
/// loaded, a page cannot be rendered, or writing a PNG fails.
pub fn render_segment_overlays(
    transcript: &LayoutTranscript,
    segment_index: &SegmentIndex,
    output_dir: &Path,
) -> Result<u32> {
    let pdf_path = &transcript.metadata().source_path;

    let extractor = PdfiumExtractor::try_new()
        .map_err(|e| EngineError::other(format!("PDFium init failed: {e}")))?;

    std::fs::create_dir_all(output_dir).map_err(EngineError::Io)?;

    // Build a map: page_index → section_id for boundary pages.
    let mut boundary_labels: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();
    for section in &segment_index.sections {
        boundary_labels
            .entry(section.start_page)
            .or_insert_with(|| section.section_id.clone());
    }

    let mut rendered = 0u32;

    for page in transcript.pages() {
        let page_index = page.page_index();

        // Build color-coded span list.
        let colored: Vec<(BBox, [u8; 3])> = page
            .spans()
            .iter()
            .map(|s| {
                let color = if s.bbox.y < 0.15 {
                    [30u8, 100u8, 220u8] // blue — header
                } else if s.bbox.y > 0.85 {
                    [220u8, 50u8, 30u8] // red — footer
                } else {
                    [0u8, 200u8, 0u8] // green — body
                };
                (s.bbox.clone(), color)
            })
            .collect();

        let file_name = if let Some(sid) = boundary_labels.get(&page_index) {
            // Sanitize section ID for use in a filename.
            let safe_id = sid.replace(' ', "_");
            format!("page-{page_index:04}-section-{safe_id}.png")
        } else {
            format!("page-{page_index:04}.png")
        };

        let output_path = output_dir.join(&file_name);

        extractor
            .render_page_with_colored_spans(pdf_path, page_index, &colored, &output_path)
            .map_err(|e| {
                EngineError::other(format!("Failed to render page {page_index}: {e}"))
            })?;

        log::debug!("Rendered segment overlay: {}", output_path.display());
        rendered += 1;
    }

    Ok(rendered)
}

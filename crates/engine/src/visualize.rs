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
use conset_pdf_ir::LayoutTranscript;
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

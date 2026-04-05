//! Extraction stage: load a PDF file and convert it to a raw `LayoutTranscript`.
//!
//! This is the Lexer stage of the compiler model.  It uses `PdfiumExtractor` from
//! `conset-pdf-extraction` to load each page, then converts each `SpanData` /
//! `RawBBox` (PDF bottom-left coordinates, points) into an `ir::Span` with a
//! normalised top-left `BBox` via `conset_pdf_ir::normalize_bbox`.
//!
//! Spans whose bounding boxes fail normalisation or whose text is invalid are
//! silently skipped with a debug log entry; the rest of the page is preserved
//! (Soft-fail policy — Non-Negotiable #13).

use crate::error::{EngineError, Result};
use conset_pdf_extraction::{PdfExtractor, PdfiumExtractor};
use conset_pdf_ir::{normalize_bbox, BoundingBox, LayoutTranscript, Page, Span, TranscriptMetadata};

/// Runs the extraction stage for the given PDF path.
///
/// # Errors
///
/// Returns [`EngineError`] if PDFium cannot be initialised, the PDF cannot be
/// loaded, or a per-page extraction call fails.
pub fn run(path: &str) -> Result<LayoutTranscript> {
    let extractor = PdfiumExtractor::try_new()
        .map_err(|e| EngineError::extraction(format!("PDFium init failed: {e}")))?;

    let doc = extractor
        .load_document(path)
        .map_err(|e| EngineError::extraction(format!("Failed to load PDF '{path}': {e}")))?;

    let page_count = extractor.get_page_count(&doc);
    log::debug!("Loaded '{path}': {page_count} page(s)");

    // Canonicalize to an absolute path so transcripts remain portable when moved
    // to a different working directory before the visualize step.
    let canonical_path = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_owned());

    let metadata = TranscriptMetadata::new(&canonical_path, page_count)
        .map_err(|e| EngineError::extraction(format!("Failed to create metadata: {e}")))?;

    let mut pages = Vec::with_capacity(page_count);

    for page_index in 0..page_count {
        let page_data = extractor
            .extract_page(&doc, page_index)
            .map_err(|e| {
                EngineError::extraction(format!("Failed to extract page {page_index}: {e}"))
            })?;

        let mut page = Page::new(
            page_index,
            f64::from(page_data.width_pts),
            f64::from(page_data.height_pts),
        )
        .map_err(|e| {
            EngineError::extraction(format!("Invalid page {page_index} dimensions: {e:?}"))
        })?;

        for span_data in &page_data.spans {
            let raw_bbox = BoundingBox {
                x: f64::from(span_data.bbox.x),
                y: f64::from(span_data.bbox.y),
                width: f64::from(span_data.bbox.width),
                height: f64::from(span_data.bbox.height),
            };

            let normalized = match normalize_bbox(&raw_bbox, &page) {
                Ok(b) => b,
                Err(e) => {
                    log::debug!(
                        "Page {page_index}: skipping span {:?} — bbox normalization failed: {e:?}",
                        span_data.text
                    );
                    continue;
                }
            };

            match Span::new(&span_data.text, normalized, f64::from(span_data.font_size)) {
                Ok(mut span) => {
                    span.font_name = span_data.font_name.clone();
                    page.add_span(span).map_err(|e| {
                        EngineError::extraction(format!(
                            "Failed to add span to page {page_index}: {e:?}"
                        ))
                    })?;
                }
                Err(e) => {
                    log::debug!(
                        "Page {page_index}: skipping span {:?} — invalid span: {e:?}",
                        span_data.text
                    );
                }
            }
        }

        log::debug!("Page {page_index}: {} span(s) extracted", page.spans().len());
        pages.push(page);
    }

    LayoutTranscript::new(pages, metadata)
        .map_err(|e| EngineError::extraction(format!("Failed to construct transcript: {e}")))
}

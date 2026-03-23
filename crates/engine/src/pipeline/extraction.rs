//! Extraction stage: load a PDF file and convert it to a raw `LayoutTranscript`.
//!
//! This is the Lexer stage of the compiler model.  The real pdfium-render
//! integration will be wired in here during Phase 1; until then the stage
//! returns a minimal single-page placeholder transcript so that the pipeline
//! can be exercised end-to-end.

use crate::error::{EngineError, Result};
use conset_pdf_ir::types::Page;
use conset_pdf_ir::{LayoutTranscript, TranscriptMetadata};

/// Runs the extraction stage for the given PDF path.
///
/// # Errors
///
/// Returns [`EngineError`] if the IR types reject the construction arguments.
pub fn run(path: &str) -> Result<LayoutTranscript> {
    // Phase 1: replace with real pdfium-render extraction via conset-pdf-extraction.
    let page = Page::new(0, 612.0, 792.0)
        .map_err(|e| EngineError::extraction(format!("Failed to create page: {e:?}")))?;

    let metadata = TranscriptMetadata::new(path, 1)
        .map_err(|e| EngineError::extraction(format!("Failed to create metadata: {e:?}")))?;

    LayoutTranscript::new(vec![page], metadata)
        .map_err(|e| EngineError::extraction(format!("Failed to construct transcript: {e:?}")))
}

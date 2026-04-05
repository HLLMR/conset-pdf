//! Furniture-detection stage: identify non-content structural elements.
//!
//! "Furniture" refers to page-chrome that is present in every layout but carries
//! no semantic document content: running headers/footers, stamp/revision blocks,
//! project title blocks, and form-label separators.
//!
//! # Phase 2 implementation note
//!
//! Furniture detection for section segmentation is handled by the standalone
//! [`crate::segment`] module which operates on a completed transcript, rather than
//! by in-place span annotation here.  In-place annotation (adding a `furniture_role`
//! to each `Span`) is deferred to Phase 3 when the parser needs to skip chrome.
//!
//! This stage remains a pass-through so the pipeline shape is preserved.

use crate::error::Result;
use conset_pdf_ir::LayoutTranscript;

/// Runs the furniture-detection stage.
///
/// # Errors
///
/// Returns [`crate::error::EngineError`] if annotation fails (future implementation).
pub fn run(transcript: LayoutTranscript) -> Result<LayoutTranscript> {
    // Phase 1: implement medium-specific chrome detectors.
    Ok(transcript)
}

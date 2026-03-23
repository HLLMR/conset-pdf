//! Furniture-detection stage: identify non-content structural elements.
//!
//! "Furniture" refers to page-chrome that is present in every layout but carries
//! no semantic document content: running headers/footers, stamp/revision blocks,
//! project title blocks, and form-label separators.
//!
//! Detecting furniture before the parsing stage allows the parser to ignore it and
//! focus on real content spans.  Detected furniture is annotated on the transcript
//! in-place rather than removed, so the audit trail remains complete.
//!
//! # Status
//!
//! Stub — returns the transcript unchanged.  Phase 1 will implement medium-specific
//! detectors (spec, drawing, submittal) as described in the V4.2 architecture.

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

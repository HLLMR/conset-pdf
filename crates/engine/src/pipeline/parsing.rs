//! Parsing stage: validate and normalise the raw layout transcript.
//!
//! This is the Parser stage of the compiler model.  It accepts the furniture-
//! annotated transcript produced by the extraction stage and applies:
//!
//! - Structural validation (bounding-box integrity, span ordering)
//! - Coordinate normalisation (absolute → relative, origin flip)
//! - Span deduplication and gap-fill heuristics
//!
//! The output is a semantically coherent `LayoutTranscript` ready for the
//! optimization stage.

use crate::error::{EngineError, Result};
use conset_pdf_ir::{validate_transcript, LayoutTranscript};

/// Runs the parsing stage.
///
/// Validates all IR invariants (empty-text spans, bbox bounds, sort order,
/// page contiguity) and returns the transcript unchanged when it passes, or
/// a [`EngineError::ValidationError`] with a descriptive message on failure.
///
/// # Errors
///
/// Returns [`crate::error::EngineError`] if validation rejects the transcript.
pub fn run(transcript: LayoutTranscript) -> Result<LayoutTranscript> {
    validate_transcript(&transcript)
        .map_err(|e| EngineError::validation(e.to_string()))?;
    Ok(transcript)
}

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
//!
//! # Status
//!
//! Stub — returns the transcript unchanged.  Phase 1 will implement validation
//! and normalisation using the rules in `crates/ir/src/validation.rs`.

use crate::error::Result;
use conset_pdf_ir::LayoutTranscript;

/// Runs the parsing stage.
///
/// # Errors
///
/// Returns [`crate::error::EngineError`] if validation rejects the transcript.
pub fn run(transcript: LayoutTranscript) -> Result<LayoutTranscript> {
    // Phase 1: invoke conset_pdf_ir::validation::validate_transcript and normalise.
    Ok(transcript)
}

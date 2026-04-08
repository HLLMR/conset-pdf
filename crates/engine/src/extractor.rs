//! PDF content extraction module.
//!
//! This module handles extracting structured content from PDF documents
//! and converting it into the IR format.

use crate::error::Result;
use conset_pdf_ir::LayoutTranscript;

/// Extracts content from PDF documents.
pub struct Extractor;

impl Extractor {
    /// Creates a new extractor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extracts content from a PDF file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file to extract from
    ///
    /// # Returns
    ///
    /// A `LayoutTranscript` containing the extracted content
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing the PDF fails.
    pub fn extract(&self, path: &str) -> Result<LayoutTranscript> {
        let transcript = crate::pipeline::extraction::run(path)?;
        crate::pipeline::furniture_detection::run(transcript)
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::error::EngineError;
    use crate::pipeline::extraction::MAX_PDF_PAGES;

    /// Verify the page cap constant has the documented default value.
    /// Changing this value is a breaking change to the documented intake policy
    /// and requires updating the constant doc comment.
    #[test]
    fn max_pdf_pages_has_expected_default() {
        assert_eq!(MAX_PDF_PAGES, 2_000);
    }

    /// Verify `PdfTooLarge` renders an actionable error message including
    /// both the actual count and the cap, and a suggested action.
    #[test]
    fn pdf_too_large_error_message_is_actionable() {
        let err = EngineError::PdfTooLarge { page_count: 2_500, max: 2_000 };
        let msg = err.to_string();
        assert!(msg.contains("2500"), "message must include actual page count: {msg}");
        assert!(msg.contains("2000"), "message must include cap: {msg}");
        // Must include actionable guidance (split or increase limit).
        assert!(
            msg.contains("split") || msg.contains("increase"),
            "message must suggest an action: {msg}"
        );
    }
}

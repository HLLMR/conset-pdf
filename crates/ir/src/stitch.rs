//! IR types for the Phase 6 PDF stitching pipeline.
//!
//! [`StitchPlan`] is the input to [`crate::PdfStitcher`] (defined in the engine crate).
//! It carries the resolved [`crate::SegmentIndex`] plus all path and option fields
//! needed to perform a section page replacement.
//!
//! [`StitchResult`] is the output: page counts, bookmark update count, and any
//! warnings produced during validation of the unchanged pages.
//!
//! [`StitchError`] enumerates all failure modes.

use crate::SegmentIndex;
use serde::{Deserialize, Serialize};

/// All inputs required to stitch a regenerated section back into an original PDF.
///
/// Constructed by the CLI handler from command-line arguments after loading the
/// [`SegmentIndex`] from disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchPlan {
    /// Absolute or relative path to the original (source) PDF.
    pub original_path: String,
    /// CSI section identifier to replace (e.g. `"23 82 16"`).
    pub section_id: String,
    /// Parsed segment index produced by the `segment` subcommand.
    pub segment_index: SegmentIndex,
    /// Absolute or relative path to the regenerated replacement PDF.
    pub replacement_path: String,
    /// Absolute or relative path for the stitched output PDF.
    pub output_path: String,
    /// When `true`, validate and compute [`StitchResult`] but do not write the
    /// output file.
    pub dry_run: bool,
}

/// Summary produced by a successful stitch operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StitchResult {
    /// CSI section identifier that was replaced.
    pub section_id: String,
    /// Number of pages removed from the original.
    pub pages_removed: usize,
    /// Number of pages inserted from the replacement PDF.
    pub pages_inserted: usize,
    /// Total page count of the original PDF before stitching.
    pub total_pages_before: usize,
    /// Total page count of the output after stitching.
    pub total_pages_after: usize,
    /// Whether any bookmark destinations were rerouted due to the page
    /// replacement (bookmarks pointing to deleted pages are updated to target
    /// the first page of the replacement).
    pub bookmarks_updated: bool,
    /// Non-fatal warnings produced during unchanged-page validation.
    pub warnings: Vec<String>,
}

/// Failure modes for the PDF stitching pipeline.
#[derive(Debug, thiserror::Error)]
pub enum StitchError {
    /// The requested section ID was not present in the segment index.
    #[error("section not found in segment index: '{0}'")]
    SectionNotFound(String),
    /// The original PDF could not be loaded.
    #[error("failed to load original PDF: {0}")]
    OriginalNotFound(String),
    /// The replacement PDF could not be loaded.
    #[error("failed to load replacement PDF: {0}")]
    ReplacementNotFound(String),
    /// The output PDF could not be written.
    #[error("failed to write output PDF: {0}")]
    WriteFailed(String),
    /// The section's page range exceeds the document's page count.
    #[error("page range out of bounds: {0}")]
    PageRangeOutOfBounds(String),
    /// An unexpected PDF structure was encountered.
    #[error("PDF structure error: {0}")]
    PdfStructure(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChromeMetadata, CoverageStats, SectionEntry, SegmentIndex};

    fn make_plan() -> StitchPlan {
        StitchPlan {
            original_path: "original.pdf".to_owned(),
            section_id: "23 82 16".to_owned(),
            segment_index: SegmentIndex {
                source_path: "original.pdf".to_owned(),
                chrome_metadata: ChromeMetadata::default(),
                sections: vec![SectionEntry {
                    section_id: "23 82 16".to_owned(),
                    section_title: "Heating Water Coils".to_owned(),
                    start_page: 0,
                    end_page: 2,
                    page_count: 3,
                    page_counter_detected: true,
                    confidence: 0.98,
                }],
                coverage: CoverageStats {
                    pages_total: 10,
                    pages_tagged: 10,
                    pages_missing_footer: 0,
                    coverage_ratio: 1.0,
                },
            },
            replacement_path: "replacement.pdf".to_owned(),
            output_path: "output.pdf".to_owned(),
            dry_run: false,
        }
    }

    #[test]
    fn stitch_plan_round_trips_via_serde() {
        let plan = make_plan();
        let json = serde_json::to_string(&plan).expect("serialize StitchPlan");
        let back: StitchPlan = serde_json::from_str(&json).expect("deserialize StitchPlan");
        assert_eq!(back.section_id, "23 82 16");
        assert_eq!(back.original_path, "original.pdf");
        assert!(!back.dry_run);
    }

    #[test]
    fn stitch_result_round_trips_via_serde() {
        let result = StitchResult {
            section_id: "23 82 16".to_owned(),
            pages_removed: 3,
            pages_inserted: 4,
            total_pages_before: 100,
            total_pages_after: 101,
            bookmarks_updated: true,
            warnings: vec!["test warning".to_owned()],
        };
        let json = serde_json::to_string(&result).expect("serialize StitchResult");
        let back: StitchResult = serde_json::from_str(&json).expect("deserialize StitchResult");
        assert_eq!(back, result);
    }

    #[test]
    fn stitch_error_display_contains_section_id() {
        let err = StitchError::SectionNotFound("23 82 16".to_owned());
        assert!(err.to_string().contains("23 82 16"));
    }

    #[test]
    fn stitch_error_display_original_not_found() {
        let err = StitchError::OriginalNotFound("bad.pdf: file not found".to_owned());
        assert!(err.to_string().contains("bad.pdf"));
    }

    #[test]
    fn stitch_error_display_write_failed() {
        let err = StitchError::WriteFailed("permission denied".to_owned());
        assert!(err.to_string().contains("permission denied"));
    }
}

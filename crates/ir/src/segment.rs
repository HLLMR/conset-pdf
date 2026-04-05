//! Segment index types for Phase 2 furniture detection and section segmentation.
//!
//! A [`SegmentIndex`] is the output of the section-segmentation stage: the set of
//! detected CSI sections, the chrome metadata extracted from headers/footers, and
//! a coverage report measuring how completely pages were tagged.
//!
//! These types are serialised to the `index.json` written by the `segment` CLI
//! subcommand and consumed by `visualize-segments`.

use serde::{Deserialize, Serialize};

/// Chrome metadata extracted from running headers and footers.
///
/// All fields are optional: an empty string means the field was not detected.
/// The extraction heuristics are best-effort; missing fields should never block
/// the segmentation pipeline.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChromeMetadata {
    /// Project identifier string (e.g. `"RWB Project No. 25063.00"`).
    pub project_id: String,
    /// Project name / building name (e.g. `"Lake Highlands High School"`).
    pub project_name: String,
    /// Firm / organization name (e.g. `"RWB Consulting Engineers"`).
    pub firm: String,
    /// Document date string as extracted (e.g. `"2025.10.01"`).
    pub date: String,
}

/// One CSI section detected by the footer-section-ID oracle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionEntry {
    /// Canonical CSI section ID (space-separated groups, e.g. `"23 82 16"`).
    pub section_id: String,
    /// Section title, if detected on the section's first page.
    /// Empty string when not available.
    pub section_title: String,
    /// Zero-based index of the first page belonging to this section.
    pub start_page: usize,
    /// Zero-based index of the last page belonging to this section (inclusive).
    pub end_page: usize,
    /// Number of pages in this section (`end_page - start_page + 1`).
    pub page_count: usize,
    /// Whether a `Page N of M` counter was detected anywhere in this section.
    pub page_counter_detected: bool,
    /// Detection confidence: 1.0 = footer ID matched on every page; lower values
    /// reflect pages where the ID was inconsistently detected.
    pub confidence: f64,
}

/// Coverage statistics for the entire document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageStats {
    /// Total number of pages in the document.
    pub pages_total: usize,
    /// Pages whose footer contained a detectable section ID.
    pub pages_tagged: usize,
    /// Pages with no detectable section ID in the footer band.
    pub pages_missing_footer: usize,
    /// Fraction of pages tagged: `pages_tagged / pages_total` (0.0–1.0).
    pub coverage_ratio: f64,
}

/// The complete output of the section-segmentation stage.
///
/// Serialised to `index.json` by the `segment` subcommand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentIndex {
    /// Absolute path of the source PDF (copied from `TranscriptMetadata`).
    pub source_path: String,
    /// Chrome metadata extracted from headers and footers.
    pub chrome_metadata: ChromeMetadata,
    /// Ordered list of detected CSI sections.
    pub sections: Vec<SectionEntry>,
    /// Coverage statistics for the whole document.
    pub coverage: CoverageStats,
}

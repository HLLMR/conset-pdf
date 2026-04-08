//! Structured diagnostic events emitted at each pipeline stage.
//!
//! Each stage of the `apply-addendum` pipeline emits a [`DiagnosticEvent`]
//! that is accumulated in [`AddendumResult::diagnostics`] and written as
//! newline-delimited JSON to the audit bundle.  The variants are designed so
//! that any failure reproducible in the wild can be triaged by reading the
//! diagnostics file without access to the original PDF.
//!
//! ## Adding new variants
//!
//! Both [`DiagnosticEvent`] and [`RenderOutcome`] are `#[non_exhaustive]`.
//! This forces any `match` on these enums (in downstream crates and tests) to
//! include a `_` catch-all arm, ensuring that adding a new variant in a future
//! sprint is a non-breaking change.

use serde::{Deserialize, Serialize};

// ── DiagnosticEvent ───────────────────────────────────────────────────────────

/// A single structured event emitted by one pipeline stage.
///
/// Accumulated in [`crate::AddendumResult::diagnostics`] and serialised to
/// `diagnostics.jsonl` in the audit bundle.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum DiagnosticEvent {
    /// Emitted once after the extraction stage completes.
    Extraction(ExtractionDiagnostic),
    /// Emitted once after the segmentation stage completes.
    Segmentation(SegmentationDiagnostic),
    /// Emitted once per section after the parse stage.
    Parse(ParseDiagnostic),
    /// Emitted once per section after the edit stage.
    Edit(EditDiagnostic),
    /// Emitted once per section after the render stage.
    Render(RenderDiagnostic),
    /// Emitted once per section after the stitch stage.
    Stitch(StitchDiagnostic),
}

// ── ExtractionDiagnostic ─────────────────────────────────────────────────────

/// Diagnostic data from the PDF extraction stage.
///
/// `zero_span_pages` and `low_span_pages` are the fastest signal for raster
/// content infiltrating a vector pipeline — pages where PDFium found nothing or
/// almost nothing indicate blank pages, full-page images, or Form XObjects that
/// the FPDFText path cannot descend into.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionDiagnostic {
    /// Total number of pages in the source PDF.
    pub page_count: usize,
    /// Total text spans across all pages.
    pub total_spans: usize,
    /// 0-indexed pages where PDFium returned 0 spans — raster/blank suspect.
    pub zero_span_pages: Vec<usize>,
    /// 0-indexed pages with fewer than 5 spans — Form XObject or near-blank suspect.
    pub low_span_pages: Vec<usize>,
    /// Wall-clock time for the extraction stage in milliseconds.
    pub elapsed_ms: u64,
}

// ── SegmentationDiagnostic ───────────────────────────────────────────────────

/// Diagnostic data from the PDF segmentation stage.
///
/// `pages_missing_footer` is the list of 0-indexed page numbers with no footer
/// match — cross-reference against the visual overlay PNGs to identify where
/// the segmenter went blind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentationDiagnostic {
    /// Number of sections detected.
    pub section_count: usize,
    /// Fraction of pages covered by at least one detected section (0.0–1.0).
    pub coverage_ratio: f64,
    /// 0-indexed page numbers with no footer match.
    pub pages_missing_footer: Vec<usize>,
    /// Per-section trace records.
    pub sections: Vec<SegmentTrace>,
}

/// Per-section trace record within a [`SegmentationDiagnostic`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentTrace {
    /// Canonical CSI section ID.
    pub section_id: String,
    /// Section title as extracted from the document footer.
    pub section_title: String,
    /// 0-indexed first page of the section.
    pub start_page: usize,
    /// 0-indexed last page of the section (inclusive).
    pub end_page: usize,
    /// Number of pages in this section run with a confirmed footer match.
    pub footer_match_count: usize,
    /// Whether a page counter was detected within the section.
    pub page_counter_detected: bool,
}

// ── ParseDiagnostic ──────────────────────────────────────────────────────────

/// Diagnostic data from the parse stage for a single section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    /// Canonical CSI section ID.
    pub section_id: String,
    /// Total lines fed into the parser (after chrome stripping).
    pub total_lines: usize,
    /// Lines discarded by the noise-only filter.
    pub noise_lines_skipped: usize,
    /// Synthetic PART nodes injected by the structural-recovery pass.
    pub inject_missing_parts_count: usize,
    /// Total AST nodes produced.
    pub node_count: usize,
    /// Counts by node type.
    pub node_distribution: NodeDistribution,
    /// Traces for nodes the classifier could not categorise.
    /// Only populated when `node_distribution.unclassified > 0`.
    pub unclassified_nodes: Vec<UnclassifiedNodeTrace>,
}

/// Node counts by outline tag within a single section parse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDistribution {
    /// PART-level nodes.
    pub part: usize,
    /// ARTICLE-level nodes.
    pub article: usize,
    /// Paragraph-level nodes.
    pub paragraph: usize,
    /// Sub-paragraph nodes.
    pub sub_paragraph: usize,
    /// Sub-sub-paragraph nodes.
    pub sub_sub_paragraph: usize,
    /// Sub-sub-sub-paragraph nodes.
    pub sub_sub_sub_paragraph: usize,
    /// Nodes where the classifier produced no match.
    pub unclassified: usize,
}

/// Minimal trace for a single unclassified AST node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnclassifiedNodeTrace {
    /// 0-indexed page where the node's first span appears.
    pub page_index: usize,
    /// Horizontal indent of the node's first span (normalised 0.0–1.0).
    pub x_indent: f64,
    /// First 80 characters of the node's text — enough to identify the line
    /// without a full content dump.
    pub text_snippet: String,
}

// ── EditDiagnostic ───────────────────────────────────────────────────────────

/// Diagnostic data from the edit stage for a single section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditDiagnostic {
    /// Canonical CSI section ID.
    pub section_id: String,
    /// Total operations requested in the manifest for this section.
    pub operations_attempted: usize,
    /// Operations that completed without error.
    pub operations_applied: usize,
    /// Operations that failed, with per-failure detail.
    pub failures: Vec<EditFailureTrace>,
}

/// Detail for a single failed edit operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditFailureTrace {
    /// 0-based index of the operation in the manifest's section operations list.
    pub operation_index: usize,
    /// Operation type string: `"insert_after"`, `"delete"`, or `"replace"`.
    pub op_type: String,
    /// The NodePath markers that were attempted.
    pub path: Vec<String>,
    /// Human-readable error description from [`EditError`].
    pub reason: String,
}

// ── RenderDiagnostic ─────────────────────────────────────────────────────────

/// Diagnostic data from the render stage for a single section.
///
/// `stderr_tail` (last 500 chars of Chrome's stderr) is the most actionable
/// data point when regeneration fails — Chrome's error messages are specific
/// and directed, but they are currently discarded entirely.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderDiagnostic {
    /// Canonical CSI section ID.
    pub section_id: String,
    /// Absolute path to the Chrome binary that was selected; `"dry-run"` when
    /// `--dry-run` is active.
    pub chrome_binary: String,
    /// Version string reported by Chrome (`{path} --version`), e.g.
    /// `"Google Chrome 120.0.6099.109"`.  Defaults to `"unknown"` if the
    /// probe fails or the build was a dry run.
    #[serde(default)]
    pub chrome_binary_version: String,
    /// Size of the HTML document fed to Chrome in bytes.
    pub html_size_bytes: usize,
    /// Wall-clock time for the render stage in milliseconds.
    pub elapsed_ms: u64,
    /// How the render concluded.
    pub outcome: RenderOutcome,
}

/// How a Chrome render concluded.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderOutcome {
    /// Chrome exited 0 and produced output.
    Success {
        /// Size of the rendered PDF in bytes.
        output_size_bytes: usize,
    },
    /// `--dry-run` was active; no Chrome invocation occurred.
    DryRun,
    /// Chrome exited non-zero or could not be launched.
    Failed {
        /// Chrome's exit code, or `None` if the process could not be started.
        exit_code: Option<i32>,
        /// Last 500 characters of Chrome's stderr output.
        stderr_tail: String,
    },
}

// ── StitchDiagnostic ─────────────────────────────────────────────────────────

/// Diagnostic data from the stitch stage for a single section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StitchDiagnostic {
    /// Canonical CSI section ID.
    pub section_id: String,
    /// Number of original pages removed from the source PDF.
    pub pages_removed: usize,
    /// Number of replacement pages inserted.
    pub pages_inserted: usize,
    /// Number of bookmark (outline) entries rerouted to the new page range.
    pub bookmarks_rerouted: usize,
    /// Wall-clock time for the stitch stage in milliseconds.
    pub elapsed_ms: u64,
    /// Non-fatal warnings raised during stitching (e.g. unchanged page objects).
    pub warnings: Vec<String>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip `DiagnosticEvent::Extraction` through JSON.
    #[test]
    fn diagnostic_event_extraction_round_trips() {
        let event = DiagnosticEvent::Extraction(ExtractionDiagnostic {
            page_count: 42,
            total_spans: 1_800,
            zero_span_pages: vec![3, 7],
            low_span_pages: vec![1, 2, 5],
            elapsed_ms: 123,
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    /// Round-trip `DiagnosticEvent::Segmentation` through JSON.
    #[test]
    fn diagnostic_event_segmentation_round_trips() {
        let event = DiagnosticEvent::Segmentation(SegmentationDiagnostic {
            section_count: 3,
            coverage_ratio: 0.92,
            pages_missing_footer: vec![0, 11],
            sections: vec![SegmentTrace {
                section_id: "23 82 16".to_owned(),
                section_title: "Fan Coil Units".to_owned(),
                start_page: 1,
                end_page: 5,
                footer_match_count: 5,
                page_counter_detected: true,
            }],
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    /// Round-trip `DiagnosticEvent::Parse` through JSON.
    #[test]
    fn diagnostic_event_parse_round_trips() {
        let event = DiagnosticEvent::Parse(ParseDiagnostic {
            section_id: "23 82 16".to_owned(),
            total_lines: 200,
            noise_lines_skipped: 12,
            inject_missing_parts_count: 0,
            node_count: 55,
            node_distribution: NodeDistribution {
                part: 3,
                article: 8,
                paragraph: 30,
                sub_paragraph: 10,
                sub_sub_paragraph: 3,
                sub_sub_sub_paragraph: 0,
                unclassified: 1,
            },
            unclassified_nodes: vec![UnclassifiedNodeTrace {
                page_index: 2,
                x_indent: 0.12,
                text_snippet: "NOTE: See detail on drawing A-101.".to_owned(),
            }],
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    /// Round-trip `DiagnosticEvent::Edit` through JSON.
    #[test]
    fn diagnostic_event_edit_round_trips() {
        let event = DiagnosticEvent::Edit(EditDiagnostic {
            section_id: "23 82 16".to_owned(),
            operations_attempted: 2,
            operations_applied: 1,
            failures: vec![EditFailureTrace {
                operation_index: 1,
                op_type: "replace".to_owned(),
                path: vec!["PART 2".to_owned(), "2.7".to_owned(), "A.".to_owned()],
                reason: "target node not found".to_owned(),
            }],
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    /// Round-trip `DiagnosticEvent::Render` with `RenderOutcome::Success`.
    #[test]
    fn diagnostic_event_render_success_round_trips() {
        let event = DiagnosticEvent::Render(RenderDiagnostic {
            section_id: "23 82 16".to_owned(),
            chrome_binary: "/usr/bin/chromium".to_owned(),
            chrome_binary_version: "Chromium 120.0.6099.109".to_owned(),
            html_size_bytes: 48_000,
            elapsed_ms: 2_100,
            outcome: RenderOutcome::Success { output_size_bytes: 95_000 },
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    /// Round-trip `DiagnosticEvent::Render` with `RenderOutcome::Failed`.
    #[test]
    fn diagnostic_event_render_failed_round_trips() {
        let event = DiagnosticEvent::Render(RenderDiagnostic {
            section_id: "23 82 16".to_owned(),
            chrome_binary: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_owned(),
            chrome_binary_version: "unknown".to_owned(),
            html_size_bytes: 52_000,
            elapsed_ms: 300,
            outcome: RenderOutcome::Failed {
                exit_code: Some(1),
                stderr_tail: "[0101/120000:ERROR:headless_shell.cc(100)] PDF export failed."
                    .to_owned(),
            },
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    /// Round-trip `DiagnosticEvent::Stitch` through JSON.
    #[test]
    fn diagnostic_event_stitch_round_trips() {
        let event = DiagnosticEvent::Stitch(StitchDiagnostic {
            section_id: "23 82 16".to_owned(),
            pages_removed: 5,
            pages_inserted: 6,
            bookmarks_rerouted: 3,
            elapsed_ms: 45,
            warnings: vec!["page 3 object unchanged after splice".to_owned()],
        });
        let json = serde_json::to_string(&event).expect("serialize");
        let back: DiagnosticEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }
}

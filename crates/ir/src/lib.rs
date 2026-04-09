//! Intermediate Representation (IR) for PDF processing in Conset PDF.
//!
//! This crate defines the core data structures and types that represent the result
//! of PDF extraction and analysis. The primary artifact is the `LayoutTranscript`,
//! which contains the structural and semantic information extracted from PDF documents.
//!
//! # Structure
//!
//! The IR is organized into three main modules:
//!
//! - [`layout`]: Core layout and document structure types
//! - [`types`]: Common type definitions used throughout the IR
//! - [`validation`]: Validation rules and constraints for IR structures
//!
//! # Example
//!
//! ```ignore
//! use conset_pdf_ir::LayoutTranscript;
//!
//! let transcript = LayoutTranscript::new();
//! ```

pub mod addendum;
pub mod ast;
pub mod diagnostics;
pub mod drawing;
pub mod edit;
pub mod layout;
pub mod render;
pub mod segment;
pub mod session;
pub mod stitch;
pub mod submittal;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use addendum::{
    AddendumManifest, AddendumResult, SectionEditSpec, SectionPatchResult, SectionPatchStatus,
};
pub use diagnostics::{
    DiagnosticEvent, EditDiagnostic, EditFailureTrace, ExtractionDiagnostic, NodeDistribution,
    ParseDiagnostic, RenderDiagnostic, RenderOutcome, SegmentTrace, SegmentationDiagnostic,
    StitchDiagnostic, UnclassifiedNodeTrace,
};
pub use ast::{AstNode, OutlineTag, ParsedDocument, SectionAst, SectionLayout};
pub use edit::{EditError, EditOperation, EditRequest, EditResult, NodePath};
pub use layout::{LayoutTranscript, MetadataError, TranscriptError, TranscriptMetadata};
pub use render::{PageSize, RenderConfig, RenderError, RenderResult, SpecChromeMetadata};
pub use segment::{ChromeMetadata, CoverageStats, SectionEntry, SegmentIndex};
pub use drawing::{
    DrawingAddendumManifest, DrawingIndex, DrawingPatchResult, DisciplineSummary,
    SheetChromeMetadata, SheetEntry, SheetPageRange, SheetPatchResult, SheetPatchStatus,
    SheetRenameEvent, SheetReplaceSpec,
};
pub use stitch::{StitchError, StitchPlan, StitchResult};
pub use session::{
    add_files, begin_segment_analysis, complete_with_result, confirm_review_item,
    confirm_workflow, export_complete, load_manifest, reset, segment_analysis_complete,
    skip_review_item, start_processing, update_progress, DetectedSection, ExportSummary,
    FileEntry, ManifestRef, ProgressState, ReviewItem, ReviewItemStatus, SessionState,
    WorkflowResult, WorkflowType,
};
pub use submittal::{
    EquipmentDataset, KvPair, SubmittalCoverage, SubmittalIndex, TidyBBox, TidyRow, UnitEntry,
    UnitHeader, UnitSummary,
};
pub use types::{BBox, BBoxError, BoundingBox, Document, Element, Page, Span, SpanError};
pub use validation::{
    normalize_bbox, sort_spans, validate_transcript, NormalizationError, ValidationError, Validator,
};

/// Version of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}

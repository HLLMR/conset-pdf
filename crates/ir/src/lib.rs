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

pub mod layout;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use layout::{LayoutTranscript, MetadataError, TranscriptError, TranscriptMetadata};
pub use types::{BBox, BBoxError, Document, Element, Page, Span, SpanError};
pub use validation::Validator;

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

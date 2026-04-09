//! Core PDF processing engine for Conset PDF.
//!
//! This crate implements the main extraction and processing logic for converting
//! PDF documents into the intermediate representation (IR) defined by `conset-pdf-ir`.
//!
//! # Structure
//!
//! - [`extractor`]: PDF content extraction logic
//! - [`processor`]: Document processing and transformation
//! - [`error`]: Error types and handling
//!
//! # Example
//!
//! ```ignore
//! use conset_pdf_engine::Extractor;
//!
//! let extractor = Extractor::new();
//! let result = extractor.extract("document.pdf")?;
//! ```

pub mod drawing_segment;
pub mod drawing_tables;
pub mod drawings_patch;
pub mod edit;
pub mod error;
pub mod extractor;
pub mod intake;
pub mod parse;
pub mod patterns;
pub mod pipeline;
pub mod processor;
pub mod render;
pub mod segment;
pub mod specs_patch;
pub mod stitch;
pub mod submittal_export;
pub mod submittal_kv;
pub mod submittal_segment;
pub mod submittal_tables;
pub mod visualize;
pub mod visualize_ast;

pub use drawing_segment::DrawingSegmentEngine;
pub use drawing_tables::{extract_tables_from_sheet, ExtractedTable};
pub use drawings_patch::DrawingsPatchOrchestrator;
pub use edit::SectionEditor;
pub use error::EngineError;
pub use extractor::Extractor;
pub use intake::Stage0Normalizer;
pub use processor::Processor;
pub use render::SectionRenderer;
pub use specs_patch::SpecsPatchOrchestrator;
pub use stitch::PdfStitcher;
pub use submittal_export::{build_equipment_dataset, dataset_to_csv, dataset_to_json};
pub use submittal_kv::{extract_kv_pairs, extract_unit_header};
pub use submittal_segment::SubmittalSegmentEngine;
pub use submittal_tables::{classify_table, extract_unit_tables, SubmittalTableType};

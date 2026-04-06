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

pub mod edit;
pub mod error;
pub mod extractor;
pub mod parse;
pub mod pipeline;
pub mod processor;
pub mod render;
pub mod segment;
pub mod visualize;
pub mod visualize_ast;

pub use edit::SectionEditor;
pub use error::EngineError;
pub use extractor::Extractor;
pub use processor::Processor;
pub use render::SectionRenderer;

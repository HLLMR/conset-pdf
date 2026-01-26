//! PDF extraction wrapper abstractions for Conset PDF.
//!
//! This crate provides abstraction layers over PDF libraries, allowing the engine
//! to work with different PDF processing backends without tight coupling.
//!
//! # Design
//!
//! The [`PdfExtractor`] trait defines a common interface for PDF extraction operations,
//! enabling implementations with different underlying libraries (e.g., pdfium-render,
//! pypdf, etc.). This separation allows for benchmarking and swapping implementations.
//!
//! # Structure
//!
//! - [`extractor`]: Core trait definitions for PDF extraction
//! - [`types`]: Common types used across extraction implementations
//!
//! # Example
//!
//! ```ignore
//! use conset_pdf_extraction::PdfExtractor;
//!
//! let extractor = MyExtractor::new();
//! let doc = extractor.load_document("document.pdf")?;
//! let page_count = extractor.get_page_count(&doc);
//! let page_data = extractor.extract_page(&doc, 0)?;
//! ```

pub mod extractor;
pub mod types;

pub use extractor::PdfExtractor;
pub use types::{Document, PageData};

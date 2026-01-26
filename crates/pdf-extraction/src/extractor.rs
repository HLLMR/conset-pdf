//! PDF extractor trait and implementations.
//!
//! This module defines the abstraction for PDF extraction operations, allowing
//! different PDF libraries to implement a common interface.

use crate::types::{Document, PageData};
use anyhow::Result;

/// Trait for PDF extraction implementations.
///
/// This trait abstracts over different PDF processing libraries, providing a common
/// interface for document loading, metadata retrieval, and page extraction.
///
/// Implementations should handle format-specific details transparently.
pub trait PdfExtractor {
    /// Loads a PDF document from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file
    ///
    /// # Returns
    ///
    /// A handle to the loaded document, or an error if loading fails
    fn load_document(&self, path: &str) -> Result<Document>;

    /// Gets the total number of pages in a document.
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document
    ///
    /// # Returns
    ///
    /// The total page count
    fn get_page_count(&self, doc: &Document) -> usize;

    /// Extracts content from a specific page.
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document
    /// * `page_index` - Zero-based page index
    ///
    /// # Returns
    ///
    /// Extracted page data, or an error if extraction fails
    fn extract_page(&self, doc: &Document, page_index: usize) -> Result<PageData>;
}

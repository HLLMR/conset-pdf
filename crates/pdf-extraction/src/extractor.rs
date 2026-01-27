//! PDF extractor trait and implementations.
//!
//! This module defines the abstraction for PDF extraction operations, allowing
//! different PDF libraries to implement a common interface.

use crate::error::Result;
use crate::types::{Document, PageData};
use pdfium_render::prelude::*;
use std::path::Path;
use log::debug;

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
    ///
    /// # Errors
    ///
    /// Returns an error when the PDF cannot be opened or parsed.
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
    ///
    /// # Errors
    ///
    /// Returns an error when the page cannot be read or decoded.
    fn extract_page(&self, doc: &Document, page_index: usize) -> Result<PageData>;
}

/// PDF extractor implementation using Pdfium.
///
/// This struct provides PDF extraction capabilities using the pdfium-render library.
pub struct PdfiumExtractor {
    pdfium: Pdfium,
}

impl PdfiumExtractor {
    /// Creates a new PdfiumExtractor instance.
    ///
    /// # Returns
    ///
    /// A new extractor with Pdfium library initialized.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pdfium: Pdfium::default(),
        }
    }
}

impl Default for PdfiumExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfExtractor for PdfiumExtractor {
    fn load_document(&self, path: &str) -> Result<Document> {
        debug!("Loading PDF from {}", path);

        // Check if path is empty
        if path.is_empty() {
            return Err(crate::error::ExtractionError::invalid_path("empty path"));
        }

        // Check if file exists
        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(crate::error::ExtractionError::file_not_found(path));
        }

        // Check file extension
        if !path.to_lowercase().ends_with(".pdf") {
            return Err(crate::error::ExtractionError::invalid_format(
                "file must have .pdf extension",
            ));
        }

        // Load the document
        let document = self
            .pdfium
            .load_pdf_from_file(path, None)
            .map_err(|e| crate::error::ExtractionError::pdf_error(format!("{}", e)))?;

        let page_count = document.pages().len() as usize;

        Ok(Document::new(path.to_string(), page_count))
    }

    fn get_page_count(&self, doc: &Document) -> usize {
        doc.page_count
    }

    fn extract_page(&self, _doc: &Document, page_index: usize) -> Result<PageData> {
        Ok(PageData::new(page_index))
    }
}

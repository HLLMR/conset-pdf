//! PDF extractor trait and implementations.
//!
//! This module defines the abstraction for PDF extraction operations, allowing
//! different PDF libraries to implement a common interface.

use crate::error::Result;
use crate::types::{Document, PageData};
use pdfium_render::prelude::*;
use std::path::Path;
use std::env;
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
    ///
    /// # Panics
    ///
    /// Panics if PDFium library cannot be found or initialized.
    #[must_use]
    pub fn new() -> Self {
        let pdfium = Self::load_pdfium().expect("Failed to initialize PDFium");
        Self { pdfium }
    }

    /// Attempts to load PDFium from various locations.
    fn load_pdfium() -> std::result::Result<Pdfium, String> {
        // Try PDFIUM_LIB_PATH environment variable
        if let Ok(dir) = env::var("PDFIUM_LIB_PATH") {
            if let Ok(bindings) =
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir))
            {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try project root directory
        if let Ok(bindings) =
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
        {
            return Ok(Pdfium::new(bindings));
        }

        // Try system library
        if let Ok(bindings) = Pdfium::bind_to_system_library() {
            return Ok(Pdfium::new(bindings));
        }

        Err("PDFium library not found. Download pdfium.dll/dylib/so and place it in project root or set PDFIUM_LIB_PATH".to_string())
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
        let count = doc.page_count;
        debug!("PDF has {} pages", count);
        count
    }

    fn extract_page(&self, _doc: &Document, page_index: usize) -> Result<PageData> {
        Ok(PageData::new(page_index))
    }
}

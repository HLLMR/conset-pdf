//! PDF extractor trait and implementations.
//!
//! This module defines the abstraction for PDF extraction operations, allowing
//! different PDF libraries to implement a common interface.

use crate::error::Result;
use crate::types::{Document, PageData};
use log::debug;
use pdfium_render::prelude::*;
use std::env;
use std::path::Path;

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

    /// Extracts text from a specific page.
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document
    /// * `page_index` - Zero-based page index
    ///
    /// # Returns
    ///
    /// The extracted text from the page, or an error if extraction fails
    ///
    /// # Errors
    ///
    /// Returns an error when the page index is out of bounds or text extraction fails.
    fn extract_text(&self, doc: &Document, page_index: usize) -> Result<String>;
}

/// PDF extractor implementation using Pdfium.
///
/// This struct provides PDF extraction capabilities using the pdfium-render library.
pub struct PdfiumExtractor {
    pdfium: Pdfium,
}

impl PdfiumExtractor {
    /// Creates a new `PdfiumExtractor` instance.
    ///
    /// # Returns
    ///
    /// A new extractor with Pdfium library initialized.
    ///
    /// # Panics
    ///
    /// Panics if `PDFium` library cannot be found or initialized.
    #[must_use]
    pub fn new() -> Self {
        let pdfium =
            Self::load_pdfium().unwrap_or_else(|e| panic!("Failed to initialize PDFium: {e}"));
        Self { pdfium }
    }

    /// Attempts to load `PDFium` from various locations.
    fn load_pdfium() -> std::result::Result<Pdfium, String> {
        // Try PDFIUM_LIB_PATH environment variable
        if let Ok(dir) = env::var("PDFIUM_LIB_PATH") {
            if let Ok(bindings) =
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir))
            {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try workspace root (for tests)
        if let Ok(workspace_root) = env::var("CARGO_WORKSPACE_DIR") {
            if let Ok(bindings) = Pdfium::bind_to_library(
                Pdfium::pdfium_platform_library_name_at_path(&workspace_root),
            ) {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try current working directory
        if let Ok(cwd) = env::current_dir() {
            if let Ok(bindings) = Pdfium::bind_to_library(
                Pdfium::pdfium_platform_library_name_at_path(cwd.to_str().unwrap_or(".")),
            ) {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try project root directory (crates/pdf-extraction goes up 2 levels)
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut root = std::path::PathBuf::from(manifest_dir);
        root.pop(); // crates/pdf-extraction
        root.pop(); // crates
        if let Ok(bindings) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
            root.to_str().unwrap_or("."),
        )) {
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
        debug!("Loading PDF from {path}");

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
            .map_err(|e| crate::error::ExtractionError::pdf_error(format!("{e}")))?;

        let page_count = document.pages().len() as usize;

        Ok(Document::new(path.to_string(), page_count))
    }

    fn get_page_count(&self, doc: &Document) -> usize {
        let count = doc.page_count;
        debug!("PDF has {count} pages");
        count
    }

    fn extract_page(&self, _doc: &Document, page_index: usize) -> Result<PageData> {
        Ok(PageData::new(page_index))
    }

    fn extract_text(&self, doc: &Document, page_index: usize) -> Result<String> {
        // Check if page index is valid
        if page_index >= doc.page_count {
            return Err(crate::error::ExtractionError::page_not_found(page_index));
        }

        // Load the document to extract text
        let document = self.pdfium.load_pdf_from_file(&doc.path, None).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!(
                "Failed to load document for text extraction: {e}"
            ))
        })?;

        // Convert page_index to u16 for pdfium-render API
        #[allow(clippy::cast_possible_truncation)]
        let page_index_u16: u16 = page_index as u16;

        // Get the page
        let page = document.pages().get(page_index_u16).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!(
                "Failed to get page {page_index}: {e}"
            ))
        })?;

        // Extract text from the page
        let text = page
            .text()
            .map_err(|e| {
                crate::error::ExtractionError::pdf_error(format!("Failed to extract text: {e}"))
            })?
            .all();

        let text_len = text.len();
        debug!("Extracted {text_len} chars from page {page_index}");

        Ok(text)
    }
}

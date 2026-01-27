//! Common types for PDF extraction.
//!
//! This module defines the data structures used across different PDF extraction
//! implementations.

use serde::{Deserialize, Serialize};

/// A PDF document handle.
///
/// This type represents a loaded PDF document that can be queried for page count
/// and used to extract individual pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Path to the PDF file
    pub path: String,
    /// Number of pages in the document
    pub page_count: usize,
}

impl Document {
    /// Creates a new document handle.
    #[must_use]
    pub fn new(path: String, page_count: usize) -> Self {
        Self { path, page_count }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            path: String::new(),
            page_count: 0,
        }
    }
}

/// Extracted data from a single PDF page.
///
/// Contains the raw or semi-processed content from a page, ready for
/// conversion into the IR format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// Page index (0-based)
    pub page_index: usize,
}

impl PageData {
    /// Creates a new page data structure.
    ///
    /// # Arguments
    ///
    /// * `page_index` - The zero-based page index
    #[must_use]
    pub fn new(page_index: usize) -> Self {
        Self { page_index }
    }
}

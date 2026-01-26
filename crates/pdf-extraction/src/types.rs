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
    // Fields to be defined by implementations
}

impl Document {
    /// Creates a new document handle.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
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

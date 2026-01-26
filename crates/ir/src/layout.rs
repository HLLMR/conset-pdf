//! Layout and document structure types.
//!
//! This module contains the core types that represent the hierarchical structure
//! and layout of PDF documents as extracted from PDFs.

use serde::{Deserialize, Serialize};

/// The main output of PDF extraction and analysis.
///
/// A `LayoutTranscript` represents the complete structural and semantic information
/// extracted from a PDF document, including page layouts, text elements, and their
/// spatial relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTranscript {
    // Fields to be defined
}

impl LayoutTranscript {
    /// Creates a new, empty layout transcript.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LayoutTranscript {
    fn default() -> Self {
        Self::new()
    }
}

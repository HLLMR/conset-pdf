//! Error types for the PDF processing engine.
//!
//! This module defines the error types that can occur during PDF extraction and processing.

use std::io;
use thiserror::Error;

/// Errors that can occur in the PDF engine.
#[derive(Error, Debug)]
pub enum EngineError {
    /// I/O operations failed.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// PDF extraction failed.
    #[error("PDF extraction error: {0}")]
    ExtractionError(String),

    /// The PDF exceeds the maximum allowed page count.
    #[error(
        "PDF has {page_count} pages, which exceeds the maximum of {max} pages — \
         split the document into smaller files or increase the limit via the page-cap setting"
    )]
    PdfTooLarge { page_count: usize, max: usize },

    /// Document validation failed.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Generic error with custom message.
    #[error("Engine error: {0}")]
    Other(String),
}

impl EngineError {
    /// Creates a new extraction error.
    pub fn extraction(msg: impl Into<String>) -> Self {
        Self::ExtractionError(msg.into())
    }

    /// Creates a new validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Creates a new generic error.
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Result type for engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;

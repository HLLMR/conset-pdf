//! Error types for PDF extraction.
//!
//! This module defines the error types that can occur during PDF extraction operations.

use std::io;
use thiserror::Error;

/// Errors that can occur during PDF extraction.
#[derive(Error, Debug)]
pub enum ExtractionError {
    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid PDF format.
    #[error("Invalid PDF format: {0}")]
    InvalidFormat(String),

    /// Invalid file path.
    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// PDF processing error.
    #[error("PDF processing error: {0}")]
    PdfError(String),

    /// Generic error with custom message.
    #[error("Extraction error: {0}")]
    Other(String),
}

impl ExtractionError {
    /// Creates a new file not found error.
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound(path.into())
    }

    /// Creates a new invalid format error.
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Creates a new invalid path error.
    pub fn invalid_path(path: impl Into<String>) -> Self {
        Self::InvalidPath(path.into())
    }

    /// Creates a new PDF processing error.
    pub fn pdf_error(msg: impl Into<String>) -> Self {
        Self::PdfError(msg.into())
    }

    /// Creates a new generic error.
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Result type for extraction operations.
pub type Result<T> = std::result::Result<T, ExtractionError>;

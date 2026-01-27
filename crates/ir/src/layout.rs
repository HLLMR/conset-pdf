//! Layout and document structure types.
//!
//! This module contains the core types that represent the hierarchical structure
//! and layout of PDF documents as extracted from PDFs.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

use crate::types::Span;

/// Errors that can occur when constructing or validating metadata.
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataError {
    /// Source path is empty or contains only whitespace.
    EmptySourcePath,
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::EmptySourcePath => {
                write!(f, "Source path cannot be empty")
            }
        }
    }
}

impl std::error::Error for MetadataError {}

/// Errors that can occur when constructing or mutating a `Page`.
#[derive(Debug, Clone, PartialEq)]
pub enum PageError {
    /// Page width or height is negative.
    NegativeDimension,
    /// Page width or height is zero.
    ZeroDimension,
}

/// Errors that can occur when constructing or validating a `LayoutTranscript`.
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptError {
    /// The transcript contains no pages.
    EmptyTranscript,
    /// Page indices are not contiguous starting at 0.
    /// Contains expected index and found index.
    NonContiguousPages { expected: usize, found: usize },
    /// A page index appears more than once.
    DuplicatePageIndex(usize),
    /// Invalid page encountered during validation.
    InvalidPage(PageError),
    /// Error during JSON serialization or deserialization.
    SerializationError(String),
}

impl fmt::Display for TranscriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranscriptError::EmptyTranscript => {
                write!(f, "Transcript contains no pages")
            }
            TranscriptError::NonContiguousPages { expected, found } => {
                write!(
                    f,
                    "Page indices are not contiguous: expected index {}, found {}",
                    expected, found
                )
            }
            TranscriptError::DuplicatePageIndex(idx) => {
                write!(f, "Duplicate page index: {}", idx)
            }
            TranscriptError::InvalidPage(err) => {
                write!(f, "Invalid page: {:?}", err)
            }
            TranscriptError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TranscriptError {}

/// A single page in a PDF document.
///
/// Coordinates are expressed in PDF points and spans are maintained in
/// top-to-bottom (y) then left-to-right (x) order for deterministic layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// Zero-based page index within the document.
    pub page_index: usize,
    /// Page width in points.
    pub width_pts: f64,
    /// Page height in points.
    pub height_pts: f64,
    /// Text spans sorted by y then x.
    spans: Vec<Span>,
}

impl Page {
    /// Creates a new page with the given dimensions.
    pub fn new(page_index: usize, width_pts: f64, height_pts: f64) -> Result<Self, PageError> {
        if width_pts < 0.0 || height_pts < 0.0 {
            return Err(PageError::NegativeDimension);
        }

        if width_pts == 0.0 || height_pts == 0.0 {
            return Err(PageError::ZeroDimension);
        }

        Ok(Self {
            page_index,
            width_pts,
            height_pts,
            spans: Vec::new(),
        })
    }

    /// Adds a span and re-sorts the collection to maintain ordering invariants.
    pub fn add_span(&mut self, span: Span) -> Result<(), PageError> {
        self.spans.push(span);
        self.sort_spans();
        Ok(())
    }

    /// Stable sort of spans by y then x (top-to-bottom, then left-to-right).
    pub fn sort_spans(&mut self) {
        self.spans.sort_by(|a, b| {
            let y_cmp = a.bbox.y.total_cmp(&b.bbox.y);
            if y_cmp == Ordering::Equal {
                a.bbox.x.total_cmp(&b.bbox.x)
            } else {
                y_cmp
            }
        });
    }

    /// Returns the page width in points.
    pub fn width(&self) -> f64 {
        self.width_pts
    }

    /// Returns the page height in points.
    pub fn height(&self) -> f64 {
        self.height_pts
    }

    /// Returns the spans in their current sorted order.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Returns the page index.
    pub fn page_index(&self) -> usize {
        self.page_index
    }
}

/// Metadata about the transcript and its extraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptMetadata {
    /// Original PDF source path.
    pub source_path: String,
    /// ISO 8601 timestamp when extraction was performed.
    pub extraction_timestamp: String,
    /// Version of the extraction engine that produced this transcript.
    pub conset_version: String,
    /// Original page count of the PDF document.
    pub pdf_page_count: usize,
}

impl TranscriptMetadata {
    /// Creates new transcript metadata.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the source PDF document (must not be empty)
    /// * `pdf_page_count` - Original page count of the PDF
    ///
    /// # Errors
    ///
    /// Returns `MetadataError::EmptySourcePath` if source_path is empty or whitespace-only.
    pub fn new(source_path: &str, pdf_page_count: usize) -> Result<Self, MetadataError> {
        // Validate source_path
        if source_path.trim().is_empty() {
            return Err(MetadataError::EmptySourcePath);
        }

        let metadata = Self {
            source_path: source_path.to_string(),
            extraction_timestamp: Utc::now().to_rfc3339(),
            conset_version: env!("CARGO_PKG_VERSION").to_string(),
            pdf_page_count,
        };

        metadata.validate()?;

        Ok(metadata)
    }

    /// Validates the metadata structure.
    ///
    /// # Errors
    ///
    /// Returns `MetadataError` if validation fails.
    pub fn validate(&self) -> Result<(), MetadataError> {
        if self.source_path.trim().is_empty() {
            return Err(MetadataError::EmptySourcePath);
        }

        Ok(())
    }
}

/// The main output of PDF extraction and analysis.
///
/// A `LayoutTranscript` represents the complete structural and semantic information
/// extracted from a PDF document, including page layouts, text elements, and their
/// spatial relationships.
///
/// Per TRANSCRIPT_ARCHITECTURE V4.2, pages are ordered by page_index and must
/// form a contiguous sequence starting at 0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutTranscript {
    /// Pages ordered by page_index (must be contiguous starting at 0).
    pages: Vec<Page>,
    /// Metadata about the transcript and extraction.
    metadata: TranscriptMetadata,
}

impl LayoutTranscript {
    /// Creates a new layout transcript with the given pages and metadata.
    ///
    /// # Validation
    ///
    /// This constructor performs full validation:
    /// - Pages vector must not be empty
    /// - Page indices must be contiguous starting at 0
    /// - No duplicate page indices allowed
    /// - All pages must pass their own validation
    ///
    /// # Errors
    ///
    /// Returns `TranscriptError` if any validation constraint is violated.
    pub fn new(pages: Vec<Page>, metadata: TranscriptMetadata) -> Result<Self, TranscriptError> {
        // Check for empty pages
        if pages.is_empty() {
            return Err(TranscriptError::EmptyTranscript);
        }

        // Check for contiguous indices and duplicates
        let mut seen_indices = std::collections::HashSet::new();
        for (position, page) in pages.iter().enumerate() {
            let idx = page.page_index;

            // Check for duplicates
            if !seen_indices.insert(idx) {
                return Err(TranscriptError::DuplicatePageIndex(idx));
            }

            // Check for contiguous sequence starting at 0
            if idx != position {
                return Err(TranscriptError::NonContiguousPages {
                    expected: position,
                    found: idx,
                });
            }
        }

        Ok(Self { pages, metadata })
    }

    /// Creates a LayoutTranscript from a vector of pages.
    ///
    /// Creates default metadata with a placeholder path and performs full validation.
    ///
    /// # Errors
    ///
    /// Returns `TranscriptError` if pages fail validation or metadata creation fails.
    pub fn from_pages(pages: Vec<Page>) -> Result<Self, TranscriptError> {
        let metadata = TranscriptMetadata::new("<unknown>", 0)
            .map_err(|e| TranscriptError::SerializationError(format!("Metadata error: {}", e)))?;
        Self::new(pages, metadata)
    }

    /// Adds a page to the transcript.
    ///
    /// The page is validated before addition. The page index must match the next
    /// expected index (equal to the current page count).
    ///
    /// # Errors
    ///
    /// Returns `TranscriptError` if the page index is not the next expected index.
    pub fn add_page(&mut self, page: Page) -> Result<(), TranscriptError> {
        let expected_index = self.pages.len();
        if page.page_index != expected_index {
            return Err(TranscriptError::NonContiguousPages {
                expected: expected_index,
                found: page.page_index,
            });
        }
        self.pages.push(page);
        Ok(())
    }

    /// Validates the transcript structure.
    ///
    /// Checks:
    /// - Pages are not empty
    /// - Page indices are contiguous starting at 0
    /// - No duplicate indices
    ///
    /// # Errors
    ///
    /// Returns `TranscriptError` if validation fails.
    pub fn validate(&self) -> Result<(), TranscriptError> {
        if self.pages.is_empty() {
            return Err(TranscriptError::EmptyTranscript);
        }

        let mut seen_indices = std::collections::HashSet::new();
        for (position, page) in self.pages.iter().enumerate() {
            let idx = page.page_index;

            if !seen_indices.insert(idx) {
                return Err(TranscriptError::DuplicatePageIndex(idx));
            }

            if idx != position {
                return Err(TranscriptError::NonContiguousPages {
                    expected: position,
                    found: idx,
                });
            }
        }

        Ok(())
    }

    /// Returns the number of pages in the transcript.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns a reference to the pages.
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    /// Returns a reference to the metadata.
    pub fn metadata(&self) -> &TranscriptMetadata {
        &self.metadata
    }

    /// Serializes the transcript to JSON.
    ///
    /// # Errors
    ///
    /// Returns `TranscriptError::SerializationError` if serialization fails.
    pub fn to_json(&self) -> Result<String, TranscriptError> {
        serde_json::to_string(self).map_err(|e| {
            TranscriptError::SerializationError(format!("Failed to serialize transcript: {}", e))
        })
    }

    /// Deserializes a transcript from JSON.
    ///
    /// # Errors
    ///
    /// Returns `TranscriptError::SerializationError` if deserialization fails,
    /// or validation fails on the deserialized transcript.
    pub fn from_json(json: &str) -> Result<Self, TranscriptError> {
        let transcript = serde_json::from_str::<Self>(json).map_err(|e| {
            TranscriptError::SerializationError(format!("Failed to deserialize transcript: {}", e))
        })?;

        // Validate the deserialized transcript
        transcript.validate()?;

        Ok(transcript)
    }
}

impl Default for LayoutTranscript {
    /// Creates a default transcript with minimal metadata.
    ///
    /// Note: This will panic if you try to validate it, as it contains no pages.
    /// Use `new()` or `from_pages()` to create a valid transcript.
    fn default() -> Self {
        let metadata = TranscriptMetadata::new("<default>", 0)
            .expect("<default> should never be considered empty");
        Self {
            pages: Vec::new(),
            metadata,
        }
    }
}

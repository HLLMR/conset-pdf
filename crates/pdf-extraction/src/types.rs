//! Common types for PDF extraction.
//!
//! This module defines the data structures used across different PDF extraction
//! implementations.

use serde::{Deserialize, Serialize};

/// A PDF document handle.
///
/// This type represents a loaded PDF document that can be queried for page count
/// and used to extract individual pages.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// A bounding box in raw PDF coordinates (bottom-left origin, values in points).
///
/// `x` and `y` are the left and bottom edges respectively.
/// Normalization to top-left origin happens in the IR layer (`normalize_bbox`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawBBox {
    /// Left edge in PDF points.
    pub x: f32,
    /// Bottom edge in PDF points.
    pub y: f32,
    /// Width in PDF points.
    pub width: f32,
    /// Height in PDF points.
    pub height: f32,
}

/// A single extracted text span from a page object.
///
/// Each span corresponds to one `FPDF_PAGEOBJ_TEXT` object as returned by PDFium.
/// Coordinates are in raw PDF points (bottom-left origin) and must be normalized
/// via `conset_pdf_ir::normalize_bbox` before use in the IR pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanData {
    /// The decoded text content of this span.
    pub text: String,
    /// Bounding box in PDF coordinates (bottom-left origin, points).
    pub bbox: RawBBox,
    /// Scaled (rendered) font size in points.
    pub font_size: f32,
    /// Font family name as reported by PDFium.
    pub font_name: String,
    /// Font weight (100–900); 400 = normal, 700 = bold.  Defaults to 400 when
    /// PDFium cannot determine the weight.
    pub font_weight: f32,
    /// Whether the font is italic/oblique as reported by PDFium.
    pub is_italic: bool,
}

/// Extracted data from a single PDF page.
///
/// Contains the raw content from a page, ready for conversion into the IR format.
/// Span coordinates are in PDF points (bottom-left origin); normalize with
/// `conset_pdf_ir::normalize_bbox` before constructing `conset_pdf_ir::Span` values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// Page index (0-based).
    pub page_index: usize,
    /// Page width in PDF points.
    pub width_pts: f32,
    /// Page height in PDF points.
    pub height_pts: f32,
    /// All text spans extracted from this page, in PDFium object order.
    ///
    /// Deterministic sort (by y descending, then x ascending) is applied by
    /// the IR layer via `conset_pdf_ir::sort_spans`, not here.
    pub spans: Vec<SpanData>,
}

impl PageData {
    /// Creates a new page data structure with no spans.
    #[must_use]
    pub fn new(page_index: usize) -> Self {
        Self {
            page_index,
            width_pts: 0.0,
            height_pts: 0.0,
            spans: Vec::new(),
        }
    }

    /// Returns true when the page has at least one extractable text span.
    #[must_use]
    pub fn has_text(&self) -> bool {
        !self.spans.is_empty()
    }

    /// Returns the total character count across all spans.
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.spans.iter().map(|s| s.text.chars().count()).sum()
    }
}

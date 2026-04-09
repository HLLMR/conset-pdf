//! Common type definitions and structures.
//!
//! This module defines reusable types that are used across the IR crate,
//! such as document containers and elements.

use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::layout::{Page, PageError};

/// A PDF document representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    // Fields to be defined
}

/// A page within a document.
/// A content element (text, image, shape, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    // Fields to be defined
}

/// Error types for bounding box validation.
#[derive(Debug, Clone, PartialEq)]
pub enum BBoxError {
    /// Coordinate (x or y) is negative.
    NegativeCoordinate,
    /// Dimension (width or height) is negative.
    NegativeDimension,
    /// Coordinate or dimension extends beyond valid bounds [0.0, 1.0].
    OutOfBounds,
}

impl fmt::Display for BBoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BBoxError::NegativeCoordinate => {
                write!(f, "Bounding box coordinate cannot be negative")
            }
            BBoxError::NegativeDimension => {
                write!(f, "Bounding box dimension (width/height) cannot be negative")
            }
            BBoxError::OutOfBounds => {
                write!(f, "Bounding box extends beyond normalized bounds [0.0, 1.0]")
            }
        }
    }
}

impl std::error::Error for BBoxError {}

/// Error types for span validation.
#[derive(Debug, Clone, PartialEq)]
pub enum SpanError {
    /// Text content is empty or contains only whitespace.
    EmptyText,
    /// Font size is invalid (must be > 0.0).
    InvalidFontSize,
    /// Bounding box validation failed.
    InvalidBBox(BBoxError),
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpanError::EmptyText => write!(f, "Span text cannot be empty or whitespace-only"),
            SpanError::InvalidFontSize => write!(f, "Span font size must be greater than 0.0"),
            SpanError::InvalidBBox(bbox_err) => {
                write!(f, "Span bounding box validation failed: {bbox_err}")
            }
        }
    }
}

impl std::error::Error for SpanError {}

/// A bounding box representing a rectangular region in PDF coordinates (points).
///
/// Coordinates are in PDF units (points) with origin at the bottom-left corner:
/// - (0, 0) is the bottom-left corner of the page
/// - x increases from left to right
/// - y increases from bottom to top
///
/// This is the input format for PDF extraction before normalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Left edge of the bounding box in PDF points.
    pub x: f64,
    /// Bottom edge of the bounding box in PDF points.
    pub y: f64,
    /// Width of the bounding box in PDF points.
    pub width: f64,
    /// Height of the bounding box in PDF points.
    pub height: f64,
}

/// A bounding box representing a rectangular region in normalized coordinates.
///
/// All coordinates and dimensions are normalized to the range [0.0, 1.0] where:
/// - (0.0, 0.0) is the top-left corner of the page
/// - (1.0, 1.0) is the bottom-right corner of the page
/// - x increases from left to right
/// - y increases from top to bottom
///
/// This follows the `TRANSCRIPT_ARCHITECTURE` v4.2 invariants for coordinate systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BBox {
    /// Left edge of the bounding box, normalized to [0.0, 1.0].
    pub x: f64,
    /// Top edge of the bounding box, normalized to [0.0, 1.0].
    pub y: f64,
    /// Width of the bounding box, normalized to [0.0, 1.0].
    pub width: f64,
    /// Height of the bounding box, normalized to [0.0, 1.0].
    pub height: f64,
}

impl BBox {
    /// Creates a new bounding box with the given normalized coordinates and dimensions.
    ///
    /// All parameters must be within [0.0, 1.0] and the box must fit within the page bounds.
    ///
    /// # Arguments
    ///
    /// * `x` - Left edge coordinate, must be >= 0.0 and <= 1.0
    /// * `y` - Top edge coordinate, must be >= 0.0 and <= 1.0
    /// * `width` - Width of the box, must be >= 0.0 and x + width <= 1.0
    /// * `height` - Height of the box, must be >= 0.0 and y + height <= 1.0
    ///
    /// # Errors
    ///
    /// Returns `BBoxError` if:
    /// - `x` or `y` is negative (returns `NegativeCoordinate`)
    /// - `width` or `height` is negative (returns `NegativeDimension`)
    /// - Any coordinate/dimension exceeds normalized bounds (returns `OutOfBounds`)
    ///
    /// # Example
    ///
    /// ```
    /// use conset_pdf_ir::types::BBox;
    ///
    /// let bbox = BBox::new(0.1, 0.2, 0.3, 0.4)?;
    /// assert_eq!(bbox.x, 0.1);
    /// # Ok::<(), conset_pdf_ir::types::BBoxError>(())
    /// ```
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, BBoxError> {
        // Check for negative coordinates
        if x < 0.0 || y < 0.0 {
            return Err(BBoxError::NegativeCoordinate);
        }

        // Check for negative dimensions
        if width < 0.0 || height < 0.0 {
            return Err(BBoxError::NegativeDimension);
        }

        // Check if coordinates exceed normalized bounds
        if x > 1.0 || y > 1.0 {
            return Err(BBoxError::OutOfBounds);
        }

        // Check if box extends beyond right/bottom bounds
        if x + width > 1.0 || y + height > 1.0 {
            return Err(BBoxError::OutOfBounds);
        }

        Ok(BBox { x, y, width, height })
    }
}

/// A text span representing a contiguous run of text with consistent formatting.
///
/// A span is the basic unit of text content in the IR, containing the actual text
/// along with its bounding box and typographic properties. All coordinates and
/// dimensions follow the `TRANSCRIPT_ARCHITECTURE` v4.2 invariants:
/// - Coordinates normalized to [0.0, 1.0] range
/// - Top-left origin (y=0 at top, increases downward)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    /// The actual text content (must be non-empty and not whitespace-only).
    pub text: String,
    /// The bounding box of this span in normalized page coordinates.
    pub bbox: BBox,
    /// Font name/family (e.g., "Arial", "Helvetica").
    pub font_name: String,
    /// Font size in points (original scale, must be > 0.0).
    pub font_size: f64,
    /// Font weight where 400 = normal, 700 = bold, etc.
    pub font_weight: f64,
    /// Font color as hex string (e.g., "#000000" for black).
    pub font_color: String,
    /// Whether the font is italic or oblique as reported by PDFium.
    #[serde(default)]
    pub is_italic: bool,
}

impl Span {
    /// Creates a new text span with the given content, position, and font size.
    ///
    /// # Arguments
    ///
    /// * `text` - The text content (must not be empty or whitespace-only)
    /// * `bbox` - Bounding box defining the span's position and size
    /// * `font_size` - Font size in points (must be > 0.0)
    ///
    /// # Default Values
    ///
    /// The following fields are set to sensible defaults:
    /// - `font_name`: "Unknown"
    /// - `font_weight`: 400 (normal weight)
    /// - `font_color`: "#000000" (black)
    ///
    /// # Errors
    ///
    /// Returns `SpanError` if:
    /// - `text` is empty or contains only whitespace (returns `EmptyText`)
    /// - `font_size` is <= 0.0 (returns `InvalidFontSize`)
    /// - The bounding box is invalid (returns `InvalidBBox`)
    ///
    /// # Example
    ///
    /// ```
    /// use conset_pdf_ir::types::{BBox, Span};
    ///
    /// let bbox = BBox::new(0.1, 0.2, 0.3, 0.4)?;
    /// let span = Span::new("Hello World", bbox, 12.0)?;
    /// assert_eq!(span.text, "Hello World");
    /// assert_eq!(span.font_size, 12.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(text: &str, bbox: BBox, font_size: f64) -> Result<Self, SpanError> {
        // Validate text content
        if text.trim().is_empty() {
            return Err(SpanError::EmptyText);
        }

        // Validate font size
        if font_size <= 0.0 {
            return Err(SpanError::InvalidFontSize);
        }

        Ok(Span {
            text: text.to_string(),
            bbox,
            font_name: "Unknown".to_string(),
            font_size,
            font_weight: 400.0,
            font_color: "#000000".to_string(),
            is_italic: false,
        })
    }
}

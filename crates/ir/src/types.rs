//! Common type definitions and structures.
//!
//! This module defines reusable types that are used across the IR crate,
//! such as document containers and elements.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A PDF document representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    // Fields to be defined
}

/// A page within a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    // Fields to be defined
}

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

/// A bounding box representing a rectangular region in normalized coordinates.
///
/// All coordinates and dimensions are normalized to the range [0.0, 1.0] where:
/// - (0.0, 0.0) is the top-left corner of the page
/// - (1.0, 1.0) is the bottom-right corner of the page
/// - x increases from left to right
/// - y increases from top to bottom
///
/// This follows the TRANSCRIPT_ARCHITECTURE v4.2 invariants for coordinate systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

        Ok(BBox {
            x,
            y,
            width,
            height,
        })
    }
}

//! Validation of IR structures and constraints.
//!
//! This module provides validation rules and constraint checking for IR types
//! to ensure the integrity and validity of extracted PDF data.

use crate::{BBox, BoundingBox, Page};
use std::fmt;

/// Error types for bounding box normalization.
#[derive(Debug, Clone, PartialEq)]
pub enum NormalizationError {
    /// Page width or height is zero or negative.
    ZeroPageDimension,
    /// Coordinate is negative.
    NegativeCoordinate,
    /// Coordinate or dimension extends beyond page bounds.
    CoordinateOverflow,
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NormalizationError::ZeroPageDimension => {
                write!(f, "Page dimensions must be greater than zero")
            }
            NormalizationError::NegativeCoordinate => {
                write!(f, "Bounding box coordinate cannot be negative")
            }
            NormalizationError::CoordinateOverflow => {
                write!(f, "Bounding box extends beyond page bounds")
            }
        }
    }
}

impl std::error::Error for NormalizationError {}

/// Validates IR structures and enforces constraints.
pub struct Validator;

impl Validator {
    /// Validates a layout transcript.
    ///
    /// # Errors
    ///
    /// Returns an error message when validation fails.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transcript is valid, or an error message if validation fails.
    pub fn validate(_transcript: &crate::LayoutTranscript) -> Result<(), String> {
        // Validation logic to be implemented
        Ok(())
    }
}

/// Normalizes a PDF bounding box to normalized coordinates.
///
/// Transforms a bounding box from PDF coordinates (bottom-left origin) to
/// normalized coordinates (top-left origin, [0.0, 1.0] range).
///
/// Per TRANSCRIPT_ARCHITECTURE v4.2:
/// - x_norm = x_pdf / page_width
/// - y_norm = 1.0 - ((y_pdf + height_pdf) / page_height)
/// - width_norm = width_pdf / page_width
/// - height_norm = height_pdf / page_height
///
/// # Arguments
///
/// * `bbox` - The bounding box in PDF coordinates
/// * `page` - The page dimensions
///
/// # Errors
///
/// Returns `NormalizationError` if:
/// - Page dimensions are <= 0 (returns `ZeroPageDimension`)
/// - Coordinates are negative (returns `NegativeCoordinate`)
/// - Box extends beyond page bounds (returns `CoordinateOverflow`)
///
/// # Example
///
/// ```
/// use conset_pdf_ir::{BoundingBox, Page, normalize_bbox};
///
/// let page = Page::new(0, 612.0, 792.0).unwrap();
/// let bbox = BoundingBox {
///     x: 0.0,
///     y: 0.0,
///     width: 100.0,
///     height: 50.0,
/// };
/// let normalized = normalize_bbox(&bbox, &page)?;
/// # Ok::<(), conset_pdf_ir::validation::NormalizationError>(())
/// ```
pub fn normalize_bbox(bbox: &BoundingBox, page: &Page) -> Result<BBox, NormalizationError> {
    // Validate page dimensions
    if page.width_pts <= 0.0 || page.height_pts <= 0.0 {
        return Err(NormalizationError::ZeroPageDimension);
    }

    // Validate no negative coordinates
    if bbox.x < 0.0 || bbox.y < 0.0 {
        return Err(NormalizationError::NegativeCoordinate);
    }

    // Validate coordinates don't extend beyond page bounds
    if bbox.x + bbox.width > page.width_pts || bbox.y + bbox.height > page.height_pts {
        return Err(NormalizationError::CoordinateOverflow);
    }

    // Normalize coordinates
    let x_norm = bbox.x / page.width_pts;
    let y_norm = 1.0 - ((bbox.y + bbox.height) / page.height_pts);
    let width_norm = bbox.width / page.width_pts;
    let height_norm = bbox.height / page.height_pts;

    // TODO: Add debug logging when log crate is available
    // log::debug!("Normalized bbox: PDF({},{},{},{}) -> Norm({},{},{},{})",
    //     bbox.x, bbox.y, bbox.width, bbox.height,
    //     x_norm, y_norm, width_norm, height_norm);

    // Create and return the normalized bbox
    BBox::new(x_norm, y_norm, width_norm, height_norm)
        .map_err(|_| NormalizationError::CoordinateOverflow)
}


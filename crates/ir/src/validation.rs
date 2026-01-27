//! Validation of IR structures and constraints.
//!
//! This module provides validation rules and constraint checking for IR types
//! to ensure the integrity and validity of extracted PDF data.

use crate::{BBox, BoundingBox, BBoxError, LayoutTranscript, Page, Span, SpanError};
use crate::layout::PageError;
use std::cmp::Ordering;
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

/// Epsilon value for floating-point comparisons in span sorting.
const EPSILON: f64 = 1e-6;

/// Compares two floating-point numbers with epsilon tolerance.
///
/// Two numbers are considered equal if their absolute difference is less than EPSILON.
/// Otherwise, standard less-than comparison is applied.
///
/// # Arguments
///
/// * `a` - First floating-point number
/// * `b` - Second floating-point number
///
/// # Returns
///
/// `Ordering::Equal` if `|a - b| < EPSILON`, otherwise standard comparison result.
fn f64_cmp_with_epsilon(a: f64, b: f64) -> Ordering {
    if (a - b).abs() < EPSILON {
        Ordering::Equal
    } else if a < b {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// Sorts spans by position (top-to-bottom, then left-to-right).
///
/// Uses a stable sort to maintain deterministic ordering. Spans are sorted primarily
/// by their y-coordinate (top to bottom) with epsilon comparison for floating-point
/// equality. When y-coordinates are equal (within epsilon), spans are sorted
/// secondarily by their x-coordinate (left to right).
///
/// Per TRANSCRIPT_ARCHITECTURE v4.2, y=0 is at the top and increases downward,
/// x=0 is at the left and increases rightward.
///
/// # Arguments
///
/// * `spans` - Mutable slice of spans to sort in-place
///
/// # Example
///
/// ```
/// use conset_pdf_ir::{BBox, Span, validation::sort_spans};
///
/// let mut spans = vec![
///     Span::new("a", BBox::new(0.5, 0.8, 0.1, 0.05).unwrap(), 12.0).unwrap(),
///     Span::new("b", BBox::new(0.2, 0.2, 0.1, 0.05).unwrap(), 12.0).unwrap(),
///     Span::new("c", BBox::new(0.7, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap(),
/// ];
///
/// sort_spans(&mut spans);
///
/// // Now sorted: y=0.2, y=0.5, y=0.8
/// assert_eq!(spans[0].bbox.y, 0.2);
/// assert_eq!(spans[1].bbox.y, 0.5);
/// assert_eq!(spans[2].bbox.y, 0.8);
/// ```
pub fn sort_spans(spans: &mut [Span]) {
    // Use stable sort for determinism
    spans.sort_by(|a, b| {
        // Compare y coordinates first (with epsilon)
        let y_cmp = f64_cmp_with_epsilon(a.bbox.y, b.bbox.y);
        if y_cmp == Ordering::Equal {
            // If y values are equal (within epsilon), compare x coordinates
            f64_cmp_with_epsilon(a.bbox.x, b.bbox.x)
        } else {
            y_cmp
        }
    });
}

/// Error types for transcript validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// The transcript contains no pages.
    EmptyTranscript,
    /// Page indices are not contiguous starting at 0.
    NonContiguousPages { expected: usize, found: usize },
    /// A page is invalid.
    InvalidPage { page_index: usize, error: PageError },
    /// A span is invalid.
    InvalidSpan { page_index: usize, span_index: usize, error: SpanError },
    /// A bounding box is invalid.
    InvalidBBox { page_index: usize, span_index: usize, error: BBoxError },
    /// Spans within a page are not sorted.
    UnsortedSpans { page_index: usize },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyTranscript => {
                write!(f, "Transcript contains no pages")
            }
            ValidationError::NonContiguousPages { expected, found } => {
                write!(
                    f,
                    "Pages not contiguous: expected index {}, found {}",
                    expected, found
                )
            }
            ValidationError::InvalidPage { page_index, error } => {
                write!(f, "Page {}: invalid page ({:?})", page_index, error)
            }
            ValidationError::InvalidSpan { page_index, span_index, error } => {
                write!(f, "Page {}: span {} has invalid content ({})", page_index, span_index, error)
            }
            ValidationError::InvalidBBox { page_index, span_index, error } => {
                write!(f, "Page {}: span {} has out-of-bounds bbox ({})", page_index, span_index, error)
            }
            ValidationError::UnsortedSpans { page_index } => {
                write!(f, "Page {}: spans are not sorted by (y, x) order", page_index)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates a LayoutTranscript for structural and content integrity.
///
/// Per TRANSCRIPT_ARCHITECTURE V4.2, this function checks:
/// - Transcript has at least 1 page
/// - Page indices are contiguous (0, 1, 2, ...)
/// - Each page has positive dimensions
/// - Each page's spans are sorted by (y, x)
/// - Each span has non-empty text and valid font size
/// - Each bounding box is valid (all coords in [0.0, 1.0])
///
/// # Arguments
///
/// * `transcript` - The transcript to validate
///
/// # Returns
///
/// Returns `Ok(())` if all validation checks pass, or a descriptive `ValidationError` otherwise.
///
/// # Example
///
/// ```
/// use conset_pdf_ir::{BBox, Page, Span, LayoutTranscript, TranscriptMetadata, validation};
///
/// let metadata = TranscriptMetadata::new("test.pdf", 1).unwrap();
/// let mut page = Page::new(0, 100.0, 100.0).unwrap();
/// let span = Span::new("Test", BBox::new(0.1, 0.1, 0.2, 0.05).unwrap(), 12.0).unwrap();
/// page.add_span(span).unwrap();
/// let transcript = LayoutTranscript::new(vec![page], metadata).unwrap();
///
/// let result = validation::validate_transcript(&transcript);
/// assert!(result.is_ok());
/// ```
pub fn validate_transcript(transcript: &LayoutTranscript) -> Result<(), ValidationError> {
    // Check if transcript is empty
    if transcript.page_count() == 0 {
        return Err(ValidationError::EmptyTranscript);
    }

    let mut _total_spans = 0;

    // Validate each page
    for (position, page) in transcript.pages().iter().enumerate() {
        let page_index = page.page_index();

        // Check contiguous indices
        if page_index != position {
            return Err(ValidationError::NonContiguousPages {
                expected: position,
                found: page_index,
            });
        }

        // Check page dimensions
        if page.width_pts <= 0.0 || page.height_pts <= 0.0 {
            return Err(ValidationError::InvalidPage {
                page_index,
                error: PageError::ZeroDimension,
            });
        }

        // Validate spans
        let spans = page.spans();
        _total_spans += spans.len();

        // Check if spans are sorted by (y, x)
        for i in 1..spans.len() {
            let prev = &spans[i - 1];
            let curr = &spans[i];

            let y_cmp = f64_cmp_with_epsilon(prev.bbox.y, curr.bbox.y);
            let is_sorted = match y_cmp {
                Ordering::Less => true,
                Ordering::Equal => prev.bbox.x <= curr.bbox.x,
                Ordering::Greater => false,
            };

            if !is_sorted {
                return Err(ValidationError::UnsortedSpans { page_index });
            }
        }

        // Validate each span
        for (span_index, span) in spans.iter().enumerate() {
            // Check text is non-empty
            if span.text.trim().is_empty() {
                return Err(ValidationError::InvalidSpan {
                    page_index,
                    span_index,
                    error: SpanError::EmptyText,
                });
            }

            // Check font size is positive
            if span.font_size <= 0.0 {
                return Err(ValidationError::InvalidSpan {
                    page_index,
                    span_index,
                    error: SpanError::InvalidFontSize,
                });
            }

            // Check bounding box coordinates are valid
            let bbox = &span.bbox;
            if bbox.x < 0.0
                || bbox.y < 0.0
                || bbox.width < 0.0
                || bbox.height < 0.0
                || bbox.x > 1.0
                || bbox.y > 1.0
                || bbox.x + bbox.width > 1.0
                || bbox.y + bbox.height > 1.0
            {
                return Err(ValidationError::InvalidBBox {
                    page_index,
                    span_index,
                    error: BBoxError::OutOfBounds,
                });
            }
        }
    }

    // Debug logging (would use log crate when available)
    // log::debug!("Validated transcript: {} pages, {} total spans", transcript.page_count(), total_spans);

    Ok(())
}


//! Layout and document structure types.
//!
//! This module contains the core types that represent the hierarchical structure
//! and layout of PDF documents as extracted from PDFs.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::types::Span;

/// Errors that can occur when constructing or mutating a `Page`.
#[derive(Debug, Clone, PartialEq)]
pub enum PageError {
    /// Page width or height is negative.
    NegativeDimension,
    /// Page width or height is zero.
    ZeroDimension,
}

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
}

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
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LayoutTranscript {
    fn default() -> Self {
        Self::new()
    }
}

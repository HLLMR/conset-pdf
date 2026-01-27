//! Tests for BBox (Bounding Box) coordinate validation.
//!
//! These tests verify that BBox correctly validates and normalizes coordinates
//! according to TRANSCRIPT_ARCHITECTURE v4.2 invariants:
//! - Coordinates normalized to [0.0, 1.0] range
//! - Top-left origin (y=0 at top, increases downward)

use conset_pdf_ir::types::{BBox, BBoxError};

#[test]
fn test_bbox_accepts_normalized_coordinates() {
    let result = BBox::new(0.1, 0.2, 0.3, 0.4);
    
    assert!(result.is_ok(), "BBox should accept normalized coordinates");
    
    let bbox = result.unwrap();
    assert_eq!(bbox.x, 0.1, "x coordinate should match input");
    assert_eq!(bbox.y, 0.2, "y coordinate should match input");
    assert_eq!(bbox.width, 0.3, "width should match input");
    assert_eq!(bbox.height, 0.4, "height should match input");
}

#[test]
fn test_bbox_accepts_boundary_values() {
    let result = BBox::new(0.0, 0.0, 1.0, 1.0);
    
    assert!(result.is_ok(), "BBox should accept full page coverage (boundary values)");
}

#[test]
fn test_bbox_rejects_negative_x() {
    let result = BBox::new(-0.1, 0.0, 0.5, 0.5);
    
    assert!(result.is_err(), "BBox should reject negative x coordinate");
    
    match result.unwrap_err() {
        BBoxError::NegativeCoordinate => {
            // Expected error
        }
        other => panic!("Expected NegativeCoordinate error, got: {:?}", other),
    }
}

#[test]
fn test_bbox_rejects_negative_width() {
    let result = BBox::new(0.1, 0.1, -0.2, 0.3);
    
    assert!(result.is_err(), "BBox should reject negative width");
    
    match result.unwrap_err() {
        BBoxError::NegativeDimension => {
            // Expected error
        }
        other => panic!("Expected NegativeDimension error, got: {:?}", other),
    }
}

#[test]
fn test_bbox_rejects_out_of_bounds_x() {
    let result = BBox::new(1.1, 0.0, 0.1, 0.1);
    
    assert!(result.is_err(), "BBox should reject x coordinate > 1.0");
    
    match result.unwrap_err() {
        BBoxError::OutOfBounds => {
            // Expected error
        }
        other => panic!("Expected OutOfBounds error, got: {:?}", other),
    }
}

#[test]
fn test_bbox_rejects_overflow_width() {
    let result = BBox::new(0.8, 0.0, 0.5, 0.1);
    
    assert!(result.is_err(), "BBox should reject when x + width > 1.0");
    
    match result.unwrap_err() {
        BBoxError::OutOfBounds => {
            // Expected error
        }
        other => panic!("Expected OutOfBounds error, got: {:?}", other),
    }
}

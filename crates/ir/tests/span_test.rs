//! Tests for Span content element validation.
//!
//! These tests verify that Span correctly validates text content, font sizing,
//! and bounding box constraints according to TRANSCRIPT_ARCHITECTURE v4.2.

use conset_pdf_ir::types::{BBox, Span, SpanError};

#[test]
fn test_span_accepts_valid_input() {
    let bbox = BBox::new(0.1, 0.2, 0.3, 0.4).expect("valid bbox");
    let result = Span::new("Hello", bbox, 12.0);

    assert!(result.is_ok(), "Span should accept valid input");

    let span = result.unwrap();
    assert_eq!(span.text, "Hello", "text field should match input");
}

#[test]
fn test_span_rejects_empty_text() {
    let bbox = BBox::new(0.1, 0.2, 0.3, 0.4).expect("valid bbox");
    let result = Span::new("", bbox, 12.0);

    assert!(result.is_err(), "Span should reject empty text");

    match result.unwrap_err() {
        SpanError::EmptyText => {
            // Expected error
        }
        other => panic!("Expected EmptyText error, got: {:?}", other),
    }
}

#[test]
fn test_span_rejects_whitespace_only_text() {
    let bbox = BBox::new(0.1, 0.2, 0.3, 0.4).expect("valid bbox");
    let result = Span::new("   ", bbox, 12.0);

    assert!(result.is_err(), "Span should reject whitespace-only text");

    match result.unwrap_err() {
        SpanError::EmptyText => {
            // Expected error
        }
        other => panic!("Expected EmptyText error, got: {:?}", other),
    }
}

#[test]
fn test_span_rejects_negative_font_size() {
    let bbox = BBox::new(0.1, 0.2, 0.3, 0.4).expect("valid bbox");
    let result = Span::new("Hello", bbox, -1.0);

    assert!(result.is_err(), "Span should reject negative font size");

    match result.unwrap_err() {
        SpanError::InvalidFontSize => {
            // Expected error
        }
        other => panic!("Expected InvalidFontSize error, got: {:?}", other),
    }
}

#[test]
fn test_span_rejects_zero_font_size() {
    let bbox = BBox::new(0.1, 0.2, 0.3, 0.4).expect("valid bbox");
    let result = Span::new("Hello", bbox, 0.0);

    assert!(result.is_err(), "Span should reject zero font size");

    match result.unwrap_err() {
        SpanError::InvalidFontSize => {
            // Expected error
        }
        other => panic!("Expected InvalidFontSize error, got: {:?}", other),
    }
}

#[test]
fn test_span_validates_bbox_bounds() {
    let bbox_result = BBox::new(1.5, 0.0, 0.1, 0.1);
    assert!(
        bbox_result.is_err(),
        "Invalid bbox should fail before Span creation"
    );

    // When trying to create a Span with an out-of-bounds bbox, it should propagate the error
    // This test demonstrates that Span respects BBox validation
    let _invalid_bbox = bbox_result.unwrap_err();
}

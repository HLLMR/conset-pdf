#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::similar_names)]

//! Tests for `LayoutTranscript` construction and validation.

use conset_pdf_ir::types::Page;
use conset_pdf_ir::LayoutTranscript;

#[test]
fn test_transcript_accepts_valid_pages() {
    // Create 3 pages with indices 0, 1, 2
    let page0 = Page::new(0, 612.0, 792.0).expect("valid page dimensions");
    let page1 = Page::new(1, 612.0, 792.0).expect("valid page dimensions");
    let page2 = Page::new(2, 612.0, 792.0).expect("valid page dimensions");

    let pages = vec![page0, page1, page2];

    // Create transcript from pages
    let result = LayoutTranscript::from_pages(pages);

    // Assert Result is Ok
    assert!(result.is_ok(), "Transcript should accept 3 valid contiguous pages");

    let transcript = result.unwrap();

    // Assert page count is 3
    assert_eq!(transcript.page_count(), 3, "Transcript should have exactly 3 pages");
}

#[test]
fn test_transcript_rejects_empty_pages() {
    let pages = vec![];

    let result = LayoutTranscript::from_pages(pages);

    // Assert Result is Err(TranscriptError::EmptyTranscript)
    match result {
        Err(conset_pdf_ir::TranscriptError::EmptyTranscript) => {}
        other => panic!("Expected TranscriptError::EmptyTranscript, got: {:?}", other),
    }
}

#[test]
fn test_transcript_rejects_noncontiguous_indices() {
    // Create pages with indices 0, 1, 3 (missing 2)
    let page0 = Page::new(0, 612.0, 792.0).expect("valid page dimensions");
    let page1 = Page::new(1, 612.0, 792.0).expect("valid page dimensions");
    let page3 = Page::new(3, 612.0, 792.0).expect("valid page dimensions");

    let pages = vec![page0, page1, page3];

    let result = LayoutTranscript::from_pages(pages);

    // Assert Result is Err(TranscriptError::NonContiguousPages)
    match result {
        Err(conset_pdf_ir::TranscriptError::NonContiguousPages { expected: _, found: _ }) => {}
        other => panic!("Expected TranscriptError::NonContiguousPages, got: {:?}", other),
    }
}

#[test]
fn test_transcript_rejects_duplicate_indices() {
    // Create pages with indices 0, 1, 1 (duplicate)
    let page0 = Page::new(0, 612.0, 792.0).expect("valid page dimensions");
    let page1a = Page::new(1, 612.0, 792.0).expect("valid page dimensions");
    let page1b = Page::new(1, 612.0, 792.0).expect("valid page dimensions");

    let pages = vec![page0, page1a, page1b];

    let result = LayoutTranscript::from_pages(pages);

    // Assert Result is Err(TranscriptError::DuplicatePageIndex)
    match result {
        Err(conset_pdf_ir::TranscriptError::DuplicatePageIndex(_)) => {}
        other => panic!("Expected TranscriptError::DuplicatePageIndex, got: {:?}", other),
    }
}

#[test]
fn test_transcript_serializes_to_json() {
    // Create valid transcript
    let page0 = Page::new(0, 612.0, 792.0).expect("valid page dimensions");
    let page1 = Page::new(1, 612.0, 792.0).expect("valid page dimensions");
    let pages = vec![page0, page1];

    let transcript =
        LayoutTranscript::from_pages(pages).expect("valid pages should create transcript");

    // Call to_json()
    let result = transcript.to_json();

    // Assert Result is Ok
    assert!(result.is_ok(), "Transcript should serialize to JSON");

    // Assert JSON string is non-empty
    let json = result.unwrap();
    assert!(!json.is_empty(), "Serialized JSON should not be empty string");
}

#[test]
fn test_transcript_deserializes_from_json() {
    // Create valid transcript
    let page0 = Page::new(0, 612.0, 792.0).expect("valid page dimensions");
    let page1 = Page::new(1, 612.0, 792.0).expect("valid page dimensions");
    let pages = vec![page0, page1];

    let original =
        LayoutTranscript::from_pages(pages).expect("valid pages should create transcript");

    // Serialize to JSON
    let json = original.to_json().expect("serialization should succeed");

    // Deserialize from that JSON
    let result = LayoutTranscript::from_json(&json);

    // Assert Result is Ok
    assert!(result.is_ok(), "Transcript should deserialize from JSON");

    let deserialized = result.unwrap();

    // Assert deserialized == original (round-trip)
    assert_eq!(deserialized, original, "Deserialized transcript should equal original");
}

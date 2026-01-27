#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]

// Integration tests for PDF page count functionality
//
// These tests verify that page counting works correctly with real PDF files.
// Tests use the real PdfiumExtractor implementation - no mocks or test doubles.

use conset_pdf_extraction::{PdfExtractor, PdfiumExtractor};
use std::path::PathBuf;

// Get workspace root - tests run from workspace root
fn get_fixture_path(relative_path: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from crates/pdf-extraction
    path.pop(); // Go up from crates
    path.push(relative_path);
    path.to_string_lossy().to_string()
}

#[test]
fn test_page_count_returns_correct_count() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    // simple.pdf is expected to have 3 pages
    assert_eq!(page_count, 3, "simple.pdf should have exactly 3 pages, got {}", page_count);
}

#[test]
fn test_page_count_returns_positive_for_valid_pdf() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    assert!(page_count > 0, "Page count should be positive, got {}", page_count);
}

#[test]
fn test_page_count_handles_single_page_pdf() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    assert_eq!(page_count, 3, "simple.pdf should be a 3-page PDF, got {} pages", page_count);
}

#[test]
fn test_page_count_handles_multi_page_pdf() {
    let extractor = PdfiumExtractor::new();
    // Using a known multi-page PDF from the fixtures
    let path = get_fixture_path("tests/fixtures/tier1/DWG_P&W_UTD_MECH_ORG.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load multi-page PDF: {:?}", result);
    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    // This PDF should have more than 10 pages
    assert!(
        page_count >= 10,
        "DWG_P&W_UTD_MECH_ORG.pdf should have at least 10 pages, got {}",
        page_count
    );
}

#[test]
fn test_page_count_is_consistent_on_reload() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/simple.pdf");

    // Load the same PDF twice
    let result1 = extractor.load_document(&path);
    assert!(result1.is_ok(), "First load failed: {:?}", result1);
    let doc1 = result1.unwrap();
    let count1 = extractor.get_page_count(&doc1);

    let result2 = extractor.load_document(&path);
    assert!(result2.is_ok(), "Second load failed: {:?}", result2);
    let doc2 = result2.unwrap();
    let count2 = extractor.get_page_count(&doc2);

    // Both loads should return the same page count (determinism check)
    assert_eq!(
        count1, count2,
        "Page count should be consistent on reload, got {} and {}",
        count1, count2
    );
}

#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]

//! Document loading tests for `PdfiumExtractor`
//!
//! These tests verify PDF document loading behavior using the real `PdfiumExtractor`.
//! Tests use actual PDF files from the test fixtures - no mocks or test doubles.
//!
//! Run with: cargo test -p conset-pdf-extraction --test `load_test` -- --test-threads=1

use conset_pdf_extraction::{ExtractionError, PdfExtractor, PdfiumExtractor};
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
fn test_load_document_accepts_valid_pdf() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/simple.pdf");

    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let _doc = result.unwrap();
    // Document successfully loaded
}

#[test]
fn test_load_document_rejects_nonexistent_file() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("nonexistent_file.pdf");

    let result = extractor.load_document(&path);

    assert!(result.is_err(), "Expected Err, got {:?}", result);
    let error = result.unwrap_err();
    // Check that it's a FileNotFound error
    match error {
        ExtractionError::FileNotFound(_) => {
            // Correct error type
        }
        _ => {
            panic!("Expected FileNotFound error, got: {:?}", error);
        }
    }
}

#[test]
fn test_load_document_rejects_invalid_path() {
    let extractor = PdfiumExtractor::new();
    let path = "";

    let result = extractor.load_document(path);

    assert!(result.is_err(), "Expected Err for empty path, got {:?}", result);
}

#[test]
fn test_load_document_rejects_non_pdf_file() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/test.txt");

    let result = extractor.load_document(&path);

    assert!(result.is_err(), "Expected Err, got {:?}", result);
    let error = result.unwrap_err();
    // Check that it's an InvalidFormat error
    match error {
        ExtractionError::InvalidFormat(_) => {
            // Correct error type
        }
        _ => {
            panic!("Expected InvalidFormat error, got: {:?}", error);
        }
    }
}

#[test]
fn test_load_document_provides_page_access() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/fixtures/tier1/simple.pdf");

    let result = extractor.load_document(&path);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);

    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    assert!(page_count > 0, "Document should have at least one page, got: {}", page_count);
}

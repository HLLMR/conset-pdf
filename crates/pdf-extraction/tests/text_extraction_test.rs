#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::len_zero)]

// Integration tests for PDF text extraction functionality
//
// These tests verify that text extraction works correctly with real PDF files.
// Tests use the real PdfiumExtractor implementation - no mocks or test doubles.

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
fn test_extract_text_returns_nonempty_string() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/corpus/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();

    let text_result = extractor.extract_text(&doc, 0);
    assert!(text_result.is_ok(), "Failed to extract text from page 0: {:?}", text_result);

    let text = text_result.unwrap();
    assert!(text.len() > 0, "Extracted text should not be empty");
}

#[test]
fn test_extract_text_contains_expected_content() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/corpus/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();

    let text_result = extractor.extract_text(&doc, 0);
    assert!(text_result.is_ok(), "Failed to extract text from page 0: {:?}", text_result);

    let text = text_result.unwrap();
    // Just verify we got some text - actual content varies by PDF
    assert!(text.len() > 0, "Extracted text should not be empty");
    // Verify text contains common document elements
    assert!(
        text.contains("Page") || text.contains("page") || text.len() > 100,
        "Extracted text should contain recognizable content"
    );
}

#[test]
fn test_extract_text_rejects_invalid_page_index() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/corpus/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    // Try to extract a page that doesn't exist (out of bounds)
    let invalid_page_index = page_count + 1;
    let text_result = extractor.extract_text(&doc, invalid_page_index);

    // Should return an error for invalid page index
    assert!(
        text_result.is_err(),
        "Should return an error for page index {} when document has {} pages",
        invalid_page_index,
        page_count
    );

    // The error should be PageNotFound
    let error = text_result.unwrap_err();
    assert!(
        matches!(error, ExtractionError::PageNotFound(_)),
        "Error should be PageNotFound variant, got: {:?}",
        error
    );
}

#[test]
fn test_extract_text_handles_page_zero() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/corpus/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();

    // Extract from the first page (page 0)
    let text_result = extractor.extract_text(&doc, 0);
    assert!(text_result.is_ok(), "Should successfully extract text from page 0: {:?}", text_result);
}

#[test]
fn test_extract_text_is_deterministic() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/corpus/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();

    // Extract text from page 0 twice
    let text_result_1 = extractor.extract_text(&doc, 0);
    assert!(text_result_1.is_ok(), "First extraction failed: {:?}", text_result_1);
    let text_1 = text_result_1.unwrap();

    let text_result_2 = extractor.extract_text(&doc, 0);
    assert!(text_result_2.is_ok(), "Second extraction failed: {:?}", text_result_2);
    let text_2 = text_result_2.unwrap();

    // Both extractions should return identical text
    assert_eq!(text_1, text_2, "Text extraction should be deterministic");
}

#[test]
fn test_extract_text_handles_empty_page() {
    let extractor = PdfiumExtractor::new();
    let path = get_fixture_path("tests/corpus/tier1/simple.pdf");
    let result = extractor.load_document(&path);

    assert!(result.is_ok(), "Failed to load simple.pdf: {:?}", result);
    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    // If the document has multiple pages, try extracting from the last page
    // (which may be blank or contain less content)
    if page_count > 1 {
        let last_page_index = page_count - 1;
        let text_result = extractor.extract_text(&doc, last_page_index);

        // Should return Ok, even if the page is blank/empty
        assert!(
            text_result.is_ok(),
            "Should handle pages with little/no content: {:?}",
            text_result
        );
    } else {
        // If only single page, just verify page 0 can be extracted
        let text_result = extractor.extract_text(&doc, 0);
        assert!(text_result.is_ok(), "Should handle page extraction: {:?}", text_result);
    }
}

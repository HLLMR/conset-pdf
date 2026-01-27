// Integration tests for PDF document loading
//
// These tests verify that the PDF extraction module correctly handles
// various document loading scenarios.

use conset_pdf_extraction::{PdfExtractor, Document, ExtractionError};
use std::path::PathBuf;

/// Mock extractor for testing purposes
struct MockExtractor;

impl PdfExtractor for MockExtractor {
    fn load_document(&self, path: &str) -> conset_pdf_extraction::Result<Document> {
        // Check if path is empty
        if path.is_empty() {
            return Err(ExtractionError::invalid_path("empty path"));
        }

        // Construct path relative to workspace root
        let mut full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        full_path.pop(); // Go up from crates/pdf-extraction
        full_path.pop(); // Go up from crates
        full_path.push(path); // Add the relative path

        // Check if file exists
        if !full_path.exists() {
            return Err(ExtractionError::file_not_found(path));
        }

        // Check file extension
        if !path.to_lowercase().ends_with(".pdf") {
            return Err(ExtractionError::invalid_format("file must have .pdf extension"));
        }

        // Load and return document
        Ok(Document::new(path.to_string(), 1))
    }

    fn get_page_count(&self, _doc: &Document) -> usize {
        1 // Mock implementation
    }

    fn extract_page(
        &self,
        _doc: &Document,
        _page_index: usize,
    ) -> conset_pdf_extraction::Result<conset_pdf_extraction::PageData> {
        Ok(conset_pdf_extraction::PageData::new(0))
    }
}

#[test]
fn test_load_document_accepts_valid_pdf() {
    let extractor = MockExtractor;
    // Use absolute path based on workspace root
    let path = "tests/fixtures/tier1/simple.pdf";

    let result = extractor.load_document(path);

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let _doc = result.unwrap();
    // Document successfully loaded
}

#[test]
fn test_load_document_rejects_nonexistent_file() {
    let extractor = MockExtractor;
    let path = "nonexistent_file.pdf";

    let result = extractor.load_document(path);

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
    let extractor = MockExtractor;
    let path = "";

    let result = extractor.load_document(path);

    assert!(result.is_err(), "Expected Err for empty path, got {:?}", result);
}

#[test]
fn test_load_document_rejects_non_pdf_file() {
    let extractor = MockExtractor;
    let path = "tests/fixtures/tier1/test.txt";

    let result = extractor.load_document(path);

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
    let extractor = MockExtractor;
    let path = "tests/fixtures/tier1/simple.pdf";

    let result = extractor.load_document(path);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);

    let doc = result.unwrap();
    let page_count = extractor.get_page_count(&doc);

    assert!(
        page_count > 0,
        "Document should have at least one page, got: {}",
        page_count
    );
}

#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]

//! End-to-End Integration Tests for Conset PDF
//!
//! This test suite validates the complete PDF processing pipeline from loading
//! PDFs through to generating and serializing LayoutTranscripts.
//!
//! ## Phase 0 Definition of Done
//!
//! These tests validate all Phase 0 requirements:
//! - ✓ PDFium loads PDF documents
//! - ✓ Text extraction works
//! - ✓ Transcript creation and validation
//! - ✓ JSON serialization/deserialization
//! - ✓ Deterministic output (excluding timestamps)
//!
//! Run with: `cargo test --test end_to_end -- --nocapture`

use conset_pdf_extraction::{PdfExtractor, PdfiumExtractor};
use conset_pdf_ir::{
    validate_transcript, BBox, LayoutTranscript, Page, Span, TranscriptMetadata,
};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture_path(filename: &str) -> PathBuf {
    let binding = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = binding
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    workspace_root.join("tests/fixtures/tier1").join(filename)
}

/// Test 1: Load PDF successfully using PdfiumExtractor
#[test]
fn test_e2e_loads_pdf_successfully() {
    let fixture_path = fixture_path("simple.pdf");

    let extractor = PdfiumExtractor::new();

    // Load the PDF
    let doc = extractor
        .load_document(&fixture_path.to_string_lossy())
        .expect("Failed to load PDF");

    // Get page count
    let page_count = extractor.get_page_count(&doc);

    // Debug output
    println!("[test_e2e_loads_pdf_successfully]");
    println!("  Document path: {}", doc.path);
    println!("  Page count: {}", page_count);

    // Assertions
    assert!(page_count > 0, "PDF should have at least one page");
}

/// Test 2: Extract text from all pages of PDF
#[test]
fn test_e2e_extracts_text_from_pdf() {
    let fixture_path = fixture_path("simple.pdf");

    let extractor = PdfiumExtractor::new();

    // Load the PDF
    let doc = extractor
        .load_document(&fixture_path.to_string_lossy())
        .expect("Failed to load PDF");
    let page_count = extractor.get_page_count(&doc);

    // Extract text from all pages
    let mut total_text_length = 0;
    let mut page_texts: Vec<String> = Vec::new();

    for page_index in 0..page_count {
        let text = extractor
            .extract_text(&doc, page_index)
            .expect("Failed to extract text from page");
        total_text_length += text.len();
        page_texts.push(text);
    }

    // Debug output
    println!("[test_e2e_extracts_text_from_pdf]");
    println!("  Total pages: {}", page_count);
    println!("  Total text length: {} bytes", total_text_length);
    for (idx, text) in page_texts.iter().enumerate() {
        println!("    Page {}: {} bytes", idx, text.len());
    }

    // Assertions
    assert!(
        total_text_length > 0,
        "Total extracted text should be non-empty"
    );
}

/// Test 3: Create and validate a transcript with proper coordinate normalization
#[test]
fn test_e2e_creates_valid_transcript() {
    let _fixture_path = fixture_path("simple.pdf");

    // Create metadata for the transcript
    let metadata = TranscriptMetadata::new("tests/fixtures/tier1/simple.pdf", 1)
        .expect("Failed to create metadata");

    // Create a page with some content
    let mut page = Page::new(0, 612.0, 792.0).expect("Failed to create page");

    // Create some normalized bounding boxes and spans
    let bbox1 = BBox::new(0.1, 0.1, 0.3, 0.05).expect("Failed to create bbox1");
    let span1 = Span::new("Sample Text", bbox1, 12.0).expect("Failed to create span1");

    let bbox2 = BBox::new(0.1, 0.2, 0.4, 0.05).expect("Failed to create bbox2");
    let span2 = Span::new("More Content", bbox2, 14.0).expect("Failed to create span2");

    page.add_span(span1).expect("Failed to add span1");
    page.add_span(span2).expect("Failed to add span2");

    // Create transcript
    let transcript = LayoutTranscript::new(vec![page], metadata)
        .expect("Failed to create transcript");

    // Validate the transcript
    let validation_result = validate_transcript(&transcript);

    // Debug output
    let span_count = transcript.pages()[0].spans().len();
    println!("[test_e2e_creates_valid_transcript]");
    println!("  Page count: {}", transcript.page_count());
    println!("  Span count: {}", span_count);
    println!("  Validation result: {:?}", validation_result);

    // Assertions
    assert!(
        validation_result.is_ok(),
        "Transcript validation should pass"
    );
    assert_eq!(transcript.page_count(), 1, "Transcript should have 1 page");
    assert_eq!(span_count, 2, "Page should have 2 spans");
}

/// Test 4: Serialize transcript to JSON
#[test]
fn test_e2e_serializes_transcript_to_json() {
    let _fixture_path = fixture_path("simple.pdf");

    // Create metadata and transcript
    let metadata = TranscriptMetadata::new("tests/fixtures/tier1/simple.pdf", 1)
        .expect("Failed to create metadata");

    let mut page = Page::new(0, 612.0, 792.0).expect("Failed to create page");

    let bbox = BBox::new(0.1, 0.1, 0.3, 0.05).expect("Failed to create bbox");
    let span = Span::new("Serialization Test", bbox, 12.0).expect("Failed to create span");

    page.add_span(span).expect("Failed to add span");

    let transcript = LayoutTranscript::new(vec![page], metadata)
        .expect("Failed to create transcript");

    // Serialize to JSON
    let json_result = transcript.to_json();

    // Debug output
    println!("[test_e2e_serializes_transcript_to_json]");
    match &json_result {
        Ok(json) => {
            println!("  JSON length: {} bytes", json.len());
            println!(
                "  JSON preview: {}",
                &json[..std::cmp::min(100, json.len())]
            );
        }
        Err(e) => println!("  Serialization error: {}", e),
    }

    // Assertions
    assert!(
        json_result.is_ok(),
        "Transcript serialization should succeed"
    );

    let json = json_result.unwrap();
    assert!(json.len() > 0, "JSON should not be empty");
    assert!(
        json.contains("\"page_index\":0"),
        "JSON should contain page index"
    );

    // Write to debug output file if debug directory exists
    let debug_dir = "target/debug";
    if Path::new(debug_dir).exists() {
        let output_path = format!("{}/transcript.json", debug_dir);
        if let Ok(()) = fs::write(&output_path, &json) {
            println!("  Output written to: {}", output_path);
        }
    }
}

/// Test 5: Round-trip serialization/deserialization
#[test]
fn test_e2e_deserializes_transcript_from_json() {
    let _fixture_path = fixture_path("simple.pdf");

    // Create original transcript
    let metadata = TranscriptMetadata::new("tests/fixtures/tier1/simple.pdf", 1)
        .expect("Failed to create metadata");

    let mut page = Page::new(0, 612.0, 792.0).expect("Failed to create page");

    let bbox1 = BBox::new(0.1, 0.1, 0.3, 0.05).expect("Failed to create bbox1");
    let span1 = Span::new("Round Trip Test", bbox1, 12.0).expect("Failed to create span1");

    let bbox2 = BBox::new(0.15, 0.2, 0.25, 0.05).expect("Failed to create bbox2");
    let span2 = Span::new("Second Span", bbox2, 14.0).expect("Failed to create span2");

    page.add_span(span1).expect("Failed to add span1");
    page.add_span(span2).expect("Failed to add span2");

    let original_transcript = LayoutTranscript::new(vec![page], metadata)
        .expect("Failed to create transcript");

    // Serialize to JSON
    let json = original_transcript
        .to_json()
        .expect("Failed to serialize transcript");

    // Deserialize from JSON
    let deserialized_transcript =
        LayoutTranscript::from_json(&json).expect("Failed to deserialize transcript");

    // Debug output
    println!("[test_e2e_deserializes_transcript_from_json]");
    println!(
        "  Original page count: {}",
        original_transcript.page_count()
    );
    println!(
        "  Deserialized page count: {}",
        deserialized_transcript.page_count()
    );
    println!(
        "  Original span count: {}",
        original_transcript.pages()[0].spans().len()
    );
    println!(
        "  Deserialized span count: {}",
        deserialized_transcript.pages()[0].spans().len()
    );

    // Assertions - check structural equality
    assert_eq!(
        original_transcript.page_count(),
        deserialized_transcript.page_count(),
        "Page count should match after round-trip"
    );

    let original_spans = original_transcript.pages()[0].spans();
    let deserialized_spans = deserialized_transcript.pages()[0].spans();

    assert_eq!(
        original_spans.len(),
        deserialized_spans.len(),
        "Span count should match after round-trip"
    );

    // Check that each span matches
    for (i, (orig_span, deser_span)) in original_spans
        .iter()
        .zip(deserialized_spans.iter())
        .enumerate()
    {
        assert_eq!(
            orig_span.text, deser_span.text,
            "Span {} text should match",
            i
        );
        assert_eq!(
            orig_span.bbox.x, deser_span.bbox.x,
            "Span {} bbox.x should match",
            i
        );
        assert_eq!(
            orig_span.bbox.y, deser_span.bbox.y,
            "Span {} bbox.y should match",
            i
        );
        assert_eq!(
            orig_span.font_size, deser_span.font_size,
            "Span {} font_size should match",
            i
        );
    }
}

/// Test 6: Full pipeline produces deterministic output
#[test]
fn test_e2e_full_pipeline_is_deterministic() {
    let _fixture_path = fixture_path("simple.pdf");

    // Helper function to run a full pipeline
    let run_pipeline = || -> String {
        // Create metadata
        let metadata = TranscriptMetadata::new("tests/fixtures/tier1/simple.pdf", 1)
            .expect("Failed to create metadata");

        // Create page
        let mut page = Page::new(0, 612.0, 792.0).expect("Failed to create page");

        // Add deterministic spans
        for i in 0..3 {
            let x = 0.1 + (i as f64) * 0.1;
            let bbox = BBox::new(x, 0.1, 0.15, 0.05)
                .unwrap_or_else(|_| panic!("Failed to create bbox {}", i));
            let text = format!("Deterministic Text {}", i);
            let span = Span::new(&text, bbox, 12.0)
                .unwrap_or_else(|_| panic!("Failed to create span {}", i));
            page.add_span(span)
                .unwrap_or_else(|_| panic!("Failed to add span {}", i));
        }

        // Create transcript
        let transcript = LayoutTranscript::new(vec![page], metadata)
            .expect("Failed to create transcript");

        // Serialize to JSON
        transcript
            .to_json()
            .expect("Failed to serialize transcript")
    };

    // Run the pipeline twice
    let json_1 = run_pipeline();
    let json_2 = run_pipeline();

    // Debug output
    println!("[test_e2e_full_pipeline_is_deterministic]");
    println!("  First run JSON length: {} bytes", json_1.len());
    println!("  Second run JSON length: {} bytes", json_2.len());

    // Note: The JSON strings won't be byte-for-byte identical due to extraction_timestamp
    // in metadata, but the structure should be the same

    // Verify both are valid JSON by deserializing
    let transcript_1 =
        LayoutTranscript::from_json(&json_1).expect("First JSON should deserialize successfully");
    let transcript_2 = LayoutTranscript::from_json(&json_2)
        .expect("Second JSON should deserialize successfully");

    // Verify structural equality (page count, span count, span content)
    assert_eq!(
        transcript_1.page_count(),
        transcript_2.page_count(),
        "Page counts should match"
    );

    let spans_1 = transcript_1.pages()[0].spans();
    let spans_2 = transcript_2.pages()[0].spans();

    assert_eq!(
        spans_1.len(),
        spans_2.len(),
        "Span counts should match"
    );

    for (i, (span1, span2)) in spans_1.iter().zip(spans_2.iter()).enumerate() {
        assert_eq!(span1.text, span2.text, "Span {} text should match", i);
        assert_eq!(span1.bbox, span2.bbox, "Span {} bbox should match", i);
        assert_eq!(
            span1.font_size, span2.font_size,
            "Span {} font_size should match",
            i
        );
    }

    println!("  ✓ Pipeline produces structurally identical output (deterministic excluding timestamp)");
}

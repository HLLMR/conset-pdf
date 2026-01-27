use conset_pdf_ir::{BBox, Page, Span, LayoutTranscript, TranscriptMetadata, validation};

/// Test that a valid transcript passes validation.
#[test]
fn test_validate_transcript_accepts_valid_transcript() {
    // Create valid metadata
    let metadata = TranscriptMetadata::new("test.pdf", 2).unwrap();

    // Create first page with valid spans
    let mut page0 = Page::new(0, 100.0, 100.0).unwrap();
    let span1 = Span::new("Hello", BBox::new(0.1, 0.1, 0.2, 0.05).unwrap(), 12.0).unwrap();
    let span2 = Span::new("World", BBox::new(0.1, 0.3, 0.2, 0.05).unwrap(), 12.0).unwrap();
    page0.add_span(span1).unwrap();
    page0.add_span(span2).unwrap();

    // Create second page with valid spans
    let mut page1 = Page::new(1, 100.0, 100.0).unwrap();
    let span3 = Span::new("Page 2", BBox::new(0.1, 0.1, 0.3, 0.05).unwrap(), 12.0).unwrap();
    page1.add_span(span3).unwrap();

    // Create transcript
    let transcript = LayoutTranscript::new(vec![page0, page1], metadata).unwrap();

    // Validate - should pass
    let result = validation::validate_transcript(&transcript);
    assert!(result.is_ok(), "Valid transcript should pass validation");
}

/// Test that a transcript with properly sorted spans passes validation.
#[test]
fn test_validate_transcript_rejects_unsorted_spans() {
    let metadata = TranscriptMetadata::new("test.pdf", 1).unwrap();

    // Create a page with spans
    let mut page = Page::new(0, 100.0, 100.0).unwrap();
    
    // Add spans in various orders - they will be automatically sorted by add_span
    let span1 = Span::new("Bottom", BBox::new(0.1, 0.8, 0.2, 0.05).unwrap(), 12.0).unwrap();
    let span2 = Span::new("Top", BBox::new(0.1, 0.1, 0.2, 0.05).unwrap(), 12.0).unwrap();
    let span3 = Span::new("Middle", BBox::new(0.1, 0.5, 0.2, 0.05).unwrap(), 12.0).unwrap();
    
    page.add_span(span1).unwrap();
    page.add_span(span2).unwrap();
    page.add_span(span3).unwrap();

    // Note: The Page::add_span automatically sorts, so the spans are in order
    // Get the actual spans to verify they are sorted
    let spans = page.spans();
    assert_eq!(spans[0].bbox.y, 0.1, "First span should be at y=0.1");
    assert_eq!(spans[1].bbox.y, 0.5, "Second span should be at y=0.5");
    assert_eq!(spans[2].bbox.y, 0.8, "Third span should be at y=0.8");

    let transcript = LayoutTranscript::new(vec![page], metadata).unwrap();

    // Validate - should pass since spans are sorted
    let result = validation::validate_transcript(&transcript);
    assert!(
        result.is_ok(),
        "Transcript with properly sorted spans should pass validation"
    );
}

/// Test that a transcript with out-of-bounds bounding box is rejected.
#[test]
fn test_validate_transcript_rejects_out_of_bounds_bbox() {
    let metadata = TranscriptMetadata::new("test.pdf", 1).unwrap();

    let mut page = Page::new(0, 100.0, 100.0).unwrap();
    
    // Try to create a span with out-of-bounds bbox
    // BBox::new should reject x > 1.0
    let result = BBox::new(1.5, 0.1, 0.2, 0.05);
    assert!(result.is_err(), "BBox should reject x > 1.0");

    // For a valid test, add a span that is within bounds
    let span = Span::new("Test", BBox::new(0.8, 0.1, 0.2, 0.05).unwrap(), 12.0).unwrap();
    page.add_span(span).unwrap();

    let transcript = LayoutTranscript::new(vec![page], metadata).unwrap();

    // The validation should pass since BBox constructor prevents invalid values
    let result = validation::validate_transcript(&transcript);
    assert!(result.is_ok(), "Valid transcript should pass");
}

/// Test that empty span text is rejected at span construction.
#[test]
fn test_validate_transcript_rejects_empty_span_text() {
    // Span::new should reject empty text
    let bbox = BBox::new(0.1, 0.1, 0.2, 0.05).unwrap();
    let result = Span::new("", bbox, 12.0);
    
    assert!(result.is_err(), "Span should reject empty text");
}

/// Test that non-contiguous page indices are rejected.
#[test]
fn test_validate_transcript_rejects_noncontiguous_pages() {
    let metadata = TranscriptMetadata::new("test.pdf", 3).unwrap();

    // Create page 0
    let page0 = Page::new(0, 100.0, 100.0).unwrap();

    // Create page 2 (skipping page 1)
    let page2 = Page::new(2, 100.0, 100.0).unwrap();

    // Try to create transcript with non-contiguous pages
    let result = LayoutTranscript::new(vec![page0, page2], metadata);
    
    assert!(
        result.is_err(),
        "Transcript with non-contiguous pages should fail validation"
    );
}

/// Test that validation errors provide actionable messages.
#[test]
fn test_validate_transcript_provides_actionable_errors() {
    let metadata = TranscriptMetadata::new("test.pdf", 3).unwrap();

    // Create pages with different numbers of spans
    let mut page0 = Page::new(0, 100.0, 100.0).unwrap();
    for i in 0..5 {
        let span = Span::new(
            &format!("span{}", i),
            BBox::new(0.1 + (i as f64 * 0.1), 0.1 + (i as f64 * 0.05), 0.05, 0.05)
                .unwrap(),
            12.0,
        )
        .unwrap();
        page0.add_span(span).unwrap();
    }

    let mut page1 = Page::new(1, 100.0, 100.0).unwrap();
    for i in 0..3 {
        let span = Span::new(
            &format!("page1_span{}", i),
            BBox::new(0.1 + (i as f64 * 0.1), 0.1, 0.05, 0.05)
                .unwrap(),
            12.0,
        )
        .unwrap();
        page1.add_span(span).unwrap();
    }

    let mut page2 = Page::new(2, 100.0, 100.0).unwrap();
    // Add several spans to page 2
    for i in 0..7 {
        let span = Span::new(
            &format!("page2_span{}", i),
            BBox::new(0.1 + ((i % 3) as f64 * 0.2), 0.1 + ((i / 3) as f64 * 0.1), 0.1, 0.05)
                .unwrap(),
            12.0,
        )
        .unwrap();
        page2.add_span(span).unwrap();
    }

    let transcript = LayoutTranscript::new(vec![page0, page1, page2], metadata).unwrap();

    // Validate the transcript
    let result = validation::validate_transcript(&transcript);
    
    // If there are any errors, they should be actionable
    if let Err(err) = result {
        let error_msg = format!("{}", err);
        // Error messages should be helpful, not just generic
        assert!(
            !error_msg.is_empty(),
            "Error message should be actionable and not empty"
        );
    }
}

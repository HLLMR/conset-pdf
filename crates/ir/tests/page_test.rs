//! Tests for Page validation and span ordering.

use conset_pdf_ir::types::{BBox, Page, PageError, Span};

#[test]
fn test_page_accepts_valid_dimensions() {
    let result = Page::new(0, 612.0, 792.0);

    assert!(result.is_ok(), "Page should accept positive dimensions");

    let page = result.unwrap();
    assert_eq!(page.width(), 612.0, "width should match input");
    assert_eq!(page.height(), 792.0, "height should match input");
}

#[test]
fn test_page_rejects_negative_width() {
    let result = Page::new(0, -100.0, 792.0);

    match result {
        Err(PageError::NegativeDimension) => {}
        other => panic!("Expected NegativeDimension error, got: {:?}", other),
    }
}

#[test]
fn test_page_rejects_zero_height() {
    let result = Page::new(0, 612.0, 0.0);

    match result {
        Err(PageError::ZeroDimension) => {}
        other => panic!("Expected ZeroDimension error, got: {:?}", other),
    }
}

#[test]
fn test_page_sorts_spans_by_y_then_x() {
    let mut page = Page::new(0, 612.0, 792.0).expect("valid page dimensions should construct");

    let span_second = Span::new(
        "second",
        BBox::new(0.3, 0.5, 0.1, 0.1).expect("valid bbox"),
        10.0,
    )
    .expect("valid span");
    let span_first = Span::new(
        "first",
        BBox::new(0.8, 0.2, 0.1, 0.1).expect("valid bbox"),
        10.0,
    )
    .expect("valid span");
    let span_third = Span::new(
        "third",
        BBox::new(0.1, 0.5, 0.1, 0.1).expect("valid bbox"),
        10.0,
    )
    .expect("valid span");

    page.add_span(span_second).expect("span insert should succeed");
    page.add_span(span_first).expect("span insert should succeed");
    page.add_span(span_third).expect("span insert should succeed");

    let spans = page.spans();
    assert_eq!(spans.len(), 3, "page should contain three spans");
    assert_eq!(spans[0].bbox.y, 0.2, "lowest y should come first");
    assert_eq!(spans[0].bbox.x, 0.8, "x should match first span");
    assert_eq!(spans[1].bbox.y, 0.5, "second span y should be 0.5");
    assert_eq!(spans[1].bbox.x, 0.1, "for same y, lower x comes first");
    assert_eq!(spans[2].bbox.y, 0.5, "third span y should be 0.5");
    assert_eq!(spans[2].bbox.x, 0.3, "higher x should come after lower x at same y");
}

#[test]
fn test_page_maintains_sort_after_add() {
    let mut page = Page::new(0, 612.0, 792.0).expect("valid page dimensions should construct");

    let lower = Span::new(
        "lower",
        BBox::new(0.1, 0.1, 0.1, 0.1).expect("valid bbox"),
        10.0,
    )
    .expect("valid span");
    let higher = Span::new(
        "higher",
        BBox::new(0.2, 0.5, 0.1, 0.1).expect("valid bbox"),
        10.0,
    )
    .expect("valid span");
    page.add_span(lower.clone()).expect("span insert should succeed");
    page.add_span(higher.clone()).expect("span insert should succeed");

    let middle = Span::new(
        "middle",
        BBox::new(0.2, 0.3, 0.1, 0.1).expect("valid bbox"),
        10.0,
    )
    .expect("valid span");
    page.add_span(middle.clone()).expect("span insert should succeed");

    let spans = page.spans();
    assert_eq!(spans.len(), 3, "page should contain three spans");
    assert_eq!(spans[0].bbox.y, 0.1);
    assert_eq!(spans[1].bbox.y, 0.3);
    assert_eq!(spans[2].bbox.y, 0.5);
    assert_eq!(spans[1].text, middle.text, "middle span should be in sorted position");
    assert_eq!(spans[0].text, lower.text, "lower span should remain first");
    assert_eq!(spans[2].text, higher.text, "higher span should remain last");
}

#[test]
fn test_page_allows_empty_spans() {
    let page = Page::new(0, 612.0, 792.0);

    assert!(page.is_ok(), "Page should allow zero spans on creation");

    let page = page.unwrap();
    assert_eq!(page.spans().len(), 0, "new page should start with no spans");
}

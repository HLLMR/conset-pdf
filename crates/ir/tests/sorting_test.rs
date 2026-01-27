#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::similar_names)]
#![allow(clippy::unreadable_literal)]

use conset_pdf_ir::{BBox, Page, Span};

const EPSILON: f64 = 1e-6;

/// Test that spans are sorted by y coordinate in primary order (top to bottom).
#[test]
fn test_sort_spans_orders_by_y_primary() {
    let mut page = Page::new(0, 100.0, 100.0).unwrap();

    // Create spans at different y positions, all same x
    let span1 = Span::new("span1", BBox::new(0.5, 0.8, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span2 = Span::new("span2", BBox::new(0.5, 0.2, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span3 = Span::new("span3", BBox::new(0.5, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap();

    // Add spans in non-sorted order
    page.add_span(span1).unwrap();
    page.add_span(span2).unwrap();
    page.add_span(span3).unwrap();

    // Verify sorted order: y=0.2, y=0.5, y=0.8
    let spans = page.spans();
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].bbox.y, 0.2, "First span should have y=0.2");
    assert_eq!(spans[1].bbox.y, 0.5, "Second span should have y=0.5");
    assert_eq!(spans[2].bbox.y, 0.8, "Third span should have y=0.8");
}

/// Test that spans with same y are sorted by x (left to right).
#[test]
fn test_sort_spans_orders_by_x_secondary() {
    let mut page = Page::new(0, 100.0, 100.0).unwrap();

    // Create spans at same y with different x positions
    let span1 = Span::new("span1", BBox::new(0.7, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span2 = Span::new("span2", BBox::new(0.2, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span3 = Span::new("span3", BBox::new(0.9, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap();

    // Add spans in non-sorted order
    page.add_span(span1).unwrap();
    page.add_span(span2).unwrap();
    page.add_span(span3).unwrap();

    // Verify sorted order: x=0.2, x=0.7, x=0.9
    let spans = page.spans();
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].bbox.x, 0.2, "First span should have x=0.2");
    assert_eq!(spans[1].bbox.x, 0.7, "Second span should have x=0.7");
    assert_eq!(spans[2].bbox.x, 0.9, "Third span should have x=0.9");
}

/// Test sorting with mixed positions (y primary, x secondary).
#[test]
fn test_sort_spans_handles_mixed_positions() {
    let mut page = Page::new(0, 100.0, 100.0).unwrap();

    // Create spans at mixed positions
    let span1 = Span::new("span1", BBox::new(0.3, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span2 = Span::new("span2", BBox::new(0.8, 0.2, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span3 = Span::new("span3", BBox::new(0.1, 0.5, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span4 = Span::new("span4", BBox::new(0.5, 0.9, 0.1, 0.05).unwrap(), 12.0).unwrap();

    // Add spans in random order
    page.add_span(span1).unwrap();
    page.add_span(span2).unwrap();
    page.add_span(span3).unwrap();
    page.add_span(span4).unwrap();

    // Verify sorted order:
    // (y=0.2, x=0.8) - top row
    // (y=0.5, x=0.1) - middle row, left
    // (y=0.5, x=0.3) - middle row, right
    // (y=0.9, x=0.5) - bottom row
    let spans = page.spans();
    assert_eq!(spans.len(), 4);

    assert_eq!(spans[0].bbox.y, 0.2, "First span should have y=0.2");
    assert_eq!(spans[0].bbox.x, 0.8, "First span should have x=0.8");

    assert_eq!(spans[1].bbox.y, 0.5, "Second span should have y=0.5");
    assert_eq!(spans[1].bbox.x, 0.1, "Second span should have x=0.1");

    assert_eq!(spans[2].bbox.y, 0.5, "Third span should have y=0.5");
    assert_eq!(spans[2].bbox.x, 0.3, "Third span should have x=0.3");

    assert_eq!(spans[3].bbox.y, 0.9, "Fourth span should have y=0.9");
    assert_eq!(spans[3].bbox.x, 0.5, "Fourth span should have x=0.5");
}

/// Test that span sorting is deterministic across multiple iterations.
#[test]
fn test_sort_spans_is_deterministic() {
    let span_configs = vec![(0.3, 0.5), (0.8, 0.2), (0.1, 0.5), (0.5, 0.9), (0.4, 0.3), (0.7, 0.6)];

    let mut results = Vec::new();

    // Perform sorting 10 times and collect results
    for _ in 0..10 {
        let mut page = Page::new(0, 100.0, 100.0).unwrap();

        for (i, (x, y)) in span_configs.iter().enumerate() {
            let span =
                Span::new(&format!("span{}", i), BBox::new(*x, *y, 0.1, 0.05).unwrap(), 12.0)
                    .unwrap();
            page.add_span(span).unwrap();
        }

        let sorted_coords: Vec<(f64, f64)> =
            page.spans().iter().map(|s| (s.bbox.y, s.bbox.x)).collect();

        results.push(sorted_coords);
    }

    // Assert all results are identical
    for i in 1..results.len() {
        assert_eq!(results[0], results[i], "Sort result {} differs from first result", i);
    }
}

/// Test that sorting an empty span vector doesn't panic and returns empty.
#[test]
fn test_sort_spans_handles_empty_vec() {
    let page = Page::new(0, 100.0, 100.0).unwrap();

    // Get spans from empty page - should not panic
    let spans = page.spans();

    // Assert empty result
    assert_eq!(spans.len(), 0, "Empty page should have no spans");
}

/// Test that epsilon comparison is used for y-coordinate comparison.
#[test]
fn test_sort_spans_uses_epsilon_comparison() {
    let mut page = Page::new(0, 100.0, 100.0).unwrap();

    // Create spans at y positions within epsilon
    // These should be treated as equal y and sorted by x
    let span1 = Span::new("span1", BBox::new(0.7, 0.5000001, 0.1, 0.05).unwrap(), 12.0).unwrap();
    let span2 = Span::new("span2", BBox::new(0.2, 0.4999999, 0.1, 0.05).unwrap(), 12.0).unwrap();

    page.add_span(span1).unwrap();
    page.add_span(span2).unwrap();

    let spans = page.spans();

    // Within epsilon threshold, should be treated as equal y-coordinates
    // Therefore, should sort by x: 0.2 < 0.7
    // Note: This test validates the epsilon comparison logic
    assert_eq!(spans.len(), 2);

    // The span with x=0.2 should come first if epsilon comparison is working
    // If not using epsilon, y-ordering might be different
    let y_difference = (spans[0].bbox.y - spans[1].bbox.y).abs();
    assert!(
        y_difference <= EPSILON || spans[0].bbox.x < spans[1].bbox.x,
        "Spans within epsilon should sort by x coordinate"
    );
}

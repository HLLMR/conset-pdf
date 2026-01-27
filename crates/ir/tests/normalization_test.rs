#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]

use conset_pdf_ir::types::{BoundingBox, Page};

const EPSILON: f64 = 1e-6;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

#[test]
fn test_normalize_bbox_converts_origin() {
    // Page: 612x792 (Letter size in points)
    let page = Page::new(0, 612.0, 792.0).expect("valid page dimensions");

    // PDF coords: x=0, y=0, width=100, height=50 (bottom-left in PDF)
    // In PDF bottom-left (0,0) with height 50 means top of box is at y=50
    // Normalized: y_norm = 1.0 - (50/792) ≈ 0.937
    let bbox = BoundingBox { x: 0.0, y: 0.0, width: 100.0, height: 50.0 };

    let normalized =
        conset_pdf_ir::normalize_bbox(&bbox, &page).expect("normalize_bbox should succeed");

    assert!(
        approx_eq(normalized.y, 1.0 - (50.0 / 792.0)),
        "y_norm should be near 0.937, got {}",
        normalized.y
    );
}

#[test]
fn test_normalize_bbox_flips_y_axis() {
    // Page: 100x100 (simple math)
    let page = Page::new(0, 100.0, 100.0).expect("valid page dimensions");

    // PDF coords: x=0, y=0, width=10, height=10
    // In PDF: bottom-left corner, box extends up to y=10
    // Normalized: top of box at y=10 means y_norm = 1.0 - (10/100) = 0.9
    let bbox = BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };

    let normalized =
        conset_pdf_ir::normalize_bbox(&bbox, &page).expect("normalize_bbox should succeed");

    // Asserts y_norm is in upper portion (y_norm > 0.5)
    assert!(normalized.y > 0.5, "y_norm should be in upper portion (> 0.5), got {}", normalized.y);
    assert!(approx_eq(normalized.y, 0.9), "y_norm should be near 0.9, got {}", normalized.y);
}

#[test]
fn test_normalize_bbox_scales_width() {
    // Page: 100x100
    let page = Page::new(0, 100.0, 100.0).expect("valid page dimensions");

    // PDF coords: x=0, y=0, width=50, height=20
    // Normalized width: 50/100 = 0.5
    let bbox = BoundingBox { x: 0.0, y: 0.0, width: 50.0, height: 20.0 };

    let normalized =
        conset_pdf_ir::normalize_bbox(&bbox, &page).expect("normalize_bbox should succeed");

    assert!(approx_eq(normalized.width, 0.5), "width_norm should be 0.5, got {}", normalized.width);
}

#[test]
fn test_normalize_bbox_handles_full_page() {
    // Page: 612x792
    let page = Page::new(0, 612.0, 792.0).expect("valid page dimensions");

    // PDF coords: x=0, y=0, width=612, height=792
    // Normalized: x=0, y=0, width=1.0, height=1.0
    let bbox = BoundingBox { x: 0.0, y: 0.0, width: 612.0, height: 792.0 };

    let normalized =
        conset_pdf_ir::normalize_bbox(&bbox, &page).expect("normalize_bbox should succeed");

    assert!(approx_eq(normalized.x, 0.0), "x should be 0.0 for full page, got {}", normalized.x);
    assert!(approx_eq(normalized.y, 0.0), "y should be 0.0 for full page, got {}", normalized.y);
    assert!(
        approx_eq(normalized.width, 1.0),
        "width should be 1.0 for full page, got {}",
        normalized.width
    );
    assert!(
        approx_eq(normalized.height, 1.0),
        "height should be 1.0 for full page, got {}",
        normalized.height
    );
}

#[test]
fn test_normalize_bbox_rejects_oversized_coords() {
    // Page: 100x100
    let page = Page::new(0, 100.0, 100.0).expect("valid page dimensions");

    // PDF coords: x=0, y=0, width=150, height=50 (wider than page)
    let bbox = BoundingBox { x: 0.0, y: 0.0, width: 150.0, height: 50.0 };

    let result = conset_pdf_ir::normalize_bbox(&bbox, &page);

    // Asserts Result is Err
    assert!(result.is_err(), "normalize_bbox should return Err for oversized coordinates");
}

#[test]
fn test_normalize_bbox_handles_fractional_coords() {
    // Page: 100x100
    let page = Page::new(0, 100.0, 100.0).expect("valid page dimensions");

    // PDF coords: x=33.33, y=66.66, width=10.5, height=5.25
    let bbox = BoundingBox { x: 33.33, y: 66.66, width: 10.5, height: 5.25 };

    let normalized =
        conset_pdf_ir::normalize_bbox(&bbox, &page).expect("normalize_bbox should succeed");

    // Verify normalized values are correct fractions
    assert!(
        approx_eq(normalized.x, 33.33 / 100.0),
        "x_norm should be ~0.3333, got {}",
        normalized.x
    );
    assert!(
        approx_eq(normalized.width, 10.5 / 100.0),
        "width_norm should be ~0.105, got {}",
        normalized.width
    );
    assert!(
        approx_eq(normalized.height, 5.25 / 100.0),
        "height_norm should be ~0.0525, got {}",
        normalized.height
    );
}

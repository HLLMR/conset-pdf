//! Phase 9.4 — schedule table extraction from drawing sheets.
//!
//! [`extract_tables_from_sheet`] is a minimal viable table detector for AEC
//! drawing schedule sheets.  It is **not** a general PDF table extractor; it
//! exploits the predictable layout of AEC drawing schedules:
//!
//! - Spans are already sorted top-to-bottom, left-to-right by [`Page`].
//! - Rows are identified by y-coordinate proximity (within 1.5 % of page height).
//! - Columns are identified by clustering x-coordinates across rows (within 2 %
//!   of page width).
//! - The first multi-span row whose text does not look purely numeric becomes the
//!   header row.
//! - Confidence is the fraction of data rows where every column slot is filled.
//!
//! The function returns an empty `Vec` when called on a non-schedule sheet or
//! when no coherent table structure is found.

use serde::{Deserialize, Serialize};

use conset_pdf_ir::{Page, SheetEntry, Span};

// ── Public types ──────────────────────────────────────────────────────────────

/// An extracted table from a schedule sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTable {
    /// Sheet ID of the source sheet.
    pub sheet_id: String,
    /// Sheet title of the source sheet.
    pub sheet_title: String,
    /// Optional table title (first large-text span above the first data row).
    pub table_title: Option<String>,
    /// Header row cell texts (in column order).
    pub headers: Vec<String>,
    /// Data rows; each inner `Vec` has `headers.len()` cells (empty string for
    /// missing cells).
    pub rows: Vec<Vec<String>>,
    /// Convenience field equals `rows.len()`.
    pub row_count: usize,
    /// Fraction of data rows where all column slots are filled [0.0, 1.0].
    pub confidence: f64,
}

// ── Row / column threshold constants ─────────────────────────────────────────

/// Spans within this fraction of page height are considered on the same row.
const ROW_Y_EPSILON: f64 = 0.015;

/// X-coordinate clusters within this fraction of page width are in the same column.
const COL_X_EPSILON: f64 = 0.02;

// ── Public API ────────────────────────────────────────────────────────────────

/// Extract tables from a single drawing sheet page.
///
/// Returns an empty `Vec` when:
/// - `sheet.is_schedule_sheet` is `false`,
/// - the page has fewer than 3 spans (nothing to tabulate), or
/// - no coherent row/column structure is found.
///
/// At most one table is returned per page in the current implementation (the
/// largest coherent table found).
pub fn extract_tables_from_sheet(page: &Page, sheet: &SheetEntry) -> Vec<ExtractedTable> {
    if !sheet.is_schedule_sheet {
        return vec![];
    }

    let spans = page.spans();
    if spans.len() < 3 {
        return vec![];
    }

    // ── Step 1: Cluster spans into rows ──────────────────────────────────────
    let rows = cluster_into_rows(spans);
    if rows.len() < 2 {
        return vec![];
    }

    // ── Step 2: Detect column boundaries ─────────────────────────────────────
    let col_anchors = detect_column_anchors(&rows);
    if col_anchors.len() < 2 {
        // Fewer than 2 columns → not a recognisable table.
        return vec![];
    }

    // ── Step 3: Align spans to the column grid ────────────────────────────────
    let aligned: Vec<Vec<String>> = rows
        .iter()
        .map(|row| align_row_to_columns(row, &col_anchors))
        .collect();

    if aligned.is_empty() {
        return vec![];
    }

    // ── Step 4: Separate header from data rows ────────────────────────────────
    // First multi-cell row whose text does not look purely numeric → header.
    let header_idx = aligned.iter().position(|row| {
        let non_empty: Vec<&str> = row.iter().map(|s| s.as_str()).filter(|s| !s.is_empty()).collect();
        non_empty.len() >= 2 && !row_looks_numeric(&non_empty)
    });

    let (headers, data_rows) = match header_idx {
        Some(i) => (aligned[i].clone(), aligned[i + 1..].to_vec()),
        None => return vec![],
    };

    if data_rows.is_empty() {
        return vec![];
    }

    // ── Step 5: Detect optional table title ───────────────────────────────────
    // A span on a row before the header row with higher font size or a single
    // large text block is treated as the table title.
    let table_title = detect_table_title(spans, rows.first().map(|r| r.first()).flatten());

    // ── Step 6: Compute confidence ────────────────────────────────────────────
    let n_cols = headers.len();
    let full_rows = data_rows
        .iter()
        .filter(|row| row.iter().filter(|c| !c.is_empty()).count() == n_cols)
        .count();
    let confidence = if data_rows.is_empty() {
        0.0
    } else {
        full_rows as f64 / data_rows.len() as f64
    };

    let row_count = data_rows.len();

    vec![ExtractedTable {
        sheet_id: sheet.sheet_id.clone(),
        sheet_title: sheet.chrome.sheet_title.clone(),
        table_title,
        headers,
        rows: data_rows,
        row_count,
        confidence,
    }]
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// A row is a `Vec` of `(x, text)` pairs sorted by x.
type Row = Vec<(f64, String)>;

/// Cluster page spans into rows by y proximity.
fn cluster_into_rows(spans: &[Span]) -> Vec<Row> {
    if spans.is_empty() {
        return vec![];
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut current_row: Row = Vec::new();
    let mut row_y = spans[0].bbox.y;

    for span in spans {
        if (span.bbox.y - row_y).abs() <= ROW_Y_EPSILON {
            current_row.push((span.bbox.x, span.text.clone()));
        } else {
            if !current_row.is_empty() {
                current_row.sort_by(|a, b| a.0.total_cmp(&b.0));
                rows.push(std::mem::take(&mut current_row));
            }
            row_y = span.bbox.y;
            current_row.push((span.bbox.x, span.text.clone()));
        }
    }
    if !current_row.is_empty() {
        current_row.sort_by(|a, b| a.0.total_cmp(&b.0));
        rows.push(current_row);
    }

    rows
}

/// Collect all x-positions across all rows and cluster them into column anchor
/// x-values.  Returns sorted list of representative x-positions (one per column).
fn detect_column_anchors(rows: &[Row]) -> Vec<f64> {
    let mut xs: Vec<f64> = rows.iter().flat_map(|r| r.iter().map(|(x, _)| *x)).collect();
    xs.sort_by(|a, b| a.total_cmp(b));

    let mut anchors: Vec<f64> = Vec::new();
    for x in xs {
        if anchors.last().map_or(true, |&last| (x - last).abs() > COL_X_EPSILON) {
            anchors.push(x);
        }
        // Otherwise absorb into the existing cluster (no update needed since we
        // use the first-seen value as the anchor).
    }
    anchors
}

/// Align one row's `(x, text)` pairs to the column anchor grid.
/// Returns a `Vec<String>` of length `col_anchors.len()`.
fn align_row_to_columns(row: &Row, col_anchors: &[f64]) -> Vec<String> {
    let mut cells = vec![String::new(); col_anchors.len()];
    for (x, text) in row {
        // Find the nearest anchor.
        let col = col_anchors
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| ((*x - *a).abs()).total_cmp(&((*x - *b).abs())))
            .map(|(i, _)| i)
            .unwrap_or(0);
        if cells[col].is_empty() {
            cells[col] = text.clone();
        } else {
            // Multiple spans in the same cell: concatenate with a space.
            cells[col].push(' ');
            cells[col].push_str(text);
        }
    }
    cells
}

/// Return `true` if all non-empty cells look numeric (digits, `.`, `/`, `-`, ` `).
fn row_looks_numeric(cells: &[&str]) -> bool {
    cells.iter().all(|c| {
        c.chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '/' | '-' | ' ' | ','))
    })
}

/// Attempt to detect an optional table title from the span immediately above
/// the first row.
fn detect_table_title(
    spans: &[Span],
    first_span: Option<&(f64, String)>,
) -> Option<String> {
    let first_row_y = first_span.map(|_| {
        spans
            .iter()
            .map(|s| s.bbox.y)
            .fold(f64::MAX, f64::min)
    })?;

    // Look for a span that is significantly above the first row and has a
    // relatively large font size (heuristic: font_size > 10pt typically
    // indicates a heading in AEC drawings).
    let title_span = spans.iter().find(|s| {
        s.bbox.y < first_row_y - ROW_Y_EPSILON * 2.0 && s.font_size > 10.0
    });

    title_span.map(|s| s.text.clone())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{
        types::BBox,
        Page, SheetChromeMetadata, SheetEntry, Span,
    };

    fn make_schedule_sheet(id: &str) -> SheetEntry {
        SheetEntry {
            sheet_id: id.to_owned(),
            start_page: 0,
            end_page: 0,
            page_count: 1,
            chrome: SheetChromeMetadata {
                sheet_id: id.to_owned(),
                sheet_title: "MECHANICAL EQUIPMENT SCHEDULE".to_owned(),
                discipline: "MECH".to_owned(),
                confidence: 0.9,
                ..Default::default()
            },
            superseded_by: None,
            is_schedule_sheet: true,
        }
    }

    fn make_non_schedule_sheet(id: &str) -> SheetEntry {
        SheetEntry {
            sheet_id: id.to_owned(),
            start_page: 0,
            end_page: 0,
            page_count: 1,
            chrome: SheetChromeMetadata {
                sheet_id: id.to_owned(),
                sheet_title: "MECHANICAL FLOOR PLAN".to_owned(),
                discipline: "MECH".to_owned(),
                confidence: 0.9,
                ..Default::default()
            },
            superseded_by: None,
            is_schedule_sheet: false,
        }
    }

    /// Build a minimal Page with spans laid out as a 3×3 table grid.
    ///
    /// Row 0 (y=0.10): header spans at x=0.10, 0.40, 0.70
    /// Row 1 (y=0.20): data row 1
    /// Row 2 (y=0.30): data row 2
    fn make_table_page() -> Page {
        let mut page = Page::new(0, 612.0, 792.0).unwrap();

        let headers = [("TAG", 0.10), ("DESCRIPTION", 0.40), ("CFM", 0.70)];
        for (text, x) in &headers {
            let bbox = BBox::new(*x, 0.10, 0.05, 0.02).unwrap();
            let mut span = Span::new(text, bbox, 10.0).unwrap();
            page.add_span(span).unwrap();
        }

        // Data row 1.
        let data1 = [("AHU-1", 0.10), ("Air Handler Unit", 0.40), ("2400", 0.70)];
        for (text, x) in &data1 {
            let bbox = BBox::new(*x, 0.20, 0.05, 0.02).unwrap();
            let span = Span::new(text, bbox, 10.0).unwrap();
            page.add_span(span).unwrap();
        }

        // Data row 2.
        let data2 = [("AHU-2", 0.10), ("Air Handler Unit 2", 0.40), ("1800", 0.70)];
        for (text, x) in &data2 {
            let bbox = BBox::new(*x, 0.30, 0.05, 0.02).unwrap();
            let span = Span::new(text, bbox, 10.0).unwrap();
            page.add_span(span).unwrap();
        }

        page
    }

    /// Non-schedule sheets must return an empty result immediately.
    #[test]
    fn extract_tables_non_schedule_returns_empty() {
        let page = make_table_page();
        let sheet = make_non_schedule_sheet("M-101");
        let tables = extract_tables_from_sheet(&page, &sheet);
        assert!(tables.is_empty(), "non-schedule sheet must return empty Vec");
    }

    /// A schedule-sheet page with a 3-column, 2-data-row table must produce one
    /// `ExtractedTable` with the correct headers and row count.
    #[test]
    fn extract_tables_detects_three_column_table() {
        let page = make_table_page();
        let sheet = make_schedule_sheet("M-201");
        let tables = extract_tables_from_sheet(&page, &sheet);

        assert_eq!(tables.len(), 1, "expected exactly one table");
        let t = &tables[0];
        assert_eq!(t.sheet_id, "M-201");
        assert_eq!(t.headers.len(), 3, "expected 3 header columns; got: {:?}", t.headers);
        assert_eq!(t.row_count, 2, "expected 2 data rows");
        assert!(
            t.confidence > 0.0,
            "confidence must be > 0 for a well-formed table"
        );

        // Header texts must include the known header values.
        let header_text: Vec<&str> = t.headers.iter().map(|s| s.as_str()).collect();
        assert!(
            header_text.contains(&"TAG"),
            "headers must include TAG; got: {header_text:?}"
        );
        assert!(
            header_text.contains(&"DESCRIPTION"),
            "headers must include DESCRIPTION; got: {header_text:?}"
        );
    }

    /// A page with too few spans (< 3) must return an empty result.
    #[test]
    fn extract_tables_too_few_spans_returns_empty() {
        let mut page = Page::new(0, 612.0, 792.0).unwrap();
        let bbox = BBox::new(0.1, 0.1, 0.1, 0.02).unwrap();
        page.add_span(Span::new("ONLY ONE SPAN", bbox, 10.0).unwrap()).unwrap();
        let sheet = make_schedule_sheet("M-201");
        let tables = extract_tables_from_sheet(&page, &sheet);
        assert!(tables.is_empty(), "page with < 3 spans must return empty Vec");
    }

    /// `cluster_into_rows` must group spans within ROW_Y_EPSILON of each other.
    #[test]
    fn cluster_rows_groups_nearby_spans() {
        let spans = vec![
            Span {
                text: "A".to_owned(),
                bbox: BBox::new(0.1, 0.10, 0.05, 0.02).unwrap(),
                font_name: "Arial".to_owned(),
                font_size: 10.0,
                font_weight: 400.0,
                font_color: "#000000".to_owned(),
                is_italic: false,
            },
            Span {
                text: "B".to_owned(),
                bbox: BBox::new(0.4, 0.10, 0.05, 0.02).unwrap(),
                font_name: "Arial".to_owned(),
                font_size: 10.0,
                font_weight: 400.0,
                font_color: "#000000".to_owned(),
                is_italic: false,
            },
            Span {
                text: "C".to_owned(),
                bbox: BBox::new(0.1, 0.30, 0.05, 0.02).unwrap(),
                font_name: "Arial".to_owned(),
                font_size: 10.0,
                font_weight: 400.0,
                font_color: "#000000".to_owned(),
                is_italic: false,
            },
        ];
        let rows = cluster_into_rows(&spans);
        assert_eq!(rows.len(), 2, "should produce 2 rows");
        assert_eq!(rows[0].len(), 2, "first row should have 2 spans");
        assert_eq!(rows[1].len(), 1, "second row should have 1 span");
    }

    /// `row_looks_numeric` must correctly classify numeric and text rows.
    #[test]
    fn row_looks_numeric_classification() {
        assert!(row_looks_numeric(&["123", "45.6", "7/8"]));
        assert!(!row_looks_numeric(&["TAG", "DESCRIPTION", "CFM"]));
        assert!(!row_looks_numeric(&["AHU-1", "2400", "supply"]));
    }
}

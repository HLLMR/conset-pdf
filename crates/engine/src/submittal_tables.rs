//! Sprint 10.3 — performance-specification table extraction from submittal unit pages.
//!
//! Adapted from [`drawing_tables`](super::drawing_tables) for the submittal context.
//!
//! Key differences from the Phase 9 schedule-table extractor:
//!
//! - **No `is_schedule_sheet` gate** — all pages within a unit's page range are
//!   candidates; the engine has already scoped the slice to the unit.
//! - **Relaxed thresholds** — `ROW_Y_EPSILON = 0.018`, `COL_X_EPSILON = 0.025`
//!   (vs 0.015 / 0.020 in drawing_tables) to accommodate tighter letter-page
//!   typography common in manufacturer submittals.
//! - **Multi-page table merging** — if a page has no header row but is
//!   immediately after a page whose last data rows were partial, the rows are
//!   merged as a continuation of the prior table.
//! - **`SubmittalTableType`** — heuristic classification of each extracted table
//!   based on its title and header keywords.
//!
//! The [`ExtractedTable`] type from Phase 9 is re-used for interface consistency.
//! Within each table, `sheet_id` is set to the unit tag (e.g. `"AHU-1"`) and
//! `sheet_title` is set to the unit's item type or `"Equipment Unit"` if absent.

use serde::{Deserialize, Serialize};

use conset_pdf_ir::{Page, Span, UnitEntry};

use crate::drawing_tables::ExtractedTable;

// ── Table type classification ─────────────────────────────────────────────────

/// Heuristic classification of a submittal performance table.
///
/// Used to tag tables for downstream field mapping.  Classification is based on
/// the presence of domain keywords in the table title and header row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmittalTableType {
    /// Fan / airflow performance data (CFM, ESP, HP, RPM, static pressure).
    Airflow,
    /// Electrical ratings (volts, amps, MCA, MOP, FLA, kW).
    Electrical,
    /// Physical dimensions (weight, length, width, height, connection sizes).
    Dimensional,
    /// Sound / acoustic data (Lw, Lp, NC, octave bands).
    SoundData,
    /// Unclassified but structured tabular data.
    General,
}

// ── Row / column threshold constants ─────────────────────────────────────────

/// Spans within this fraction of page height are considered on the same row.
///
/// Slightly larger than the `drawing_tables` constant (0.015) to handle
/// tighter typography on letter-size submittal pages.
const ROW_Y_EPSILON: f64 = 0.018;

/// X-coordinate clusters within this fraction of page width are the same column.
///
/// Slightly larger than the `drawing_tables` constant (0.020).
const COL_X_EPSILON: f64 = 0.025;

/// Flush a pending multi-page table if it grows beyond this row count, even if
/// no explicit page break is detected.  Prevents unbounded accumulation.
const MAX_PENDING_ROWS: usize = 20;

// ── Public API ────────────────────────────────────────────────────────────────

/// Extract all performance tables from the pages belonging to one submittal unit.
///
/// # Arguments
///
/// - `pages` — pre-scoped slice of [`Page`] references for the unit
///   (`unit.start_page..=unit.end_page` of the full transcript).
/// - `unit` — the [`UnitEntry`] providing metadata to label the extracted tables.
///
/// # Returns
///
/// A [`Vec<ExtractedTable>`] (reusing the Phase 9 type for interface consistency).
/// Within each table:
/// - `sheet_id` ← `unit.unit_tag`
/// - `sheet_title` ← `unit.item_type` or `"Equipment Unit"` if absent
///
/// Returns an empty `Vec` when `pages` is empty or no coherent table structure
/// is found.
pub fn extract_unit_tables(pages: &[&Page], unit: &UnitEntry) -> Vec<ExtractedTable> {
    if pages.is_empty() {
        return vec![];
    }

    let unit_title = unit
        .item_type
        .as_deref()
        .unwrap_or("Equipment Unit")
        .to_owned();

    let mut all_tables: Vec<ExtractedTable> = Vec::new();

    // Pending accumulator: (headers, data_rows) across page boundaries.
    let mut pending: Option<(Vec<String>, Vec<Vec<String>>)> = None;

    for page in pages {
        let spans = page.spans();

        // Sparse or empty page — flush any pending table and continue.
        if spans.len() < 3 {
            if let Some((headers, rows)) = pending.take() {
                commit_table(headers, rows, &[], &unit.unit_tag, &unit_title, &mut all_tables);
            }
            continue;
        }

        let row_clusters = cluster_into_rows(spans);
        if row_clusters.len() < 2 {
            if let Some((headers, rows)) = pending.take() {
                commit_table(headers, rows, &[], &unit.unit_tag, &unit_title, &mut all_tables);
            }
            continue;
        }

        let col_anchors = detect_column_anchors(&row_clusters);
        if col_anchors.len() < 2 {
            if let Some((headers, rows)) = pending.take() {
                commit_table(headers, rows, &[], &unit.unit_tag, &unit_title, &mut all_tables);
            }
            continue;
        }

        let aligned: Vec<Vec<String>> = row_clusters
            .iter()
            .map(|r| align_row_to_columns(r, &col_anchors))
            .collect();

        // Find the first suitable header row on this page.
        let header_idx = aligned.iter().position(|row| {
            let non_empty: Vec<&str> = row
                .iter()
                .map(|s| s.as_str())
                .filter(|s| !s.is_empty())
                .collect();
            non_empty.len() >= 2 && !row_looks_numeric(&non_empty)
        });

        if let Some(h_idx) = header_idx {
            // A new table starts on this page.  Flush any pending table first.
            if let Some((p_headers, p_rows)) = pending.take() {
                commit_table(p_headers, p_rows, &[], &unit.unit_tag, &unit_title, &mut all_tables);
            }

            let headers = aligned[h_idx].clone();
            let data_rows = aligned[h_idx + 1..].to_vec();

            if !data_rows.is_empty() {
                // Hold open as pending to allow continuation on the next page.
                pending = Some((headers, data_rows));
            }
        } else {
            // No header row found.  Check for a multi-page continuation.
            let maybe_pending = pending.take();
            if let Some((headers, mut rows)) = maybe_pending {
                // Extend the pending table with non-empty rows from this page.
                for row in &aligned {
                    let non_empty = row.iter().filter(|c| !c.is_empty()).count();
                    if non_empty > 0 {
                        rows.push(row.clone());
                    }
                }
                // Flush eagerly if the table has grown very large.
                if rows.len() > MAX_PENDING_ROWS {
                    commit_table(
                        headers,
                        rows,
                        spans,
                        &unit.unit_tag,
                        &unit_title,
                        &mut all_tables,
                    );
                } else {
                    pending = Some((headers, rows));
                }
            }
            // If no pending table exists, this page has unstructured text — skip.
        }
    }

    // Flush any remaining pending table after all pages have been processed.
    if let Some((headers, rows)) = pending.take() {
        if !rows.is_empty() {
            commit_table(headers, rows, &[], &unit.unit_tag, &unit_title, &mut all_tables);
        }
    }

    all_tables
}

/// Classify a table by keyword-scanning its title and header cell texts.
///
/// Returns the most specific [`SubmittalTableType`] whose keywords are present,
/// or [`SubmittalTableType::General`] if no classification matches.
pub fn classify_table(
    table_title: Option<&str>,
    headers: &[String],
) -> SubmittalTableType {
    let mut haystack = table_title.unwrap_or("").to_ascii_uppercase();
    for h in headers {
        haystack.push(' ');
        haystack.push_str(&h.to_ascii_uppercase());
    }

    if haystack.contains("CFM")
        || haystack.contains("ESP")
        || haystack.contains("RPM")
        || haystack.contains("AIRFLOW")
        || haystack.contains("STATIC PRESSURE")
    {
        SubmittalTableType::Airflow
    } else if haystack.contains("VOLT")
        || haystack.contains("AMP")
        || haystack.contains("MCA")
        || haystack.contains("MOP")
        || haystack.contains("FLA")
        || haystack.contains("ELECTRICAL")
        || haystack.contains("KW")
    {
        SubmittalTableType::Electrical
    } else if haystack.contains("WEIGHT")
        || haystack.contains("DIM")
        || haystack.contains("LENGTH")
        || haystack.contains("WIDTH")
        || haystack.contains("HEIGHT")
        || haystack.contains("PHYSICAL")
    {
        SubmittalTableType::Dimensional
    } else if haystack.contains("SOUND")
        || haystack.contains("ACOUST")
        || haystack.contains(" NC ")
        || haystack.contains("OCTAVE")
        || haystack.contains(" LW")
        || haystack.contains(" LP")
    {
        SubmittalTableType::SoundData
    } else {
        SubmittalTableType::General
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Commit a completed table entry to `out`.
///
/// No-op when `headers` or `data_rows` is empty.
fn commit_table(
    headers: Vec<String>,
    data_rows: Vec<Vec<String>>,
    spans: &[Span],
    unit_tag: &str,
    unit_title: &str,
    out: &mut Vec<ExtractedTable>,
) {
    if headers.is_empty() || data_rows.is_empty() {
        return;
    }

    let table_title = detect_table_title(spans);
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

    out.push(ExtractedTable {
        sheet_id: unit_tag.to_owned(),
        sheet_title: unit_title.to_owned(),
        table_title,
        headers,
        rows: data_rows,
        row_count,
        confidence,
    });
}

/// A row is a `Vec` of `(x, text)` pairs sorted by x.
type Row = Vec<(f64, String)>;

/// Cluster page spans into rows by y-coordinate proximity.
///
/// Relies on spans being pre-sorted top-to-bottom as guaranteed by [`Page`].
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
/// x-values.
///
/// Returns a sorted list of representative x-positions (one per column).
fn detect_column_anchors(rows: &[Row]) -> Vec<f64> {
    let mut xs: Vec<f64> = rows
        .iter()
        .flat_map(|r| r.iter().map(|(x, _)| *x))
        .collect();
    xs.sort_by(|a, b| a.total_cmp(b));

    let mut anchors: Vec<f64> = Vec::new();
    for x in xs {
        if anchors
            .last()
            .map_or(true, |&last| (x - last).abs() > COL_X_EPSILON)
        {
            anchors.push(x);
        }
    }
    anchors
}

/// Align one row's `(x, text)` pairs to the column anchor grid.
///
/// Returns a `Vec<String>` of length `col_anchors.len()`.  Multiple spans in
/// the same bucket are space-concatenated.
fn align_row_to_columns(row: &Row, col_anchors: &[f64]) -> Vec<String> {
    let mut cells = vec![String::new(); col_anchors.len()];
    for (x, text) in row {
        let col = col_anchors
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                ((*x - *a).abs()).total_cmp(&((*x - *b).abs()))
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        if cells[col].is_empty() {
            cells[col] = text.clone();
        } else {
            cells[col].push(' ');
            cells[col].push_str(text);
        }
    }
    cells
}

/// Return `true` if all non-empty cells look numeric (digits, `.`, `/`, `-`, ` `, `,`).
fn row_looks_numeric(cells: &[&str]) -> bool {
    cells.iter().all(|c| {
        c.chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '/' | '-' | ' ' | ','))
    })
}

/// Attempt to detect an optional table title from a span above the first data
/// cluster that has a relatively large font size.
fn detect_table_title(spans: &[Span]) -> Option<String> {
    if spans.is_empty() {
        return None;
    }
    let min_y = spans
        .iter()
        .map(|s| s.bbox.y)
        .fold(f64::MAX, f64::min);
    // A heading span is significantly above the first row and has a large font.
    let title_span = spans.iter().find(|s| {
        s.bbox.y < min_y - ROW_Y_EPSILON * 2.0 && s.font_size > 10.0
    });
    title_span.map(|s| s.text.clone())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{types::BBox, Page, Span};

    // ── Test fixtures ─────────────────────────────────────────────────────────

    fn make_unit(tag: &str) -> UnitEntry {
        UnitEntry {
            unit_tag: tag.to_owned(),
            model: None,
            manufacturer: None,
            item_type: Some("Air Handling Unit".to_owned()),
            start_page: 0,
            end_page: 2,
            page_count: 3,
            is_cover: false,
            confidence: 1.0,
        }
    }

    /// Build a page with a 3-column table: header row + 2 data rows.
    ///
    /// Columns: `UNIT` (x=0.10), `CFM` (x=0.40), `ESP (IN WG)` (x=0.70)
    fn make_table_page(page_idx: usize) -> Page {
        let mut page = Page::new(page_idx, 612.0, 792.0).unwrap();

        let headers = [("UNIT", 0.10), ("CFM", 0.40), ("ESP (IN WG)", 0.70)];
        for (text, x) in &headers {
            let bbox = BBox::new(*x, 0.10, 0.05, 0.02).unwrap();
            page.add_span(Span::new(text, bbox, 10.0).unwrap()).unwrap();
        }
        let row1 = [("AHU-1", 0.10), ("5000", 0.40), ("1.25", 0.70)];
        for (text, x) in &row1 {
            let bbox = BBox::new(*x, 0.20, 0.05, 0.02).unwrap();
            page.add_span(Span::new(text, bbox, 10.0).unwrap()).unwrap();
        }
        let row2 = [("AHU-2", 0.10), ("3500", 0.40), ("1.00", 0.70)];
        for (text, x) in &row2 {
            let bbox = BBox::new(*x, 0.30, 0.05, 0.02).unwrap();
            page.add_span(Span::new(text, bbox, 10.0).unwrap()).unwrap();
        }
        page
    }

    /// Build a page with only 1 span (below the 3-span minimum).
    fn make_sparse_page(page_idx: usize) -> Page {
        let mut page = Page::new(page_idx, 612.0, 792.0).unwrap();
        let bbox = BBox::new(0.10, 0.10, 0.05, 0.02).unwrap();
        page.add_span(Span::new("SPARSE", bbox, 10.0).unwrap())
            .unwrap();
        page
    }

    /// Build a page with only purely-numeric data rows (no text header).
    ///
    /// All cell values are digits/decimals so `row_looks_numeric` returns `true`
    /// for every row, which causes the algorithm to treat the page as a
    /// continuation of the prior table rather than starting a new one.
    fn make_data_only_page(page_idx: usize) -> Page {
        let mut page = Page::new(page_idx, 612.0, 792.0).unwrap();
        let rows: &[(&str, &str, &str, f64)] = &[
            ("3500", "1.50", "12.0", 0.10),
            ("2800", "1.25", "10.0", 0.20),
            ("2000", "1.00", "8.0", 0.30),
        ];
        for (t1, t2, t3, y) in rows {
            for (text, x) in &[(*t1, 0.10_f64), (*t2, 0.40_f64), (*t3, 0.70_f64)] {
                let bbox = BBox::new(*x, *y, 0.05, 0.02).unwrap();
                page.add_span(Span::new(text, bbox, 9.0).unwrap()).unwrap();
            }
        }
        page
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    /// Empty input returns empty Vec.
    #[test]
    fn empty_pages_returns_empty() {
        let unit = make_unit("AHU-1");
        assert!(extract_unit_tables(&[], &unit).is_empty());
    }

    /// A page with fewer than 3 spans returns empty Vec.
    #[test]
    fn sparse_page_returns_empty() {
        let unit = make_unit("AHU-1");
        let page = make_sparse_page(0);
        assert!(extract_unit_tables(&[&page], &unit).is_empty());
    }

    /// A well-formed 3-column table on a single page extracts correctly.
    #[test]
    fn single_page_table_extracts_correctly() {
        let unit = make_unit("AHU-1");
        let page = make_table_page(0);
        let result = extract_unit_tables(&[&page], &unit);
        assert_eq!(result.len(), 1);
        let table = &result[0];
        assert_eq!(table.headers.len(), 3);
        assert_eq!(table.row_count, 2);
        assert_eq!(table.confidence, 1.0);
    }

    /// `sheet_id` and `sheet_title` are populated from the unit metadata.
    #[test]
    fn unit_tag_and_title_propagated_to_sheet_fields() {
        let unit = make_unit("RTU-3");
        let page = make_table_page(0);
        let result = extract_unit_tables(&[&page], &unit);
        assert!(!result.is_empty());
        assert_eq!(result[0].sheet_id, "RTU-3");
        assert_eq!(result[0].sheet_title, "Air Handling Unit");
    }

    /// Two pages with the same table layout extract two independent tables.
    #[test]
    fn two_table_pages_extract_two_tables() {
        let unit = make_unit("AHU-1");
        let page0 = make_table_page(0);
        let page1 = make_table_page(1);
        let result = extract_unit_tables(&[&page0, &page1], &unit);
        // Each page has a header row, so they are treated as two independent tables.
        assert_eq!(result.len(), 2);
    }

    /// A page with only data rows (no header) immediately after a table page
    /// is merged as a continuation of the prior table.
    #[test]
    fn data_only_page_merges_into_prior_table() {
        let unit = make_unit("AHU-1");
        let header_page = make_table_page(0); // 2 data rows
        let cont_page = make_data_only_page(1); // 3 data rows (no text header)
        let result = extract_unit_tables(&[&header_page, &cont_page], &unit);
        // Both sets of data rows should land in the same table.
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].row_count, 5, "2 + 3 merged rows expected");
    }

    // ── Classification tests ──────────────────────────────────────────────────

    /// Airflow keywords in the title trigger `Airflow` classification.
    #[test]
    fn classify_cfm_title_is_airflow() {
        let t = classify_table(
            Some("AIRFLOW PERFORMANCE"),
            &["CFM".to_owned(), "ESP".to_owned()],
        );
        assert_eq!(t, SubmittalTableType::Airflow);
    }

    /// Electrical keywords in headers trigger `Electrical` classification.
    #[test]
    fn classify_electrical_headers() {
        let t = classify_table(
            None,
            &[
                "VOLTS".to_owned(),
                "MCA".to_owned(),
                "MOP".to_owned(),
            ],
        );
        assert_eq!(t, SubmittalTableType::Electrical);
    }

    /// Unrecognised keywords yield `General`.
    #[test]
    fn classify_unknown_is_general() {
        let t = classify_table(None, &["ITEM".to_owned(), "QTY".to_owned()]);
        assert_eq!(t, SubmittalTableType::General);
    }

    /// Dimensional keywords in headers trigger `Dimensional` classification.
    #[test]
    fn classify_dimensional_headers() {
        let t = classify_table(
            Some("PHYSICAL DATA"),
            &["WEIGHT".to_owned(), "LENGTH".to_owned(), "WIDTH".to_owned()],
        );
        assert_eq!(t, SubmittalTableType::Dimensional);
    }
}

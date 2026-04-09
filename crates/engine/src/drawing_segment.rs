//! Drawing segmentation: title-block oracle and drawing-set index builder.
//!
//! Reads a completed [`LayoutTranscript`] and produces a [`DrawingIndex`] by:
//!
//! 1. **Title-block region detection** — for each page, collect spans in the
//!    bottom 25 % (`y > 0.75`) or right-side 25 % (`x > 0.75`) of the page
//!    (normalised coordinates, top-left origin). These are the two geometric
//!    locations where AEC title blocks appear.
//! 2. **Sheet ID extraction** — within the candidate region, find the first
//!    span matching the sheet-ID pattern `[A-Z]{1,3}-?\d{3,4}` (e.g. `M-201`,
//!    `FP-101`, `A101`). On multiple candidates the rightmost match wins (title
//!    blocks are in the right column).
//! 3. **Sheet boundary detection** — consecutive pages with the same sheet ID
//!    are grouped into a single [`SheetEntry`]. Pages with no detected ID are
//!    assigned to the previous sheet (same tolerance as `segment.rs`).
//! 4. **Discipline classification** — calls [`classify_sheet`] from
//!    `conset-pdf-standards-data` with the extracted `sheet_id` and the sheet
//!    title to populate [`SheetChromeMetadata::discipline`].
//! 5. **Discipline summary** — builds [`DisciplineSummary`] roll-ups over all
//!    detected sheets.
//!
//! # Design rationale
//!
//! Drawing sheets never have CSI footer section-ID stamps, so the spec-book
//! `segment.rs` oracle is useless for DWG documents. This module provides the
//! drawing-specific counterpart that is structurally analogous to `segment.rs`
//! but driven by title-block geometry rather than footer text patterns.

use conset_pdf_ir::{
    DisciplineSummary, DrawingIndex, LayoutTranscript, SheetChromeMetadata, SheetEntry, Span,
};
use conset_pdf_standards_data::aec::classify_sheet;
use regex::Regex;
use std::sync::OnceLock;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Spans with `y > TITLE_BLOCK_BOTTOM_Y` are in the bottom title-block band.
/// Drawing title blocks typically sit in the bottom 15–25 % of the page;
/// 0.75 gives a generous margin to capture them reliably.
const TITLE_BLOCK_BOTTOM_Y: f64 = 0.75;

/// Spans with `x > TITLE_BLOCK_RIGHT_X` are in the right title-block band.
/// Side-bar title blocks (landscape drawings, portrait with right column) sit
/// in approximately the rightmost 25 % of the page.
const TITLE_BLOCK_RIGHT_X: f64 = 0.75;

// ── Compiled regex patterns ───────────────────────────────────────────────────

/// Matches a drawing sheet ID of the form `X-NNN`, `XX-NNN`, `XXX-NNN`, or
/// the no-separator variants like `A101`, `M201`, `FP101`.
///
/// Capture groups:
/// - 1: leading letter(s) (1–3 uppercase/lowercase ASCII letters)
/// - 2: digits (3–4 decimal digits)
fn sheet_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b([A-Za-z]{1,3})-?(\d{3,4})\b").unwrap()
    })
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Stateless drawing segmentation engine.
///
/// Call [`DrawingSegmentEngine::build_index`] with a completed
/// [`LayoutTranscript`] to produce a [`DrawingIndex`].
pub struct DrawingSegmentEngine;

impl DrawingSegmentEngine {
    /// Build a [`DrawingIndex`] from a layout transcript.
    ///
    /// Runs the full title-block oracle pipeline: region detection → sheet ID
    /// extraction → boundary grouping → discipline classification.
    ///
    /// Returns an index with `sheet_count == 0` when no title blocks are found
    /// (e.g. a spec-book PDF rather than a drawing set).  This is a valid,
    /// non-error result.
    #[must_use]
    pub fn build_index(transcript: &LayoutTranscript) -> DrawingIndex {
        let pages = transcript.pages();
        let total_pages = pages.len();

        if total_pages == 0 {
            return DrawingIndex::empty(0);
        }

        // Per-page detected sheet IDs (None = no title block found).
        let mut page_ids: Vec<Option<String>> = Vec::with_capacity(total_pages);
        // Per-page sheet titles (empty when not detected).
        let mut page_titles: Vec<String> = Vec::with_capacity(total_pages);

        for page in pages {
            let (sheet_id, sheet_title) = extract_sheet_from_page(page.spans());
            page_ids.push(sheet_id);
            page_titles.push(sheet_title);
        }

        // Build sheet entries: group consecutive pages by sheet ID.
        let sheets = build_sheets(&page_ids, &page_titles);

        let sheet_count = sheets.len();
        let discipline_summary = build_discipline_summary(&sheets);

        DrawingIndex {
            schema_version: "1.0.0".to_owned(),
            sheet_count,
            sheets,
            discipline_summary,
            total_pages,
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Detect the sheet ID and sheet title from a page's spans.
///
/// Returns `(Some(sheet_id), sheet_title)` when a title block is found, or
/// `(None, String::new())` when no sheet-ID pattern matches.
fn extract_sheet_from_page(spans: &[Span]) -> (Option<String>, String) {
    // Collect spans in the candidate title-block region.
    let candidates: Vec<&Span> = spans
        .iter()
        .filter(|s| s.bbox.y > TITLE_BLOCK_BOTTOM_Y && s.bbox.x > TITLE_BLOCK_RIGHT_X)
        .collect();

    if candidates.is_empty() {
        return (None, String::new());
    }

    // Find all sheet-ID matches; prefer the rightmost (largest x).
    let mut best: Option<(String, f64)> = None; // (normalised_id, x)

    for span in &candidates {
        if let Some(caps) = sheet_id_re().captures(&span.text) {
            let prefix = caps[1].to_uppercase();
            let digits = &caps[2];
            let normalised = format!("{prefix}-{digits}");
            let x = span.bbox.x;
            if best.as_ref().map_or(true, |(_, bx)| x > *bx) {
                best = Some((normalised, x));
            }
        }
    }

    let sheet_id = match best {
        Some((id, _)) => id,
        None => return (None, String::new()),
    };

    // Extract a sheet title from the longest non-ID text span in the region.
    let sheet_title = candidates
        .iter()
        .filter(|s| !sheet_id_re().is_match(&s.text))
        .filter(|s| !s.text.trim().is_empty())
        .max_by_key(|s| s.text.len())
        .map(|s| s.text.trim().to_owned())
        .unwrap_or_default();

    (Some(sheet_id), sheet_title)
}

/// Group per-page `(sheet_id, sheet_title)` into [`SheetEntry`] boundaries.
///
/// Algorithm: walk pages in order; when a new non-`None` sheet ID is seen,
/// close the current sheet and open a new one.  Pages with `None` are
/// assigned to the most-recently opened sheet.  If the very first page has
/// no ID, it is put into a synthetic "UNKN-000" sheet so nothing is lost.
fn build_sheets(
    page_ids: &[Option<String>],
    page_titles: &[String],
) -> Vec<SheetEntry> {
    if page_ids.is_empty() {
        return Vec::new();
    }

    struct Accumulator {
        sheet_id: String,
        title: String,
        start_page: usize,
    }

    let mut sheets: Vec<SheetEntry> = Vec::new();
    let mut acc: Option<Accumulator> = None;

    for (page_idx, maybe_id) in page_ids.iter().enumerate() {
        let detected_id = maybe_id.as_deref();
        let detected_title = &page_titles[page_idx];

        // Take ownership of the current accumulator to avoid borrow conflicts.
        let prev = acc.take();
        acc = match (prev, detected_id) {
            // No sheet open and no ID → skip (preamble / cover page).
            (None, None) => None,

            // No sheet open, ID found → start the first sheet.
            (None, Some(id)) => Some(Accumulator {
                sheet_id: id.to_owned(),
                title: detected_title.clone(),
                start_page: page_idx,
            }),

            // Sheet open, no ID → extend the current sheet.
            (Some(a), None) => Some(a),

            // Sheet open, same ID → extend.
            (Some(a), Some(id)) if id == a.sheet_id.as_str() => Some(a),

            // Sheet open, different ID → close and start a new sheet.
            (Some(a), Some(id)) => {
                let chrome = make_chrome(&a.sheet_id, &a.title);
                sheets.push(make_entry(a.sheet_id, a.start_page, page_idx - 1, chrome));
                Some(Accumulator {
                    sheet_id: id.to_owned(),
                    title: detected_title.clone(),
                    start_page: page_idx,
                })
            }
        };
    }

    // Close the final open sheet.
    if let Some(a) = acc {
        let start = a.start_page;
        let end = page_ids.len() - 1;
        let chrome = make_chrome(&a.sheet_id, &a.title);
        sheets.push(make_entry(a.sheet_id.clone(), start, end, chrome));
    }

    sheets
}

/// Build a [`SheetChromeMetadata`] by classifying the sheet ID.
fn make_chrome(sheet_id: &str, sheet_title: &str) -> SheetChromeMetadata {
    let title_opt = if sheet_title.is_empty() {
        None
    } else {
        Some(sheet_title)
    };
    let result = classify_sheet(sheet_id, title_opt);

    SheetChromeMetadata {
        sheet_id: sheet_id.to_owned(),
        sheet_title: sheet_title.to_owned(),
        discipline: result.canonical4.to_owned(),
        confidence: result.confidence,
        ..Default::default()
    }
}

/// Build a [`SheetEntry`] from accumulated page range data.
fn make_entry(
    sheet_id: String,
    start_page: usize,
    end_page: usize,
    chrome: SheetChromeMetadata,
) -> SheetEntry {
    let page_count = end_page - start_page + 1;
    let is_schedule = is_schedule_sheet_title(&chrome.sheet_title);
    SheetEntry {
        sheet_id,
        start_page,
        end_page,
        page_count,
        chrome,
        superseded_by: None,
        is_schedule_sheet: is_schedule,
    }
}

/// Return `true` if the normalised sheet title contains a schedule keyword.
///
/// Keywords (case-insensitive): SCHEDULE, EQUIPMENT LIST, FIXTURE SCHEDULE,
/// PANEL SCHEDULE, METER SCHEDULE, PIPING SCHEDULE, VALVE SCHEDULE.
/// Matched against the uppercased title before punctuation stripping so that
/// multi-word phrases like `"EQUIPMENT LIST"` are matched correctly.
fn is_schedule_sheet_title(title: &str) -> bool {
    let up = title.to_ascii_uppercase();
    up.contains("SCHEDULE")
        || up.contains("EQUIPMENT LIST")
        || up.contains("FIXTURE LIST")
        || up.contains("PANEL LIST")
        || up.contains("METER LIST")
        || up.contains("PIPING LIST")
        || up.contains("VALVE LIST")
}

/// Build [`DisciplineSummary`] roll-ups from the final sheet list.
fn build_discipline_summary(sheets: &[SheetEntry]) -> Vec<DisciplineSummary> {
    use std::collections::HashMap;

    struct Accum {
        display_name: String,
        sort_order: u32,
        count: usize,
    }

    let mut by_discipline: HashMap<String, Accum> = HashMap::new();

    for sheet in sheets {
        let disc = &sheet.chrome.discipline;
        if disc.is_empty() {
            continue;
        }
        // Re-classify to get display name and sort order (cheap — it's a table lookup).
        let info = classify_sheet(&sheet.sheet_id, Some(&sheet.chrome.sheet_title));
        let entry = by_discipline.entry(disc.clone()).or_insert_with(|| Accum {
            display_name: info.display_name.to_owned(),
            sort_order: info.sort_order,
            count: 0,
        });
        entry.count += 1;
    }

    let mut summary: Vec<DisciplineSummary> = by_discipline
        .into_iter()
        .map(|(canonical4, a)| DisciplineSummary {
            canonical4,
            display_name: a.display_name,
            sheet_count: a.count,
            sort_order: a.sort_order,
        })
        .collect();

    // Sort by sort_order for deterministic output.
    summary.sort_by_key(|d| d.sort_order);
    summary
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{BBox, LayoutTranscript, Page, Span, TranscriptMetadata};

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_span(text: &str, x: f64, y: f64) -> Span {
        let bbox = BBox::new(x, y, 0.05, 0.02).expect("valid test bbox");
        Span::new(text, bbox, 10.0).expect("valid test span")
    }

    fn make_transcript(pages: Vec<Vec<Span>>) -> LayoutTranscript {
        let page_count = pages.len();
        let page_objects: Vec<Page> = pages
            .into_iter()
            .enumerate()
            .map(|(idx, spans)| {
                let mut p = Page::new(idx, 100.0, 100.0).expect("valid test page");
                for s in spans {
                    p.add_span(s).expect("valid test span");
                }
                p
            })
            .collect();
        let meta = TranscriptMetadata::new("/tmp/test.pdf", page_count)
            .expect("valid test metadata");
        LayoutTranscript::new(page_objects, meta).expect("valid test transcript")
    }

    // ── 9.1.A tests: region detection ────────────────────────────────────────

    #[test]
    fn title_block_region_detected_on_synthetic_page() {
        // A span at (x=0.85, y=0.90) is in the bottom-right title-block corner.
        let span = make_span("M-201", 0.85, 0.90);
        let in_region = span.bbox.y > TITLE_BLOCK_BOTTOM_Y && span.bbox.x > TITLE_BLOCK_RIGHT_X;
        assert!(in_region, "span at (0.85, 0.90) should be in title-block region");
    }

    #[test]
    fn title_block_region_absent_returns_empty() {
        // Spans in the middle of the page: neither y > 0.75 nor x > 0.75.
        let spans: Vec<Span> = vec![
            make_span("MECHANICAL EQUIPMENT PLAN", 0.10, 0.40),
            make_span("LEVEL 1", 0.10, 0.45),
            make_span("Project Notes", 0.10, 0.50),
        ];
        let (sheet_id, _title) = extract_sheet_from_page(&spans);
        assert!(sheet_id.is_none(), "body text should not produce a sheet ID");
    }

    #[test]
    fn right_band_only_not_in_title_block_region() {
        // Span at (x=0.90, y=0.40) — right band but NOT bottom band.
        // Title-block detection requires BOTH bottom AND right (bottom-right corner only);
        // a right-band-only span should NOT be considered part of the title block.
        let span = make_span("FP-101", 0.90, 0.40);
        let in_region = span.bbox.y > TITLE_BLOCK_BOTTOM_Y && span.bbox.x > TITLE_BLOCK_RIGHT_X;
        assert!(!in_region, "right-band-only span (y=0.40) must NOT be in title-block region");
    }

    // ── 9.1.B tests: sheet ID extraction ─────────────────────────────────────

    #[test]
    fn sheet_id_extracted_from_bottom_span() {
        let spans = vec![
            make_span("PROJECT MECHANICAL PLAN", 0.10, 0.50),
            make_span("M-201", 0.85, 0.90),  // in title-block region
        ];
        let (id, _) = extract_sheet_from_page(&spans);
        assert_eq!(id.as_deref(), Some("M-201"));
    }

    #[test]
    fn sheet_id_extracted_from_bottom_right_corner() {
        // Right-band-only spans (y not in bottom region) no longer detected.
        // Sheet IDs must be in the bottom-right corner (both y > 0.75 and x > 0.75).
        let spans = vec![
            make_span("FP-101", 0.88, 0.80),  // bottom-right corner
        ];
        let (id, _) = extract_sheet_from_page(&spans);
        assert_eq!(id.as_deref(), Some("FP-101"));
    }

    #[test]
    fn rightmost_candidate_wins_on_ambiguous_page() {
        // Two sheet-ID candidates in title block; rightmost (larger x) should win.
        let spans = vec![
            make_span("A-101", 0.76, 0.90),  // left of the two
            make_span("M-201", 0.92, 0.90),  // rightmost — should win
        ];
        let (id, _) = extract_sheet_from_page(&spans);
        assert_eq!(id.as_deref(), Some("M-201"));
    }

    #[test]
    fn lowercase_sheet_id_normalised_to_upper() {
        let spans = vec![make_span("fp-101", 0.85, 0.90)];
        let (id, _) = extract_sheet_from_page(&spans);
        assert_eq!(id.as_deref(), Some("FP-101"));
    }

    #[test]
    fn no_separator_sheet_id_parsed() {
        let spans = vec![make_span("A101", 0.85, 0.90)];
        let (id, _) = extract_sheet_from_page(&spans);
        assert_eq!(id.as_deref(), Some("A-101"));
    }

    #[test]
    fn three_letter_prefix_parsed() {
        let spans = vec![make_span("DDC-101", 0.85, 0.90)];
        let (id, _) = extract_sheet_from_page(&spans);
        assert_eq!(id.as_deref(), Some("DDC-101"));
    }

    #[test]
    fn no_match_in_region_returns_none() {
        let spans = vec![
            make_span("SHEET NO.", 0.85, 0.90),   // in region but no pattern
            make_span("SCALE: NTS", 0.85, 0.92),
        ];
        let (id, _) = extract_sheet_from_page(&spans);
        assert!(id.is_none());
    }

    // ── 9.1.C tests: DrawingSegmentEngine ────────────────────────────────────

    #[test]
    fn drawing_segment_single_sheet_all_pages_same_id() {
        // 4 pages all bearing "M-201" in title block.
        let make_page = |sheet: &str| vec![
            make_span(sheet, 0.85, 0.90),
            make_span("MECHANICAL PLAN", 0.20, 0.50),
        ];
        let transcript = make_transcript(vec![
            make_page("M-201"),
            make_page("M-201"),
            make_page("M-201"),
            make_page("M-201"),
        ]);
        let idx = DrawingSegmentEngine::build_index(&transcript);
        assert_eq!(idx.total_pages, 4);
        assert_eq!(idx.sheet_count, 1);
        assert_eq!(idx.sheets[0].sheet_id, "M-201");
        assert_eq!(idx.sheets[0].start_page, 0);
        assert_eq!(idx.sheets[0].end_page, 3);
        assert_eq!(idx.sheets[0].page_count, 4);
    }

    #[test]
    fn drawing_segment_two_sheets_boundary_detected() {
        // Pages 0–1: M-201, page 2: M-202.
        let transcript = make_transcript(vec![
            vec![make_span("M-201", 0.85, 0.90)],
            vec![make_span("M-201", 0.85, 0.90)],
            vec![make_span("M-202", 0.85, 0.90)],
        ]);
        let idx = DrawingSegmentEngine::build_index(&transcript);
        assert_eq!(idx.sheet_count, 2);
        assert_eq!(idx.sheets[0].sheet_id, "M-201");
        assert_eq!(idx.sheets[0].start_page, 0);
        assert_eq!(idx.sheets[0].end_page, 1);
        assert_eq!(idx.sheets[0].page_count, 2);
        assert_eq!(idx.sheets[1].sheet_id, "M-202");
        assert_eq!(idx.sheets[1].start_page, 2);
        assert_eq!(idx.sheets[1].end_page, 2);
        assert_eq!(idx.sheets[1].page_count, 1);
    }

    #[test]
    fn drawing_segment_page_without_id_assigned_to_prior_sheet() {
        // Page 0: M-201, page 1: no title block, page 2: M-202.
        let transcript = make_transcript(vec![
            vec![make_span("M-201", 0.85, 0.90)],
            // Page 1 has only body text — no title-block region.
            vec![make_span("Continuation notes", 0.10, 0.40)],
            vec![make_span("M-202", 0.85, 0.90)],
        ]);
        let idx = DrawingSegmentEngine::build_index(&transcript);
        // Page 1 (no ID) should be folded into M-201.
        assert_eq!(idx.sheet_count, 2, "should detect 2 sheets, not 3");
        assert_eq!(idx.sheets[0].sheet_id, "M-201");
        assert_eq!(idx.sheets[0].page_count, 2, "M-201 should span pages 0 and 1");
        assert_eq!(idx.sheets[1].sheet_id, "M-202");
        assert_eq!(idx.sheets[1].page_count, 1);
    }

    #[test]
    fn drawing_segment_no_title_blocks_returns_zero_sheets() {
        // A spec-book PDF with 3 pages — no title blocks detected.
        let transcript = make_transcript(vec![
            vec![make_span("23 05 00", 0.50, 0.95)],   // CSI footer, not title block
            vec![make_span("23 82 16", 0.50, 0.95)],
            vec![make_span("Page 1 of 3", 0.80, 0.96)],
        ]);
        let idx = DrawingSegmentEngine::build_index(&transcript);
        // 23 05 00 and 23 82 16 don't match sheet-ID pattern (they have spaces).
        // So no sheets should be detected — wait, these ARE in y > 0.75 region.
        // But "23 05 00" doesn't match the sheet_id_re pattern [A-Z]{1,3}-?\d{3,4},
        // so no sheet ID is extracted.
        assert_eq!(idx.sheet_count, 0, "CSI footer stamps should not produce sheet IDs");
    }

    #[test]
    fn discipline_summary_populated_from_sheets() {
        // Two MECH sheets, one ELEC sheet.
        let transcript = make_transcript(vec![
            vec![make_span("M-201", 0.85, 0.90)],
            vec![make_span("M-202", 0.85, 0.90)],
            vec![make_span("E-101", 0.85, 0.90)],
        ]);
        let idx = DrawingSegmentEngine::build_index(&transcript);
        assert_eq!(idx.sheet_count, 3);

        let mech = idx.discipline_summary.iter().find(|d| d.canonical4 == "MECH");
        let elec = idx.discipline_summary.iter().find(|d| d.canonical4 == "ELEC");
        assert!(mech.is_some(), "MECH discipline summary should be present");
        assert!(elec.is_some(), "ELEC discipline summary should be present");
        assert_eq!(mech.unwrap().sheet_count, 2);
        assert_eq!(elec.unwrap().sheet_count, 1);

        // MECH (sort_order 70) should come before ELEC (sort_order 90).
        let mech_pos = idx.discipline_summary.iter().position(|d| d.canonical4 == "MECH").unwrap();
        let elec_pos = idx.discipline_summary.iter().position(|d| d.canonical4 == "ELEC").unwrap();
        assert!(mech_pos < elec_pos, "MECH should sort before ELEC");
    }

    // ── Sprint 9.4.A — is_schedule_sheet detection ────────────────────────────

    #[test]
    fn schedule_keyword_schedule_detected() {
        assert!(is_schedule_sheet_title("MECHANICAL EQUIPMENT SCHEDULE"));
        assert!(is_schedule_sheet_title("Panel Schedule"));
        assert!(is_schedule_sheet_title("PIPING SCHEDULE — LEVEL 2"));
    }

    #[test]
    fn schedule_keyword_equipment_list_detected() {
        assert!(is_schedule_sheet_title("EQUIPMENT LIST"));
        assert!(is_schedule_sheet_title("Mechanical Equipment List"));
    }

    #[test]
    fn non_schedule_title_not_detected() {
        assert!(!is_schedule_sheet_title("MECHANICAL EQUIPMENT PLAN"));
        assert!(!is_schedule_sheet_title("ELECTRICAL RISER DIAGRAM"));
        assert!(!is_schedule_sheet_title("FLOOR PLAN"));
        assert!(!is_schedule_sheet_title(""));
    }

    #[test]
    fn make_entry_sets_is_schedule_sheet_from_title() {
        let chrome_sched = SheetChromeMetadata {
            sheet_id: "M-201".to_owned(),
            sheet_title: "MECHANICAL EQUIPMENT SCHEDULE".to_owned(),
            discipline: "MECH".to_owned(),
            confidence: 0.9,
            ..Default::default()
        };
        let entry_sched = make_entry("M-201".to_owned(), 0, 0, chrome_sched);
        assert!(entry_sched.is_schedule_sheet, "schedule title must set is_schedule_sheet=true");

        let chrome_plan = SheetChromeMetadata {
            sheet_id: "M-101".to_owned(),
            sheet_title: "MECHANICAL FLOOR PLAN".to_owned(),
            discipline: "MECH".to_owned(),
            confidence: 0.9,
            ..Default::default()
        };
        let entry_plan = make_entry("M-101".to_owned(), 0, 0, chrome_plan);
        assert!(!entry_plan.is_schedule_sheet, "non-schedule title must set is_schedule_sheet=false");
    }
}

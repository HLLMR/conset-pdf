//! Submittal segmentation: unit-boundary oracle and submittal index builder.
//!
//! Reads a completed [`LayoutTranscript`] and produces a [`SubmittalIndex`] by:
//!
//! 1. **Prominent span extraction** — for each page, collect spans whose
//!    `font_size` is at least `UNIT_HEADER_MIN_FONT_RATIO` × the corpus-median
//!    font size AND that appear in the upper half of the page (`y < 0.5`).
//!    These are likely unit header text elements.
//!
//! 2. **Unit tag detection** — within the prominent spans, find the first span
//!    matching the unit-tag pattern `[A-Z]{1,4}-?\d{1,4}[A-Z]?` (e.g. `AHU-1`,
//!    `RTU-3`, `FCU-2A`).
//!
//! 3. **Cover page detection** — leading pages (up to `COVER_PAGE_MAX`) with
//!    no unit tag detected are marked as a synthetic cover entry
//!    (`is_cover = true`).
//!
//! 4. **Unit boundary detection** — a change in detected tag (or a new non-None
//!    tag after a None run past cover-page range) triggers a boundary.
//!    Pages with no detected tag are folded into the most recently opened unit.
//!
//! 5. **Fallback** — when no unit tags are detected on any page, a single unit
//!    spanning all pages is emitted with `confidence = 0.3`.
//!
//! # Design rationale
//!
//! Equipment submittals use unit tags (e.g. "AHU-1", "RTU-3") as section
//! markers rather than CSI section IDs.  The tag typically appears in a large
//! font on a header page at the start of each unit's section.  This module
//! provides the submittal-specific counterpart of `drawing_segment.rs`.

use crate::submittal_kv::extract_unit_header;
use conset_pdf_ir::{LayoutTranscript, Span, SubmittalCoverage, SubmittalIndex, UnitEntry};
use regex::Regex;
use std::sync::OnceLock;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Pages with index < COVER_PAGE_MAX that carry no unit tag are eligible to be
/// grouped into a synthetic cover entry.
const COVER_PAGE_MAX: usize = 3;

/// A span's font_size must be at least this multiple of the corpus median to be
/// considered a unit-header candidate.
const UNIT_HEADER_MIN_FONT_RATIO: f64 = 1.3;

/// Confidence assigned to units detected via prominent span matching.
const CONFIDENCE_HIGH: f64 = 0.9;

/// Confidence assigned to units detected via body-text tag matching (fallback
/// within the detection pass, not the total-fallback branch).
const CONFIDENCE_LOW: f64 = 0.7;

/// Confidence assigned when no unit tags were detected and a single fallback
/// unit is emitted for the entire document.
const CONFIDENCE_FALLBACK: f64 = 0.3;

// ── Compiled regex patterns ───────────────────────────────────────────────────

/// Matches a submittal unit tag such as `AHU-1`, `RTU-3`, `FCU-2A`, `VAV-10`.
///
/// Pattern: 1–4 uppercase letters, optional hyphen, 1–4 digits, optional
/// trailing letter (e.g. "2A").
///
/// Does **not** match plain numbers or date fragments.
fn unit_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b([A-Z]{1,4})-?(\d{1,4}[A-Z]?)\b").unwrap()
    })
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Stateless submittal segmentation engine.
///
/// Call [`SubmittalSegmentEngine::build_index`] with a completed
/// [`LayoutTranscript`] and the packet name to produce a [`SubmittalIndex`].
pub struct SubmittalSegmentEngine;

impl SubmittalSegmentEngine {
    /// Build a [`SubmittalIndex`] from a layout transcript.
    ///
    /// Runs the full unit-boundary oracle pipeline:
    /// prominent-span collection → unit tag detection → cover detection →
    /// boundary grouping → coverage computation.
    ///
    /// Returns a one-unit fallback index with `confidence = 0.3` when no tags
    /// are detected on any page.  This is a valid, non-error result (e.g. for
    /// raster-only or low-text submittals).
    #[must_use]
    pub fn build_index(transcript: &LayoutTranscript, packet_name: &str) -> SubmittalIndex {
        let pages = transcript.pages();
        let total_pages = pages.len();

        if total_pages == 0 {
            return SubmittalIndex {
                schema_version: "1.0.0".to_owned(),
                packet_name: packet_name.to_owned(),
                units: vec![],
                coverage: SubmittalCoverage {
                    total_pages: 0,
                    assigned_pages: 0,
                    unassigned_pages: 0,
                    coverage_ratio: 0.0,
                    unit_count: 0,
                },
            };
        }

        // Compute corpus-median font size for prominence gating.
        let median_font = compute_median_font_size(transcript);

        // Per-page detected unit tags and per-page confidence signals.
        let mut page_tags: Vec<Option<(String, f64)>> = Vec::with_capacity(total_pages);

        for page in pages {
            let tag = detect_unit_tag_on_page(page.spans(), median_font);
            page_tags.push(tag);
        }

        // Build unit entries from per-page tag detections.
        let mut units = build_units(&page_tags, total_pages);

        // Sprint 10.2: enrich each non-cover unit with header field extraction.
        for unit in &mut units {
            if unit.is_cover {
                continue;
            }
            let start = unit.start_page;
            let end = unit.end_page.min(total_pages.saturating_sub(1));
            let unit_pages: Vec<&_> = pages[start..=end].iter().collect();
            let header = extract_unit_header(&unit_pages, 2);
            if unit.model.is_none() {
                unit.model = header.model;
            }
            if unit.manufacturer.is_none() {
                unit.manufacturer = header.manufacturer;
            }
            if unit.item_type.is_none() {
                unit.item_type = header.item_type;
            }
        }

        let assigned_pages: usize = units.iter().map(|u| u.page_count).sum();
        let unassigned_pages = total_pages.saturating_sub(assigned_pages);
        let coverage_ratio = if total_pages > 0 {
            assigned_pages as f64 / total_pages as f64
        } else {
            0.0
        };
        let unit_count = units.iter().filter(|u| !u.is_cover).count();

        let coverage = SubmittalCoverage {
            total_pages,
            assigned_pages,
            unassigned_pages,
            coverage_ratio,
            unit_count,
        };

        SubmittalIndex {
            schema_version: "1.0.0".to_owned(),
            packet_name: packet_name.to_owned(),
            units,
            coverage,
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Compute the median `font_size` across all spans in the transcript.
///
/// Returns a reasonable default (12.0 pt) when the transcript is empty.
fn compute_median_font_size(transcript: &LayoutTranscript) -> f64 {
    let mut sizes: Vec<f64> = transcript
        .pages()
        .iter()
        .flat_map(|p| p.spans().iter())
        .map(|s| s.font_size)
        .collect();

    if sizes.is_empty() {
        return 12.0;
    }

    sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sizes.len() / 2;
    if sizes.len() % 2 == 0 {
        (sizes[mid - 1] + sizes[mid]) / 2.0
    } else {
        sizes[mid]
    }
}

/// Detect the best unit tag match on a single page.
///
/// Returns `Some((tag, confidence))` when a match is found, `None` otherwise.
///
/// Strategy:
/// 1. Look in prominent spans (font_size ≥ median × UNIT_HEADER_MIN_FONT_RATIO)
///    that are in the upper half of the page (y < 0.5).  → confidence HIGH.
/// 2. Fall back to prominent spans anywhere on the page.  → confidence LOW.
/// 3. No match → None.
fn detect_unit_tag_on_page(spans: &[Span], median_font: f64) -> Option<(String, f64)> {
    let threshold = median_font * UNIT_HEADER_MIN_FONT_RATIO;

    // Pass 1: upper-half prominent spans.
    for span in spans {
        if span.font_size >= threshold && span.bbox.y < 0.5 {
            if let Some(tag) = extract_tag(&span.text) {
                return Some((tag, CONFIDENCE_HIGH));
            }
        }
    }

    // Pass 2: prominent spans anywhere.
    for span in spans {
        if span.font_size >= threshold {
            if let Some(tag) = extract_tag(&span.text) {
                return Some((tag, CONFIDENCE_LOW));
            }
        }
    }

    None
}

/// Extract the canonical unit tag string from a text value.
///
/// Returns the first match normalised to `LETTERS-DIGITS` form (e.g. `"ahu-1"`
/// → `"AHU-1"`).
fn extract_tag(text: &str) -> Option<String> {
    unit_tag_re().captures(text).map(|caps| {
        let letters = caps[1].to_ascii_uppercase();
        let digits = caps[2].to_ascii_uppercase();
        format!("{letters}-{digits}")
    })
}

/// Internal accumulator for page-grouping during unit boundary detection.
struct Accumulator {
    unit_tag: String,
    start_page: usize,
    confidence_sum: f64,
    tagged_page_count: usize,
    is_cover: bool,
}

/// Convert an [`Accumulator`] into a final [`UnitEntry`].
fn finalise(a: Accumulator, end_page: usize) -> UnitEntry {
    let page_count = end_page - a.start_page + 1;
    let confidence = if a.is_cover || a.tagged_page_count == 0 {
        // Cover entries don't carry an extraction-quality confidence.
        0.5
    } else {
        // Average detected confidence across pages that had a tag match.
        (a.confidence_sum / a.tagged_page_count as f64).clamp(0.0, 1.0)
    };
    UnitEntry {
        unit_tag: a.unit_tag,
        model: None,
        manufacturer: None,
        item_type: None,
        start_page: a.start_page,
        end_page,
        page_count,
        is_cover: a.is_cover,
        confidence,
    }
}

/// Build [`UnitEntry`] slices from per-page tag detections.
///
/// Rules:
/// - Leading pages (index < COVER_PAGE_MAX) with no detection → synthetic
///   cover entry (`is_cover = true`, `unit_tag = "COVER"`).
/// - When a new, different tag is seen → close current unit, open new one.
/// - Pages with no tag after cover range → fold into the most recently opened
///   unit.
/// - Total fallback: if no tags at all → single unit spanning all pages with
///   `confidence = CONFIDENCE_FALLBACK`.
fn build_units(page_tags: &[Option<(String, f64)>], total_pages: usize) -> Vec<UnitEntry> {
    if total_pages == 0 {
        return Vec::new();
    }

    // Check for total fallback (no tags anywhere).
    let any_tag = page_tags.iter().any(|t| t.is_some());
    if !any_tag {
        return vec![UnitEntry {
            unit_tag: "UNIT-1".to_owned(),
            model: None,
            manufacturer: None,
            item_type: None,
            start_page: 0,
            end_page: total_pages - 1,
            page_count: total_pages,
            is_cover: false,
            confidence: CONFIDENCE_FALLBACK,
        }];
    }

    let mut units: Vec<UnitEntry> = Vec::new();
    let mut acc: Option<Accumulator> = None;

    for (page_idx, maybe_tag) in page_tags.iter().enumerate() {
        match (acc.take(), maybe_tag) {
            // Nothing open, no tag, within cover range → start cover entry.
            (None, None) if page_idx < COVER_PAGE_MAX => {
                acc = Some(Accumulator {
                    unit_tag: "COVER".to_owned(),
                    start_page: page_idx,
                    confidence_sum: 0.0,
                    tagged_page_count: 0,
                    is_cover: true,
                });
            }

            // Nothing open, no tag, outside cover range → no-op.
            (None, None) => {}

            // Nothing open, tag found → start new unit entry.
            (None, Some((tag, conf))) => {
                acc = Some(Accumulator {
                    unit_tag: tag.clone(),
                    start_page: page_idx,
                    confidence_sum: *conf,
                    tagged_page_count: 1,
                    is_cover: false,
                });
            }

            // Cover entry open, no tag → extend cover.
            (Some(a), None) if a.is_cover => {
                acc = Some(a);
            }

            // Cover entry open, tag found → close cover entry, start new unit.
            (Some(a), Some((tag, conf))) if a.is_cover => {
                let end_page = page_idx - 1;
                units.push(finalise(a, end_page));
                acc = Some(Accumulator {
                    unit_tag: tag.clone(),
                    start_page: page_idx,
                    confidence_sum: *conf,
                    tagged_page_count: 1,
                    is_cover: false,
                });
            }

            // Unit entry open, no new tag → extend current unit.
            (Some(a), None) => {
                acc = Some(a);
            }

            // Unit entry open, same tag → extend, accumulate confidence signal.
            (Some(mut a), Some((tag, conf))) if tag.as_str() == a.unit_tag.as_str() => {
                a.confidence_sum += *conf;
                a.tagged_page_count += 1;
                acc = Some(a);
            }

            // Unit entry open, different tag → close current unit, start new.
            (Some(a), Some((tag, conf))) => {
                let end_page = page_idx - 1;
                units.push(finalise(a, end_page));
                acc = Some(Accumulator {
                    unit_tag: tag.clone(),
                    start_page: page_idx,
                    confidence_sum: *conf,
                    tagged_page_count: 1,
                    is_cover: false,
                });
            }
        }
    }

    // Close the final open entry.
    if let Some(a) = acc {
        let end_page = total_pages - 1;
        units.push(finalise(a, end_page));
    }

    units
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{BBox, LayoutTranscript, Page, Span, TranscriptMetadata};

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_span_with_font(text: &str, x: f64, y: f64, font_size: f64) -> Span {
        let bbox = BBox::new(x, y, 0.05, 0.02).expect("valid test bbox");
        Span::new(text, bbox, font_size).expect("valid test span")
    }

    fn make_span(text: &str, x: f64, y: f64) -> Span {
        make_span_with_font(text, x, y, 10.0)
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

    // ── Tag extraction tests ──────────────────────────────────────────────────

    #[test]
    fn extract_tag_parses_ahu_tag() {
        assert_eq!(extract_tag("AHU-1"), Some("AHU-1".to_owned()));
    }

    #[test]
    fn extract_tag_parses_rtu_tag() {
        assert_eq!(extract_tag("RTU-3"), Some("RTU-3".to_owned()));
    }

    #[test]
    fn extract_tag_normalises_lowercase() {
        assert_eq!(extract_tag("ahu-1"), Some("AHU-1".to_owned()));
    }

    #[test]
    fn extract_tag_parses_trailing_letter() {
        assert_eq!(extract_tag("FCU-2A"), Some("FCU-2A".to_owned()));
    }

    #[test]
    fn extract_tag_parses_no_separator() {
        assert_eq!(extract_tag("RTU3"), Some("RTU-3".to_owned()));
    }

    #[test]
    fn extract_tag_rejects_plain_number() {
        // Plain numbers should not match the unit-tag pattern.
        assert_eq!(extract_tag("12"), None);
    }

    #[test]
    fn extract_tag_rejects_body_text_without_tag() {
        assert_eq!(extract_tag("Submit for approval prior to purchase"), None);
    }

    // ── Detection tests ───────────────────────────────────────────────────────

    #[test]
    fn prominent_upper_span_detected_with_high_confidence() {
        // Median font is 10.0; span at 16.0 (> 1.3 × 10.0) in upper half.
        let spans = vec![
            make_span("Project Notes", 0.10, 0.60),  // body text, lower half
            make_span_with_font("AHU-1", 0.40, 0.20, 16.0),  // prominent, upper
        ];
        let result = detect_unit_tag_on_page(&spans, 10.0);
        assert!(result.is_some());
        let (tag, conf) = result.unwrap();
        assert_eq!(tag, "AHU-1");
        assert!((conf - CONFIDENCE_HIGH).abs() < 1e-9);
    }

    #[test]
    fn non_prominent_span_not_detected_in_pass1() {
        // Font size 10.0 = median; 10.0 < 1.3 × 10.0 = 13.0 → not prominent.
        let spans = vec![
            make_span_with_font("AHU-1", 0.40, 0.20, 10.0),
        ];
        // With median=10.0, threshold=13.0; span at 10.0 is below threshold.
        let result = detect_unit_tag_on_page(&spans, 10.0);
        assert!(result.is_none());
    }

    #[test]
    fn no_tag_in_spans_returns_none() {
        let spans = vec![
            make_span("VFD drive specifications", 0.10, 0.20),
            make_span("Airflow: 4000 CFM", 0.10, 0.30),
        ];
        let result = detect_unit_tag_on_page(&spans, 10.0);
        assert!(result.is_none());
    }

    // ── Build index tests ─────────────────────────────────────────────────────

    #[test]
    fn empty_transcript_returns_empty_index() {
        // LayoutTranscript requires at least 1 page, so test the 1-page no-tag scenario
        // which exercises the zero-unit detection path.
        let transcript = make_transcript(vec![
            vec![make_span("Cover Sheet", 0.10, 0.30)],
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "TEST_PKT");
        // With no tags, we get the fallback single unit.
        assert_eq!(idx.coverage.total_pages, 1);
        assert_eq!(idx.units.len(), 1);
        assert_eq!(idx.units[0].confidence, CONFIDENCE_FALLBACK);
    }

    #[test]
    fn no_tags_emits_fallback_single_unit() {
        // 4 pages, no detectable tags → single fallback unit.
        let transcript = make_transcript(vec![
            vec![make_span("Cover Sheet", 0.10, 0.30)],
            vec![make_span("Specification data", 0.10, 0.40)],
            vec![make_span("Performance table", 0.10, 0.50)],
            vec![make_span("Dimensional drawing", 0.10, 0.60)],
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "TEST_PKT");
        assert_eq!(idx.units.len(), 1);
        assert_eq!(idx.units[0].unit_tag, "UNIT-1");
        assert_eq!(idx.units[0].start_page, 0);
        assert_eq!(idx.units[0].end_page, 3);
        assert!((idx.units[0].confidence - CONFIDENCE_FALLBACK).abs() < 1e-9);
    }

    #[test]
    fn single_unit_all_pages_tagged() {
        // 3 pages all bearing "AHU-1" in a large-font upper span.
        let make_page = |tag: &str| vec![
            make_span_with_font(tag, 0.40, 0.15, 20.0),  // prominent, upper
            make_span("Airflow: 4000 CFM", 0.10, 0.50),
        ];
        let transcript = make_transcript(vec![
            make_page("AHU-1"),
            make_page("AHU-1"),
            make_page("AHU-1"),
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "TEST_PKT");
        // Should produce one non-cover unit.
        let real_units: Vec<_> = idx.units.iter().filter(|u| !u.is_cover).collect();
        assert_eq!(real_units.len(), 1);
        assert_eq!(real_units[0].unit_tag, "AHU-1");
        assert_eq!(real_units[0].start_page, 0);
        assert_eq!(real_units[0].end_page, 2);
    }

    #[test]
    fn two_units_boundary_detected() {
        // Pages 0–1: AHU-1, page 2: AHU-2.
        // Each page has a prominent header span (20.0) plus a body span (10.0)
        // so median = 15.0 and threshold = 19.5 — header at 20.0 passes.
        let make_page = |tag: &str| vec![
            make_span_with_font(tag, 0.40, 0.15, 20.0),
            make_span("body text reference", 0.10, 0.60),
        ];
        let transcript = make_transcript(vec![
            make_page("AHU-1"),
            make_page("AHU-1"),
            make_page("AHU-2"),
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "TEST_PKT");
        let real_units: Vec<_> = idx.units.iter().filter(|u| !u.is_cover).collect();
        assert_eq!(real_units.len(), 2);
        assert_eq!(real_units[0].unit_tag, "AHU-1");
        assert_eq!(real_units[0].start_page, 0);
        assert_eq!(real_units[0].end_page, 1);
        assert_eq!(real_units[1].unit_tag, "AHU-2");
        assert_eq!(real_units[1].start_page, 2);
        assert_eq!(real_units[1].end_page, 2);
    }

    #[test]
    fn cover_pages_plus_two_units() {
        // Pages 0–1: no tag (cover), pages 2–3: AHU-1, page 4: RTU-3.
        let transcript = make_transcript(vec![
            vec![make_span("Cover Sheet", 0.10, 0.30)],       // 0: cover
            vec![make_span("Table of Contents", 0.10, 0.30)],  // 1: cover
            vec![make_span_with_font("AHU-1", 0.40, 0.15, 20.0)], // 2: unit start
            vec![make_span("Performance data", 0.10, 0.50)],   // 3: extends AHU-1
            vec![make_span_with_font("RTU-3", 0.40, 0.15, 20.0)], // 4: new unit
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "TEST_PKT");

        // There should be a cover + 2 real units.
        let cover_units: Vec<_> = idx.units.iter().filter(|u| u.is_cover).collect();
        let real_units: Vec<_> = idx.units.iter().filter(|u| !u.is_cover).collect();
        assert_eq!(cover_units.len(), 1, "should have 1 cover entry");
        assert_eq!(cover_units[0].start_page, 0);
        assert_eq!(cover_units[0].end_page, 1);
        assert_eq!(real_units.len(), 2);
        assert_eq!(real_units[0].unit_tag, "AHU-1");
        assert_eq!(real_units[0].start_page, 2);
        assert_eq!(real_units[0].end_page, 3);
        assert_eq!(real_units[1].unit_tag, "RTU-3");
        assert_eq!(real_units[1].start_page, 4);
        assert_eq!(real_units[1].end_page, 4);
    }

    #[test]
    fn coverage_computed_correctly() {
        // 4 pages total: 2 cover (unassigned by design in coverage), 2 real.
        let transcript = make_transcript(vec![
            vec![make_span("Cover", 0.10, 0.30)],
            vec![make_span("Cover 2", 0.10, 0.30)],
            vec![make_span_with_font("AHU-1", 0.40, 0.15, 20.0)],
            vec![make_span("Performance data", 0.10, 0.50)],
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "TEST_PKT");
        assert_eq!(idx.coverage.total_pages, 4);
        // All 4 pages are assigned (cover entry + AHU-1 entry).
        assert_eq!(idx.coverage.assigned_pages, 4);
        assert_eq!(idx.coverage.unassigned_pages, 0);
        // unit_count counts only non-cover entries.
        assert_eq!(idx.coverage.unit_count, 1);
    }

    #[test]
    fn packet_name_propagated() {
        let transcript = make_transcript(vec![
            vec![make_span("Cover", 0.10, 0.30)],
        ]);
        let idx = SubmittalSegmentEngine::build_index(&transcript, "MY_PACKET");
        assert_eq!(idx.packet_name, "MY_PACKET");
        assert_eq!(idx.schema_version, "1.0.0");
    }
}

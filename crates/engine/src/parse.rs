//! Phase 3 paragraph-parsing engine.
//!
//! Converts a [`LayoutTranscript`] + [`SegmentIndex`] pair into a
//! [`ParsedDocument`] by:
//!
//! 1. For each section in the index, extracting body-band spans from the
//!    transcript pages in `start_page..=end_page`.
//! 2. **Line clustering** — grouping spans whose Y coordinates are within
//!    [`LINE_Y_EPSILON`] into a single visual line.
//! 3. **Line classification** — matching each line against the CSI 3-part
//!    outline-marker patterns to assign an [`OutlineTag`] and depth level.
//! 4. **Continuation folding** — non-matching lines are appended to the text
//!    of the most-recent structural node rather than becoming orphan nodes.
//! 5. **Tree building** — structuring the flat classified list into a nested
//!    `Vec<AstNode>` by using depth ordering.
//!
//! # Body band
//!
//! Only spans with `y ∈ [0.15, 0.85]` (normalised top-left) are included;
//! header and footer chrome are excluded so they do not pollute the outline.
//!
//! # CSI level assignments
//!
//! | Level | Pattern | Example |
//! |-------|---------|---------|
//! | 0 | `PART N [— TITLE]` | `PART 2 — PRODUCTS` |
//! | 1 | `N.N [TITLE]` | `1.1 RELATED DOCUMENTS` |
//! | 2 | `[A-Z]. text` | `A. Basis of Design: …` |
//! | 3 | `[0-9]. text` | `1. Carrier` |
//! | 4 | `[a-z]. text` | `a. Sub-item` |
//! | 5 | `N) text` | `1) Note …` |

use crate::error::Result;
use conset_pdf_ir::{AstNode, OutlineTag, ParsedDocument, SectionAst, SectionLayout, SegmentIndex, LayoutTranscript};
use regex::Regex;
use std::sync::OnceLock;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum Y difference (normalised) between spans on the same visual line.
/// PDFium sometimes places characters on very slightly different baselines
/// within the same visual line (e.g. a dash rendered 0.006 below the word it
/// separates), so a tolerance of 0.012 is used.
const LINE_Y_EPSILON: f64 = 0.012;

/// Body band: top boundary (normalized, top-left origin).
const BODY_Y_MIN: f64 = 0.15;
/// Body band: bottom boundary.
const BODY_Y_MAX: f64 = 0.85;

// ── Compiled regex patterns ───────────────────────────────────────────────────

fn part_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: PART 1   PART 1 - GENERAL   PART 1 — PRODUCTS
    RE.get_or_init(|| {
        Regex::new(r"^PART\s+(\d+)(?:\s*[-\u{2013}\u{2014}]\s*(.+))?$").unwrap()
    })
}

fn article_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: 1.1 SUMMARY   2.7 HYDRONIC HEATING COILS
    // Major number must be ≥ 1 (no "0.x") and title must start with an
    // uppercase letter so that decimal measurements such as "0.016 inches
    // thick" or continuation items like "0.26 Acceptable Manufacturers" are
    // not falsely matched as article markers.
    RE.get_or_init(|| Regex::new(r"^([1-9]\d*\.\d+)\s+([A-Z].*)\.?$").unwrap())
}

fn paragraph_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: A. text   B. text
    RE.get_or_init(|| Regex::new(r"^([A-Z])\.\s+(.+)$").unwrap())
}

fn sub_paragraph_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: 1. Carrier   2. York  (NOT 1.1 which is an article)
    RE.get_or_init(|| Regex::new(r"^(\d+)\.\s+(.+)$").unwrap())
}

fn sub_sub_paragraph_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: a. Sub-item   b. Sub-item
    RE.get_or_init(|| Regex::new(r"^([a-z])\.\s+(.+)$").unwrap())
}

fn sub_sub_sub_paragraph_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: 1) Note   2) Note
    RE.get_or_init(|| Regex::new(r"^(\d+\))\s+(.+)$").unwrap())
}

// ── Internal flat item ────────────────────────────────────────────────────────

struct FlatItem {
    tag: OutlineTag,
    marker: String,
    text: String,
    page_index: usize,
    level: u8,
    /// Normalized x position of the leftmost span on this item's line.
    x_indent: f64,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse all (or a filtered subset of) sections from a transcript + segment index.
///
/// When `section_filter` is `Some(id)`, only the section whose `section_id`
/// matches exactly is parsed; all others are skipped.
///
/// # Errors
///
/// Returns [`crate::error::EngineError`] if required state is inconsistent.
pub fn parse_document(
    transcript: &LayoutTranscript,
    index: &SegmentIndex,
    section_filter: Option<&str>,
) -> Result<ParsedDocument> {
    let source_path = index.source_path.clone();
    let mut sections = Vec::new();
    let mut global_warnings = Vec::new();

    for section_entry in &index.sections {
        if let Some(filter) = section_filter {
            if section_entry.section_id != filter {
                continue;
            }
        }
        sections.push(parse_section(transcript, section_entry));
    }

    if sections.is_empty() {
        let msg = match section_filter {
            Some(f) => format!("No section '{f}' found in segment index"),
            None => "No sections found in segment index".to_owned(),
        };
        global_warnings.push(msg);
    }

    Ok(ParsedDocument { source_path, sections, global_warnings })
}

// ── Internal: per-section parsing ────────────────────────────────────────────

fn parse_section(
    transcript: &LayoutTranscript,
    entry: &conset_pdf_ir::SectionEntry,
) -> SectionAst {
    let pages = transcript.pages();
    let mut all_lines: Vec<(String, usize, f64)> = Vec::new();

    // Span data collected for SectionLayout computation.
    let mut x_vals: Vec<f64> = Vec::new();
    let mut x_right_vals: Vec<f64> = Vec::new();
    let mut font_sizes: Vec<f64> = Vec::new();
    let mut span_ys: Vec<f64> = Vec::new();
    let mut font_names: Vec<String> = Vec::new();

    for page_idx in entry.start_page..=entry.end_page {
        if let Some(page) = pages.get(page_idx) {
            // Filter to body band only, then sort by (y, x) to normalise
            // content-stream order into visual reading order.
            let mut body_spans: Vec<_> = page
                .spans()
                .iter()
                .filter(|s| s.bbox.y >= BODY_Y_MIN && s.bbox.y <= BODY_Y_MAX)
                .collect();
            body_spans.sort_by(|a, b| {
                a.bbox.y
                    .partial_cmp(&b.bbox.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(
                        a.bbox.x
                            .partial_cmp(&b.bbox.x)
                            .unwrap_or(std::cmp::Ordering::Equal),
                    )
            });

            // Collect raw span data for layout geometry.
            for s in &body_spans {
                x_vals.push(s.bbox.x);
                x_right_vals.push(s.bbox.x + s.bbox.width);
                font_sizes.push(s.font_size);
                span_ys.push(s.bbox.y);
                font_names.push(s.font_name.clone());
            }

            all_lines.extend(cluster_lines(&body_spans, page_idx));
        }
    }

    let layout = compute_section_layout(&x_vals, &x_right_vals, &font_sizes, &span_ys, &font_names);

    let flat_items = classify_lines(&all_lines);
    let flat_items = inject_missing_parts(flat_items);
    let nodes = build_tree(&flat_items);

    SectionAst {
        section_id: entry.section_id.clone(),
        section_title: entry.section_title.clone(),
        start_page: entry.start_page,
        end_page: entry.end_page,
        nodes,
        parse_warnings: Vec::new(),
        layout,
    }
}

// ── Line clustering ───────────────────────────────────────────────────────────

/// Groups body-band spans into visual lines by Y proximity.
///
/// Spans are assumed to be pre-sorted by (y, x).  Within each line the span
/// texts are joined with a single space.
///
/// Returns `(text, page_index, x_min)` where `x_min` is the normalized x
/// coordinate of the leftmost span on the line — used to measure indentation.
fn cluster_lines(
    spans: &[&conset_pdf_ir::Span],
    page_index: usize,
) -> Vec<(String, usize, f64)> {
    let mut lines: Vec<(String, usize, f64)> = Vec::new();
    let mut current_line: Vec<&&conset_pdf_ir::Span> = Vec::new();
    let mut line_y_start: Option<f64> = None;

    let flush = |line: &[&&conset_pdf_ir::Span], lines: &mut Vec<(String, usize, f64)>| {
        if line.is_empty() {
            return;
        }
        // Sort by x so that spans on the same visual line appear in reading order
        // regardless of their content-stream order.
        let mut sorted = line.to_vec();
        sorted.sort_by(|a, b| {
            a.bbox.x
                .partial_cmp(&b.bbox.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // After sorting by x, the first span has the minimum x — the leftmost indent.
        let x_min = sorted.first().map_or(0.0, |s| s.bbox.x);
        let text = sorted.iter().map(|s| s.text.trim()).collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            lines.push((text, page_index, x_min));
        }
    };

    for span in spans {
        match line_y_start {
            None => {
                line_y_start = Some(span.bbox.y);
                current_line.push(span);
            }
            Some(y_start) => {
                if (span.bbox.y - y_start).abs() <= LINE_Y_EPSILON {
                    current_line.push(span);
                } else {
                    flush(&current_line, &mut lines);
                    current_line = vec![span];
                    line_y_start = Some(span.bbox.y);
                }
            }
        }
    }
    flush(&current_line, &mut lines);

    lines
}

// ── Outline classification ────────────────────────────────────────────────────

/// Classifies a list of `(text, page_index, x_indent)` pairs into structural `FlatItem`s.
///
/// Lines that do not match any outline pattern are folded into the text of the
/// previously emitted structural item.  This handles wrapped paragraph text
/// without creating spurious Unclassified nodes.
fn classify_lines(lines: &[(String, usize, f64)]) -> Vec<FlatItem> {
    let mut items: Vec<FlatItem> = Vec::new();

    for (text, page_index, x_indent) in lines {
        if let Some(mut item) = try_classify(text, *page_index) {
            item.x_indent = *x_indent;
            items.push(item);
        } else if !text.trim().is_empty() {
            // Continuation: append to previous structural node if available.
            if let Some(prev) = items.last_mut() {
                if !prev.text.is_empty() {
                    prev.text.push(' ');
                }
                prev.text.push_str(text.trim());
            } else {
                // No previous node: only emit as root-level unclassified if the
                // line contains meaningful alphanumeric content.  Decoration
                // lines (e.g. a lone "-" or "–") are silently discarded so they
                // cannot become anchor nodes that absorb all subsequent content
                // via the continuation-folding path.
                if text.chars().any(|c| c.is_alphanumeric()) {
                    items.push(FlatItem {
                        tag: OutlineTag::Unclassified,
                        marker: String::new(),
                        text: text.trim().to_owned(),
                        page_index: *page_index,
                        level: 0,
                        x_indent: *x_indent,
                    });
                }
            }
        }
    }

    items
}

/// Attempt to match `text` against the CSI outline-marker patterns.
///
/// Returns `None` when the line is a continuation, not a structural marker.
/// The `x_indent` field on the returned item is always `0.0`; callers must
/// overwrite it with the measured value after this function returns.
fn try_classify(text: &str, page_index: usize) -> Option<FlatItem> {
    let t = text.trim();

    // PART N [— TITLE]  (level 0)
    if let Some(caps) = part_re().captures(t) {
        let num = &caps[1];
        let title = caps.get(2).map_or("", |m| m.as_str()).trim();
        return Some(FlatItem {
            tag: OutlineTag::Part,
            marker: format!("PART {num}"),
            text: title.to_owned(),
            page_index,
            level: 0,
            x_indent: 0.0,
        });
    }

    // N.N TITLE  (level 1) — must come before sub-paragraph check
    if let Some(caps) = article_re().captures(t) {
        return Some(FlatItem {
            tag: OutlineTag::Article,
            marker: caps[1].to_owned(),
            text: caps[2].trim().to_owned(),
            page_index,
            level: 1,
            x_indent: 0.0,
        });
    }

    // A. text  (level 2)
    if let Some(caps) = paragraph_re().captures(t) {
        return Some(FlatItem {
            tag: OutlineTag::Paragraph,
            marker: format!("{}.", &caps[1]),
            text: caps[2].trim().to_owned(),
            page_index,
            level: 2,
            x_indent: 0.0,
        });
    }

    // 1. text  (level 3) — article_re already matched N.N so this is N. only
    if let Some(caps) = sub_paragraph_re().captures(t) {
        return Some(FlatItem {
            tag: OutlineTag::SubParagraph,
            marker: format!("{}.", &caps[1]),
            text: caps[2].trim().to_owned(),
            page_index,
            level: 3,
            x_indent: 0.0,
        });
    }

    // a. text  (level 4)
    if let Some(caps) = sub_sub_paragraph_re().captures(t) {
        return Some(FlatItem {
            tag: OutlineTag::SubSubParagraph,
            marker: format!("{}.", &caps[1]),
            text: caps[2].trim().to_owned(),
            page_index,
            level: 4,
            x_indent: 0.0,
        });
    }

    // 1) text  (level 5)
    if let Some(caps) = sub_sub_sub_paragraph_re().captures(t) {
        return Some(FlatItem {
            tag: OutlineTag::SubSubSubParagraph,
            marker: caps[1].to_owned(),
            text: caps[2].trim().to_owned(),
            page_index,
            level: 5,
            x_indent: 0.0,
        });
    }

    None
}

// ── Part-gap recovery ─────────────────────────────────────────────────────────

/// Returns the numeric part index from a `"PART N"` marker, e.g. `"PART 2"` → 2.
fn part_number_from_marker(marker: &str) -> Option<u32> {
    marker.strip_prefix("PART ")?.trim().parse().ok()
}

/// Returns the major (part) component of an article marker, e.g. `"3.1"` → 3.
fn article_part_number(marker: &str) -> Option<u32> {
    let dot = marker.find('.')?;
    marker[..dot].trim().parse().ok()
}

/// Scan the flat item list and insert synthetic `Part` items whenever an
/// article's major number differs from the currently-open part.
///
/// This recovers from two failure modes:
/// 1. A PART heading whose text had kerning artifacts (`"P ART 3"`) that
///    `part_re` could not match.
/// 2. The segmenter ending a section one page too early, causing the PART
///    heading page to be excluded from the parsed range.
///
/// Injection only occurs when at least one explicit PART has already been seen
/// (front-matter sections with no PART structure are left unchanged) and the
/// article's major number is ≥ 1 (guards against stray `0.x` decimal items).
fn inject_missing_parts(items: Vec<FlatItem>) -> Vec<FlatItem> {
    let mut result: Vec<FlatItem> = Vec::with_capacity(items.len() + 4);
    let mut current_part: Option<u32> = None;

    for item in items {
        if item.tag == OutlineTag::Part {
            if let Some(n) = part_number_from_marker(&item.marker) {
                current_part = Some(n);
            }
            result.push(item);
        } else if item.tag == OutlineTag::Article {
            if let Some(art_part) = article_part_number(&item.marker) {
                if art_part >= 1
                    && current_part.is_some()
                    && current_part != Some(art_part)
                {
                    result.push(FlatItem {
                        tag: OutlineTag::Part,
                        marker: format!("PART {art_part}"),
                        text: String::new(),
                        page_index: item.page_index,
                        level: 0,
                        x_indent: 0.0,
                    });
                    current_part = Some(art_part);
                }
            }
            result.push(item);
        } else {
            result.push(item);
        }
    }

    result
}

// ── Tree builder ──────────────────────────────────────────────────────────────

/// Builds a nested `Vec<AstNode>` from a flat classified list.
///
/// The algorithm: for each item, all immediately following items whose
/// `level` is strictly greater are its children (processed recursively).
/// This mirrors how a stack-based tree construction works, but avoids
/// lifetime complexity by operating on index ranges.
fn build_tree(items: &[FlatItem]) -> Vec<AstNode> {
    if items.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < items.len() {
        let item = &items[i];
        let child_start = i + 1;

        // Scan forward to find the end of this item's children (next item at
        // same or lesser level).
        let mut child_end = child_start;
        while child_end < items.len() && items[child_end].level > item.level {
            child_end += 1;
        }

        let children = build_tree(&items[child_start..child_end]);
        result.push(AstNode {
            tag: item.tag.clone(),
            marker: item.marker.clone(),
            text: item.text.clone(),
            page_index: item.page_index,
            level: item.level,
            x_indent: item.x_indent,
            children,
        });

        i = child_end;
    }

    result
}

// ── Layout geometry computation ───────────────────────────────────────────────

/// Computes [`SectionLayout`] from raw body-span data collected during parsing.
///
/// Returns `None` when fewer than two spans are available (empty / title-only
/// sections have no meaningful layout geometry).
fn compute_section_layout(
    x_vals: &[f64],
    x_right_vals: &[f64],
    font_sizes: &[f64],
    span_ys: &[f64],
    font_names: &[String],
) -> Option<SectionLayout> {
    if x_vals.len() < 2 {
        return None;
    }

    let body_left = x_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let body_right = x_right_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let font_size_pt = median_val(font_sizes);

    // Compute the modal (most-frequent) font family name.
    let body_font_name = modal_font_name(font_names);

    // Compute line gaps: sort unique y values (one per visual line), then take
    // differences between consecutive lines.
    let mut sorted_ys = span_ys.to_vec();
    sorted_ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut line_ys: Vec<f64> = Vec::new();
    let mut prev_y: Option<f64> = None;
    for y in sorted_ys {
        match prev_y {
            None => { prev_y = Some(y); line_ys.push(y); }
            Some(py) if (y - py).abs() > LINE_Y_EPSILON => { prev_y = Some(y); line_ys.push(y); }
            _ => {}
        }
    }

    let gaps: Vec<f64> = line_ys.windows(2).map(|w| w[1] - w[0]).collect();
    let line_gap_norm = if gaps.is_empty() { 0.0 } else { median_val(&gaps) };

    Some(SectionLayout { body_left, body_right, font_size_pt, line_gap_norm, body_font_name })
}

/// Returns the most-frequent font name from a slice of names.
/// Falls back to `"Unknown"` for an empty slice.
fn modal_font_name(names: &[String]) -> String {
    use std::collections::HashMap;
    if names.is_empty() {
        return "Unknown".to_string();
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for name in names {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Returns the median of a non-empty slice.  Returns `0.0` for an empty slice.
fn median_val(vals: &[f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    let mut sorted = vals.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

//! Section segmentation: furniture detection and CSI section-ID oracle.
//!
//! Reads a completed [`LayoutTranscript`] and produces a [`SegmentIndex`] by:
//!
//! 1. **Footer oracle** — for each page, scan spans in the bottom 15 % of the
//!    page (Y > 0.85) for a CSI MasterFormat section-ID pattern
//!    (`\b\d{2}\s+\d{2}(?:\s+\d{2})?\b`).
//! 2. **Header oracle** — scan spans in the top 15 % (Y < 0.15) of the first
//!    few pages for project-ID, project-name, firm, and date strings.
//! 3. **Section boundary detection** — group consecutive pages by section ID;
//!    a boundary is raised when the section ID changes.
//! 4. **Page-counter detection** — scan footer spans for `Page N of M` patterns
//!    to produce the `page_counter_detected` flag per section.
//! 5. **Coverage stats** — count pages with and without a detected section ID.
//!
//! # Design decisions
//!
//! - Chrome metadata extraction is best-effort: missing fields never block
//!   segmentation.  An empty string means the field was not detected.
//! - Section IDs are normalised to canonical form: space-separated two-digit
//!   groups, e.g. `"23 82 16"`.  Leading zeros are preserved.
//! - Page-counter detection is per-section, not per-page.
//! - Confidence is the hit-rate within the section: pages with a footer ID
//!   detected / total pages in the section.

use crate::error::{EngineError, Result};
use conset_pdf_ir::{ChromeMetadata, CoverageStats, LayoutTranscript, SegmentIndex, SectionEntry, Span};
use regex::Regex;
use std::sync::OnceLock;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Bottom-band Y threshold (normalised, top-left origin).
/// Set to 0.90 rather than 0.85 so that body text which bleeds slightly
/// past the nominal body band (y ≈ 0.86-0.89) is excluded from section-ID
/// detection.  Valid section stamps in this corpus sit at y ≈ 0.943.
const FOOTER_Y: f64 = 0.90;

/// Maximum normalised-X gap between consecutive footer spans that are still
/// considered part of the same token cluster.  The date block and section-ID
/// block are separated by a much larger gap (≥ 0.3), so this threshold
/// cleanly splits them.
const FOOTER_CLUSTER_GAP: f64 = 0.06;

// ── Compiled regex patterns (lazily initialised) ──────────────────────────────

fn section_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(\d{2})\s+(\d{2})(?:\s+(\d{2}))?\b").unwrap())
}

fn page_counter_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bpage\s+\d+\s+of\s+\d+\b").unwrap())
}

/// Matches a 4-digit year or any other 4-digit token — used to identify the
/// date cluster in footer so it can be skipped.
fn four_digit_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d{4}\b").unwrap())
}

/// Merges adjacent single-digit tokens separated by whitespace into 2-digit
/// tokens.  Handles PDFium rendering artifacts where a two-digit group like
/// `00` is split into two adjacent `0` spans (e.g. `"0 0"` → `"00"`).
fn merge_single_digits_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(\d)\s(\d)\b").unwrap())
}

fn project_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)project\s+no[\.\:]?\s*([\w\.\-]+(?:\.\d+)*)").unwrap()
    })
}

fn date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(\d{4}[.\-/]\d{2}[.\-/]\d{2}|\d{2}[.\-/]\d{2}[.\-/]\d{4})\b").unwrap()
    })
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Segments a [`LayoutTranscript`] into a [`SegmentIndex`].
///
/// # Errors
///
/// Returns [`EngineError`] if required transcript state is inconsistent.
pub fn segment_transcript(transcript: &LayoutTranscript) -> Result<SegmentIndex> {
    let source_path = transcript.metadata().source_path.clone();
    let pages = transcript.pages();

    if pages.is_empty() {
        return Err(EngineError::other("transcript has no pages".to_owned()));
    }

    // Per-page footer section IDs (None = not detected).
    let mut page_section_ids: Vec<Option<String>> = Vec::with_capacity(pages.len());
    // Per-page section titles (empty string when not detected).
    let mut page_section_titles: Vec<String> = Vec::with_capacity(pages.len());
    // Per-page page-counter presence.
    let mut page_counter_flags: Vec<bool> = Vec::with_capacity(pages.len());

    for page in pages {
        let mut found_counter = false;

        for span in page.spans() {
            let y = span.bbox.y;
            // Page-counter detection (still per-span — phrase is rarely split).
            if y > FOOTER_Y && !found_counter && page_counter_re().is_match(&span.text) {
                found_counter = true;
            }
        }
        // Section-ID detection: build X-clusters across the footer band so that
        // digits split across multiple adjacent spans are reassembled.
        let footer_spans: Vec<&Span> = page
            .spans()
            .iter()
            .filter(|s| s.bbox.y > FOOTER_Y)
            .collect();
        let (found_id, found_title) = match detect_section_id(&footer_spans) {
            Some((id, title)) => (Some(id), title),
            None => (None, String::new()),
        };

        page_section_ids.push(found_id);
        page_section_titles.push(found_title);
        page_counter_flags.push(found_counter);
    }

    // Chrome metadata from first pages (scan up to 5).
    let chrome_metadata = extract_chrome_metadata(transcript);

    // Build section list.
    let sections = build_sections(&page_section_ids, &page_section_titles, &page_counter_flags);

    // Coverage.
    let pages_tagged = page_section_ids.iter().filter(|id| id.is_some()).count();
    let pages_total = pages.len();
    let pages_missing_footer = pages_total - pages_tagged;
    let coverage_ratio = if pages_total == 0 {
        0.0
    } else {
        pages_tagged as f64 / pages_total as f64
    };

    Ok(SegmentIndex {
        source_path,
        chrome_metadata,
        sections,
        coverage: CoverageStats {
            pages_total,
            pages_tagged,
            pages_missing_footer,
            coverage_ratio,
        },
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Detect the CSI section-ID and section title from a pre-filtered slice of
/// footer-band spans.
///
/// The footer layout for this corpus is:
/// `[yyyy-mm-dd date]  [NN NN NN section-id]  [– section-name]  [Page N]`
///
/// Because the section-ID digits are frequently split across individual spans,
/// we group spans into X-clusters (consecutive spans within
/// [`FOOTER_CLUSTER_GAP`] horizontally / same Y-row), then find the first
/// cluster that:
/// * does **not** contain a 4-digit year (which would mark the date cluster),
/// * **does** match the CSI section-ID pattern.
///
/// A final pre-processing pass merges adjacent single-digit tokens (`"0 0"` →
/// `"00"`) to handle split-zero rendering artefacts before pattern matching.
///
/// Returns `Some((section_id, section_title))` where `section_title` may be an
/// empty string when no title cluster is found after the section-ID cluster.
fn detect_section_id(footer_spans: &[&Span]) -> Option<(String, String)> {
    if footer_spans.is_empty() {
        return None;
    }

    // Sort by (y, x) to get reading order.
    let mut sorted = footer_spans.to_vec();
    sorted.sort_by(|a, b| {
        a.bbox.y
            .partial_cmp(&b.bbox.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                a.bbox.x
                    .partial_cmp(&b.bbox.x)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    // Build X-clusters.
    let mut clusters: Vec<String> = Vec::new();
    let mut cluster_tokens: Vec<&str> = Vec::new();
    let mut last_x: f64 = f64::NEG_INFINITY;
    let mut last_y: f64 = f64::NEG_INFINITY;

    for span in &sorted {
        let t = span.text.trim();
        if t.is_empty() {
            continue;
        }
        let x = span.bbox.x;
        let y = span.bbox.y;

        // A new cluster starts when the Y row changes, the X position jumps
        // back to the left (new visual row), or the X gap forward is large.
        let new_cluster = (y - last_y).abs() > 0.01
            || x < last_x
            || (x - last_x) > FOOTER_CLUSTER_GAP;
        if new_cluster && !cluster_tokens.is_empty() {
            clusters.push(cluster_tokens.join(" "));
            cluster_tokens.clear();
        }
        cluster_tokens.push(t);
        last_x = x;
        last_y = y;
    }
    if !cluster_tokens.is_empty() {
        clusters.push(cluster_tokens.join(" "));
    }

    // Inspect each cluster for a section ID.
    for (i, cluster) in clusters.iter().enumerate() {
        // Skip clusters that contain a 4-digit year (date cluster).
        if four_digit_re().is_match(cluster) {
            continue;
        }
        // Merge adjacent single-digit tokens before pattern matching so that
        // a split "0 0" becomes "00" and forms a valid 2-digit group.
        let merged = merge_single_digits_re()
            .replace_all(cluster, |caps: &regex::Captures| {
                format!("{}{}", &caps[1], &caps[2])
            })
            .into_owned();
        if let Some(caps) = section_id_re().captures(&merged) {
            let section_id = normalise_section_id(&caps);
            let section_title = extract_title_from_clusters(&clusters, i + 1);
            return Some((section_id, section_title));
        }
    }

    None
}

/// Extract the section title from the clusters that follow the section-ID cluster.
///
/// The title cluster is typically formatted as `"– SECTION TITLE"` or
/// `"SECTION TITLE"`.  Leading em-dashes, hyphens, and whitespace are stripped.
/// Clusters that look like page counters or are purely numeric are skipped.
fn extract_title_from_clusters(clusters: &[String], start: usize) -> String {
    if start >= clusters.len() {
        return String::new();
    }
    for cluster in &clusters[start..] {
        // Skip page-counter clusters ("Page N of M").
        if page_counter_re().is_match(cluster) {
            continue;
        }
        // Skip pure-digit or date clusters.
        let letters: usize = cluster.chars().filter(|c| c.is_alphabetic()).count();
        if letters == 0 {
            continue;
        }
        // Strip leading dash variants and whitespace.
        let stripped = cluster
            .trim_start_matches(|c: char| c == '\u{2013}' || c == '\u{2014}' || c == '-' || c == ' ')
            .trim();
        if !stripped.is_empty() {
            return stripped.to_owned();
        }
    }
    String::new()
}

/// Normalise a section-ID regex capture to canonical form `"NN NN"` or `"NN NN NN"`.
fn normalise_section_id(caps: &regex::Captures<'_>) -> String {
    match caps.get(3) {
        Some(third) => format!("{} {} {}", &caps[1], &caps[2], third.as_str()),
        None => format!("{} {}", &caps[1], &caps[2]),
    }
}

/// Build the ordered section list from per-page section IDs and titles.
///
/// The section title stored on each [`SectionEntry`] is taken from the first
/// page in the run that has a non-empty detected title.
fn build_sections(
    page_section_ids: &[Option<String>],
    page_section_titles: &[String],
    page_counter_flags: &[bool],
) -> Vec<SectionEntry> {
    let mut sections: Vec<SectionEntry> = Vec::new();
    let n = page_section_ids.len();

    // Track current run.
    let mut run_id: Option<String> = None;
    let mut run_start: usize = 0;
    let mut run_hits: usize = 0; // pages in the run that had a detected ID

    let flush = |run_id: &Option<String>,
                 run_start: usize,
                 run_end: usize,
                 run_hits: usize,
                 page_section_titles: &[String],
                 page_counter_flags: &[bool],
                 sections: &mut Vec<SectionEntry>| {
        let sid = match run_id {
            Some(id) => id.clone(),
            None => return, // skip runs with no detected section ID
        };
        let page_count = run_end - run_start + 1;
        let confidence = run_hits as f64 / page_count as f64;
        let page_counter_detected = page_counter_flags[run_start..=run_end].iter().any(|&f| f);
        // Use the first non-empty title found within the run's pages.
        let section_title = page_section_titles[run_start..=run_end]
            .iter()
            .find(|t| !t.is_empty())
            .cloned()
            .unwrap_or_default();
        sections.push(SectionEntry {
            section_id: sid,
            section_title,
            start_page: run_start,
            end_page: run_end,
            page_count,
            page_counter_detected,
            confidence,
        });
    };

    for (i, id_opt) in page_section_ids.iter().enumerate() {
        let current_id_str = id_opt.as_deref();
        let run_id_str = run_id.as_deref();

        if i == 0 {
            // Start the first run.
            run_id = id_opt.clone();
            run_start = 0;
            run_hits = usize::from(id_opt.is_some());
            continue;
        }

        // A boundary is detected when the section ID changes (None→Some, Some→None,
        // or Some(a)→Some(b) where a != b).
        let boundary = match (run_id_str, current_id_str) {
            (Some(a), Some(b)) => a != b,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        };

        if boundary {
            flush(&run_id, run_start, i - 1, run_hits, page_section_titles, page_counter_flags, &mut sections);
            run_id = id_opt.clone();
            run_start = i;
            run_hits = usize::from(id_opt.is_some());
        } else {
            if id_opt.is_some() {
                run_hits += 1;
            }
        }
    }

    // Flush the last run.
    if n > 0 {
        flush(&run_id, run_start, n - 1, run_hits, page_section_titles, page_counter_flags, &mut sections);
    }

    sections
}

/// Extract best-effort chrome metadata from the first few pages.
fn extract_chrome_metadata(transcript: &LayoutTranscript) -> ChromeMetadata {
    let mut meta = ChromeMetadata::default();

    let pages = transcript.pages();
    let scan_pages = pages.len().min(5);

    for page in &pages[..scan_pages] {
        for span in page.spans() {
            let y = span.bbox.y;
            let text = &span.text;

            // Header band: top 15 %.
            if y < 0.15 {
                // Project ID
                if meta.project_id.is_empty() {
                    if let Some(caps) = project_id_re().captures(text) {
                        let matched =
                            &text[caps.get(0).unwrap().start()..caps.get(0).unwrap().end()];
                        meta.project_id = matched.to_owned();
                    }
                }

                // Firm name: look for professional-firm keywords
                if meta.firm.is_empty() && is_firm_name(text) {
                    meta.firm = text.trim().to_owned();
                }

                // Project name: long, mixed-case line that's not a firm or ID
                if meta.project_name.is_empty()
                    && text.trim().len() > 10
                    && !is_firm_name(text)
                    && meta.project_id.is_empty()
                    && !is_all_upper(text)
                    && !date_re().is_match(text)
                {
                    meta.project_name = text.trim().to_owned();
                }

                // Date
                if meta.date.is_empty() {
                    if let Some(m) = date_re().find(text) {
                        meta.date = m.as_str().to_owned();
                    }
                }
            }

            // Footer band may also contain project ID or date.
            if y > 0.85 {
                if meta.project_id.is_empty() {
                    if let Some(caps) = project_id_re().captures(text) {
                        let matched = &text[caps.get(0).unwrap().start()..caps.get(0).unwrap().end()];
                        meta.project_id = matched.to_owned();
                    }
                }
                if meta.date.is_empty() {
                    if let Some(m) = date_re().find(text) {
                        meta.date = m.as_str().to_owned();
                    }
                }
            }
        }
    }

    meta
}

/// Returns `true` when `text` looks like a firm / organization name.
fn is_firm_name(text: &str) -> bool {
    let lower = text.to_lowercase();
    let keywords = [
        "engineers",
        "engineering",
        "architects",
        "architecture",
        "consulting",
        "consultants",
        "design",
        "inc.",
        "llc",
        "corp.",
        "company",
        "associates",
        "group",
    ];
    keywords.iter().any(|kw| lower.contains(kw))
}

/// Returns `true` when most characters in `text` are uppercase letters.
fn is_all_upper(text: &str) -> bool {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return false;
    }
    let upper_count = letters.iter().filter(|c| c.is_uppercase()).count();
    upper_count as f64 / letters.len() as f64 > 0.8
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{BBox, Span};

    /// Build a minimal `Span` fixture at the given x/y coordinates.
    fn make_span(text: &str, x: f64, y: f64) -> Span {
        Span::new(text, BBox::new(x, y, 0.05, 0.01).unwrap(), 10.0).unwrap()
    }

    // ── detect_section_id ─────────────────────────────────────────────────────

    /// Canonical corpus footer layout:
    ///   date cluster | section-id cluster | em-dash title cluster | page counter
    #[test]
    fn test_detect_section_id_with_title() {
        let spans = vec![
            make_span("2025-10-01", 0.05, 0.95),
            make_span("23", 0.30, 0.95),
            make_span("82", 0.35, 0.95),
            make_span("16", 0.40, 0.95),
            make_span("– HEATING WATER COILS", 0.50, 0.95),
            make_span("Page 2 of 3", 0.85, 0.95),
        ];
        let span_refs: Vec<&Span> = spans.iter().collect();
        let result = detect_section_id(&span_refs);
        let (id, title) = result.expect("should detect section id and title");
        assert_eq!(id, "23 82 16");
        assert_eq!(title, "HEATING WATER COILS");
    }

    /// Title cluster without the em-dash prefix.
    #[test]
    fn test_detect_section_id_title_no_dash() {
        let spans = vec![
            make_span("2025-10-01", 0.05, 0.95),
            make_span("23 82 16", 0.30, 0.95),
            // Gap between section id and title spans
            make_span("HEATING WATER COILS", 0.60, 0.95),
        ];
        let span_refs: Vec<&Span> = spans.iter().collect();
        let (id, title) = detect_section_id(&span_refs).expect("should detect id");
        assert_eq!(id, "23 82 16");
        assert_eq!(title, "HEATING WATER COILS");
    }

    /// No title cluster present — title should come back empty.
    #[test]
    fn test_detect_section_id_no_title() {
        let spans = vec![
            make_span("2025-10-01", 0.05, 0.95),
            make_span("23 82 16", 0.30, 0.95),
        ];
        let span_refs: Vec<&Span> = spans.iter().collect();
        let (id, title) = detect_section_id(&span_refs).expect("should detect id");
        assert_eq!(id, "23 82 16");
        assert_eq!(title, "");
    }

    /// No recognised section-ID in the footer — returns None.
    #[test]
    fn test_detect_section_id_no_match_returns_none() {
        let spans = vec![
            make_span("2025-10-01", 0.05, 0.95),
            make_span("G1.01", 0.30, 0.95),
        ];
        let span_refs: Vec<&Span> = spans.iter().collect();
        assert!(detect_section_id(&span_refs).is_none());
    }

    /// Empty span slice — returns None.
    #[test]
    fn test_detect_section_id_empty_spans() {
        assert!(detect_section_id(&[]).is_none());
    }

    // ── extract_title_from_clusters ───────────────────────────────────────────

    #[test]
    fn test_extract_title_strips_em_dash() {
        let clusters =
            vec!["– HEATING WATER COILS".to_owned(), "Page 2 of 3".to_owned()];
        let title = extract_title_from_clusters(&clusters, 0);
        assert_eq!(title, "HEATING WATER COILS");
    }

    #[test]
    fn test_extract_title_strips_hyphen() {
        let clusters = vec!["- TABLE OF CONTENTS".to_owned()];
        let title = extract_title_from_clusters(&clusters, 0);
        assert_eq!(title, "TABLE OF CONTENTS");
    }

    #[test]
    fn test_extract_title_skips_page_counter() {
        let clusters =
            vec!["Page 1 of 5".to_owned(), "HVAC SPECIFICATIONS".to_owned()];
        let title = extract_title_from_clusters(&clusters, 0);
        assert_eq!(title, "HVAC SPECIFICATIONS");
    }

    #[test]
    fn test_extract_title_empty_when_only_page_counter() {
        let clusters = vec!["Page 1 of 5".to_owned()];
        assert_eq!(extract_title_from_clusters(&clusters, 0), "");
    }

    #[test]
    fn test_extract_title_start_beyond_end() {
        let clusters = vec!["23 82 16".to_owned()];
        assert_eq!(extract_title_from_clusters(&clusters, 5), "");
    }

    // ── build_sections ────────────────────────────────────────────────────────

    /// Section titles should be populated from the first non-empty title in each run.
    #[test]
    fn test_build_sections_title_populated() {
        let ids = vec![
            Some("23 82 16".to_owned()),
            Some("23 82 16".to_owned()),
            Some("23 82 16".to_owned()),
        ];
        let titles = vec![
            "HEATING WATER COILS".to_owned(),
            "HEATING WATER COILS".to_owned(),
            "HEATING WATER COILS".to_owned(),
        ];
        let counters = vec![false, true, false];
        let sections = build_sections(&ids, &titles, &counters);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].section_id, "23 82 16");
        assert_eq!(sections[0].section_title, "HEATING WATER COILS");
    }

    /// When only later pages in a run have a detectable title, it is still captured.
    #[test]
    fn test_build_sections_title_from_later_page_in_run() {
        let ids = vec![
            Some("23 82 16".to_owned()),
            Some("23 82 16".to_owned()),
        ];
        let titles = vec![
            String::new(), // first page footer had no title
            "HEATING WATER COILS".to_owned(),
        ];
        let counters = vec![false, false];
        let sections = build_sections(&ids, &titles, &counters);
        assert_eq!(sections[0].section_title, "HEATING WATER COILS");
    }
}

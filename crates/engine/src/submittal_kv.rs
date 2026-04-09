//! Key-value extraction from submittal unit pages.
//!
//! This module provides two complementary extraction passes:
//!
//! 1. **Header extraction** (`extract_unit_header`) — focused on the first 2 pages
//!    of each unit, looking for structured label fields that identify the equipment
//!    tag, model number, and manufacturer.  Regex-based pattern matching with
//!    confidence bands.
//!
//! 2. **KV pair extraction** (`extract_kv_pairs`) — applied across all pages of
//!    a unit, using a colon-split heuristic to identify `"Label: Value"` inline
//!    pairs from the PDF text layer.  Lower confidence than header extraction.
//!
//! # Design notes
//!
//! Both functions operate on pre-extracted [`Page`] slices (already in normalised
//! transcript form) rather than raw PDF bytes.  This ensures all confidence is
//! measured over the same coordinate-normalised representation used by
//! `SubmittalSegmentEngine`.
//!
//! Header field confidence bands:
//! - Regex exact-label match (e.g. `"Tag No.:"`) → `CONF_EXACT = 1.0`
//! - Regex flex-label match (e.g. `"Tag :"`, `"ITEM:"`) → `CONF_FLEX = 0.9`
//! - Fallback / positional match → `CONF_POSITIONAL = 0.7`

use conset_pdf_ir::{KvPair, Page, TidyBBox, UnitHeader};
use regex::Regex;
use std::sync::OnceLock;

// ── Confidence constants ──────────────────────────────────────────────────────

/// Confidence for an exact label regex match (e.g. `"Tag No.:"` → tag).
const CONF_EXACT: f64 = 1.0;

/// Confidence for a flexible label regex match (broader pattern).
const CONF_FLEX: f64 = 0.9;

/// Confidence for a colon-heuristic KV pair extracted from body text.
const CONF_KV: f64 = 0.7;

/// Minimum value length in characters to be accepted as a real value (not noise).
const MIN_VALUE_LEN: usize = 1;

/// Maximum label length in characters; labels longer than this are likely
/// partial sentence continuations, not field labels.
const MAX_LABEL_LEN: usize = 60;

// ── Compiled regex patterns ───────────────────────────────────────────────────

/// Matches a unit tag label pattern (flexible).
///
/// Captures the value after the label.
fn tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(?:tag\s*(?:no\.?|number|#)?|item\s*(?:no\.?|number|tag)?)\s*[:\-]?\s*([A-Z]{1,4}-?\d{1,4}[A-Z]?)\b"
        )
        .unwrap()
    })
}

/// Matches a model number label pattern.
fn model_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(?:model\s*(?:no\.?|number)?|part\s*(?:no\.?|number)?)\s*[:\-]?\s*(\S[\S\s]{0,40}?)(?:\s*$|\s{2,})"
        )
        .unwrap()
    })
}

/// Matches a manufacturer / brand label pattern.
fn manufacturer_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(?:manufacturer|brand|mfr?\.?|mfg\.?)\s*[:\-]?\s*(\S[\S\s]{0,40}?)(?:\s*$|\s{2,})"
        )
        .unwrap()
    })
}

/// Matches an equipment type / description label.
fn type_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(?:description|equipment\s+type|product\s+type|unit\s+type)\s*[:\-]?\s*(\S[\S\s]{0,60}?)(?:\s*$|\s{2,})"
        )
        .unwrap()
    })
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Extract a [`UnitHeader`] from the first pages of a unit.
///
/// Scans the first `header_page_limit` pages (default 2) for structured header
/// fields: tag, model, manufacturer, and equipment type.  Uses a priority order:
/// first exact match wins; flexible matches fill remaining empty fields.
///
/// # Arguments
///
/// * `pages` — The unit's pages (pre-sliced to `unit.start_page..=unit.end_page`).
/// * `header_page_limit` — How many leading pages to scan (usually 2).
///
/// # Confidence
///
/// The returned `confidence` is the average of all field confidences that were
/// successfully extracted, or `0.0` when no fields are found.
#[must_use]
pub fn extract_unit_header(pages: &[&Page], header_page_limit: usize) -> UnitHeader {
    let scan_pages = pages.iter().take(header_page_limit.max(1));

    let mut unit_tag: Option<(String, f64)> = None;
    let mut model: Option<(String, f64)> = None;
    let mut manufacturer: Option<(String, f64)> = None;
    let mut item_type: Option<(String, f64)> = None;

    for page in scan_pages {
        // Build a per-line text representation for regex matching.
        // Each span contributes its text; spans are already sorted by (y, x).
        let page_idx = page.page_index();

        for span in page.spans() {
            let text = span.text.trim();
            if text.is_empty() {
                continue;
            }
            let _ = page_idx; // used for future provenance; suppress warning

            // Try to fill tag.
            if unit_tag.is_none() {
                if let Some(caps) = tag_re().captures(text) {
                    let val = normalise_value(&caps[1]);
                    if !val.is_empty() {
                        let conf = choose_conf(text, &val, CONF_EXACT, CONF_FLEX);
                        unit_tag = Some((val, conf));
                    }
                }
            }

            // Try to fill model.
            if model.is_none() {
                if let Some(caps) = model_re().captures(text) {
                    let val = normalise_value(&caps[1]);
                    if val.len() >= MIN_VALUE_LEN {
                        let conf = choose_conf(text, &val, CONF_EXACT, CONF_FLEX);
                        model = Some((val, conf));
                    }
                }
            }

            // Try to fill manufacturer.
            if manufacturer.is_none() {
                if let Some(caps) = manufacturer_re().captures(text) {
                    let val = normalise_value(&caps[1]);
                    if val.len() >= MIN_VALUE_LEN {
                        let conf = choose_conf(text, &val, CONF_EXACT, CONF_FLEX);
                        manufacturer = Some((val, conf));
                    }
                }
            }

            // Try to fill item_type.
            if item_type.is_none() {
                if let Some(caps) = type_re().captures(text) {
                    let val = normalise_value(&caps[1]);
                    if val.len() >= MIN_VALUE_LEN {
                        item_type = Some((val, CONF_FLEX));
                    }
                }
            }
        }
    }

    // Compute aggregate confidence.
    let fields: &[Option<&(String, f64)>] = &[
        unit_tag.as_ref(),
        model.as_ref(),
        manufacturer.as_ref(),
        item_type.as_ref(),
    ];
    let extracted: Vec<f64> = fields.iter().filter_map(|f| f.map(|(_, c)| *c)).collect();
    let confidence = if extracted.is_empty() {
        0.0
    } else {
        extracted.iter().sum::<f64>() / extracted.len() as f64
    };

    UnitHeader {
        unit_tag: unit_tag.map(|(v, _)| v),
        model: model.map(|(v, _)| v),
        manufacturer: manufacturer.map(|(v, _)| v),
        item_type: item_type.map(|(v, _)| v),
        confidence,
    }
}

/// Extract key-value pairs from all pages of a unit using a colon-split heuristic.
///
/// For each span whose text contains `:`, splits the text at the first colon
/// into a label and a value.  Applies filtering:
/// - Label must not be empty, must be ≤ `MAX_LABEL_LEN` characters.
/// - Value must be non-empty after trimming.
/// - Single-word values shorter than `MIN_VALUE_LEN` are kept (short values
///   like `"Yes"` are valid).
///
/// Results are returned in page order.  Duplicate `label`/`value` pairs within
/// a unit are not deduplicated here (deduplication happens in the export layer).
///
/// # Arguments
///
/// * `pages` — The unit's pages (pre-sliced to `unit.start_page..=unit.end_page`).
///
/// # Confidence
///
/// All KV pairs receive `CONF_KV = 0.7` because this is lower-precision
/// heuristic extraction (no label schema validation).
#[must_use]
pub fn extract_kv_pairs(pages: &[&Page]) -> Vec<KvPair> {
    let mut pairs: Vec<KvPair> = Vec::new();

    for page in pages {
        let page_idx = page.page_index();

        for span in page.spans() {
            let text = span.text.trim();

            // Only process spans containing a colon.
            let Some(colon_pos) = text.find(':') else {
                continue;
            };

            let label = text[..colon_pos].trim();
            let value = text[colon_pos + 1..].trim();

            // Filter out noise.
            if label.is_empty()
                || label.len() > MAX_LABEL_LEN
                || value.is_empty()
            {
                continue;
            }

            // Skip spans where the label looks like a URL or file path (heuristic).
            if label.contains("//") || label.starts_with("http") {
                continue;
            }

            let bbox = Some(TidyBBox {
                x: span.bbox.x,
                y: span.bbox.y,
                width: span.bbox.width,
                height: span.bbox.height,
            });

            pairs.push(KvPair {
                label: label.to_owned(),
                value: value.to_owned(),
                page: page_idx,
                bbox,
                confidence: CONF_KV,
            });
        }
    }

    pairs
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Normalise an extracted value: trim and collapse internal whitespace.
fn normalise_value(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Choose confidence based on whether the label uses an exact well-known form.
///
/// If the text contains a known exact label keyword (e.g. `"Tag No."`, `"Model No."`)
/// returns `exact`; otherwise returns `flex`.
fn choose_conf(text: &str, _value: &str, exact: f64, flex: f64) -> f64 {
    let up = text.to_ascii_uppercase();
    if up.contains("TAG NO")
        || up.contains("MODEL NO")
        || up.contains("MANUFACTURER:")
        || up.contains("MFR:")
        || up.contains("DESCRIPTION:")
    {
        exact
    } else {
        flex
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{BBox, Page, Span};

    fn make_span(text: &str, x: f64, y: f64) -> Span {
        let bbox = BBox::new(x, y, 0.2, 0.02).expect("valid bbox");
        Span::new(text, bbox, 10.0).expect("valid span")
    }

    fn make_page(idx: usize, spans: Vec<Span>) -> Page {
        let mut p = Page::new(idx, 100.0, 100.0).expect("valid page");
        for s in spans {
            p.add_span(s).expect("valid span");
        }
        p
    }

    // ── Header extraction tests ───────────────────────────────────────────────

    #[test]
    fn header_extracts_model_from_label_colon() {
        let page = make_page(0, vec![
            make_span("Model No.: CLCP036", 0.1, 0.1),
            make_span("Some other text", 0.1, 0.3),
        ]);
        let pages = vec![&page];
        let header = extract_unit_header(&pages, 2);
        assert_eq!(header.model.as_deref(), Some("CLCP036"));
        assert!(header.confidence > 0.0);
    }

    #[test]
    fn header_extracts_manufacturer_from_label() {
        let page = make_page(0, vec![
            make_span("Manufacturer: Carrier", 0.1, 0.15),
        ]);
        let pages = vec![&page];
        let header = extract_unit_header(&pages, 2);
        assert_eq!(header.manufacturer.as_deref(), Some("Carrier"));
    }

    #[test]
    fn header_extracts_tag_from_item_tag() {
        let page = make_page(0, vec![
            make_span("Tag No. AHU-1", 0.1, 0.1),
        ]);
        let pages = vec![&page];
        let header = extract_unit_header(&pages, 2);
        assert_eq!(header.unit_tag.as_deref(), Some("AHU-1"));
    }

    #[test]
    fn header_returns_zero_confidence_when_no_fields() {
        let page = make_page(0, vec![
            make_span("General product information", 0.1, 0.1),
        ]);
        let pages = vec![&page];
        let header = extract_unit_header(&pages, 2);
        assert_eq!(header.confidence, 0.0);
        assert!(header.model.is_none());
        assert!(header.manufacturer.is_none());
    }

    #[test]
    fn header_respects_page_limit() {
        // Model is on page 2 (index 1), but header_page_limit = 1 means we
        // only scan page 0 — model should not be found.
        let page0 = make_page(0, vec![make_span("Cover page", 0.1, 0.1)]);
        let page1 = make_page(1, vec![make_span("Model No.: CLCP036", 0.1, 0.1)]);
        let pages = vec![&page0, &page1];
        let header = extract_unit_header(&pages, 1);
        assert!(header.model.is_none(), "model should not be found with page_limit=1");
    }

    // ── KV pair extraction tests ──────────────────────────────────────────────

    #[test]
    fn kv_pairs_extracted_from_colon_spans() {
        let page = make_page(0, vec![
            make_span("Airflow: 4500 CFM", 0.1, 0.2),
            make_span("ESP: 0.75 in-wg", 0.1, 0.3),
            make_span("No colon here at all", 0.1, 0.4),
        ]);
        let pages = vec![&page];
        let pairs = extract_kv_pairs(&pages);

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].label, "Airflow");
        assert_eq!(pairs[0].value, "4500 CFM");
        assert_eq!(pairs[1].label, "ESP");
        assert_eq!(pairs[1].value, "0.75 in-wg");
        assert!((pairs[0].confidence - CONF_KV).abs() < 1e-9);
    }

    #[test]
    fn kv_pairs_skip_empty_value_after_colon() {
        let page = make_page(0, vec![
            make_span("Empty label:", 0.1, 0.2),
            make_span(": value with no label", 0.1, 0.3),
        ]);
        let pages = vec![&page];
        let pairs = extract_kv_pairs(&pages);
        assert!(pairs.is_empty(), "no valid pairs should be extracted from these spans");
    }

    #[test]
    fn kv_pairs_page_provenance_set_correctly() {
        let page = make_page(7, vec![make_span("CFM: 3600", 0.1, 0.2)]);
        let pages = vec![&page];
        let pairs = extract_kv_pairs(&pages);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].page, 7);
        assert!(pairs[0].bbox.is_some());
    }
}

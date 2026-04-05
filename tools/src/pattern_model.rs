//! Deterministic pattern and heuristic model for Phase 0.5 pattern development.
//!
//! This module is the canonical data model (Phase D output) used by `pattern-dev`.
//! It defines the types that flow through the single-PDF test loop, validate-corpus,
//! overlay rendering, and sidecar emission.
//!
//! # Confidence policy (from DEV_STANDARDS)
//!
//! | Range          | Behaviour                                               |
//! |----------------|---------------------------------------------------------|
//! | `< 0.80`       | Hard fail — emit [`FailureCode`], escalate to review    |
//! | `0.80 – 0.95`  | Pass with warning flag                                  |
//! | `>= 0.95`      | Proceed normally                                        |
//!
//! # Runtime-ready vs. schema-only families
//!
//! Families marked **runtime-ready** have working detection logic (Phase E+).
//! Families marked **schema-only** emit schema-complete sidecars with
//! `source = SourceTag::SchemaOnly` but no detection logic until Phase 1.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

// ── Confidence thresholds (locked, DEV_STANDARDS §Principle 3) ──────────────

/// Minimum confidence to proceed without a warning flag.
pub const CONFIDENCE_HIGH: f32 = 0.95;

/// Minimum confidence to proceed at all (emit with flag below this).
pub const CONFIDENCE_PASS: f32 = 0.80;

// ── HeuristicFamily ───────────────────────────────────────────────────────────

/// Every heuristic family recognized by `pattern-dev`.
///
/// This enum is the single source of truth for family identity — the clap CLI,
/// sidecars, overlays, and corpus manifests all use `HeuristicFamily::as_str()`.
///
/// Ordering is stable and significant: it defines the default iteration order
/// for batch validation (deterministic per region requirements of the project).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum HeuristicFamily {
    /// Regex-based spec footer section-ID detection.
    ///
    /// Matches CSI MasterFormat section IDs (e.g., `23 82 16`) in the footer
    /// band of specification pages. **Runtime-ready (Phase E).**
    FooterSectionId,

    /// Footer page-in-section counter detection (`Page N of M`).
    ///
    /// Matches `Page N of M` / `PAGE N OF M` patterns in the footer band.
    /// **Runtime-ready (Phase E).**
    PageCounter,

    /// Top-band text/logo presence heuristic.
    ///
    /// Detects non-empty spans in the header band as evidence of a running
    /// header (project name, firm name, logo area). **Runtime-ready (Phase E).**
    HeaderBand,

    /// Drawing corner-band title-block candidate generation.
    ///
    /// Scores each corner band for table-structure density (axis-aligned lines,
    /// intersections, rectangular cells). **Schema-only — runtime in Phase 1.**
    TitleBlockAnchor,

    /// Autonomous ROI ranking evidence.
    ///
    /// Captures candidate regions and their ranking scores for the deterministic
    /// ROI selection policy. **Schema-only — runtime in Phase 1.**
    RoiCandidate,

    /// Spec heading line-feature diagnostics.
    ///
    /// Records font-delta, caps-ratio, indent, and leading-gap features for
    /// heading candidate spans. **Schema-only — runtime in Phase 1.**
    SpecHeading,
}

impl HeuristicFamily {
    /// Stable kebab-case string identifier used in filenames and sidecars.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FooterSectionId => "footer-section-id",
            Self::PageCounter => "page-counter",
            Self::HeaderBand => "header-band",
            Self::TitleBlockAnchor => "title-block-anchor",
            Self::RoiCandidate => "roi-candidate",
            Self::SpecHeading => "spec-heading",
        }
    }

    /// Returns `true` when this family has working detection logic (Phase E+).
    ///
    /// Returns `false` for schema-only families that emit structurally correct
    /// sidecars but whose runtime detection is deferred to Phase 1.
    #[must_use]
    pub fn is_runtime_ready(&self) -> bool {
        matches!(self, Self::FooterSectionId | Self::PageCounter | Self::HeaderBand)
    }

    /// All families in stable iteration order.
    ///
    /// Used in corpus-wide validation loops (Phase I+).
    #[allow(dead_code)]
    #[must_use]
    pub fn all() -> &'static [HeuristicFamily] {
        &[
            Self::FooterSectionId,
            Self::PageCounter,
            Self::HeaderBand,
            Self::TitleBlockAnchor,
            Self::RoiCandidate,
            Self::SpecHeading,
        ]
    }
}

impl fmt::Display for HeuristicFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── RegionBand ─────────────────────────────────────────────────────────────────

/// A horizontal band on the page defined in normalized coordinates [0.0, 1.0],
/// top-left origin (y=0.0 = top of page, y=1.0 = bottom of page).
///
/// Used to constrain pattern matching to a specific region of the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionBand {
    /// Top edge of the band (inclusive), normalized [0.0, 1.0].
    pub y_min: f32,
    /// Bottom edge of the band (inclusive), normalized [0.0, 1.0].
    pub y_max: f32,
}

impl RegionBand {
    /// Creates a new region band.
    ///
    /// # Panics
    ///
    /// Panics in debug builds when `y_min >= y_max` or either value is out of
    /// [0.0, 1.0]. These are programming errors, not runtime inputs.
    #[must_use]
    pub fn new(y_min: f32, y_max: f32) -> Self {
        debug_assert!(y_min >= 0.0 && y_min < y_max && y_max <= 1.0,
            "RegionBand({y_min}, {y_max}): must satisfy 0.0 <= y_min < y_max <= 1.0");
        Self { y_min, y_max }
    }

    /// Standard spec/drawing footer band: bottom 15 % of the page.
    #[must_use]
    pub fn footer() -> Self {
        Self::new(0.85, 1.0)
    }

    /// Standard spec/drawing header band: top 15 % of the page.
    #[must_use]
    pub fn header() -> Self {
        Self::new(0.0, 0.15)
    }

    /// Body region: everything between header and footer bands.
    #[must_use]
    pub fn body() -> Self {
        Self::new(0.15, 0.85)
    }

    /// Full page (no band restriction).
    #[must_use]
    pub fn full_page() -> Self {
        Self::new(0.0, 1.0)
    }

    /// Returns `true` when `y` (normalized, top-left origin) falls within this band.
    #[must_use]
    pub fn contains(&self, y: f32) -> bool {
        y >= self.y_min && y <= self.y_max
    }
}

// ── NormalizedBBox ────────────────────────────────────────────────────────────

/// A bounding box in normalized coordinates [0.0, 1.0], top-left origin.
///
/// Matches the coordinate system of `conset_pdf_ir::BBox`. Coordinates are
/// produced by applying `conset_pdf_ir::normalize_bbox` to the raw PDF points
/// from `conset_pdf_extraction::RawBBox`.
///
/// Used in `MatchEvidence` and serialized directly into per-page sidecar JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedBBox {
    /// Left edge, normalized [0.0, 1.0].
    pub x: f32,
    /// Top edge, normalized [0.0, 1.0] (top-left origin).
    pub y: f32,
    /// Width, normalized [0.0, 1.0].
    pub width: f32,
    /// Height, normalized [0.0, 1.0].
    pub height: f32,
}

impl NormalizedBBox {
    /// Convert from raw PDF coordinates to normalized top-left origin.
    ///
    /// Applies the same formula as `conset_pdf_ir::normalize_bbox`:
    /// - `x_norm = x_pdf / page_width`
    /// - `y_norm = 1.0 - (y_pdf + height_pdf) / page_height`
    /// - `width_norm = width_pdf / page_width`
    /// - `height_norm = height_pdf / page_height`
    ///
    /// Returns `None` when page dimensions are zero or the box extends beyond
    /// page bounds (would produce values outside [0.0, 1.0]).
    #[must_use]
    pub fn from_raw(x: f32, y: f32, width: f32, height: f32, page_w: f32, page_h: f32)
        -> Option<Self>
    {
        if page_w <= 0.0 || page_h <= 0.0 {
            return None;
        }
        let x_norm = x / page_w;
        let y_norm = 1.0 - (y + height) / page_h;
        let w_norm = width / page_w;
        let h_norm = height / page_h;
        // Clamp minor floating-point drift to [0.0, 1.0] rather than rejecting.
        Some(Self {
            x: x_norm.clamp(0.0, 1.0),
            y: y_norm.clamp(0.0, 1.0),
            width: w_norm.clamp(0.0, 1.0),
            height: h_norm.clamp(0.0, 1.0),
        })
    }

    /// Returns the normalized y-coordinate of the top edge (same as `y`).
    ///
    /// Used in Phase F overlay rendering to position annotation boxes.
    #[allow(dead_code)]
    #[must_use]
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the normalized y-coordinate of the bottom edge.
    ///
    /// Used in Phase F overlay rendering.
    #[allow(dead_code)]
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns the midpoint y-coordinate (top-left origin).
    #[must_use]
    pub fn mid_y(&self) -> f32 {
        self.y + self.height * 0.5
    }
}

// ── FailureCode ───────────────────────────────────────────────────────────────

/// Deterministic reason codes for why a pattern match failed or was flagged.
///
/// These codes are surfaced in sidecar JSON and overlays, and will drive the
/// exception-triage queue in later phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureCode {
    /// No span in the target region matched the pattern.
    NoMatch,
    /// A candidate was found but its confidence is below [`CONFIDENCE_PASS`].
    LowConfidence,
    /// The matched span was found but lies outside the required region band.
    RegionMiss,
    /// Two or more candidates scored within tie-break margin; cannot choose.
    AmbiguousTie,
}

impl fmt::Display for FailureCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NoMatch => "NO_MATCH",
            Self::LowConfidence => "LOW_CONFIDENCE",
            Self::RegionMiss => "REGION_MISS",
            Self::AmbiguousTie => "AMBIGUOUS_TIE",
        };
        f.write_str(s)
    }
}

// ── SourceTag ─────────────────────────────────────────────────────────────────

/// Indicates how the evidence in a `MatchEvidence` was produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceTag {
    /// Evidence came from the vector text layer (PDFium text objects).
    Vector,
    /// OCR-derived text — deferred to Phase 1 (G-027).
    #[allow(dead_code)]
    Ocr,
    /// Sidecar was emitted for schema validation only; no detection ran.
    /// Used by schema-only families (TitleBlockAnchor, RoiCandidate, SpecHeading)
    /// until Phase 1 implements their runtime detection.
    SchemaOnly,
}

// ── MatchEvidence ─────────────────────────────────────────────────────────────

/// The result of applying one `PatternSpec` to one page.
///
/// Every field maps directly to a key in the locked sidecar JSON schema
/// (version `0.5.0`). Field names and types must not change without a
/// schema version bump.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchEvidence {
    /// Schema version — locked at `"0.5.0"`.
    pub schema_version: &'static str,

    /// Source PDF path (as provided to the command).
    pub pdf_path: String,

    /// Zero-based page index.
    pub page_index: usize,

    /// Heuristic family that produced this result.
    pub family: String,

    /// Matched text spans, in page-order. Empty when no match was found.
    pub matched_spans: Vec<MatchedSpan>,

    /// Confidence in [0.0, 1.0], or `None` when detection did not run
    /// (schema-only families or Phase E placeholder).
    pub confidence: Option<f32>,

    /// Why the match failed, or `None` on success.
    pub failure_reason: Option<FailureCode>,

    /// Human-readable explanation of which branch the detection logic took.
    pub branch_reason: String,

    /// How the evidence was produced.
    pub source: SourceTag,

    /// Engine version (`CARGO_PKG_VERSION` of the `classify-pdf` package).
    pub engine_version: &'static str,

    /// Pattern model version — `"0.5.0"`.
    pub pattern_version: &'static str,
}

impl MatchEvidence {
    /// Schema version constant for sidecar JSON (locked from Phase F).
    pub const SCHEMA_VERSION: &'static str = "0.5.0";

    /// Pattern model version constant.
    pub const PATTERN_VERSION: &'static str = "0.5.0";

    /// Creates a placeholder (not-yet-implemented) evidence record.
    ///
    /// Used by Phase E before real detection is wired, and by schema-only
    /// families. Sets `failure_reason = None` and `confidence = None` since
    /// detection has not run.
    #[must_use]
    pub fn placeholder(pdf_path: String, page_index: usize, family: &HeuristicFamily) -> Self {
        let source = if family.is_runtime_ready() {
            SourceTag::Vector
        } else {
            SourceTag::SchemaOnly
        };
        Self {
            schema_version: Self::SCHEMA_VERSION,
            pdf_path,
            page_index,
            family: family.as_str().to_owned(),
            matched_spans: Vec::new(),
            confidence: None,
            failure_reason: None,
            branch_reason: if family.is_runtime_ready() {
                "detection not yet implemented — Phase E".to_owned()
            } else {
                "schema-only family — runtime detection deferred to Phase 1".to_owned()
            },
            source,
            engine_version: env!("CARGO_PKG_VERSION"),
            pattern_version: Self::PATTERN_VERSION,
        }
    }

    /// Returns `true` when this evidence represents a successful match at or
    /// above the high-confidence threshold.
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.failure_reason.is_none()
            && self.confidence.is_some_and(|c| c >= CONFIDENCE_HIGH)
    }

    /// Returns `true` when the match passed but with a warning flag.
    #[must_use]
    pub fn is_flagged(&self) -> bool {
        self.failure_reason.is_none()
            && self.confidence.is_some_and(|c| (CONFIDENCE_PASS..CONFIDENCE_HIGH).contains(&c))
    }

    /// Returns `true` when the match failed (below pass threshold or no match).
    #[must_use]
    pub fn is_failure(&self) -> bool {
        self.failure_reason.is_some()
            || self.confidence.is_some_and(|c| c < CONFIDENCE_PASS)
    }
}

/// A single matched text span within a `MatchEvidence`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedSpan {
    /// The matched text content.
    pub text: String,
    /// Normalized bounding box (top-left origin, [0.0, 1.0]).
    pub bbox: NormalizedBBox,
    /// Confidence contribution from this span (0.0–1.0).
    pub span_confidence: f32,
}

// ── PatternSpec ───────────────────────────────────────────────────────────────

/// Configuration for one heuristic family's detection policy.
///
/// This is the "compiled" policy for a family — containing the region band,
/// regex pattern string, confidence threshold, and version. Constructed via
/// [`PatternSpec::for_family`]; not persisted directly (sidecars use
/// [`MatchEvidence`]).
#[allow(dead_code)] // `family`, `confidence_threshold`, `version` consumed in Phase E+
#[derive(Debug, Clone)]
pub struct PatternSpec {
    /// Family this spec is configured for.
    pub family: HeuristicFamily,

    /// Optional regex pattern string. `None` for families that use pure
    /// geometric/structural heuristics rather than regex matching.
    ///
    /// The compiled [`regex::Regex`] is constructed on demand in Phase E
    /// matching functions, not stored here, to keep this type `Clone`.
    pub regex_pattern: Option<String>,

    /// Page region this pattern targets.
    pub region_band: RegionBand,

    /// Minimum confidence to emit a passing result (below this → failure code).
    pub confidence_threshold: f32,

    /// Pattern version string — must match the sidecar's `"pattern_version"` field.
    pub version: &'static str,
}

impl PatternSpec {
    /// Constructs the canonical `PatternSpec` for a given family.
    ///
    /// These configurations are the deterministic policy for Phase 0.5; they
    /// will evolve only through explicit versioned changes to this function.
    #[must_use]
    pub fn for_family(family: &HeuristicFamily) -> Self {
        match family {
            HeuristicFamily::FooterSectionId => Self {
                family: family.clone(),
                // CSI MasterFormat section IDs: two or three numeric groups
                // separated by spaces, e.g. "23 82 16" or "01 00 00".
                // Anchored to word boundaries to avoid partial matches.
                regex_pattern: Some(
                    r"\b\d{2}\s+\d{2}(?:\s+\d{2})?\b".to_owned(),
                ),
                region_band: RegionBand::footer(),
                confidence_threshold: CONFIDENCE_PASS,
                version: MatchEvidence::PATTERN_VERSION,
            },
            HeuristicFamily::PageCounter => Self {
                family: family.clone(),
                // Matches "Page 2 of 3", "PAGE 2 OF 3", "page 2 of 3"
                // Allows optional leading context (e.g. "- Page 2 of 3").
                regex_pattern: Some(
                    r"(?i)\bpage\s+(\d+)\s+of\s+(\d+)\b".to_owned(),
                ),
                region_band: RegionBand::footer(),
                confidence_threshold: CONFIDENCE_PASS,
                version: MatchEvidence::PATTERN_VERSION,
            },
            HeuristicFamily::HeaderBand => Self {
                family: family.clone(),
                // Header band detection is purely geometric (non-empty spans in
                // header band). No regex needed.
                regex_pattern: None,
                region_band: RegionBand::header(),
                confidence_threshold: CONFIDENCE_PASS,
                version: MatchEvidence::PATTERN_VERSION,
            },
            // Schema-only families: full-page band, no regex.
            // Runtime detection deferred to Phase 1 (G-018, G-019, G-020, G-022).
            HeuristicFamily::TitleBlockAnchor => Self {
                family: family.clone(),
                regex_pattern: None,
                region_band: RegionBand::full_page(),
                confidence_threshold: CONFIDENCE_PASS,
                version: MatchEvidence::PATTERN_VERSION,
            },
            HeuristicFamily::RoiCandidate => Self {
                family: family.clone(),
                regex_pattern: None,
                region_band: RegionBand::full_page(),
                confidence_threshold: CONFIDENCE_PASS,
                version: MatchEvidence::PATTERN_VERSION,
            },
            HeuristicFamily::SpecHeading => Self {
                family: family.clone(),
                regex_pattern: None,
                region_band: RegionBand::body(),
                confidence_threshold: CONFIDENCE_PASS,
                version: MatchEvidence::PATTERN_VERSION,
            },
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_families_have_a_spec() {
        for family in HeuristicFamily::all() {
            let spec = PatternSpec::for_family(family);
            assert_eq!(&spec.family, family);
            // Runtime-ready families must have a regex or a defined region band.
            if family.is_runtime_ready() {
                // Header band is geometric only; others need regex.
                if *family != HeuristicFamily::HeaderBand {
                    assert!(
                        spec.regex_pattern.is_some(),
                        "{family} is runtime-ready but has no regex pattern"
                    );
                }
            }
        }
    }

    #[test]
    fn family_as_str_round_trips_via_serde() {
        for family in HeuristicFamily::all() {
            let json = serde_json::to_string(family).unwrap();
            let back: HeuristicFamily = serde_json::from_str(&json).unwrap();
            assert_eq!(family, &back);
        }
    }

    #[test]
    fn region_band_contains() {
        let footer = RegionBand::footer();
        assert!(footer.contains(0.90));
        assert!(!footer.contains(0.50));
        assert!(footer.contains(0.85));
        assert!(footer.contains(1.0));
    }

    #[test]
    fn normalized_bbox_from_raw_letter_page() {
        // 612×792 pt page, bbox at bottom-left spanning 100×50 pts
        let bbox = NormalizedBBox::from_raw(0.0, 0.0, 100.0, 50.0, 612.0, 792.0).unwrap();
        // x_norm = 0.0 / 612 = 0.0
        assert!((bbox.x - 0.0).abs() < 1e-4);
        // y_norm = 1.0 - (0 + 50) / 792 = ~0.9369
        assert!((bbox.y - (1.0 - 50.0 / 792.0)).abs() < 1e-4);
        assert!((bbox.width - 100.0 / 612.0).abs() < 1e-4);
        assert!((bbox.height - 50.0 / 792.0).abs() < 1e-4);
    }

    #[test]
    fn normalized_bbox_zero_page_returns_none() {
        assert!(NormalizedBBox::from_raw(0.0, 0.0, 10.0, 10.0, 0.0, 792.0).is_none());
        assert!(NormalizedBBox::from_raw(0.0, 0.0, 10.0, 10.0, 612.0, 0.0).is_none());
    }

    #[test]
    fn match_evidence_placeholder_source_tag() {
        let rt = MatchEvidence::placeholder(
            "test.pdf".to_owned(), 0, &HeuristicFamily::FooterSectionId,
        );
        assert!(matches!(rt.source, SourceTag::Vector));
        assert!(rt.confidence.is_none());

        let schema_only = MatchEvidence::placeholder(
            "test.pdf".to_owned(), 0, &HeuristicFamily::TitleBlockAnchor,
        );
        assert!(matches!(schema_only.source, SourceTag::SchemaOnly));
    }

    #[test]
    fn failure_code_display() {
        assert_eq!(FailureCode::NoMatch.to_string(), "NO_MATCH");
        assert_eq!(FailureCode::LowConfidence.to_string(), "LOW_CONFIDENCE");
        assert_eq!(FailureCode::RegionMiss.to_string(), "REGION_MISS");
        assert_eq!(FailureCode::AmbiguousTie.to_string(), "AMBIGUOUS_TIE");
    }

    #[test]
    fn footer_section_id_regex_matches_csi_ids() {
        use regex::Regex;
        let spec = PatternSpec::for_family(&HeuristicFamily::FooterSectionId);
        let re = Regex::new(spec.regex_pattern.as_ref().unwrap()).unwrap();
        assert!(re.is_match("23 82 16 – HEATING WATER COILS - Page 2 of 3"));
        assert!(re.is_match("01 00 00 – SUMMARY"));
        assert!(re.is_match("09 90 00"));
        // Should not match purely numeric strings that aren't CSI IDs
        assert!(!re.is_match("page 2 of 3"));
    }

    #[test]
    fn page_counter_regex_case_insensitive() {
        use regex::Regex;
        let spec = PatternSpec::for_family(&HeuristicFamily::PageCounter);
        let re = Regex::new(spec.regex_pattern.as_ref().unwrap()).unwrap();
        assert!(re.is_match("Page 2 of 3"));
        assert!(re.is_match("PAGE 14 OF 32"));
        assert!(re.is_match("page 1 of 1"));
        assert!(!re.is_match("23 82 16 section"));
    }
}

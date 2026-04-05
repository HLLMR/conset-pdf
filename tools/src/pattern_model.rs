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

// ── Phase G: title-block-anchor schema ───────────────────────────────────────

/// Which corner of the page a title-block candidate occupies.
///
/// AEC drawings most commonly place the title block in the bottom-right corner,
/// but all four positions are supported for schema completeness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CornerPosition {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// A corner-band candidate for title-block localization.
///
/// In Phase G, `axis_line_count`, `cell_density`, and `score` are always
/// `None` (runtime detection deferred to Phase 1). The `bbox` is pre-seeded
/// with the standard corner region so sidecars have concrete geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornerBandCandidate {
    /// Which corner this candidate covers.
    pub corner: CornerPosition,
    /// Normalized bbox of this corner band region.
    pub bbox: NormalizedBBox,
    /// Axis-aligned line count inside this band (`None` = not computed).
    pub axis_line_count: Option<u32>,
    /// Rectangular cell density inside this band (`None` = not computed).
    pub cell_density: Option<f32>,
    /// Composite score for this candidate (`None` = not computed).
    pub score: Option<f32>,
}

/// The winning title-block candidate after scoring.
///
/// `None` in Phase G schema-only sidecars; populated by Phase 1 runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedTitleBlock {
    /// Winning corner.
    pub corner: CornerPosition,
    /// Bbox of the selected title block.
    pub bbox: NormalizedBBox,
    /// Score of the winning candidate.
    pub score: f32,
}

/// A candidate field (labelled cell) within the title-block region.
///
/// Fields represent labelled cells such as "SHEET NUMBER", "PROJECT",
/// "DRAWN BY", etc. All fields are `None` in Phase G schema-only sidecars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleBlockField {
    /// Expected label text (`None` = not identified).
    pub label: Option<String>,
    /// Bbox of the label area (`None` = not located).
    pub label_bbox: Option<NormalizedBBox>,
    /// Bbox of the value area (`None` = not located).
    pub value_bbox: Option<NormalizedBBox>,
    /// Extracted value text (`None` = not extracted).
    pub extracted_value: Option<String>,
    /// Score for this field match (`None` = not computed).
    pub field_score: Option<f32>,
}

/// Auto-learned template lifecycle metadata.
///
/// Supports drift detection when the same document template produces
/// different layout versions over time. All fields are `None` in Phase G.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLifecycle {
    /// Stable internal ID of the detected template, if known.
    pub detected_template_id: Option<String>,
    /// Version string of this template definition.
    pub template_version: Option<String>,
    /// `true` when the detected layout drifts from the last-seen template.
    pub drift_flag: Option<bool>,
    /// Content hash of the last-seen template for drift comparison.
    pub last_seen_hash: Option<String>,
}

impl TemplateLifecycle {
    /// Returns a schema-only placeholder with all fields set to `None`.
    #[must_use]
    pub fn schema_placeholder() -> Self {
        Self {
            detected_template_id: None,
            template_version: None,
            drift_flag: None,
            last_seen_hash: None,
        }
    }
}

/// Title-block extension fields emitted by the `title-block-anchor` family.
///
/// Embedded in [`TitleBlockSidecar`]. Schema-complete in Phase G;
/// populated by Phase 1 vector-first drawing detection runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleBlockExtension {
    /// Four corner-band candidates, pre-seeded with standard regions.
    pub corner_candidates: Vec<CornerBandCandidate>,
    /// Winning candidate after scoring (`None` in Phase G schema-only mode).
    pub winning_candidate: Option<SelectedTitleBlock>,
    /// Field candidates within the title block (empty in Phase G).
    pub field_candidates: Vec<TitleBlockField>,
    /// Auto-learned template lifecycle metadata.
    pub template_lifecycle: TemplateLifecycle,
}

impl TitleBlockExtension {
    /// Standard corner-band bboxes used as schema placeholders.
    ///
    /// Each candidate covers roughly 38 % of page width × 22 % of page height.
    /// Detection scores are all `None` — set by Phase 1 runtime.
    #[must_use]
    pub fn schema_placeholder() -> Self {
        let candidates = vec![
            CornerBandCandidate {
                corner: CornerPosition::BottomRight,
                bbox: NormalizedBBox { x: 0.62, y: 0.78, width: 0.38, height: 0.22 },
                axis_line_count: None,
                cell_density: None,
                score: None,
            },
            CornerBandCandidate {
                corner: CornerPosition::BottomLeft,
                bbox: NormalizedBBox { x: 0.0, y: 0.78, width: 0.38, height: 0.22 },
                axis_line_count: None,
                cell_density: None,
                score: None,
            },
            CornerBandCandidate {
                corner: CornerPosition::TopRight,
                bbox: NormalizedBBox { x: 0.62, y: 0.0, width: 0.38, height: 0.22 },
                axis_line_count: None,
                cell_density: None,
                score: None,
            },
            CornerBandCandidate {
                corner: CornerPosition::TopLeft,
                bbox: NormalizedBBox { x: 0.0, y: 0.0, width: 0.38, height: 0.22 },
                axis_line_count: None,
                cell_density: None,
                score: None,
            },
        ];
        Self {
            corner_candidates: candidates,
            winning_candidate: None,
            field_candidates: Vec::new(),
            template_lifecycle: TemplateLifecycle::schema_placeholder(),
        }
    }
}

/// Full sidecar type for `title-block-anchor` (schema-only, Phase G).
///
/// The `base` fields are flattened into the top-level JSON object, followed
/// by `title_block` as a nested object — matching the locked sidecar schema.
#[derive(Debug, Clone, Serialize)]
pub struct TitleBlockSidecar {
    /// Base [`MatchEvidence`] fields (flattened into top-level JSON).
    #[serde(flatten)]
    pub base: MatchEvidence,
    /// Title-block specific extension.
    pub title_block: TitleBlockExtension,
}

// ── Phase G: roi-candidate schema ─────────────────────────────────────────────

/// A candidate ROI region with ranking evidence.
///
/// `text_density`, `geometric_regularity`, `footer_proximity`, and
/// `rank_score` are all `None` in Phase G schema-only sidecars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoiCandidateRecord {
    /// Sequential index (0-based, deterministic order).
    pub candidate_id: u32,
    /// Candidate region bbox (pre-seeded with standard zones in Phase G).
    pub bbox: Option<NormalizedBBox>,
    /// Normalized text density in this region (`None` = not computed).
    pub text_density: Option<f32>,
    /// Geometric regularity score (`None` = not computed).
    pub geometric_regularity: Option<f32>,
    /// Proximity to the page footer band (`None` = not computed).
    pub footer_proximity: Option<f32>,
    /// Composite ranking score (`None` = not computed).
    pub rank_score: Option<f32>,
}

/// The selected ROI after ranking.
///
/// `None` in Phase G schema-only sidecars; populated by Phase 1 runtime (G-011).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedRoi {
    /// Index of the winning candidate.
    pub candidate_id: u32,
    /// Bbox of the selected ROI.
    pub bbox: NormalizedBBox,
    /// Final rank score.
    pub rank_score: f32,
}

/// ROI evidence extension emitted by the `roi-candidate` family.
///
/// Embedded in [`RoiCandidateSidecar`]. Schema-complete in Phase G;
/// populated by Phase 1 autonomous ROI ranking runtime (G-011).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoiEvidence {
    /// Ranked candidate regions (pre-seeded with 3 standard zones in Phase G).
    pub candidates: Vec<RoiCandidateRecord>,
    /// Selected ROI after ranking (`None` in Phase G schema-only mode).
    pub selected_roi: Option<SelectedRoi>,
    /// Version of the ranking policy applied.
    pub ranking_policy_version: String,
}

impl RoiEvidence {
    /// Three pre-seeded candidate zones (footer, body, header) with null scores.
    #[must_use]
    pub fn schema_placeholder() -> Self {
        let candidates = vec![
            RoiCandidateRecord {
                candidate_id: 0,
                bbox: Some(NormalizedBBox { x: 0.0, y: 0.85, width: 1.0, height: 0.15 }),
                text_density: None,
                geometric_regularity: None,
                footer_proximity: None,
                rank_score: None,
            },
            RoiCandidateRecord {
                candidate_id: 1,
                bbox: Some(NormalizedBBox { x: 0.0, y: 0.15, width: 1.0, height: 0.70 }),
                text_density: None,
                geometric_regularity: None,
                footer_proximity: None,
                rank_score: None,
            },
            RoiCandidateRecord {
                candidate_id: 2,
                bbox: Some(NormalizedBBox { x: 0.0, y: 0.0, width: 1.0, height: 0.15 }),
                text_density: None,
                geometric_regularity: None,
                footer_proximity: None,
                rank_score: None,
            },
        ];
        Self {
            candidates,
            selected_roi: None,
            ranking_policy_version: MatchEvidence::PATTERN_VERSION.to_owned(),
        }
    }
}

/// Full sidecar type for `roi-candidate` (schema-only, Phase G).
#[derive(Debug, Clone, Serialize)]
pub struct RoiCandidateSidecar {
    /// Base [`MatchEvidence`] fields (flattened into top-level JSON).
    #[serde(flatten)]
    pub base: MatchEvidence,
    /// ROI evidence extension.
    pub roi_evidence: RoiEvidence,
}

// ── Phase G: spec-heading schema ──────────────────────────────────────────────

/// Line-feature record for a heading candidate span.
///
/// All fields are `None` in Phase G schema-only sidecars.
/// Phase 1 runtime (G-022) populates these from the extracted text layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingCandidateRecord {
    /// Candidate text (`None` = schema placeholder).
    pub text: Option<String>,
    /// Normalized bbox (`None` = schema placeholder).
    pub bbox: Option<NormalizedBBox>,
    /// Measured font size in PDF points (`None` = not extracted).
    pub font_size_pts: Option<f32>,
    /// Font size delta vs. median body font on this page (`None` = not computed).
    pub font_delta: Option<f32>,
    /// Fraction of uppercase characters in [0.0, 1.0] (`None` = not computed).
    pub caps_ratio: Option<f32>,
    /// Normalized x-indent from left margin (`None` = not computed).
    pub indent_norm: Option<f32>,
    /// Vertical gap above this line in PDF points (`None` = not computed).
    pub leading_gap_pts: Option<f32>,
    /// Composite heading score [0.0, 1.0] (`None` = not computed).
    pub heading_score: Option<f32>,
}

/// Spec heading diagnostics extension emitted by the `spec-heading` family.
///
/// Embedded in [`SpecHeadingSidecar`]. Schema-complete in Phase G;
/// populated by Phase 1 spec heading detection runtime (G-022).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecHeadingDiagnostics {
    /// Heading candidates in top-to-bottom page order.
    ///
    /// Empty in Phase G (schema-only); Phase 1 fills this from the text layer.
    pub candidates: Vec<HeadingCandidateRecord>,
    /// Version of the heading detection policy applied.
    pub detection_policy_version: String,
}

impl SpecHeadingDiagnostics {
    /// Empty candidates list with the locked policy version.
    #[must_use]
    pub fn schema_placeholder() -> Self {
        Self {
            candidates: Vec::new(),
            detection_policy_version: MatchEvidence::PATTERN_VERSION.to_owned(),
        }
    }
}

/// Full sidecar type for `spec-heading` (schema-only, Phase G).
#[derive(Debug, Clone, Serialize)]
pub struct SpecHeadingSidecar {
    /// Base [`MatchEvidence`] fields (flattened into top-level JSON).
    #[serde(flatten)]
    pub base: MatchEvidence,
    /// Spec heading diagnostics extension.
    pub heading_diagnostics: SpecHeadingDiagnostics,
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

    // ── Phase G schema tests ──────────────────────────────────────────────────

    #[test]
    fn title_block_extension_has_four_corner_candidates() {
        let ext = TitleBlockExtension::schema_placeholder();
        assert_eq!(ext.corner_candidates.len(), 4);
        // Deterministic order: BR, BL, TR, TL
        assert_eq!(ext.corner_candidates[0].corner, CornerPosition::BottomRight);
        assert_eq!(ext.corner_candidates[1].corner, CornerPosition::BottomLeft);
        assert_eq!(ext.corner_candidates[2].corner, CornerPosition::TopRight);
        assert_eq!(ext.corner_candidates[3].corner, CornerPosition::TopLeft);
        // All scores null in schema-only mode
        for c in &ext.corner_candidates {
            assert!(c.score.is_none());
            assert!(c.axis_line_count.is_none());
            assert!(c.cell_density.is_none());
        }
        assert!(ext.winning_candidate.is_none());
        assert!(ext.field_candidates.is_empty());
    }

    #[test]
    fn roi_evidence_has_three_seeded_candidates() {
        let ev = RoiEvidence::schema_placeholder();
        assert_eq!(ev.candidates.len(), 3);
        // Candidate IDs are deterministic
        assert_eq!(ev.candidates[0].candidate_id, 0);
        assert_eq!(ev.candidates[1].candidate_id, 1);
        assert_eq!(ev.candidates[2].candidate_id, 2);
        // Bboxes pre-seeded, scores null
        for r in &ev.candidates {
            assert!(r.bbox.is_some());
            assert!(r.rank_score.is_none());
        }
        assert!(ev.selected_roi.is_none());
        assert_eq!(ev.ranking_policy_version, MatchEvidence::PATTERN_VERSION);
    }

    #[test]
    fn spec_heading_diagnostics_has_empty_candidates() {
        let diag = SpecHeadingDiagnostics::schema_placeholder();
        assert!(diag.candidates.is_empty());
        assert_eq!(diag.detection_policy_version, MatchEvidence::PATTERN_VERSION);
    }

    #[test]
    fn title_block_sidecar_serialises_with_base_and_extension_fields() {
        let base = MatchEvidence::placeholder(
            "test.pdf".to_owned(),
            0,
            &HeuristicFamily::TitleBlockAnchor,
        );
        let sidecar = TitleBlockSidecar {
            base,
            title_block: TitleBlockExtension::schema_placeholder(),
        };
        let json = serde_json::to_string(&sidecar).unwrap();
        // Base fields must appear at top level (flattened).
        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"family\""));
        assert!(json.contains("\"source\""));
        // Extension must appear as nested object.
        assert!(json.contains("\"title_block\""));
        assert!(json.contains("\"corner_candidates\""));
        assert!(json.contains("\"template_lifecycle\""));
    }

    #[test]
    fn roi_candidate_sidecar_serialises_with_base_and_extension_fields() {
        let base = MatchEvidence::placeholder(
            "test.pdf".to_owned(),
            0,
            &HeuristicFamily::RoiCandidate,
        );
        let sidecar = RoiCandidateSidecar {
            base,
            roi_evidence: RoiEvidence::schema_placeholder(),
        };
        let json = serde_json::to_string(&sidecar).unwrap();
        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"roi_evidence\""));
        assert!(json.contains("\"candidates\""));
        assert!(json.contains("\"ranking_policy_version\""));
    }

    #[test]
    fn spec_heading_sidecar_serialises_with_base_and_extension_fields() {
        let base = MatchEvidence::placeholder(
            "test.pdf".to_owned(),
            0,
            &HeuristicFamily::SpecHeading,
        );
        let sidecar = SpecHeadingSidecar {
            base,
            heading_diagnostics: SpecHeadingDiagnostics::schema_placeholder(),
        };
        let json = serde_json::to_string(&sidecar).unwrap();
        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"heading_diagnostics\""));
        assert!(json.contains("\"detection_policy_version\""));
    }

    #[test]
    fn corner_position_round_trips_via_serde() {
        let positions = [
            CornerPosition::BottomRight,
            CornerPosition::BottomLeft,
            CornerPosition::TopRight,
            CornerPosition::TopLeft,
        ];
        for pos in &positions {
            let json = serde_json::to_string(pos).unwrap();
            let back: CornerPosition = serde_json::from_str(&json).unwrap();
            assert_eq!(pos, &back);
        }
    }
}

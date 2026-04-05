//! OCR routing manifest contract — Phase 0.5 contract-shaping.
//!
//! Defines per-page OCR routing decisions and the manifest that
//! aggregates them for a full document. The routing manifest is the
//! handoff point between the vector-first extraction path and the
//! OCR path (G-027).
//!
//! **CONTRACT-ONLY** — OCR runtime deferred to Band 2 (G-027).

use serde::{Deserialize, Serialize};

/// Schema version for OCR routing contract types.
pub const OCR_ROUTING_SCHEMA_VERSION: &str = "0.5.0";

// ── Source classification ──────────────────────────────────────────────────

/// Classification of the text source for a page or document.
///
/// Mirrors `SourceTag` in `pattern_model` at the contract layer so that
/// downstream consumers don't need to depend on the `tools` crate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageTextSource {
    /// Page contains extractable vector text.
    Vector,
    /// Page is a raster image — OCR required.
    Raster,
    /// Page has both vector text and embedded raster regions.
    Mixed,
    /// Source could not be determined.
    Unknown,
}

// ── Per-page routing decision ──────────────────────────────────────────────

/// OCR routing decision for a single page.
///
/// `text_quality_score` ranges [0.0, 1.0] (higher = better vector quality).
/// `requires_ocr` is the actionable flag for the Band 2 OCR runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrPageDecision {
    pub page_index: usize,
    /// Classified text source for this page.
    pub text_source: PageTextSource,
    /// Normalized text quality score (`None` = not computed).
    pub text_quality_score: Option<f32>,
    /// Whether OCR should be run on this page.
    pub requires_ocr: bool,
    /// Recommended OCR engine identifier (`None` = use project default).
    pub recommended_engine: Option<String>,
    /// Human-readable routing rationale.
    pub reason: Option<String>,
}

// ── Routing manifest ───────────────────────────────────────────────────────

/// Manifest of OCR routing decisions for every page of a document.
///
/// Produced once per intake document and consumed by the Band 2 OCR
/// runtime to decide which pages to process.
///
/// **CONTRACT-ONLY** — runtime deferred to Band 2 (G-027).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrRoutingManifest {
    pub schema_version: String,
    /// Intake bundle ID of the document this manifest covers.
    pub bundle_id: Option<String>,
    /// Source file path.
    pub source_path: String,
    /// Per-page decisions in page-index order.
    pub page_decisions: Vec<OcrPageDecision>,
    /// Version string of the OCR engine that will process this document.
    pub ocr_engine_version: Option<String>,
    /// Version of the routing policy that produced this manifest.
    pub routing_policy_version: String,
}

impl OcrRoutingManifest {
    /// Returns a schema-placeholder manifest with no page decisions.
    #[must_use]
    pub fn schema_placeholder(source_path: impl Into<String>) -> Self {
        Self {
            schema_version: OCR_ROUTING_SCHEMA_VERSION.to_owned(),
            bundle_id: None,
            source_path: source_path.into(),
            page_decisions: Vec::new(),
            ocr_engine_version: None,
            routing_policy_version: OCR_ROUTING_SCHEMA_VERSION.to_owned(),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_routing_manifest_placeholder_round_trips_via_serde() {
        let m = OcrRoutingManifest::schema_placeholder("tests/corpus/tier1/DWG.pdf");
        let json = serde_json::to_string_pretty(&m).unwrap();
        let back: OcrRoutingManifest = serde_json::from_str(&json).unwrap();
        assert!(back.page_decisions.is_empty());
        assert_eq!(back.schema_version, OCR_ROUTING_SCHEMA_VERSION);
    }

    #[test]
    fn page_text_source_round_trips_via_serde() {
        for src in [
            PageTextSource::Vector,
            PageTextSource::Raster,
            PageTextSource::Mixed,
            PageTextSource::Unknown,
        ] {
            let s = serde_json::to_string(&src).unwrap();
            let back: PageTextSource = serde_json::from_str(&s).unwrap();
            assert_eq!(back, src);
        }
    }

    #[test]
    fn ocr_page_decision_with_data_round_trips() {
        let decision = OcrPageDecision {
            page_index: 3,
            text_source: PageTextSource::Raster,
            text_quality_score: Some(0.12),
            requires_ocr: true,
            recommended_engine: Some("tesseract-5".to_owned()),
            reason: Some("text density below threshold".to_owned()),
        };
        let json = serde_json::to_string(&decision).unwrap();
        let back: OcrPageDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back.page_index, 3);
        assert!(back.requires_ocr);
        assert_eq!(back.text_source, PageTextSource::Raster);
    }
}

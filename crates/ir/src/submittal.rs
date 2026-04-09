//! Submittal-medium IR types for Phase 10 submittal data extraction.
//!
//! These types are the submittal analog of `DrawingIndex`/`SheetEntry` from
//! `crates/ir/src/drawing.rs`.  The key structural difference is that submittal
//! packets are segmented by *equipment unit* (identified by a unit tag such as
//! `"AHU-1"` or `"RTU-3"`) rather than by sheet ID or CSI section.
//!
//! # Schema
//!
//! All serde-serialisable types in this module use `schema_version = "1.0.0"`.
//! Breaking changes must bump the version and add a migration path.
//!
//! # Tidy-format export
//!
//! The canonical output of Phase 10 is the [`EquipmentDataset`], a flat
//! collection of [`TidyRow`] records.  Each row represents one extracted
//! field/value pair for one unit, with full provenance (page, bbox, confidence,
//! source).  This format matches the canonical schema defined in
//! `MASTER_PLAN.md §Submittals: Data Extraction`.

use serde::{Deserialize, Serialize};

// ── Coverage ──────────────────────────────────────────────────────────────────

/// Coverage summary for a [`SubmittalIndex`].
///
/// Mirrors the structure of `CoverageStats` in `crates/ir/src/segment.rs` but
/// uses submittal-centric terminology.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SubmittalCoverage {
    /// Total pages in the source document.
    pub total_pages: usize,
    /// Pages assigned to at least one [`UnitEntry`] (including cover pages).
    pub assigned_pages: usize,
    /// Pages not assigned to any unit (gap pages, appendices, etc.).
    pub unassigned_pages: usize,
    /// Fraction of pages assigned: `assigned_pages / total_pages` (0.0–1.0).
    pub coverage_ratio: f64,
    /// Number of detected units (excluding the cover-page pseudo-unit).
    pub unit_count: usize,
}

// ── Unit boundary ──────────────────────────────────────────────────────────────

/// One equipment unit detected in a submittal packet.
///
/// A unit typically begins on a header page where the unit tag appears in a
/// prominent (large) font, and ends on the page before the next unit header or
/// the end of the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitEntry {
    /// Unit tag as extracted (e.g. `"AHU-1"`, `"RTU-3"`, `"COVER"`).
    ///
    /// Cover pages use the synthetic tag `"COVER"`.  Units for which no tag was
    /// detected use `"UNKNOWN-<n>"` where `n` is the zero-based page index.
    pub unit_tag: String,
    /// Equipment model number, if extracted from the unit header.
    #[serde(default)]
    pub model: Option<String>,
    /// Manufacturer name, if extracted (e.g. `"Trane"`, `"Carrier"`).
    #[serde(default)]
    pub manufacturer: Option<String>,
    /// Human-readable equipment type (e.g. `"Air Handling Unit"`,
    /// `"Rooftop Unit"`), if extractable.
    #[serde(default)]
    pub item_type: Option<String>,
    /// Zero-based index of the first page belonging to this unit (inclusive).
    pub start_page: usize,
    /// Zero-based index of the last page belonging to this unit (inclusive).
    pub end_page: usize,
    /// Number of pages in this unit (`end_page - start_page + 1`).
    pub page_count: usize,
    /// `true` when this entry represents the submittal cover / table-of-contents
    /// pages rather than a specific equipment unit.
    pub is_cover: bool,
    /// Detection confidence (0.0–1.0).
    ///
    /// - `1.0`: unit tag found in a large-font header region on the boundary page.
    /// - `0.7–0.9`: tag found in body text on the boundary page.
    /// - `0.3`: fallback — no clear boundary detected; whole document treated as
    ///   one unit.
    pub confidence: f64,
}

// ── Submittal index ────────────────────────────────────────────────────────────

/// Index of all unit boundaries detected in one submittal packet.
///
/// Serialised to `submittal-index.json` by the `index-submittal` CLI subcommand
/// and consumed by `extract-submittal` to scope table/KV extraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmittalIndex {
    /// Schema version — currently `"1.0.0"`.
    pub schema_version: String,
    /// Submittal packet identifier (typically the base filename without extension).
    pub packet_name: String,
    /// Ordered list of detected unit boundaries (in page order, cover first if present).
    pub units: Vec<UnitEntry>,
    /// Coverage statistics for this packet.
    pub coverage: SubmittalCoverage,
}

impl SubmittalIndex {
    /// Total pages in the source document (convenience accessor).
    #[must_use]
    pub fn total_pages(&self) -> usize {
        self.coverage.total_pages
    }

    /// Number of detected equipment units (excluding the cover pseudo-unit).
    #[must_use]
    pub fn unit_count(&self) -> usize {
        self.coverage.unit_count
    }
}

// ── Tidy-row export ────────────────────────────────────────────────────────────

/// Canonical bounding box representation for tidy-row provenance.
///
/// Uses normalised [0.0, 1.0] page-relative coordinates (top-left origin),
/// matching the coordinate system used throughout the rest of the pipeline.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TidyBBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// One extracted field/value record in the canonical tidy format.
///
/// The 14 columns correspond exactly to the schema defined in
/// `MASTER_PLAN.md §Submittals: Data Extraction — Canonical Export Format`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TidyRow {
    /// Submittal packet identifier (e.g. `"SUB_PESH_BERG_TRANE-AHU"`).
    pub packet_name: String,
    /// Revision identifier or date string (empty if not available).
    #[serde(default)]
    pub revision_id: String,
    /// Equipment tag (e.g. `"AHU-1"`, `"RTU-3"`).
    pub item_tag: String,
    /// Equipment type (e.g. `"Air Handling Unit"`); empty if unknown.
    #[serde(default)]
    pub equipment_type: String,
    /// Spec section or category (e.g. `"15750"`, `"HVAC"`); empty if unknown.
    #[serde(default)]
    pub section: String,
    /// Field name / label (e.g. `"Cooling Airflow CFM"`, `"ESP"`)
    pub field: String,
    /// Raw extracted text value, verbatim from the PDF.
    pub value_raw: String,
    /// Parsed numeric value when `value_raw` contains a number; `None` otherwise.
    #[serde(default)]
    pub value_num: Option<f64>,
    /// Unit string (e.g. `"CFM"`, `"HP"`, `"in-wg"`); empty when not present.
    #[serde(default)]
    pub unit: String,
    /// Zero-based source page index.
    pub page: usize,
    /// Normalised bounding box of the source region.  `None` when provenance
    /// cannot be determined (e.g. multi-cell table merges).
    #[serde(default)]
    pub bbox: Option<TidyBBox>,
    /// Extraction confidence (0.0–1.0).
    pub confidence: f64,
    /// Extraction source:
    /// - `"table"`: value came from a tabular row.
    /// - `"keyvalue"`: value came from a `Label: Value` pair.
    pub source: String,
    /// Conflict flags — empty in normal operation; populated when multiple
    /// extracted values for the same field disagree (e.g. `["duplicate_field"]`).
    #[serde(default)]
    pub conflict_flags: Vec<String>,
}

// ── Equipment dataset ──────────────────────────────────────────────────────────

/// Full equipment dataset for one submittal packet.
///
/// Wraps a [`Vec<TidyRow>`] with schema metadata and per-unit quality summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipmentDataset {
    /// Schema version — currently `"1.0.0"`.
    pub schema_version: String,
    /// Submittal packet identifier.
    pub packet_name: String,
    /// Total number of records (sum of records across all units).
    pub record_count: usize,
    /// Number of distinct units represented in the records.
    pub unit_count: usize,
    /// All extracted tidy rows, ordered by (start_page, field).
    pub records: Vec<TidyRow>,
    /// Per-unit extraction quality summary.
    pub unit_summaries: Vec<UnitSummary>,
}

/// Extraction quality summary for a single unit within an [`EquipmentDataset`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UnitSummary {
    /// The unit tag this summary covers.
    pub unit_tag: String,
    /// Number of tidy rows extracted for this unit.
    pub record_count: usize,
    /// Average confidence across all records for this unit.
    pub avg_confidence: f64,
    /// Number of records with `source == "table"`.
    pub table_record_count: usize,
    /// Number of records with `source == "keyvalue"`.
    pub kv_record_count: usize,
    /// Human-readable warnings about extraction quality (empty = clean).
    #[serde(default)]
    pub warnings: Vec<String>,
}

// ── Key-value extraction types ─────────────────────────────────────────────────

/// A single label/value pair extracted from a submittal page via colon-heuristic.
///
/// Produced by `crates/engine/src/submittal_kv.rs`; consumed by `submittal_export.rs`
/// to populate [`TidyRow`] records with `source = "keyvalue"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KvPair {
    /// Extracted label (left side of the colon, normalised trim).
    pub label: String,
    /// Extracted value (right side of the colon, normalised trim).
    pub value: String,
    /// Zero-based source page index.
    pub page: usize,
    /// Bounding box of the label span, when available.
    #[serde(default)]
    pub bbox: Option<TidyBBox>,
    /// Extraction confidence (0.0–1.0).
    pub confidence: f64,
}

/// Structured header fields extracted from the first pages of a unit.
///
/// Populated by `submittal_kv::extract_unit_header()`.  All fields are `Option`
/// because real-world submittals vary widely; any field may be absent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UnitHeader {
    /// Extracted unit tag (e.g. `"AHU-1"`), `None` if not confirmed.
    #[serde(default)]
    pub unit_tag: Option<String>,
    /// Equipment model number (e.g. `"CLCP036"`).
    #[serde(default)]
    pub model: Option<String>,
    /// Manufacturer / brand name (e.g. `"Carrier"`, `"Trane"`).
    #[serde(default)]
    pub manufacturer: Option<String>,
    /// Human-readable equipment type (e.g. `"Air Handling Unit"`).
    #[serde(default)]
    pub item_type: Option<String>,
    /// Overall confidence for the header extraction (0.0–1.0).
    pub confidence: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // ── TidyRow round-trip ────────────────────────────────────────────────────

    #[test]
    fn tidy_row_serde_round_trip_full() {
        let row = TidyRow {
            packet_name:    "SUB_PESH_BERG_TRANE-AHU".to_string(),
            revision_id:    "ADD-1".to_string(),
            item_tag:       "AHU-1".to_string(),
            equipment_type: "Air Handling Unit".to_string(),
            section:        "15750".to_string(),
            field:          "Cooling Airflow CFM".to_string(),
            value_raw:      "4500".to_string(),
            value_num:      Some(4500.0),
            unit:           "CFM".to_string(),
            page:           5,
            bbox:           Some(TidyBBox { x: 0.1, y: 0.2, width: 0.5, height: 0.05 }),
            confidence:     0.95,
            source:         "table".to_string(),
            conflict_flags: vec![],
        };

        let json = serde_json::to_string_pretty(&row).expect("serialize TidyRow");
        let back: TidyRow = serde_json::from_str(&json).expect("deserialize TidyRow");
        assert_eq!(row, back);
        assert!(json.contains("\"packet_name\""));
        assert!(json.contains("\"value_num\""));
    }

    #[test]
    fn tidy_row_serde_round_trip_minimal() {
        // Fields with serde(default) can be absent during deserialization.
        let json = r#"{
            "packet_name": "SUB_TEST",
            "item_tag": "RTU-1",
            "field": "ESP",
            "value_raw": "0.5",
            "page": 3,
            "confidence": 0.9,
            "source": "keyvalue"
        }"#;
        let row: TidyRow = serde_json::from_str(json).expect("deserialize minimal TidyRow");
        assert_eq!(row.packet_name, "SUB_TEST");
        assert_eq!(row.item_tag, "RTU-1");
        assert_eq!(row.value_num, None);
        assert!(row.conflict_flags.is_empty());
        assert!(row.bbox.is_none());
    }

    // ── UnitEntry round-trip ──────────────────────────────────────────────────

    #[test]
    fn unit_entry_serde_round_trip() {
        let entry = UnitEntry {
            unit_tag:     "AHU-1".to_string(),
            model:        Some("CLCP".to_string()),
            manufacturer: Some("Carrier".to_string()),
            item_type:    Some("Air Handling Unit".to_string()),
            start_page:   5,
            end_page:     22,
            page_count:   18,
            is_cover:     false,
            confidence:   0.95,
        };
        let json = serde_json::to_string_pretty(&entry).expect("serialize UnitEntry");
        let back: UnitEntry = serde_json::from_str(&json).expect("deserialize UnitEntry");
        assert_eq!(entry, back);
    }

    #[test]
    fn unit_entry_cover_serde_round_trip() {
        let entry = UnitEntry {
            unit_tag:     "COVER".to_string(),
            model:        None,
            manufacturer: None,
            item_type:    None,
            start_page:   0,
            end_page:     2,
            page_count:   3,
            is_cover:     true,
            confidence:   1.0,
        };
        let json = serde_json::to_string_pretty(&entry).expect("serialize cover UnitEntry");
        let back: UnitEntry = serde_json::from_str(&json).expect("deserialize cover UnitEntry");
        assert_eq!(entry, back);
        // Optional fields serialized as null or absent — both must deserialize cleanly.
        assert!(back.model.is_none());
    }

    // ── SubmittalIndex round-trip ─────────────────────────────────────────────

    #[test]
    fn submittal_index_serde_round_trip() {
        let index = SubmittalIndex {
            schema_version: "1.0.0".to_string(),
            packet_name:    "SUB_PESH_BERG_TRANE-AHU".to_string(),
            units: vec![
                UnitEntry {
                    unit_tag: "COVER".to_string(), model: None, manufacturer: None,
                    item_type: None, start_page: 0, end_page: 1, page_count: 2,
                    is_cover: true, confidence: 1.0,
                },
                UnitEntry {
                    unit_tag: "AHU-1".to_string(), model: Some("CLCP".to_string()),
                    manufacturer: Some("Carrier".to_string()), item_type: None,
                    start_page: 2, end_page: 73, page_count: 72,
                    is_cover: false, confidence: 0.92,
                },
            ],
            coverage: SubmittalCoverage {
                total_pages: 74, assigned_pages: 74, unassigned_pages: 0,
                coverage_ratio: 1.0, unit_count: 1,
            },
        };

        let json = serde_json::to_string_pretty(&index).expect("serialize SubmittalIndex");
        let back: SubmittalIndex = serde_json::from_str(&json).expect("deserialize SubmittalIndex");
        assert_eq!(index, back);
        assert_eq!(back.units.len(), 2);
        assert_eq!(back.coverage.unit_count, 1);
    }

    // ── EquipmentDataset round-trip ───────────────────────────────────────────

    #[test]
    fn equipment_dataset_serde_round_trip() {
        let ds = EquipmentDataset {
            schema_version: "1.0.0".to_string(),
            packet_name:    "SUB_TEST".to_string(),
            record_count:   2,
            unit_count:     1,
            records: vec![
                TidyRow {
                    packet_name: "SUB_TEST".to_string(),
                    item_tag: "AHU-1".to_string(),
                    field: "CFM".to_string(),
                    value_raw: "4000".to_string(),
                    value_num: Some(4000.0),
                    page: 5, confidence: 0.95,
                    source: "table".to_string(),
                    ..Default::default()
                },
                TidyRow {
                    packet_name: "SUB_TEST".to_string(),
                    item_tag: "AHU-1".to_string(),
                    field: "Model".to_string(),
                    value_raw: "CLCP036".to_string(),
                    page: 2, confidence: 0.9,
                    source: "keyvalue".to_string(),
                    ..Default::default()
                },
            ],
            unit_summaries: vec![UnitSummary {
                unit_tag: "AHU-1".to_string(),
                record_count: 2,
                avg_confidence: 0.925,
                table_record_count: 1,
                kv_record_count: 1,
                warnings: vec![],
            }],
        };

        let json = serde_json::to_string_pretty(&ds).expect("serialize EquipmentDataset");
        let back: EquipmentDataset = serde_json::from_str(&json).expect("deserialize EquipmentDataset");
        assert_eq!(ds, back);
        assert_eq!(back.record_count, 2);
        assert_eq!(back.unit_summaries[0].avg_confidence, 0.925);
    }

    // ── SubmittalCoverage corner cases ────────────────────────────────────────

    #[test]
    fn submittal_coverage_zero_pages() {
        let cov = SubmittalCoverage::default();
        let json = serde_json::to_string(&cov).expect("serialize default coverage");
        let back: SubmittalCoverage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.total_pages, 0);
        assert_eq!(back.coverage_ratio, 0.0);
    }

    #[test]
    fn unit_summary_default_round_trips() {
        let s = UnitSummary::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: UnitSummary = serde_json::from_str(&json).unwrap();
        assert!(back.warnings.is_empty());
    }
}

//! Canonical schedule schema and export mapping contracts — Phase 0.5 contract-shaping.
//!
//! AEC drawings contain door schedules, window schedules, equipment schedules,
//! and similar tabular data. This module defines the canonical extraction schema
//! and JSON/CSV/XML export mapping hints that downstream phases will use.
//!
//! **CONTRACT-ONLY** — schedule parser and export runtime deferred to Band 2
//! (G-028, G-029).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Schema version for schedule contract types.
pub const SCHEDULE_SCHEMA_VERSION: &str = "0.5.0";

// ── Column definition ──────────────────────────────────────────────────────

/// Primitive data type for a schedule column value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleDataType {
    Text,
    Integer,
    Float,
    Boolean,
    Date,
}

/// Definition of one column in a canonical schedule table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleColumn {
    /// Stable column identifier (snake_case, unique within the schedule).
    pub column_id: String,
    /// Human-readable header label as it appears in the drawing.
    pub header_label: String,
    /// Expected data type for values in this column.
    pub data_type: ScheduleDataType,
    /// Physical unit (e.g. `"mm"`, `"ft"`, `"CFM"`), if applicable.
    pub unit: Option<String>,
    /// CSI section ID or drawing reference where this column originates.
    pub source_section_id: Option<String>,
}

// ── Row ───────────────────────────────────────────────────────────────────

/// A single data row in a canonical schedule.
///
/// Values are stored as `Option<String>` (raw text as extracted). Type
/// coercion to the column's `data_type` is deferred to the Band 2 runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRow {
    /// Stable row identifier (e.g. door tag, equipment tag).
    pub row_id: String,
    /// Column values keyed by `column_id`. `None` = cell was blank.
    pub values: HashMap<String, Option<String>>,
    /// Page index where this row was extracted.
    pub source_page_index: Option<usize>,
    /// Extraction confidence for this row.
    pub confidence: Option<f32>,
}

// ── Export mapping ─────────────────────────────────────────────────────────

/// Export key/header/element names for one column across three export formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleExportMapping {
    /// Column this mapping applies to.
    pub column_id: String,
    /// JSON key name (camelCase recommended).
    pub json_key: String,
    /// CSV column header string.
    pub csv_header: String,
    /// XML element name (no namespace prefix).
    pub xml_element: String,
}

// ── Canonical schedule ─────────────────────────────────────────────────────

/// Canonical schedule extracted from a drawing or specification section.
///
/// One [`CanonicalSchedule`] is emitted per detected schedule table.
///
/// **CONTRACT-ONLY** — schedule parser deferred to Band 2 (G-028).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSchedule {
    pub schema_version: String,
    /// Stable schedule identifier unique within the document set.
    pub schedule_id: String,
    /// Free-text schedule type label (e.g. `"door"`, `"window"`, `"equipment"`).
    pub schedule_type: Option<String>,
    /// Intake bundle ID of the source document.
    pub source_bundle_id: Option<String>,
    /// Column definitions in left-to-right display order.
    pub columns: Vec<ScheduleColumn>,
    /// Extracted rows.
    pub rows: Vec<ScheduleRow>,
    /// Export key mappings for all columns.
    pub export_mappings: Vec<ScheduleExportMapping>,
    /// Overall extraction confidence for this schedule.
    pub extraction_confidence: Option<f32>,
}

impl CanonicalSchedule {
    /// Returns a schema-placeholder schedule with no columns or rows.
    #[must_use]
    pub fn schema_placeholder(schedule_id: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEDULE_SCHEMA_VERSION.to_owned(),
            schedule_id: schedule_id.into(),
            schedule_type: None,
            source_bundle_id: None,
            columns: Vec::new(),
            rows: Vec::new(),
            export_mappings: Vec::new(),
            extraction_confidence: None,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_schedule_placeholder_round_trips_via_serde() {
        let s = CanonicalSchedule::schema_placeholder("sched-door-001");
        let json = serde_json::to_string_pretty(&s).unwrap();
        let back: CanonicalSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schedule_id, "sched-door-001");
        assert!(back.columns.is_empty());
        assert!(back.rows.is_empty());
    }

    #[test]
    fn schedule_row_with_values_round_trips() {
        let mut vals = HashMap::new();
        vals.insert("mark".to_owned(), Some("D-101".to_owned()));
        vals.insert("width".to_owned(), Some("3'-0\"".to_owned()));
        vals.insert("material".to_owned(), None);
        let row = ScheduleRow {
            row_id: "row-1".to_owned(),
            values: vals,
            source_page_index: Some(5),
            confidence: Some(0.97),
        };
        let json = serde_json::to_string(&row).unwrap();
        let back: ScheduleRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back.row_id, "row-1");
        assert_eq!(back.values["mark"], Some("D-101".to_owned()));
        assert_eq!(back.values["material"], None);
    }

    #[test]
    fn schedule_data_type_round_trips_via_serde() {
        for dt in [
            ScheduleDataType::Text,
            ScheduleDataType::Integer,
            ScheduleDataType::Float,
            ScheduleDataType::Boolean,
            ScheduleDataType::Date,
        ] {
            let s = serde_json::to_string(&dt).unwrap();
            let back: ScheduleDataType = serde_json::from_str(&s).unwrap();
            assert_eq!(back, dt);
        }
    }
}

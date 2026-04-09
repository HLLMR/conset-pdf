//! Sprint 10.4 — tidy-format export pipeline for submittal equipment data.
//!
//! This module assembles per-unit KV pairs and table rows into the canonical
//! [`EquipmentDataset`] / [`TidyRow`] format and provides JSON + CSV serialisers.
//!
//! # Usage
//!
//! ```text
//! // 1. Extract the SubmittalIndex (from index-submittal).
//! // 2. For each non-cover unit, scope pages and extract:
//! let kv   = extract_kv_pairs(&unit_pages);
//! let tbls = extract_unit_tables(&unit_pages, &unit);
//! // 3. Assemble:
//! let dataset = build_equipment_dataset(&index, &tables_by_unit, &kv_by_unit);
//! // 4. Export:
//! let json = dataset_to_json(&dataset);
//! let csv  = dataset_to_csv(&dataset);
//! ```
//!
//! # Schema
//!
//! All [`TidyRow`] records carry `schema_version = "1.0.0"` (via the parent
//! [`EquipmentDataset`]).  The 14-column CSV layout follows the canonical schema
//! defined in `MASTER_PLAN.md §Submittals: Data Extraction`.

use conset_pdf_ir::{
    EquipmentDataset, KvPair, SubmittalIndex, TidyRow, UnitSummary,
};
use regex::Regex;
use std::sync::OnceLock;

use crate::drawing_tables::ExtractedTable;

// ── Compiled patterns ─────────────────────────────────────────────────────────

/// Matches a leading numeric value (int or decimal, optionally signed).
fn numeric_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(-?\d+\.?\d*)(.*)$").unwrap())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Assemble a full [`EquipmentDataset`] from unit metadata, extracted tables,
/// and extracted KV pairs.
///
/// # Arguments
///
/// - `index` — the submittal index containing unit metadata (tag, item_type, etc.)
/// - `tables_by_unit` — pairs of `(unit_idx, Vec<ExtractedTable>)` for each unit
///   that produced at least one table; `unit_idx` is a 0-based index into
///   `index.units`.
/// - `kv_by_unit` — pairs of `(unit_idx, Vec<KvPair>)` for each unit.
///
/// Records are ordered by `(start_page, source, field)`.  Cover units
/// (`is_cover = true`) are skipped — the caller may choose to pass only
/// non-cover unit indices, but the function also guards internally.
pub fn build_equipment_dataset(
    index: &SubmittalIndex,
    tables_by_unit: &[(usize, Vec<ExtractedTable>)],
    kv_by_unit: &[(usize, Vec<KvPair>)],
) -> EquipmentDataset {
    let mut all_records: Vec<TidyRow> = Vec::new();
    let mut unit_summaries: Vec<UnitSummary> = Vec::new();

    for (unit_idx, unit) in index.units.iter().enumerate() {
        if unit.is_cover {
            continue;
        }

        let unit_start_page = unit.start_page;
        let packet_name = &index.packet_name;
        let item_tag = &unit.unit_tag;
        let equipment_type = unit.item_type.as_deref().unwrap_or("").to_owned();

        let mut unit_records: Vec<TidyRow> = Vec::new();

        // ── KV pair rows ──────────────────────────────────────────────────────
        let empty_kv: Vec<KvPair> = Vec::new();
        let kv_records = kv_by_unit
            .iter()
            .find(|(idx, _)| *idx == unit_idx)
            .map(|(_, v)| v.as_slice())
            .unwrap_or(empty_kv.as_slice());

        for kv in kv_records {
            let (value_num, unit_str) = parse_value_and_unit(&kv.value);
            unit_records.push(TidyRow {
                packet_name: packet_name.clone(),
                revision_id: String::new(),
                item_tag: item_tag.clone(),
                equipment_type: equipment_type.clone(),
                section: String::new(),
                field: kv.label.clone(),
                value_raw: kv.value.clone(),
                value_num,
                unit: unit_str,
                page: kv.page,
                bbox: kv.bbox.clone(),
                confidence: kv.confidence,
                source: "keyvalue".to_owned(),
                conflict_flags: Vec::new(),
            });
        }

        // ── Table rows ────────────────────────────────────────────────────────
        let empty_tables: Vec<ExtractedTable> = Vec::new();
        let unit_tables = tables_by_unit
            .iter()
            .find(|(idx, _)| *idx == unit_idx)
            .map(|(_, v)| v.as_slice())
            .unwrap_or(empty_tables.as_slice());

        for table in unit_tables {
            let table_conf = table.confidence;
            for row in &table.rows {
                for (col_idx, header) in table.headers.iter().enumerate() {
                    let raw = row
                        .get(col_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_owned();
                    if raw.is_empty() || header.trim().is_empty() {
                        continue;
                    }
                    let (value_num, unit_str) = parse_value_and_unit(&raw);
                    unit_records.push(TidyRow {
                        packet_name: packet_name.clone(),
                        revision_id: String::new(),
                        item_tag: item_tag.clone(),
                        equipment_type: equipment_type.clone(),
                        section: String::new(),
                        field: header.trim().to_owned(),
                        value_raw: raw,
                        value_num,
                        unit: unit_str,
                        page: unit_start_page,
                        bbox: None,
                        confidence: table_conf,
                        source: "table".to_owned(),
                        conflict_flags: Vec::new(),
                    });
                }
            }
        }

        // ── Unit summary ──────────────────────────────────────────────────────
        let record_count = unit_records.len();
        let table_record_count =
            unit_records.iter().filter(|r| r.source == "table").count();
        let kv_record_count =
            unit_records.iter().filter(|r| r.source == "keyvalue").count();
        let avg_confidence = if record_count == 0 {
            0.0
        } else {
            unit_records.iter().map(|r| r.confidence).sum::<f64>() / record_count as f64
        };

        let mut warnings: Vec<String> = Vec::new();
        if record_count == 0 {
            warnings.push("No extractable data found for this unit.".to_owned());
        } else if avg_confidence < 0.5 {
            warnings.push(format!(
                "Low average extraction confidence ({avg_confidence:.2}); results may require review."
            ));
        }

        unit_summaries.push(UnitSummary {
            unit_tag: item_tag.clone(),
            record_count,
            avg_confidence,
            table_record_count,
            kv_record_count,
            warnings,
        });

        all_records.extend(unit_records);
    }

    let record_count = all_records.len();
    let unit_count = unit_summaries.len();

    EquipmentDataset {
        schema_version: "1.0.0".to_owned(),
        packet_name: index.packet_name.clone(),
        record_count,
        unit_count,
        records: all_records,
        unit_summaries,
    }
}

/// Serialise an [`EquipmentDataset`] to pretty-printed JSON.
#[must_use]
pub fn dataset_to_json(dataset: &EquipmentDataset) -> String {
    serde_json::to_string_pretty(dataset).unwrap_or_else(|e| {
        format!("{{\"error\": \"serialisation failed: {e}\"}}")
    })
}

/// Serialise an [`EquipmentDataset`] to a 14-column CSV.
///
/// Columns (in order):
/// `packet_name`, `revision_id`, `item_tag`, `equipment_type`, `section`,
/// `field`, `value_raw`, `value_num`, `unit`, `page`, `bbox`,
/// `confidence`, `source`, `conflict_flags`
///
/// - `value_num` is empty when `None`.
/// - `bbox` is serialised as a compact JSON object `{"x":…,"y":…,"width":…,"height":…}`
///   or an empty string when absent.
/// - `conflict_flags` entries are joined with `|`.
/// - All fields are CSV-quoted when they contain commas, double-quotes, or newlines.
#[must_use]
pub fn dataset_to_csv(dataset: &EquipmentDataset) -> String {
    let mut out = String::new();

    // Header row.
    out.push_str(
        "packet_name,revision_id,item_tag,equipment_type,section,field,\
         value_raw,value_num,unit,page,bbox,confidence,source,conflict_flags\n",
    );

    for row in &dataset.records {
        let bbox_str = match &row.bbox {
            Some(b) => format!(
                "{{\"x\":{},\"y\":{},\"width\":{},\"height\":{}}}",
                b.x, b.y, b.width, b.height
            ),
            None => String::new(),
        };
        let conflict_str = row.conflict_flags.join("|");
        let value_num_str = row
            .value_num
            .map(|n| format!("{n}"))
            .unwrap_or_default();

        let cols: [&str; 14] = [
            &row.packet_name,
            &row.revision_id,
            &row.item_tag,
            &row.equipment_type,
            &row.section,
            &row.field,
            &row.value_raw,
            &value_num_str,
            &row.unit,
            &row.page.to_string(),
            &bbox_str,
            &format!("{:.4}", row.confidence),
            &row.source,
            &conflict_str,
        ];

        let fields: Vec<String> = cols.iter().map(|s| csv_quote(s)).collect();
        out.push_str(&fields.join(","));
        out.push('\n');
    }

    out
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Parse a raw value string into an optional numeric value and a trailing unit
/// string.
///
/// Examples:
/// - `"4500 CFM"` → `(Some(4500.0), "CFM")`
/// - `"1.25 in-wg"` → `(Some(1.25), "in-wg")`
/// - `"Carrier"` → `(None, "")`
/// - `"42"` → `(Some(42.0), "")`
fn parse_value_and_unit(raw: &str) -> (Option<f64>, String) {
    let trimmed = raw.trim();
    match numeric_re().captures(trimmed) {
        Some(caps) => {
            let num_str = caps.get(1).map_or("", |m| m.as_str());
            let rest = caps.get(2).map_or("", |m| m.as_str()).trim();
            let num = num_str.parse::<f64>().ok();
            (num, rest.to_owned())
        }
        None => (None, String::new()),
    }
}

/// CSV-quote a field value according to RFC 4180.
///
/// Wraps the field in double-quotes when it contains commas, double-quotes, or
/// newlines.  Interior double-quotes are escaped by doubling.
fn csv_quote(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_owned()
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{
        SubmittalCoverage, SubmittalIndex, TidyBBox, UnitEntry,
    };

    fn make_index(tags: &[&str]) -> SubmittalIndex {
        let units: Vec<UnitEntry> = tags
            .iter()
            .enumerate()
            .map(|(i, tag)| UnitEntry {
                unit_tag: (*tag).to_owned(),
                model: None,
                manufacturer: None,
                item_type: Some("Air Handling Unit".to_owned()),
                start_page: i * 5,
                end_page: i * 5 + 4,
                page_count: 5,
                is_cover: false,
                confidence: 1.0,
            })
            .collect();
        SubmittalIndex {
            schema_version: "1.0.0".to_owned(),
            packet_name: "SUB_TEST".to_owned(),
            coverage: SubmittalCoverage {
                total_pages: tags.len() * 5,
                assigned_pages: tags.len() * 5,
                unassigned_pages: 0,
                coverage_ratio: 1.0,
                unit_count: tags.len(),
            },
            units,
        }
    }

    fn make_kv(label: &str, value: &str, page: usize) -> KvPair {
        KvPair {
            label: label.to_owned(),
            value: value.to_owned(),
            page,
            bbox: Some(TidyBBox { x: 0.1, y: 0.2, width: 0.3, height: 0.05 }),
            confidence: 0.9,
        }
    }

    // ── parse_value_and_unit ──────────────────────────────────────────────────

    #[test]
    fn parse_integer_with_unit() {
        let (num, unit) = parse_value_and_unit("4500 CFM");
        assert_eq!(num, Some(4500.0));
        assert_eq!(unit, "CFM");
    }

    #[test]
    fn parse_decimal_with_dash_unit() {
        let (num, unit) = parse_value_and_unit("1.25 in-wg");
        assert_eq!(num, Some(1.25));
        assert_eq!(unit, "in-wg");
    }

    #[test]
    fn parse_bare_integer() {
        let (num, unit) = parse_value_and_unit("42");
        assert_eq!(num, Some(42.0));
        assert_eq!(unit, "");
    }

    #[test]
    fn parse_non_numeric_string() {
        let (num, unit) = parse_value_and_unit("Carrier HVAC");
        assert_eq!(num, None);
        assert_eq!(unit, "");
    }

    // ── build_equipment_dataset ───────────────────────────────────────────────

    #[test]
    fn build_dataset_from_kv_pairs() {
        let index = make_index(&["AHU-1"]);
        let kv = vec![(
            0usize,
            vec![
                make_kv("Cooling Airflow CFM", "4500 CFM", 1),
                make_kv("ESP", "1.25", 1),
            ],
        )];

        let dataset = build_equipment_dataset(&index, &[], &kv);

        assert_eq!(dataset.schema_version, "1.0.0");
        assert_eq!(dataset.packet_name, "SUB_TEST");
        assert_eq!(dataset.record_count, 2);
        assert_eq!(dataset.unit_count, 1);

        let r0 = &dataset.records[0];
        assert_eq!(r0.item_tag, "AHU-1");
        assert_eq!(r0.source, "keyvalue");
        assert_eq!(r0.value_num, Some(4500.0));
        assert_eq!(r0.unit, "CFM");
    }

    #[test]
    fn build_dataset_from_table() {
        let index = make_index(&["RTU-1"]);
        let table = ExtractedTable {
            sheet_id: "RTU-1".to_owned(),
            sheet_title: "Air Handling Unit".to_owned(),
            table_title: None,
            headers: vec!["CFM".to_owned(), "HP".to_owned()],
            rows: vec![
                vec!["3500".to_owned(), "10".to_owned()],
                vec!["2800".to_owned(), "7.5".to_owned()],
            ],
            row_count: 2,
            confidence: 0.85,
        };
        let tables = vec![(0usize, vec![table])];
        let dataset = build_equipment_dataset(&index, &tables, &[]);

        // 2 rows × 2 non-empty columns = 4 records
        assert_eq!(dataset.record_count, 4);
        assert!(dataset.records.iter().all(|r| r.source == "table"));
        assert_eq!(dataset.records[0].field, "CFM");
        assert_eq!(dataset.records[0].value_num, Some(3500.0));
    }

    #[test]
    fn cover_units_are_skipped() {
        let mut index = make_index(&["AHU-1"]);
        // Prepend a cover unit.
        index.units.insert(
            0,
            UnitEntry {
                unit_tag: "COVER".to_owned(),
                model: None,
                manufacturer: None,
                item_type: None,
                start_page: 0,
                end_page: 2,
                page_count: 3,
                is_cover: true,
                confidence: 0.3,
            },
        );
        let kv = vec![
            (0usize, vec![make_kv("Model", "TEST", 0)]),      // unit_idx=0 → COVER (skipped)
            (1usize, vec![make_kv("Airflow", "5000 CFM", 5)]), // unit_idx=1 → AHU-1
        ];
        let dataset = build_equipment_dataset(&index, &[], &kv);
        // Only AHU-1's record should appear.
        assert_eq!(dataset.record_count, 1);
        assert_eq!(dataset.records[0].item_tag, "AHU-1");
    }

    #[test]
    fn unit_summary_warns_when_no_records() {
        let index = make_index(&["AHU-1"]);
        // No KV or table data provided.
        let dataset = build_equipment_dataset(&index, &[], &[]);
        assert_eq!(dataset.unit_summaries.len(), 1);
        assert!(
            !dataset.unit_summaries[0].warnings.is_empty(),
            "should warn when no records extracted"
        );
    }

    // ── dataset_to_csv ────────────────────────────────────────────────────────

    #[test]
    fn csv_output_has_correct_column_count() {
        let index = make_index(&["AHU-1"]);
        let kv = vec![(0usize, vec![make_kv("Model", "CLCP036", 0)])];
        let dataset = build_equipment_dataset(&index, &[], &kv);

        let csv = dataset_to_csv(&dataset);
        let lines: Vec<&str> = csv.lines().collect();
        // header + 1 data row
        assert_eq!(lines.len(), 2);
        // 14 columns in header
        assert_eq!(lines[0].split(',').count(), 14);
    }

    #[test]
    fn csv_quotes_fields_with_commas() {
        let index = make_index(&["AHU-1"]);
        let kv = vec![(
            0usize,
            vec![make_kv("Description", "Unit, Type A", 0)],
        )];
        let dataset = build_equipment_dataset(&index, &[], &kv);
        let csv = dataset_to_csv(&dataset);
        assert!(csv.contains('"'), "fields with commas must be quoted in CSV");
    }
}

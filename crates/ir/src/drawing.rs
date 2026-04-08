//! Drawing-medium IR types for Phase 9 drawing sheet management.
//!
//! These types are the drawing-set analog of `SegmentIndex`/`SectionEntry` from
//! `crates/ir/src/segment.rs`.  The key structural difference is that drawing
//! sheets carry AEC-specific fields (discipline code, revision, firm, project)
//! extracted from the title block, whereas spec sections carry CSI section IDs
//! extracted from the footer.
//!
//! # Schema
//!
//! All serde-serialisable types in this module use `schema_version = "1.0.0"`.
//! Breaking changes must bump the version and add a migration path.

use serde::{Deserialize, Serialize};

// ── Chrome metadata ───────────────────────────────────────────────────────────

/// Chrome metadata extracted from a drawing sheet's title block.
///
/// All fields are `String`; an empty string means the field was not detected.
/// `confidence` is the fraction of expected fields that were successfully
/// extracted (1.0 = all fields present and non-empty).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SheetChromeMetadata {
    /// Sheet ID as printed in the title block (e.g. `"M-201"`, `"FP-101"`).
    pub sheet_id: String,
    /// Full sheet title (e.g. `"MECHANICAL EQUIPMENT PLAN \u2014 LEVEL 1"`).
    pub sheet_title: String,
    /// Canonical4 discipline code (e.g. `"MECH"`, `"FIRP"`); empty if unknown.
    pub discipline: String,
    /// Revision string from the revision block (e.g. `"ADD-2"`, `"ISSUED FOR BID"`).
    pub revision: String,
    /// Date string from the title block as extracted (e.g. `"2026-04-08"`).
    pub date: String,
    /// Firm / architect-of-record name from the title block.
    pub firm_name: String,
    /// Project name from the title block.
    pub project_name: String,
    /// Project number / ID from the title block.
    pub project_id: String,
    /// Confidence in the title-block extraction: 1.0 = all fields extracted.
    pub confidence: f64,
}

// ── Sheet boundary ────────────────────────────────────────────────────────────

/// One sheet boundary detected by the title-block oracle.
///
/// Single-page sheets satisfy `start_page == end_page`.  Multi-page sheets
/// (e.g. a sheet with a continuation page) satisfy `end_page > start_page`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetEntry {
    /// Sheet ID normalised to uppercase (e.g. `"M-201"`).
    pub sheet_id: String,
    /// Zero-based index of the first page belonging to this sheet.
    pub start_page: usize,
    /// Zero-based index of the last page belonging to this sheet (inclusive).
    pub end_page: usize,
    /// Number of pages in this sheet (`end_page - start_page + 1`).
    pub page_count: usize,
    /// Chrome metadata extracted from the title block.
    pub chrome: SheetChromeMetadata,
    /// If the same `sheet_id` appeared in a *prior* (addendum) version with a
    /// different ID, this field holds the newer ID that supersedes it.
    pub superseded_by: Option<String>,
    /// True when the sheet title contains a schedule keyword
    /// (`SCHEDULE`, `EQUIPMENT LIST`, etc.).  Populated by
    /// `DrawingSegmentEngine::build_index()`.
    #[serde(default)]
    pub is_schedule_sheet: bool,
}

// ── Discipline summary ────────────────────────────────────────────────────────

/// Aggregated count of sheets per discipline detected in a `DrawingIndex`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisciplineSummary {
    /// Canonical4 discipline code (e.g. `"MECH"`).
    pub canonical4: String,
    /// Human-readable display name (e.g. `"Mechanical"`).
    pub display_name: String,
    /// Number of sheets with this discipline code.
    pub sheet_count: usize,
    /// Sort order for discipline grouping (lower = first).
    pub sort_order: u32,
}

// ── Drawing index ─────────────────────────────────────────────────────────────

/// Index of all sheets detected in a drawing set.
///
/// Serialised to `drawing-index.json` by the `index-drawing` CLI subcommand and
/// consumed by `DrawingsPatchOrchestrator` in `apply-sheet-addendum`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingIndex {
    /// Schema version — currently `"1.0.0"`.
    pub schema_version: String,
    /// Total number of sheets detected.
    pub sheet_count: usize,
    /// Ordered list of detected sheets (in page order).
    pub sheets: Vec<SheetEntry>,
    /// Per-discipline sheet counts, sorted by `sort_order`.
    pub discipline_summary: Vec<DisciplineSummary>,
    /// Total number of pages in the source PDF.
    pub total_pages: usize,
}

impl DrawingIndex {
    /// Create an empty `DrawingIndex` (no sheets detected).
    #[must_use]
    pub fn empty(total_pages: usize) -> Self {
        Self {
            schema_version: "1.0.0".to_owned(),
            sheet_count: 0,
            sheets: Vec::new(),
            discipline_summary: Vec::new(),
            total_pages,
        }
    }
}

// ── Manifest types for apply-sheet-addendum ───────────────────────────────────

/// Declares which drawing set the addendum sheets apply to and what to replace.
///
/// Written by the user (or tooling) as a JSON file and passed to
/// `apply-sheet-addendum --manifest <path>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingAddendumManifest {
    /// Schema version — currently `"1.0.0"`.
    pub schema_version: String,
    /// Path to the original drawing-set PDF.
    pub original_drawing_set: String,
    /// Path to the addendum PDF whose sheets replace matching sheets in the
    /// original.
    pub addendum_pdf: String,
    /// Path for the stitched output PDF.
    pub output_path: String,
    /// Optional directory for audit bundle artifacts.
    pub audit_bundle_dir: Option<String>,
    /// When `true`, validate sheet matching but do not write any output PDF.
    pub dry_run: bool,
    /// Sheets to replace.  If empty, auto-detection from the addendum index is
    /// required (not yet implemented in Sprint 9.2).
    pub sheets: Vec<SheetReplaceSpec>,
}

/// Specifies one sheet replacement within a `DrawingAddendumManifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetReplaceSpec {
    /// Sheet ID to replace (e.g. `"M-201"`).
    pub sheet_id: String,
    /// Page range in the addendum PDF that contains the replacement for this
    /// sheet (1-indexed, inclusive).  `None` means auto-detect from the
    /// addendum's `DrawingIndex`.
    pub addendum_pages: Option<SheetPageRange>,
}

/// A 1-indexed inclusive page range within a PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetPageRange {
    /// First page (1-based).
    pub start: usize,
    /// Last page (1-based, inclusive).
    pub end: usize,
}

// ── Result types for apply-sheet-addendum ─────────────────────────────────────

/// The aggregate result of a `apply-sheet-addendum` run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingPatchResult {
    /// Schema version — currently `"1.0.0"`.
    pub schema_version: String,
    /// Path to the original drawing-set PDF.
    pub original_drawing_set: String,
    /// Path to the addendum PDF.
    pub addendum_pdf: String,
    /// Output PDF path, or `None` on dry-run.
    pub output_path: Option<String>,
    /// Whether this was a dry-run.
    pub dry_run: bool,
    /// Per-sheet replacement results.
    pub sheet_results: Vec<SheetPatchResult>,
    /// Rename events detected during sheet matching.
    pub renames: Vec<SheetRenameEvent>,
    /// Pattern database version used (from embedded `default.json`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_db_version: Option<String>,
}

/// Result for one sheet replacement attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetPatchResult {
    /// Sheet ID that was targeted.
    pub sheet_id: String,
    /// Outcome of the replacement.
    pub status: SheetPatchStatus,
    /// Human-readable reason string, present on failure or skip.
    pub reason: Option<String>,
    /// Number of pages replaced, present only on `Replaced`.
    pub pages_replaced: Option<usize>,
}

/// Outcome variants for one sheet replacement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SheetPatchStatus {
    /// Sheet successfully replaced in the output PDF.
    Replaced,
    /// `sheet_id` was not found in the original drawing-set index.
    NotFound,
    /// `dry_run = true`; no PDF was written.
    Skipped,
    /// Engine returned an error; `error_code` identifies the failure.
    Failed { error_code: String },
}

/// Records a sheet that was renumbered between the original and the addendum.
///
/// Detected when two `SheetEntry` records share the same normalised `sheet_title`
/// but differ on `sheet_id` (with similarity ≥ 0.75).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRenameEvent {
    /// Sheet ID in the original drawing set.
    pub original_sheet_id: String,
    /// Replacement sheet ID in the addendum.
    pub new_sheet_id: String,
    /// The sheet title used for the match.
    pub sheet_title: String,
    /// Title-match similarity score [0.0, 1.0].
    pub confidence: f64,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip serialisation for `DrawingIndex` with a multi-discipline fixture.
    #[test]
    fn drawing_index_serde_round_trip() {
        let index = DrawingIndex {
            schema_version: "1.0.0".to_owned(),
            sheet_count: 3,
            sheets: vec![
                SheetEntry {
                    sheet_id: "M-201".to_owned(),
                    start_page: 0,
                    end_page: 0,
                    page_count: 1,
                    chrome: SheetChromeMetadata {
                        sheet_id: "M-201".to_owned(),
                        sheet_title: "MECHANICAL EQUIPMENT PLAN".to_owned(),
                        discipline: "MECH".to_owned(),
                        revision: "ORG".to_owned(),
                        date: "2025-10-01".to_owned(),
                        firm_name: "RWB Consulting Engineers".to_owned(),
                        project_name: "Lake Highlands High School".to_owned(),
                        project_id: "RWB-25063".to_owned(),
                        confidence: 1.0,
                    },
                    superseded_by: None,
                    is_schedule_sheet: false,
                },
                SheetEntry {
                    sheet_id: "FP-101".to_owned(),
                    start_page: 1,
                    end_page: 1,
                    page_count: 1,
                    chrome: SheetChromeMetadata {
                        sheet_id: "FP-101".to_owned(),
                        sheet_title: "FIRE PROTECTION PLAN".to_owned(),
                        discipline: "FIRP".to_owned(),
                        ..Default::default()
                    },
                    superseded_by: None,
                    is_schedule_sheet: false,
                },
                SheetEntry {
                    sheet_id: "E-101".to_owned(),
                    start_page: 2,
                    end_page: 3,
                    page_count: 2,
                    chrome: SheetChromeMetadata {
                        sheet_id: "E-101".to_owned(),
                        sheet_title: "ELECTRICAL PANEL SCHEDULE".to_owned(),
                        discipline: "ELEC".to_owned(),
                        ..Default::default()
                    },
                    superseded_by: Some("E-102".to_owned()),
                    is_schedule_sheet: true,
                },
            ],
            discipline_summary: vec![
                DisciplineSummary {
                    canonical4: "MECH".to_owned(),
                    display_name: "Mechanical".to_owned(),
                    sheet_count: 1,
                    sort_order: 70,
                },
                DisciplineSummary {
                    canonical4: "FIRP".to_owned(),
                    display_name: "Fire Protection".to_owned(),
                    sheet_count: 1,
                    sort_order: 75,
                },
                DisciplineSummary {
                    canonical4: "ELEC".to_owned(),
                    display_name: "Electrical".to_owned(),
                    sheet_count: 1,
                    sort_order: 90,
                },
            ],
            total_pages: 4,
        };

        let json = serde_json::to_string(&index).expect("serialise DrawingIndex");
        let back: DrawingIndex = serde_json::from_str(&json).expect("deserialise DrawingIndex");
        assert_eq!(index, back);
    }

    /// `SheetEntry` with `superseded_by = Some(...)` survives a round-trip.
    #[test]
    fn sheet_entry_superseded_by_round_trip() {
        let entry = SheetEntry {
            sheet_id: "M-201".to_owned(),
            start_page: 0,
            end_page: 0,
            page_count: 1,
            chrome: SheetChromeMetadata::default(),
            superseded_by: Some("M-204".to_owned()),
            is_schedule_sheet: false,
        };

        let json = serde_json::to_string(&entry).expect("serialise SheetEntry");
        let back: SheetEntry = serde_json::from_str(&json).expect("deserialise SheetEntry");
        assert_eq!(entry, back);
        assert_eq!(back.superseded_by, Some("M-204".to_owned()));
    }

    /// `SheetEntry` with `superseded_by = None` survives a round-trip.
    #[test]
    fn sheet_entry_not_superseded_round_trip() {
        let entry = SheetEntry {
            sheet_id: "A-101".to_owned(),
            start_page: 5,
            end_page: 6,
            page_count: 2,
            chrome: SheetChromeMetadata::default(),
            superseded_by: None,
            is_schedule_sheet: false,
        };

        let json = serde_json::to_string(&entry).expect("serialise SheetEntry");
        let back: SheetEntry = serde_json::from_str(&json).expect("deserialise SheetEntry");
        assert_eq!(entry, back);
        assert!(back.superseded_by.is_none());
    }

    /// `DrawingAddendumManifest` serde round-trip.
    #[test]
    fn drawing_addendum_manifest_round_trip() {
        let manifest = DrawingAddendumManifest {
            schema_version: "1.0.0".to_owned(),
            original_drawing_set: "/docs/DWG_ORG.pdf".to_owned(),
            addendum_pdf: "/docs/DWG_ADD2.pdf".to_owned(),
            output_path: "/output/DWG_REVISED.pdf".to_owned(),
            audit_bundle_dir: Some("/output/audit".to_owned()),
            dry_run: true,
            sheets: vec![SheetReplaceSpec {
                sheet_id: "M-201".to_owned(),
                addendum_pages: Some(SheetPageRange { start: 3, end: 3 }),
            }],
        };

        let json = serde_json::to_string(&manifest).expect("serialise manifest");
        let back: DrawingAddendumManifest =
            serde_json::from_str(&json).expect("deserialise manifest");
        assert_eq!(back.schema_version, "1.0.0");
        assert_eq!(back.sheets.len(), 1);
        assert_eq!(back.sheets[0].sheet_id, "M-201");
        assert!(back.dry_run);
    }

    /// `DrawingPatchResult` serde round-trip including a rename event.
    #[test]
    fn drawing_patch_result_round_trip() {
        let result = DrawingPatchResult {
            schema_version: "1.0.0".to_owned(),
            original_drawing_set: "/docs/DWG_ORG.pdf".to_owned(),
            addendum_pdf: "/docs/DWG_ADD2.pdf".to_owned(),
            output_path: None,
            dry_run: true,
            sheet_results: vec![SheetPatchResult {
                sheet_id: "M-201".to_owned(),
                status: SheetPatchStatus::Skipped,
                reason: Some("dry_run".to_owned()),
                pages_replaced: None,
            }],
            renames: vec![SheetRenameEvent {
                original_sheet_id: "M-201".to_owned(),
                new_sheet_id: "M-204".to_owned(),
                sheet_title: "MECHANICAL EQUIPMENT PLAN".to_owned(),
                confidence: 1.0,
            }],
            pattern_db_version: Some("1.0.0".to_owned()),
        };

        let json = serde_json::to_string(&result).expect("serialise DrawingPatchResult");
        let back: DrawingPatchResult =
            serde_json::from_str(&json).expect("deserialise DrawingPatchResult");
        assert_eq!(back.sheet_results[0].status, SheetPatchStatus::Skipped);
        assert_eq!(back.renames[0].new_sheet_id, "M-204");
    }
}

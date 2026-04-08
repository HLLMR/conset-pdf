//! IR types for the Phase 7 `apply-addendum` end-to-end workflow.
//!
//! [`AddendumManifest`] is the JSON contract read from `--addendum` when
//! invoking the `apply-addendum` CLI subcommand.  It describes which sections
//! to patch and what edits to apply to each.
//!
//! [`SectionEditSpec`] is one entry in the manifest's section list: a section
//! identifier plus the ordered edit operations to apply.  It may carry an
//! optional [`crate::SpecChromeMetadata`] override that takes precedence over
//! metadata extracted from the source PDF's headers/footers.
//!
//! [`AddendumResult`] is the orchestrator's output: per-section patch results,
//! success/failure counts, and the output path if not a dry run.
//!
//! [`SectionPatchResult`] records the outcome for a single section: success
//! with page-count deltas, or failure with a human-readable reason.
//!
//! # Merge semantics
//!
//! Chrome metadata is merged at runtime as follows:
//! 1. Base: extracted [`crate::ChromeMetadata`] from the source PDF.
//! 2. Override: `AddendumManifest::project_metadata`, if present.
//! 3. Per-section override: `SectionEditSpec::chrome_override`, if present.
//!
//! Lower levels take precedence over higher ones.  Non-empty strings in an
//! override replace the corresponding base field; empty strings are ignored.

use crate::{EditOperation, SpecChromeMetadata};
use serde::{Deserialize, Serialize};

// ── AddendumManifest ──────────────────────────────────────────────────────────

/// JSON manifest consumed by `apply-addendum`.
///
/// At minimum it must contain one entry in `sections`.  All other fields are
/// optional and have safe defaults.
///
/// # Example (minimal)
///
/// ```json
/// {
///   "sections": [
///     {
///       "section_id": "23 82 16",
///       "operations": [
///         {
///           "op": "replace",
///           "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.7", "A."] },
///           "new_text": "Provide return air damper with motorized actuator."
///         }
///       ]
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddendumManifest {
    /// Human-readable description of this addendum (e.g. `"Addendum 3 — HVAC edits"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Addendum issue date in any displayable format (e.g. `"2025-10-17"`).
    /// When present it overrides the date extracted from the source PDF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<String>,

    /// Project-level chrome metadata override.  Fields present here take
    /// precedence over the metadata extracted from the source PDF's
    /// headers/footers.  Per-section `chrome_override` in each
    /// [`SectionEditSpec`] takes precedence over this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_metadata: Option<SpecChromeMetadata>,

    /// Ordered list of sections to patch.  Must not be empty.
    pub sections: Vec<SectionEditSpec>,
}

// ── SectionEditSpec ───────────────────────────────────────────────────────────

/// One section's patch specification within an [`AddendumManifest`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionEditSpec {
    /// Canonical CSI section ID to patch (e.g. `"23 82 16"`).
    pub section_id: String,

    /// Ordered edit operations to apply to this section's AST.
    ///
    /// May be empty — an empty list causes the section to be parsed and
    /// re-rendered unchanged (useful to force a typographic refresh or to
    /// update chrome metadata without textual edits).
    #[serde(default)]
    pub operations: Vec<EditOperation>,

    /// Per-section chrome metadata override.  When present, these fields take
    /// precedence over both the extracted source-PDF metadata and the
    /// manifest-level `project_metadata` override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chrome_override: Option<SpecChromeMetadata>,
}

// ── SectionPatchStatus ────────────────────────────────────────────────────────

/// Outcome of patching a single section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SectionPatchStatus {
    /// All stages completed successfully.
    Success {
        /// Pages removed from the original PDF during stitch.
        pages_removed: usize,
        /// Pages inserted from the regenerated replacement.
        pages_inserted: usize,
    },
    /// One or more stages failed; this section was not stitched.
    Failed {
        /// Human-readable description of what went wrong.
        reason: String,
    },
}

// ── SectionPatchResult ────────────────────────────────────────────────────────

/// Per-section result produced by the orchestrator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionPatchResult {
    /// Canonical CSI section ID.
    pub section_id: String,
    /// Section title as detected in the source PDF's footer, or as provided by
    /// the chrome metadata override.  Empty string when not available.
    pub section_title: String,
    /// Whether this section succeeded or failed.
    pub status: SectionPatchStatus,
}

impl SectionPatchResult {
    /// Convenience constructor for a successful result.
    pub fn success(
        section_id: impl Into<String>,
        section_title: impl Into<String>,
        pages_removed: usize,
        pages_inserted: usize,
    ) -> Self {
        Self {
            section_id: section_id.into(),
            section_title: section_title.into(),
            status: SectionPatchStatus::Success { pages_removed, pages_inserted },
        }
    }

    /// Convenience constructor for a failed result.
    pub fn failed(
        section_id: impl Into<String>,
        section_title: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            section_id: section_id.into(),
            section_title: section_title.into(),
            status: SectionPatchStatus::Failed { reason: reason.into() },
        }
    }

    /// Returns `true` when the status is [`SectionPatchStatus::Success`].
    pub fn is_success(&self) -> bool {
        matches!(self.status, SectionPatchStatus::Success { .. })
    }
}

// ── AddendumResult ────────────────────────────────────────────────────────────

/// Final output from the `apply-addendum` orchestrator.
///
/// Serialised to `change-report.json` in the audit bundle directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddendumResult {
    /// Human-readable description copied from the manifest, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_description: Option<String>,

    /// Total number of sections in the manifest.
    pub total_sections: usize,

    /// Number of sections that were successfully patched.
    pub succeeded: usize,

    /// Number of sections that failed.
    pub failed: usize,

    /// Per-section results in manifest order.
    pub section_results: Vec<SectionPatchResult>,

    /// Absolute path to the stitched output PDF, or `None` on dry run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
}

impl AddendumResult {
    /// Constructs an `AddendumResult` from a list of per-section results.
    pub fn from_results(
        manifest_description: Option<String>,
        section_results: Vec<SectionPatchResult>,
        output_path: Option<String>,
    ) -> Self {
        let total = section_results.len();
        let succeeded = section_results.iter().filter(|r| r.is_success()).count();
        let failed = total - succeeded;
        Self {
            manifest_description,
            total_sections: total,
            succeeded,
            failed,
            section_results,
            output_path,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditOperation, NodePath};

    // ── AddendumManifest ──────────────────────────────────────────────────────

    #[test]
    fn test_addendum_manifest_serde_minimal() {
        let manifest = AddendumManifest {
            description: None,
            issue_date: None,
            project_metadata: None,
            sections: vec![SectionEditSpec {
                section_id: "23 82 16".to_owned(),
                operations: vec![],
                chrome_override: None,
            }],
        };
        let json = serde_json::to_string(&manifest).expect("serialize");
        let roundtrip: AddendumManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(manifest, roundtrip);
    }

    #[test]
    fn test_addendum_manifest_serde_full() {
        let manifest = AddendumManifest {
            description: Some("Addendum 3 — HVAC edits".to_owned()),
            issue_date: Some("2025-10-17".to_owned()),
            project_metadata: Some(SpecChromeMetadata {
                project_id: "RWB Project No. 25063.00".to_owned(),
                project_name: "Lake Highlands High School".to_owned(),
                section_id: String::new(),
                section_title: String::new(),
                date: "2025-10-17".to_owned(),
                firm: "RWB Consulting Engineers".to_owned(),
            }),
            sections: vec![SectionEditSpec {
                section_id: "23 82 16".to_owned(),
                operations: vec![EditOperation::Delete {
                    path: NodePath::new("23 82 16", vec!["PART 2", "2.7", "A."]),
                }],
                chrome_override: None,
            }],
        };
        let json = serde_json::to_string_pretty(&manifest).expect("serialize");
        let roundtrip: AddendumManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(manifest, roundtrip);
    }

    /// `sections` field default: an empty sections list round-trips correctly.
    #[test]
    fn test_addendum_manifest_empty_sections() {
        let json = r#"{"sections": []}"#;
        let manifest: AddendumManifest = serde_json::from_str(json).expect("deserialize");
        assert!(manifest.sections.is_empty());
        assert!(manifest.description.is_none());
        assert!(manifest.issue_date.is_none());
        assert!(manifest.project_metadata.is_none());
    }

    // ── SectionEditSpec ───────────────────────────────────────────────────────

    #[test]
    fn test_section_edit_spec_serde() {
        let spec = SectionEditSpec {
            section_id: "23 82 16".to_owned(),
            operations: vec![
                EditOperation::Replace {
                    path: NodePath::new("23 82 16", vec!["PART 2", "2.7", "A."]),
                    new_text: "Provide return air damper.".to_owned(),
                },
            ],
            chrome_override: Some(SpecChromeMetadata {
                date: "2025-10-17".to_owned(),
                ..Default::default()
            }),
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let roundtrip: SectionEditSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec, roundtrip);
    }

    /// When `operations` is omitted from JSON it defaults to an empty vec.
    #[test]
    fn test_section_edit_spec_operations_defaults_empty() {
        let json = r#"{"section_id": "23 82 16"}"#;
        let spec: SectionEditSpec = serde_json::from_str(json).expect("deserialize");
        assert_eq!(spec.section_id, "23 82 16");
        assert!(spec.operations.is_empty());
        assert!(spec.chrome_override.is_none());
    }

    // ── SectionPatchStatus / SectionPatchResult ───────────────────────────────

    #[test]
    fn test_section_patch_status_success_serde() {
        let status = SectionPatchStatus::Success { pages_removed: 3, pages_inserted: 4 };
        let json = serde_json::to_string(&status).expect("serialize");
        let roundtrip: SectionPatchStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(status, roundtrip);
    }

    #[test]
    fn test_section_patch_status_failed_serde() {
        let status = SectionPatchStatus::Failed { reason: "Chrome not found".to_owned() };
        let json = serde_json::to_string(&status).expect("serialize");
        let roundtrip: SectionPatchStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(status, roundtrip);
    }

    #[test]
    fn test_section_patch_result_constructors() {
        let ok = SectionPatchResult::success("23 82 16", "HEATING WATER COILS", 3, 4);
        assert!(ok.is_success());
        assert_eq!(ok.section_id, "23 82 16");
        assert_eq!(ok.section_title, "HEATING WATER COILS");
        match &ok.status {
            SectionPatchStatus::Success { pages_removed, pages_inserted } => {
                assert_eq!(*pages_removed, 3);
                assert_eq!(*pages_inserted, 4);
            }
            _ => panic!("expected Success"),
        }

        let fail = SectionPatchResult::failed("23 82 16", "HEATING WATER COILS", "parse error");
        assert!(!fail.is_success());
        match &fail.status {
            SectionPatchStatus::Failed { reason } => assert_eq!(reason, "parse error"),
            _ => panic!("expected Failed"),
        }
    }

    #[test]
    fn test_section_patch_result_serde() {
        let result = SectionPatchResult::success("23 82 16", "HEATING WATER COILS", 3, 4);
        let json = serde_json::to_string(&result).expect("serialize");
        let roundtrip: SectionPatchResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, roundtrip);
    }

    // ── AddendumResult ────────────────────────────────────────────────────────

    #[test]
    fn test_addendum_result_from_results() {
        let results = vec![
            SectionPatchResult::success("23 82 16", "HEATING WATER COILS", 3, 4),
            SectionPatchResult::failed("23 00 00", "HVAC GENERAL", "section not found"),
            SectionPatchResult::success("23 05 93", "TESTING AND BALANCING", 2, 3),
        ];
        let ar = AddendumResult::from_results(
            Some("Addendum 3".to_owned()),
            results,
            Some("/output/result.pdf".to_owned()),
        );
        assert_eq!(ar.total_sections, 3);
        assert_eq!(ar.succeeded, 2);
        assert_eq!(ar.failed, 1);
        assert_eq!(ar.manifest_description.as_deref(), Some("Addendum 3"));
        assert_eq!(ar.output_path.as_deref(), Some("/output/result.pdf"));
    }

    #[test]
    fn test_addendum_result_serde() {
        let ar = AddendumResult::from_results(
            None,
            vec![SectionPatchResult::success("23 82 16", "HEATING WATER COILS", 3, 4)],
            None,
        );
        let json = serde_json::to_string(&ar).expect("serialize");
        let roundtrip: AddendumResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ar, roundtrip);
    }

    #[test]
    fn test_addendum_result_all_failed() {
        let ar = AddendumResult::from_results(
            None,
            vec![SectionPatchResult::failed("23 82 16", "", "Chrome not found")],
            None,
        );
        assert_eq!(ar.succeeded, 0);
        assert_eq!(ar.failed, 1);
        assert!(ar.output_path.is_none());
    }
}

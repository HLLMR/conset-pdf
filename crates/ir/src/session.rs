//! GUI session state machine for the Conset PDF desktop application.
//!
//! This module defines the session state types and **pure** transition functions.
//! All side effects (subprocess spawn, file I/O) happen in the Tauri command handlers
//! — never here. The pure design makes every transition snapshot-testable.
//!
//! # State graph
//!
//! ```text
//! Idle
//!   ↓ add_files
//! FilesAdded { files, workflow_type }
//!
//!   [Drawings / Submittals]
//!   ↓ confirm_workflow
//! WorkflowReady { files, workflow_type, manifest: None }
//!
//!   [Specs — Manifest Assistant path]
//!   ↓ begin_segment_analysis
//! ManifestDraft { files, workflow_type: SpecsAddendum, sections: [], manifest: None }
//!   ↓ segment_analysis_complete
//! ManifestDraft { files, sections: Vec<DetectedSection>, manifest: None }
//!   ↓ load_manifest
//! WorkflowReady { files, workflow_type: SpecsAddendum, manifest: Some(...) }
//!
//!   ↓ start_processing
//! Processing { progress }
//!   ↓ complete_with_result
//! ReviewReady { result }
//!   ↓ export_complete
//! Exported { result, summary }
//!   ↓ reset
//! Idle
//! ```

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::types::BBox;

// ---------------------------------------------------------------------------
// Workflow type
// ---------------------------------------------------------------------------

/// The three supported workflows in Phase 11 / Lane 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    SpecsAddendum,
    DrawingAddendum,
    SubmittalExtract,
}

// ---------------------------------------------------------------------------
// File entry (one item in the file picker)
// ---------------------------------------------------------------------------

/// An entry in the file picker with validation status and workflow hint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FileEntry {
    pub path: PathBuf,
    pub file_stem: String,
    /// Advisory hint derived from filename prefix (`SPEC_`, `DWG_`, `SUB_`).
    /// Non-binding — user must explicitly confirm the workflow.
    pub workflow_hint: Option<WorkflowType>,
    pub valid: bool,
    pub error: Option<String>,
}

impl FileEntry {
    /// Creates a validated file entry from a path.
    ///
    /// Marks the entry as invalid if the path does not have a `.pdf` extension.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        let file_stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        let valid = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false);

        let error = if valid {
            None
        } else {
            Some("Only PDF files are accepted".to_owned())
        };

        let workflow_hint = detect_workflow_hint(&file_stem);

        Self {
            path,
            file_stem,
            workflow_hint,
            valid,
            error,
        }
    }
}

/// Detects the probable workflow type from a file stem, advisory only.
fn detect_workflow_hint(stem: &str) -> Option<WorkflowType> {
    let upper = stem.to_uppercase();
    if upper.starts_with("SPEC_") || upper.starts_with("SPECS_") {
        Some(WorkflowType::SpecsAddendum)
    } else if upper.starts_with("DWG_") || upper.starts_with("DRAW_") {
        Some(WorkflowType::DrawingAddendum)
    } else if upper.starts_with("SUB_") || upper.starts_with("SUBMITTAL_") {
        Some(WorkflowType::SubmittalExtract)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Manifest reference
// ---------------------------------------------------------------------------

/// A validated manifest file loaded by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ManifestRef {
    pub path: PathBuf,
    pub manifest_type: WorkflowType,
}

// ---------------------------------------------------------------------------
// Progress state
// ---------------------------------------------------------------------------

/// Real-time progress information emitted by the backend via `--progress-events`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ProgressState {
    /// Human-readable current stage label, e.g. `"Extracting layout…"`.
    pub stage: String,
    /// Percentage 0–100 if the stage has a known item count; `None` = indeterminate.
    pub pct: Option<u8>,
    pub processed: usize,
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Review item
// ---------------------------------------------------------------------------

/// Disposition status for a single review item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemStatus {
    #[default]
    NeedsReview,
    Confirmed,
    Skipped,
    Noted,
}

/// A flagged item in the post-processing review queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ReviewItem {
    pub id: String,
    /// `"section"` | `"sheet"` | `"unit"`
    pub source: String,
    /// Human-readable label, e.g. `"Section 23 82 16"` or `"Sheet A-101"`.
    pub label: String,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f64,
    pub status: ReviewItemStatus,
    pub page: usize,
    /// Plain-language description of the confidence concern.
    pub warning_text: String,
    /// Optional bounding box (from audit bundle diagnostics).
    pub bbox: Option<BBox>,
    /// Raw audit evidence text, shown under the "Advanced" toggle.
    pub audit_evidence: Option<String>,
    /// Free-text note captured via "Note edit needed" action in Sprint 11.5.
    pub note: Option<String>,
}

// ---------------------------------------------------------------------------
// Workflow result + export summary
// ---------------------------------------------------------------------------

/// Result of a completed workflow operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WorkflowResult {
    pub workflow_type: WorkflowType,
    pub output_path: Option<PathBuf>,
    pub audit_bundle_dir: Option<PathBuf>,
    pub succeeded: usize,
    pub failed: usize,
    pub warnings: usize,
    pub items: Vec<ReviewItem>,
}

/// Summary shown on the export/completion screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ExportSummary {
    pub path: PathBuf,
    pub format: String,
    pub items_exported: usize,
    pub items_skipped: usize,
    pub pending_manual_actions: usize,
}

// ---------------------------------------------------------------------------
// Manifest Assistant types
// ---------------------------------------------------------------------------

/// A section detected by the Manifest Assistant (cmd_segment auto-analysis).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DetectedSection {
    /// CSI section ID, e.g. `"23 82 16"`.
    pub id: String,
    pub title: String,
    pub page_range: (usize, usize),
}

// ---------------------------------------------------------------------------
// Session state machine
// ---------------------------------------------------------------------------

/// The canonical session state.
///
/// All variants are serializable so they can be persisted in `sessionStorage`
/// on the frontend and restored across page reloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionState {
    /// No files loaded. Initial state.
    Idle,

    /// One or more files have been added; workflow type not yet confirmed.
    FilesAdded {
        files: Vec<FileEntry>,
        workflow_type: Option<WorkflowType>,
    },

    /// Specs workflow only — auto-segment analysis in progress or completed.
    ///
    /// The user reviews detected sections and loads + validates the manifest JSON.
    ManifestDraft {
        files: Vec<FileEntry>,
        /// Detected sections populated by `segment_analysis_complete`.
        sections: Vec<DetectedSection>,
        /// Set to `Some` after the user loads and validates a manifest.
        manifest: Option<ManifestRef>,
    },

    /// All inputs confirmed; ready to start processing.
    WorkflowReady {
        files: Vec<FileEntry>,
        workflow_type: WorkflowType,
        /// Present for specs; synthesized (not user-authored) for drawings; absent for submittals.
        manifest: Option<ManifestRef>,
    },

    /// Backend subprocess is running.
    Processing {
        progress: ProgressState,
    },

    /// Processing complete; user reviews confidence-flagged items.
    ReviewReady {
        result: WorkflowResult,
    },

    /// Export complete; shows the export summary.
    Exported {
        result: WorkflowResult,
        summary: ExportSummary,
    },
}

// ---------------------------------------------------------------------------
// Transition functions (pure — no side effects)
// ---------------------------------------------------------------------------

/// Transition: add or replace files in the session.
///
/// Always succeeds regardless of current state (replaces any in-progress data).
#[must_use]
pub fn add_files(files: Vec<FileEntry>) -> SessionState {
    // Compute the dominant workflow hint for pre-selection.
    let workflow_type = files
        .iter()
        .filter_map(|f| f.workflow_hint.as_ref())
        .next()
        .cloned();

    SessionState::FilesAdded {
        files,
        workflow_type,
    }
}

/// Transition: confirm workflow type and move to `WorkflowReady` or `ManifestDraft`.
///
/// For `SpecsAddendum`, moves to `FilesAdded` with the workflow type set.
/// The `begin_segment_analysis` transition must then be called to enter `ManifestDraft`.
/// For other workflows, moves directly to `WorkflowReady`.
///
/// # Errors
/// Returns an error if the current state does not have files loaded.
pub fn confirm_workflow(
    state: &SessionState,
    workflow_type: WorkflowType,
    manifest: Option<ManifestRef>,
) -> Result<SessionState, String> {
    match state {
        SessionState::FilesAdded { files, .. }
        | SessionState::WorkflowReady { files, .. }
        | SessionState::ManifestDraft { files, .. } => Ok(SessionState::WorkflowReady {
            files: files.clone(),
            workflow_type,
            manifest,
        }),
        _ => Err("confirm_workflow requires FilesAdded, WorkflowReady, or ManifestDraft state".into()),
    }
}

/// Transition: begin Manifest Assistant segment analysis (Specs workflow only).
///
/// Moves `FilesAdded` → `ManifestDraft` with an empty section list.
/// The sections will be populated by `segment_analysis_complete` when the
/// backend subprocess returns.
///
/// # Errors
/// Returns an error if the workflow type is not `SpecsAddendum` or files are not loaded.
pub fn begin_segment_analysis(state: &SessionState) -> Result<SessionState, String> {
    match state {
        SessionState::FilesAdded {
            files,
            workflow_type: Some(WorkflowType::SpecsAddendum),
        } => Ok(SessionState::ManifestDraft {
            files: files.clone(),
            sections: Vec::new(),
            manifest: None,
        }),
        SessionState::FilesAdded {
            workflow_type: Some(wt),
            ..
        } => Err(format!(
            "begin_segment_analysis is only valid for SpecsAddendum; current hint is {wt:?}"
        )),
        SessionState::FilesAdded {
            workflow_type: None,
            ..
        } => Err("begin_segment_analysis requires workflow_type to be SpecsAddendum".into()),
        _ => Err("begin_segment_analysis requires FilesAdded state".into()),
    }
}

/// Transition: populate detected sections after auto-segment analysis completes.
pub fn segment_analysis_complete(
    state: &SessionState,
    sections: Vec<DetectedSection>,
) -> SessionState {
    match state {
        SessionState::ManifestDraft { files, manifest, .. } => SessionState::ManifestDraft {
            files: files.clone(),
            sections,
            manifest: manifest.clone(),
        },
        // If called on an unexpected state, return the state unchanged.
        _ => state.clone(),
    }
}

/// Transition: user has loaded and validated a manifest (Manifest Assistant step 6).
///
/// Moves `ManifestDraft` → `WorkflowReady`.
///
/// # Errors
/// Returns an error if the current state is not `ManifestDraft`.
pub fn load_manifest(
    state: &SessionState,
    manifest: ManifestRef,
) -> Result<SessionState, String> {
    match state {
        SessionState::ManifestDraft { files, .. } => Ok(SessionState::WorkflowReady {
            files: files.clone(),
            workflow_type: WorkflowType::SpecsAddendum,
            manifest: Some(manifest),
        }),
        _ => Err("load_manifest requires ManifestDraft state".into()),
    }
}

/// Transition: start processing (WorkflowReady → Processing).
///
/// # Errors
/// Returns an error if the current state is not `WorkflowReady`.
pub fn start_processing(state: &SessionState) -> Result<SessionState, String> {
    match state {
        SessionState::WorkflowReady { .. } => Ok(SessionState::Processing {
            progress: ProgressState::default(),
        }),
        _ => Err("start_processing requires WorkflowReady state".into()),
    }
}

/// Transition: update progress display while processing.
#[must_use]
pub fn update_progress(state: &SessionState, progress: ProgressState) -> SessionState {
    match state {
        SessionState::Processing { .. } => SessionState::Processing { progress },
        _ => state.clone(),
    }
}

/// Transition: processing complete (Processing → ReviewReady).
#[must_use]
pub fn complete_with_result(result: WorkflowResult) -> SessionState {
    SessionState::ReviewReady { result }
}

/// Transition: set a review item's status to `Confirmed`.
#[must_use]
pub fn confirm_review_item(state: &SessionState, item_id: &str) -> SessionState {
    update_item_status(state, item_id, ReviewItemStatus::Confirmed)
}

/// Transition: set a review item's status to `Skipped`.
#[must_use]
pub fn skip_review_item(state: &SessionState, item_id: &str) -> SessionState {
    update_item_status(state, item_id, ReviewItemStatus::Skipped)
}

/// Transition: export complete (ReviewReady → Exported).
///
/// # Errors
/// Returns an error if the current state is not `ReviewReady`.
pub fn export_complete(
    state: &SessionState,
    summary: ExportSummary,
) -> Result<SessionState, String> {
    match state {
        SessionState::ReviewReady { result } => Ok(SessionState::Exported {
            result: result.clone(),
            summary,
        }),
        _ => Err("export_complete requires ReviewReady state".into()),
    }
}

/// Transition: reset to `Idle` from any state.
#[must_use]
pub fn reset() -> SessionState {
    SessionState::Idle
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn update_item_status(
    state: &SessionState,
    item_id: &str,
    new_status: ReviewItemStatus,
) -> SessionState {
    let SessionState::ReviewReady { result } = state else {
        return state.clone();
    };

    let mut result = result.clone();
    for item in &mut result.items {
        if item.id == item_id {
            item.status = new_status;
            break;
        }
    }
    SessionState::ReviewReady { result }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pdf_entry(name: &str) -> FileEntry {
        let path = PathBuf::from(format!("{name}.pdf"));
        FileEntry::new(path)
    }

    fn specs_entry() -> FileEntry {
        let path = PathBuf::from("SPEC_RWB_LHHS_ALL_ORG.pdf");
        FileEntry::new(path)
    }

    fn dummy_result() -> WorkflowResult {
        WorkflowResult {
            workflow_type: WorkflowType::SpecsAddendum,
            output_path: None,
            audit_bundle_dir: None,
            succeeded: 1,
            failed: 0,
            warnings: 0,
            items: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // add_files
    // -----------------------------------------------------------------------

    #[test]
    fn idle_to_files_added_transition() {
        let state = add_files(vec![specs_entry()]);
        assert!(matches!(
            state,
            SessionState::FilesAdded { .. }
        ));
    }

    #[test]
    fn add_files_sets_workflow_hint_from_first_pdf() {
        let state = add_files(vec![specs_entry()]);
        let SessionState::FilesAdded { workflow_type, .. } = state else {
            panic!("Expected FilesAdded");
        };
        assert_eq!(workflow_type, Some(WorkflowType::SpecsAddendum));
    }

    // -----------------------------------------------------------------------
    // confirm_workflow
    // -----------------------------------------------------------------------

    #[test]
    fn files_added_drawings_to_workflow_ready() {
        let files_added = add_files(vec![pdf_entry("DWG_TEST")]);
        let result = confirm_workflow(&files_added, WorkflowType::DrawingAddendum, None);
        assert!(matches!(result, Ok(SessionState::WorkflowReady { .. })));
    }

    #[test]
    fn files_added_submittals_to_workflow_ready() {
        let files_added = add_files(vec![pdf_entry("SUB_TEST")]);
        let result = confirm_workflow(&files_added, WorkflowType::SubmittalExtract, None);
        assert!(matches!(result, Ok(SessionState::WorkflowReady { .. })));
    }

    #[test]
    fn confirm_workflow_from_idle_returns_error() {
        let result = confirm_workflow(&SessionState::Idle, WorkflowType::DrawingAddendum, None);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // begin_segment_analysis
    // -----------------------------------------------------------------------

    #[test]
    fn files_added_specs_triggers_manifest_draft() {
        let state = add_files(vec![specs_entry()]);
        let result = begin_segment_analysis(&state);
        assert!(matches!(result, Ok(SessionState::ManifestDraft { .. })));
    }

    #[test]
    fn begin_segment_analysis_wrong_workflow_returns_error() {
        let state = add_files(vec![pdf_entry("DWG_TEST")]);
        // Override hint to DrawingAddendum (just to show confirm_workflow works; then re-construct)
        let _ready = confirm_workflow(&state, WorkflowType::DrawingAddendum, None).unwrap();
        // Put back into FilesAdded with Drawing workflow type
        let state = SessionState::FilesAdded {
            files: vec![pdf_entry("DWG_TEST")],
            workflow_type: Some(WorkflowType::DrawingAddendum),
        };
        let result = begin_segment_analysis(&state);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // segment_analysis_complete
    // -----------------------------------------------------------------------

    #[test]
    fn manifest_draft_sections_populated_on_analysis_complete() {
        let state = add_files(vec![specs_entry()]);
        let draft = begin_segment_analysis(&state).unwrap();
        let sections = vec![DetectedSection {
            id: "23 82 16".to_owned(),
            title: "Fan Coil Units".to_owned(),
            page_range: (10, 25),
        }];
        let result = segment_analysis_complete(&draft, sections);
        let SessionState::ManifestDraft { sections, .. } = result else {
            panic!("Expected ManifestDraft");
        };
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].id, "23 82 16");
    }

    // -----------------------------------------------------------------------
    // load_manifest
    // -----------------------------------------------------------------------

    #[test]
    fn manifest_draft_to_workflow_ready_on_valid_manifest_load() {
        let state = add_files(vec![specs_entry()]);
        let draft = begin_segment_analysis(&state).unwrap();
        let manifest_ref = ManifestRef {
            path: PathBuf::from("manifest.json"),
            manifest_type: WorkflowType::SpecsAddendum,
        };
        let result = load_manifest(&draft, manifest_ref);
        assert!(matches!(result, Ok(SessionState::WorkflowReady { .. })));
    }

    #[test]
    fn load_manifest_from_idle_returns_error() {
        let manifest_ref = ManifestRef {
            path: PathBuf::from("manifest.json"),
            manifest_type: WorkflowType::SpecsAddendum,
        };
        let result = load_manifest(&SessionState::Idle, manifest_ref);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // start_processing
    // -----------------------------------------------------------------------

    #[test]
    fn workflow_ready_to_processing() {
        let files_added = add_files(vec![pdf_entry("DWG_TEST")]);
        let ready = confirm_workflow(&files_added, WorkflowType::DrawingAddendum, None).unwrap();
        let result = start_processing(&ready);
        assert!(matches!(result, Ok(SessionState::Processing { .. })));
    }

    // -----------------------------------------------------------------------
    // update_progress
    // -----------------------------------------------------------------------

    #[test]
    fn processing_progress_update() {
        let files_added = add_files(vec![pdf_entry("DWG_TEST")]);
        let ready = confirm_workflow(&files_added, WorkflowType::DrawingAddendum, None).unwrap();
        let processing = start_processing(&ready).unwrap();
        let new_progress = ProgressState {
            stage: "Extracting…".to_owned(),
            pct: Some(20),
            processed: 5,
            total: 25,
        };
        let updated = update_progress(&processing, new_progress.clone());
        let SessionState::Processing { progress } = updated else {
            panic!("Expected Processing");
        };
        assert_eq!(progress.pct, Some(20));
        assert_eq!(progress.stage, "Extracting…");
    }

    // -----------------------------------------------------------------------
    // complete_with_result
    // -----------------------------------------------------------------------

    #[test]
    fn processing_to_review_ready_on_success() {
        let result_state = complete_with_result(dummy_result());
        assert!(matches!(result_state, SessionState::ReviewReady { .. }));
    }

    #[test]
    fn processing_to_review_ready_on_partial_success() {
        let mut r = dummy_result();
        r.failed = 2;
        r.warnings = 1;
        let result_state = complete_with_result(r);
        let SessionState::ReviewReady { result } = result_state else {
            panic!("Expected ReviewReady");
        };
        assert_eq!(result.failed, 2);
        assert_eq!(result.warnings, 1);
    }

    // -----------------------------------------------------------------------
    // confirm_review_item / skip_review_item
    // -----------------------------------------------------------------------

    fn review_ready_with_item() -> SessionState {
        let item = ReviewItem {
            id: "item-1".to_owned(),
            source: "section".to_owned(),
            label: "Section 23 82 16".to_owned(),
            confidence: 0.75,
            status: ReviewItemStatus::NeedsReview,
            page: 10,
            warning_text: "Low confidence".to_owned(),
            bbox: None,
            audit_evidence: None,
            note: None,
        };
        let mut r = dummy_result();
        r.items = vec![item];
        SessionState::ReviewReady { result: r }
    }

    #[test]
    fn review_confirm_item_updates_status() {
        let state = review_ready_with_item();
        let updated = confirm_review_item(&state, "item-1");
        let SessionState::ReviewReady { result } = updated else {
            panic!("Expected ReviewReady");
        };
        assert_eq!(result.items[0].status, ReviewItemStatus::Confirmed);
    }

    #[test]
    fn review_skip_item_updates_status() {
        let state = review_ready_with_item();
        let updated = skip_review_item(&state, "item-1");
        let SessionState::ReviewReady { result } = updated else {
            panic!("Expected ReviewReady");
        };
        assert_eq!(result.items[0].status, ReviewItemStatus::Skipped);
    }

    // -----------------------------------------------------------------------
    // reset
    // -----------------------------------------------------------------------

    #[test]
    fn reset_from_any_state_returns_idle() {
        // From Idle
        assert!(matches!(reset(), SessionState::Idle));
        // From ReviewReady
        let was_reviewing = review_ready_with_item();
        let _ = was_reviewing; // reset() ignores current state entirely
        assert!(matches!(reset(), SessionState::Idle));
    }
}

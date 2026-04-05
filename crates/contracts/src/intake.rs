//! Intake Triage contract types — Phase 0.5 contract-shaping.
//!
//! Defines the canonical shapes for Stage 0 intake triage: document
//! classification, addenda ordering, advisory manifests, and the root
//! [`NormalizedIntakeBundle`] that downstream phases consume.
//!
//! **CONTRACT-ONLY** — runtime execution deferred to Band 1 (G-013, G-014, G-016).
//! All fields that runtime will populate are typed but may carry `None` or
//! empty `Vec` values in schema-placeholder construction.

use serde::{Deserialize, Serialize};

/// Schema version applied to all intake contract types.
pub const INTAKE_SCHEMA_VERSION: &str = "0.5.0";

// ── Document classification ────────────────────────────────────────────────

/// High-level document class assigned during Stage 0 triage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentClass {
    Specification,
    Drawing,
    Addendum,
    Substitution,
    Rfi,
    Submittal,
    Unknown,
}

/// Confidence band for a triage decision, mapped from the locked
/// confidence thresholds in DEV_STANDARDS (< 0.80 / 0.80–0.95 / ≥ 0.95).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriageConfidence {
    /// Confidence ≥ 0.95 — proceed normally.
    High,
    /// Confidence 0.80–0.95 — output with warning flag.
    Medium,
    /// Confidence < 0.80 — escalate for human review.
    Low,
}

// ── Issues ─────────────────────────────────────────────────────────────────

/// Severity level for an [`IntakeIssue`] or exception.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Informational note with no action required.
    Info,
    /// A non-blocking anomaly that reviewers should inspect.
    Warning,
    /// A problem occurred but processing continued with degraded output.
    Error,
    /// Processing cannot continue for this document.
    Fatal,
}

/// An issue detected during intake triage.
///
/// Issues are attached to the [`NormalizedIntakeBundle`] and surfaced
/// to downstream review queues and exception trackers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeIssue {
    /// Stable, corpus-unique ID for this issue record.
    pub issue_id: String,
    /// Severity level.
    pub severity: IssueSeverity,
    /// Short machine-readable code, e.g. `"MISSING_FOOTER"`.
    pub code: String,
    /// Human-readable description.
    pub description: String,
    /// Page index where the issue was detected, if applicable.
    pub page_index: Option<usize>,
    /// Suggested remediation action for the reviewer.
    pub suggested_action: Option<String>,
}

// ── Addenda ordering ───────────────────────────────────────────────────────

/// One addendum in a canonical addenda sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddendumEntry {
    /// Stable ID matching the document's intake bundle ID.
    pub addendum_id: String,
    /// Canonical sequence number within this document set (1-based).
    pub sequence_number: u32,
    /// ISO-8601 issue date, e.g. `"2025-10-01"`.
    pub issue_date: Option<String>,
    /// IDs of prior addenda that this addendum supersedes.
    pub supersedes: Vec<String>,
    /// Source file path for this addendum's intake bundle.
    pub source_path: Option<String>,
}

/// Canonical ordered sequence of addenda for a document set.
///
/// **CONTRACT-ONLY** — ordering logic deferred to Band 1 (G-016).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddendaOrdering {
    pub schema_version: String,
    /// Project or engagement identifier.
    pub project_id: Option<String>,
    /// Stable ID for the document set these addenda belong to.
    pub document_set_id: Option<String>,
    /// Ordered list of addenda, lowest sequence number first.
    pub entries: Vec<AddendumEntry>,
    /// IDs of addenda where ordering conflicts could not be resolved.
    pub unresolved_conflicts: Vec<String>,
}

impl AddendaOrdering {
    /// Returns a schema-placeholder with empty entries.
    #[must_use]
    pub fn schema_placeholder() -> Self {
        Self {
            schema_version: INTAKE_SCHEMA_VERSION.to_owned(),
            project_id: None,
            document_set_id: None,
            entries: Vec::new(),
            unresolved_conflicts: Vec::new(),
        }
    }
}

// ── Advisory classification manifest ──────────────────────────────────────

/// Advisory category for a document or section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdvisoryCategory {
    /// Content matches a known standard reference.
    Standard,
    /// Substitution request detected.
    Substitution,
    /// RFI reference detected.
    Rfi,
    /// Detected conflict with a prior document in the set.
    Conflict,
    /// Category could not be determined.
    Unknown,
}

/// A single advisory classification entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryEntry {
    pub entry_id: String,
    pub category: AdvisoryCategory,
    /// Document that generated this advisory.
    pub source_document_id: Option<String>,
    pub page_index: Option<usize>,
    /// CSI section ID or other cross-reference key.
    pub section_id: Option<String>,
    pub description: Option<String>,
    pub confidence: Option<f32>,
}

/// Manifest of all advisory classifications across a document set.
///
/// **CONTRACT-ONLY** — classification logic deferred to Band 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryClassificationManifest {
    pub schema_version: String,
    pub document_set_id: Option<String>,
    pub entries: Vec<AdvisoryEntry>,
}

impl AdvisoryClassificationManifest {
    /// Returns a schema-placeholder with no entries.
    #[must_use]
    pub fn schema_placeholder() -> Self {
        Self {
            schema_version: INTAKE_SCHEMA_VERSION.to_owned(),
            document_set_id: None,
            entries: Vec::new(),
        }
    }
}

// ── Normalized intake bundle ───────────────────────────────────────────────

/// Root output of Stage 0 Intake Triage — one bundle per input document.
///
/// All downstream phases consume this bundle as their primary input
/// reference. Fields that runtime will populate carry `None` / empty
/// `Vec` in schema-placeholder construction.
///
/// **CONTRACT-ONLY** — runtime deferred to Band 1 (G-013, G-014, G-016).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedIntakeBundle {
    pub schema_version: String,
    /// Stable, corpus-unique ID for this intake bundle.
    pub bundle_id: String,
    /// High-level document class assigned by triage.
    pub document_class: DocumentClass,
    /// Overall confidence of the triage decision.
    pub triage_confidence: TriageConfidence,
    /// Path to the source PDF at intake time.
    pub source_path: String,
    /// Total page count of the source document.
    pub page_count: Option<u32>,
    /// Detected AEC discipline (e.g. `"structural"`, `"mechanical"`).
    pub detected_discipline: Option<String>,
    /// Detected primary CSI spec section ID (e.g. `"23 82 16"`).
    pub detected_spec_section: Option<String>,
    /// Canonical addenda ordering when this document is an addendum.
    pub addenda_ordering: Option<AddendaOrdering>,
    /// Advisory classifications detected during triage.
    pub advisory_manifest: Option<AdvisoryClassificationManifest>,
    /// Issues requiring attention.
    pub issues: Vec<IntakeIssue>,
    /// Version of the contracts crate that produced this bundle.
    pub contracts_version: String,
}

impl NormalizedIntakeBundle {
    /// Returns a schema-placeholder bundle for a given source path.
    ///
    /// All detection fields are `None`; `document_class` defaults to
    /// `Unknown` and `triage_confidence` to `Low` until runtime populates them.
    #[must_use]
    pub fn schema_placeholder(bundle_id: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self {
            schema_version: INTAKE_SCHEMA_VERSION.to_owned(),
            bundle_id: bundle_id.into(),
            document_class: DocumentClass::Unknown,
            triage_confidence: TriageConfidence::Low,
            source_path: source_path.into(),
            page_count: None,
            detected_discipline: None,
            detected_spec_section: None,
            addenda_ordering: None,
            advisory_manifest: None,
            issues: Vec::new(),
            contracts_version: crate::CONTRACTS_VERSION.to_owned(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intake_bundle_placeholder_round_trips_via_serde() {
        let bundle = NormalizedIntakeBundle::schema_placeholder("b-001", "tests/corpus/tier1/SPEC.pdf");
        let json = serde_json::to_string_pretty(&bundle).unwrap();
        let back: NormalizedIntakeBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.bundle_id, "b-001");
        assert_eq!(back.document_class, DocumentClass::Unknown);
        assert_eq!(back.triage_confidence, TriageConfidence::Low);
        assert!(back.issues.is_empty());
    }

    #[test]
    fn addenda_ordering_placeholder_is_empty() {
        let ao = AddendaOrdering::schema_placeholder();
        assert!(ao.entries.is_empty());
        assert!(ao.unresolved_conflicts.is_empty());
        assert_eq!(ao.schema_version, INTAKE_SCHEMA_VERSION);
    }

    #[test]
    fn advisory_manifest_placeholder_is_empty() {
        let m = AdvisoryClassificationManifest::schema_placeholder();
        assert!(m.entries.is_empty());
    }

    #[test]
    fn issue_severity_ordering() {
        assert!(IssueSeverity::Fatal > IssueSeverity::Error);
        assert!(IssueSeverity::Error > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }

    #[test]
    fn document_class_round_trips_via_serde() {
        let cls = DocumentClass::Addendum;
        let s = serde_json::to_string(&cls).unwrap();
        let back: DocumentClass = serde_json::from_str(&s).unwrap();
        assert_eq!(back, DocumentClass::Addendum);
    }
}

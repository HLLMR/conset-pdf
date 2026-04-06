//! Canonical request/response and audit contract types.
//!
//! During 0.x, these contracts are locked to the engine version and shared by
//! all integration points (CLI, GUI IPC, and internal orchestration layers).
//!
//! ## Phase 0.5-H modules
//!
//! | Module | Domain |
//! |--------|--------|
//! | [`intake`] | Intake triage, addenda ordering, advisory classification |
//! | [`ocr_routing`] | Per-page OCR routing manifest |
//! | [`schedule`] | Canonical schedule schema and export mappings |
//! | [`assisted_intelligence`] | Micro-ML, LLM validation/instruction contracts |
//! | [`operational_trust`] | Corrections, instruction DSL, diffs, review, job-state |
//! | [`knowledge`] | Normalization, entity, and index records |

pub mod assisted_intelligence;
pub mod intake;
pub mod knowledge;
pub mod ocr_routing;
pub mod operational_trust;
pub mod schedule;

use serde::{Deserialize, Serialize};

/// Canonical version for all serialized contracts in this crate.
pub const CONTRACTS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns true when an external component uses the same contracts version.
#[must_use]
pub fn versions_match(component_contracts_version: &str) -> bool {
    component_contracts_version == CONTRACTS_VERSION
}

/// Request envelope consumed by workflow runners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRequest {
    pub request_id: String,
    pub session_id: String,
    pub operation_id: String,
    pub operation: WorkflowOperation,
    pub input_path: String,
    pub output_path: Option<String>,
    pub options: WorkflowOptions,
}

/// Response envelope returned by workflow runners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub request_id: String,
    pub session_id: String,
    pub operation_id: String,
    pub contracts_version: String,
    pub result: OperationResult,
    pub audit_events: Vec<AuditEventData>,
}

impl WorkflowResponse {
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        session_id: impl Into<String>,
        operation_id: impl Into<String>,
        result: OperationResult,
        audit_events: Vec<AuditEventData>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            session_id: session_id.into(),
            operation_id: operation_id.into(),
            contracts_version: CONTRACTS_VERSION.to_owned(),
            result,
            audit_events,
        }
    }
}

/// Workflow operations currently supported by orchestration layers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowOperation {
    Extract,
    Segment,
    Parse,
    MergeAddenda,
    SplitSet,
    AssembleSet,
    FixBookmarks,
    Detect,
    SpecsPatch,
    /// Render a `LayoutTranscript` JSON as per-page PNG overlays for layout inspection.
    Visualize,
    /// Render per-page PNGs with furniture regions (header/footer/body bands) and
    /// section boundaries marked from a `SegmentIndex` JSON.
    VisualizeSegments,
    /// Render a `ParsedDocument` JSON as a self-contained HTML tree for outline inspection.
    VisualizeAst,
}

/// Per-request optional context used by execution and audit pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowOptions {
    pub profile_id: Option<String>,
    pub dry_run: bool,
    pub metadata: Vec<KeyValuePair>,
}

/// Canonical operation result payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub status: OperationStatus,
    pub summary: String,
    pub warnings: Vec<String>,
    pub error_code: Option<String>,
    pub output_artifacts: Vec<OutputArtifact>,
}

/// High-level terminal status for a workflow operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Succeeded,
    SucceededWithWarnings,
    Failed,
}

/// Named outputs produced by a workflow operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputArtifact {
    pub kind: String,
    pub path: String,
}

/// A simple key-value metadata pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

/// Aggregate operation counts required for session end logging (M-003).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationCounts {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub warned: u32,
}

/// Behavior class for feature gates and deprecation gates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateBehavior {
    HardFail,
    SoftWarn,
}

/// Evaluated state of a gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateOutcome {
    Allowed,
    Blocked,
    Warned,
}

/// Canonical audit event payloads aligned to Phase D migration M-003.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEventData {
    SessionStarted {
        session_id: String,
        started_at_utc: String,
        contracts_version: String,
        engine_version: Option<String>,
    },
    SessionEnded {
        session_id: String,
        ended_at_utc: String,
        duration_ms: u64,
        operation_counts: OperationCounts,
    },
    OperationStarted {
        session_id: String,
        operation_id: String,
        operation: WorkflowOperation,
        started_at_utc: String,
        page_count: Option<u32>,
        file_size_bucket: Option<String>,
        detection_source: Option<String>,
    },
    OperationEnded {
        session_id: String,
        operation_id: String,
        operation: WorkflowOperation,
        ended_at_utc: String,
        duration_ms: u64,
        result: OperationStatus,
    },
    GateEvaluated {
        session_id: String,
        operation_id: Option<String>,
        gate_name: String,
        behavior: GateBehavior,
        outcome: GateOutcome,
        reason_code: Option<String>,
    },
    FeatureDisabled {
        session_id: String,
        operation_id: Option<String>,
        feature_name: String,
        behavior: GateBehavior,
        reason_code: String,
    },
}

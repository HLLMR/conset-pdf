//! Assisted intelligence contract types — Phase 0.5 contract-shaping.
//!
//! Covers three deferred AI paths:
//!
//! 1. **Local micro-ML** — `MicroMlDecisionRecord` captures the inputs,
//!    model identity, and outcome of a local model call for audit and replay
//!    (Band 2, G-024/G-025).
//!
//! 2. **Cloud LLM validation** — `LlmValidationRequest`/`LlmValidationResponse`
//!    wrap the prompt payload and model reply for provenance tracking
//!    (Band 4, G-026).
//!
//! 3. **LLM instruction** — `LlmInstructionRequest` supports power-user
//!    correction/override commands expressed as natural-language instructions
//!    (Band 4, G-026).
//!
//! **CONTRACT-ONLY** — all three paths deferred to the indicated bands.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Schema version for assisted-intelligence contract types.
pub const AI_SCHEMA_VERSION: &str = "0.5.0";

// ── Intelligence source tag ────────────────────────────────────────────────

/// Labels which AI/human path produced a given result or decision.
///
/// Used in review objects and audit events to distinguish deterministic
/// vector extraction from model-assisted outputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntelligenceSource {
    /// Deterministic rule-based extraction (vector text, regex, heuristics).
    Deterministic,
    /// Local micro-ML model (Band 2, G-024/G-025).
    LocalMicroMl,
    /// Cloud or local LLM call (Band 4, G-026).
    CloudLlm,
    /// Human reviewer override.
    HumanReview,
}

// ── Micro-ML decision record ───────────────────────────────────────────────

/// A single named feature fed into a micro-ML model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlFeature {
    pub name: String,
    /// Numeric feature value (normalized to model's expected range).
    pub value: f64,
    /// Importance weight from the model, if available.
    pub weight: Option<f64>,
}

/// Audit record for a single micro-ML model decision.
///
/// Provides full replay capability: given `model_id`, `model_version`,
/// and `input_features`, the decision can be reproduced deterministically.
///
/// **CONTRACT-ONLY** — runtime deferred to Band 2 (G-024, G-025).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroMlDecisionRecord {
    pub schema_version: String,
    /// Stable ID for this decision record.
    pub decision_id: String,
    /// Registry identifier for the model that made this decision.
    pub model_id: String,
    /// Semantic version of the model weights.
    pub model_version: String,
    /// All features passed to the model.
    pub input_features: Vec<MlFeature>,
    /// Winning class label (`None` = model abstained).
    pub predicted_class: Option<String>,
    /// Model confidence for `predicted_class` (`None` = not computed).
    pub predicted_confidence: Option<f32>,
    /// Decision threshold applied (may differ from default).
    pub threshold_applied: Option<f32>,
    /// Post-threshold outcome label (e.g. `"accept"`, `"reject"`, `"escalate"`).
    pub outcome: Option<String>,
    /// ISO-8601 UTC timestamp of the decision.
    pub timestamp_utc: Option<String>,
}

impl MicroMlDecisionRecord {
    /// Returns a schema-placeholder decision record.
    #[must_use]
    pub fn schema_placeholder(decision_id: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            schema_version: AI_SCHEMA_VERSION.to_owned(),
            decision_id: decision_id.into(),
            model_id: model_id.into(),
            model_version: "0.0.0".to_owned(),
            input_features: Vec::new(),
            predicted_class: None,
            predicted_confidence: None,
            threshold_applied: None,
            outcome: None,
            timestamp_utc: None,
        }
    }
}

// ── LLM validation request / response ─────────────────────────────────────

/// Request envelope for a cloud or local LLM validation call.
///
/// `payload` is a string-keyed map of context values (raw-string form).
/// Structured JSON values are serialized to strings before sending to
/// keep this contract free of a `serde_json` dependency.
///
/// **CONTRACT-ONLY** — runtime deferred to Band 4 (G-026).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmValidationRequest {
    pub schema_version: String,
    pub request_id: String,
    /// Intake bundle ID of the document being validated.
    pub source_bundle_id: Option<String>,
    pub page_index: Option<usize>,
    /// Registered template ID used to construct the prompt.
    pub prompt_template_id: String,
    /// Key-value context injected into the prompt template.
    pub payload: HashMap<String, String>,
    /// Token budget for the model response.
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0 = deterministic, 1.0 = creative).
    pub temperature: Option<f32>,
}

/// Response envelope from a LLM validation call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmValidationResponse {
    pub schema_version: String,
    /// Echoed `request_id` from the originating request.
    pub request_id: String,
    /// Model identifier (e.g. `"gpt-4o"`, `"llama-3-8b"`).
    pub model_id: String,
    /// Raw text response from the model.
    pub response_text: Option<String>,
    /// Structured output parsed from the response (raw JSON string).
    pub structured_output: Option<String>,
    /// Model's self-reported confidence, if available.
    pub confidence: Option<f32>,
    /// Tokens consumed (prompt + completion).
    pub tokens_used: Option<u32>,
    /// End-to-end wall-clock latency in milliseconds.
    pub latency_ms: Option<u64>,
    pub timestamp_utc: Option<String>,
}

// ── LLM instruction request ────────────────────────────────────────────────

/// Power-user instruction expressed as natural language, forwarded to an LLM
/// for interpretation and conversion into typed [`super::operational_trust::InstructionBlock`]
/// ops.
///
/// **CONTRACT-ONLY** — runtime deferred to Band 4 (G-026).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInstructionRequest {
    pub schema_version: String,
    pub instruction_id: String,
    /// Workflow operation this instruction targets.
    pub operation_id: Option<String>,
    /// Natural-language instruction text from the power user.
    pub instruction_text: String,
    /// Additional context key-value pairs injected alongside the instruction.
    pub context: HashMap<String, String>,
    /// When `true`, the LLM response should describe the planned ops without
    /// executing them. The dry-run bridge is deferred to Band 3 (G-035).
    pub dry_run: bool,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn micro_ml_decision_placeholder_round_trips_via_serde() {
        let r = MicroMlDecisionRecord::schema_placeholder("dec-001", "title-block-classifier");
        let json = serde_json::to_string_pretty(&r).unwrap();
        let back: MicroMlDecisionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.decision_id, "dec-001");
        assert_eq!(back.model_id, "title-block-classifier");
        assert!(back.input_features.is_empty());
        assert!(back.predicted_class.is_none());
    }

    #[test]
    fn llm_validation_request_round_trips_via_serde() {
        let mut payload = HashMap::new();
        payload.insert("sheet_number".to_owned(), "S1.0".to_owned());
        let req = LlmValidationRequest {
            schema_version: AI_SCHEMA_VERSION.to_owned(),
            request_id: "req-001".to_owned(),
            source_bundle_id: Some("b-001".to_owned()),
            page_index: Some(0),
            prompt_template_id: "title-block-validate-v1".to_owned(),
            payload,
            max_tokens: Some(256),
            temperature: Some(0.0),
        };
        let json = serde_json::to_string_pretty(&req).unwrap();
        let back: LlmValidationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.request_id, "req-001");
        assert_eq!(back.payload["sheet_number"], "S1.0");
        assert_eq!(back.temperature, Some(0.0));
    }

    #[test]
    fn intelligence_source_round_trips_via_serde() {
        for src in [
            IntelligenceSource::Deterministic,
            IntelligenceSource::LocalMicroMl,
            IntelligenceSource::CloudLlm,
            IntelligenceSource::HumanReview,
        ] {
            let s = serde_json::to_string(&src).unwrap();
            let back: IntelligenceSource = serde_json::from_str(&s).unwrap();
            assert_eq!(back, src);
        }
    }
}

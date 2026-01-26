//! Audit event definitions and types.
//!
//! This module defines the event types that can occur during PDF processing
//! and the structure for recording them with full context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An audit event recording an operation on a PDF document.
///
/// Each event captures a point-in-time record of what happened, when it happened,
/// and relevant metadata about the operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,

    /// Type/classification of the event
    pub event_type: String,

    /// Additional context and data about the event
    pub metadata: Value,
}

impl AuditEvent {
    /// Creates a new audit event with the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `event_type` - Classification of the event (e.g., "extraction_started")
    /// * `metadata` - JSON object containing event-specific data
    pub fn new(event_type: impl Into<String>, metadata: Value) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type: event_type.into(),
            metadata,
        }
    }

    /// Creates an event with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - When the event occurred
    /// * `event_type` - Classification of the event
    /// * `metadata` - Event-specific data
    pub fn with_timestamp(
        timestamp: DateTime<Utc>,
        event_type: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            timestamp,
            event_type: event_type.into(),
            metadata,
        }
    }
}

/// Common event types for audit trails.
pub mod event_types {
    /// Extraction operation started
    pub const EXTRACTION_STARTED: &str = "extraction_started";
    /// Extraction operation completed
    pub const EXTRACTION_COMPLETED: &str = "extraction_completed";
    /// Extraction operation failed
    pub const EXTRACTION_FAILED: &str = "extraction_failed";

    /// Validation passed
    pub const VALIDATION_PASSED: &str = "validation_passed";
    /// Validation failed
    pub const VALIDATION_FAILED: &str = "validation_failed";

    /// Processing operation started
    pub const PROCESSING_STARTED: &str = "processing_started";
    /// Processing operation completed
    pub const PROCESSING_COMPLETED: &str = "processing_completed";
    /// Processing operation failed
    pub const PROCESSING_FAILED: &str = "processing_failed";
}

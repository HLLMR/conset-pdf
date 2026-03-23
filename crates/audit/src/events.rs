//! Audit event definitions and types.
//!
//! This module re-exports [`conset_pdf_contracts::AuditEventData`] as the
//! canonical set of typed event payloads and wraps each payload in an
//! [`AuditEvent`] that attaches a wall-clock timestamp for storage ordering.
//!
//! # Alignment with contracts
//!
//! [`AuditEventData`] is a closed enum defined in `crates/contracts` and covers
//! all events described in Phase D migration M-003:
//! `SessionStarted`, `SessionEnded`, `OperationStarted`, `OperationEnded`,
//! `GateEvaluated`, `FeatureDisabled`.
//!
//! Use the enum variants directly instead of string constants to ensure
//! exhaustive matching across the codebase.

use chrono::{DateTime, Utc};
use conset_pdf_contracts::AuditEventData;
use serde::{Deserialize, Serialize};

/// An audit event recording a typed lifecycle occurrence with a wall-clock timestamp.
///
/// The `data` field carries the structured payload; see [`AuditEventData`] for
/// the full set of recognised variants and their fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Wall-clock time when the event was recorded (UTC).
    pub timestamp: DateTime<Utc>,

    /// Typed event payload aligned to the contracts schema.
    pub data: AuditEventData,
}

impl AuditEvent {
    /// Creates a new audit event stamped with the current UTC time.
    #[must_use]
    pub fn new(data: AuditEventData) -> Self {
        Self { timestamp: Utc::now(), data }
    }

    /// Creates an audit event with an explicit timestamp (useful for replays and tests).
    #[must_use]
    pub fn with_timestamp(timestamp: DateTime<Utc>, data: AuditEventData) -> Self {
        Self { timestamp, data }
    }
}

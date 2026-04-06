//! Audit bundle collection and management.
//!
//! This module provides the container for audit events and operations
//! for building and querying complete audit trails.

use crate::events::AuditEvent;
use serde::{Deserialize, Serialize};

/// A collection of audit events forming a complete audit trail.
///
/// An audit bundle groups related events together, typically representing
/// a single PDF processing operation or session. Bundles can be serialized
/// to JSON for storage, transmission, or compliance reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBundle {
    /// Collection of events in the bundle
    pub events: Vec<AuditEvent>,
}

impl AuditBundle {
    /// Creates a new, empty audit bundle.
    #[must_use]
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Adds an event to the bundle.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to add
    pub fn add_event(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    /// Adds multiple events to the bundle.
    ///
    /// # Arguments
    ///
    /// * `events` - Iterator of events to add
    pub fn add_events(&mut self, events: impl IntoIterator<Item = AuditEvent>) {
        self.events.extend(events);
    }

    /// Returns the number of events in the bundle.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns an iterator over the events.
    pub fn iter(&self) -> impl Iterator<Item = &AuditEvent> {
        self.events.iter()
    }

    /// Clears all events from the bundle.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for AuditBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_contracts::AuditEventData;

    #[test]
    fn test_bundle_creation() {
        let bundle = AuditBundle::new();
        assert_eq!(bundle.event_count(), 0);
    }

    #[test]
    fn test_add_event() {
        let mut bundle = AuditBundle::new();
        let event = AuditEvent::new(AuditEventData::SessionStarted {
            session_id: "test-session".into(),
            started_at_utc: "2026-01-01T00:00:00Z".into(),
            contracts_version: "0.1.0".into(),
            engine_version: None,
        });
        bundle.add_event(event);
        assert_eq!(bundle.event_count(), 1);
    }

    #[test]
    fn test_event_insertion_order_preserved() {
        let mut bundle = AuditBundle::new();
        for i in 0u32..3 {
            bundle.add_event(AuditEvent::new(AuditEventData::SessionStarted {
                session_id: format!("s-{i}"),
                started_at_utc: "2026-01-01T00:00:00Z".into(),
                contracts_version: "0.1.0".into(),
                engine_version: None,
            }));
        }
        let ids: Vec<String> = bundle
            .iter()
            .map(|e| match &e.data {
                AuditEventData::SessionStarted { session_id, .. } => session_id.clone(),
                _ => panic!("unexpected variant"),
            })
            .collect();
        assert_eq!(ids, vec!["s-0", "s-1", "s-2"]);
    }

    #[test]
    fn test_clear_resets_count() {
        let mut bundle = AuditBundle::new();
        bundle.add_event(AuditEvent::new(AuditEventData::SessionStarted {
            session_id: "s".into(),
            started_at_utc: "2026-01-01T00:00:00Z".into(),
            contracts_version: "0.1.0".into(),
            engine_version: None,
        }));
        assert_eq!(bundle.event_count(), 1);
        bundle.clear();
        assert_eq!(bundle.event_count(), 0);
    }

    #[test]
    fn test_bundle_json_round_trip() {
        let mut bundle = AuditBundle::new();
        bundle.add_event(AuditEvent::new(AuditEventData::SessionStarted {
            session_id: "round-trip-session".into(),
            started_at_utc: "2026-04-06T00:00:00Z".into(),
            contracts_version: "0.1.0".into(),
            engine_version: Some("0.1.0".into()),
        }));
        let json = serde_json::to_string(&bundle).expect("serialization failed");
        let restored: AuditBundle = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(restored.event_count(), 1);
        match &restored.events[0].data {
            AuditEventData::SessionStarted { session_id, .. } => {
                assert_eq!(session_id, "round-trip-session");
            }
            _ => panic!("unexpected event variant after round-trip"),
        }
    }
}

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
    use serde_json::json;

    #[test]
    fn test_bundle_creation() {
        let bundle = AuditBundle::new();
        assert_eq!(bundle.event_count(), 0);
    }

    #[test]
    fn test_add_event() {
        let mut bundle = AuditBundle::new();
        let event = AuditEvent::new("test_event", json!({}));
        bundle.add_event(event);
        assert_eq!(bundle.event_count(), 1);
    }
}

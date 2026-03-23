//! Audit trail generation and management for Conset PDF.
//!
//! This crate provides comprehensive audit trail capabilities for tracking all
//! operations performed on PDF documents during extraction and processing. Audit
//! bundles capture events, metadata, and state changes for compliance and debugging.
//!
//! # Audit Bundle Format
//!
//! An audit bundle is a serialized collection of audit events that documents the
//! complete lifecycle of a PDF extraction operation. Each event wraps a typed
//! [`AuditEventData`] payload (from `crates/contracts`) with a wall-clock timestamp.
//!
//! Bundles can be serialized to JSON for storage, analysis, and compliance reporting.
//!
//! # Structure
//!
//! - [`events`]: Event wrapper and typed payload re-export
//! - [`bundle`]: Audit bundle collection and management
//! - [`writer`]: I/O operations for persisting audit trails
//!
//! # Example
//!
//! ```ignore
//! use conset_pdf_audit::{AuditEvent, AuditBundle, AuditEventData};
//!
//! let event = AuditEvent::new(AuditEventData::SessionStarted {
//!     session_id: "s1".into(),
//!     started_at_utc: "2026-01-01T00:00:00Z".into(),
//!     contracts_version: "0.1.0".into(),
//!     engine_version: None,
//! });
//! let mut bundle = AuditBundle::new();
//! bundle.add_event(event);
//! ```

pub mod bundle;
pub mod events;
pub mod writer;

pub use bundle::AuditBundle;
pub use events::AuditEvent;
// Re-export for callers who build events without importing contracts directly.
pub use conset_pdf_contracts::AuditEventData;

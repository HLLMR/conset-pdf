//! Audit trail generation and management for Conset PDF.
//!
//! This crate provides comprehensive audit trail capabilities for tracking all
//! operations performed on PDF documents during extraction and processing. Audit
//! bundles capture events, metadata, and state changes for compliance and debugging.
//!
//! # Audit Bundle Format
//!
//! An audit bundle is a serialized collection of audit events that documents the
//! complete lifecycle of a PDF extraction operation. Each event includes:
//! - **Timestamp**: When the event occurred
//! - **Event Type**: Classification of the operation (extraction, validation, processing, etc.)
//! - **Metadata**: Context-specific information about the event
//!
//! Bundles can be serialized to JSON for storage, analysis, and compliance reporting.
//!
//! # Structure
//!
//! - [`events`]: Event types and definitions
//! - [`bundle`]: Audit bundle collection and management
//! - [`writer`]: I/O operations for persisting audit trails
//!
//! # Example
//!
//! ```ignore
//! use conset_pdf_audit::{AuditEvent, AuditBundle};
//!
//! let event = AuditEvent::new("extraction_started", serde_json::json!({}));
//! let mut bundle = AuditBundle::new();
//! bundle.add_event(event);
//! ```

pub mod bundle;
pub mod events;
pub mod writer;

pub use bundle::AuditBundle;
pub use events::AuditEvent;

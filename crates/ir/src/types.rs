//! Common type definitions and structures.
//!
//! This module defines reusable types that are used across the IR crate,
//! such as document containers and elements.

use serde::{Deserialize, Serialize};

/// A PDF document representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    // Fields to be defined
}

/// A page within a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    // Fields to be defined
}

/// A content element (text, image, shape, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    // Fields to be defined
}

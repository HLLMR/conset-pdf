//! Audit bundle persistence and I/O operations.
//!
//! This module provides functionality for reading and writing audit bundles
//! to persistent storage in various formats.

use crate::bundle::AuditBundle;
use anyhow::Result;
use std::path::Path;

/// Writes an audit bundle to a JSON file.
///
/// # Arguments
///
/// * `bundle` - The audit bundle to write
/// * `path` - Path where the bundle should be written
///
/// # Returns
///
/// Ok if successful, or an error if I/O or serialization fails
///
/// # Errors
///
/// Returns an error if the bundle cannot be serialized or written to disk.
pub fn write_bundle_json(bundle: &AuditBundle, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(bundle)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Reads an audit bundle from a JSON file.
///
/// # Arguments
///
/// * `path` - Path to the bundle file
///
/// # Returns
///
/// The deserialized bundle, or an error if I/O or deserialization fails
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed as JSON.
pub fn read_bundle_json(path: impl AsRef<Path>) -> Result<AuditBundle, anyhow::Error> {
    let json = std::fs::read_to_string(path)?;
    let bundle = serde_json::from_str(&json)?;
    Ok(bundle)
}

/// Serializes an audit bundle to a JSON string.
///
/// # Arguments
///
/// * `bundle` - The bundle to serialize
///
/// # Returns
///
/// JSON string representation of the bundle
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn serialize_bundle_json(bundle: &AuditBundle) -> Result<String> {
    Ok(serde_json::to_string_pretty(bundle)?)
}

/// Deserializes an audit bundle from a JSON string.
///
/// # Arguments
///
/// * `json` - JSON string containing the bundle
///
/// # Returns
///
/// The deserialized bundle
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed.
pub fn deserialize_bundle_json(json: &str) -> Result<AuditBundle, anyhow::Error> {
    Ok(serde_json::from_str(json)?)
}

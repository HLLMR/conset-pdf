#![allow(clippy::disallowed_methods)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_component_path_imports)]

//! Tests for `TranscriptMetadata` construction, validation, and serialization.

use conset_pdf_ir::layout::MetadataError;
use conset_pdf_ir::TranscriptMetadata;
use serde_json;

#[test]
fn test_metadata_accepts_valid_source_path() {
    let result = TranscriptMetadata::new("/path/to/file.pdf", 5);

    assert!(result.is_ok(), "Should accept valid source path");

    let metadata = result.unwrap();
    assert_eq!(metadata.source_path, "/path/to/file.pdf", "Source path should match");
}

#[test]
fn test_metadata_rejects_empty_source_path() {
    let result = TranscriptMetadata::new("", 0);

    assert!(result.is_err(), "Should reject empty source path");

    match result {
        Err(MetadataError::EmptySourcePath) => {
            // Expected
        }
        other => panic!("Expected MetadataError::EmptySourcePath, got {:?}", other),
    }
}

#[test]
fn test_metadata_generates_valid_timestamp() {
    let metadata = TranscriptMetadata::new("/path/to/file.pdf", 5).expect("valid source path");

    assert!(!metadata.extraction_timestamp.is_empty(), "Timestamp should not be empty");

    // Verify ISO8601 format: should contain 'T' and 'Z' or timezone offset
    assert!(
        metadata.extraction_timestamp.contains('T'),
        "Timestamp should be ISO8601 format (contain 'T')"
    );
}

#[test]
fn test_metadata_uses_crate_version() {
    let metadata = TranscriptMetadata::new("/path/to/file.pdf", 5).expect("valid source path");

    assert_eq!(
        metadata.conset_version,
        env!("CARGO_PKG_VERSION"),
        "conset_version should match CARGO_PKG_VERSION"
    );
}

#[test]
fn test_metadata_serializes_to_json() {
    let metadata = TranscriptMetadata::new("/path/to/file.pdf", 5).expect("valid source path");

    let json_result = serde_json::to_string(&metadata);
    assert!(json_result.is_ok(), "Should serialize to JSON successfully");

    let json_string = json_result.unwrap();
    assert!(json_string.contains("source_path"), "JSON should contain 'source_path' field");
}

#[test]
fn test_metadata_deserializes_from_json() {
    let original = TranscriptMetadata::new("/path/to/file.pdf", 5).expect("valid source path");

    let json_string = serde_json::to_string(&original).expect("should serialize");

    let deserialized: Result<TranscriptMetadata, _> = serde_json::from_str(&json_string);
    assert!(deserialized.is_ok(), "Should deserialize from JSON successfully");

    let deserialized_metadata = deserialized.unwrap();
    assert_eq!(
        original, deserialized_metadata,
        "Round-trip serialization should preserve equality"
    );
}

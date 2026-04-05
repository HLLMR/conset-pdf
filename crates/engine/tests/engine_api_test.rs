//! Regression tests for the engine public API.
//!
//! These tests lock the `Extractor` and `Processor` public signatures so that
//! future pipeline work cannot silently break callers.  If a test here fails
//! after modifying engine, the impact on downstream consumers must be assessed
//! before the change is accepted.

use conset_pdf_engine::{Extractor, Processor};
use std::path::PathBuf;

fn fixture_path(filename: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // engine
    p.pop(); // crates
    p.push("tests/corpus/tier1");
    p.push(filename);
    p.to_string_lossy().into_owned()
}

#[test]
fn extractor_returns_layout_transcript() {
    let path = fixture_path("simple.pdf");
    let extractor = Extractor::new();
    let result = extractor.extract(&path);
    assert!(result.is_ok(), "Extractor::extract should succeed: {:?}", result.err());
    let transcript = result.unwrap();
    assert!(transcript.page_count() > 0, "transcript must contain at least one page");
}

#[test]
fn processor_preserves_transcript_page_count() {
    let path = fixture_path("simple.pdf");
    let extractor = Extractor::new();
    let processor = Processor::new();
    let transcript = extractor.extract(&path).expect("extract should succeed");
    let expected_pages = transcript.page_count();

    let result = processor.process(transcript);
    assert!(result.is_ok(), "Processor::process should succeed: {:?}", result.err());
    let processed = result.unwrap();
    assert_eq!(processed.page_count(), expected_pages, "processor must not alter page count");
}

#[test]
fn extractor_default_equals_new() {
    let path = fixture_path("simple.pdf");
    let a = Extractor::new();
    let b = Extractor;
    let ta = a.extract(&path).expect("new extractor must succeed");
    let tb = b.extract(&path).expect("default extractor must succeed");
    assert_eq!(
        ta.page_count(),
        tb.page_count(),
        "Extractor::new and Extractor::default must be equivalent"
    );
}

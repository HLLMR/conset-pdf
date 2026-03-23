//! Regression tests for the engine public API.
//!
//! These tests lock the `Extractor` and `Processor` public signatures so that
//! future pipeline work cannot silently break callers.  If a test here fails
//! after modifying engine, the impact on downstream consumers must be assessed
//! before the change is accepted.

use conset_pdf_engine::{Extractor, Processor};

#[test]
fn extractor_returns_layout_transcript() {
    let extractor = Extractor::new();
    // The stub accepts any string; a real path is not required at this stage.
    let result = extractor.extract("dummy.pdf");
    assert!(result.is_ok(), "Extractor::extract should succeed for stub path");
    let transcript = match result {
        Ok(transcript) => transcript,
        Err(error) => panic!("Extractor::extract returned error: {error}"),
    };
    assert_eq!(transcript.page_count(), 1, "stub transcript must contain exactly one page");
}

#[test]
fn processor_preserves_transcript_page_count() {
    let extractor = Extractor::new();
    let processor = Processor::new();
    let transcript = match extractor.extract("dummy.pdf") {
        Ok(transcript) => transcript,
        Err(error) => panic!("extractor stub should succeed: {error}"),
    };
    let expected_pages = transcript.page_count();

    let result = processor.process(transcript);
    assert!(result.is_ok(), "Processor::process should succeed for stub transcript");
    let processed = match result {
        Ok(transcript) => transcript,
        Err(error) => panic!("processor stub should succeed: {error}"),
    };
    assert_eq!(processed.page_count(), expected_pages, "processor stub must not alter page count");
}

#[test]
fn extractor_default_equals_new() {
    let a = Extractor::new();
    let b = Extractor;
    // Both should produce the same transcript for the same input.
    let ta = match a.extract("dummy.pdf") {
        Ok(transcript) => transcript,
        Err(error) => panic!("new extractor must succeed: {error}"),
    };
    let tb = match b.extract("dummy.pdf") {
        Ok(transcript) => transcript,
        Err(error) => panic!("default extractor must succeed: {error}"),
    };
    assert_eq!(
        ta.page_count(),
        tb.page_count(),
        "Extractor::new and Extractor::default must be equivalent"
    );
}

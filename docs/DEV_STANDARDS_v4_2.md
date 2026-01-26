# Conset PDF: Development Standards V4.2

**Version:** 4.2.0  
**Date:** January 23, 2026  
**Owner:** HLLMR LLC  
**Status:** ✅ ACTIVE

---

## Overview

This document defines the **day-to-day coding standards** for Conset PDF development. It's the practical companion to the Master Plan—focused on **how we write code**, **how we test**, and **how we work with AI coding agents**.

**Audience:** Developers (human and AI agents)

**Purpose:** Ensure consistency, quality, and maintainability across the codebase

**Philosophy:** Standards exist to enable velocity + reliability. Any standard without clear rationale is waste.

---

## Table of Contents

1. [Core Principles](#core-principles)
2. [Rust Coding Standards](#rust-coding-standards)
3. [Testing Requirements](#testing-requirements)
4. [Debug Logging Standards](#debug-logging-standards)
5. [Error Handling](#error-handling)
6. [Documentation Requirements](#documentation-requirements)
7. [Git Workflow](#git-workflow)
8. [AI Coding Agent Workflow](#ai-coding-agent-workflow)
9. [Code Review Checklist](#code-review-checklist)
10. [Performance Guidelines](#performance-guidelines)

---

## Core Principles

### Principle 1: Determinism is Sacred

**Rule:** Same input + same profile + same engine version = identical output, always.

**Application:**
- No runtime randomness (no `rand()`, no shuffles, no stochastic selection)
- No floating-point variance in coordinate calculations (use fixed-point or rational arithmetic where precision matters)
- All operations must be deterministic and reproducible
- Tie-breaks must be deterministic (e.g., sort by ID, then by position)

**Testing:**
- Every function must produce identical output on identical input
- Regression tests must verify bit-identical outputs

**Example:**
```rust
// ❌ BAD - Non-deterministic
use rand::Rng;
let threshold = rand::thread_rng().gen_range(0.8..0.9);

// ✅ GOOD - Deterministic
const CONFIDENCE_THRESHOLD: f32 = 0.85;
```

---

### Principle 2: Correctness Over Cleverness

**Rule:** Readable, correct code beats clever, fast code.

**Application:**
- Explicit > Implicit (be obvious, not clever)
- Proven > Novel (use battle-tested algorithms)
- Annotated > Magic (document assumptions, invariants, rationale)
- Conservative > Permissive (fail safe on ambiguity)

**Decision Method:**
- Correctness vs Speed? Choose correctness, optimize later with profiling
- Correctness vs Brevity? Choose correctness, even if verbose
- Correctness vs Cleverness? Choose correctness, always

**Example:**
```rust
// ❌ BAD - Clever but opaque
let zones = spans.iter()
    .fold(Vec::new(), |mut acc, s| {
        acc.last_mut().map_or_else(|| acc.push(vec![s]), |z| {
            if (s.bbox.y - z.last().unwrap().bbox.y).abs() < 5.0 { z.push(s); }
        });
        acc
    });

// ✅ GOOD - Clear and correct
let mut zones: Vec<Vec<&Span>> = Vec::new();
for span in spans {
    let mut placed = false;
    
    // Try to add to existing zone
    for zone in &mut zones {
        if let Some(last_span) = zone.last() {
            let vertical_gap = (span.bbox.y - last_span.bbox.y).abs();
            if vertical_gap < 5.0 {
                zone.push(span);
                placed = true;
                break;
            }
        }
    }
    
    // Create new zone if needed
    if !placed {
        zones.push(vec![span]);
    }
}
```

---

### Principle 3: Fail Explicitly, Never Silently

**Rule:** If confidence drops below threshold or something looks wrong, FAIL LOUD. Never guess.

**Application:**
- Low confidence (<0.80) → hard fail, escalate to human review
- Medium confidence (0.80–0.95) → emit output + warning flag
- High confidence (≥0.95) → proceed normally
- Ambiguous cases → log all candidates, choose deterministically, flag in audit

**Example:**
```rust
// ❌ BAD - Silent failure
pub fn extract_section_id(footer: &str) -> Option<String> {
    // If parsing fails, return None without explanation
    footer.split("–").next().map(|s| s.trim().to_string())
}

// ✅ GOOD - Explicit failure with context
pub fn extract_section_id(footer: &str) -> Result<SectionId, ExtractionError> {
    let parts: Vec<&str> = footer.split("–").collect();
    
    if parts.is_empty() {
        return Err(ExtractionError::MissingSectionDelimiter {
            footer: footer.to_string(),
            confidence: 0.0,
        });
    }
    
    let id_candidate = parts[0].trim();
    
    if !SECTION_ID_PATTERN.is_match(id_candidate) {
        return Err(ExtractionError::InvalidSectionIdFormat {
            candidate: id_candidate.to_string(),
            footer: footer.to_string(),
            confidence: 0.3,
        });
    }
    
    Ok(SectionId {
        value: id_candidate.to_string(),
        confidence: 0.98,
    })
}
```

---

## Rust Coding Standards

### Naming Conventions

**Types:**
```rust
// Structs: PascalCase
pub struct LayoutTranscript { }
pub struct BoundingBox { }

// Enums: PascalCase, variants PascalCase
pub enum Medium {
    Specifications,
    Drawings,
    Submittals,
}

// Traits: PascalCase, often adjective
pub trait Extractable { }
pub trait Renderable { }
```

**Functions and Variables:**
```rust
// Functions: snake_case, verb phrases
pub fn extract_footer(page: &Page) -> Result<Footer> { }
pub fn normalize_coordinates(bbox: BBox) -> BBox { }

// Variables: snake_case, descriptive nouns
let section_id = "23 82 16";
let confidence_score = 0.95;
let bounding_box = BBox { x: 0.0, y: 0.0, width: 1.0, height: 1.0 };
```

**Constants:**
```rust
// Constants: SCREAMING_SNAKE_CASE
const FOOTER_BAND_START: f32 = 0.85;
const CONFIDENCE_THRESHOLD: f32 = 0.80;
const MAX_PAGE_SIZE: usize = 10_000;
```

---

### Module Organization

**Standard structure:**
```rust
// At top of file
//! Module-level documentation
//! 
//! This module handles furniture detection for spec documents.
//! It identifies headers, footers, and chrome regions that should
//! be excluded from content parsing.

// Imports grouped and sorted
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ir::{LayoutTranscript, Page, Span};
use crate::audit::{AuditEvent, AuditLogger};

// Public interface first
pub struct FurnitureDetector { }

impl FurnitureDetector {
    pub fn new() -> Self { }
    pub fn detect(&self, transcript: &LayoutTranscript) -> Result<FurnitureRegions> { }
}

// Private implementation
struct InternalHelper { }

// Tests at end
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_footer() { }
}
```

---

### Type Safety

**Use newtype pattern for domain types:**
```rust
// ❌ BAD - Primitive obsession
pub fn calculate_confidence(correct: f32, total: f32) -> f32 {
    correct / total
}

// ✅ GOOD - Type-safe domain types
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Confidence(f32);

impl Confidence {
    pub fn new(value: f32) -> Result<Self, InvalidConfidence> {
        if !(0.0..=1.0).contains(&value) {
            return Err(InvalidConfidence { value });
        }
        Ok(Confidence(value))
    }
    
    pub fn value(&self) -> f32 {
        self.0
    }
    
    pub fn is_high(&self) -> bool {
        self.0 >= 0.95
    }
    
    pub fn is_low(&self) -> bool {
        self.0 < 0.80
    }
}

pub fn calculate_confidence(correct: usize, total: usize) -> Confidence {
    let ratio = correct as f32 / total as f32;
    Confidence::new(ratio).expect("confidence ratio must be 0.0-1.0")
}
```

---

### Error Handling

**Use Result types, not panics:**
```rust
// ❌ BAD - Panics are for bugs, not errors
pub fn parse_section_id(s: &str) -> SectionId {
    let parts: Vec<&str> = s.split_whitespace().collect();
    assert_eq!(parts.len(), 3, "section ID must have 3 parts");
    SectionId {
        division: parts[0].parse().unwrap(),
        section: parts[1].parse().unwrap(),
        subsection: parts[2].parse().unwrap(),
    }
}

// ✅ GOOD - Return Result, provide context
pub fn parse_section_id(s: &str) -> Result<SectionId, ParseError> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    
    if parts.len() != 3 {
        return Err(ParseError::InvalidSectionIdFormat {
            input: s.to_string(),
            expected: "XX YY ZZ",
            found_parts: parts.len(),
        });
    }
    
    let division = parts[0].parse()
        .map_err(|e| ParseError::InvalidDivision {
            input: parts[0].to_string(),
            source: e,
        })?;
    
    let section = parts[1].parse()
        .map_err(|e| ParseError::InvalidSection {
            input: parts[1].to_string(),
            source: e,
        })?;
    
    let subsection = parts[2].parse()
        .map_err(|e| ParseError::InvalidSubsection {
            input: parts[2].to_string(),
            source: e,
        })?;
    
    Ok(SectionId { division, section, subsection })
}
```

---

### Unsafe Code

**Rule:** No `unsafe` blocks in critical parsing paths. Pre-approve all `unsafe` code.

**When unsafe is allowed:**
- FFI bindings (PDFium, Chrome)
- Performance-critical inner loops (with benchmarks proving necessity)
- Memory-mapped file I/O (with clear safety rationale)

**When unsafe is NOT allowed:**
- Parsing logic
- AST construction
- Decision-making code
- Confidence calculation

**Required documentation for unsafe:**
```rust
// ✅ GOOD - Documented unsafe with safety rationale
/// # Safety
/// 
/// This function is safe because:
/// 1. `ptr` is guaranteed to be valid for `len` bytes by PDFium's API contract
/// 2. The data at `ptr` is guaranteed to remain valid during this call
/// 3. We immediately copy the data to owned memory
/// 4. The slice lifetime is tied to the borrow of `page`
unsafe fn extract_text_from_pdfium(page: &Page) -> String {
    let ptr = pdfium_sys::FPDFText_GetBoundedText(page.handle, ...);
    let len = pdfium_sys::FPDFText_CountChars(page.handle);
    
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    String::from_utf8_lossy(slice).to_string()
}
```

---

## Testing Requirements

### Test Coverage

**Minimum requirements:**
- Unit tests: Every public function
- Integration tests: Every phase/stage
- Regression tests: Torture corpus must pass ≥95%
- Golden file tests: Deterministic output verification

**Coverage target:** ≥85% line coverage (measure with `cargo tarpaulin`)

---

### Test Structure

**Standard test pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test fixtures at top
    fn sample_transcript() -> LayoutTranscript {
        LayoutTranscript {
            pages: vec![
                Page {
                    index: 0,
                    width: 612.0,
                    height: 792.0,
                    spans: vec![
                        Span {
                            text: "23 82 16 – Heating Water Coils".to_string(),
                            bbox: BBox { x: 0.1, y: 0.9, width: 0.8, height: 0.05 },
                            font: FontInfo::default(),
                            flags: SpanFlags::default(),
                        },
                    ],
                },
            ],
            metadata: DocumentMetadata::default(),
        }
    }
    
    // Test cases: test_<function>_<scenario>_<expected_outcome>
    #[test]
    fn test_detect_footer_with_valid_pattern_returns_footer() {
        let transcript = sample_transcript();
        let detector = FurnitureDetector::new();
        
        let result = detector.detect_footer(&transcript.pages[0]);
        
        assert!(result.is_ok());
        let footer = result.unwrap();
        assert_eq!(footer.section_id, "23 82 16");
        assert!(footer.confidence > 0.95);
    }
    
    #[test]
    fn test_detect_footer_with_missing_pattern_returns_error() {
        let mut transcript = sample_transcript();
        transcript.pages[0].spans[0].text = "Invalid footer".to_string();
        
        let detector = FurnitureDetector::new();
        let result = detector.detect_footer(&transcript.pages[0]);
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExtractionError::MissingSectionId { .. }));
    }
    
    #[test]
    fn test_detect_footer_deterministic_output() {
        let transcript = sample_transcript();
        let detector = FurnitureDetector::new();
        
        // Run twice, results must be identical
        let result1 = detector.detect_footer(&transcript.pages[0]).unwrap();
        let result2 = detector.detect_footer(&transcript.pages[0]).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(result1.confidence, result2.confidence);
    }
}
```

---

### Test-Driven Development (TDD)

**Workflow:**
1. Write the test FIRST (defines what you want)
2. Run test (should fail - red)
3. Implement minimum code to pass test (green)
4. Refactor if needed
5. Commit

**Example workflow:**
```rust
// Step 1: Write failing test
#[test]
fn test_normalize_coordinates_flips_y_axis() {
    let bbox = BBox { x: 0.0, y: 100.0, width: 50.0, height: 20.0 };
    let page_height = 792.0;
    
    let normalized = normalize_coordinates(bbox, page_height);
    
    // Expect Y to be flipped: 792 - 100 - 20 = 672
    assert_eq!(normalized.y, 672.0);
}

// Step 2: Run test (fails - function doesn't exist)
// $ cargo test
// error[E0425]: cannot find function `normalize_coordinates`

// Step 3: Implement minimum code to pass
pub fn normalize_coordinates(bbox: BBox, page_height: f32) -> BBox {
    BBox {
        x: bbox.x,
        y: page_height - bbox.y - bbox.height, // Flip Y
        width: bbox.width,
        height: bbox.height,
    }
}

// Step 4: Run test (passes - green)
// $ cargo test
// test tests::test_normalize_coordinates_flips_y_axis ... ok

// Step 5: Commit
// $ git commit -m "Add coordinate normalization with Y-axis flip"
```

---

### Integration Tests

**Location:** `tests/` directory (separate from unit tests)

**Structure:**
```
tests/
├── integration/
│   ├── phase1_layout_extraction.rs
│   ├── phase2_furniture_detection.rs
│   ├── phase3_paragraph_parsing.rs
│   └── ...
├── fixtures/
│   ├── test_spec.pdf
│   ├── test_drawing.pdf
│   └── expected_transcript.json
└── golden/
    ├── spec_23_82_16.json
    └── ...
```

**Example integration test:**
```rust
// tests/integration/phase1_layout_extraction.rs
use conset_pdf::engine::Engine;
use std::path::Path;

#[test]
fn test_extract_layout_from_sample_spec() {
    let engine = Engine::new();
    let pdf_path = Path::new("tests/fixtures/test_spec.pdf");
    
    let transcript = engine.extract_layout(pdf_path)
        .expect("layout extraction should succeed");
    
    // Validate structure
    assert_eq!(transcript.pages.len(), 3);
    assert!(transcript.pages[0].spans.len() > 10);
    
    // Validate coordinate normalization
    for page in &transcript.pages {
        for span in &page.spans {
            assert!(span.bbox.x >= 0.0 && span.bbox.x <= 1.0);
            assert!(span.bbox.y >= 0.0 && span.bbox.y <= 1.0);
        }
    }
    
    // Compare against golden file
    let expected = std::fs::read_to_string("tests/golden/test_spec_transcript.json")
        .expect("golden file should exist");
    let expected_transcript: LayoutTranscript = serde_json::from_str(&expected)
        .expect("golden file should be valid JSON");
    
    assert_eq!(transcript, expected_transcript);
}
```

---

### Torture Corpus Tests

**Purpose:** Validate against real-world nightmare PDFs

**Location:** Separate repository (large files)

**CI Integration:**
```bash
# In CI pipeline
cargo test --release
cargo run --release -- validate-corpus \
    --corpus-path /corpus \
    --min-pass-rate 0.95 \
    --output-report corpus-report.json
```

**Torture corpus test structure:**
```rust
#[test]
#[ignore] // Only run with --ignored flag
fn test_torture_corpus_tier1_baseline() {
    let corpus_path = Path::new("../torture-corpus/tier1");
    let engine = Engine::new();
    
    let mut passed = 0;
    let mut failed = 0;
    
    for entry in std::fs::read_dir(corpus_path).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() != Some(OsStr::new("pdf")) {
            continue;
        }
        
        match engine.extract_layout(&path) {
            Ok(_) => passed += 1,
            Err(e) => {
                eprintln!("Failed on {}: {:?}", path.display(), e);
                failed += 1;
            }
        }
    }
    
    let pass_rate = passed as f32 / (passed + failed) as f32;
    assert!(pass_rate >= 1.0, "Tier1 must have 100% pass rate, got {:.2}%", pass_rate * 100.0);
}
```

---

## Debug Logging Standards

**This is CRITICAL. Debug logging is your primary debugging tool when working with AI-generated code.**

### Rule: Add Debug Hooks Everywhere

**In every function, log at key decision points:**

```rust
use log::{debug, info, warn, error};

pub fn extract_footer(page: &Page) -> Result<Footer, ExtractionError> {
    debug!("Extracting footer from page {}", page.index);
    
    // Step 1: Detect footer region
    let footer_region = detect_footer_region(page);
    debug!("Footer region detected: {:?}", footer_region);
    
    if footer_region.confidence < 0.80 {
        warn!("Low confidence footer region on page {}: {:.2}", 
              page.index, footer_region.confidence);
    }
    
    // Step 2: Extract text
    let text = extract_text_from_region(page, &footer_region);
    debug!("Footer text: '{}'", text);
    
    // Step 3: Parse section ID
    let section_id = parse_section_id(&text)?;
    debug!("Parsed section ID: {:?}", section_id);
    
    // Step 4: Validate
    if section_id.confidence < 0.95 {
        warn!("Medium confidence section ID on page {}: {:.2}",
              page.index, section_id.confidence);
    }
    
    info!("Successfully extracted footer from page {}: section_id={}, confidence={:.2}",
          page.index, section_id.value, section_id.confidence);
    
    Ok(Footer {
        section_id,
        region: footer_region,
        raw_text: text,
    })
}
```

---

### Log Levels

**Use the right level for the right context:**

```rust
// DEBUG: Detailed flow, variable values, step-by-step progress
debug!("Processing span {} at bbox {:?}", span.id, span.bbox);
debug!("Confidence score: {:.2}", confidence);

// INFO: High-level operations, successful completions
info!("Extracted layout from {} pages", pages.len());
info!("Section segmentation complete: {} sections found", sections.len());

// WARN: Low confidence, degraded performance, non-critical issues
warn!("Low confidence footer on page {}: {:.2}", page_idx, confidence);
warn!("Missing section ID on page {}, skipping", page_idx);

// ERROR: Failures, errors, critical issues
error!("Failed to extract text from page {}: {}", page_idx, e);
error!("Invalid section ID format: '{}'", raw_text);
```

---

### Running with Debug Logging

**Command line:**
```bash
# Enable all debug logs
RUST_LOG=debug cargo run -- extract test.pdf

# Enable debug logs for specific module
RUST_LOG=conset_pdf::furniture=debug cargo run -- extract test.pdf

# Enable info for most, debug for one module
RUST_LOG=info,conset_pdf::furniture=debug cargo run -- extract test.pdf

# Output to file
RUST_LOG=debug cargo run -- extract test.pdf 2> debug.log
```

**Example debug output:**
```
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Extracting footer from page 0
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Footer region detected: BBox { x: 0.1, y: 0.93, width: 0.8, height: 0.05 }
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Footer text: '2025-10-01    23 82 16 – Heating Water Coils - Page 2 of 3'
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Parsed section ID: SectionId { value: "23 82 16", confidence: 0.98 }
[2026-01-23T10:30:00Z INFO conset_pdf::furniture] Successfully extracted footer from page 0: section_id=23 82 16, confidence=0.98
```

**If something breaks, you can see exactly where.**

---

### Structured Logging

**For machine-parseable logs, use structured logging:**

```rust
use slog::{debug, info, o, Logger};

pub fn extract_footer(page: &Page, logger: &Logger) -> Result<Footer> {
    let log = logger.new(o!("page" => page.index, "function" => "extract_footer"));
    
    debug!(log, "Starting footer extraction");
    
    let footer_region = detect_footer_region(page);
    debug!(log, "Footer region detected";
           "bbox" => ?footer_region.bbox,
           "confidence" => footer_region.confidence);
    
    let text = extract_text_from_region(page, &footer_region);
    debug!(log, "Footer text extracted"; "text" => &text);
    
    let section_id = parse_section_id(&text)?;
    info!(log, "Footer extraction complete";
          "section_id" => &section_id.value,
          "confidence" => section_id.confidence);
    
    Ok(Footer { section_id, region: footer_region, raw_text: text })
}
```

---

## Error Handling

### Error Types

**Define domain-specific error types:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("Missing section ID delimiter in footer: '{footer}'")]
    MissingSectionDelimiter {
        footer: String,
        confidence: f32,
    },
    
    #[error("Invalid section ID format: expected XX YY ZZ, got '{candidate}' in footer '{footer}'")]
    InvalidSectionIdFormat {
        candidate: String,
        footer: String,
        confidence: f32,
    },
    
    #[error("Low confidence section ID: {candidate} (confidence: {confidence:.2}, threshold: 0.80)")]
    LowConfidenceSectionId {
        candidate: String,
        confidence: f32,
    },
    
    #[error("PDF extraction failed on page {page_index}: {source}")]
    PdfExtractionFailed {
        page_index: usize,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

### Error Context

**Always provide context when converting errors:**

```rust
use anyhow::{Context, Result};

pub fn load_pdf(path: &Path) -> Result<Document> {
    let bytes = std::fs::read(path)
        .context(format!("Failed to read PDF file: {}", path.display()))?;
    
    let doc = pdfium::Document::from_bytes(&bytes)
        .context(format!("Failed to parse PDF: {}", path.display()))?;
    
    if doc.page_count() == 0 {
        anyhow::bail!("PDF has no pages: {}", path.display());
    }
    
    Ok(doc)
}
```

---

### Error Messages

**Make error messages actionable:**

```rust
// ❌ BAD - Vague, no context
return Err(ExtractionError::ParseFailed);

// ❌ BAD - Technical jargon, no solution
return Err(ExtractionError::RegexMatchFailure { pattern: r"^\d{2}\s\d{2}\s\d{2}" });

// ✅ GOOD - Clear, actionable, contextual
return Err(ExtractionError::InvalidSectionIdFormat {
    candidate: footer_text.to_string(),
    footer: full_footer.to_string(),
    confidence: 0.3,
    help: "Section ID should be in format 'XX YY ZZ' (e.g., '23 82 16'). \
           Check if this is a spec document with standard MasterFormat sections.",
});
```

---

## Documentation Requirements

### Inline Documentation

**Every public function must have documentation:**

```rust
/// Extracts the footer region from a page.
///
/// This function identifies the footer band (typically bottom 15% of page)
/// and extracts the text content within that region. The footer is expected
/// to contain the section ID, date, and page number.
///
/// # Arguments
///
/// * `page` - The page to extract the footer from
///
/// # Returns
///
/// Returns `Ok(Footer)` if a valid footer is found with confidence ≥0.80,
/// or `Err(ExtractionError)` if:
/// - No footer pattern is detected
/// - Footer text doesn't match expected format
/// - Confidence is below threshold
///
/// # Examples
///
/// ```
/// use conset_pdf::furniture::extract_footer;
///
/// let page = load_page("test.pdf", 0)?;
/// let footer = extract_footer(&page)?;
/// assert_eq!(footer.section_id, "23 82 16");
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The page has no text in the footer region
/// - The footer text doesn't contain a valid section ID
/// - The confidence score is below 0.80
pub fn extract_footer(page: &Page) -> Result<Footer, ExtractionError> {
    // Implementation...
}
```

---

### Module Documentation

**Every module must have overview documentation:**

```rust
//! Furniture detection for spec documents.
//!
//! This module identifies "chrome" regions in spec PDFs that should be
//! excluded from content parsing. Chrome includes:
//! - Headers (project name, firm logo, project number)
//! - Footers (date, section ID, section title, page counter)
//! - Addendum markers
//!
//! The furniture detection uses pattern matching on known footer formats
//! and validates results using page-in-section counters. The primary
//! detection strategy is "footer-first oracle" because spec footers are
//! the most reliable source of section boundaries.
//!
//! # Examples
//!
//! ```
//! use conset_pdf::furniture::FurnitureDetector;
//!
//! let detector = FurnitureDetector::new();
//! let transcript = load_transcript("test.pdf")?;
//! let furniture = detector.detect(&transcript)?;
//!
//! for region in furniture.footer_regions {
//!     println!("Footer on page {}: {}", region.page_index, region.section_id);
//! }
//! ```
//!
//! # Architecture
//!
//! Furniture detection operates in stages:
//! 1. Detect footer band regions (bottom 15% of each page)
//! 2. Extract text from footer bands
//! 3. Match footer text against known patterns
//! 4. Validate section boundaries using page counters
//! 5. Compute confidence scores based on pattern coverage
//!
//! See [`FurnitureDetector`] for the main entry point.

pub struct FurnitureDetector { }
```

---

### README.md

**Every crate must have a README.md:**

```markdown
# conset-pdf-furniture

Furniture (chrome) detection for AEC specification documents.

## Overview

This crate identifies headers, footers, and other "chrome" regions in spec PDFs
that should be excluded from content parsing. The primary strategy is "footer-first
oracle" which uses footer patterns as the ground truth for section boundaries.

## Usage

```rust
use conset_pdf_furniture::FurnitureDetector;

let detector = FurnitureDetector::new();
let transcript = load_transcript("spec.pdf")?;
let furniture = detector.detect(&transcript)?;
```

## Features

- Footer pattern matching (supports common AEC firm formats)
- Page-in-section counter validation
- Confidence scoring
- Visual overlay generation for debugging

## Testing

```bash
cargo test
cargo test --ignored  # Run torture corpus tests
```

## License

Apache-2.0
```

---

## Git Workflow

### Branch Strategy

**Main branches:**
- `main` - Production-ready code
- `develop` - Integration branch for features

**Feature branches:**
- `feature/<phase>-<description>` (e.g., `feature/phase1-layout-extraction`)
- `fix/<issue>-<description>` (e.g., `fix/footer-parsing-crash`)
- `docs/<description>` (e.g., `docs/update-readme`)

**Workflow:**
```bash
# Start new feature
git checkout -b feature/phase1-layout-extraction

# Work in micro-tasks, commit frequently
git add .
git commit -m "Add LayoutTranscript struct"

# Push and create PR
git push origin feature/phase1-layout-extraction
# Create PR on GitHub: feature/phase1-layout-extraction -> develop
```

---

### Commit Messages

**Format:** Conventional Commits

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature (e.g., `feat(furniture): add footer detection`)
- `fix`: Bug fix (e.g., `fix(parsing): handle missing section ID`)
- `docs`: Documentation (e.g., `docs(readme): add usage examples`)
- `test`: Test additions (e.g., `test(furniture): add edge case tests`)
- `refactor`: Code refactoring (e.g., `refactor(ir): simplify bbox normalization`)
- `perf`: Performance improvement (e.g., `perf(extraction): optimize span clustering`)
- `chore`: Build/tooling changes (e.g., `chore(ci): add clippy checks`)

**Examples:**

```
feat(furniture): add footer detection

- Implement FurnitureDetector struct
- Add regex patterns for common footer formats
- Validate footer regions using page counters
- Compute confidence scores based on pattern coverage

Tests verify 95% coverage on tier1 corpus.
```

```
fix(parsing): handle missing section ID gracefully

Previously, missing section IDs would panic. Now returns
ExtractionError::MissingSectionId with context.

Fixes #42
```

```
test(furniture): add edge case for rotated pages

Adds test for footer detection on pages rotated 90 degrees.
Validates that coordinate normalization handles rotation.
```

---

### Pull Request Checklist

**Before submitting PR:**

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated (inline + README if needed)
- [ ] Debug logging added at key decision points
- [ ] Commit messages follow conventional format
- [ ] PR description includes context and testing notes

**PR template:**
```markdown
## Description

Brief description of what this PR does.

## Changes

- Added X
- Fixed Y
- Refactored Z

## Testing

- Unit tests: 15 new tests added, all passing
- Integration tests: Phase 1 extraction tested on 5 sample PDFs
- Torture corpus: Tier1 still passes at 100%

## Checklist

- [x] Tests pass
- [x] Clippy clean
- [x] Formatted
- [x] Docs updated
- [x] Debug logging added
```

---

## AI Coding Agent Workflow

**This section is critical for effective AI agent collaboration.**

### Micro-Task Development

**Core principle:** Break every phase into tiny, testable increments (50-100 lines per task).

**Workflow (15-30 minutes per task):**

1. **Write micro-task spec** (1 paragraph, super specific)
2. **Ask agent to write test first** (TDD)
3. **Review the test** (does it make sense?)
4. **Ask agent to implement** (just enough to pass)
5. **Run the test** (red → green)
6. **Ask agent to explain code** (rubber duck review)
7. **Add debug logging** (trace execution)
8. **Commit with clear message**
9. **Ask for next task suggestion**

---

### Example: Phase 1 Micro-Tasks

**DON'T do this:**
```
You: "Build Phase 1: Layout Extraction"
Agent: [Generates 3000 lines across 15 files]
You: "WTF just happened?"
```

**DO this instead:**

```
Task 1.1: "Create LayoutTranscript data structure"
→ Agent generates: layout.rs (50 lines)
→ You review: "Does this struct make sense?"
→ Test: cargo build
→ Commit: "Add LayoutTranscript struct"

Task 1.2: "Add PDFium text extraction (just one page)"
→ Agent generates: extract.rs (80 lines)
→ You review: "Does it extract text from page 1?"
→ Test: cargo test test_extract_page_1
→ Commit: "Add basic text extraction"

Task 1.3: "Add bbox extraction for spans"
→ Agent generates: extract.rs (40 lines modified)
→ You review: "Are bboxes reasonable?"
→ Test: cargo test test_bbox_extraction
→ Commit: "Add bbox extraction"

Task 1.4: "Add coordinate normalization"
→ Agent generates: normalize.rs (60 lines)
→ You review: "Does Y flip correctly?"
→ Test: cargo test test_coordinate_normalization
→ Commit: "Add coordinate normalization"
```

---

### Test-First Workflow

**Ask agent to write test first, then implement:**

```
You: "Write a test that verifies we can extract text from page 1 of test.pdf"

Agent generates test:
```rust
#[test]
fn test_extract_text_page_1() {
    let pdf = load_pdf("tests/fixtures/test.pdf");
    let page1 = extract_page(&pdf, 0);
    assert!(page1.text.contains("RWB Project"));
    assert!(page1.spans.len() > 10);
}
```

You: "Good. Now implement the function to make this test pass."

Agent generates implementation:
```rust
pub fn extract_page(pdf: &Document, page_index: usize) -> Page {
    // Just enough code to pass the test
}
```

You: "Run test. Does it pass?"
Agent: "Yes, test passes."
You: "Commit."
```

---

### Rubber Duck Reviews

**After agent generates code, ask for explanations:**

```
You: "Explain what this normalize_coordinates function does, line by line."

Agent:
"Line 5: We get the page height
Line 7: We flip the Y coordinate by subtracting from page height
Line 9: We return the normalized bbox
..."

You: "Why do we flip the Y coordinate?"

Agent:
"PDFium uses bottom-left origin with Y increasing upward.
We normalize to top-left origin with Y increasing downward
so that 0.0 is at the top and 1.0 is at the bottom."

You: "What happens if a span extends off the page?"

Agent:
"Currently it would result in negative Y coordinates. We should
add bounds checking and either clamp or emit a warning."

You: "Add bounds checking with warning."
```

---

### Debug Logging Requirement

**Always ask agent to add debug logging:**

```
You: "Add debug logging to the extract_footer function at every decision point."

Agent generates:
```rust
pub fn extract_footer(page: &Page) -> Result<Footer> {
    debug!("Extracting footer from page {}", page.index);
    
    let footer_region = detect_footer_region(page);
    debug!("Footer region: {:?}", footer_region);
    
    let text = extract_text_from_region(page, footer_region);
    debug!("Footer text: {}", text);
    
    let section_id = parse_section_id(&text)?;
    debug!("Parsed section ID: {:?}", section_id);
    
    Ok(Footer { section_id, ... })
}
```
```

---

### Staying in Control

**Key practices:**

1. **Micro-tasks only** (50-100 lines per task, not 3000)
2. **Test first** (write test, then implement)
3. **Review everything** (never merge without understanding)
4. **Debug hooks everywhere** (log at every decision point)
5. **Commit frequently** (after every micro-task, 15-30 min)
6. **Rubber duck** (ask agent to explain code)
7. **Validate incrementally** (run tests after each task)
8. **Ask "what's next"** (let agent suggest next micro-task)

**If you follow this workflow, you'll never be lost in a sea of generated code again.**

---

## Code Review Checklist

**Before approving PR:**

### Functionality
- [ ] Code does what it's supposed to do
- [ ] Edge cases handled
- [ ] Error cases handled
- [ ] No panics or unwraps in production paths

### Testing
- [ ] All tests pass
- [ ] New tests added for new functionality
- [ ] Edge cases tested
- [ ] Regression tests still pass

### Code Quality
- [ ] Follows naming conventions
- [ ] No clippy warnings
- [ ] Formatted with `cargo fmt`
- [ ] No unsafe code (or justified + documented)
- [ ] Type-safe (newtype pattern used)

### Documentation
- [ ] Public functions documented
- [ ] Module docs updated if needed
- [ ] README updated if needed
- [ ] Comments explain "why", not "what"

### Debug Logging
- [ ] Debug logging added at key decision points
- [ ] Log levels appropriate (debug/info/warn/error)
- [ ] Sensitive data not logged

### Determinism
- [ ] No randomness
- [ ] Tie-breaks deterministic
- [ ] Output reproducible

### Error Handling
- [ ] Errors returned, not panicked
- [ ] Error messages actionable
- [ ] Context provided with errors

### Git
- [ ] Commit messages follow convention
- [ ] Commits focused (one logical change per commit)
- [ ] No merge conflicts

---

## Performance Guidelines

### Premature Optimization

**Rule:** Don't optimize until you profile.

**Process:**
1. Write correct code first
2. Add tests
3. Profile with real data
4. Optimize hot paths only
5. Re-profile to verify improvement
6. Document trade-offs

**Example:**
```rust
// Phase 1: Correct implementation
pub fn cluster_spans(spans: &[Span]) -> Vec<Vec<&Span>> {
    // Clear, correct O(n²) algorithm
    let mut clusters = Vec::new();
    for span in spans {
        // ... find matching cluster ...
    }
    clusters
}

// Phase 2: Profile shows this is a bottleneck

// Phase 3: Optimize with spatial index
pub fn cluster_spans_optimized(spans: &[Span]) -> Vec<Vec<&Span>> {
    // O(n log n) with R-tree
    let tree = RTree::from_spans(spans);
    // ... use spatial index ...
}

// Phase 4: Benchmark shows 10x speedup
// Phase 5: Document in commit message
```

---

### Performance Budgets

**Maximum times (on typical hardware):**
- Layout extraction: <5 sec per 100 pages
- Section segmentation: <1 sec per section
- Paragraph parsing: <2 sec per section
- Section regeneration: <5 sec per section
- Full spec addendum: <30 sec for typical 300-page spec

**If exceeding budget:**
1. Profile to find bottleneck
2. Document issue
3. Optimize hot path
4. Verify improvement
5. Update budget if needed

---

### Memory Usage

**Guidelines:**
- Maximum memory: 500 MB per document (typical)
- Streaming for large documents (>1000 pages)
- No unbounded collections
- Clean up resources promptly

**Example:**
```rust
// ❌ BAD - Loads entire document into memory
pub fn extract_all_pages(pdf: &Document) -> Vec<Page> {
    (0..pdf.page_count())
        .map(|i| extract_page(pdf, i))
        .collect()
}

// ✅ GOOD - Processes incrementally
pub fn extract_all_pages(pdf: &Document) -> impl Iterator<Item = Page> + '_ {
    (0..pdf.page_count())
        .map(move |i| extract_page(pdf, i))
}
```

---

## Appendix: Quick Reference

### Principles Checklist
- [ ] Deterministic (no randomness, reproducible)
- [ ] Correct (readable, tested, validated)
- [ ] Auditable (logged, measured, explainable)
- [ ] Explicit failures (no silent errors)

### Code Checklist
- [ ] Named correctly (PascalCase types, snake_case functions)
- [ ] Type-safe (newtype pattern for domain types)
- [ ] Error handling (Result types, context provided)
- [ ] No unsafe (or justified + documented)

### Testing Checklist
- [ ] Test first (TDD workflow)
- [ ] Test coverage (≥85%)
- [ ] Edge cases tested
- [ ] Deterministic output verified

### Debug Logging Checklist
- [ ] Logged at key decision points
- [ ] Appropriate log levels
- [ ] No sensitive data
- [ ] Runnable with RUST_LOG=debug

### Documentation Checklist
- [ ] Public functions documented
- [ ] Module overview documented
- [ ] README updated
- [ ] Comments explain "why"

### Git Checklist
- [ ] Conventional commits
- [ ] Micro-tasks (15-30 min)
- [ ] Commits focused
- [ ] PR description clear

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 4.0.0 | 2026-01-21 | Initial release (high-level principles) |
| 4.2.0 | 2026-01-23 | **Major revision aligned with Master Plan V4.2.** Added: (1) Tactical debug logging standards with examples, (2) Micro-task development workflow for AI agents, (3) Test-driven development workflow, (4) Rubber duck review process, (5) Explicit code examples (good vs bad), (6) Expanded error handling section, (7) Structured logging guidance, (8) Performance guidelines, (9) Memory usage guidelines. Reorganized: moved governance to separate document, focused on day-to-day coding practices. |

---

**Status:** ✅ ACTIVE  
**Owner:** HLLMR LLC  
**Last Updated:** January 23, 2026  
**Version:** 4.2.0

---

**End of DEV_STANDARDS Document**

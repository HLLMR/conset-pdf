# Contributing to Conset PDF

**Our Philosophy:** Quality over speed. Accuracy over shortcuts. Determinism over convenience.

This document describes how to contribute to Conset PDF. It's the practical guide to turning ideas into production code. Follow these standards not because they're rules, but because they preserve correctness and auditability in a mission-critical system.

**Reference:** [DEV_STANDARDS_v4_2.md](docs/DEV_STANDARDS_v4_2.md) is the authoritative source for all standards. This document summarizes key requirements and adds contribution-specific guidance.

---

## 1. Code Style

### Rustfmt Configuration

All Rust code is formatted with `rustfmt`. This is non-negotiable.

```bash
# Before committing, format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

**Configuration:** [rustfmt.toml](rustfmt.toml) in the workspace root defines standards. Do not override these locally.

---

### Clippy Lints

All code must pass Clippy without warnings.

```bash
# Check for all clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix --allow-dirty
```

**Enabled lints:**
- Pedantic lints (warnings by default)
- All `clippy::*` categories except explicitly allowed
- Module name repetitions allowed (AEC domain requires context in names)

**If you disagree with a lint:**
1. Document the exception with `#[allow(...)]` and rationale
2. Mention it in your PR description
3. Request approval from code review

**Example:**
```rust
// ❌ BAD - No explanation
#[allow(clippy::module_name_repetitions)]
pub struct FooterFooter;

// ✅ GOOD - Rationale documented
/// Represents a detected footer region in a spec document.
/// Named "Footer" to match AEC domain terminology (MasterFormat footer sections).
#[allow(clippy::module_name_repetitions)]
pub struct FooterFooter;
```

---

### Naming Conventions

Follow these strictly—consistency is correctness.

**Types (PascalCase):**
```rust
pub struct LayoutTranscript { }
pub enum ExtractionError { }
pub trait Extractable { }
```

**Functions & variables (snake_case):**
```rust
pub fn extract_footer(page: &Page) -> Result<Footer> { }
pub fn normalize_coordinates(bbox: BBox) -> BBox { }
let section_id = "23 82 16";
let confidence_score = 0.95;
```

**Constants (SCREAMING_SNAKE_CASE):**
```rust
const FOOTER_BAND_START: f32 = 0.85;
const CONFIDENCE_THRESHOLD: f32 = 0.80;
const MAX_PAGE_SIZE: usize = 10_000;
```

---

### Module Organization

Every file follows this structure:

```rust
//! Module-level documentation explaining purpose and usage.

// Standard library imports
use std::collections::HashMap;

// Third-party crate imports
use serde::{Deserialize, Serialize};

// Internal crate imports (grouped by path)
use crate::ir::{Page, Span};
use crate::audit::AuditEvent;

// Public interface first
pub struct MyStruct { }

impl MyStruct {
    pub fn new() -> Self { }
    pub fn main_operation(&self) -> Result<Output> { }
}

// Private implementation details
struct InternalHelper { }

// Tests at end
#[cfg(test)]
mod tests { }
```

---

### Type Safety

**Use newtypes to prevent bugs:**

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
    
    pub fn is_high(&self) -> bool {
        self.0 >= 0.95
    }
}

pub fn calculate_confidence(correct: usize, total: usize) -> Confidence {
    let ratio = correct as f32 / total as f32;
    Confidence::new(ratio).expect("ratio must be 0.0-1.0")
}
```

---

## 2. Testing Requirements

### Coverage Minimums

**Non-negotiable:**
- **Unit tests:** Every public function must have at least one test
- **Line coverage:** Minimum 85% (measure with `cargo tarpaulin`)
- **Integration tests:** Every phase/pipeline stage must have integration tests
- **Regression tests:** All tests must pass on torture corpus (≥95% pass rate)

```bash
# Check line coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/index.html
```

---

### Test-Driven Development (TDD)

**This is not optional—it's how we ensure correctness.**

**Workflow:**
1. Write the test FIRST (defines what you want)
2. Run test (should fail - red)
3. Implement minimum code to pass (green)
4. Refactor if needed
5. Commit

**Why TDD?**
- Forces clear thinking about requirements
- Provides immediate feedback on implementation
- Creates regression tests automatically
- Documents expected behavior

**Example:**
```rust
// Step 1: Write the test first
#[test]
fn test_normalize_coordinates_flips_y_axis() {
    let bbox = BBox { x: 0.0, y: 100.0, width: 50.0, height: 20.0 };
    let page_height = 792.0;
    
    let normalized = normalize_coordinates(bbox, page_height);
    
    // Y should be: page_height - original_y - height = 792 - 100 - 20 = 672
    assert_eq!(normalized.y, 672.0);
    assert_eq!(normalized.x, 0.0);  // X unchanged
    assert_eq!(normalized.width, 50.0);  // Width unchanged
}

// Step 2: Run test (fails - function doesn't exist)
// $ cargo test test_normalize_coordinates_flips_y_axis
// error[E0425]: cannot find function `normalize_coordinates`

// Step 3: Implement minimum code to pass
pub fn normalize_coordinates(bbox: BBox, page_height: f32) -> BBox {
    BBox {
        x: bbox.x,
        y: page_height - bbox.y - bbox.height,  // Flip Y axis
        width: bbox.width,
        height: bbox.height,
    }
}

// Step 4: Test passes (green)
// $ cargo test test_normalize_coordinates_flips_y_axis
// test tests::test_normalize_coordinates_flips_y_axis ... ok

// Step 5: Commit
// $ git commit -m "feat: implement Y-axis coordinate normalization"
```

---

### Test Structure

**Use Arrange-Act-Assert pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test fixtures at top
    fn sample_page() -> Page {
        Page {
            index: 0,
            width: 612.0,
            height: 792.0,
            spans: vec![
                Span {
                    text: "23 82 16 – Heating Water Coils".to_string(),
                    bbox: BBox { x: 0.1, y: 0.9, width: 0.8, height: 0.05 },
                    font: FontInfo::default(),
                },
            ],
        }
    }
    
    // Test naming: test_<function>_<scenario>_<expected_result>
    #[test]
    fn test_extract_footer_with_valid_pattern_returns_section_id() {
        // Arrange: Set up test data
        let page = sample_page();
        let extractor = FooterExtractor::new();
        
        // Act: Execute the function under test
        let result = extractor.extract(&page);
        
        // Assert: Verify the result
        assert!(result.is_ok());
        let footer = result.unwrap();
        assert_eq!(footer.section_id, "23 82 16");
        assert!(footer.confidence >= 0.95);
    }
    
    #[test]
    fn test_extract_footer_with_invalid_pattern_returns_error() {
        // Arrange
        let mut page = sample_page();
        page.spans[0].text = "Invalid footer".to_string();
        let extractor = FooterExtractor::new();
        
        // Act
        let result = extractor.extract(&page);
        
        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ExtractionError::InvalidSectionIdFormat { .. } => {},
            _ => panic!("Expected InvalidSectionIdFormat error"),
        }
    }
    
    #[test]
    fn test_extract_footer_deterministic_output() {
        // Determinism is sacred—same input must produce identical output
        let page = sample_page();
        let extractor = FooterExtractor::new();
        
        let result1 = extractor.extract(&page).unwrap();
        let result2 = extractor.extract(&page).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(result1.confidence, result2.confidence);
    }
}
```

---

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run with output (see debug logs)
cargo test --workspace -- --nocapture

# Run specific crate tests
cargo test -p conset-pdf-ir
cargo test -p conset-pdf-engine

# Run specific test by name
cargo test test_extract_footer_with_valid_pattern

# Run tests in parallel (default) or sequentially
cargo test -- --test-threads=1

# Run integration tests only
cargo test --test '*' --workspace

# Run torture corpus tests (slow)
cargo test --ignored -- --nocapture
```

---

## 3. Commit Message Format

### Conventional Commits

All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) format. This enables automated changelog generation and clear history.

**Format:**
```
<type>: <subject>

<body (optional)>

Fixes: #<issue-number> (optional)
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `test` - Add or update tests
- `docs` - Documentation only
- `refactor` - Code restructuring without behavior change
- `perf` - Performance improvement
- `ci` - CI/CD configuration

**Subject line (50 characters max):**
- Imperative mood ("add", not "added" or "adds")
- No period at end
- Lowercase

**Body (optional but encouraged):**
- Explain *what* and *why*, not *how*
- Wrap at 72 characters
- Separate from subject with blank line

---

### Examples

**Good commits:**
```
feat: implement footer extraction for specifications

- Extract footer band from bottom 15% of page
- Match footer text against known section ID patterns
- Validate section boundaries using page counters
- Compute confidence scores (threshold 0.80)
- Add 50+ test cases covering AEC formats

Fixes: #42
```

```
fix: correct Y-axis coordinate normalization

The previous implementation flipped Y incorrectly for pages
with variable heights. This commit fixes the calculation to:

  normalized_y = page_height - original_y - element_height

Tested against torture corpus tier 1 (100% pass rate).

Fixes: #123
```

```
test: add golden file regression for section parsing

Tests deterministic output by comparing against reference files.
Snapshots stored in tests/golden/ and versioned with code.

Run with: cargo test -- --nocapture
```

**Bad commits:**
```
❌ "fix stuff"  (too vague)
❌ "Fixed footer extraction logic" (wrong mood)
❌ "feat: added a new cool feature" (marketing language)
❌ "Merge branch 'feature/xyz' into main" (auto-generated)
```

---

### Atomic Commits

Each commit should be a **single, complete unit of work**:
- Solves one problem
- All tests pass
- No partial implementations
- Can be reverted without breaking other commits

**Antipatterns:**
```
❌ feat: add extraction framework AND furniture detection AND parsing
✅ feat: add extraction framework
✅ feat: add furniture detection
✅ feat: add parsing
```

---

## 4. Branch Strategy

### Branch Naming

```
main                          # Stable, always builds, all tests pass
  └─ feature/phase1-layout    # New feature (from main)
  └─ fix/footer-parsing       # Bug fix (from main)
  └─ docs/contributing        # Documentation only (from main)
```

**Format:** `<type>/<short-description>`

**Types:**
- `feature/` - New feature (usually aligns with roadmap phase)
- `fix/` - Bug fix
- `docs/` - Documentation only
- `perf/` - Performance improvement
- `refactor/` - Code cleanup (no behavior change)

---

### Rules

1. **Branch from main:** All new branches start from `main`
2. **All tests must pass:** Before merging to main, CI must be green
3. **No force-pushes:** Never force-push to main or shared branches
4. **Delete after merge:** Delete feature branch after PR is merged
5. **One PR per branch:** Don't reuse branches for multiple PRs

---

### Creating a Branch

```bash
# Update main
git checkout main
git pull origin main

# Create feature branch
git checkout -b feature/my-feature

# Make changes, commit, push
git add .
git commit -m "feat: implement my feature"
git push origin feature/my-feature

# Open PR on GitHub
# Once approved and merged, delete branch
git branch -d feature/my-feature
git push origin --delete feature/my-feature
```

---

## 5. Pull Request Process

### Before Opening a PR

**Run all checks locally:**

```bash
# Format code
cargo fmt --all

# Run tests
cargo test --workspace

# Check clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check coverage (target: ≥85%)
cargo tarpaulin --out Html
```

**Commit checklist:**
- [ ] Commit messages follow Conventional Commits format
- [ ] All commits are atomic (each solves one problem)
- [ ] Tests pass locally (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Line coverage ≥85%
- [ ] Determinism preserved (no randomness, no timing dependencies)
- [ ] Confidence scores assigned (if applicable)
- [ ] Audit events logged (if decision-making)

---

### PR Template

Use this template when opening a PR:

```markdown
## Description

[What does this PR do? Provide context and rationale.]

## Type

- [ ] Feature
- [ ] Bug fix
- [ ] Documentation
- [ ] Refactor
- [ ] Performance improvement

## Changes

- [Change 1]
- [Change 2]
- [Change 3]

## Testing

- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing completed
- [ ] Torture corpus tested (if applicable)
- [ ] Golden files updated (if applicable)

## Determinism & Audit

- [ ] No randomness or timing dependencies
- [ ] Confidence scores assigned (if applicable)
- [ ] Audit events logged (if decision-making)
- [ ] Debug logging at all decision points

## Checklist

- [ ] Follows [DEV_STANDARDS_v4_2.md](docs/DEV_STANDARDS_v4_2.md)
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy clean (`cargo clippy`)
- [ ] Line coverage ≥85%
- [ ] Tests pass (`cargo test --workspace`)
- [ ] Commit messages follow format
- [ ] No unsafe code (or pre-approved with rationale)

## Related Issues

Fixes #<issue-number>
Relates to #<issue-number>
```

---

### Code Review Checklist

Reviewers will verify:

- **Correctness:** Does the code solve the stated problem?
- **Standards:** Does it follow DEV_STANDARDS_v4_2?
- **Testing:** Are tests comprehensive and passing?
- **Determinism:** Is randomness/timing eliminated?
- **Audit:** Are confidence scores and logging present?
- **Quality:** Is it readable, documented, and maintainable?
- **Safety:** No unsafe code without approval?

---

### Addressing Feedback

1. Make requested changes
2. Push updates to same branch
3. Respond to review comments
4. Re-request review when ready
5. PR will auto-update on GitHub

---

### Merging

Once approved:
1. Maintainer will **squash-and-merge** to main
2. Commit message will be the PR title
3. Branch will be automatically deleted
4. CI will verify merge doesn't break anything

---

## 6. Micro-Task Development

### What is a Micro-Task?

A **micro-task** is a focused unit of work that:
- Takes 15–30 minutes to implement
- Is 50–100 lines of code
- Can be tested independently
- Has a clear acceptance criterion
- Produces a working intermediate state
- Can be reviewed and merged in one session

**Why?** Small tasks are easier to reason about, less likely to contain bugs, and can be reviewed quickly.

---

### Micro-Task Example: Footer Extraction

**Epic:** "Extract section IDs from specifications"

**Micro-tasks:**
1. **Define types (20 min, 50 lines)**
   - [ ] Create `FooterExtractor` struct
   - [ ] Define `Footer` result type
   - [ ] Define `ExtractionError` enum
   - **Test:** Types compile and serialize

2. **Implement footer region detection (25 min, 80 lines)**
   - [ ] Detect footer band (bottom 15% of page)
   - [ ] Extract text from footer band
   - [ ] Return region with confidence score
   - **Test:** `test_detect_footer_band_returns_region`

3. **Implement section ID extraction (20 min, 60 lines)**
   - [ ] Parse footer text for section ID pattern
   - [ ] Match against MasterFormat patterns
   - [ ] Score match confidence
   - **Test:** `test_extract_section_id_with_valid_format`

4. **Add error handling (15 min, 40 lines)**
   - [ ] Handle missing footer region
   - [ ] Handle invalid patterns
   - [ ] Provide actionable error messages
   - **Test:** `test_extract_section_id_with_invalid_format_returns_error`

5. **Add integration test (20 min, 50 lines)**
   - [ ] Load sample PDF
   - [ ] Extract layout
   - [ ] Run footer extraction
   - [ ] Compare against golden file
   - **Test:** `test_end_to_end_footer_extraction_on_sample_spec`

**Total:** ~2 hours, ~280 lines, 5 independent commits

---

### Breakdown Strategy

When planning work:
1. Identify the feature or fix
2. Break into micro-tasks (each 15–30 min)
3. Prioritize: dependencies first, then most valuable
4. Each task gets its own branch and PR
5. Tasks are reviewed and merged independently

**Tool:** Use `cargo check --lib` frequently to catch compilation errors early.

---

## 7. Rubber Duck Reviews

Before opening a PR, **explain your code out loud** to catch bugs.

### What is a Rubber Duck Review?

A technique where you explain your code line-by-line to someone (or something—historically a rubber duck) to find bugs through explanation.

### Process

1. **Create a simple document or open issue describing:**
   - What the code does
   - Why each decision was made
   - Edge cases you handled
   - Edge cases you *didn't* handle

2. **For each function, trace through:**
   - Input examples
   - Each decision point
   - Expected outputs
   - Error cases

3. **Ask yourself:**
   - Could this fail silently?
   - Did I handle all error cases?
   - Is there a simpler way to write this?
   - Would someone else understand this?
   - Is this deterministic?
   - Did I log at decision points?

### Example

**Code:**
```rust
pub fn extract_footer(page: &Page) -> Result<Footer> {
    let footer_region = detect_footer_region(page)?;
    let text = extract_text_from_region(page, &footer_region);
    let section_id = parse_section_id(&text)?;
    Ok(Footer { section_id, region: footer_region })
}
```

**Rubber Duck Explanation:**
```
This function extracts a footer from a PDF page:

1. It calls detect_footer_region(). If this fails, we return the error (?)
   because a missing footer is a real problem—we can't proceed.

2. It calls extract_text_from_region(). This doesn't return a Result, so it
   must always succeed (or panic). What if the region is empty? 
   
   PROBLEM: We're not checking if text is empty. If the region contains no
   text, we'll pass an empty string to parse_section_id, which will fail.
   Better to validate the text upfront.

3. It calls parse_section_id(&text). If this fails, we return the error
   because we couldn't find a valid section ID.

4. We construct the result.

ISSUES FOUND:
- No validation that text is non-empty before parsing
- No debug logging at decision points
- No confidence score attached to the result
- Error from detect_footer_region() masks the specific failure

FIXES:
- Add validation: if text is empty, return error
- Add debug logging: debug!("Footer text: '{}'", text)
- Include confidence scores in Footer struct
- Provide better error context
```

---

## 8. Debug Logging

### Rule: Log at Every Decision Point

Debug logging is **CRITICAL** when working with AI-generated code. If something breaks, you need to see exactly where.

```rust
use log::{debug, info, warn, error};

pub fn extract_footer(page: &Page) -> Result<Footer, ExtractionError> {
    debug!("Extracting footer from page {}", page.index);
    
    // Step 1: Detect footer region
    let footer_region = detect_footer_region(page)?;
    debug!("Footer region detected: {:?}", footer_region);
    
    if footer_region.confidence < 0.80 {
        warn!("Low confidence footer region on page {}: {:.2}", 
              page.index, footer_region.confidence);
    }
    
    // Step 2: Extract text
    let text = extract_text_from_region(page, &footer_region);
    debug!("Footer text extracted: '{}'", text);
    
    if text.is_empty() {
        return Err(ExtractionError::EmptyFooterRegion {
            page_index: page.index,
        });
    }
    
    // Step 3: Parse section ID
    let section_id = parse_section_id(&text)?;
    debug!("Parsed section ID: {:?}, confidence: {:.2}", 
           section_id.value, section_id.confidence);
    
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

Use the right level for context:

```rust
// DEBUG: Detailed flow, variable values, step-by-step progress
debug!("Processing span {} at bbox {:?}", span.id, span.bbox);
debug!("Confidence score: {:.2}", confidence);
debug!("Parsed {} characters from region", text.len());

// INFO: High-level operations, successful completions
info!("Extracted layout from {} pages", pages.len());
info!("Section segmentation complete: {} sections found", sections.len());
info!("Successfully extracted footer from page {}", page_idx);

// WARN: Low confidence, degraded performance, non-critical issues
warn!("Low confidence footer on page {}: {:.2}", page_idx, confidence);
warn!("Missing section ID on page {}, skipping", page_idx);
warn!("Ambiguous pattern match with multiple candidates");

// ERROR: Failures, errors, critical issues
error!("Failed to extract text from page {}: {}", page_idx, e);
error!("Invalid section ID format: '{}' (expected XX YY ZZ)", raw_text);
error!("PDF parsing failed with {}", e);
```

---

### Running with Debug Logging

```bash
# Enable all debug logs
RUST_LOG=debug cargo run -- extract test.pdf

# Enable debug logs for specific module
RUST_LOG=conset_pdf::furniture=debug cargo run -- extract test.pdf

# Enable info for most, debug for one module
RUST_LOG=info,conset_pdf::furniture=debug cargo run -- extract test.pdf

# Output to file for later analysis
RUST_LOG=debug cargo run -- extract test.pdf 2> debug.log

# Tail the log in real-time (on Unix)
RUST_LOG=debug cargo run -- extract test.pdf 2>&1 | tee debug.log
```

---

### Example Debug Output

```
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Extracting footer from page 0
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Footer region detected: BBox { x: 0.1, y: 0.93, width: 0.8, height: 0.05 }, confidence: 0.98
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Footer text extracted: '2025-10-01    23 82 16 – Heating Water Coils - Page 2 of 3'
[2026-01-23T10:30:00Z DEBUG conset_pdf::furniture] Parsed section ID: "23 82 16", confidence: 0.98
[2026-01-23T10:30:00Z INFO conset_pdf::furniture] Successfully extracted footer from page 0: section_id=23 82 16, confidence=0.98
[2026-01-23T10:30:01Z DEBUG conset_pdf::furniture] Extracting footer from page 1
...
```

**If something breaks, you can see exactly where.**

---

### Structured Logging for Machine Parsing

For JSON-structured logs (useful for log aggregation):

```rust
use slog::{debug, info, o, Logger};

pub fn extract_footer(page: &Page, logger: &Logger) -> Result<Footer> {
    let log = logger.new(o!("page" => page.index, "function" => "extract_footer"));
    
    debug!(log, "Starting footer extraction");
    
    let footer_region = detect_footer_region(page)?;
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

## Quick Reference: Pre-PR Checklist

Before opening a PR:

```bash
# 1. Format code
cargo fmt --all

# 2. Run full test suite
cargo test --workspace

# 3. Check clippy
cargo clippy --all-targets --all-features -- -D warnings

# 4. Verify coverage
cargo tarpaulin --out Html --output-dir coverage
# Open coverage/index.html and verify ≥85%

# 5. Run full integration tests
cargo test --test '*' --workspace

# 6. Check commit messages
git log origin/main..HEAD --oneline
# Verify all follow conventional commits format

# 7. Review your own code
git diff origin/main
# Check for:
# - Debug logging at decision points
# - Confidence scores assigned
# - Error handling
# - Type safety
# - Determinism preserved
```

---

## Help & Questions

- **Documentation:** See [docs/](docs/) directory
- **Standards:** [DEV_STANDARDS_v4_2.md](docs/DEV_STANDARDS_v4_2.md)
- **Architecture:** [ARCHITECTURE_v4_2.md](docs/ARCHITECTURE_v4_2.md)
- **Issues:** [GitHub Issues](https://github.com/your-org/conset-pdf/issues)
- **Discussions:** [GitHub Discussions](https://github.com/your-org/conset-pdf/discussions)

---

**Remember:** We don't care about hard. We don't care about fast. We care about RIGHT.

**Last Updated:** January 28, 2026 | **Owned By:** Development Team

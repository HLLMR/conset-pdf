# Conset PDF Test Fixtures

This directory contains the test PDF corpus for validating the Conset PDF extraction engine across different complexity levels and edge cases.

## Torture Corpus Tier System

The fixture corpus is organized into progressive difficulty tiers, each designed to validate specific capabilities and robustness of the extraction engine:

### Tier 1: Baseline (30-40 PDFs)
**Expected Pass Rate: 100%**

Simple, well-formed PDF documents that represent the baseline functionality. These PDFs follow standard PDF specifications and contain:
- Standard text layouts
- Simple tables
- Basic images
- Single-column documents
- Standard fonts

**Use Case**: Regression testing and core functionality validation. All tier 1 fixtures must pass consistently.

### Tier 2: Variation (50-60 PDFs)
**Expected Pass Rate: ≥90%**

PDFs with increased complexity and variation that test robustness. These documents include:
- Multi-column layouts
- Complex tables with merged cells
- Mixed text and images
- Multiple fonts and sizes
- Rotated content
- Embedded forms
- Scanned documents

**Use Case**: Feature completeness testing and edge case handling. Some failures are acceptable but should be investigated.

### Tier 3: Chaos (30-40 PDFs)
**Expected Pass Rate: <50%**

Deliberately challenging and malformed PDFs that stress-test the engine:
- Corrupted or truncated PDFs
- Non-standard encodings
- Unusual font embeddings
- Extreme aspect ratios
- Very dense layouts
- PDFs created by obscure tools
- Documents with errors in the PDF specification

**Use Case**: Stress testing, error handling validation, and graceful degradation. High failure rates are expected and acceptable.

### Holdout: Never-Seen (12-15 PDFs)
**Expected Pass Rate**: Variable (depends on content)

A carefully curated set of PDFs that are never used during development or testing, reserved exclusively for final validation in Phase 8. These documents represent unknown unknowns and provide a true measure of generalization.

**Important**: Do not use holdout fixtures during development or benchmarking. They are sealed for final evaluation only.

## Adding New Fixtures

To add new test PDFs to the corpus:

1. **Categorize the PDF**: Determine which tier it belongs to based on complexity and expected functionality
2. **Verify Format**: Ensure the PDF is valid and can be opened in standard readers
3. **Document the PDF**: Create a metadata file (`PDF_NAME.metadata.json`) with:
   ```json
   {
     "filename": "example.pdf",
     "tier": "tier1",
     "description": "Brief description of content",
     "expected_pages": 5,
     "special_features": ["tables", "images"],
     "known_issues": [],
     "source": "Where the PDF came from (optional)"
   }
   ```
4. **Place in Appropriate Tier**: Add the PDF to the corresponding `tierN/` directory
5. **Update Inventory**: Add an entry to `INVENTORY.md` in each tier directory

## Fixture Management

### File Size Considerations

PDFs are stored using Git LFS (Large File Storage) to keep the repository lightweight. Before committing:
- Ensure `.gitattributes` includes `*.pdf filter=lfs diff=lfs merge=lfs -text`
- PDFs should not be committed directly to git without LFS configured

### Metadata Files

Each tier should maintain an `INVENTORY.md` file listing:
- Filename
- Page count
- Primary features tested
- Any known extraction issues

Example:
```markdown
## Tier 1 Inventory

| Filename | Pages | Features | Status |
|----------|-------|----------|--------|
| simple_text.pdf | 1 | Basic text | ✓ Pass |
| two_column.pdf | 3 | Multi-column | ✓ Pass |
```

## Usage in Tests

### Programmatic Access

```rust
use std::path::Path;

#[test]
fn test_tier1_fixtures() {
    let fixture_dir = Path::new("tests/fixtures/tier1");
    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "pdf") {
            // Test extraction
            let result = extract_pdf(&path);
            assert!(result.is_ok(), "Failed to extract {:?}", path);
        }
    }
}
```

### Running Tier-Specific Tests

```bash
# Test baseline fixtures only
cargo test tier1 -- --nocapture

# Test all tiers
cargo test fixtures -- --nocapture

# Test with specific fixture
cargo test -- --exact test_fixture_example --nocapture
```

## Performance Considerations

- **Tier 1**: Fast baseline (should complete in < 1 second per PDF)
- **Tier 2**: Standard testing (1-5 seconds per PDF)
- **Tier 3**: Extended timeout (> 5 seconds acceptable, up to 30s limit)

## Maintenance

### Quarterly Review

Every quarter:
- Review tier distribution and rebalance if needed
- Evaluate pass rates and adjust tier assignments
- Add new edge cases discovered in production
- Document any fixture additions or removals

### Deprecation

Fixtures that become redundant or outdated should be:
1. Moved to `deprecated/` subdirectory (not deleted)
2. Documented with deprecation reason and date
3. Kept for historical reference and regression analysis

## Contact & Questions

For questions about fixture organization or suggestions for new test cases, please consult the development standards in `/docs/DEV_STANDARDS_v4_2.md`.

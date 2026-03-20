# PDF Locator Strategies Guide

**Component**: Locators (`packages/core/src/locators/`)  
**Purpose**: Detect sheet IDs and section IDs on PDF pages

## Overview

**Locators** identify where sheet/section IDs appear on pages. Different strategies work best for different document layouts.

```
Sheet ID Detection Strategies:

┌──────────────────────────┐
│   ROI Locator (Layout)   │ ← Uses ROI profile from layout
│   High precision, fast   │   Example: bottom-right corner
│   Requires profile       │
└──────────────────────────┘

┌──────────────────────────┐
│  Legacy Titleblock       │ ← Auto-detects title block
│  Moderate precision      │   Falls back if no layout
│  Slower, no config       │
└──────────────────────────┘

┌──────────────────────────┐
│  Composite Locator       │ ← Tries ROI, falls back to legacy
│  Best of both            │   Recommended for general use
│  Configurable fallback   │
└──────────────────────────┘
```

---

## SheetLocator Interface

All locators implement:

```typescript
interface SheetLocator {
  /**
   * Locate sheet ID and title on a page
   */
  locate(page: PageContext): Promise<SheetLocationResult>;
  
  /**
   * Get locator name for debugging
   */
  getName(): string;
}

interface SheetLocationResult {
  id?: string;                    // Detected ID (raw)
  sheetIdNormalized?: string;     // Normalized form (canonical)
  sectionIdNormalized?: string;   // For specs
  title?: string;                 // Detected title
  confidence: number;             // 0.0-1.0
  method: string;                 // 'roi', 'legacy', 'text', etc.
  warnings: string[];             // Issues found
  context?: string;               // Debug info
}
```

---

## ROI-Based Locator

**Class**: `RoiSheetLocator`  
**Strategy**: Use layout profile ROI regions to find ID

### How It Works

1. Load layout profile with ROI coordinates
2. For each page, extract text from ROI region
3. Parse text for sheet ID pattern
4. Return detected ID with high confidence (if found)

### Usage

```typescript
import {
  RoiSheetLocator,
  loadLayoutProfile,
} from '@conset-pdf/core';

// Load profile with ROI definitions
const profile = await loadLayoutProfile('layout-profile.json');

const locator = new RoiSheetLocator(profile);

// Use
const page = await docContext.getPageContext(0);
const result = await locator.locate(page);

console.log(result);
// {
//   id: "A1.0",
//   sheetIdNormalized: "A1.0",
//   title: "Site Plan",
//   confidence: 0.98,
//   method: "roi"
// }
```

### Layout Profile Format

```json
{
  "profileId": "standard-title-block",
  "sheetIdRoi": {
    "x": 0.05,
    "y": 0.85,
    "width": 0.2,
    "height": 0.1
  },
  "sheetTitleRoi": {
    "x": 0.05,
    "y": 0.75,
    "width": 0.9,
    "height": 0.08
  }
}
```

**Coordinates**: Normalized 0-1 where:
- `x=0` = left edge, `x=1` = right edge
- `y=0` = top edge, `y=1` = bottom edge
- `width` and `height` as fractions of page dimensions

### Finding Right ROI Coordinates

1. **Visual inspection**: Open PDF, estimate region position
2. **Measurement tool**: Use PDF reader with ruler overlay
3. **Trial and error**: Start with `x=0.05, y=0.85, width=0.2, height=0.1` and adjust
4. **Testing**: Use `detect` command with different ROI values

```bash
# Test with inline ROI
conset-pdf detect \
  --pdf drawings.pdf \
  --type drawings \
  --sheet-id-roi "0.05,0.85,0.2,0.1" \
  --output test-roi.json

# Check confidence scores in output
# If < 0.8, adjust ROI and retest
```

### ROI Locator Advantages & Disadvantages

| Pros | Cons |
|------|------|
| Fast: Direct text region extraction | Requires profile customization |
| High confidence when profile accurate | Fails on non-standard layouts |
| Works with rotated pages | No fallback if region empty |

---

## Legacy Titleblock Locator

**Class**: `LegacyTitleblockLocator`  
**Strategy**: Auto-detect title block, extract ID from it

### How It Works

1. Scan page for title block (rectangles, boxes, borders)
2. Identify text regions within title block
3. Parse regions for sheet ID pattern
4. Cache detection for reuse

### Usage

```typescript
import { LegacyTitleblockLocator } from '@conset-pdf/core';

const locator = new LegacyTitleblockLocator();

// Use
const page = await docContext.getPageContext(0);
const result = await locator.locate(page);

console.log(result);
// {
//   id: "A1.0",
//   sheetIdNormalized: "A1.0",
//   confidence: 0.75,       // Lower than ROI
//   method: "legacy",
//   context: "Found in bottom-right title block"
// }
```

### Legacy Locator Advantages & Disadvantages

| Pros | Cons |
|------|------|
| No configuration needed | Slower than ROI |
| Works with any title block layout | Lower confidence scores |
| Automatic fallback solution | Fails on non-standard formats |

---

## Composite Locator

**Class**: `CompositeLocator`  
**Strategy**: Try ROI first, fall back to legacy if not found

### How It Works

1. Try ROI locator (if profile available)
2. If ROI returns no ID, use legacy locator
3. Return best result (highest confidence)

### Usage

```typescript
import {
  CompositeLocator,
  RoiSheetLocator,
  LegacyTitleblockLocator,
  loadLayoutProfile,
} from '@conset-pdf/core';

const profile = await loadLayoutProfile('profile.json');
const roi = new RoiSheetLocator(profile);
const legacy = new LegacyTitleblockLocator();

const composite = new CompositeLocator(roi, legacy);

// Use
const result = await composite.locate(page);

// If ROI finds ID, returns that. Otherwise tries legacy.
```

### When to Use Composite

✓ **Recommended for production** - handles mixed layouts  
✓ Best of both strategies  
✓ Graceful fallback

---

## Specs-Specific Locators

### SpecsSectionLocator

Text-based extractor for MasterFormat section IDs.

```typescript
import { SpecsSectionLocator } from '@conset-pdf/core';

const locator = new SpecsSectionLocator();

const result = await locator.locate(page);
// Returns: sectionIdNormalized = "23 09 00"
```

**How It Works**:
1. Extract all text from page
2. Search for MasterFormat patterns (NN NN NN)
3. Return first valid section ID found

### RoiSpecsSectionLocator

ROI-based extraction for specs (if you know where section ID appears).

```typescript
const roi = {
  x: 0.05,
  y: 0.05,
  width: 0.3,
  height: 0.15
};

const locator = new RoiSpecsSectionLocator(roi);
const result = await locator.locate(page);
```

---

## Advanced: Custom Locator

Implement your own detection strategy:

```typescript
import type { SheetLocator, SheetLocationResult } from '@conset-pdf/core';
import type { PageContext } from '@conset-pdf/core';

class MyCustomLocator implements SheetLocator {
  async locate(page: PageContext): Promise<SheetLocationResult> {
    // Get page text
    const text = page.getText();
    
    // Your detection logic
    const id = this.extractMyIdPattern(text);
    
    return {
      id,
      sheetIdNormalized: id?.toUpperCase() || undefined,
      confidence: id ? 0.90 : 0.0,
      method: 'custom',
      warnings: []
    };
  }

  getName(): string {
    return 'custom-locator';
  }

  private extractMyIdPattern(text: string): string | null {
    // Custom regex for your format
    const match = text.match(/ID[\s-]?([A-Z]\d+\.\d+)/i);
    return match ? match[1] : null;
  }
}

// Use it
const locator = new MyCustomLocator();
const result = await locator.locate(pageContext);
```

Then pass to merge operation:

```typescript
import { mergeAddenda } from '@conset-pdf/core';

const report = await mergeAddenda({
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  outputPdfPath: 'result.pdf',
  type: 'drawings',
  locator: new MyCustomLocator()  // Use custom locator
});
```

---

## Confidence Scoring

Location results include `confidence` (0.0-1.0):

| Confidence | Interpretation | Action |
|------------|-----------------|--------|
| 0.95-1.0 | Certain | Use in production |
| 0.80-0.95 | High | Review in merges |
| 0.50-0.80 | Medium | Requires validation |
| < 0.50 | Low | Likely incorrect, skip |

### Using Confidence

```typescript
const locator = new CompositeLocator(roi, legacy);

for (let i = 0; i < pageCount; i++) {
  const page = await docContext.getPageContext(i);
  const result = await locator.locate(page);

  if (result.confidence >= 0.95) {
    console.log(`High confidence: ${result.id}`);
  } else if (result.confidence >= 0.70) {
    console.log(`Medium confidence: ${result.id} - review needed`);
  } else if (result.confidence < 0.5) {
    console.warn(`Low confidence or no ID detected on page ${i + 1}`);
  }
}
```

---

## Testing Locators

### Unit Test

```typescript
import { RoiSheetLocator } from '@conset-pdf/core';
import { DocumentContext } from '@conset-pdf/core';

describe('RoiSheetLocator', () => {
  it('should detect sheet ID from ROI', async () => {
    const profile = {
      profileId: 'test',
      sheetIdRoi: { x: 0.05, y: 0.85, width: 0.2, height: 0.1 }
    };

    const locator = new RoiSheetLocator(profile);
    const docContext = new DocumentContext('test/fixtures/drawing.pdf');
    await docContext.initialize();

    const page = await docContext.getPageContext(0);
    const result = await locator.locate(page);

    expect(result.sheetIdNormalized).toBe('A1.0');
    expect(result.confidence).toBeGreaterThan(0.9);
  });
});
```

### Accuracy Metrics

```typescript
async function evaluateLocator(
  locator: SheetLocator,
  pdfPath: string,
  expectedIds: string[]  // Ground truth
): Promise<{
  accuracy: number;      // % of correctly detected IDs
  precision: number;     // % of detections that were correct
}> {
  const docContext = new DocumentContext(pdfPath);
  await docContext.initialize();

  let correct = 0;
  let detected = 0;

  for (let i = 0; i < Math.min(100, docContext.pageCount); i++) {
    const page = await docContext.getPageContext(i);
    const result = await locator.locate(page);

    detected += result.id ? 1 : 0;
    if (result.sheetIdNormalized === expectedIds[i]) {
      correct++;
    }
  }

  return {
    accuracy: correct / expectedIds.length,
    precision: correct / detected
  };
}

// Use it
const roi = new RoiSheetLocator(profile);
const metrics = await evaluateLocator(roi, 'test.pdf', [
  'A1.0', 'A1.1', 'A2.0', /* ... */
]);

console.log(`Accuracy: ${(metrics.accuracy * 100).toFixed(1)}%`);
```

---

## Performance Optimization

### Batch Location Detection

```typescript
// ✓ Good: Extract all pages first, then locate
const pageIndexes = Array.from({ length: 50 }, (_, i) => i);
await docContext.extractTextForPages(pageIndexes);

const results = await Promise.all(
  pageIndexes.map(async (i) => {
    const page = await docContext.getPageContext(i);
    return locator.locate(page);  // Now fast, no I/O
  })
);
```

### Caching

```typescript
// ROI and legacy locators cache internally
// Reusing same locator instance is faster

const locator = new CompositeLocator(roi, legacy);

// First call - slower
let result1 = await locator.locate(page);

// Second call - cached title block detection
let result2 = await locator.locate(page);
```

---

## Troubleshooting

### Issue: "No ID detected" on every page

**Possible Causes**:
- ROI region empty (text not in that area)
- Sheet ID pattern doesn't match document format
- Text extraction poor (PDF.js falling back)

**Solutions**:
1. Verify layout profile coordinates are correct
2. Check text extraction with plain `getText()`:
   ```typescript
   const page = await docContext.getPageContext(0);
   const text = page.getText();
   console.log(text);  // Is sheet ID present?
   ```
3. Try legacy locator as fallback
4. Check if PyMuPDF is installed (better text detection)

### Issue: Low confidence scores (< 0.70)

**Possible Causes**:
- Using legacy locator on standard layout
- ROI region partially misaligned
- PDF text quality poor

**Solutions**:
1. Use ROI locator instead of legacy
2. Fine-tune ROI coordinates
3. Check PDF for text layer vs. scanned image:
   ```typescript
   const docContext = new DocumentContext('doc.pdf');
   await docContext.initialize();
   const page = await docContext.getPageContext(0);
   const text = page.getText();
   if (text.length < 50) {
     console.warn('PDF may be scanned/image-based');
   }
   ```

### Issue: Different IDs detected with different locators

**Example**:
- ROI found: "A1-0" (confidence 0.92)
- Legacy found: "A10" (confidence 0.75)

**Resolution**:
- Both are valid detections, different parsers
- Normalization should reconcile them
- Higher confidence score is preferred

---

## Best Practices

### 1. Always Use Composite Locator in Production

```typescript
// ✓ Good
const composite = new CompositeLocator(roi, legacy);
const result = await composite.locate(page);

// ✗ Risky - single point of failure
const roi = new RoiSheetLocator(profile);
const result = await roi.locate(page);
```

### 2. Validate Confidence Before Use

```typescript
const result = await locator.locate(page);

if (result.confidence < 0.5) {
  // Mark for manual review
  issues.push({
    page: i + 1,
    message: 'Sheet ID detection failed or low confidence',
    suggested: result.id,
    confidence: result.confidence
  });
} else {
  acceptedIds[i] = result.sheetIdNormalized;
}
```

### 3. Extract Text Batch Before Location

```typescript
// ✓ Good: One load, fast location detection
const indexes = Array.from({ length: 100 }, (_, i) => i);
await docContext.extractTextForPages(indexes);

const locations = await Promise.all(
  indexes.map(i => locator.locate(
    await docContext.getPageContext(i)
  ))
);

// ✗ Bad: Loads extraction for each locate call
for (let i = 0; i < 100; i++) {
  const location = await locator.locate(
    await docContext.getPageContext(i)  // Triggers load if not cached
  );
}
```

### 4. Profile Different Document Sets

```typescript
// Each document format may need custom profile
const profiles = {
  'acme-arch': { /* ROI coords for Acme projects */ },
  'standard': { /* Generic layout */ },
  'legacy': { /* Older format */ }
};

// Detect which format and use appropriate profile
const profile = profiles[detectFormat(samplePages)];
const locator = new RoiSheetLocator(profile);
```

---

## See Also

- [Layout Profiles](./LAYOUTS.md) - Creating and managing profiles
- [Core API](./CORE_API.md#sheet-locators) - API reference
- [Tests](../tests/locators/) - Real test examples

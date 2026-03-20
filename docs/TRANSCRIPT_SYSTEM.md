# Transcript Extraction System

**Component**: Transcript System (`packages/core/src/transcript/`)  
**Purpose**: Abstract PDF text/layout extraction across multiple backends

## Overview

The **Transcript System** abstracts PDF extraction behind a unified interface. Instead of tying code to a specific extraction backend (PyMuPDF, PDF.js), the system defines a **LayoutTranscript** format that all backends produce.

**Key Features**:
- 🔄 Multiple backends (PyMuPDF, PDF.js) with automatic fallback
- 📏 High-fidelity bounding box extraction
- 🎯 Deterministic canonicalization
- 📊 Quality scoring per page
- 💾 Single-load caching via DocumentContext
- 🔍 Layout-aware span extraction

---

## Architecture

```
┌─────────────────────────────────────┐
│    User Code (mergeAddenda, etc.)   │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│      DocumentContext / PageContext   │
│   (caching, instrumentation)         │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│     TranscriptExtractor Interface   │
│   (abstract extraction contract)    │
└────────────┬──────────────┬─────────┘
             │              │
    ┌────────▼──┐    ┌─────▼──────┐
    │  PyMuPDF  │    │  PDF.js    │
    │ (95-99%   │    │ (15-25%    │
    │  accuracy)│    │  accuracy) │
    └───────────┘    └────────────┘

        ↓↓↓ Output ↓↓↓

    LayoutTranscript
      + LayoutPage[]
      + LayoutSpan[]
      + BBox annotations
```

---

## LayoutTranscript Format

The **LayoutTranscript** is the standardized output format. All backends produce this.

```typescript
interface LayoutTranscript {
  filePath: string;
  extractionEngine: string;      // "pymupdf-1.24.1" or "pdfjs-5.4.530"
  extractionDate: string;        // ISO 8601 timestamp
  pages: LayoutPage[];
  metadata: TranscriptMetadata;
}

interface TranscriptMetadata {
  totalPages: number;
  hasTrueTextLayer: boolean;     // Not OCR'd?
  contentHash?: string;          // Deterministic hash
  spanHash?: string;             // Structure hash
}

interface LayoutPage {
  pageNumber: number;            // 1-based
  pageIndex: number;             // 0-based
  width: number;                 // Points (after rotation)
  height: number;
  rotation: number;              // 0, 90, 180, 270
  spans: LayoutSpan[];
  lines?: LayoutLine[];
  images?: LayoutImage[];
  bbox: BBox;
}

interface LayoutSpan {
  text: string;
  bbox: BBox;
  font: {
    name: string;
    size: number;
  };
  color?: string;
  confidence?: number;           // OCR confidence
}

interface BBox {
  x0: number;                    // Canonical: top-left origin
  y0: number;
  x1: number;
  y1: number;
}
```

---

## Extraction Backends

### PyMuPDF Backend

**Status**: Primary, recommended  
**Accuracy**: 95-99% bbox accuracy  
**Speed**: ~500ms/page  
**Requirements**: Python 3.8+ with `pymupdf` package

**Advantages**:
- Highest bounding box accuracy
- Preserves true text structure
- Handles rotations correctly
- Deterministic output

**Disadvantages**:
- Requires external Python dependency
- Slower than PDF.js

**Installation**:
```bash
# Windows
pip install pymupdf

# Or with conda
conda install -c conda-forge pymupdf
```

**Configuration** (if not in PATH):
```bash
export CONSET_PDF_PYMUPDF=/path/to/pymupdf
```

---

### PDF.js Backend

**Status**: Fallback, always available  
**Accuracy**: 15-25% bbox accuracy (unreliable)  
**Speed**: ~50ms/page  
**Requirements**: None (pure JavaScript)

**Advantages**:
- Pure JavaScript (no dependencies)
- Fast extraction
- Works everywhere

**Disadvantages**:
- Poor bounding box reliability
- Not recommended for layout-critical operations
- Falls back on PDF.js when PyMuPDF unavailable

**When Used**:
- Automatic fallback if PyMuPDF unavailable
- Quick previews or data-only extraction
- Production: Use only if PyMuPDF impossible

---

## Using DocumentContext

**DocumentContext** manages PDF caching and provides single-load semantics.

### Initialization

```typescript
import { DocumentContext } from '@conset-pdf/core';

const docContext = new DocumentContext('document.pdf');
await docContext.initialize();

console.log(`Pages: ${docContext.pageCount}`);
```

### Extract Text from Pages

```typescript
// Extract text from multiple pages at once
const pageIndexes = [0, 1, 2, 3, 4];
await docContext.extractTextForPages(pageIndexes);

// Now all those pages are cached, subsequent calls are fast
const page0 = await docContext.getPageContext(0);
const text = page0.getText();
```

### Get Layout-Aware Text

```typescript
const page = await docContext.getPageContext(0);

// Raw text
const plainText = page.getText();

// Layout-aware (with bounding boxes)
const layoutSpans = page.getTextWithLayout();

for (const span of layoutSpans) {
  console.log(`"${span.text}" at (${span.bbox.x0}, ${span.bbox.y0})`);
}
```

### One-Load Guarantee

```typescript
// Loading happens ONCE, even with multiple calls
const doc1 = new DocumentContext('doc.pdf');
const doc2 = new DocumentContext('doc.pdf');

await doc1.initialize();
await doc2.initialize();

const instrumentation1 = doc1.getInstrumentation();
const instrumentation2 = doc2.getInstrumentation();

// Note: Each instance tracks its own loads, but underlying PDF bytes cached globally
```

---

## Advanced: TranscriptExtractor Interface

For custom extraction backends, implement `TranscriptExtractor`:

```typescript
interface TranscriptExtractor {
  /**
   * Extract layout transcript from PDF
   */
  extractTranscript(
    pdfPath: string,
    options?: ExtractOptions
  ): Promise<LayoutTranscript>;

  /**
   * Get engine information
   */
  getEngineInfo(): EngineInfo;
}

interface ExtractOptions {
  pages?: number[];              // Specific pages to extract
  includeImages?: boolean;
  includeLines?: boolean;
  qualityThreshold?: number;     // 0.0-1.0
}

interface EngineInfo {
  name: string;                  // "pymupdf", "pdfjs", etc.
  version: string;
  capabilities: string[];        // ["text", "layout", "images", ...]
}
```

### Creating Custom Extractor

```typescript
class CustomExtractor implements TranscriptExtractor {
  async extractTranscript(
    pdfPath: string,
    options?: ExtractOptions
  ): Promise<LayoutTranscript> {
    // Your extraction logic
    return {
      filePath: pdfPath,
      extractionEngine: 'custom-v1',
      extractionDate: new Date().toISOString(),
      pages: [ /* your pages */ ],
      metadata: { /* ... */ }
    };
  }

  getEngineInfo(): EngineInfo {
    return {
      name: 'custom',
      version: '1.0.0',
      capabilities: ['text', 'layout']
    };
  }
}
```

---

## Quality & Determinism

### Canonicalization

Output is **canonicalized** for determinism:
- ✓ Coordinates normalized to 0-1 range
- ✓ Spans sorted deterministically
- ✓ Rotation normalized to 0 degrees
- ✓ Content hashes for change detection

### Quality Scoring

Each page gets a **confidence score** (0.0-1.0):
- High (0.9-1.0): Clear text, good positioning
- Medium (0.5-0.9): Some issues but usable
- Low (<0.5): Unreliable, may be OCR'd scanned document

```typescript
const transcript = await extractor.extractTranscript('doc.pdf');

for (const page of transcript.pages) {
  console.log(`Page ${page.pageNumber}: quality = ${page.quality}`);
}
```

### Validation Gates

The system applies **quality gates**:
- Bbox accuracy validation
- Span structure validation
- PDF metadata checks

If a page fails validation, it may be marked with `quality: 'low'` or flagged for manual review.

---

## Performance Optimization

### Batch Extraction

```typescript
// ✓ Good: Extract all pages at once
const pageIndexes = Array.from({ length: 100 }, (_, i) => i);
await docContext.extractTextForPages(pageIndexes);

for (let i = 0; i < 100; i++) {
  const page = await docContext.getPageContext(i);
  // Fast: already cached
  processPage(page);
}
```

### Selective Extraction

```typescript
// ✓ Good: Extract only needed pages
const pageIndexes = [0, 5, 10, 15];  // Skip pages 1-4, 6-9, etc.
await docContext.extractTextForPages(pageIndexes);
```

### Parallel Processing

```typescript
// After extraction, process pages in parallel
const results = await Promise.all(
  pageIndexes.map(async (idx) => {
    const page = await docContext.getPageContext(idx);
    return analyzePageContent(page);  // No I/O
  })
);
```

---

## Bounding Box Coordinate System

### Canonical Form

All coordinates use **top-left origin**:
- `x0, y0` = top-left corner
- `x1, y1` = bottom-right corner
- X increases to the right
- Y increases downward

```
(0, 0) ─────────────────────► x
  │
  │  (x0,y0)
  │    ┌─────────┐
  │    │ span    │
  │    │ text    │
  │    └─────────┐(x1,y1)
  │
  ▼
  y
```

### Normalized Coordinates

In **LayoutTranscript**, coordinates are normalized to page dimensions:
- 0.0 = left/top edge
- 1.0 = right/bottom edge

```typescript
// Example: Sheet ID in bottom-right
const sheetIdSpan = spans.find(s =>
  s.bbox.x0 > 0.75 &&
  s.bbox.y0 > 0.85 &&
  s.text.match(/^[A-Z]\d+/)
);
```

---

## Handling Rotation

PDFs can have **rotation metadata** (0°, 90°, 180°, 270°).

The transcript system **normalizes rotation to 0°**:
- All coordinates are in the "visual" space after rotation
- `width` and `height` are swapped if rotation is 90° or 270°
- Extraction backend handles rotation internally

```typescript
const page = transcript.pages[0];

console.log(`Rotation: ${page.rotation}`);  // Always 0 after canonicalization
console.log(`Width: ${page.width}`);         // In visual space
console.log(`Height: ${page.height}`);       // In visual space
```

---

## Text Extraction Methods

### Method 1: Plain Text (No Layout)

```typescript
const page = await docContext.getPageContext(0);
const plainText = page.getText();

console.log(plainText);
// Output: "This is page content\nas continuous text"
```

**Use When**:
- Analyzing text content only
- Searching/parsing text
- No layout information needed

### Method 2: Layout-Aware (With Boxes)

```typescript
const page = await docContext.getPageContext(0);
const spans = page.getTextWithLayout();

for (const span of spans) {
  console.log(`"${span.text}" @ (${span.bbox.x0}, ${span.bbox.y0})`);
}
```

**Use When**:
- Building layout profiles
- Detecting regions (header, footer, body)
- Extracting from specific positions

### Method 3: Full Transcript

```typescript
const transcript = await extractor.extractTranscript('document.pdf');

for (const page of transcript.pages) {
  console.log(`Page ${page.pageNumber}:`);
  for (const span of page.spans) {
    // Full span data: text, bbox, font, color, confidence
    console.log(JSON.stringify(span, null, 2));
  }
}
```

**Use When**:
- Deep analysis required
- Font/color information needed
- Building custom detection algorithms

---

## Troubleshooting

### Issue: "PyMuPDF not found, falling back to PDF.js"

**Cause**: PyMuPDF not installed or not in PATH

**Solution**:
```bash
# Install PyMuPDF
pip install pymupdf

# Or set explicit path
export CONSET_PDF_PYMUPDF=/path/to/pymupdf
```

### Issue: Low bounding box accuracy (< 50%)

**Cause**: Using PDF.js backend instead of PyMuPDF

**Solution**: Install and verify PyMuPDF availability

```typescript
// Check which backend is being used
const info = extractor.getEngineInfo();
console.log(`Using: ${info.name} v${info.version}`);
```

### Issue: Transcript gives different results between runs

**Cause**: Non-deterministic extraction (unlikely but possible)

**Debug**:
```typescript
const hash1 = transcript.metadata.contentHash;
const hash2 = transcript2.metadata.contentHash;

if (hash1 !== hash2) {
  console.warn('Transcripts differ!');
  // Investigate PDF changes or extraction differences
}
```

---

## Testing Extraction Quality

### Validation Test

```typescript
import { DocumentContext } from '@conset-pdf/core';

async function validateExtraction(pdfPath: string) {
  const docContext = new DocumentContext(pdfPath);
  await docContext.initialize();
  
  for (let i = 0; i < docContext.pageCount; i++) {
    const page = await docContext.getPageContext(i);
    const text = page.getText();
    
    if (!text || text.length === 0) {
      console.warn(`Page ${i + 1}: No text extracted`);
    }
  }
}

await validateExtraction('document.pdf');
```

### Determinism Test

```typescript
// Extract twice, compare hashes
const ext = new PyMuPdfExtractor();
const t1 = await ext.extractTranscript('doc.pdf');
const t2 = await ext.extractTranscript('doc.pdf');

console.assert(
  t1.metadata.contentHash === t2.metadata.contentHash,
  'Extraction changed!'
);
```

---

## Performance Metrics

| Operation | PyMuPDF | PDF.js |
|-----------|---------|--------|
| Page Load | ~500ms | ~50ms |
| Bounding Box Accuracy | 95-99% | 15-25% |
| Memory per 100-page PDF | ~200MB | ~300MB |
| Rotation Handling | Perfect | Good |
| Image Extraction | Yes | Limited |
| Font Info | Detailed | Basic |

---

## See Also

- [Core API Documentation](./CORE_API.md#documentpage-context) - DocumentContext API
- [Locators Guide](./LOCATORS.md) - Using extracted text for position detection
- [Testing](../tests/transcript/) - Transcript test cases

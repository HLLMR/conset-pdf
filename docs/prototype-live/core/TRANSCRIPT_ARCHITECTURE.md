# Transcript Architecture

## Overview

The Transcript Architecture provides a **backend-agnostic, transcript-first** approach to PDF extraction. It separates rendering (PDF.js) from extraction (PyMuPDF), enabling high-fidelity layout transcript extraction with 95-99% bbox accuracy required for reliable parsing.

## Core Principle

**Viewer ≠ Extractor**: PDF.js remains for GUI rendering; PyMuPDF becomes the primary extraction engine via Python sidecar pattern.

## Architecture Components

### 1. Transcript Abstraction Layer

The transcript system provides a standardized `LayoutTranscript` format that abstracts away the underlying extraction backend.

**Key Types:**
- `LayoutTranscript`: Complete transcript with pages, spans, and metadata
- `LayoutPage`: Single page with spans, dimensions, and rotation
- `LayoutSpan`: Text span with bbox, font, and style information
- `TranscriptExtractor`: Interface for extraction backends

### 2. Extraction Backends

**PyMuPDF (Primary):**
- High-fidelity extraction using dict/rawdict-first approach
- Extracts spans directly from PyMuPDF's dict structure
- Python sidecar pattern (similar to bookmark-writer.py)

**PDF.js (Fallback):**
- Legacy fallback when PyMuPDF unavailable
- Lower confidence scores due to reduced bbox accuracy

**Extractor Factory:**
- Automatic fallback chain: PyMuPDF → PDF.js
- All transcripts automatically canonicalized

### 3. Canonicalization

All transcripts are canonicalized to ensure deterministic output:

- **Rotation Normalization**: All pages normalized to rotation=0, bboxes transformed
- **Coordinate Normalization**: Consistent top-left origin (y=0 at top)
- **Stable Sort**: Spans sorted deterministically (y, then x)
- **Deterministic Hashes**: `contentHash` and `spanHash` exclude `extractionDate`

### 4. Quality Scoring

Quality metrics and validation gates:

- **Per-page metrics**: Char count, whitespace ratio, replacement chars, ordering sanity
- **Aggregate metrics**: Overall confidence score
- **Quality gates**: Minimum char count, max replacement ratio, min ordering sanity, min confidence

### 5. Candidate Generation

Deterministic pre-analysis for structural elements:

- **Header/footer bands**: Y clustering + repetition detection
- **Font-size clusters**: Typography pattern analysis
- **Heading candidates**: Regex-based heading detection
- **Column hints**: X coordinate clustering for tables
- **Table candidates**: Line density + grid pattern detection

### 6. Profile System

Extended profile types for different document types:

- **SpecProfile**: Section-based specification documents
- **SheetTemplateProfile**: Drawing templates with title blocks
- **EquipmentSubmittalProfile**: Equipment submittal documents

Profile registry with validation gates and automatic matching.

### 7. Privacy Layer (Abstraction)

Privacy-preserving pattern abstraction:

- **TokenVault**: Replaces sensitive content with structural tokens
- **Sanitization**: Deterministic pseudonymization and sampling
- **Privacy modes**: STRICT_STRUCTURE_ONLY, WHITELIST_ANCHORS, FULL_TEXT_OPT_IN

### 8. Enhanced Parsers

**Spec Parser:**
- Chrome removal using candidate detection
- Paragraph normalization (wrap join + hyphen repair)
- Basic table detection

**Schedule Extraction:**
- Geometry-first table builder
- Column/row detection via coordinate clustering
- CSV/JSON export

**Submittal Parser:**
- Packet segmentation using pattern matching
- Key/value field extraction
- Performance table extraction

## Data Flow

```
PDF File
  ↓
[TranscriptExtractor Interface]
  ├─ PyMuPDFExtractor (primary) → Python sidecar (dict/rawdict-first)
  ├─ PDFiumExtractor (fallback) → Python sidecar  
  └─ PDFjsExtractor (legacy) → Direct PDF.js
  ↓
Raw LayoutTranscript (from extractor)
  ↓
CanonicalizeTranscript() → Normalize rotation, coordinates, stable-sort spans, compute deterministic hashes
  ↓
Canonicalized LayoutTranscript (standardized format)
  ↓
DocumentContext (adapted to consume transcript)
  ↓
PageContext (adapted to consume transcript)
  ↓
Locators/Parsers (work on reliable transcripts)
```

## Usage

### Basic Extraction

```typescript
import { createTranscriptExtractor } from '@conset-pdf/core';

const extractor = createTranscriptExtractor();
const transcript = await extractor.extractTranscript('path/to/file.pdf');
```

### Quality Scoring

```typescript
import { scoreTranscriptQuality } from '@conset-pdf/core';

const qualityReport = scoreTranscriptQuality(transcript);
if (qualityReport.passes) {
  // Proceed with parsing
}
```

### Candidate Generation

```typescript
import { generateCandidates } from '@conset-pdf/core';

const candidates = generateCandidates(transcript);
// Access: candidates.headerBands, candidates.headingCandidates, etc.
```

### Privacy-Preserving Abstraction

```typescript
import { sanitizeTranscript, PrivacyMode } from '@conset-pdf/core';

const { abstractTranscript, tokenVault } = sanitizeTranscript(transcript, {
  privacyMode: PrivacyMode.STRICT_STRUCTURE_ONLY,
  sampling: { maxPages: 5, includeChromeBands: true, includeHeadings: true },
});

// Abstract transcript includes:
// - placeholderId (hash-based, stable for identical shapes)
// - tokenShape, charClassFlags, lengthBucket (shape features)
// - repetition metrics (repeatCountDoc, repeatRateDoc, repeatRateByBand)
// - line grouping (page.lines with reading order)
// - band definitions and sampling metadata
// - coordinate system metadata
```

## Determinism

The transcript system ensures deterministic output:

- **Same PDF → Same contentHash**: Excludes `extractionDate` from hash computation
- **Same PDF → Same span structure**: Span counts and IDs are stable
- **Same PDF → Same bboxes**: Bboxes align within tolerance across extractions

## Packaging

**Phase 1 (Current)**: External Python required
- Python 3.8+ must be installed
- PyMuPDF installed via `pip install pymupdf>=1.24.0`
- Clear error messages if Python/PyMuPDF not found

**Phase 2 (Future)**: Bundled runtime / local service
- Options: Bundled Python runtime, local HTTP service, or hybrid approach
- Eliminates external dependency for end-user distribution

## Migration from PDF.js

The transcript system is designed for backward compatibility:

- `DocumentContext` now uses transcripts internally
- Existing APIs remain unchanged
- PDF.js still used for bookmarks (temporary, until migrated)
- Gradual migration path with feature flags

## Testing

See `tests/transcript/` for:
- Determinism tests
- Quality scoring tests
- Extraction accuracy tests

## See Also

- [MIGRATION_V3.md](./MIGRATION_V3.md) - Migration guide

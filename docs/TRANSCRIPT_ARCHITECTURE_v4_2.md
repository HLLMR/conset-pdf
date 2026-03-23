# Conset PDF: Transcript Architecture V4.2

**Version:** 4.2.0  
**Date:** January 23, 2026  
**Owner:** HLLMR LLC  
**Status:** ✅ ACTIVE  
**Doc Status Tag:** Implemented
**Alignment:** ARCHITECTURE V4.2 + DEV_STANDARDS V4.2

---

## Overview

This document defines the **LayoutTranscript** - the core intermediate representation (IR) for PDF extraction in Conset PDF. The LayoutTranscript is the **backend-agnostic contract** that all PDF extraction libraries must satisfy.

This is a canonical derived document under `MASTER_PLAN_v4.md` per `DOC_GOVERNANCE.md`.

**Scope:**
- LayoutTranscript structure (spans, pages, metadata)
- Invariants and validation rules
- Canonicalization pipeline
- DocumentContext integration (SSOT pattern)
- Determinism enforcement

**Audience:** Developers implementing Phase 1 (Layout Extraction) and anyone working with the IR

**Philosophy:** The transcript is the **single source of truth** for all PDF geometric data. All extraction backends must produce deterministic, reproducible transcripts that conform to this contract.

---

## Table of Contents

1. [LayoutTranscript Structure](#layouttranscript-structure)
2. [Invariants](#invariants)
3. [Canonicalization](#canonicalization)
4. [DocumentContext Integration](#documentcontext-integration)
5. [Determinism Requirements](#determinism-requirements)
6. [Validation](#validation)

---

## Backend Quality Tiers (Imported)

Backend implementations may differ in extraction fidelity. Canonical transcript architecture treats this as an explicit quality-tier concern.

Rules:

1. Backends must declare quality tier and capability limits (bbox fidelity, rotation handling, table robustness).
2. Low-fidelity fallback backends are allowed only for explicitly supported non-critical paths.
3. Deterministic canonicalization requirements remain identical across all enabled backends.
4. If a fallback cannot satisfy required quality gates for an operation, fail explicitly instead of silently degrading output quality.

---

## LayoutTranscript Structure

### Purpose

**LayoutTranscript** represents the **raw extracted layout** from a PDF: text spans with positions, fonts, and bounding boxes. It contains **no semantics**—only geometry and text.

**It is NOT:**
- A specific library output (PDFium struct, etc.)
- Tied to any extraction engine
- Opinionated about interpretation (headers, sections, tables)

**It IS:**
- A deterministic, reproducible representation of text + geometry
- The foundation for all downstream processing
- Validated against invariants

---

### Core Types

```rust
// ir/src/transcript.rs

/// A complete transcript: all pages extracted from a PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTranscript {
    pub metadata: TranscriptMetadata,
    pub pages: Vec<LayoutPage>,
    /// Deterministic hash of PDF content (excludes extraction timestamp)
    pub content_hash: String,
}

/// Metadata about the extraction process (for auditability).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMetadata {
    /// PDF file hash (SHA256 of file bytes)
    pub pdf_content_hash: String,
    /// Extraction backend used (e.g., "pdfium", "custom")
    pub extractor_name: String,
    /// Extractor version (e.g., "4.2.0")
    pub extractor_version: String,
    /// Timestamp of extraction (ISO 8601 UTC)
    pub extracted_at: String,
    /// Total span count across all pages
    pub total_spans: usize,
    /// Total character count across all spans
    pub total_chars: usize,
}
```

---

### Page Structure

```rust
/// A single page from the PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPage {
    /// 0-indexed page number (0 = first page)
    pub page_index: usize,
    /// Page width in points (1/72 inch)
    pub width: i32,
    /// Page height in points
    pub height: i32,
    /// Rotation: 0, 90, 180, 270 (normalized to 0 after canonicalization)
    pub rotation: u16,
    /// Extracted text spans (sorted deterministically after canonicalization)
    pub spans: Vec<LayoutSpan>,
    /// Per-page quality metrics
    pub quality: PageQuality,
}

/// Per-page quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageQuality {
    /// Character count on this page
    pub char_count: usize,
    /// Percentage of replacement characters (e.g., U+FFFD)
    pub replacement_char_ratio: f32,
    /// Confidence score (0.0–1.0)
    pub confidence: f32,
    /// Issues found: decoding errors, bbox anomalies, etc.
    pub issues: Vec<String>,
}
```

---

### Span Structure

```rust
/// A single text span (contiguous text with uniform formatting).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSpan {
    /// Unique ID within page (assigned after canonicalization)
    pub id: u32,
    /// Text content (exact UTF-8 from PDF)
    pub text: String,
    /// Bounding box (normalized coordinates after canonicalization)
    pub bbox: BBox,
    /// Font name / family (e.g., "Helvetica", "Times-Roman")
    pub font_name: String,
    /// Font size in points
    pub font_size: f32,
    /// Flags: is_bold, is_italic, is_monospace, etc.
    pub flags: SpanFlags,
    /// Span-level hash (for deduplication detection)
    pub span_hash: String,
}

/// Bounding box in page coordinates.
/// After canonicalization: top-left origin, normalized to [0.0, 1.0] range.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BBox {
    pub x: f32,      // Left edge, [0.0, 1.0]
    pub y: f32,      // Top edge, [0.0, 1.0]
    pub width: f32,  // Width, [0.0, 1.0]
    pub height: f32, // Height, [0.0, 1.0]
}

impl BBox {
    /// Check if bbox is within normalized bounds
    pub fn is_valid(&self) -> bool {
        self.x >= 0.0 && self.x <= 1.0
            && self.y >= 0.0 && self.y <= 1.0
            && self.width >= 0.0 && (self.x + self.width) <= 1.0
            && self.height >= 0.0 && (self.y + self.height) <= 1.0
    }
    
    /// Convert from page coordinates (points) to normalized [0, 1]
    pub fn normalize(x: i32, y: i32, width: i32, height: i32, page_width: i32, page_height: i32) -> Self {
        BBox {
            x: x as f32 / page_width as f32,
            y: y as f32 / page_height as f32,
            width: width as f32 / page_width as f32,
            height: height as f32 / page_height as f32,
        }
    }
}

/// Span formatting flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanFlags {
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_monospace: bool,
}
```

---

## Invariants

### Critical Rules

Every `LayoutTranscript` **MUST** satisfy these invariants after canonicalization:

#### 1. Coordinate System

**Origin:** Top-left (0, 0)
- `x=0` at left, increases rightward
- `y=0` at top, increases downward

**Normalization:** All coordinates in `[0.0, 1.0]` range
- `0 ≤ bbox.x ≤ 1.0`
- `0 ≤ bbox.y ≤ 1.0`
- `0 ≤ bbox.width ≤ 1.0`
- `0 ≤ bbox.height ≤ 1.0`
- `bbox.x + bbox.width ≤ 1.0`
- `bbox.y + bbox.height ≤ 1.0`

**Why normalized?**
- Page-size independent (works for any PDF dimensions)
- Simplifies geometric operations
- Prevents coordinate overflow

---

#### 2. Determinism

**Same input → same output, always.**

**Span sorting:** On each page, spans sorted by `(y, x)`:
```rust
spans.sort_by(|a, b| {
    (a.bbox.y, a.bbox.x)
        .partial_cmp(&(b.bbox.y, b.bbox.x))
        .unwrap_or(Ordering::Equal)
});
```

**Content hash:** Deterministic (excludes extraction timestamp):
- Hash computed from page dimensions + sorted spans
- Same PDF → same hash, always
- Used for cache validation

**No randomness:**
- No shuffles, random selection
- No timeout variance
- No non-deterministic tie-breaking

---

#### 3. Uniqueness

**Span IDs:** Unique within each page
- Assigned after sorting: `span.id = idx as u32`
- Sequential: 0, 1, 2, 3, ...

**Page indices:** Sequential, 0-indexed
- First page = 0
- Second page = 1
- No gaps, no duplicates

---

#### 4. Validation

**Text:** All text must be valid UTF-8
- No invalid byte sequences
- Replacement character (U+FFFD) allowed but counted in quality metrics

**Geometry:** All numeric values must be valid
- No NaN, no Inf in bboxes
- Font sizes > 0
- Coordinates within bounds

**Quality:** Confidence scores in `[0.0, 1.0]`

---

#### 5. Immutability

**Once created, `LayoutTranscript` is immutable.**
- No modifications to spans, pages, or metadata after creation
- Any transformations create new transcript
- Original transcript preserved for audit trail

---

## Canonicalization

### Purpose

After extraction from PDF library (PDFium, etc.), the raw transcript must be **canonicalized** to enforce invariants.

**Canonicalization ensures:**
- Rotation normalized (all pages rotation=0)
- Coordinates normalized to [0, 1] range
- Bboxes recalculated if rotation changed
- Spans sorted deterministically
- Hashes computed

---

### Canonicalization Pipeline

```rust
// ir/src/canonicalize.rs

/// Canonicalize a raw transcript from extraction.
pub fn canonicalize_transcript(
    raw_transcript: LayoutTranscript,
) -> Result<LayoutTranscript> {
    let mut canonical_pages = Vec::new();

    for page in raw_transcript.pages {
        // 1. Normalize rotation (convert to 0)
        let normalized_page = if page.rotation != 0 {
            normalize_rotation(&page)?
        } else {
            page
        };

        // 2. Normalize coordinates (to [0, 1] range)
        let mut normalized = normalized_page;
        for span in &mut normalized.spans {
            span.bbox = normalize_bbox(&span.bbox, normalized.width, normalized.height);
        }

        // 3. Validate bboxes
        validate_bboxes_on_page(&normalized)?;

        // 4. Sort spans deterministically (by y, then x)
        normalized.spans.sort_by(|a, b| {
            (a.bbox.y, a.bbox.x)
                .partial_cmp(&(b.bbox.y, b.bbox.x))
                .unwrap_or(Ordering::Equal)
        });

        // 5. Recompute span IDs (sequential after sort)
        for (idx, span) in normalized.spans.iter_mut().enumerate() {
            span.id = idx as u32;
            span.span_hash = compute_span_hash(span);
        }

        // 6. Compute page quality metrics
        normalized.quality = compute_page_quality(&normalized);

        canonical_pages.push(normalized);
    }

    // 7. Compute deterministic content hash
    let content_hash = compute_content_hash(&canonical_pages);

    Ok(LayoutTranscript {
        pages: canonical_pages,
        content_hash,
        ..raw_transcript
    })
}
```

---

### Rotation Normalization

**PDFs can be rotated:** 0°, 90°, 180°, 270°

**Canonicalization converts all pages to 0° rotation:**

```rust
fn normalize_rotation(page: &LayoutPage) -> Result<LayoutPage> {
    match page.rotation {
        0 => Ok(page.clone()),
        90 => rotate_90_cw(page),
        180 => rotate_180(page),
        270 => rotate_270_cw(page),
        _ => Err(anyhow!("Invalid rotation: {}", page.rotation)),
    }
}

fn rotate_90_cw(page: &LayoutPage) -> Result<LayoutPage> {
    let mut rotated = page.clone();
    rotated.rotation = 0;
    
    // Swap width/height
    std::mem::swap(&mut rotated.width, &mut rotated.height);
    
    // Transform each span's bbox
    for span in &mut rotated.spans {
        let old_bbox = span.bbox;
        // Rotate clockwise: (x, y) → (page_height - y - height, x)
        span.bbox = BBox {
            x: (page.height - old_bbox.y - old_bbox.height) as f32 / page.width as f32,
            y: old_bbox.x,
            width: old_bbox.height,
            height: old_bbox.width,
        };
    }
    
    Ok(rotated)
}
```

---

### Coordinate Normalization

**Convert page-relative points to normalized [0, 1] range:**

```rust
fn normalize_bbox(bbox: &BBox, page_width: i32, page_height: i32) -> BBox {
    BBox {
        x: bbox.x / page_width as f32,
        y: bbox.y / page_height as f32,
        width: bbox.width / page_width as f32,
        height: bbox.height / page_height as f32,
    }
}
```

**Example:**
- Page: 612pt × 792pt (letter size)
- Span bbox: `x=72, y=144, width=468, height=12`
- Normalized: `x=0.118, y=0.182, width=0.765, height=0.015`

---

## DocumentContext Integration

### SSOT Pattern

Once canonicalized, `LayoutTranscript` becomes the **Single Source of Truth** cached in `DocumentContext`.

```rust
// engine/src/io/document_context.rs

pub struct DocumentContext {
    /// PDF file path
    pdf_path: PathBuf,
    /// SSOT: canonical transcript (immutable after creation)
    transcript: LayoutTranscript,
    /// Cache of per-page contexts (lazy-created)
    page_contexts: RwLock<HashMap<usize, Arc<PageContext>>>,
}

impl DocumentContext {
    /// Load PDF and create document context.
    pub async fn load(pdf_path: &Path) -> Result<Arc<Self>> {
        // 1. Extract transcript via pdf-extraction crate
        let extractor = PdfExtractor::new()?;
        let raw_transcript = extractor.extract(pdf_path).await?;

        // 2. Canonicalize
        let canonical = canonicalize_transcript(raw_transcript)?;

        // 3. Cache in context
        Ok(Arc::new(DocumentContext {
            pdf_path: pdf_path.to_path_buf(),
            transcript: canonical,
            page_contexts: RwLock::new(HashMap::new()),
        }))
    }

    /// Get canonical transcript (immutable).
    pub fn get_transcript(&self) -> &LayoutTranscript {
        &self.transcript
    }

    /// Get per-page context (lazy-created).
    pub async fn get_page_context(&self, page_index: usize) -> Result<Arc<PageContext>> {
        let mut cache = self.page_contexts.write().await;

        if let Some(ctx) = cache.get(&page_index) {
            return Ok(Arc::clone(ctx));
        }

        let page = self.transcript.pages
            .get(page_index)
            .ok_or(anyhow!("Page {} not found", page_index))?;

        let ctx = Arc::new(PageContext::from_layout_page(page));
        cache.insert(page_index, Arc::clone(&ctx));

        Ok(ctx)
    }
}
```

---

### PageContext

**Per-page helper for common operations:**

```rust
pub struct PageContext {
    page: LayoutPage,
    /// Spans grouped by vertical bands (for header/footer detection)
    bands: Vec<Band>,
    /// Spans grouped by horizontal alignment (for column detection)
    columns: Vec<Column>,
}

impl PageContext {
    pub fn from_layout_page(page: &LayoutPage) -> Self {
        let bands = detect_bands(page);
        let columns = detect_columns(page);
        
        PageContext {
            page: page.clone(),
            bands,
            columns,
        }
    }
    
    /// Get spans in a specific region
    pub fn get_spans_in_region(&self, region: &BBox) -> Vec<&LayoutSpan> {
        self.page.spans
            .iter()
            .filter(|span| bbox_intersects(&span.bbox, region))
            .collect()
    }
    
    /// Get spans in footer region (bottom 15% of page)
    pub fn get_footer_spans(&self) -> Vec<&LayoutSpan> {
        let footer_region = BBox {
            x: 0.0,
            y: 0.85,
            width: 1.0,
            height: 0.15,
        };
        self.get_spans_in_region(&footer_region)
    }
}
```

---

## Determinism Requirements

### Regression Testing

**Every extraction must be deterministic:**

```rust
// tests/transcript/determinism_test.rs

#[tokio::test]
async fn test_transcript_determinism() {
    let pdf_path = Path::new("tests/fixtures/sample.pdf");

    // Extract 3 times
    let ctx1 = DocumentContext::load(pdf_path).await.unwrap();
    let ctx2 = DocumentContext::load(pdf_path).await.unwrap();
    let ctx3 = DocumentContext::load(pdf_path).await.unwrap();

    let t1 = ctx1.get_transcript();
    let t2 = ctx2.get_transcript();
    let t3 = ctx3.get_transcript();

    // Hashes must be identical
    assert_eq!(t1.content_hash, t2.content_hash);
    assert_eq!(t2.content_hash, t3.content_hash);

    // Page counts must match
    assert_eq!(t1.pages.len(), t2.pages.len());
    assert_eq!(t2.pages.len(), t3.pages.len());

    // Span counts must match
    for (p1, p2, p3) in izip!(&t1.pages, &t2.pages, &t3.pages) {
        assert_eq!(p1.spans.len(), p2.spans.len());
        assert_eq!(p2.spans.len(), p3.spans.len());
        
        // Span ordering must match
        for (s1, s2, s3) in izip!(&p1.spans, &p2.spans, &p3.spans) {
            assert_eq!(s1.text, s2.text);
            assert_eq!(s2.text, s3.text);
            assert_eq!(s1.bbox, s2.bbox);
            assert_eq!(s2.bbox, s3.bbox);
        }
    }
}
```

---

### Content Hash Computation

**Deterministic hash excludes timestamp:**

```rust
fn compute_content_hash(pages: &[LayoutPage]) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    
    // Hash page count
    hasher.update(pages.len().to_le_bytes());
    
    // Hash each page
    for page in pages {
        hasher.update(page.page_index.to_le_bytes());
        hasher.update(page.width.to_le_bytes());
        hasher.update(page.height.to_le_bytes());
        
        // Hash each span
        for span in &page.spans {
            hasher.update(span.text.as_bytes());
            hasher.update(span.bbox.x.to_le_bytes());
            hasher.update(span.bbox.y.to_le_bytes());
            hasher.update(span.bbox.width.to_le_bytes());
            hasher.update(span.bbox.height.to_le_bytes());
        }
    }
    
    format!("{:x}", hasher.finalize())
}
```

**Timestamp explicitly excluded** so same PDF always produces same hash.

---

## Validation

### Invariant Checks

**Every transcript must pass validation:**

```rust
// ir/src/validation.rs

pub fn validate_transcript(transcript: &LayoutTranscript) -> Result<()> {
    // 1. Check page indices sequential
    for (expected, page) in transcript.pages.iter().enumerate() {
        if page.page_index != expected {
            return Err(anyhow!(
                "Page index mismatch: expected {}, got {}",
                expected,
                page.page_index
            ));
        }
    }
    
    // 2. Validate each page
    for page in &transcript.pages {
        validate_page(page)?;
    }
    
    // 3. Check content hash
    let computed_hash = compute_content_hash(&transcript.pages);
    if transcript.content_hash != computed_hash {
        return Err(anyhow!("Content hash mismatch"));
    }
    
    Ok(())
}

fn validate_page(page: &LayoutPage) -> Result<()> {
    // 1. Check rotation normalized
    if page.rotation != 0 {
        return Err(anyhow!("Page {} rotation not normalized", page.page_index));
    }
    
    // 2. Check page dimensions
    if page.width <= 0 || page.height <= 0 {
        return Err(anyhow!("Invalid page dimensions: {}×{}", page.width, page.height));
    }
    
    // 3. Validate each span
    for (idx, span) in page.spans.iter().enumerate() {
        // Check ID sequential
        if span.id != idx as u32 {
            return Err(anyhow!(
                "Span ID mismatch on page {}: expected {}, got {}",
                page.page_index,
                idx,
                span.id
            ));
        }
        
        // Check bbox valid
        if !span.bbox.is_valid() {
            return Err(anyhow!(
                "Invalid bbox on page {}, span {}: {:?}",
                page.page_index,
                span.id,
                span.bbox
            ));
        }
        
        // Check font size
        if span.font_size <= 0.0 {
            return Err(anyhow!(
                "Invalid font size on page {}, span {}: {}",
                page.page_index,
                span.id,
                span.font_size
            ));
        }
    }
    
    // 4. Check span sorting
    for window in page.spans.windows(2) {
        let (prev, next) = (&window[0], &window[1]);
        if (prev.bbox.y, prev.bbox.x) > (next.bbox.y, next.bbox.x) {
            return Err(anyhow!(
                "Spans not sorted on page {}: span {} at ({}, {}) comes before span {} at ({}, {})",
                page.page_index,
                prev.id, prev.bbox.y, prev.bbox.x,
                next.id, next.bbox.y, next.bbox.x
            ));
        }
    }
    
    Ok(())
}
```

---

## Appendix: Example Transcript

**Minimal example (JSON serialized):**

```json
{
  "metadata": {
    "pdf_content_hash": "a1b2c3d4...",
    "extractor_name": "pdfium",
    "extractor_version": "4.2.0",
    "extracted_at": "2026-01-23T10:30:00Z",
    "total_spans": 150,
    "total_chars": 12500
  },
  "content_hash": "e5f6g7h8...",
  "pages": [
    {
      "page_index": 0,
      "width": 612,
      "height": 792,
      "rotation": 0,
      "quality": {
        "char_count": 2500,
        "replacement_char_ratio": 0.0,
        "confidence": 1.0,
        "issues": []
      },
      "spans": [
        {
          "id": 0,
          "text": "RWB Project No. 25063.00",
          "bbox": {
            "x": 0.1,
            "y": 0.05,
            "width": 0.3,
            "height": 0.02
          },
          "font_name": "Helvetica",
          "font_size": 10.0,
          "flags": {
            "is_bold": false,
            "is_italic": false,
            "is_monospace": false
          },
          "span_hash": "abc123..."
        },
        {
          "id": 1,
          "text": "SECTION 23 82 16",
          "bbox": {
            "x": 0.1,
            "y": 0.1,
            "width": 0.3,
            "height": 0.02
          },
          "font_name": "Helvetica-Bold",
          "font_size": 14.0,
          "flags": {
            "is_bold": true,
            "is_italic": false,
            "is_monospace": false
          },
          "span_hash": "def456..."
        }
      ]
    }
  ]
}
```

---

## Quality Gate Thresholds (Imported)

Transcript acceptance requires passing all four quality gates. These thresholds are architectural baselines; per-document-class tuning may narrow but not eliminate any gate.

| Gate | Metric | Canonical Threshold | Failure Condition |
|---|---|---|---|
| Text Presence | Characters per page | >= 50 | Page has insufficient text for reliable analysis |
| Encoding Integrity | Replacement character (U+FFFD) ratio | <= 0.05 | Encoding or OCR failure detected |
| Ordering Sanity | Source order vs. geometric y/x sort agreement | >= 0.80 | Span ordering is scrambled |
| Aggregate Confidence | Overall extraction confidence | >= 0.85 | Transcript does not meet quality bar for production use |

Quality scoring must report per-gate failure diagnostics, not a pass/fail aggregate. Legitimately text-sparse pages (separators, cover pages) must not be treated as universal quality failures.

Quality-driven extractor fallback (auto-switching to a secondary extractor when the primary scores below gate) is an architectural requirement, not an optional optimization.

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 4.0.0 | 2026-01-21 | Initial transcript architecture |
| 4.2.0 | 2026-01-23 | **Aligned with Master Plan V4.2 and ARCHITECTURE V4.2.** Changes: (1) Simplified by removing AbstractTranscript (semantic overlay now medium-specific, in processors), (2) Removed TokenVault (privacy abstraction out of scope), (3) Clarified coordinate normalization to [0, 1] range, (4) Updated BBox to use normalized floats instead of page-relative integers, (5) Changed page numbering to 0-indexed for consistency with Rust conventions, (6) Added PageContext helper patterns, (7) Expanded validation examples, (8) Added determinism test patterns from DEV_STANDARDS. Focus: Deep dive on LayoutIR only, companion to ARCHITECTURE V4.2. |

---

**Status:** ✅ ACTIVE  
**Owner:** HLLMR LLC  
**Last Updated:** January 23, 2026  
**Version:** 4.2.0

---

**End of TRANSCRIPT_ARCHITECTURE Document**

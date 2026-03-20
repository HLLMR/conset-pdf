# Phase 07 - Sidecar Replacement Plan

**Document Type**: Rust Handoff — Python Sidecar Replacement  
**Date**: 2026-03-19  
**Reference ADR**: `03-adrs/phase-03-adr-001-python-sidecar.md`  
**Non-Negotiable**: NN-10 (No Python Runtime), NN-11 (Licensing Hygiene)

---

## Overview

The prototype uses two Python sidecar scripts to isolate LGPL-licensed library operations from the Apache-2.0 core. Both sidecars are spawn-based: TypeScript spawns Python, passes file paths and/or JSON, and reads results back.

| Sidecar | Script | Library | Purpose |
|---|---|---|---|
| Transcript extraction | `transcript/sidecar/extract-transcript.py` | PyMuPDF (LGPL) | Extract text spans with accurate bounding boxes |
| Bookmark writing | `bookmarks/sidecar/bookmark-writer.py` | pikepdf/QPDF (LGPL) | Write indirect-object outline + final PDF passthrough |

V4 Rust replaces both with native libraries that are Apache-2.0 or MIT licensed.

---

## Sidecar 1: Transcript Extraction

### What the sidecar does (prototype behavior)

`extract-transcript.py` accepts a PDF file path via command-line argument, opens the file with PyMuPDF (`fitz`), iterates over every page using `page.get_text("rawdict")`, and writes a JSON file containing:

```json
{
  "filePath": "<absolute path>",
  "extractionEngine": "pymupdf",
  "extractionDate": "<ISO timestamp>",
  "pages": [
    {
      "pageNumber": 1,
      "pageIndex": 0,
      "width": 2592,
      "height": 2006,
      "rotation": 0,
      "spans": [
        {
          "text": "M6.02",
          "bbox": [1800.5, 1920.3, 1950.2, 1960.1],
          "fontName": "Arial",
          "fontSize": 9.0,
          "flags": { "isBold": false, "isItalic": false },
          "color": "#000000",
          "spanId": "<hash>",
          "pageIndex": 0
        }
      ],
      "metadata": { ... }
    }
  ]
}
```

The TS adapter (`pymupdfExtractor.ts`) spawns the script, reads the JSON from a temp file, and validates it against the `LayoutTranscript` contract schema.

**Why PyMuPDF was chosen over PDF.js**: `page.get_text("rawdict")` gives per-character and per-span bounding boxes with publication-grade accuracy. PDF.js `getTextContent()` provides text content but with significantly less reliable bounding box precision, which broke geometric analysis (ROI filtering, chrome band detection).

### V4 Rust replacement: `pdfium-render`

**Crate**: `pdfium-render` (Apache-2.0)  
**Link**: https://crates.io/crates/pdfium-render  
**Distribution**: Requires `libpdfium.so` / `pdfium.dll` — either statically linked or bundled alongside the binary.

PDFium provides text extraction accuracy equivalent to PyMuPDF. It is the extraction library used by Google Chrome and Chromium and has excellent coverage of real-world PDFs including malformed ones.

#### Extraction implementation pattern

```rust
use pdfium_render::prelude::*;

pub fn extract_transcript(
    pdf_path: &Path,
    pdfium: &Pdfium,
) -> Result<LayoutTranscript, ExtractionError> {
    let doc = pdfium.load_pdf_from_file(pdf_path, None)?;
    let total_pages = doc.pages().len() as u32;

    let pages = doc.pages()
        .iter()
        .enumerate()
        .map(|(page_idx, page)| extract_page(page_idx, &page))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(LayoutTranscript {
        file_path: pdf_path.to_string_lossy().to_string(),
        extraction_engine: "pdfium".to_string(),
        extraction_date: Utc::now().to_rfc3339(),  // excluded from content hash
        pages,
        metadata: TranscriptMetadata {
            total_pages: total_pages as usize,
            has_true_text_layer: pages.iter().any(|p| !p.spans.is_empty()),
            content_hash: None,  // populated by canonicalize step
            span_hash: None,
        },
    })
}

fn extract_page(
    page_idx: usize,
    page: &PdfPage,
) -> Result<LayoutPage, ExtractionError> {
    let width = page.width().value;
    let height = page.height().value;
    let rotation = page.rotation()?.degrees();

    let spans = page.text()?
        .chars()
        .iter()
        .map(|ch| extract_span(ch, page_idx, width, height))
        .collect::<Vec<_>>();

    Ok(LayoutPage {
        page_number: page_idx + 1,
        page_index: page_idx,
        width,
        height,
        rotation,
        spans,
        metadata: PageMetadata::default(),
        images: None,
        lines: None,
    })
}
```

#### Coordinate transformation (critical)

PDFium uses bottom-left origin with Y increasing upward. The `LayoutTranscript` contract uses top-left origin with Y increasing downward, normalized to `[0.0, 1.0]`.

Apply this transformation in the extraction layer, before any span is stored:

```rust
fn pdfium_bbox_to_normalized(
    left: f32, bottom: f32, right: f32, top: f32,
    page_width: f32, page_height: f32,
) -> [f32; 4] {
    let x0 = left / page_width;
    let y0 = 1.0 - (top / page_height);    // flip Y: PDFium top → normalized y0
    let x1 = right / page_width;
    let y1 = 1.0 - (bottom / page_height); // flip Y: PDFium bottom → normalized y1
    [x0, y0, x1, y1]
}
```

**Rotation normalization**: Page `rotation` field in PDFium reports the physical rotation angle (0, 90, 180, 270). Apply the rotation matrix to all span coordinates to produce rotation=0 basis coordinates before storing in `LayoutPage`. This matches the behavior of `canonicalize.ts`'s `normalizePageCoordinates()` in the prototype.

```rust
fn apply_rotation_normalization(bbox: [f32; 4], rotation: u32) -> [f32; 4] {
    match rotation {
        90 =>  { /* rotate CCW 90: (x,y) → (y, 1-x) */ }
        180 => { /* rotate 180: (x,y) → (1-x, 1-y) */ }
        270 => { /* rotate CW 90: (x,y) → (1-y, x) */ }
        _ =>   bbox  // no rotation
    }
}
```

#### Stable sort (must match prototype)

After extraction, apply the stable sort before caching:

1. Primary: `y0` ascending (top of page first)
2. Secondary: `x0` ascending (left to right)
3. Tie-break: `sha256(text + font_name + font_size.to_bits())` as `u64`

All comparisons must use exact `f32` bit representations (after rounding to 2 decimal places) to avoid non-deterministic float equality.

#### PDFium distribution strategy

| Platform | Approach |
|---|---|
| Linux (CI, server) | Link against `libpdfium.so` bundled in package; or use `pdfium-render` with `feature = "static"` |
| macOS | Bundle `libpdfium.dylib` |
| Windows | Bundle `pdfium.dll` adjacent to binary |
| Tauri GUI | PDFium `.dll`/`.so` placed in Tauri `resources/` directory and loaded at runtime |

**Licensing check**: PDFium is Apache-2.0. No LGPL-at-link-time concern.

---

## Sidecar 2: Bookmark Writing

### What the sidecar does (prototype behavior)

`bookmark-writer.py` has two modes:

**Mode: `write-bookmarks`**

Accepts a PDF file path + JSON bookmark payload, uses pikepdf/QPDF to:
1. Open the PDF
2. Construct an outline dictionary using `pikepdf.make_indirect()` (produces indirect objects)
3. Build the `/First`, `/Last`, `/Next`, `/Prev`, `/Parent`, `/Count` chains for all bookmark nodes
4. Write `/Action` GoTo destinations pointing to page objects by index
5. Save the PDF
6. Re-open and traverse the outline to verify structural integrity (post-write validation)

**Mode: `passthrough`**

Accepts a PDF file path + optional basic write options. Opens the PDF with pikepdf and writes it out. Used as the final write step in merge (`pikepdfWriter.ts`) to ensure the output PDF is well-formed QPDF output.

### Why pikepdf/QPDF was required

`pdf-lib` was unable to produce bookmark structures that rendered correctly in AEC PDF viewers (Bluebeam Revu, PDF-XChange, Foxit). The issue: `pdf-lib` writes outline items as direct objects; most professional viewers require outline items to be **indirect objects** with a full bidirectional link chain (`/First`, `/Last`, `/Next`, `/Prev`, `/Parent`).

QPDF (via pikepdf) produces exactly the required structure. The post-write validation in the sidecar re-traverses the outline after writing to confirm link integrity.

### V4 Rust replacement: `lopdf`

**Crate**: `lopdf` (MIT)  
**Link**: https://crates.io/crates/lopdf

`lopdf` provides direct access to PDF object graphs and supports creating indirect objects via `Document::add_object()`. This gives the same structural control that pikepdf/QPDF provided.

#### Bookmark writing implementation pattern

The PDF outline structure that all professional viewers require:

```
/Catalog
  └── /Outlines  (indirect object, type /Outlines)
        ├── /Count  N
        ├── /First  (ref to first top-level item)
        └── /Last   (ref to last top-level item)

Top-level item (indirect object, type /Outlines):
  ├── /Title   (string)
  ├── /Parent  (ref to /Outlines root)
  ├── /Next    (ref to next sibling, if any)
  ├── /Prev    (ref to previous sibling, if any)
  ├── /First   (ref to first child, if has children)
  ├── /Last    (ref to last child, if has children)
  ├── /Count   (total descendant count, negative if collapsed)
  └── /A       (action dict: /Type /Action, /S /GoTo, /D [page_obj 0 /XYZ null null null])
```

```rust
use lopdf::{Document, Dictionary, Object, ObjectId};

pub fn write_bookmarks(
    pdf_path: &Path,
    output_path: &Path,
    tree: &BookmarkTree,
) -> Result<(), BookmarkError> {
    let mut doc = Document::load(pdf_path)?;

    // Build all outline item objects as indirect objects
    let root_id = build_outline_tree(&mut doc, &tree.nodes, None)?;

    // Set /Outlines entry in /Catalog
    let catalog_id = doc.catalog_mut()?.object_id()?;
    let catalog = doc.get_dictionary_mut(catalog_id)?;
    catalog.set("Outlines", Object::Reference(root_id));

    // Post-write validation before saving
    doc.save(output_path)?;
    validate_outline_after_write(output_path, tree)?;
    Ok(())
}

fn build_outline_tree(
    doc: &mut Document,
    nodes: &[BookmarkNode],
    parent_id: Option<ObjectId>,
) -> Result<ObjectId, BookmarkError> {
    // 1. Create an indirect object for each node
    // 2. Set /Title, /Parent, /Next, /Prev, /First, /Last, /Count
    // 3. /A = GoTo action with page object reference
    // ...
    todo!("Implement full chain construction")
}
```

#### Post-write validation (mandatory)

After writing, re-open the PDF with `lopdf` and traverse the outline to verify:
1. `/Catalog` has a valid `/Outlines` reference
2. Every outline item is an indirect object
3. `/Next`/`/Prev` chains are consistent (forward + backward)
4. `/First`/`/Last` of each parent point to correct children
5. `/Count` values match actual descendant counts
6. `/A` actions reference valid page indices

If validation fails, delete the output file and return a structured error. Never leave a partially-written or corrupt PDF.

#### GoTo destination format

Use `XYZ null null null` destination to jump to the top of the target page without changing zoom:

```
[page_object_ref 0 /XYZ null null null]
```

Page object reference: use `doc.get_page_id(page_index)` to get the page's object ID. Never use page index integers as destinations — use object references for cross-viewer compatibility.

---

## Sidecar 3: PDF Passthrough Write (Merge Output)

### What it does

`pikepdfWriter.ts` routes final merge output through `bookmark-writer.py --mode passthrough`. This ensures the output PDF is written by a QPDF-backed writer, which normalizes the cross-reference table and produces clean PDF structure.

### V4 approach

In V4, the merging and writing is done entirely within `lopdf` or a streaming approach using `lopdf`. The "passthrough" step is not needed separately because:

1. `lopdf` writes valid PDFs with normalized xref tables
2. The disk-streaming merge (see below) moves pages without re-encoding, so no post-processing passthrough is needed

If validation of xref integrity is required, use `lopdf`'s own document validation or the `pdfium-render` document loading check to verify the output.

---

## Disk-Streaming Merge (ADR-004 — Restore)

The prototype described disk-streaming merge in `docs/largeFileRefactorPlan.md` but never fully implemented it. Current prototype: in-memory assembly via `pdf-lib` + pikepdf passthrough write. V4 must implement disk-streaming from day one.

### Target architecture

```rust
pub fn apply_merge_plan(
    plan: &MergePlan,
    output_path: &Path,
) -> Result<MergeReport, MergeError> {
    // Open a new lopdf Document for writing
    let mut output_doc = Document::new();

    // For each action in plan, in order:
    for action in &plan.actions {
        match action {
            MergeAction::Keep { source_path, page_index } => {
                // Stream page content object bytes verbatim
                let src = Document::load(source_path)?;
                let page_id = src.get_page_id(*page_index)?;
                copy_page_verbatim(&src, page_id, &mut output_doc)?;
            }
            MergeAction::Replace { source_path, page_index } => {
                // Same as Keep but from addendum source
                let src = Document::load(source_path)?;
                let page_id = src.get_page_id(*page_index)?;
                copy_page_verbatim(&src, page_id, &mut output_doc)?;
            }
            MergeAction::Insert { source_path, page_index } => {
                let src = Document::load(source_path)?;
                let page_id = src.get_page_id(*page_index)?;
                copy_page_verbatim(&src, page_id, &mut output_doc)?;
            }
        }
    }

    // Atomic write: write to temp path, then rename
    let temp_path = output_path.with_extension("tmp");
    output_doc.save(&temp_path)?;
    fs::rename(&temp_path, output_path)?;

    Ok(MergeReport::from_plan(plan))
}

fn copy_page_verbatim(
    src: &Document,
    page_id: ObjectId,
    dst: &mut Document,
) -> Result<(), MergeError> {
    // Copy page dict and content streams without decoding
    // Preserve resource dictionaries, fonts, images
    // …
    todo!("Implement verbatim page copy")
}
```

**Invariant**: The bytes of each page content stream (`stream...endstream`) in the output must be byte-identical to the corresponding source bytes. Verified by SHA-256 comparison in integration tests.

**Memory constraint**: No more than one page's content bytes should be held in memory at a time during the copy loop. For extremely large drawing sheets (some are 50–100 MB per page), this prevents OOM.

---

## Migration Timeline Recommendation

| Priority | Sidecar | Replacement | Notes |
|---|---|---|---|
| P0 — Phase 1 | `extract-transcript.py` | `pdfium-render` | Required for any workflow to function |
| P0 — Phase 1 | `bookmark-writer.py` | `lopdf` indirect objects | Required for bookmarks workflow |
| P0 — Phase 1 | `pikepdfWriter.ts` passthrough | `lopdf` direct write | Required for merge output |
| P0 — Phase 1 | In-memory merge assembly | Disk-streaming `lopdf` merge | Required for large-file correctness |

All four replacements are P0 because they are foundational to the core workflows. No V4 build should ship with Python sidecar dependencies.

---

## Testing Requirements for Sidecar Replacements

For each replacement:

1. **Transcript extraction**: Run identical PDF through prototype PyMuPDF path and V4 PDFium path; compare span counts, bbox coordinates (within 0.5pt tolerance), and text content. 100% text match required; minor coordinate differences acceptable in first iteration.

2. **Bookmark writing**: Write a known `BookmarkTree` (including nested hierarchy and mixed /GoTo destinations) to a PDF; read bookmarks back with a PDF viewer and assert all titles, destinations, and nesting levels are correct.

3. **Byte-verbatim page copy**: Copy a page from a known PDF using the lopdf merge path; assert `sha256(original_content_stream) == sha256(output_content_stream)`.

4. **Disk-streaming memory**: Run a merge of a 500 MB PDF set; assert peak RSS does not exceed 256 MB during the merge operation.

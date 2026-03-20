# Phase 06 - Workflow Breakdown: Specs Patch

**Document Type**: Post-Mortem Workflow Breakdown  
**Workflow**: Specs Patch  
**Implementation Status**: Abandoned per current ROADMAP (engine + CLI existed; now deprecated)  
**Current Executable State**: Core pipeline (AST extraction + bookmarkTreeGenerator) remains active; Playwright render + full workflow orchestration is deprecated  
**Date**: 2026-03-19

---

## Domain Purpose

Extract Word-generated specification PDFs to a structured hierarchical AST (Abstract Syntax Tree), apply deterministic patch operations (insert/move/renumber/replace/delete) to individual spec sections without regenerating whole books, render the patched AST back to PDF, and output a `BookmarkAnchorTree` for downstream bookmark generation.

This workflow addresses one of the core moat features: **spec section-only regeneration** — the ability to modify a single spec section and extract it as a print-ready PDF without re-flowing or reformatting the entire specification book.

**Status Note**: The Specs Patch workflow is labeled Abandoned in the current ROADMAP (superseded label). However, per SOT-009 and the source code, the AST extraction pipeline, patch machinery, and bookmarkTreeGenerator remain active and are used by the bookmarks workflow. The end-to-end Playwright render path is the deprecated portion. The Rust implementation should not discard this workflow — it should be redesigned with a native rendering path. See ADR-008.

---

## Workflow Engine Entry Points

| Phase | Method | Key Type |
|---|---|---|
| Analyze | `SpecsPatchWorkflow.analyze(input: SpecsPatchAnalyzeInput)` | Returns `InventoryResult` |
| Corrections | `SpecsPatchWorkflow.applyCorrections(input, corrections)` | Returns revised `InventoryResult` |
| Execute | `SpecsPatchWorkflow.execute(input: SpecsPatchExecuteInput)` | Returns `ExecuteResult` |

Factory: internal; not exposed in primary `createWorkflowRunner` factory (per abandonment)  
Implementation: `workflows/specs-patch/specsPatchWorkflow.ts`  
CLI Command: `specs-patch` (registered but deprecated per ROADMAP)

---

## Analyze Phase: AST Extraction Pipeline

This is the most algorithmically complex analyze phase in the codebase. It converts a raw PDF into a fully structured hierarchical document AST.

### Input

```typescript
SpecsPatchAnalyzeInput {
  inputPdfPath: string;
  customSectionPattern?: string;     // Optional override for section ID detection regex
  options?: {
    verbose?: boolean;
    jsonOutputDir?: string;          // Write extracted AST JSON to this directory
  };
}
```

### Pipeline Stages

The extraction pipeline is a sequential transformation. Each stage consumes normalized text items from the transcript and produces increasingly structured output.

#### Stage 1: Chrome Removal (`specs/extract/chromeRemoval.ts`)

**Purpose**: Suppress header and footer bands before structural analysis.

**Algorithm**:
1. Load `candidates.ts` — structural candidates from Y-coordinate clustering + repetition analysis
2. Identify repeating text patterns (same text appearing on multiple pages at same Y position)
3. Classify pages' top and bottom bands as chrome based on:
   - Y position within top 8–12% or bottom 88–95% of page
   - Text repetition rate across pages (threshold: appears on ≥50% of pages)
   - Explicit chrome patterns (date stamps, section ID in footer, page counters)
4. Filter out all spans classified as chrome before downstream processing

**Known edge cases** (from Phase 5 catalog):
- Stamps and revision clouds misclassified as chrome (false positive)
- Watermarks at mid-page misclassified as chrome
- Headers that appear only on first page of each section may be excluded when they should signal section start

#### Stage 2: Section Detection (`specs/extract/sectionDetector.ts`)

**Purpose**: Identify section header lines that mark section boundaries.

**Algorithm**:
- Uses `SpecsSectionLocator` pattern matching on remaining (non-chrome) text
- Pattern battery covers: `XX YY ZZ` (modern), `XXYYY` (legacy)
- Section heading pattern: large font size, left-aligned, appears at top of page after chrome removed
- Returns `SectionHeader[]` with section ID, title, page index, and y-position

**Issue codes emitted**:
- `NO_SECTION_HEADER` — no sections detected at all
- `SCANNED_PDF` — no text layer; extraction impossible

#### Stage 3: Text Extraction (`specs/extract/textExtractor.ts`)

**Purpose**: Convert remaining text spans into typed AST nodes.

- Maps each text span to a `SpecNode` with: `nodeType`, `level`, `text`, `page`, `yPosition`, `fontMetric`
- Node types: `'heading'` | `'paragraph'` | `'list-item'` | `'table-row'` | `'unknown'`
- Font metric (size + weight) used as primary level discriminator

#### Stage 4: Paragraph Normalization (`specs/extract/paragraphNormalizer.ts`)

**Purpose**: Rejoin paragraph lines that were split by text extraction.

- **Wrap-join**: Consecutive text spans on adjacent Y positions with same indentation level are joined if the prior span does not end with a sentence-terminal character
- **Hyphen repair**: trailing hyphen at line end + next line not starting with uppercase → splice and remove hyphen
- **List preservation**: list bullet/number patterns defeat wrap-join (list items are always kept separate)

#### Stage 5: List Detection (`specs/extract/listDetector.ts`)

**Purpose**: Identify ordered and unordered list items and validate numbering sequences.

- Regex battery for CSI spec numbering patterns: `1.1`, `A.`, `a.`, `1)`, arabic, roman, alpha
- Detects numbering breaks (gap in sequence) → emits `NUMBERING_BREAK` issue
- Assigns `listLevel` depth by indentation relative to section baseline

#### Stage 6: Anchor Detection (`specs/extract/anchorDetector.ts`)

**Purpose**: Assign stable hierarchical anchors to nodes.

Anchors are CSI-style hierarchical identifiers: `PART.ARTICLE.PARAGRAPH.ITEM` (e.g., `2.4-T.5.b.1`).

**Algorithm**:
- Walk nodes in page order
- For each numbered heading/list-item, compute anchor from its position in the numbering hierarchy
- Anchors are unique within a section
- Ambiguous anchors: multiple candidates → emit `AMBIGUOUS_ANCHOR` issue
- Missing anchors: node has structural role but no parseable number → emit `ANCHOR_REQUIRED` issue

Anchors are the primary key for patch operations. All patches are specified by anchor.

#### Stage 7: Hierarchy Building (`specs/extract/hierarchyBuilder.ts`)

**Purpose**: Construct the `SpecDoc` tree from flat node list.

- Uses indentation level + font metric to determine parent/child relationships
- Spec sections are top-level children of `SpecDoc`
- Within each section: PART → ARTICLE → PARAGRAPH → ITEM nesting
- Validates that hierarchy is consistent with standard CSI three-part format (Parts 1/2/3 = General/Products/Execution)

#### Stage 8: Bookmark Tree Generation (`specs/extract/bookmarkTreeGenerator.ts`)

**Purpose**: Generate `BookmarkAnchorTree` for consumption by bookmarks workflow.

- Maps each spec section to a `BookmarkAnchorNode` with: `anchor`, `title`, `pageIndexHint`, `children[]`
- The `pageIndexHint` is the best-guess page from extraction (may be overridden by footer-first resolution in bookmarks workflow)
- `BookmarkAnchorTree` = `{ sections: BookmarkAnchorNode[] }`

### Analyze Output: InventoryResult

One row per extracted AST node. Row fields:
- `id`: `${source}:${pageIndex}:${nodeId}`
- `anchor`: CSI hierarchical anchor (e.g., `2.4-T.5.b.1`) or null
- `page`: 1-based page
- `status`: `'ok'` | `'warning'` | `'error'`
- `nodeType`: `'heading'` | `'paragraph'` | `'list-item'` | `'table-row'`
- `level`: indentation depth
- `textPreview`: first 100 chars

Issue codes:
- `NO_SECTION_HEADER`, `ANCHOR_REQUIRED`, `DUPLICATE_ANCHOR`, `AMBIGUOUS_ANCHOR`, `NUMBERING_BREAK`, `SCANNED_PDF`

Special metadata on `InventoryResult.meta`:
- `specDoc: SpecDoc` — complete AST
- `bookmarkTree: BookmarkAnchorTree` — for bookmarks pipeline

---

## Corrections Phase: Patch Operations

### Patch Types

Patch operations mutate the `SpecDoc` AST. All operations are target-anchor-addressed.

```typescript
SpecPatchOperation {
  op: 'insert' | 'move' | 'renumber' | 'replace' | 'delete';
  targetAnchor: string;             // Required: anchor of target node
  sectionId?: string;               // Disambiguation: specify section if anchor is not unique
  mustMatchText?: string;           // Disambiguation: confirm anchor by text match
}
```

### Insert

```json
{
  "op": "insert",
  "targetAnchor": "2.4-T.5.b.1",
  "position": "after",
  "content": {
    "type": "list-item",
    "text": "New requirement text",
    "level": 3
  }
}
```

Insert a new node before or after the target. Content can be a single node or a subtree.

### Move

```json
{
  "op": "move",
  "targetAnchor": "2.4-T.5.b.1",
  "destinationAnchor": "2.4-T.5.c",
  "position": "before"
}
```

Move the target node (and its subtree) to a new position in the document.

### Renumber

```json
{
  "op": "renumber",
  "targetAnchor": "2.4-T.5",
  "scope": "section"
}
```

Renumber all children of the target anchor to fix sequence after inserts/moves/deletes.

### Replace

```json
{
  "op": "replace",
  "targetAnchor": "2.4-T.5.b.1",
  "content": {
    "type": "list-item",
    "text": "Updated requirement text"
  }
}
```

Replace node text or content without changing structural position.

### Delete

```json
{
  "op": "delete",
  "targetAnchor": "2.4-T.5.b.1"
}
```

Remove the target node and its subtree.

### Patch Validation (`specs/patch/validator.ts`)

Before execution, patch validator checks:
- All target anchors exist in the AST
- No null anchors referenced
- No circular moves (target is not a descendant of destination)
- No disambiguation conflicts (ambiguous anchors with `sectionId` or `mustMatchText` resolution)

---

## Execute Phase: HTML/CSS → PDF Rendering

### Why Playwright

AEC spec PDFs require precise typographic fidelity: specific fonts, indentation levels, numbering styles, table formatting. The prototype chose an HTML/CSS rendering approach (AST → HTML → headless browser → PDF) for:

1. Leverage existing browser PDF rendering engine (consistent, high fidelity)
2. CSS is much more maintainable than raw PDF stream generation for rich spec formatting
3. Playwright's headless Chromium renders to PDF with CSS paging and print support

**Critical limitation**: This adds a large Chromium binary as a runtime dependency. For a production CLI tool, this is unacceptable. The Playwright dependency is why specs-patch is marked abandoned — the rendering approach needs to be redesigned with a native PDF generation path.

### Rendering Pipeline

1. `specs/patch/apply.ts` applies patch operations to `SpecDoc` AST
2. `specs/render/htmlGenerator.ts` converts patched `SpecDoc` → HTML string using `specs/render/templates/specs.css`
3. `specs/render/pdfRenderer.ts` invokes Playwright:
   - Launch headless Chromium
   - Load HTML with embedded CSS
   - Print to PDF with print media query
   - Save to output path
4. `bookmarkTreeGenerator.ts` regenerates `BookmarkAnchorTree` from patched AST
5. Optional: pass `BookmarkAnchorTree` to bookmarks workflow for final bookmark writing

### Execute Output: ExecuteResult

```
{
  outputs: {
    outputPdfPath: string;
    bookmarkTreeJsonPath?: string;
    reportPath?: string;
  }
  summary: {
    success: boolean;
    sectionsPatched: number;
    nodesInserted: number;
    nodesDeleted: number;
    nodesMoved: number;
    renderTimeMs: number;
  }
  warnings?: string[]
}
```

---

## `BookmarkAnchorTree` as Cross-Workflow Output

The most valuable output of the specs-patch workflow (and the only currently active downstream use) is the `BookmarkAnchorTree`. This is used by the bookmarks workflow to write structured hierarchical bookmarks into any output spec PDF.

```typescript
BookmarkAnchorTree {
  sections: BookmarkAnchorNode[];
}

BookmarkAnchorNode {
  anchor: string;           // Section ID (e.g., "23 05 53")
  title: string;            // Section title text
  pageIndexHint: number;    // Best-guess 0-based page index from extraction
  children?: BookmarkAnchorNode[];  // Article-level hierarchy
}
```

This contract is stable and should be preserved verbatim in Rust. It is the handoff interface between the extraction/patch pipeline and the bookmark writing pipeline.

---

## Known Failure Modes

| Mode | Symptom | Root Cause |
|---|---|---|
| Scanned PDF | `SCANNED_PDF` issue; no AST output | PDF has no text layer (OCR not implemented) |
| Chrome removal false positive | Content removed that should be kept | Repeating content classified as chrome (e.g., repeated section title in headers) |
| Anchor ambiguity | `AMBIGUOUS_ANCHOR`; patch fails validation | Same numbering pattern appears in multiple sections (e.g., each section has `1.1`) |
| Numbering sequence break | `NUMBERING_BREAK`; renumber needed | Source PDF has numbering gaps due to prior editorial changes |
| Playwright availability | Execute fails silently or crashes | Playwright browser not installed or headless environment misconfigured |
| Long spec books (>500 pages) | Memory pressure during AST construction | Full AST held in memory; no streaming; large books approach Node.js heap limits |

---

## Rust Implementation Notes

- Replace Playwright rendering with a native PDF generation path (e.g., typed-html + wkhtmltopdf alternative, or a purpose-built spec PDF renderer using printpdf/lopdf)
- The `BookmarkAnchorTree` contract is stable — port verbatim and use as the primary inter-subsystem handoff type
- Anchor detection is the most domain-specific algorithm; port regex patterns from `anchorDetector.ts` verbatim
- The three-part CSI format assumption (Parts 1/2/3 = General/Products/Execution) should be configurable rather than hardcoded
- Chrome removal threshold tuning should be exposed as a profile parameter (rather than hardcoded percentages)
- Consider streaming AST construction for large spec books (section-by-section rather than full-document)
- Patch validation must run as a separate pre-execute gate, not interleaved with apply

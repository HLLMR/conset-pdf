# Phase 06 - Workflow Breakdown: Fix Bookmarks

**Document Type**: Post-Mortem Workflow Breakdown  
**Workflow**: Fix Bookmarks  
**Implementation Status**: Complete — Engine + CLI  
**Date**: 2026-03-19

---

## Domain Purpose

Read, validate, repair, and write PDF bookmarks (PDF outline/navigation tree). Can operate in three modes:

1. **Repair existing**: Read current bookmarks, validate destinations, apply user corrections, rewrite
2. **Rebuild from BookmarkAnchorTree**: Use specs extraction output to generate structured spec bookmarks
3. **Rebuild from inventory**: Use sheet/section detection to generate simple bookmarks from PDF content

This workflow is the canonical path for getting reliable bookmarks into any PDF output, including post-merge drawing sets and post-patch spec PDFs.

---

## Workflow Engine Entry Points

| Phase | Method | Key Type |
|---|---|---|
| Analyze | `BookmarksWorkflow.analyze(input: BookmarksAnalyzeInput)` | Returns `InventoryResult` |
| Corrections | `BookmarksWorkflow.applyCorrections(input, corrections)` | Returns revised `InventoryResult` |
| Execute | `BookmarksWorkflow.execute(input: BookmarksExecuteInput)` | Returns `ExecuteResult` |

Factory: `createBookmarksWorkflowRunner()` in `workflows/engine.ts`  
Implementation: `workflows/bookmarks/bookmarksWorkflow.ts`  
CLI Command: `fix-bookmarks`  
GUI: Placeholder only (no production bookmarks GUI wired as of prototype end)

---

## Analyze Phase

### Input

```typescript
BookmarksAnalyzeInput {
  inputPdfPath: string;
  bookmarkTree?: BookmarkAnchorTree;   // From Specs Pipeline; triggers rebuild mode
  docType?: 'drawings' | 'specs';      // For inventory-based fallback
  profile?: LayoutProfile;              // For drawings detection
  options?: {
    verbose?: boolean;
    jsonOutputDir?: string;             // Write bookmark tree JSON for inspection
  };
}
```

### Processing Steps

1. **DocumentContext initialization**
   - Transcript extracted and cached

2. **Bookmark source selection** (priority order):
   - If `bookmarkTree` provided: use `BookmarkAnchorTree` as authoritative source for rebuild
   - If `docType` provided: run inventory detection (same locator path as merge workflow) to generate tree
   - If neither: read existing bookmarks from PDF (`bookmarks/reader.ts`)

3. **Bookmark tree construction** (`bookmarks/treeBuilder.ts`):
   - From `BookmarkAnchorTree`: map each spec section anchor to a page via `headingResolver.ts` (Footer-First Section Anchoring)
   - From inventory: one bookmark per detected page, grouped by discipline/division
   - From existing: parse PDF outline structure into `BookmarkTree`

4. **Destination validation** (`bookmarks/validator.ts`):
   - Each bookmark destination checked against actual PDF page count
   - Issue codes: `BOOKMARK_ORPHAN`, `BOOKMARK_DEAD_DEST`, `BOOKMARK_INVALID_FIT`, `BOOKMARK_MISMATCHED_HIERARCHY`, `BOOKMARK_DUPLICATE_TITLE`

5. **InventoryResult assembly**

### Output: InventoryResult

One row per bookmark node. Row fields:
- `id`: Stable identifier — `${source}:${anchor || logicalPath}`
- `page`: 1-based page number of bookmark destination
- `status`: `'ok'` | `'warning'` | `'error'`
- `confidence`: 0.0–1.0
- `source`: `'existing'` | `'bookmarkTree'` | `'inventory'`

Issue codes:
- `BOOKMARK_ORPHAN` — no valid destination
- `BOOKMARK_DEAD_DEST` — destination points to out-of-range page index
- `BOOKMARK_INVALID_FIT` — unsupported fit mode (reported, not dropped)
- `BOOKMARK_MISMATCHED_HIERARCHY` — hierarchy level inconsistency
- `BOOKMARK_DUPLICATE_TITLE` — duplicate title at same hierarchy level
- `BOOKMARK_ANCHOR_NOT_FOUND` — `BookmarkAnchorTree` anchor not found in PDF
- `BOOKMARK_PAGE_HINT_MISMATCH` — resolved page differs from `pageIndexHint`

---

## Footer-First Section Anchoring (`headingResolver.ts`)

This is the most complex and critical algorithm in the bookmarks workflow. It resolves each spec section anchor in `BookmarkAnchorTree` to its definitive starting page in the PDF.

### Algorithm

1. **ROI Band Detection** (auto, no profile required):
   - Page is divided into zones: header (0–12%), heading (0–30%), body (12–88%), footer (88–100%)
   - Zone boundaries established via text density analysis
   
2. **Section Code Extraction from Footer Band**:
   - Footer band text of each page is scanned for CSI section ID patterns
   - Pattern: `XX YY ZZ` (modern) or `XXYYY` (legacy)
   - Extracted section codes are associated with the page

3. **First-Page Mapping**:
   - For each section code, find the lowest page index where that code appears in the footer
   - That page index is the section's start page for bookmark destination purposes

4. **Resolution Priority Chain**:
   ```
   1. Footer-first: footer band contains explicit section code → use first occurrence page
   2. Heading-based: footer unavailable → scan heading band for section code
   3. pageIndexHint: no text match → use hint from BookmarkAnchorTree (clamped to valid range)
   4. Invalid: hint out of range → mark as BOOKMARK_ANCHOR_NOT_FOUND
   ```

5. **Strategy Override**:
   - `--section-start-strategy footer-first` (default)
   - `--section-start-strategy heading-only`
   - `--section-start-strategy hint-only`

### Why Footer-First is Necessary

Section pages often have the section ID printed in the footer (as part of the spec section chrome), not in a large heading. If the bookmark destination is placed at the heading, it may land on page 2 of a 3-page section when the cover page (with intro text) has no obvious heading. Footer-based detection is more reliable because it is consistently formatted boilerplate. See ADR-002 and Phase 2 algorithm docs.

### Section Page Ranges

After all section start pages are established, page ranges are computed:
- Section N occupies pages from `sectionN.startPage` to `sectionN+1.startPage - 1`
- Article hierarchy items within a section are placed at pages within the section's range
- Numeric tuple sort ensures sections are ordered: `[division, section, subsection]`

---

## Corrections Phase

Five correction types supported:

```typescript
BookmarksCorrectionOverlay {
  renames?: Record<rowId, { title: string }>;
  reorders?: string[];             // Array of row.id in desired order
  deletes?: string[];              // Array of row.id to remove
  retargets?: Record<rowId, { page: number; fit?: string }>;
  rebuild?: boolean;               // Full authoritative rebuild
}
```

**Rename**: Updates bookmark title text without changing destination.  
**Reorder**: Reorders top-level or sibling bookmarks (order array of stable IDs).  
**Delete**: Removes specified bookmarks from tree.  
**Retarget**: Moves a bookmark's destination to a different page.  
**Rebuild**: Full rebuild from input source; ignores any existing bookmarks entirely.

---

## Execute Phase (QPDF/pikepdf Sidecar)

### Why Python Sidecar for Bookmark Writing

PDF.js and pdf-lib cannot reliably write bookmarks that display correctly across all PDF viewers (particularly PDF-XChange Viewer, which is dominant in AEC). The key requirement is that outline items must be **indirect PDF objects** with properly set `/Parent`, `/Next`, `/Prev`, `/First`, `/Last`, `/Count` fields. The pikepdf/QPDF library produces viewer-compatible bookmarks reliably; pdf-lib does not. See ADR-001.

### Sidecar Protocol

1. `pikepdfBookmarkWriter.ts` serializes `BookmarkTree` to JSON
2. Invokes `bookmark-writer.py` as subprocess: `python bookmark-writer.py <input.pdf> <output.pdf> <tree.json>`
3. Python script:
   - Opens input PDF with pikepdf
   - Removes existing outline
   - Creates new `OutlineItem` for each bookmark node (recursive)
   - Each outline item is created as an indirect object via `pdf.make_indirect()`
   - Bidirectional `/Next`-`/Prev` links set at every level
   - `/First`, `/Last`, `/Count` set on parent nodes
   - PDF saved with `fix_metadata=False` to avoid content modification
4. Post-write verification: re-reads written PDF and validates outline structure

### Outline Structure Requirements (Critical)

The QPDF/pikepdf indirect-object pattern is non-negotiable for cross-viewer compatibility:

```
OutlineItem (indirect object)
  ├── /Title: string
  ├── /Dest: [pageRef /XYZ x y null]    or  /A: << /Type /Action /S /GoTo /D [...] >>
  ├── /Parent: ref (to parent item or to /Outlines root)
  ├── /Next: ref (to next sibling, if any)
  ├── /Prev: ref (to previous sibling, if any)
  ├── /First: ref (to first child, if has children)
  ├── /Last: ref (to last child, if has children)
  └── /Count: ±N (positive = open, negative = closed)
```

All items must be indirect (not inline dictionaries). This is a known PDF viewer compatibility requirement; inline outline items cause display failures in some viewers.

### Post-Write Validation

After writing, `bookmarks/validator.ts` re-reads the output PDF:
- Confirms all outline items are reachable via `/Next` chain from root
- Confirms all items are indirect objects
- Confirms `/Dest` or `/A` is valid for each item
- Returns `destinationsValidated` and `destinationsInvalid` counts in `ExecuteResult`

---

## Execute Output: ExecuteResult

```
{
  outputs: {
    outputPdfPath: string;
    reportPath?: string;
    bookmarkTreeJsonPath?: string;
  }
  summary: {
    success: boolean;
    bookmarksRead: number;
    bookmarksWritten: number;
    destinationsValidated: number;
    destinationsInvalid: number;
  }
  warnings?: string[]
}
```

---

## Bookmark Style Profiles

Three built-in style profiles control bookmark tree depth and formatting:

| Profile | Description | Use Case |
|---|---|---|
| `raw` | All detected nodes, no filtering | Debugging and inspection |
| `specs-v1` | Division-level only (flat) | Quick navigation of multi-division spec books |
| `specs-v2-detailed` | Division + section + article levels | Full hierarchical spec book navigation |

Style options include `maxDepth` (integer cutoff) and `profile` (style name). Profiles are defined in `bookmarks/profiles/`.

---

## Integration with Other Workflows

### Specs Patch → Fix Bookmarks Pipeline

The specs-patch workflow produces a `BookmarkAnchorTree` as part of its analyze output. This tree can be:
1. Passed directly to the bookmarks workflow via `bookmarkTree` input
2. Saved as JSON and provided via `--bookmark-tree` CLI option
3. Consumed by GUI wizard (not yet wired as of prototype end)

### Merge → Fix Bookmarks

After a merge execute, if `regenerateBookmarks: true` is set, the merge workflow invokes the bookmarks workflow pipeline internally. This regenerates drawing sheet bookmarks in the merged output PDF using the merged inventory.

---

## Known Failure Modes

| Mode | Symptom | Root Cause |
|---|---|---|
| Anchor not found in PDF | `BOOKMARK_ANCHOR_NOT_FOUND` | Section code in BookmarkAnchorTree not found in footer or heading bands |
| Page hint mismatch | `BOOKMARK_PAGE_HINT_MISMATCH` | pageIndexHint out of sync with actual PDF page count (e.g., after merge) |
| Wrong section start page | Bookmark destinations land 1–2 pages late | Footer-first detection misidentifies cover page vs. content start |
| Cross-viewer display failure | Bookmarks show in Acrobat but not PDF-XChange | Bookmark items written as inline objects (not via pikepdf sidecar) |
| Viewer count sign | Children collapsed unexpectedly | `/Count` sign convention differs: positive = open, negative = closed |

---

## Bookmark Destination Regression Risk

Bookmark destination correctness is a high-regression-risk area. Three dedicated test suites exist:
- `packages/core/src/bookmarks/tests/bookmarkPageMapping.test.ts`
- `packages/core/src/bookmarks/tests/bookmarkDestinations.test.ts`
- `packages/core/src/bookmarks/tests/bookmarkViewerCompatibility.test.ts`

Any refactor of `headingResolver.ts`, `treeBuilder.ts`, or `bookmark-writer.py` must pass these suites before merge.

---

## CLI Command

```bash
conset-pdf fix-bookmarks --input <path> --output <path> \
  [--bookmark-tree <tree.json>] [--doc-type drawings|specs] \
  [--layout <profile.json>] [--rebuild] [--style specs-v2-detailed] \
  [--section-start-strategy footer-first] [--report <report.json>]
```

---

## Rust Implementation Notes

- Replace `bookmark-writer.py` (pikepdf) with native lopdf/pdf-rs indirect object construction
- The indirect-object outline structure requirement is invariant; Rust implementation must enforce this at construction time
- Footer-First Section Anchoring algorithm should be a first-class Rust implementation (not a port of stub code)
- Post-write validation should be integrated into the write pipeline, not post-hoc
- Bookmark style profiles should be defined declaratively (e.g., as TOML/JSON configs, not hardcoded)
- `pageIndexHint` clamping rules must be documented and tested at boundary conditions (0, totalPages-1)

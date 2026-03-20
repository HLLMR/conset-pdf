# Workflows Documentation

**Last verified**: 2026-03-01

This document describes all workflows implemented in the conset-pdf core library, their domain meaning, inputs, outputs, and current implementation status.

## Workflow Overview

All workflows follow the **analyze → applyCorrections → execute** pattern:

1. **Analyze**: Dry-run inventory analysis (no file writes)
2. **Apply Corrections**: User edits applied to inventory
3. **Execute**: Produces output files

## Update Documents (Merge)

**Domain Meaning**: Merge addenda into an original construction document set. Replace updated sheets from addenda and insert new sheets in the correct sequence. This is the primary workflow for keeping construction documents up-to-date as addenda are issued.

**Status**: ✅ **Implemented** (workflow engine + CLI + GUI)

**Document Types Supported**:
- **Drawings** (`docType: 'drawings'`): Uses ROI-based detection (with layout profiles) or legacy title block detection
  - Supports layout profiles for ROI-based detection
  - Falls back to legacy title block detection if no profile provided
  - **Standards**: UDS-style discipline identification and sorting
    - Optional `discipline` field added to inventory rows
    - Discipline-based sorting with `compareDrawingsRows()`
- **Specs** (`docType: 'specs'`): Uses text-based section ID detection (`SpecsSectionLocator`)
  - Detects section IDs (e.g., "23 02 00", "00 31 21") from page text
  - Does not support layout profiles (uses text-based detection only)
  - Same merge logic as drawings (replace/insert/append modes work identically)
  - **Standards**: CSI MasterFormat classification and sorting
    - Optional `specs` field added to inventory rows
    - MasterFormat-based sorting with `compareSpecsRows()`

### Inputs

**Analyze Input** (`MergeAnalyzeInput`):
```typescript
{
  docType: 'drawings' | 'specs';
  originalPdfPath: string;
  addendumPdfPaths: string[];
  profile?: LayoutProfile;  // Optional layout profile for ROI detection
  options?: {
    mode?: 'replace+insert' | 'replace-only' | 'append-only';
    strict?: boolean;
    verbose?: boolean;
    inventoryOutputDir?: string;
  };
  narrativePdfPath?: string;  // Optional narrative PDF for advisory analysis
}
```

**Execute Input** (`MergeExecuteInput`):
```typescript
{
  docType: 'drawings' | 'specs';
  originalPdfPath: string;
  addendumPdfPaths: string[];
  outputPdfPath: string;
  profile?: LayoutProfile;
  options?: {
    mode?: 'replace+insert' | 'replace-only' | 'append-only';
    strict?: boolean;
    verbose?: boolean;
    reportPath?: string;
    regenerateBookmarks?: boolean;
    inventoryOutputDir?: string;
  };
  analyzed?: { plan?: MergePlan };  // Optional plan from analyze (not used in Phase 1)
  corrections?: CorrectionOverlay;   // Optional corrections (not applied in Phase 1)
}
```

### Analyze Outputs

**Inventory Result** (`InventoryResult`):
- **Rows**: One row per page from all PDFs (original + addenda)
  - `id`: Stable identifier (format: `${source}:${pageIndex}:${idPart}`)
  - `normalizedId`: Detected/overridden sheet ID (e.g., "A-101")
  - `page`: Page number (1-based)
  - `status`: `'ok'`, `'warning'`, `'error'`, or `'conflict'`
  - `confidence`: Detection confidence (0.0 to 1.0)
  - `source`: Detection source (`'roi'`, `'legacy'`, etc.)
  - `action`: Planned action (`'replace'`, `'insert'`, `'keep'`, etc.)

- **Issues**: Detection problems
  - `NO_ID`: Page has no detected sheet ID
  - `LOW_CONFIDENCE`: Detection confidence below threshold
  - `DUPLICATE`: Multiple pages with same ID
  - `UNMATCHED`: Pages that couldn't be matched
  - `ROI_DETECTION_FAILURE`: ROI-based detection failed
  - `LEGACY_FALLBACK`: Fell back to legacy detection

- **Conflicts**: Currently empty (reserved for future conflict types)

- **Summary**: Statistics
  - `totalRows`: Total pages analyzed
  - `rowsWithIds`: Pages with detected IDs
  - `rowsWithoutIds`: Pages without IDs
  - `rowsOk`: Pages with no issues
  - `rowsWarning`: Pages with warnings
  - `rowsError`: Pages with errors
  - `replaced`: Number of sheets to be replaced
  - `inserted`: Number of new sheets to be inserted
  - `unmatched`: Number of unmatched pages

- **Meta**: Workflow metadata
  - `docType`: Document type
  - `originalPdfPath`: Original PDF path
  - `addendumPdfPaths`: Addendum PDF paths
  - `mode`: Merge mode
  - `strict`: Strict mode flag

- **Narrative**: Optional narrative instruction set (advisory only, read-only)
  - Extracted and parsed from narrative PDF if provided
  - Contains drawing and spec instructions from narrative

- **Narrative Validation**: Optional narrative validation report (advisory only, read-only)
  - Validates narrative instructions against detected inventory
  - Contains issues with codes: `NARR_SHEET_NOT_FOUND`, `NARR_NEAR_MATCH`, `NARR_AMBIGUOUS_MATCH`, `NARR_INVENTORY_NOT_MENTIONED`
  - Includes near-match suggestions and suggested corrections
  - Never modifies inventory or detection results

### Corrections Supported

1. **Ignore Rows**: Exclude pages from merge
   - Key: `ignoredRowIds: string[]` (array of stable `row.id` values)
   - Effect: Rows remain visible but excluded from summary counts
   - Use case: Skip pages that shouldn't be merged

2. **Override IDs**: Correct detected sheet IDs
   - Key: `overrides[rowId].fields.normalizedId: string`
   - Effect: Updates `row.normalizedId` (stable `row.id` unchanged)
   - Use case: Fix incorrect detections

**Example CorrectionOverlay**:
```json
{
  "ignoredRowIds": ["original-pdf:4:", "addendum1-pdf:10:A-999"],
  "overrides": {
    "original-pdf:5:": {
      "fields": {
        "normalizedId": "A-102"
      }
    }
  }
}
```

### Execute Outputs

**Execute Result** (`ExecuteResult`):
- **Outputs**: File paths (Record<string, string>)
  - `outputPdfPath`: Path to merged PDF
  - `drawings` or `specs`: Type-specific key (same path, based on docType)

- **Summary**: Execution statistics
  - `success`: `true`
  - `replaced`: Number of sheets replaced
  - `inserted`: Number of sheets inserted
  - `unmatched`: Number of unmatched addendum groups (count of `appendedUnmatched` array items)
  - `finalPages`: Final page count
  - `parseTimeMs`: Time spent on detection/parsing
  - `mergeTimeMs`: Time spent on PDF assembly

- **Warnings**: Array of warning messages (if any, undefined if empty)
- **Errors**: Array of error messages (currently always undefined)

### Implementation Details

- **Workflow Runner**: `createMergeWorkflowRunner()`
- **Implementation**: `workflows/merge/mergeWorkflow.ts`
- **CLI Command**: `merge-addenda`
- **GUI Wizard**: Update Documents (4-step wizard)

### Merge Modes

- **`replace+insert`** (default): Replace updated sheets and insert new sheets in correct sequence
- **`replace-only`**: Only replace existing sheets, don't insert new ones
- **`append-only`**: Append all addendum pages to end, don't replace

---

## Extract Documents (Split Set)

**Domain Meaning**: Extract and separate documents by sheet ID with deterministic inventory analysis. For drawings, split by discipline prefix (M, E, P, etc.). For specs, split by division or section.

**Status**: ✅ **Fully Implemented** (Workflow engine + CLI + GUI)

### Implementation

- **CLI Command**: `split-set` (uses workflow engine with analyze/execute pattern)
- **Workflow Engine**: ✅ Fully implemented
- **GUI**: ✅ Fully implemented  
- **Factory**: `createExtractWorkflowRunner()`

### Inputs

**Analyze Input**:
- Input PDF path
- Document type (`drawings` or `specs`)
- Layout profile (optional, for drawings)
- Custom regex pattern (optional)

**Execute Input**:
- Output directory
- Filename format pattern
- Options (verbose, report path, etc.)

### Outputs

**InventoryResult**:
- Rows with detected sheet IDs and pages
- Issues (missing IDs, duplicates, conflicts)
- Summary with pagination and statistics

**ExecuteResult**:
- Multiple PDF files (one per subset)
- Table of contents JSON
- Audit trail report

---

## Assemble Set (Abandoned)

**Status**: ❌ **ABANDONED** - Functionality replaced by Extract Documents workflow composition

**See**: Extract Documents workflow for document separation and composition capabilities

---

## Specs Patch (Abandoned)

**Status**: ❌ **ABANDONED** - Functionality integrated into Extract Documents workflow

**See**: Extract Documents workflow for specification document processing
- Order JSON file (optional, specifies assembly order)

### Planned Outputs

- Single assembled PDF file

---

## Fix Bookmarks

**Domain Meaning**: Read, validate, repair, and write PDF bookmarks. Can rebuild bookmarks from `BookmarkAnchorTree` (Specs Pipeline) or from sheet/section inventory. Useful when bookmarks are missing, incorrect, or need to be regenerated after document changes.

**Status**: ✅ **Implemented** (workflow engine + CLI)

### Inputs

**Analyze Input** (`BookmarksAnalyzeInput`):
```typescript
{
  inputPdfPath: string;
  bookmarkTree?: BookmarkAnchorTree;  // Optional: from Specs Pipeline
  docType?: 'drawings' | 'specs';     // Optional: for inventory-based fallback
  profile?: LayoutProfile;             // Optional: for drawings detection
  options?: {
    verbose?: boolean;
    jsonOutputDir?: string;
  };
}
```

**Execute Input** (`BookmarksExecuteInput`):
```typescript
{
  inputPdfPath: string;
  outputPdfPath: string;
  bookmarkTree?: BookmarkAnchorTree;
  docType?: 'drawings' | 'specs';
  profile?: LayoutProfile;
  options?: {
    verbose?: boolean;
    reportPath?: string;        // Audit trail JSON
    jsonOutputPath?: string;     // Bookmark tree JSON (post-write)
    rebuild?: boolean;           // Full rebuild mode
    style?: BookmarkStyleOptions; // Bookmark style options (profile, maxDepth, etc.)
  };
  analyzed?: {
    bookmarkTree?: BookmarkTree; // From applyCorrections
  };
  corrections?: BookmarksCorrectionOverlay;
}
```

### Analyze Outputs

**Inventory Result** (`InventoryResult`):
- **Rows**: One row per bookmark node
  - `id`: Stable identifier (format: `${source}:${anchor || logicalPath}`)
  - `page`: Page number (1-based)
  - `status`: `'ok'`, `'warning'`, or `'error'`
  - `confidence`: Validation confidence (0.0 to 1.0)
  - `source`: Source of bookmark (`'existing'`, `'bookmarkTree'`, `'inventory'`)

- **Issues**: Validation problems
  - `BOOKMARK_ORPHAN`: Bookmark has no valid destination
  - `BOOKMARK_DEAD_DEST`: Bookmark destination points to invalid page index
  - `BOOKMARK_INVALID_FIT`: Unsupported fit type (reported, not dropped)
  - `BOOKMARK_MISMATCHED_HIERARCHY`: Hierarchy level doesn't match expected structure
  - `BOOKMARK_DUPLICATE_TITLE`: Multiple bookmarks with same title at same level
  - `BOOKMARK_ANCHOR_NOT_FOUND`: Anchor from BookmarkAnchorTree not found in PDF
  - `BOOKMARK_PAGE_HINT_MISMATCH`: pageIndexHint doesn't match resolved page

- **Summary**: Statistics
  - `totalRows`: Total bookmarks analyzed
  - `rowsWithIds`: Bookmarks with stable IDs
  - `rowsOk`: Bookmarks with no issues
  - `rowsWarning`: Bookmarks with warnings
  - `rowsError`: Bookmarks with errors
  - `issuesCount`: Total issues detected

- **Meta**: Workflow metadata
  - `bookmarkTree`: Extracted/validated bookmark tree
  - `sourceTree`: Original BookmarkAnchorTree (if provided)

### Corrections Supported

1. **Rename**: Update bookmark titles (keyed by stable `row.id`)
2. **Reorder**: Reorder bookmarks (array of `row.id` values)
3. **Delete**: Remove bookmarks (array of `row.id` values)
4. **Retarget**: Update destinations (keyed by `row.id`)
5. **Rebuild**: Full rebuild mode (authoritative tree wins, ignore existing)

### Execute Outputs

**Execute Result** (`ExecuteResult`):
- **Outputs**: File paths
  - `outputPdfPath`: Path to PDF with updated bookmarks
  - `reportPath`: Audit trail JSON (if requested)
  - `bookmarkTreeJsonPath`: Post-write bookmark tree JSON (if requested)

- **Summary**: Execution statistics
  - `success`: `true` if validation passed
  - `bookmarksRead`: Number of bookmarks read back after write
  - `bookmarksWritten`: Number of bookmarks written
  - `destinationsValidated`: Number of valid destinations
  - `destinationsInvalid`: Number of invalid destinations

- **Warnings**: Validation issues (if any)

### Implementation Details

- **Workflow Runner**: `createBookmarksWorkflowRunner()`
- **Implementation**: `workflows/bookmarks/bookmarksWorkflow.ts`
- **CLI Command**: `fix-bookmarks`
- **Bookmark Writer**: Python sidecar (pikepdf/QPDF) for cross-viewer reliability

**Outline Tree Structure**:
- All outline items are created as **indirect objects** using `pdf.make_indirect()` (not inline dictionaries)
- Root-level items are linked via `/Next` and `/Prev` for bidirectional traversal
- Items with children have `/First`, `/Last`, and `/Count` properly set; children have `/Parent` references
- Post-write verification ensures all items are indirect, reachable via `/Next` chain, and have proper `/Dest` and `/A` actions
- This ensures PDF-XChange and other viewers can properly display the full bookmark tree

### Integration with Specs Pipeline

The bookmarks workflow can consume `BookmarkAnchorTree` from the Specs Pipeline:
- Provide `BookmarkAnchorTree` JSON via `--bookmark-tree` option
- **Footer-First Section Anchoring** (default with `--rebuild` and `--bookmark-tree`):
  - **ROI Band Detection**: Auto-detects page regions (header: 0-12%, heading: 0-30%, body: 12-88%, footer: 88-100%) using text density analysis
  - **Section Code Extraction**: Extracts section codes (e.g., "23 05 53", "01 23 31") from footer band text only
  - **First-Page Mapping**: Maps each section code to its first occurrence page (lowest page index, 0-based)
  - **Resolution Priority**: Footer-first → heading-based (heading band only) → `pageIndexHint` (clamped) → invalid (marked as error)
  - **Section Page Ranges**: Computed from sorted section start pages for article hierarchy assignment
  - **Junk Title Rejection**: Articles must have title starting with `${anchor} ` or are skipped
  - **Numeric Sorting**: Sections sorted by 3-component tuple `[division, section, subsection]` (e.g., 23 05 48 < 23 05 53 < 23 07 00)
  - **Validation Gate**: Fails on invalid section destinations unless `--allow-invalid-destinations` provided
  - **Strategy Override**: Use `--section-start-strategy <footer-first|heading-only|hint-only>` to override default
- **Page Indexing**: Internal operations use 0-based page indices (PDF convention); viewer display uses 1-based page numbers
- Anchors provide stable identifiers across document revisions

---

## Specs Patch

**Domain Meaning**: Extract Word-generated spec PDFs to structured AST, apply deterministic patch operations (insert/move/renumber/replace/delete), and render back to PDF. Treats specs as structured documents with hierarchical anchors as primary navigation mechanism.

**Status**: ✅ **Implemented** (workflow engine + CLI)

**Document Types Supported**:
- **Specs** (`docType: 'specs'`): Word-generated spec PDFs with hierarchical structure

### Inputs

**Analyze Input** (`SpecsPatchAnalyzeInput`):
```typescript
{
  inputPdfPath: string;
  customSectionPattern?: string;  // Optional custom regex for section ID detection
  options?: {
    verbose?: boolean;
    jsonOutputDir?: string;  // Directory for AST JSON output
  };
}
```

**Execute Input** (`SpecsPatchExecuteInput`):
```typescript
{
  inputPdfPath: string;
  outputPdfPath: string;
  patchPath?: string;  // Path to patch JSON file
  patch?: SpecPatch;   // Inline patch (alternative to patchPath)
  options?: {
    verbose?: boolean;
    reportPath?: string;  // Path for audit trail JSON
    jsonOutputPath?: string;  // Path for AST JSON output
  };
  analyzed?: {
    ast?: SpecDoc;  // Extracted AST from analyze()
  };
  corrections?: CorrectionOverlay;  // Contains patches
}
```

### Analyze Outputs

**Inventory Result** (`InventoryResult`):
- **Rows**: One row per node extracted from PDF
  - `id`: Stable identifier (format: `${source}:${pageIndex}:${nodeId}`)
  - `anchor`: Hierarchical anchor (e.g., "2.4-T.5.b.1") or null if not found
  - `page`: Page number (1-based)
  - `status`: `'ok'`, `'warning'`, or `'error'`
  - `confidence`: Extraction confidence (0.0 to 1.0)
  - `nodeType`: Node type (`'heading'`, `'paragraph'`, `'list-item'`, etc.)
  - `level`: Indentation level
  - `textPreview`: First 100 chars of text

- **Issues**: Extraction problems
  - `NO_SECTION_HEADER`: No section headers detected
  - `ANCHOR_REQUIRED`: Node missing anchor (blocking for patchability)
  - `DUPLICATE_ANCHOR`: Duplicate anchor found within section (blocking)
  - `AMBIGUOUS_ANCHOR`: Multiple anchor candidates (requires disambiguation)
  - `NUMBERING_BREAK`: Numbering sequence breaks
  - `SCANNED_PDF`: PDF appears to be scanned (no text layer)

- **Summary**: Statistics
  - `totalRows`: Total nodes extracted
  - `rowsWithIds`: Nodes with anchors
  - `rowsWithoutIds`: Nodes without anchors
  - `sectionsExtracted`: Number of sections detected
  - `nodesExtracted`: Total nodes extracted

- **Meta**: Workflow metadata
  - `specDoc`: Complete SpecDoc AST
  - `bookmarkTree`: BookmarkAnchorTree for bookmarks pipeline integration

### Corrections Supported

1. **Patch Operations**: Apply patch operations via `CorrectionOverlay.patches` or `patchPath`
   - Operations: `insert`, `move`, `renumber`, `replace`, `delete`
   - All operations require target anchors to be non-null (validation fails otherwise)
   - Disambiguation: Provide `sectionId` or `mustMatchText` if anchor is ambiguous

2. **Ignore Rows**: Exclude nodes from processing
   - Key: `ignoredRowIds: string[]` (array of stable `row.id` values)
   - Effect: Rows remain visible but excluded from summary counts

**Example CorrectionOverlay**:
```json
{
  "patches": [
    {
      "op": "insert",
      "targetAnchor": "2.4-T.5.b.1",
      "position": "after",
      "content": {
        "type": "list-item",
        "text": "New requirement text",
        "level": 2
      }
    }
  ]
}
```

### Execute Outputs

**Execute Result** (`ExecuteResult`):
- **Outputs**: File paths
  - `outputPdfPath`: Path to rendered PDF
  - `astJsonPath`: Path to AST JSON (if `jsonOutputPath` provided)
  - `reportPath`: Path to audit trail JSON (if `reportPath` provided)

- **Summary**: Execution statistics
  - `success`: `true`
  - `sectionsExtracted`: Number of sections extracted
  - `nodesExtracted`: Number of nodes extracted
  - `patchesApplied`: Number of patch operations applied
  - `pagesRendered`: Number of pages in output PDF
  - `issuesCount`: Number of issues detected

- **Warnings**: Array of warning messages (if any)
- **Errors**: Array of error messages (if any)

### Implementation Details

- **Workflow Runner**: `createSpecsPatchWorkflowRunner()`
- **Implementation**: `workflows/specs-patch/specsPatchWorkflow.ts`
- **CLI Command**: `specs-patch`
- **Rendering**: HTML/CSS → PDF via Playwright

### Patch Language

Patch operations are defined in JSON format:

```json
{
  "meta": {
    "version": "1.0",
    "createdAt": "2024-01-01T00:00:00Z",
    "description": "Add new requirement"
  },
  "operations": [
    {
      "op": "insert",
      "targetAnchor": "2.4-T.5.b.1",
      "position": "after",
      "content": {
        "type": "list-item",
        "text": "New requirement",
        "level": 2
      }
    }
  ]
}
```

**Operation Types**:
- `insert`: Insert new node before/after/child of target anchor
- `move`: Move node(s) to new position
- `renumber`: Renumber list items starting from anchor
- `replace`: Replace text content of node
- `delete`: Delete node(s) by anchor

---

## Workflow Comparison

| Workflow | Engine | CLI | GUI | Status |
|----------|--------|-----|-----|--------|
| Update Documents (merge) | ✅ | ✅ | ✅ | Fully implemented |
| Extract Documents (split) | ✅ | ✅ | ✅ | Fully implemented |
| Fix Bookmarks | ✅ | ✅ | ✅ | Fully implemented |
| Specs Patch | ❌ | ❌ | ❌ | Abandoned |
| Assemble Set | ❌ | ❌ | ❌ | Abandoned |

---

## Adding New Workflows

See [ARCHITECTURE.md](ARCHITECTURE.md#extension-guide-adding-a-new-workflow) for a guide on adding new workflows without breaking invariants.

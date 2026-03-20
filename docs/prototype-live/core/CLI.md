# CLI Documentation

**Last verified**: 2026-03-01

This document describes all CLI commands available in `@conset-pdf/cli`.

## Overview

The CLI routes through the same workflow engine as the core library for consistency. Implemented workflow commands include:
- **`merge-addenda`**: Merge addenda into original set (Update Documents workflow)
- **`split-set`**: Extract and separate documents by sheet ID (Extract Documents workflow)
- **`fix-bookmarks`**: Read, validate, and repair PDF bookmarks (Fix Bookmarks workflow)
- **`specs-inventory`**: Generate deterministic spec inventory using footer-first sectionization

Developer diagnostics:
- **`debug-walkthrough`**: Generate deep debug artifacts for troubleshooting and workflow introspection

Abandoned/Superseded commands:
- **`specs-patch`**: Abandoned - functionality integrated into Extract workflow
- **`assemble-set`**: Abandoned - replaced by modular composition in Extract workflow

## Installation

```bash
npm install -g @conset-pdf/cli
```

Or use via npx:
```bash
npx @conset-pdf/cli <command>
```

## Commands

### merge-addenda

Merge an original PDF with one or more addendum PDFs. Replaces updated sheets and inserts new sheets.

**Purpose**: Update construction document sets by merging addenda in chronological order.

**Usage**:
```bash
conset-pdf merge-addenda \
  --original <path> \
  --addenda <paths...> \
  --output <path> \
  --type <type> \
  [options]
```

**Required Arguments**:
- `--original <path>`: Path to original PDF
- `--addenda <paths...>`: Paths to addendum PDFs (one or more, in chronological order)
- `--output <path>`: Path to output PDF (required but not used in `--dry-run` mode)
- `--type <type>`: Document type (`drawings` or `specs`)
  - **`drawings`**: Uses ROI-based detection with layout profiles (or legacy fallback)
  - **`specs`**: Uses text-based section ID detection (layout profiles not supported/ignored)

**Options**:
- `--report <path>`: Path to write JSON report
- `--mode <mode>`: Merge mode (`replace+insert`, `replace-only`, or `append-only`). Default: `replace+insert`
- `--strict`: Fail on pages without IDs (default: false)
- `--dry-run`: Plan merge without writing output PDF (outputs inventory JSON)
- `--json-output <path>`: Path to write dry-run inventory JSON (default: stdout if `--dry-run`)
- `--verbose`: Verbose output (default: false)
- `--bookmark`: Regenerate bookmarks from detected sheet numbers and titles (default: false)
- `--layout <path>`: Path to layout profile JSON
- `--sheet-id-roi <roi>`: Sheet ID ROI: `"x,y,width,height"` (normalized 0-1)
- `--sheet-title-roi <roi>`: Sheet title ROI: `"x,y,width,height"` (normalized 0-1)
- `--auto-layout`: Reserved flag (currently unimplemented/no-op; kept for planned V4 capability)
- `--save-layout <path>`: Reserved flag (currently unimplemented/no-op; kept for planned V4 capability)
- `--inventory-dir <path>`: Directory for inventory JSON files (default: next to source PDFs)
- `--narrative <path>`: Path to narrative PDF for advisory analysis (optional)

**Input Format**: PDF files (original + addenda)

**Output Format**:
- **Normal mode**: Output PDF file
- **Dry-run mode**: JSON inventory file (or stdout)

**Dry-Run Inventory JSON Shape**:
```json
{
  "workflowId": "merge",
  "rows": [
    {
      "id": "original-pdf:0:A-101",
      "page": 1,
      "status": "ok",
      "confidence": 0.95,
      "source": "roi",
      "normalizedId": "A-101",
      "notes": null,
      "tags": ["roi"]
    }
  ],
  "issues": [
    {
      "id": "issue-0",
      "severity": "warning",
      "code": "NO_ID",
      "message": "No sheet ID found on page 5",
      "rowIds": ["original-pdf:4:"]
    }
  ],
  "conflicts": [],
  "summary": {
    "totalRows": 50,
    "rowsWithIds": 48,
    "rowsWithoutIds": 2,
    "rowsOk": 48,
    "rowsWarning": 2,
    "rowsError": 0,
    "rowsConflict": 0,
    "issuesCount": 2,
    "conflictsCount": 0,
    "replaced": 10,
    "inserted": 5,
    "unmatched": 2
  },
  "meta": {
    "docType": "drawings",
    "originalPdfPath": "Original.pdf",
    "addendumPdfPaths": ["Addendum1.pdf"],
    "mode": "replace+insert",
    "strict": false
  }
}
```

**Examples**:

Basic merge (drawings):
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf Addendum2.pdf \
  --output Final.pdf \
  --type drawings \
  --layout layout.json
```

Basic merge (specs):
```bash
conset-pdf merge-addenda \
  --original Original-Specs.pdf \
  --addenda Addendum1-Specs.pdf \
  --output Final-Specs.pdf \
  --type specs
```

**Note**: Specs merging uses text-based section ID detection and does not require (or support) layout profiles. The `--layout` option is ignored for specs type.

Dry-run inventory analysis:
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --type drawings \
  --dry-run \
  --json-output inventory.json
```

With inline ROI:
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --output Final.pdf \
  --type drawings \
  --sheet-id-roi "0.1,0.05,0.3,0.05" \
  --sheet-title-roi "0.1,0.1,0.6,0.05"
```

With narrative PDF (advisory):
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --output Final.pdf \
  --type drawings \
  --layout layout.json \
  --narrative Narrative.pdf
```

**Exit Codes**:
- `0`: Success
- `2`: Invalid arguments or validation error
- `3`: Strict mode violation (unmatched pages)
- `4`: File I/O error (file not found, permission denied)

**Important**:
- `--auto-layout` and `--save-layout` are accepted by the command parser but are not currently wired into merge execution behavior.

---

### detect

Preview sheet ID/title extraction using layout profile. Tests detection on specified pages.

**Purpose**: Test layout profile configuration and verify sheet ID detection accuracy.

**Usage**:
```bash
conset-pdf detect \
  --input <path> \
  [options]
```

**Required Arguments**:
- `--input <path>`: Path to input PDF

**Options**:
- `--layout <path>`: Path to layout profile JSON
- `--sheet-id-roi <roi>`: Sheet ID ROI: `"x,y,width,height"` (normalized 0-1)
- `--sheet-title-roi <roi>`: Sheet title ROI: `"x,y,width,height"` (normalized 0-1)
- `--pages <pages>`: Comma-separated page numbers (1-based). Default: `1,5,10`
- `--type <type>`: Document type (`drawings` or `specs`). Default: `drawings`
- `--output-preview <path>`: Path to write preview JSON
- `--preview-dir <path>`: Directory for preview JSON (auto-generates filename)
- `--verbose`: Verbose output (default: false)

**Input Format**: PDF file

**Output Format**: JSON preview file (or console summary)

**Preview JSON Shape**:
```json
{
  "profile": "layout.json",
  "locator": "RoiSheetLocator",
  "pages": [
    {
      "pageIndex": 0,
      "pageNumber": 1,
      "sheetId": {
        "found": true,
        "value": "A-101",
        "normalized": "A-101",
        "confidence": 0.95,
        "method": "roi"
      },
      "sheetTitle": {
        "found": true,
        "value": "Floor Plan - First Floor"
      },
      "warnings": [],
      "context": "Additional context from detection"
    },
    {
      "pageIndex": 1,
      "pageNumber": 2,
      "sheetId": {
        "found": false,
        "error": "No sheet ID found",
        "failureReason": "ROI_EMPTY",
        "method": "roi"
      },
      "sheetTitle": {
        "found": false,
        "error": "No title found"
      },
      "warnings": ["ROI region is empty"]
    }
  ],
  "summary": {
    "totalPages": 2,
    "pagesWithId": 1,
    "pagesWithTitle": 1,
    "successRate": 0.5
  }
}
```

**Failure Reasons** (when `sheetId.found: false`):
- `ROI_EMPTY`: ROI region contains no text
- `ROI_LOW_TEXT_DENSITY`: ROI region has insufficient text
- `ROI_NO_PATTERN_MATCH`: No pattern match found in ROI
- `ROI_PREFIX_REJECTED`: Prefix validation failed
- `ROI_MULTIPLE_MATCHES`: Multiple matches found (ambiguous)
- `ROI_FAILED`: General ROI detection failure
- `LEGACY_FALLBACK`: Fell back to legacy detection
- `NO_MATCH`: No match found (no warnings)

**Examples**:

Test layout profile:
```bash
conset-pdf detect \
  --input Set.pdf \
  --layout layout.json \
  --pages 1,5,20 \
  --output-preview preview.json
```

Test with inline ROI:
```bash
conset-pdf detect \
  --input Set.pdf \
  --sheet-id-roi "0.1,0.05,0.3,0.05" \
  --pages 1,5,10 \
  --verbose
```

**Exit Codes**:
- `0`: Success
- `2`: Invalid arguments or validation error
- `4`: File I/O error

---

### split-set

Split a PDF set into discipline-specific subsets.

**Purpose**: Divide a construction document set into separate PDFs by discipline (e.g., M, E, P for mechanical, electrical, plumbing).

**Usage**:
```bash
conset-pdf split-set \
  --input <path> \
  --output-dir <path> \
  --type <type> \
  [options]
```

**Required Arguments**:
- `--input <path>`: Path to input PDF
- `--output-dir <path>`: Path to output directory
- `--type <type>`: Document type (`drawings` or `specs`)

**Options**:
- `--group-by <method>`: Grouping method (`prefix`, `section`, or `division`)
- `--prefixes <prefixes...>`: Allowed prefixes for drawings (e.g., `M E P`)
- `--toc-json <path>`: Path to write table of contents JSON
- `--pattern <regex>`: Custom regex pattern for ID detection
- `--verbose`: Verbose output (default: false)

**Input Format**: PDF file

**Output Format**: Multiple PDF files in output directory, one per subset

**Examples**:

Split by prefix:
```bash
conset-pdf split-set \
  --input Set.pdf \
  --output-dir ./output \
  --type drawings \
  --group-by prefix \
  --prefixes M E P
```

Split specs by division:
```bash
conset-pdf split-set \
  --input Specs.pdf \
  --output-dir ./output \
  --type specs \
  --group-by division \
  --toc-json toc.json
```

**Exit Codes**:
- `0`: Success
- `2`: Invalid arguments or validation error
- `4`: File I/O error

**Note**: This command now uses the complete workflow engine (analyze/execute pattern) with full inventory analysis and corrections support.

---

### assemble-set (Abandoned)

**Status**: ❌ **ABANDONED** - This command is no longer maintained. Functionality has been replaced by the Extract Documents (split-set) workflow's modular composition capabilities.

**See**: [split-set](#split-set) for document extraction and composition.

**Note**: This command previously used the legacy `assembleSet()` API.

---

---

### specs-patch (Abandoned)

**Status**: ❌ **ABANDONED** - This command is no longer maintained. Functionality has been integrated into the Extract Documents (split-set) workflow.

**See**: [split-set](#split-set) for the current approach to document extraction and processing.

---

**Input Format**: PDF file (Word-generated spec PDF)

**Output Format**:
- **Normal mode**: Output PDF file + optional AST JSON + optional audit trail JSON
- **Dry-run mode**: JSON inventory file (or stdout)

**Dry-Run Inventory JSON Shape**:
```json
{
  "workflowId": "specs-patch",
  "rows": [
    {
      "id": "input-pdf:0:section-23-09-00-5-node-0",
      "page": 5,
      "status": "ok",
      "confidence": 1.0,
      "anchor": "2.4-T.5.b.1",
      "nodeType": "list-item",
      "level": 2,
      "textPreview": "Requirement text..."
    }
  ],
  "issues": [
    {
      "id": "issue-0",
      "severity": "error",
      "code": "ANCHOR_REQUIRED",
      "message": "Node missing anchor (required for patchability)",
      "rowIds": ["input-pdf:0:section-23-09-00-5-node-1"]
    }
  ],
  "conflicts": [],
  "summary": {
    "totalRows": 50,
    "rowsWithIds": 48,
    "rowsWithoutIds": 2,
    "rowsOk": 48,
    "rowsWarning": 0,
    "rowsError": 2,
    "sectionsExtracted": 3,
    "nodesExtracted": 50
  },
  "meta": {
    "specDoc": { /* SpecDoc AST */ },
    "bookmarkTree": { /* BookmarkAnchorTree */ }
  }
}
```

**Canonical Usage Example**:

Run from the repository root. This example demonstrates the recommended workflow:

```bash
# Dry-run: Analyze spec PDF and generate inventory JSON
node packages/cli/dist/cli.js specs-patch \
  --input path/to/Specs.pdf \
  --dry-run \
  --json-output specs-patch-inventory.json \
  --verbose

# Execute: Apply patches and render to PDF (with AST output)
node packages/cli/dist/cli.js specs-patch \
  --input path/to/Specs.pdf \
  --output Specs-Patched.pdf \
  --patch patch.json \
  --json-output specs-patch-ast.json \
  --report specs-patch-audit-trail.json \
  --verbose
```

**Examples**:

Basic extract and render:
```bash
node packages/cli/dist/cli.js specs-patch \
  --input Specs.pdf \
  --output Specs-Output.pdf
```

With patch file and outputs:
```bash
node packages/cli/dist/cli.js specs-patch \
  --input Specs.pdf \
  --output Specs-Patched.pdf \
  --patch patch.json \
  --json-output specs-patch-ast.json \
  --report specs-patch-audit-trail.json
```

Dry-run analysis (inventory JSON):
```bash
node packages/cli/dist/cli.js specs-patch \
  --input Specs.pdf \
  --dry-run \
  --json-output specs-patch-inventory.json
```

**Note**: All examples assume execution from the repository root. For installed CLI usage, replace `node packages/cli/dist/cli.js` with `conset-pdf` (if installed globally) or `npx @conset-pdf/cli`.

**Exit Codes**:
- `0`: Success
- `2`: Invalid arguments or validation error
- `4`: File I/O error (file not found, permission denied)

---

### fix-bookmarks

Fix PDF bookmarks: read, validate, repair, and write bookmarks. Can rebuild bookmarks from `BookmarkAnchorTree` (Specs Pipeline) or from sheet/section inventory.

**Purpose**: Regenerate or repair PDF bookmarks that are missing, incorrect, or need updating after document changes.

**Usage**:
```bash
conset-pdf fix-bookmarks \
  --input <path> \
  --output <path> \
  [options]
```

**Required Arguments**:
- `--input <path>`: Input PDF path

**Options**:
- `--output <path>`: Output PDF path (required unless `--dry-run`)
- `--source <type>`: Source type for inventory-based fallback (`specs`, `drawings`, or `auto`). Default: `auto`
- `--bookmark-tree <path>`: Path to `BookmarkAnchorTree` JSON (from Specs Pipeline)
- `--profile <path>`: Path to layout profile JSON (for drawings detection)
- `--sheet-id-roi <roi>`: Sheet ID ROI: `"x,y,width,height"` (normalized 0-1)
- `--sheet-title-roi <roi>`: Sheet title ROI: `"x,y,width,height"` (normalized 0-1)
- `--bookmark-profile <id>`: Bookmark profile: `raw`, `specs-v1`, or `specs-v2-detailed`. Default: `specs-v1` if `--bookmark-tree` and `--rebuild`, `raw` otherwise
- `--max-depth <number>`: Maximum bookmark depth (overrides profile default)
- `--max-title-length <number>`: Maximum title length before truncation (overrides profile default)
- `--include-subsections`: Include subsections (only meaningful for `specs-v2-detailed`)
- `--dry-run`: Analyze only, output inventory JSON (no PDF write)
- `--json-output <path>`: Path to write JSON output (behavior depends on mode):
  - **Dry-run mode**: Writes full inventory JSON (analyze results with rows, issues, summary, and meta including bookmark tree)
  - **Execute mode**: Writes bookmark tree JSON (post-write bookmark tree structure only)
- `--report <path>`: Path for audit trail JSON (execute mode only)
- `--rebuild`: Full rebuild mode (authoritative tree wins, ignore existing)
- `--section-start-strategy <strategy>`: Section start resolution strategy:
  - `footer-first` (default when `--bookmark-tree` provided): Extract section codes from footer text, map to first occurrence page
  - `heading-only`: Use layout-aware heading resolver (heading band only)
  - `hint-only`: Use `pageIndexHint` from bookmark tree (least reliable)
- `--allow-invalid-destinations`: Allow invalid section destinations (override validation gate)
- `--verbose`: Verbose output

**Examples**:

1. **Dry-run: Analyze existing bookmarks**
```bash
conset-pdf fix-bookmarks \
  --input drawing-set.pdf \
  --dry-run \
  --json-output fix-bookmarks-inventory.json
```

2. **Fix bookmarks using Specs Pipeline output (defaults to specs-v1 with --rebuild)**
```bash
conset-pdf fix-bookmarks \
  --input spec.pdf \
  --output spec-fixed.pdf \
  --bookmark-tree specs-bookmark-tree.json \
  --rebuild \
  --json-output fix-bookmarks-tree.json \
  --report fix-bookmarks-audit.json
```

3. **Fix bookmarks with detailed subsections (specs-v2-detailed)**
```bash
conset-pdf fix-bookmarks \
  --input spec.pdf \
  --output spec-fixed.pdf \
  --bookmark-tree specs-bookmark-tree.json \
  --rebuild \
  --bookmark-profile specs-v2-detailed \
  --include-subsections \
  --max-depth 4 \
  --json-output fix-bookmarks-tree.json \
  --report fix-bookmarks-audit.json
```

4. **Preserve existing bookmarks (raw profile)**
```bash
conset-pdf fix-bookmarks \
  --input existing.pdf \
  --output existing-preserved.pdf \
  --bookmark-profile raw \
  --json-output fix-bookmarks-tree.json
```

5. **Rebuild bookmarks from sheet inventory**
```bash
conset-pdf fix-bookmarks \
  --input drawing-set.pdf \
  --output drawing-set-fixed.pdf \
  --source drawings \
  --profile layout.json \
  --rebuild \
  --json-output fix-bookmarks-tree.json
```

6. **Fix bookmarks with inventory fallback**
```bash
conset-pdf fix-bookmarks \
  --input drawing-set.pdf \
  --output drawing-set-fixed.pdf \
  --source drawings \
  --sheet-id-roi "0.1,0.9,0.15,0.05"
```

**Outputs**:
- **Dry-run mode**: Prints full inventory JSON to stdout (or `--json-output` file). Includes rows, issues, summary, and meta with bookmark tree.
- **Execute mode**: Writes PDF with updated bookmarks, optionally writes:
  - Audit trail JSON (`--report`): Complete execution report with validation results
  - Bookmark tree JSON (`--json-output`): Post-write bookmark tree structure only

**Integration with Specs Pipeline**:
- Provide `BookmarkAnchorTree` JSON via `--bookmark-tree` option
- **Footer-First Section Anchoring** (default with `--rebuild`):
  - Auto-detects page regions (header, heading, body, footer) using text density analysis
  - Extracts section codes (e.g., "23 05 53") from footer text on each page
  - Maps each section code to its first occurrence page
  - Uses footer mapping for section bookmark destinations (avoids cross-reference false matches)
  - Falls back to heading-based resolver if footer mapping unavailable
  - Falls back to `pageIndexHint` if both footer and heading resolution fail
- **Page Indexing**: Internal operations use 0-based page indices (PDF convention); viewer display uses 1-based page numbers
- Anchors provide stable identifiers across document revisions

**Bookmark Profiles**:
- **raw**: Preserves bookmarks as-is with minimal normalization (safe default for existing bookmarks)
- **specs-v1**: Filters to SECTION/PART/Article hierarchy only (max depth 2). Default when `--bookmark-tree` and `--rebuild` are provided.
- **specs-v2-detailed**: Like specs-v1 but can include deeper structural subsections when `--include-subsections` is enabled (max depth 4)

**Default Behavior**:
- When `--bookmark-tree` and `--rebuild` are provided: defaults to `specs-v1` profile
- Otherwise: defaults to `raw` profile (preserves existing bookmarks)
- Use `--bookmark-profile` to override the default

**Profile Features**:
- **specs-v1**: Filters out non-structural nodes (body text fragments, list items), normalizes titles, deduplicates roots, sorts by section/part/article order
- **specs-v2-detailed**: Same as specs-v1, plus allows structural subsections (e.g., "2.4-T.5.b.1") when `--include-subsections` is enabled
- **raw**: Minimal normalization only (whitespace collapse, truncation if max-title-length specified)

**Destination Format**:
- Bookmarks are written with `/Fit` view type by default for maximum viewer compatibility
- Page references use indirect objects (not inline dictionaries) for proper viewer support
- Both `/Dest` and `/A GoTo` actions are written for maximum compatibility (some viewers like PDF-XChange prefer `/A`)
- Post-write verification ensures destinations are valid and viewer-compatible

**Section Start Resolution** (controlled by `--section-start-strategy`):
- **Footer-First** (default with `--rebuild` and `--bookmark-tree`): 
  - Extracts section codes from footer text (footer band: bottom 12% of page)
  - Maps each section code to its first occurrence page (lowest page index)
  - Most reliable for specs documents with repeating footer lines
  - Falls back to heading-based resolver if footer mapping unavailable
- **Heading-Only**: Searches heading band (top 0-30%) for section headings. Layout-aware, avoids cross-references in body text.
- **Hint-Only**: Uses `pageIndexHint` from `BookmarkAnchorTree` as last resort. May be incorrect/stale.
- Section bookmarks are sorted numerically by section code (e.g., 23 05 48 < 23 05 53 < 23 07 00)
- **Validation Gate**: If any section destination is invalid, `fix-bookmarks` fails unless `--allow-invalid-destinations` is provided

**Requirements**:
- Python 3.8+ with `pikepdf>=8.0.0` installed (for bookmark writing)
- Run `pip install pikepdf>=8.0.0` before using

---

### specs-inventory

Generate deterministic spec inventory using footer-first sectionization.

**Purpose**: Extract section structure from spec PDFs using footer-based section detection. Useful for analyzing spec document structure and preparing for bookmark generation.

**Usage**:
```bash
conset-pdf specs-inventory \
  --input <path> \
  [options]
```

**Required Arguments**:
- `--input <path>`: Path to input PDF

**Options**:
- `--output <path>`: Path to write JSON output (default: stdout)
- `--verbose`: Verbose output (default: false)
- `--sample-count <count>`: Number of pages to sample for region detection (default: 30)

**Input Format**: PDF file (spec PDF)

**Output Format**: JSON inventory file (or stdout)

**Output JSON Shape**:
```json
{
  "inputPdf": "Specs.pdf",
  "pageCount": 500,
  "regions": {
    "header": { "yMin": 0.0, "yMax": 0.12 },
    "footer": { "yMin": 0.88, "yMax": 1.0 }
  },
  "sectionRuns": [
    {
      "sectionId": "23 09 00",
      "startPageIndex": 10,
      "endPageIndex": 45,
      "pageCount": 36,
      "needsCorrection": false,
      "pages": [...]
    }
  ],
  "auditRecords": [...],
  "pageAssignments": [...],
  "summary": {
    "totalRuns": 25,
    "totalPages": 500,
    "pagesWithSectionId": 485,
    "pagesNeedingCorrection": 15,
    "auditRecordCount": 20
  }
}
```

**Examples**:

Basic inventory generation:
```bash
conset-pdf specs-inventory \
  --input Specs.pdf \
  --output specs-inventory.json
```

Verbose output:
```bash
conset-pdf specs-inventory \
  --input Specs.pdf \
  --output specs-inventory.json \
  --verbose \
  --sample-count 50
```

**Exit Codes**:
- `0`: Success
- `2`: Invalid arguments or validation error
- `4`: File I/O error

**Note**: This command uses `detectPageRegions()` and `sectionizePages()` from `@conset-pdf/core` to perform footer-first sectionization. It's a utility command for analyzing spec document structure.

---

## Common Patterns

### Using Layout Profiles

Layout profiles define ROI regions for sheet ID and title detection. Create a profile JSON:

```json
{
  "name": "Standard Title Block",
  "version": "1.0",
  "sheetId": {
    "rois": [
      {
        "x": 0.1,
        "y": 0.05,
        "width": 0.3,
        "height": 0.05
      }
    ]
  },
  "sheetTitle": {
    "rois": [
      {
        "x": 0.1,
        "y": 0.1,
        "width": 0.6,
        "height": 0.05
      }
    ]
  }
}
```

Use with `--layout` option:
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --output Final.pdf \
  --type drawings \
  --layout layout.json
```

### Dry-Run Workflow

1. Run dry-run to analyze inventory:
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --type drawings \
  --dry-run \
  --json-output inventory.json
```

2. Review `inventory.json` for issues and conflicts

3. Run actual merge:
```bash
conset-pdf merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --output Final.pdf \
  --type drawings \
  --layout layout.json
```

### Testing Detection

Before merging, test detection accuracy:
```bash
conset-pdf detect \
  --input Original.pdf \
  --layout layout.json \
  --pages 1,5,10,15,20 \
  --output-preview preview.json
```

Review `preview.json` to verify sheet IDs are detected correctly.

---

## Integration with Workflow Engine

The CLI routes through the same workflow engine as the GUI:

- **`merge-addenda`** (with `--dry-run`): Uses `runner.analyze()`
- **`merge-addenda`** (normal): Uses `runner.execute()`
- **`fix-bookmarks`** (with `--dry-run`): Uses `runner.analyze()`
- **`fix-bookmarks`** (normal): Uses `runner.execute()`
- **`specs-patch`** (with `--dry-run`): Uses `runner.analyze()`
- **`specs-patch`** (normal): Uses `runner.execute()`
- Other commands: Use legacy APIs directly (workflow engine not yet implemented)

This ensures consistency between CLI and GUI behavior.

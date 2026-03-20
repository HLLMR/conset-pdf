# CLI Reference Guide

**Tool**: `conset-pdf`  
**Version**: 1.0.0  
**Node.js**: ≥ 18.0.0

## Usage

```bash
npx conset-pdf [command] [options]
```

## Global Options

- `--help` - Show help message
- `--version` - Show version

---

## Commands

### `merge-addenda`

**Purpose**: Merge addenda PDFs into an original PDF, replacing or inserting sheets.

**Syntax**:
```bash
conset-pdf merge-addenda --original <path> --addenda <paths...> --output <path> --type <type> [options]
```

**Required Options**:
- `--original <path>` - Path to original PDF file
- `--addenda <paths...>` - One or more addendum PDF paths (space-separated, in chronological order)
- `--output <path>` - Output PDF path (not used with `--dry-run`)
- `--type <type>` - Document type: `drawings` or `specs`

**Optional Options**:
- `--mode <mode>` - Merge mode (default: `replace+insert`)
  - `replace+insert` - Replace matched sheets, insert new ones
  - `replace-only` - Replace matched sheets, skip unmatched
  - `append-only` - Append all as new, no replacements
  
- `--strict` - Fail if any page lacks an ID (default: false)

- `--dry-run` - Plan merge without writing output PDF
  - Outputs inventory JSON instead of final PDF
  - Use with `--json-output` to capture results

- `--json-output <path>` - Path to write inventory JSON (dry-run only)

- `--report <path>` - Write detailed JSON merge report

- `--bookmark` - Regenerate bookmarks from detected sheet numbers and titles

- `--layout <path>` - Path to layout profile JSON file
  - See [Layout Profile Format](#layout-profile-format) below

- `--sheet-id-roi <roi>` - Define sheet ID region inline
  - Format: `x,y,width,height` (0-1 normalized, e.g., "0.05,0.85,0.2,0.1")

- `--sheet-title-roi <roi>` - Define sheet title region inline
  - Format: `x,y,width,height` (0-1 normalized)

- `--inventory-dir <path>` - Directory for intermediate inventory JSON files

- `--narrative <path>` - Path to narrative addenda PDF for advisory analysis

- `--verbose` - Verbose console output

#### Examples

**Basic merge with replacement+insertion**:
```bash
conset-pdf merge-addenda \
  --original drawings.pdf \
  --addenda addendum1.pdf addendum2.pdf \
  --output merged.pdf \
  --type drawings
```

**Dry-run to inspect changes**:
```bash
conset-pdf merge-addenda \
  --original specs.pdf \
  --addenda addendum.pdf \
  --output specs-merged.pdf \
  --type specs \
  --dry-run \
  --json-output inventory.json
```

**With layout profile and narrative**:
```bash
conset-pdf merge-addenda \
  --original drawings.pdf \
  --addenda addendum.pdf \
  --output result.pdf \
  --type drawings \
  --layout profile.json \
  --narrative addendum-notes.pdf \
  --verbose
```

**Using inline ROI regions**:
```bash
conset-pdf merge-addenda \
  --original drawings.pdf \
  --addenda addendum.pdf \
  --output result.pdf \
  --type drawings \
  --sheet-id-roi "0.05,0.85,0.2,0.1" \
  --sheet-title-roi "0.05,0.75,0.9,0.08"
```

---

### `split-set`

**Purpose**: Split a PDF by sheet IDs (drawings) or section IDs (specs) into multiple files.

**Syntax**:
```bash
conset-pdf split-set --input <path> --output-dir <dir> --type <type> [options]
```

**Required Options**:
- `--input <path>` - Input PDF file path
- `--output-dir <dir>` - Output directory for split files
- `--type <type>` - Document type: `drawings` or `specs`

**Optional Options**:
- `--group-by <strategy>` - Grouping strategy (default: depends on type)
  - `drawings`: `prefix` (groups by discipline prefix: A, S, M, E, etc.)
  - `specs`: `section` (groups by MasterFormat section)
  - Alternative: `division` (CSI division level)

- `--prefixes <prefixes...>` - Filter by discipline prefixes (drawings only)
  - Example: `--prefixes A S M` (only Architectural, Structural, Mechanical)

- `--pattern <regex>` - Custom regex for ID extraction

- `--toc-json <path>` - JSON file mapping group keys to titles

- `--verbose` - Verbose output

#### Examples

**Split drawings by discipline prefix**:
```bash
conset-pdf split-set \
  --input drawings.pdf \
  --output-dir ./drawings-split \
  --type drawings \
  --group-by prefix
```

**Split specs by section**:
```bash
conset-pdf split-set \
  --input specifications.pdf \
  --output-dir ./specs-split \
  --type specs \
  --group-by section
```

**Split only specific disciplines**:
```bash
conset-pdf split-set \
  --input drawings.pdf \
  --output-dir ./arch-struct \
  --type drawings \
  --prefixes A S
```

**Output**: Creates files like:
- `A-sheets.pdf` (Architecture)
- `S-sheets.pdf` (Structural)
- `M-sheets.pdf` (Mechanical)

---

### `assemble-set`

**Purpose**: Combine multiple PDF files into a single output PDF.

**Syntax**:
```bash
conset-pdf assemble-set --input-dir <dir> --output <path> --type <type> [options]
```

**Required Options**:
- `--input-dir <dir>` - Directory containing PDF files to assemble
- `--output <path>` - Output PDF file path
- `--type <type>` - Document type: `drawings` or `specs`

**Optional Options**:
- `--order-json <path>` - JSON file specifying file order
  - Default: Alphabetical sorting

- `--verbose` - Verbose output

#### Order JSON Format

```json
{
  "order": [
    "A-sheets.pdf",
    "S-sheets.pdf",
    "M-sheets.pdf",
    "E-sheets.pdf"
  ]
}
```

#### Examples

**Assemble with alphabetical order**:
```bash
conset-pdf assemble-set \
  --input-dir ./drawings-split \
  --output assembled.pdf \
  --type drawings
```

**Assemble with custom order**:
```bash
conset-pdf assemble-set \
  --input-dir ./drawings-split \
  --output assembled.pdf \
  --type drawings \
  --order-json order.json
```

---

### `fix-bookmarks`

**Purpose**: Repair or regenerate PDF bookmarks.

**Syntax**:
```bash
conset-pdf fix-bookmarks --pdf <path> --output <path> --type <type> [options]
```

**Required Options**:
- `--pdf <path>` - Input PDF file path
- `--output <path>` - Output PDF with fixed bookmarks
- `--type <type>` - Document type: `drawings` or `specs`

**Optional Options**:
- `--section-start-strategy <strategy>` - How to detect section starts (default: `footer-first`)
  - `footer-first` - Extract section codes from footer band
  - `inventory` - Use page inventory mapping

- `--allow-invalid-destinations` - Permit invalid bookmark destinations (default: false)

- `--corrections <path>` - JSON file with bookmark corrections

- `--verbose` - Verbose output

#### Corrections Format

```json
{
  "corrections": [
    {
      "action": "rename",
      "from": "Old Title",
      "to": "New Title"
    },
    {
      "action": "delete",
      "title": "Unwanted Section"
    },
    {
      "action": "retarget",
      "title": "Section A",
      "pageNumber": 5
    }
  ]
}
```

#### Examples

**Generate bookmarks from footer**:
```bash
conset-pdf fix-bookmarks \
  --pdf document.pdf \
  --output bookmarked.pdf \
  --type drawings \
  --section-start-strategy footer-first
```

**Fix with corrections**:
```bash
conset-pdf fix-bookmarks \
  --pdf document.pdf \
  --output fixed.pdf \
  --type specs \
  --corrections bookmark-corrections.json
```

---

### `specs-patch`

**Purpose**: Apply patches to specification PDFs.

**Syntax**:
```bash
conset-pdf specs-patch --pdf <path> --patches <path> --output <path> [options]
```

**Required Options**:
- `--pdf <path>` - Input specifications PDF
- `--patches <path>` - JSON file with patches to apply
- `--output <path>` - Output PDF path

**Optional Options**:
- `--dry-run` - Plan patches without writing output
- `--report <path>` - Write patch report
- `--verbose` - Verbose output

#### Patches Format

```json
{
  "patches": [
    {
      "sectionId": "23 09 00",
      "action": "revise",
      "content": "Replacement text or JSON description"
    },
    {
      "sectionId": "07 41 13",
      "action": "add",
      "content": "New section content"
    },
    {
      "sectionId": "01 41 00",
      "action": "delete"
    }
  ]
}
```

#### Examples

```bash
conset-pdf specs-patch \
  --pdf specifications.pdf \
  --patches patches.json \
  --output patched-specs.pdf
```

---

### `detect`

**Purpose**: Detect and extract sheet IDs or section IDs from a PDF without modification.

**Syntax**:
```bash
conset-pdf detect --pdf <path> --type <type> [options]
```

**Required Options**:
- `--pdf <path>` - Input PDF file
- `--type <type>` - Document type: `drawings` or `specs`

**Optional Options**:
- `--layout <path>` - Layout profile for ROI-based detection
- `--sheet-id-roi <roi>` - Inline sheet ID ROI
- `--sheet-title-roi <roi>` - Inline sheet title ROI
- `--output <path>` - Write results to JSON file
- `--verbose` - Verbose output

#### Example Output (JSON)

```json
{
  "pages": [
    {
      "pageIndex": 0,
      "pageNumber": 1,
      "id": "A1.0",
      "normalized": "A1.0",
      "title": "Cover Sheet",
      "confidence": 0.95,
      "method": "roi"
    },
    {
      "pageIndex": 1,
      "pageNumber": 2,
      "id": "A1.1",
      "normalized": "A1.1",
      "title": "Site Plan",
      "confidence": 0.92,
      "method": "roi"
    }
  ]
}
```

#### Examples

```bash
# Detect sheet IDs
conset-pdf detect --pdf drawings.pdf --type drawings --verbose

# Detect with layout profile
conset-pdf detect \
  --pdf drawings.pdf \
  --type drawings \
  --layout profile.json \
  --output detection-results.json

# Detect using inline ROI
conset-pdf detect \
  --pdf drawings.pdf \
  --type drawings \
  --sheet-id-roi "0.05,0.85,0.2,0.1" \
  --output results.json
```

---

### `specs-inventory`

**Purpose**: Extract specification section inventory without modification.

**Syntax**:
```bash
conset-pdf specs-inventory --pdf <path> [options]
```

**Required Options**:
- `--pdf <path>` - Input specifications PDF

**Optional Options**:
- `--output <path>` - Write inventory to JSON file
- `--include-text` - Include full section text (default: headers only)
- `--verbose` - Verbose output

#### Example Output

```json
{
  "sections": [
    {
      "sectionId": "01 41 00",
      "normalized": "01 41 00",
      "title": "Regulatory Requirements",
      "pageStart": 1,
      "pageEnd": 5,
      "confidence": 0.98
    },
    {
      "sectionId": "23 09 00",
      "normalized": "23 09 00",
      "title": "Mechanical HVAC",
      "pageStart": 45,
      "pageEnd": 142,
      "confidence": 0.95
    }
  ]
}
```

#### Examples

```bash
conset-pdf specs-inventory --pdf specifications.pdf --output inventory.json
```

---

### `debug-walkthrough`

**Purpose**: Step through PDF operations interactively for debugging.

**Syntax**:
```bash
conset-pdf debug-walkthrough --pdf <path> --type <type> [options]
```

This command is intended for development and troubleshooting, allowing inspection of intermediate stages.

---

## Exit Codes

- `0` - Success
- `1` - Unspecified error
- `2` - Invalid options/arguments
- `3` - File not found
- `4` - File system error
- `5` - PDF processing error

---

## Environment Variables

- `CONSET_PDF_VERBOSE` - Set `1` to enable verbose logging globally
- `CONSET_PDF_PYMUPDF` - Path to PyMuPDF (pymupdf) if not in system PATH

---

## Layout Profile Format

Layout profiles define **ROI (Region of Interest)** for sheet ID/title extraction.

**JSON Schema**:
```json
{
  "profileId": "my-custom-layout",
  "sheetIdRoi": {
    "x": 0.05,      // Normalized 0.0-1.0 (left edge position)
    "y": 0.85,      // Normalized 0.0-1.0 (top edge position)
    "width": 0.2,   // As fraction of page width
    "height": 0.1   // As fraction of page height
  },
  "sheetTitleRoi": {
    "x": 0.05,
    "y": 0.75,
    "width": 0.9,
    "height": 0.08
  }
}
```

**Coordinate System**:
- Origin (0,0) at **top-left** of page
- X increases to the right
- Y increases downward
- All values are **0-1 normalized** (0 = minimum, 1 = maximum)

**Example: Bottom-right corner detection**:
```json
{
  "profileId": "bottom-right",
  "sheetIdRoi": {
    "x": 0.75,
    "y": 0.90,
    "width": 0.23,
    "height": 0.08
  }
}
```

---

## Common Workflows

### Workflow 1: Merge Addenda into Drawings Set

```bash
# Step 1: Dry-run to inspect changes
conset-pdf merge-addenda \
  --original original-drawings.pdf \
  --addenda addendum1.pdf addendum2.pdf \
  --output /dev/null \  # Dummy output for dry-run
  --type drawings \
  --dry-run \
  --json-output merge-plan.json

# Step 2: Review merge-plan.json for accuracy

# Step 3: Execute merge if plan looks good
conset-pdf merge-addenda \
  --original original-drawings.pdf \
  --addenda addendum1.pdf addendum2.pdf \
  --output merged-drawings.pdf \
  --type drawings \
  --report merge-report.json
```

### Workflow 2: Split, Modify, and Reassemble

```bash
# Step 1: Detect current state
conset-pdf detect \
  --pdf drawings.pdf \
  --type drawings \
  --output detection.json

# Step 2: Split by discipline
conset-pdf split-set \
  --input drawings.pdf \
  --output-dir ./split \
  --type drawings \
  --group-by prefix

# Step 3: Modify individual discipline PDFs externally
# (use your PDF editor to update A-sheets.pdf, M-sheets.pdf, etc.)

# Step 4: Reassemble with custom order
conset-pdf assemble-set \
  --input-dir ./split \
  --output reassembled.pdf \
  --type drawings \
  --order-json order.json
```

### Workflow 3: Fix Bookmarks on Merged Document

```bash
# Step 1: Merge addenda (may have invalid bookmarks)
conset-pdf merge-addenda \
  --original drawings.pdf \
  --addenda addendum.pdf \
  --output merged.pdf \
  --type drawings

# Step 2: Fix bookmarks using footer extraction
conset-pdf fix-bookmarks \
  --pdf merged.pdf \
  --output merged-fixed.pdf \
  --type drawings \
  --section-start-strategy footer-first
```

---

## Tips & Tricks

### Detecting the Best Layout Profile

When you have a consistent document format but don't know the ROI coordinates:

1. **Visual inspection**: Open the PDF and estimate the region in normalized coordinates
2. **Trial and error**: Start with `--sheet-id-roi "0.05,0.85,0.2,0.1"` and adjust
3. **Use detection dry-run**: Test with `detect` command and check `confidence` scores

### Validating Merge Results

Always inspect the merge report:
- Check `replaced` count matches expectations
- Review `warnings` for ID confidence issues
- Inspect `appendedUnmatched` for unmatched sheets

### Batch Processing

Process multiple documents in a script:

```bash
for doc in *.pdf; do
  echo "Processing $doc..."
  conset-pdf detect --pdf "$doc" --type drawings --output "${doc%.pdf}-detected.json"
done
```

---

See also:
- [Core API Documentation](./CORE_API.md) - Using the library from code
- [Workflow Engine Guide](./WORKFLOWS.md) - Understanding workflow patterns
- [Codebase Overview](./CODEBASE_OVERVIEW.md) - Architecture overview

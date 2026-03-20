# Output Structure

**Last verified**: 2026-03-01

This document describes the exact output files and folders produced by `conset-pdf` commands.

## Recommended Structure

```
project-root/
├── layouts/                    # Layout profiles (shared across projects)
│   ├── layout-template.json
│   └── project-xyz-layout.json
│
├── conset-output/              # All conset-pdf output files
│   ├── inventories/            # Master inventory index files
│   │   ├── original-inventory.json
│   │   ├── addendum1-inventory.json
│   │   └── addendum2-inventory.json
│   │
│   ├── previews/              # Detect command preview outputs
│   │   ├── original-preview.json
│   │   └── layout-test-preview.json
│   │
│   ├── reports/               # Merge operation reports
│   │   ├── merge-2025-01-15-report.json
│   │   └── merge-add4-report.json
│   │
│   └── merged/                # Final merged PDFs
│       ├── IFC+Add4.pdf
│       └── Final-Set.pdf
│
└── source-pdfs/               # Your source PDFs (optional organization)
    ├── original/
    │   └── IFC Set_FULL_2025-10-01.pdf
    └── addenda/
        └── Add4 Set_FULL_2025-10-20.pdf
```

## File Naming Conventions

### Inventory Files
- Pattern: `{source-filename}-inventory.json`
- Location: `conset-output/inventories/` (or next to source PDF)
- Contains: Detected sheet IDs, titles, confidence scores, parse times

### Preview Files
- Pattern: `{source-filename}-preview-{timestamp}.json` (when using `--preview-dir`) or user-specified path
- Location: `conset-output/previews/` (or user-specified via `--output-preview` or `--preview-dir`)
- Contains: Detection results from `detect` command (see CLI.md for exact JSON structure)

### Report Files
- Pattern: User-specified path (via `--report` option)
- Location: `conset-output/reports/` (recommended) or user-specified
- Contains: Merge operation summary with structure:
  - `kind`: Document type (`"drawings"` or `"specs"`)
  - `originalPath`: Path to original PDF
  - `addendumPaths`: Array of addendum PDF paths
  - `outputPath`: Path to output PDF (if not dry-run)
  - `replaced`: Array of replacement operations
  - `inserted`: Array of insertion operations
  - `appendedUnmatched`: Array of unmatched pages
  - `warnings`: Array of warning strings
  - `notices`: Optional array of informational notices
  - `stats`: Statistics (originalPages, finalPagesPlanned, parseTimeMs, mergeTimeMs)

### Merged PDFs
- Pattern: User-defined (e.g., `IFC+Add4.pdf`, `Final-Set.pdf`)
- Location: `conset-output/merged/` (or user-specified)
- Contains: Final assembled PDF

## Usage Examples

### Using Organized Output Structure

```bash
# Create output directories (one-time setup)
mkdir -p conset-output/{inventories,previews,reports,merged}

# Test detection with preview output
conset-pdf detect \
  --input source-pdfs/original/IFC\ Set_FULL_2025-10-01.pdf \
  --layout layouts/project-layout.json \
  --pages 1,5,10,20 \
  --output-preview conset-output/previews/original-preview.json

# Run merge with organized outputs
conset-pdf merge-addenda \
  --original source-pdfs/original/IFC\ Set_FULL_2025-10-01.pdf \
  --addenda source-pdfs/addenda/Add4\ Set_FULL_2025-10-20.pdf \
  --output conset-output/merged/IFC+Add4.pdf \
  --type drawings \
  --layout layouts/project-layout.json \
  --report conset-output/reports/merge-add4-report.json \
  --verbose
```

### Inventory Files

Inventory files are automatically generated for each input PDF during merge operations. They contain:

- **Source file path**
- **Document type** (drawings/specs)
- **Total pages**
- **Parse statistics** (time, success rate)
- **Per-page inventory**:
  - Sheet ID (raw and normalized)
  - Title
  - Confidence score
  - Detection method/source
  - Warnings

**Example inventory file:**
```json
{
  "sourceFile": "IFC Set_FULL_2025-10-01.pdf",
  "type": "drawings",
  "totalPages": 120,
  "parseTimeMs": 1250,
  "pagesWithIds": 119,
  "pagesWithoutIds": 1,
  "warnings": 2,
  "inventory": [
    {
      "pageIndex": 0,
      "sheetId": "COVER",
      "normalizedId": "COVER",
      "title": "Cover Page",
      "confidence": 1.0,
      "source": "cover-page",
      "context": "Cover page (no title block detected)"
    },
    {
      "pageIndex": 1,
      "sheetId": "M1-01",
      "normalizedId": "M1-01",
      "title": "Floor Plan - First Floor",
      "confidence": 0.95,
      "source": "roi"
    },
    {
      "pageIndex": 4,
      "sheetId": "A-101",
      "normalizedId": "A-101",
      "confidence": 0.75,
      "source": "composite-fallback-legacy-titleblock",
      "warning": "ROI detection failed, using legacy fallback"
    }
  ],
  "performance": {
    "totalParseTimeMs": 1250,
    "averageTimePerPageMs": 10.4,
    "slowestPages": [
      {
        "pageIndex": 15,
        "parseTimeMs": 45
      }
    ]
  }
}
```

**Inventory item fields:**
- `pageIndex`: 0-based page index
- `sheetId`: Detected sheet ID (raw string, or `"COVER"` for cover pages)
- `normalizedId`: Normalized sheet ID
- `title`: Sheet title (if detected)
- `confidence`: Detection confidence (0.0-1.0)
- `source`: Detection method (`"roi"`, `"legacy-titleblock"`, `"composite-fallback-legacy-titleblock"`, `"cover-page"`, etc.)
- `context`: Additional context string (optional)
- `warning`: Warning message (optional, present when detection had issues)

## Benefits of Organized Structure

1. **Separation of concerns**: Outputs don't clutter source directories
2. **Easy cleanup**: Delete `conset-output/` to remove all generated files
3. **Version control**: Can gitignore `conset-output/` while keeping layouts
4. **Multi-project**: Each project can have its own `conset-output/` directory
5. **Master index**: All inventories in one place for analysis

## Git Integration

Recommended `.gitignore` entries:

```
# conset-pdf outputs
conset-output/
*.inventory.json
*-preview.json
*-report.json
```

Keep in version control:
- `layouts/` directory (layout profiles)
- Source PDFs (if desired)
- Configuration files

## Alternative: Next to Source Files

If you prefer outputs next to source files:

```bash
# Inventory files written next to PDFs
source-pdfs/
├── original/
│   ├── IFC Set_FULL_2025-10-01.pdf
│   └── IFC Set_FULL_2025-10-01-inventory.json  # Auto-generated
```

This is the default behavior. Use `--inventory-dir` to organize inventory files into a specific directory.

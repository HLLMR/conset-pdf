# conset-pdf-v3

Core library and CLI for building "latest-and-greatest" construction document sets. A construction-focused PDF workflow tool that provides APIs for merging addenda, splitting sets, detecting sheet IDs, and assembling documents.

## Current Status (2026-03-01)

**✅ Fully Implemented:**
- **V3 PDF Extraction Architecture** - Complete transcript-based extraction system
  - Backend-agnostic transcript extraction (PyMuPDF primary, PDF.js fallback)
  - High-fidelity bbox accuracy (95-99% with PyMuPDF vs 15-25% with PDF.js)
  - Deterministic canonicalization (rotation, coordinates, stable-sort, hashes)
  - Quality scoring and validation gates
  - Candidate generation (headers, footers, headings, tables, columns)
  - Privacy-preserving pattern abstraction (TokenVault, sanitization)
  - ML-assisted profile generation (RulesetCompiler with LLM integration)
  - Enhanced parsers (specs with chrome removal, schedules, submittals)
  - Comprehensive test coverage (determinism, quality, extraction accuracy, bbox validation)
- **Extract Documents (Split) Workflow** - Complete workflow engine implementation
  - Extract and separate documents by sheet ID with deterministic inventory
  - Sheet detection and ID validation
  - Deterministic inventory analysis and corrections support
  - PDF extraction and subset assembly
  - Conflict detection (duplicate IDs, missing IDs)
  - CLI command: `split-set`
- **Fix Bookmarks Workflow** - Complete workflow engine implementation
  - Read, validate, and repair PDF bookmarks
  - Rebuild bookmarks from `BookmarkAnchorTree` (Specs Pipeline) or sheet/section inventory
  - **Footer-First Section Anchoring**: Deterministic section destination resolution using footer text extraction
    - Auto-detects page regions (header/heading/body/footer) using ROI bands
    - Extracts section codes from footer band, maps to first occurrence page
    - Numeric sorting, proper hierarchy, junk title rejection
    - Validation gates with override flag
  - Deterministic sidecar writer (QPDF/pikepdf) for cross-viewer compatibility
  - Bookmark correction support (rename, reorder, delete, retarget, rebuild)
  - CLI command: `fix-bookmarks` with `--section-start-strategy` and `--allow-invalid-destinations` flags
- Merge workflow (Update Documents) - Complete workflow engine implementation
  - **Drawings**: ROI-based detection with layout profiles (or legacy fallback)
  - **Specs**: Text-based section ID detection
- **Standards Module (UCS/CSI)** - Complete implementation
  - **Drawings**: UDS-style discipline identification and sorting
  - **Specs**: CSI MasterFormat classification and sorting
  - Integrated into merge workflow inventory mapping
- **Narrative PDF Processing** - Complete implementation
  - Text extraction and algorithmic parsing
  - **Deterministic validation** against inventory
  - Issue detection with near-match suggestions
  - Suggested corrections (advisory only)
- CLI commands for all operations
- ROI-based sheet detection with layout profiles (drawings)
- Legacy detection fallback (drawings)
- Inventory analysis with corrections support

**⚠️ Known Gaps / Legacy Paths:**
- Split and Fix Bookmarks GUI flows remain placeholder UI in `conset-pdf-gui` (core + CLI are implemented)
- `specs-patch` and `assemble-set` are retained as abandoned legacy paths for reference

**📚 Documentation:**
- Complete API documentation
- Comprehensive CLI usage guide
- Workflow engine architecture documented
- V3 transcript architecture documentation
- ML Ruleset Compiler guide
- Migration guide (V2 → V3)
- Extension guide for adding new workflows

## Repo Scope

This repository contains:
- **Core library** (`@conset-pdf/core`): Publishable npm package with PDF processing APIs
- **CLI tool** (`@conset-pdf/cli`): Command-line interface for all workflows

Both CLI and GUI applications route through the same workflow engine for consistency.

## What Core Provides

**Merge APIs**:
- `mergeAddenda()` - Replace updated sheets from addenda, insert new sheets
- `assembleSet()` - Assemble multiple PDFs into a single set

**Split APIs**:
- `splitSet()` - Split PDF into discipline-specific subsets

**Detection APIs**:
- `DocumentContext` - PDF loading and text extraction (now uses transcript system)
- `RoiSheetLocator` - ROI-based sheet ID detection
- `LegacyTitleblockLocator` - Auto-detected title block detection
- `CompositeLocator` - ROI-first with legacy fallback
- `SpecsSectionLocator` - Specs section ID detection

**Transcript System (V3)**:
- `createTranscriptExtractor()` - Backend-agnostic transcript extraction
- `canonicalizeTranscript()` - Deterministic normalization
- `scoreTranscriptQuality()` - Quality scoring with validation gates
- `generateCandidates()` - Structural candidate detection
- `sanitizeTranscript()` - Privacy-preserving abstraction
- `createAPIRulesetCompiler()` - ML-assisted profile generation
- `extractSchedules()` - Schedule/table extraction
- `parseSubmittal()` - Equipment submittal parsing

**Layout System**:
- Layout profiles with ROI definitions
- Extended profile system (SpecProfile, SheetTemplateProfile, EquipmentSubmittalProfile)
- Profile registry with validation and matching
- Profile loading and validation

**Narrative Processing**:
- `extractNarrativeTextFromPdf()` - Extract text from narrative PDFs
- `parseNarrativeAlgorithmic()` - Parse narrative instructions
- `validateNarrativeAgainstInventory()` - Validate narrative against inventory (advisory only)

**Standards (UCS/CSI)**:
- `normalizeDrawingsDiscipline()` - UDS-style discipline identification
- `normalizeSpecsMasterformat()` - CSI MasterFormat classification
- `compareDrawingsRows()` - Discipline-based sorting comparator
- `compareSpecsRows()` - MasterFormat-based sorting comparator

**Workflow Engine**:
- `createMergeWorkflowRunner()` - Merge workflow with analyze/execute pattern
- `createExtractWorkflowRunner()` - Extract workflow with analyze/execute pattern
- `createBookmarksWorkflowRunner()` - Bookmarks workflow with analyze/execute pattern
- `createWorkflowRunner()` - Generic workflow runner factory
- `InventoryResult` - Standardized inventory analysis result
- `CorrectionOverlay` - User corrections (ignore rows, override IDs, patch operations)
- Workflow types: `merge`, `extract`, `fix-bookmarks` (all fully implemented); `specs-patch`, `assemble` (abandoned)

## Install

```bash
npm install @conset-pdf/core
```

## Build

```bash
npm install
npm run build          # Build all packages
npm run build:core     # Build core only
npm run build:cli      # Build CLI only
```

## Test

```bash
npm test               # Run all tests
npm run test:smoke     # Smoke tests
npm run verify:invariants  # Architecture invariants
```

## Quick-Start Examples

### CLI Usage

**Merge addenda** (most common):
```bash
node packages/cli/dist/cli.js merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf Addendum2.pdf \
  --output Final.pdf \
  --type drawings \
  --layout layout.json
```

**Dry-run inventory analysis** (preview before merging):
```bash
node packages/cli/dist/cli.js merge-addenda \
  --original Original.pdf \
  --addenda Addendum1.pdf \
  --type drawings \
  --dry-run \
  --json-output inventory.json
```

**Note**: All CLI examples assume execution from the repository root. For installed CLI usage, replace `node packages/cli/dist/cli.js` with `conset-pdf` (if installed globally) or `npx @conset-pdf/cli`.

**Extract documents** (split by sheet ID):
```bash
node packages/cli/dist/cli.js split-set \
  --input Full-Set.pdf \
  --output-dir ./extracted \
  --format "D-{id}-{pageCount}p.pdf" \
  --type drawings
```

**Extract dry-run** (analyze only, produces inventory JSON):
```bash
node packages/cli/dist/cli.js split-set \
  --input Full-Set.pdf \
  --type drawings \
  --dry-run \
  --json-output split-inventory.json
```

**Fix bookmarks** (dry-run to analyze existing bookmarks):
```bash
node packages/cli/dist/cli.js fix-bookmarks \
  --input drawing-set.pdf \
  --dry-run \
  --json-output fix-bookmarks-inventory.json
```

**Fix bookmarks** (rebuild from Specs Pipeline output, defaults to specs-v1):
```bash
node packages/cli/dist/cli.js fix-bookmarks \
  --input spec.pdf \
  --output spec-fixed.pdf \
  --bookmark-tree specs-bookmark-tree.json \
  --rebuild \
  --json-output fix-bookmarks-tree.json \
  --report fix-bookmarks-audit.json
```

**Fix bookmarks with detailed subsections**:
```bash
node packages/cli/dist/cli.js fix-bookmarks \
  --input spec.pdf \
  --output spec-fixed.pdf \
  --bookmark-tree specs-bookmark-tree.json \
  --rebuild \
  --bookmark-profile specs-v2-detailed \
  --include-subsections \
  --max-depth 4 \
  --json-output fix-bookmarks-tree.json
```

**Note**: Bookmark profiles control how bookmarks are shaped:
- **raw**: Preserves existing bookmarks (safe default)
- **specs-v1**: Clean SECTION/PART/Article hierarchy (default when `--bookmark-tree` and `--rebuild` provided)
- **specs-v2-detailed**: Like specs-v1 but can include structural subsections when `--include-subsections` is enabled

Bookmarks are written with viewer-compatible destinations:
- Uses `/Fit` view type by default (most compatible)
- Page references are indirect (not inline dictionaries)
- Writes both `/Dest` and `/A GoTo` actions for maximum viewer compatibility
- Post-write verification ensures destinations work in PDF-XChange and other viewers

**Note**: Examples assume execution from the repository root. The `--json-output` flag behavior differs by mode:
- **Dry-run mode**: Writes full inventory JSON (analyze results with rows, issues, summary, meta)
- **Execute mode**: Writes AST JSON (SpecDoc structure only) or bookmark tree JSON (bookmarks workflow)

**Detect sheet IDs** (test layout profile):
```bash
node packages/cli/dist/cli.js detect \
  --input Set.pdf \
  --layout layout.json \
  --pages 1,5,20 \
  --output-preview preview.json
```

**Split set**:
```bash
node packages/cli/dist/cli.js split-set \
  --input Set.pdf \
  --output-dir ./output \
  --type drawings
```

**Assemble set**:
```bash
node packages/cli/dist/cli.js assemble-set \
  --input-dir ./subsets \
  --output Final.pdf \
  --type drawings
```

### API Usage

```typescript
import { mergeAddenda, DocumentContext, RoiSheetLocator } from '@conset-pdf/core';
import { loadLayoutProfile } from '@conset-pdf/core';

// Load layout profile
const layout = await loadLayoutProfile('layout.json');

// Create locator
const locator = new RoiSheetLocator(layout);

// Merge addenda
const report = await mergeAddenda({
  originalPdfPath: 'Original.pdf',
  addendumPdfPaths: ['Addendum1.pdf', 'Addendum2.pdf'],
  outputPdfPath: 'Final.pdf',
  type: 'drawings',
  locator,
  mode: 'replace+insert',
  regenerateBookmarks: true,
});

console.log(`Merged ${report.stats.finalPagesPlanned} pages`);
```

## High-Level Architecture

### Workflow Engine Pattern

All workflows follow the **analyze → applyCorrections → execute** pattern:

1. **Analyze**: Dry-run inventory analysis (no file writes)
   - Detects sheet IDs from all input PDFs
   - Builds inventory of all pages with detected IDs
   - Identifies issues (missing IDs, duplicates, conflicts)
   - Returns `InventoryResult` for review

2. **Apply Corrections**: User edits applied to inventory
   - Ignore rows (visible but excluded from counts)
   - Override IDs (update `normalizedId`, stable `row.id` unchanged)
   - Returns corrected `InventoryResult`

3. **Execute**: Produces output files
   - Applies merge/split/assemble plan
   - Writes output PDFs
   - Returns `ExecuteResult` with file paths and statistics

### Inventory Model

- **Stable `row.id`**: Unique identifier per row (format: `${source}:${pageIndex}:${idPart}`)
  - Never changes when corrections are applied
  - Used as key for corrections overlay
  
- **`normalizedId`**: Detected/overridden sheet ID
  - Updated when user overrides ID
  - Used for matching and planning

- **Corrections keyed by `row.id`**: 
  - `ignoredRowIds`: Array of stable row IDs to ignore
  - `overrides[rowId].fields.normalizedId`: Override detected ID

- **Ignored rows**: Remain visible in inventory but excluded from summary counts

## Project Structure

```
conset-pdf/
├── packages/
│   ├── core/          # Core library (@conset-pdf/core)
│   │   └── src/
│   │       ├── analyze/      # PDF loading, text extraction (uses transcript)
│   │       ├── core/         # Merge/split/assemble logic
│   │       ├── locators/     # Detection strategies
│   │       ├── parser/       # ID parsing/normalization
│   │       ├── layout/       # Layout profile system
│   │       ├── narrative/   # Narrative PDF processing
│   │       ├── transcript/  # V3 transcript system (NEW)
│   │       │   ├── extractors/    # PyMuPDF, PDF.js extractors
│   │       │   ├── profiles/      # Extended profile system
│   │       │   ├── abstraction/   # Privacy-preserving abstraction
│   │       │   ├── ml/            # ML Ruleset Compiler
│   │       │   ├── schedules/     # Schedule extraction
│   │       │   └── sidecar/       # Python extraction scripts
│   │       ├── schedules/    # Schedule extraction (uses transcript)
│   │       ├── submittals/   # Submittal parsing (uses transcript)
│   │       ├── workflows/    # Workflow engine (analyze/execute pattern)
│   │       └── utils/        # Utilities
│   └── cli/           # CLI tool (@conset-pdf/cli)
└── docs/              # Documentation
```

**Monorepo Structure**: This is an npm workspaces monorepo. Root-level commands (`npm install`, `npm run build`, `npm test`) are the authoritative workflows. Individual package scripts exist but are not the primary entry point.

## Workflows Implemented

| Workflow | Status | Description |
|----------|--------|-------------|
| **Update Documents** (merge) | ✅ Implemented | Merge addenda into original set, replace updated sheets, insert new sheets |
| **Extract Documents** (split) | ✅ Implemented | Extract and separate documents by sheet ID with deterministic inventory |
| **Fix Bookmarks** | ✅ Implemented | Read, validate, repair, and write PDF bookmarks. Rebuild from BookmarkAnchorTree or inventory |
| Specs Patch | ❌ Abandoned | Extraction and patching functionality integrated into Extract workflow |
| Assemble Set | ❌ Abandoned | Superseded by modular composition in Extract workflow |

## Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - Module overview, invariants, data flow, extension guide
- **[Transcript Architecture](docs/TRANSCRIPT_ARCHITECTURE.md)** - V3 transcript system overview
- **[Migration Guide](docs/MIGRATION_V3.md)** - Migrating from PDF.js to transcript system
- **[ML Ruleset Compiler](docs/ML_RULESET_COMPILER.md)** - ML-assisted profile generation
- **[Public API](docs/PUBLIC_API.md)** - Stable API contracts
- **[CLI](docs/CLI.md)** - All CLI commands, arguments, options, examples
- **[Workflows](docs/WORKFLOWS.md)** - Workflow details, inputs, outputs, implementation status
- **[Standards](docs/STANDARDS.md)** - UCS/CSI standards module (discipline & MasterFormat)
- **[Quick Start](docs/QUICK_START.md)** - Happy-path usage

## Quick Onboarding for New Developers

1. **Start Here**: Read [QUICK_START.md](docs/QUICK_START.md) for a hands-on walkthrough
2. **Understand Architecture**: Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) for module structure and invariants
3. **API Reference**: See [PUBLIC_API.md](docs/PUBLIC_API.md) for stable API contracts
4. **CLI Usage**: See [CLI.md](docs/CLI.md) for all CLI commands and examples
5. **Workflows**: See [WORKFLOWS.md](docs/WORKFLOWS.md) for workflow details and implementation status

**Key Concepts:**
- **Workflow Engine**: All operations follow analyze → applyCorrections → execute pattern
- **Inventory Model**: Stable `row.id` for corrections, `normalizedId` for matching
- **Single-Load Pipeline**: Only `DocumentContext` loads PDFs (architecture invariant)
- **ROI-First Detection**: ROI-based detection with legacy fallback

## Related Projects

- **[conset-pdf-v3-gui](https://github.com/HLLMR/conset-pdf-v3-gui)** - Electron GUI (uses `@conset-pdf/core`)

## License

MIT

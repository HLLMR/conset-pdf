# Architecture

**Last verified**: 2026-03-01

## Module Overview

```
packages/core/src/
├── analyze/           # PDF loading, text extraction, caching
│   ├── documentContext.ts  # Document-level state and caching (uses transcript)
│   ├── pageContext.ts      # Per-page caching (uses transcript)
│   └── readingOrder.ts     # Visual reading-order assembly helpers
├── core/             # Business logic (merge/split/assemble)
├── locators/         # Detection strategies (pluggable)
├── parser/           # ID parsing/normalization (pure)
├── layout/           # Layout profile system
├── standards/        # UCS/CSI standards (discipline & MasterFormat)
├── narrative/        # Narrative PDF processing (advisory analysis)
│   ├── text-extract.ts     # Extract text from narrative PDFs
│   ├── parse-algorithmic.ts # Parse narrative into instruction sets
│   ├── normalize.ts        # Normalize sheet/spec IDs from narrative
│   └── types.ts            # Narrative types and interfaces
├── transcript/       # V3 transcript system (NEW)
│   ├── types.ts            # Core transcript types (LayoutTranscript, LayoutSpan, etc.)
│   ├── interfaces.ts       # TranscriptExtractor interface
│   ├── factory.ts          # Extractor factory with fallback chain
│   ├── canonicalize.ts     # Deterministic canonicalization
│   ├── quality.ts          # Quality scoring and validation gates
│   ├── candidates.ts       # Structural candidate generation
│   ├── extractors/         # Extraction backends
│   │   ├── pymupdfExtractor.ts  # PyMuPDF extractor (primary)
│   │   └── pdfjsExtractor.ts   # PDF.js extractor (fallback)
│   ├── sidecar/            # Python sidecar scripts
│   │   ├── extract-transcript.py  # PyMuPDF extraction script
│   │   └── requirements.txt
│   ├── profiles/           # Extended profile system
│   │   ├── types.ts        # SpecProfile, SheetTemplateProfile, EquipmentSubmittalProfile
│   │   ├── registry.ts     # Profile registry with versioning
│   │   └── validation.ts  # Profile validation gates
│   ├── abstraction/        # Privacy-preserving pattern abstraction
│   │   ├── abstractTranscript.ts  # Abstract transcript types
│   │   ├── tokenVault.ts   # Token mapping and tokenization
│   │   └── sanitize.ts     # Sanitization with privacy modes
│   ├── ml/                 # ML Ruleset Compiler
│   │   ├── rulesetCompiler.ts  # Compiler interface
│   │   ├── apiCompiler.ts  # API-based compiler implementation
│   │   └── types.ts        # Profile proposal types
│   └── schedules/          # Schedule extraction
│       ├── extractor.ts     # Schedule extraction engine
│       ├── tableBuilder.ts  # Geometry-first table builder
│       └── types.ts         # Schedule table types
├── schedules/        # Schedule extraction (uses transcript)
├── submittals/       # Submittal parsing (uses transcript)
│   ├── extract/
│   │   └── submittalParser.ts
│   └── types.ts
├── specs/            # Spec processing (enhanced with transcript)
│   └── extract/
│       ├── chromeRemoval.ts      # Chrome removal using candidates
│       ├── paragraphNormalizer.ts # Paragraph normalization
│       └── tableDetector.ts       # Basic table detection
├── workflows/        # Workflow engine (analyze/execute pattern)
│   ├── types.ts      # Core workflow types (InventoryResult, CorrectionOverlay, etc.)
│   ├── engine.ts     # WorkflowRunner factory
│   ├── merge/         # Merge workflow implementation
│   ├── specs-patch/   # Specs patch workflow implementation
│   ├── bookmarks/     # Bookmarks workflow implementation
│   └── mappers/       # Mapping from legacy structures to workflow types
├── bookmarks/        # Bookmarks pipeline module
│   ├── types.ts      # BookmarkNode, BookmarkDestination, BookmarkTree, BookmarkAnchorTree types
│   ├── reader.ts     # Read bookmarks from PDF via DocumentContext
│   ├── validator.ts  # Validate bookmark destinations
│   ├── treeBuilder.ts # Build bookmark tree from BookmarkAnchorTree or inventory
│   ├── corrections.ts # Apply correction overlays (rename, reorder, delete, retarget, rebuild)
│   ├── headingResolver.ts # Layout-aware heading-based section resolution
│   ├── pikepdfBookmarkWriter.ts # Sidecar-based bookmark writer (primary)
│   ├── profiles/     # Bookmark style profiles (raw, specs-v1, specs-v2-detailed)
│   ├── sidecar/      # Python sidecar
│   │   ├── bookmark-writer.py
│   │   └── requirements.txt
│   └── tests/        # Bookmark tests
└── utils/            # Utilities (legacy bookmark helpers, PDF helpers, sorting)
    ├── bookmarks.ts  # Legacy bookmark generation (deprecated for new workflows)
    ├── bookmarkWriter.ts # Bookmark writing interface
    ├── pdfLibBookmarkWriter.ts # pdf-lib implementation (development/testing only)
    ├── pdf.ts
    ├── fs.ts
    └── sort.ts
```

### Analyze (`analyze/`)

**Purpose**: Single source of truth for PDF operations

- **`DocumentContext`**: Document-level state
  - Loads PDF once via `initialize()`
  - **Uses transcript system** for text extraction (PyMuPDF primary, PDF.js fallback)
  - Caches transcript and PDF bytes
  - Creates/manages `PageContext` instances
  - Coordinates text extraction (demand-driven)
  - PDF.js document still loaded temporarily for bookmarks (until migrated)

- **`PageContext`**: Per-page caching
  - Page dimensions, rotation
  - Text items (extracted from transcript, cached)
  - Plain text (derived)
  - ROI-filtered views (derived)

- **`readingOrder.ts`**: Visual reading-order assembly
  - Handles text runs out of visual order from PDF.js `getTextContent()`
  - Reconstructs wrapped titles correctly
  - Used by `RoiSheetLocator` for text assembly

**Rule**: Only `analyze/` may load PDFs or create PDF.js documents. `DocumentContext` now uses transcript system internally.

### Core (`core/`)

**Purpose**: Business logic for operations

- **`mergeAddenda()`**: Merge addenda into original (legacy direct API)
- **`splitSet()`**: Split PDF into subsets
- **`assembleSet()`**: Assemble multiple PDFs
- **`planner.ts`**: Plans merge operations (supports `includeInventory: true` for workflow engine)
- **`applyPlan.ts`**: Applies merge plan
- **`report.ts`**: Generates reports

### Workflows (`workflows/`)

**Purpose**: Reusable workflow engine pattern (analyze/execute)

- **`types.ts`**: Core workflow types (`InventoryResult`, `CorrectionOverlay`, `ExecuteResult`, etc.)
- **`engine.ts`**: `WorkflowRunner` factory and `WorkflowImpl` interface
- **`merge/mergeWorkflow.ts`**: Merge workflow implementation
  - `analyze()`: Runs `planMerge(includeInventory: true)`, maps to `InventoryResult`
  - `applyCorrections()`: Re-analyzes and applies corrections overlay (ignored rows, ID overrides)
  - `execute()`: Runs `mergeAddenda()`, returns `ExecuteResult`
- **`specs-patch/specsPatchWorkflow.ts`**: Specs patch workflow implementation
  - `analyze()`: Extracts spec PDF to structured AST, detects sections/anchors/list items, builds hierarchy
  - `applyCorrections()`: Re-analyzes and applies patch operations from `CorrectionOverlay`
  - `execute()`: Renders corrected AST to PDF via HTML/CSS → Playwright
- **`bookmarks/bookmarksWorkflow.ts`**: Bookmarks workflow implementation
  - `analyze()`: Reads existing bookmarks, validates destinations, builds tree from `BookmarkAnchorTree` or inventory
  - `applyCorrections()`: Re-analyzes and applies bookmark corrections (rename, reorder, delete, retarget, rebuild)
  - `execute()`: Writes bookmarks to PDF via Python sidecar (pikepdf/QPDF), validates post-write
- **`mappers/merge.ts`**: Maps `ParseResult.inventory` → `InventoryRowBase[]`, `MergePlan` → `InventoryResult.summary`

**Rule**: Workflow engine is canonical for all operations. CLI and GUI route through workflow runners.

### Locators (`locators/`)

**Purpose**: Pluggable detection strategies

- **`SheetLocator`**: Interface contract
- **`RoiSheetLocator`**: ROI-based detection (drawings, requires layout profile)
- **`LegacyTitleblockLocator`**: Auto-detected title block (drawings fallback)
- **`SpecsSectionLocator`**: Specs section ID detection (text-based, no layout profile)
- **`CompositeLocator`**: ROI-first with fallback (drawings only)

**Rule**: Locators are pure (no IO). Consume cached `PageContext` only.

### Parser (`parser/`)

**Purpose**: Pure ID parsing/normalization

- **`normalize.ts`**: ID normalization (canonical dash format for drawings, space-separated for specs)
- **`drawingsSheetId.ts`**: Drawing sheet ID patterns (e.g., "A-101", "M1-01")
- **`specsSectionId.ts`**: Spec section ID patterns (e.g., "23 02 00", "00 31 21")

**Rule**: Parsers are pure functions (no IO).

### Layout (`layout/`)

**Purpose**: User-defined ROI regions

- **`types.ts`**: Layout profile schema, ROI types
- **`load.ts`**: Load profiles from JSON or create inline

### Standards (`standards/`)

**Purpose**: UCS/CSI standards for drawings discipline identification and specs MasterFormat classification

- **`normalizeDrawingsDiscipline.ts`**: UDS-style discipline normalization
  - Identifies discipline designators (G, C, L, A, I, S, M, P, E, F, T)
  - Handles multi-letter aliases (FP, DDC, ATC, SEC, AV, IT, etc.)
  - Heuristic-based disambiguation (Controls vs Civil)
  - Returns `DrawingsDisciplineMeta` with canonical codes, order, confidence

- **`normalizeSpecsMasterformat.ts`**: CSI MasterFormat normalization
  - Extracts section IDs in format "DD SS SS" (e.g., "23 09 00")
  - Identifies divisions (00-49) with titles from dataset
  - Returns `SpecsMasterformatMeta` with division, order, confidence

- **`compare.ts`**: Row comparators for standards-based sorting
  - `compareDrawingsRows()`: Sorts by discipline order, then normalizedId
  - `compareSpecsRows()`: Sorts by division order, then section tuple

- **`datasets/`**: Standards datasets
  - `drawingsDesignators.ts`: UDS designators and aliases
  - `drawingsOrderHeuristic.ts`: Discipline ordering table
  - `masterformatDivisions.ts`: CSI MasterFormat 2018 divisions (00-49)

- **`types.ts`**: Standards types and interfaces
  - `DrawingsDisciplineMeta`: Complete discipline metadata
  - `SpecsMasterformatMeta`: Complete MasterFormat metadata

**Integration**: Applied in `workflows/mappers/merge.ts` during inventory mapping. Adds optional `discipline` field to drawings rows and optional `specs` field to specs rows. Non-breaking (optional fields only).

**Rule**: Standards module is pure (no IO, no side effects). All functions are deterministic.

### Specs (`specs/`)

**Purpose**: Specs PDF extraction, patching, and rendering

- **`ast/types.ts`**: SpecDoc AST types (`SpecDoc`, `SpecSection`, `SpecNode`, `BookmarkAnchorTree`)
- **`extract/`**: Extraction components
  - `sectionDetector.ts`: Section header detection using `SpecsSectionLocator`
  - `textExtractor.ts`: Text extraction and node creation
  - `anchorDetector.ts`: Hierarchical anchor detection
  - `listDetector.ts`: List item detection and numbering validation
  - `hierarchyBuilder.ts`: Hierarchy building from indentation
  - `bookmarkTreeGenerator.ts`: Generate `BookmarkAnchorTree` from AST
- **`patch/`**: Patch operations
  - `types.ts`: Patch operation types (`SpecPatch`, `SpecPatchOperation`, etc.)
  - `validator.ts`: Patch validation (target exists, anchors non-null, disambiguation)
  - `apply.ts`: Patch application (insert, delete, move, renumber, replace)
- **`render/`**: PDF rendering
  - `htmlGenerator.ts`: AST → HTML generator
  - `pdfRenderer.ts`: HTML → PDF renderer (Playwright)
  - `templates/specs.css`: CSS template for spec formatting

**Rule**: Specs pipeline treats specs as structured documents with hierarchical anchors as primary navigation mechanism.

### Narrative (`narrative/`)

**Purpose**: Extract, parse, and validate instructions from narrative PDFs (advisory analysis)

- **`text-extract.ts`**: Extract page-aware text from narrative PDFs using `DocumentContext`
  - Uses `DocumentContext` to comply with architecture constraints
  - Generates file hash for change detection
  - Returns `NarrativeTextDocument` with page-aware text

- **`parse-algorithmic.ts`**: Deterministic parsing of narrative text into instruction sets
  - No LLM usage - pure algorithmic pattern matching
  - Extracts drawing and spec section revisions
  - Returns `NarrativeInstructionSet` with instructions and parse issues

- **`normalize.ts`**: Normalize sheet IDs and spec section IDs from narrative text
  - `normalizeSheetId()`: Normalize drawing sheet IDs from narrative
  - `normalizeSpecSectionId()`: Normalize spec section IDs from narrative

- **`validate.ts`**: Deterministic validation of narrative instructions against inventory
  - Compares narrative claims against detected inventory
  - Identifies discrepancies (missing sheets, typos, ambiguous matches)
  - Uses Levenshtein similarity for near-match detection
  - Generates `NarrativeIssue[]` with issue codes and suggested corrections
  - Never modifies inventory or corrections (advisory only)

- **`types.ts`**: Narrative types and interfaces
  - `NarrativeTextDocument`: Extracted text with page awareness
  - `NarrativeInstructionSet`: Parsed instructions (drawings, specs, issues)
  - `NarrativeValidationReport`: Validation results with issues and suggestions
  - `NarrativeIssue`: Individual validation issue with codes and metadata
  - `DrawingInstruction`, `SpecInstruction`: Individual instruction types

**Status**: ✅ Complete - narrative instructions are extracted, parsed, and validated against inventory. Validation report included in `InventoryResult.narrativeValidation` during merge workflow `analyze()` phase. All validation is advisory only and does not modify detection results.

**Rule**: Narrative processing uses `DocumentContext` for PDF operations (complies with single-load pipeline).

### Transcript (`transcript/`) (V3)

**Purpose**: Backend-agnostic, transcript-first PDF extraction architecture

- **`types.ts`**: Core transcript types
  - `LayoutTranscript`: Complete transcript with pages, spans, metadata
  - `LayoutPage`: Single page with spans, dimensions, rotation
  - `LayoutSpan`: Text span with bbox, font, style information
  - `QualityMetrics`: Quality scoring metrics

- **`interfaces.ts`**: Extraction backend interface
  - `TranscriptExtractor`: Interface for any extraction backend
  - `ExtractOptions`: Extraction options (pages, includeLines, etc.)
  - `EngineInfo`: Engine capability information

- **`factory.ts`**: Extractor factory
  - `createTranscriptExtractor()`: Creates extractor with fallback chain (PyMuPDF → PDF.js)
  - Automatic canonicalization of all transcripts
  - `isPyMuPDFAvailable()`, `isPDFjsAvailable()`: Runtime availability checks

- **`extractors/pymupdfExtractor.ts`**: PyMuPDF extractor (primary)
  - Uses Python sidecar (`extract-transcript.py`) for extraction
  - Dict/rawdict-first approach for high-fidelity span extraction
  - 95-99% bbox accuracy
  - Requires Python 3.8+ and PyMuPDF

- **`extractors/pdfjsExtractor.ts`**: PDF.js extractor (fallback)
  - Direct PDF.js integration for environments without Python
  - Lower bbox accuracy (15-25%) but always available
  - Used when PyMuPDF unavailable

- **`canonicalize.ts`**: Deterministic canonicalization
  - Normalizes rotation to 0 (transforms bboxes)
  - Normalizes coordinate origins (top-left visual space)
  - Stable-sorts spans (deterministic order)
  - Computes deterministic hashes (`contentHash`, `spanHash` excluding `extractionDate`)

- **`quality.ts`**: Quality scoring and validation
  - Per-page and aggregate quality metrics
  - Quality gates (min char count, max replacement ratio, min ordering sanity, min confidence)
  - Quality report generation

- **`candidates.ts`**: Structural candidate generation
  - Header/footer band detection (Y clustering + repetition)
  - Font-size clustering
  - Heading candidate detection (regex-based)
  - Column hints (X clustering for tables)
  - Table candidate detection (line density + grid patterns)

- **`profiles/`**: Extended profile system
  - `types.ts`: `SpecProfile`, `SheetTemplateProfile`, `EquipmentSubmittalProfile`
  - `registry.ts`: Profile registry with versioning, validation, matching
  - `validation.ts`: Profile validation gates

- **`abstraction/`**: Privacy-preserving pattern abstraction
  - `abstractTranscript.ts`: Abstract transcript types (tokenized content)
  - `tokenVault.ts`: Token mapping and tokenization
  - `sanitize.ts`: Sanitization with privacy modes (STRICT_STRUCTURE_ONLY, WHITELIST_ANCHORS, FULL_TEXT_OPT_IN)

- **`ml/`**: ML Ruleset Compiler
  - `rulesetCompiler.ts`: Compiler interface
  - `apiCompiler.ts`: API-based compiler with LLM integration
  - Compile-validate loop with automatic re-prompting
  - Profile parsing and validation

- **`schedules/`**: Schedule extraction
  - `extractor.ts`: Schedule extraction engine
  - `tableBuilder.ts`: Geometry-first table builder
  - `types.ts`: Schedule table types

- **`sidecar/extract-transcript.py`**: Python extraction script
  - PyMuPDF-based extraction using dict/rawdict-first approach
  - Extracts spans, bboxes, fonts, lines
  - Quality metrics calculation

**Rule**: Transcript system is backend-agnostic. All transcripts are automatically canonicalized. `DocumentContext` uses transcript system internally while maintaining backward-compatible API.

### Bookmarks (`bookmarks/`)

**Purpose**: Bookmarks pipeline module for reading, validating, and writing PDF bookmarks

- **`types.ts`**: Bookmark data models (`BookmarkNode`, `BookmarkDestination`, `BookmarkTree`, `BookmarkAnchorTree`)
- **`reader.ts`**: Read bookmarks from PDF via `DocumentContext`
- **`validator.ts`**: Validate bookmark destinations
- **`treeBuilder.ts`**: Build bookmark tree from `BookmarkAnchorTree` or inventory
- **`corrections.ts`**: Apply correction overlays (rename, reorder, delete, retarget, rebuild)
- **`pikepdfBookmarkWriter.ts`**: Sidecar-based bookmark writer (primary implementation)
- **`headingResolver.ts`**: Layout-aware heading-based section resolution
- **`profiles/`**: Bookmark style profiles
  - `types.ts`: Bookmark profile types (`BookmarkProfileId`, `BookmarkStyleOptions`)
  - `raw.ts`: Raw profile (preserves existing bookmarks)
  - `specsV1.ts`: Specs v1 profile (SECTION/PART/Article hierarchy)
  - `specsV2Detailed.ts`: Specs v2 detailed profile (includes subsections)
- **`sidecar/bookmark-writer.py`**: Python script for bookmark writing via pikepdf/QPDF
- **`settings.ts`**: Bookmark writer settings and configuration

**Rule**: Bookmarks pipeline uses sidecar-first strategy with pikepdf/QPDF for cross-viewer reliability. Supports `BookmarkAnchorTree` from Specs Pipeline and inventory-based fallback.

### Utils (`utils/`)

**Purpose**: Utilities

- **`bookmarks.ts`**: Bookmark generation (legacy, used by merge workflow)
- **`bookmarkWriter.ts`**: Bookmark writing interface
- **`pdfLibBookmarkWriter.ts`**: pdf-lib implementation of bookmark writer (development/testing only, used by merge workflow)
- **`pdf.ts`**: PDF helpers (legacy fallback)
- **`fs.ts`**: File system helpers (`fileExists`, `writeJson`)
- **`sort.ts`**: Natural sort comparator for construction document IDs
  - Handles mixed alphanumeric strings like "M1-01A", "E2.101", "23 09 00"

#### Bookmark Writing Strategy

**Current State (Legacy)**:
- `pdfLibBookmarkWriter.ts` is used by merge workflow (`applyPlan.ts`) for bookmark writing
- pdf-lib bookmark support is limited and may have cross-viewer compatibility issues

**Bookmarks Pipeline (Fix Bookmarks Workflow)**:
- Uses **sidecar-first strategy** with pikepdf/QPDF for cross-viewer reliability
- `PikepdfBookmarkWriter` is the primary bookmark writer implementation
- Python sidecar (`bookmark-writer.py`) handles bookmark writing via pikepdf
- Sidecar approach ensures bookmarks work consistently across PDF viewers (PDF-XChange, Bluebeam, Foxit, browsers, Adobe)
- Supports `BookmarkAnchorTree` from Specs Pipeline and inventory-based fallback
- Legacy pdf-lib writer will remain for backward compatibility with merge workflow, but new bookmark operations will prefer the sidecar writer
  - Tokenizes strings into alternating alpha and numeric segments

## Intentional `any` Policy

**pdf.js/pdf-lib surfaces** use `any` intentionally:

- **pdf.js**: Types are incomplete/outdated
- **pdf-lib**: Types don't match runtime behavior
- **Strategy**: Use `any` at boundaries, type internally

**Example**:
```typescript
// Boundary: pdf.js types are incomplete
const page: any = await pdfDoc.getPage(pageIndex);

// Internal: Type what we extract
const viewport = page.getViewport({ scale: 1.0 });
const width: number = viewport.width;
```

## Architecture Invariants

### 1. Single-Load PDF Pipeline

Only `analyze/` may load PDFs. All operations go through `DocumentContext`.

### 2. PageContext Caching

Expensive per-page operations cached once per page.

### 3. Locator Seam

Planner uses `locator.locate()`, not direct detection.

### 4. ROI-First Detection

ROI-based detection is default, with explicit fallback tracking.

## Data Flow

### Merge Operation

```
1. Create locator (ROI or legacy)
2. For each PDF:
   a. Create DocumentContext
   b. Initialize (load once)
   c. For each page:
      - Get PageContext (cached)
      - Extract text if needed (demand-driven)
      - Run locator.locate(pageContext)
3. Build ID maps
4. Plan merge
5. Apply plan
6. Generate report
```

### Detection Flow

```
PageContext (cached text items)
  → Locator.locate()
    → Filter by ROI (if ROI locator)
    → Apply regex pattern
    → Calculate confidence
    → Return SheetLocationResult
```

## Performance

- **Text extraction**: First page ~100ms, subsequent ~2ms (cached)
- **ID detection**: ROI ~5ms, Legacy ~50ms
- **Merge planning**: ~10ms (in-memory)
- **PDF assembly**: ~5ms/page

## Workflow Engine API

### WorkflowImpl Interface

All workflows implement the `WorkflowImpl<IAnalyze, ICorrections, IExecute>` interface:

```typescript
interface WorkflowImpl<IAnalyze, ICorrections, IExecute> {
  /**
   * Analyze input and produce inventory result
   * Must NOT write output files - this is a dry-run operation
   */
  analyze(input: IAnalyze): Promise<InventoryResult>;

  /**
   * Apply corrections overlay to inventory
   * Returns modified inventory with corrections applied
   */
  applyCorrections(
    input: ICorrections,
    inventory: InventoryResult,
    corrections: CorrectionOverlay
  ): Promise<InventoryResult>;

  /**
   * Execute the workflow
   * Produces output files and returns execution result
   */
  execute(input: IExecute): Promise<ExecuteResult>;
}
```

### WorkflowRunner Factory

```typescript
// Create a workflow runner from an implementation
const runner = createWorkflowRunner('merge', mergeWorkflowImpl);

// Or use the merge-specific factory
const mergeRunner = createMergeWorkflowRunner();
```

### WorkflowRunner Interface

```typescript
interface WorkflowRunner<IAnalyze, ICorrections, IExecute> {
  analyze: (input: IAnalyze) => Promise<InventoryResult>;
  applyCorrections: (
    input: ICorrections,
    inventory: InventoryResult,
    corrections: CorrectionOverlay
  ) => Promise<InventoryResult>;
  execute: (input: IExecute) => Promise<ExecuteResult>;
}
```

## Inventory Schema

### InventoryResult

```typescript
interface InventoryResult {
  /** Workflow identifier ('merge', 'split', 'assemble', 'bookmark') */
  workflowId: WorkflowId;
  /** Optional lane identifier (for multi-lane workflows) */
  laneId?: string;
  /** Inventory rows (one per page/sheet) */
  rows: InventoryRowBase[];
  /** Issues detected during analysis */
  issues: Issue[];
  /** Conflicts detected (e.g., narrative vs detection) */
  conflicts: Conflict[];
  /** Summary statistics */
  summary: {
    totalRows: number;
    rowsWithIds: number;
    rowsWithoutIds: number;
    rowsOk: number;
    rowsWarning: number;
    rowsError: number;
    rowsConflict: number;
    issuesCount: number;
    conflictsCount: number;
    // Workflow-specific fields (e.g., replaced, inserted, unmatched for merge)
    [key: string]: number | string | undefined;
  };
  /** Optional metadata */
  meta?: Record<string, unknown>;
  /** Optional narrative instruction set (advisory only, read-only) */
  narrative?: NarrativeInstructionSet;
}
```

### InventoryRowBase

```typescript
interface InventoryRowBase {
  /** Unique row identifier (stable, never changes with corrections) */
  id: string;
  /** Optional lane identifier for multi-lane workflows */
  laneId?: string;
  /** Source file or origin */
  source?: string;
  /** Page number (1-based) or page index (0-based) */
  page?: number;
  /** Status of this row */
  status: RowStatus; // 'ok' | 'warning' | 'error' | 'conflict'
  /** Confidence level (0.0 to 1.0) */
  confidence: Confidence;
  /** Action to be taken (e.g., 'replace', 'insert', 'skip') */
  action?: string;
  /** Additional notes */
  notes?: string;
  /** Tags for categorization */
  tags?: string[];
  // Workflow-specific extensions (e.g., normalizedId for merge)
}
```

**Invariants**:
- `row.id` MUST be stable across re-analysis within a workflow (never changes when corrections applied)
- `row.id` format varies by workflow type:
  - **Page-based workflows** (e.g., merge): `${source}:${pageIndex}:${idPart}` where `pageIndex` is stable
  - **Node/anchor-based workflows** (e.g., specs-patch): `${source}:${anchorOrPath}` or `${source}:${idPart}` where `idPart` is stable and independent of page numbers
- `row.normalizedId` (merge-specific): detected/overridden sheet ID, updated by corrections
- Rows are keyed by `row.id` in corrections overlay

**Examples**:
- Merge workflow: `"original.pdf:5:sheet-A-101"` (page-based, page 5)
- Specs-patch workflow: `"specs.pdf:section-23-09-00-5-node-0"` or `"specs.pdf:anchor-2.4-T.5.b.1"` (anchor/path-based, page may change after regeneration)

### Issue

```typescript
interface Issue {
  /** Unique issue identifier */
  id: string;
  /** Severity level */
  severity: Severity; // 'error' | 'warning' | 'info'
  /** Issue code (e.g., 'NO_ID', 'LOW_CONFIDENCE', 'DUPLICATE') */
  code: string;
  /** Human-readable message */
  message: string;
  /** Row IDs affected by this issue */
  rowIds: string[];
  /** Optional lane identifier */
  laneId?: string;
  /** Additional details */
  details?: Record<string, unknown>;
}
```

### Conflict

```typescript
interface Conflict {
  /** Unique conflict identifier */
  id: string;
  /** Conflict code */
  code: string;
  /** Human-readable message */
  message: string;
  /** Row IDs involved in conflict */
  rowIds: string[];
  /** Optional lane identifier */
  laneId?: string;
  /** Narrative instruction (if applicable) */
  narrative?: unknown;
  /** Detected value (if applicable) */
  detected?: unknown;
  /** Suggested resolutions */
  suggestions?: unknown[];
}
```

## Corrections Model

### CorrectionOverlay

```typescript
interface CorrectionOverlay {
  /** Optional lane identifier */
  laneId?: string;
  /** Row IDs to ignore (visible but excluded from counts) */
  ignoredRowIds?: string[];
  /** Page numbers to ignore (alternative to ignoredRowIds) */
  ignoredPages?: number[];
  /** Overrides per row ID (keyed by stable row.id) */
  overrides: {
    [rowId: string]: {
      /** Action override (not implemented in Phase 1) */
      action?: string;
      /** Field overrides */
      fields?: Record<string, unknown>;
      /** Conflict resolution choice (not implemented in Phase 1) */
      resolvedConflictChoice?: 'narrative' | 'detected' | 'manual';
    };
  };
  /** Notes */
  notes?: string;
}
```

### Correction Application Rules

1. **Ignored rows**:
   - Rows remain visible in inventory (not filtered out)
   - Status set to `'ok'` and tag `'ignored'` added
   - Excluded from summary counts (`rowsOk`, `rowsWithIds`, etc.)
   - Issues/conflicts affecting only ignored rows are filtered out

2. **ID overrides**:
   - Keyed by stable `row.id` (not `normalizedId`)
   - Updates `row.normalizedId` field (stable `row.id` unchanged)
   - Used for matching and planning in execute phase

3. **Re-analysis**:
   - `applyCorrections()` re-runs `analyze()` first to get fresh state
   - Then applies corrections overlay to the fresh inventory
   - Ensures corrections are applied to current analysis results

## Logging/Error Conventions

### Logging Levels

- **Verbose mode** (`verbose: true`): Detailed progress, timing, warnings
- **Normal mode**: Errors and summary only

### Error Handling

- **File I/O errors**: Exit code 4 (file not found, permission denied)
- **Validation errors**: Exit code 2 (invalid arguments, mode, etc.)
- **Strict mode violations**: Exit code 3 (unmatched pages in strict mode)
- **Success**: Exit code 0

### Error Messages

- Construction-first language: "Sheet ID", "Addendum", "Spec Section"
- Include context: file paths, page numbers, detected values
- Actionable: suggest fixes when possible

## Extension Guide: Adding a New Workflow

To add a new workflow without breaking invariants:

### 1. Define Input Types

```typescript
// workflows/myworkflow/types.ts
export interface MyWorkflowAnalyzeInput {
  // Workflow-specific input fields
}

export interface MyWorkflowExecuteInput {
  // Workflow-specific input fields
}
```

### 2. Implement WorkflowImpl

```typescript
// workflows/myworkflow/myWorkflow.ts
import type { WorkflowImpl, InventoryResult, ExecuteResult, CorrectionOverlay } from '../types.js';
import type { MyWorkflowAnalyzeInput, MyWorkflowExecuteInput } from './types.js';

export const myWorkflowImpl: WorkflowImpl<
  MyWorkflowAnalyzeInput,
  MyWorkflowAnalyzeInput, // Usually same as analyze
  MyWorkflowExecuteInput
> = {
  async analyze(input: MyWorkflowAnalyzeInput): Promise<InventoryResult> {
    // 1. Load PDFs via DocumentContext (single load per PDF)
    // 2. Detect IDs using locators (pure, no IO)
    // 3. Build inventory rows with stable row.id
    // 4. Identify issues and conflicts
    // 5. Return InventoryResult (no file writes)
  },

  async applyCorrections(
    input: MyWorkflowAnalyzeInput,
    _inventory: InventoryResult,
    corrections: CorrectionOverlay
  ): Promise<InventoryResult> {
    // 1. Re-run analyze() for fresh state
    // 2. Apply ignoredRowIds (mark as ignored, exclude from counts)
    // 3. Apply overrides (update fields, preserve stable row.id)
    // 4. Filter issues/conflicts affecting only ignored rows
    // 5. Recalculate summary
    // 6. Return corrected InventoryResult
  },

  async execute(input: MyWorkflowExecuteInput): Promise<ExecuteResult> {
    // 1. Use corrected inventory if provided
    // 2. Execute workflow (write output files)
    // 3. Return ExecuteResult with output paths and statistics
  },
};
```

### 3. Create Factory Function

```typescript
// workflows/myworkflow/index.ts
import { createWorkflowRunner } from '../engine.js';
import { myWorkflowImpl } from './myWorkflow.js';

export function createMyWorkflowRunner() {
  return createWorkflowRunner('myworkflow', myWorkflowImpl);
}
```

### 4. Export from Core

```typescript
// packages/core/src/index.ts
export { createMyWorkflowRunner } from './workflows/myworkflow/index.js';
export type * from './workflows/myworkflow/types.js';
```

### 5. Add CLI Command (Optional)

```typescript
// packages/cli/src/commands/myworkflow.ts
import { createMyWorkflowRunner } from '@conset-pdf/core';

export function myWorkflowCommand(program: Command) {
  program
    .command('my-workflow')
    .description('Description of workflow')
    // ... options
    .action(async (options) => {
      const runner = createMyWorkflowRunner();
      if (options.dryRun) {
        const inventory = await runner.analyze({ /* ... */ });
        // Output inventory JSON
      } else {
        const result = await runner.execute({ /* ... */ });
        // Print summary
      }
    });
}
```

### Invariants to Maintain

1. **Single-load PDF pipeline**: Use `DocumentContext`, never load PDFs directly
2. **Stable row.id**: Must be stable across re-analysis within a workflow (format varies: page-based workflows use `${source}:${pageIndex}:${idPart}`, anchor-based workflows use `${source}:${anchorOrPath}` or similar)
3. **Pure locators**: No IO, consume `PageContext` only
4. **Analyze is dry-run**: Never write files in `analyze()`
5. **Corrections keyed by row.id**: Use stable ID, not normalizedId
6. **Ignored rows visible**: Don't filter, mark with tag and exclude from counts

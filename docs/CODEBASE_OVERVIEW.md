# Conset PDF v3 - Codebase Overview

**Last Updated**: March 2, 2026  
**Version**: 1.0.0  
**Status**: Actively Maintained

## Executive Summary

Conset PDF v3 is a monorepo containing a **construction document processing library** and **command-line interface** for PDF workflow automation. The system specializes in managing construction document sets (specifications, drawings, and addenda) with deterministic, reproducible operations.

**Key Capabilities**:
- 🔀 Merge addenda into original PDFs (replace/insert sheets)
- ✂️ Split documents by sheet ID or specification section
- ✓ Validate narrative addenda against actual documents
- 📌 Fix and regenerate PDF bookmarks

## Repository Structure

```
conset-pdf-v3/
├── conset-pdf/               # Core library + CLI monorepo
│   ├── packages/
│   │   ├── core/             # @conset-pdf/core (publishable npm package)
│   │   │   ├── src/
│   │   │   │   ├── core/                 # Core public APIs (mergeAddenda, splitSet, assembleSet)
│   │   │   │   ├── workflows/            # Workflow engine (analyze → execute pattern)
│   │   │   │   ├── transcript/           # PDF extraction (PyMuPDF, PDF.js backends)
│   │   │   │   ├── analyze/              # Document/page context caching
│   │   │   │   ├── parser/               # Sheet ID & section ID extraction
│   │   │   │   ├── narrative/            # Addenda text parsing & validation
│   │   │   │   ├── standards/            # Discipline & MasterFormat normalization
│   │   │   │   ├── specs/                # Specification processing (AST, extraction, patching)
│   │   │   │   ├── bookmarks/            # PDF bookmark reading/writing
│   │   │   │   ├── layout/               # Layout profile loading
│   │   │   │   ├── locators/             # Sheet ID detection strategies (ROI, legacy)
│   │   │   │   ├── submittals/           # Submittal schedule processing
│   │   │   │   ├── utils/                # PDF, file, sorting utilities
│   │   │   │   └── __tests__/            # Integration tests
│   │   │   └── package.json
│   │   │
│   │   └── cli/               # @conset-pdf/cli (command-line interface)
│   │       ├── src/
│   │       │   ├── commands/             # CLI subcommands
│   │       │   │   ├── mergeAddenda.ts
│   │       │   │   ├── splitSet.ts
│   │       │   │   ├── assembleSet.ts
│   │       │   │   ├── fixBookmarks.ts
│   │       │   │   ├── specsPatch.ts
│   │       │   │   ├── detect.ts
│   │       │   │   └── ...
│   │       │   ├── cli.ts                # CLI entry point
│   │       │   └── bin/
│   │       └── package.json
│   │
│   ├── tests/                # Repository-level tests
│   │   ├── smoke/            # Smoke tests & invariant checks
│   │   ├── transcript/       # Transcript extraction quality tests
│   │   ├── narrative/        # Narrative parsing tests
│   │   ├── standards/        # Discipline/MasterFormat normalization tests
│   │   └── workflows/        # End-to-end workflow tests
│   │
│   ├── scripts/              # Utility scripts
│   ├── docs/                 # (Deprecated - use this codebase overview instead)
│   ├── layouts/              # Example layout profiles
│   └── examples/             # Example JSON files
│
└── conset-pdf-gui/           # Electron desktop application
    ├── src/
    │   ├── main.ts           # Electron main process
    │   ├── main/
    │   │   ├── ipc/          # IPC handlers (route to core library)
    │   │   ├── history/      # Recent files tracking
    │   │   ├── profiles/     # Profile management
    │   │   └── utils/        # Electron utilities
    │   ├── modules/          # UI modules (merge, split, bookmarks, ROI, PDF)
    │   ├── shared/           # Shared UI utilities
    │   ├── app.html          # Main window template
    │   └── *.js              # UI scripts (merge-wizard, split-drawings-wizard, etc.)
    └── package.json
```

## Technology Stack

### Runtime & Build
- **Node.js**: ≥ 18.0.0
- **TypeScript**: 5.3+ (strict mode)
- **Module System**: ES Modules (type: module)

### Core Libraries
- **pdf-lib**: PDF assembly/manipulation
- **pdfjs-dist**: PDF extraction fallback engine
- **PyMuPDF** (optional): Primary extraction backend (external Python dependency)
- **xlsx**: Spreadsheet support

### Desktop Application
- **Electron**: 39.x (cross-platform desktop)
- **Electron Builder**: Installers (NSIS, MSI, Portable)

### Testing
- **Jest**: 29.x (unit & integration tests)
- **ts-jest**: TypeScript test runner
- **Playwright**: E2E testing infrastructure

## High-Level Architecture

### Core Package Architecture

```
┌─────────────────────────────────────────────────────────┐
│               User-Facing APIs                          │
│  mergeAddenda() | splitSet() | assembleSet()            │
│  Workflow Runners (merge, bookmarks, specs-patch)       │
└──────────────────────────┬──────────────────────────────┘
                           │
        ┌──────────────────┴──────────────────┐
        │                                     │
┌───────▼────────────┐         ┌─────────────▼────────────┐
│  ANALYSIS LAYER    │         │  EXECUTION LAYER         │
│                    │         │                          │
│ • DocumentContext  │         │ • MergePlanner           │
│ • PageContext      │         │ • MergeApplier           │
│ • ROI Locators     │         │ • PDFAssembly            │
│ • Layout Profiles  │         │ • BookmarkWriter         │
│ • Narrative Parser │         │ • SpecsPatcher           │
└────────┬───────────┘         └────────┬─────────────────┘
         │                              │
         └──────────────┬───────────────┘
                        │
        ┌───────────────▼────────────────┐
        │   TRANSCRIPT EXTRACTION        │
        │                                │
        │ • PyMuPDF Backend (primary)    │
        │ • PDF.js Backend (fallback)    │
        │ • Canonicalization             │
        │ • Quality Scoring              │
        │ • Caching & Instrumentation    │
        └────────────────┬───────────────┘
                         │
        ┌────────────────▼────────────────┐
        │      PDF I/O & UTILITIES        │
        │                                 │
        │ • PDF reading (pdf-lib)         │
        │ • PDF saving (pdf-lib)          │
        │ • Text extraction (transcript)  │
        │ • Page copying & merging        │
        │ • File I/O & JSON               │
        └────────────────────────────────┘
```

### Workflow Engine Pattern

The system uses a **consistent 3-phase workflow pattern**:

```typescript
// Phase 1: ANALYZE (dry-run, no file writes)
const inventory = await runner.analyze(input);
// Returns: InventoryResult with rows, issues, conflicts

// Phase 2: APPLY CORRECTIONS (optional)
const corrected = await runner.applyCorrections(
  input,
  inventory,
  corrections
);

// Phase 3: EXECUTE (write output files)
const result = await runner.execute({
  ...input,
  outputPath: '...',
  corrections: Optional<CorrectionOverlay>
});
```

**Workflow Implementations**:
- **Merge Workflow**: `createMergeWorkflowRunner()`
- **Specs-Patch Workflow**: `createSpecsPatchWorkflowRunner()`
- **Bookmarks Workflow**: `createBookmarksWorkflowRunner()`
- **Custom Workflows**: Use `createWorkflowRunner()` factory

## Key Concepts

### Document Types

The system operates on two construction document categories:

| Type | Typical Content | Key ID | Standards |
|------|-----------------|--------|-----------|
| **drawings** | Architectural/engineering drawings | Sheet ID (e.g., "A1.2") | UDS disciplines |
| **specs** | Specification sections | Section ID (e.g., "23 09 00") | CSI MasterFormat |

### Locators & Routers

**SheetLocator** determines how sheet IDs are found on each page:

- **RoiSheetLocator**: Layout-profile-based ROI regions (highest accuracy, layout-aware)
- **LegacyTitleblockLocator**: Auto-detected title block (fallback, automatic)
- **CompositeLocator**: Try ROI → fallback to legacy
- **SpecsSectionLocator**: Text-based section parsing (specs-only)
- **RoiSpecsSectionLocator**: ROI-based section extraction (specs, profile-aware)

### Transcript Extraction

**LayoutTranscript** is the abstraction layer for PDF content:

- **Spans**: Individual text elements with position, font, size
- **Pages**: Structured page data with dimensions, rotation
- **Layout**: Box models for text regions
- **Canonical Form**: Normalized coordinate system, deterministic order
- **Quality Scoring**: Confidence metrics per page

**Backends**:
- PyMuPDF (95-99% bbox accuracy, requires Python)
- PDF.js (15-25% bbox accuracy, pure JS, always available)

### Standards Registries

**Drawings Disciplines** (UDS-style):
- Categories: Arch (A), Structural (S), Mechanical (M), Electrical (E), etc.
- Registry maps normalized codes to full names
- used in merge workflows for discipline-aware grouping

**Specs MasterFormat** (CSI 2020):
- Division format: `NN NN NN` (e.g., "23 09 00")
- Hierarchy: 5 levels (Level 0 = Section, Level 4 = Item)
- Normalization handles aliases, spacing variants

## Entry Points

### As an npm Package

```typescript
import {
  mergeAddenda,
  splitSet,
  assembleSet,
  createMergeWorkflowRunner,
  createBookmarksWorkflowRunner,
  // ... etc
} from '@conset-pdf/core';

// Use APIs directly
const report = await mergeAddenda({ /* ... */ });
```

### As a CLI

```bash
npx conset-pdf merge-addenda \
  --original original.pdf \
  --addenda addendum1.pdf addendum2.pdf \
  --output result.pdf \
  --type drawings

npx conset-pdf fix-bookmarks --pdf doc.pdf --output fixed.pdf
```

### As a Desktop Application

Launch the Electron GUI for interactive workflow management:

```bash
npm run start  # in conset-pdf-gui/
```

## Data Flow Example: Merge Workflow

```
┌─────────────────────────────────────────────────────────┐
│ User: mergeAddenda({ originalPdfPath, addendumPdfPaths })
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
        ┌──────────────────────────────┐
        │ 1. LOAD & PARSE PDFS         │
        │                              │
        │ • Load original PDF bytes    │
        │ • Load each addendum PDF     │
        │ • Extract transcript (layout │
        │   + text spans)              │
        │ • Cache in DocumentContext   │
        └──────────────┬───────────────┘
                       │
                       ▼
        ┌──────────────────────────────┐
        │ 2. LOCATE SHEET IDs          │
        │                              │
        │ For each page:               │
        │ • Use SheetLocator strategy  │
        │ • Extract ID with confidence │
        │ • Normalize (canonical form) │
        │ • Create ParsedPage record   │
        └──────────────┬───────────────┘
                       │
                       ▼
        ┌──────────────────────────────┐
        │ 3. PLAN MERGE                │
        │                              │
        │ planMerge():                 │
        │ • Match IDs across files     │
        │ • Determine replacements     │
        │ • Determine insertions       │
        │ • Flag unmatched pages       │
        │ • Generate MergePlan         │
        └──────────────┬───────────────┘
                       │
                       ▼
        ┌──────────────────────────────┐
        │ 4. APPLY PLAN                │
        │                              │
        │ applyMergePlan():            │
        │ • Load original PDF doc      │
        │ • Copy/reorder pages per     │
        │   plan                       │
        │ • Save assembly              │
        └──────────────┬───────────────┘
                       │
                       ▼
        ┌──────────────────────────────┐
        │ 5. GENERATE REPORT           │
        │                              │
        │ • Replaced sheets list       │
        │ • Inserted sheets list       │
        │ • Unmatched pages            │
        │ • Warnings & notices         │
        │ • Timing metrics             │
        └──────────────┬───────────────┘
                       │
                       ▼
        ┌──────────────────────────────┐
        │ RESULT: MergeReport          │
        │ + output.pdf (if dryRun=false)
        └──────────────────────────────┘
```

## Key Files & Modules

| File/Module | Purpose |
|-------------|---------|
| `index.ts` | Public API exports |
| `core/*.ts` | Core APIs (merge, split, assemble) |
| `workflows/` | Workflow engine & implementations |
| `transcript/` | PDF extraction & caching |
| `analyze/` | Document/page context & caching |
| `parser/` | Sheet ID & section ID extraction |
| `narrative/` | Addenda text parsing & validation |
| `standards/` | Discipline & MasterFormat registries |
| `specs/` | Spec section AST, extraction, patching |
| `bookmarks/` | PDF bookmark operations |
| `locators/` | Sheet ID detection strategies |
| `layout/` | Layout profile loading & validation |
| `utils/` | PDF I/O, file I/O, sorting, etc. |

## Testing Strategy

- **Unit Tests**: Module-level functionality
- **Integration Tests**: Cross-module workflows (merge, split, etc.)
- **Smoke Tests**: Invariant checks & key functionality
- **Quality Tests**: Transcript extraction accuracy & determinism
- **Fixture-Based**: Real PDF test cases with known outputs

Run tests:
```bash
npm test              # All tests
npm run test:smoke    # Smoke tests only
npm run verify        # Full verification (build + tests + invariants)
```

## Configuration & Customization

### Layout Profiles
Layout profiles define **ROI regions** for sheet ID/title extraction:
```json
{
  "profileId": "custom-layout",
  "sheetIdRoi": { "x": 0.05, "y": 0.85, "width": 0.2, "height": 0.1 },
  "sheetTitleRoi": { "x": 0.05, "y": 0.75, "width": 0.9, "height": 0.08 }
}
```

### Custom Sheet Locators
Implement `SheetLocator` interface for domain-specific ID detection:
```typescript
interface SheetLocator {
  locate(page: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}
```

### Workflow Implementation
Extend the workflow engine with custom logic:
```typescript
const runner = createWorkflowRunner('custom', {
  analyze(input) { /* ... */ },
  applyCorrections(input, inventory, corrections) { /* ... */ },
  execute(input) { /* ... */ }
});
```

## Development Guidelines

### Code Organization
- **Functional architecture**: Pure functions where possible
- **Type safety**: Full TypeScript strict mode
- **Error handling**: Detailed error messages with context
- **Logging**: Use verbose flags, avoid console.log directly
- **Caching**: Use DocumentContext for single-load PDF operations

### Adding New Workflows
1. Create workflow module: `src/workflows/my-workflow/`
2. Implement `analyze()`, `applyCorrections()`, `execute()`
3. Export from `workflows/index.ts`
4. Add CLI command in `packages/cli/src/commands/`
5. Write integration tests

### Adding Standards
1. Update registry: `standards/registry.ts`
2. Add dataset: `standards/datasets/myDataset.ts`
3. Create normalization function
4. Test against real documents

## Performance Considerations

### PDF Caching
- **One Load Strategy**: Load PDF bytes once, reuse across operations
- **DocumentContext**: Caches parsed transcript, bookmarks, page contexts
- **Instrumentation**: Track load counts, timing for optimization

### Extraction Quality vs Speed
- **PyMuPDF**: Slower (~500ms/page) but high accuracy (95-99%)
- **PDF.js**: Fast (~50ms/page) but low accuracy (15-25%)
- Use PDF.js for quick previews, PyMuPDF for merge planning

### Batch Operations
- Process multiple pages in parallel where possible
- Reuse DocumentContext across operations
- Write results incrementally for large files

## Common Patterns

### Using DocumentContext
```typescript
const docContext = new DocumentContext(pdfPath);
await docContext.initialize();

// Extract text from multiple pages
const pageIndexes = [0, 1, 2, 3, 4];
await docContext.extractTextForPages(pageIndexes);

// Get page context (cached)
const page = await docContext.getPageContext(0);
const text = page.getText();
```

### Creating Workflow Runners
```typescript
const runner = createMergeWorkflowRunner();

// Analyze (dry-run)
const inventory = await runner.analyze({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum1.pdf'],
});

// Execute (with output)
const result = await runner.execute({
  ...analyzeInput,
  outputPdfPath: 'result.pdf',
});
```

### Handling Errors
- Check `warnings` array in reports for non-fatal issues
- Check `issues` array in inventory for validation failures
- Check `conflicts` for cross-source disagreements
- Implementation should provide detailed `message` and `code` for each issue

## Next Steps for Understanding

1. **Start Here**: Read [Core API Documentation](./CORE_API.md)
2. **Understanding Workflows**: See [Workflow Engine Guide](./WORKFLOWS.md)
3. **PDF Extraction**: See [Transcript System](./TRANSCRIPT_SYSTEM.md)
4. **CLI Usage**: See [CLI Reference](./CLI_REFERENCE.md)
5. **For Desktop App**: See [GUI Architecture](./GUI_ARCHITECTURE.md)

---

**Questions?** Review the test files for concrete usage examples, or examine the command implementations in `packages/cli/src/commands/`.

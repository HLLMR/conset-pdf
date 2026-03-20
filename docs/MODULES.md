# Module Ecosystem & Component Guide

**Overview Document**: Core package modules and their responsibilities  
**Target Audience**: Developers integrating with conset-pdf or extending functionality

## Module Directory Map

```
packages/core/src/
├── analyze/                 # Document & page caching
├── bookmarks/               # Bookmark reading/writing/repairs
├── core/                    # High-level public APIs
├── layout/                  # Layout profile loading
├── locators/                # Sheet ID/section ID detection
├── narrative/               # Narrative PDF parsing & validation
├── parser/                  # Parsing sheet IDs & section IDs
├── specs/                   # Spec section processing
├── standards/               # Discipline & MasterFormat normalization
├── submittals/              # Submittal schedule processing
├── text/                    # Text utilities
├── transcript/              # PDF extraction abstraction
├── utils/                   # General utilities
├── workflows/               # Workflow engine & implementations
└── index.ts                 # Public API exports
```

---

## Core Modules by Function

### 1. EXTRACTION & CACHING

#### `transcript/` - PDF Extraction Backend
**Responsibility**: Abstract PDF text/layout extraction  
**Key Exports**:
- `TranscriptExtractor` - Interface for extraction backends
- `LayoutTranscript` - Standardized output format
- `createTranscriptExtractor()` - Factory for configured extractors
- PyMuPDF and PDF.js backend implementations

**When to Use**:  
- Implementing custom extraction backends
- Direct transcript access for analysis
- See: [Transcript System Guide](./TRANSCRIPT_SYSTEM.md)

**Key Concepts**:
- One PDF → LayoutTranscript (standardized format)
- Multiple backends (PyMuPDF 95-99%, PDF.js 15-25% accuracy)
- Single-load caching via DocumentContext
- Deterministic canonicalization

---

#### `analyze/` - Document & Page Context
**Responsibility**: Cache PDF operations, provide single-load semantics  
**Key Exports**:
- `DocumentContext` - Manages single PDF load and extraction
- `PageContext` - Per-page cached data and text extraction

**When to Use**:
- Accessing PDF content multiple times
- Extracting text from specific pages
- Checking document page count

**Pattern**:
```typescript
const docContext = new DocumentContext('pdf.path');
await docContext.initialize();

// Single load, reusable across operations
for (let i = 0; i < docContext.pageCount; i++) {
  const page = await docContext.getPageContext(i);
  const text = page.getText();
  // ... process
}
```

**See**: [Core API - DocumentContext](./CORE_API.md#documentpage-context)

---

### 2. LAYOUT & LOCATION

#### `layout/` - Profile Management
**Responsibility**: Load and validate layout profiles  
**Key Exports**:
- `loadLayoutProfile()` - Load from JSON file
- `createInlineLayout()` - Create from inline ROI strings
- `LayoutProfile` type definition

**When to Use**:
- Working with ROI-based sheet ID detection
- Creating/loading layout profiles
- Passing profiles to merge operations

**Profile Format**:
```json
{
  "profileId": "my-layout",
  "sheetIdRoi": { "x": 0.05, "y": 0.85, "width": 0.2, "height": 0.1 },
  "sheetTitleRoi": { "x": 0.05, "y": 0.75, "width": 0.9, "height": 0.08 }
}
```

---

#### `locators/` - Sheet ID Detection
**Responsibility**: Locate sheet IDs and section IDs on pages  
**Key Classes**:
- `RoiSheetLocator` - ROI-based detection (fastest, requires profile)
- `LegacyTitleblockLocator` - Auto-detect title block (slower, no config)
- `CompositeLocator` - Try ROI, fallback to legacy (recommended)
- `SpecsSectionLocator` - Text-based MasterFormat extraction
- `RoiSpecsSectionLocator` - ROI-based section extraction

**When to Use**:
- Detecting sheet IDs during merge
- Building custom detection strategies
- Evaluating detection accuracy

**Interface**:
```typescript
interface SheetLocator {
  locate(page: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}
```

**See**: [Locators Guide](./LOCATORS.md)

---

### 3. TEXT PARSING & NORMALIZATION

#### `parser/` - ID Extraction
**Responsibility**: Parse sheet IDs and section IDs from text  
**Key Exports**:
- `getBestDrawingsSheetId()` - Extract sheet ID from text
- `getBestSpecsSectionId()` - Extract section ID from text

**When to Use**:
- Custom text-based ID detection
- Normalizing raw ID strings
- Building ID patterns

**Example**:
```typescript
const text = "Sheet: A1.0 - Site Plan";
const result = getBestDrawingsSheetId(text, pageIndex);
// { id: "A1.0", normalized: "A1.0", confidence: 0.95 }
```

---

#### `standards/` - Discipline & MasterFormat
**Responsibility**: Normalize and map discipline/section codes  
**Key Exports**:
- `normalizDrawingsDiscipline()` - Map "ARCH" → "A"
- `normalizeSpecsMasterformat()` - Normalize "23 9" → "23 09 00"
- `getDrawingsDisciplineInfo()` - Details for discipline code
- `getSpecsDivisionInfo()` - Details for section code
- Discipline/division registries (UDS, CSI MasterFormat)

**When to Use**:
- Normalizing detected IDs to canonical form
- Sorting by discipline or section
- Building discipline/section hierarchies
- Validating ID formats

**Example**:
```typescript
const normalized = normalizeSpecsMasterformat("23 9 00");
// "23 09 00"

const info = getSpecsDivisionInfo("23");
// { code: "23", name: "Heating, Ventilating, and Air Conditioning", ... }
```

---

### 4. NARRATIVE & VALIDATION

#### `narrative/` - Addenda Parsing
**Responsibility**: Extract and parse change instructions from addenda  
**Key Exports**:
- `extractNarrativeTextFromPdf()` - Get text from PDF
- `parseNarrativeAlgorithmic()` - Parse instructions algorithmically
- `validateNarrativeAgainstInventory()` - Cross-check with document
- Types: `NarrativeInstructionSet`, `DrawingInstruction`, `SpecInstruction`

**When to Use**:
- Validating that a narrative matches actual document changes
- Parsing change notes from addenda
- Advisory analysis during merge planning

**Output Structure**:
```typescript
interface NarrativeInstructionSet {
  meta: { fileHash, pageCount, extractedAt };
  drawings: DrawingInstruction[];    // Sheet changes
  specs: SpecInstruction[];          // Section changes
  issues: NarrativeParseIssue[];     // Parsing problems
}
```

**See**: [Core API - Narrative Types](./CORE_API.md#narrative-types)

---

### 5. SPEC PROCESSING

#### `specs/` - Specification Section Management
**Responsibility**: Parse, extract, and patch specifications  
**Key Components**:
- `extract/` - Section detection and extraction
- `ast/` - Abstract syntax tree for spec content
- `patch/` - Apply patches to specifications
- `footerSectionIdParser.ts` - Extract section IDs from footer
- `inventory/` - Section inventory analysis

**When to Use**:
- Working with specification PDFs
- Extracting spec section boundaries
- Applying patches to specifications
- Validating section structure

**Key Functions**:
- `detectSections()` - Find section boundaries
- `extractSpecsSections()` - Get section text/layout
- `applySpecsPatch()` - Apply changes to PDF

---

### 6. BOOKMARKS

#### `bookmarks/` - PDF Bookmark Management
**Responsibility**: Read, validate, and repair PDF bookmarks  
**Key Components**:
- `reader.ts` - Read bookmarks from PDF
- `treeBuilder.ts` - Build/rebuild bookmark hierarchy
- `pikepdfBookmarkWriter.ts` - Write bookmarks back to PDF
- `validator.ts` - Validate bookmark structure
- `footerSectionIdParser.ts` - Extract anchors from footer

**When to Use**:
- Fixing invalid bookmarks
- Regenerating bookmarks from detected sections
- Building custom bookmark hierarchies
- Validating bookmark destinations

**Key Concepts**:
- **Section Anchors**: Where each section starts
- **Strategies**: `footer-first` (extraction) vs `inventory` (mapping)
- **Validation**: Check all destinations are on valid pages

---

### 7. WORKFLOWS

#### `workflows/` - Workflow Engine
**Responsibility**: Provide consistent analyze → execute pattern  
**Key Exports**:
- `createWorkflowRunner()` - Expert API for custom workflows
- `createMergeWorkflowRunner()` - Merge implementation
- `createSpecsPatchWorkflowRunner()` - Specs patch implementation
- `createBookmarksWorkflowRunner()` - Bookmarks implementation
- Types: `InventoryResult`, `Issue`, `Conflict`, `CorrectionOverlay`

**When to Use**:
- Building interactive workflows (plan → review → execute)
- Implementing new workflow types
- Applying user corrections before execution

**Pattern**:
```
analyze() → [plan shown to user] → applyCorrections() → execute()
```

**See**: [Workflow Engine Guide](./WORKFLOWS.md)

---

### 8. HIGH-LEVEL APIs

#### `core/` - Public APIs
**Responsibility**: High-level functions for common operations  
**Key Exports**:
- `mergeAddenda()` - Merge addenda into original
- `splitSet()` - Split by sheet ID or section ID
- `assembleSet()` - Combine multiple PDFs
- Helper functions (planner, applier, reporter)

**When to Use**:
- Simple operations that don't need workflow interaction
- Quick scripting or batch processing
- Direct import for library use

**Examples**:
```typescript
const report = await mergeAddenda({
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  outputPdfPath: 'result.pdf',
  type: 'drawings'
});
```

**See**: [Core API Documentation](./CORE_API.md)

---

### 9. UTILITIES

#### `utils/` - Helpers
**Responsibility**: PDF I/O, file I/O, sorting, and common operations  
**Key Exports**:
- `loadPdf()` / `savePdf()` / `copyPages()` - PDF operations
- `fileExists()` / `writeJson()` / `ensureDir()` - File operations
- `naturalSort()` - Human-friendly sorting
- PDF utilities (page counting, text extraction, etc.)

**When to Use**:
- File system operations
- PDF page manipulation
- Sorting with human-friendly ordering

---

#### `text/` - Text Utilities
**Responsibility**: Text processing helpers  
**Key Functions**:
- Text normalization
- Whitespace handling
- Character encoding utilities

---

#### `submittals/` - Submittal Processing
**Responsibility**: Parse and extract submittal schedules  
**When to Use**:
- Working with submittal documents
- Extracting submittal schedules from specs

---

## Data Flow Through Modules

### Example: Merge Workflow

```
Input (mergeAddenda options)
  │
  ├─► DocumentContext (load PDFs) [analyze/]
  │     │
  │     ├─► TranscriptExtractor (extract text) [transcript/]
  │     │
  │     └─► PageContext (cache pages) [analyze/]
  │
  ├─► SheetLocator (detect IDs) [locators/]
  │     │
  │     ├─► Parser (extract ID pattern) [parser/]
  │     │
  │     └─► Standards (normalize) [standards/]
  │
  ├─► MergePlanner (match sheets) [core/]
  │     │
  │     └─► NarrativeValidator (advisory) [narrative/]
  │
  ├─► UserReview / Corrections [workflows/]
  │
  ├─► MergeApplier (execute plan) [core/]
  │     │
  │     └─► PDF Assembly (save result) [utils/]
  │
  └─► MergeReport (write report) [core/]

Output (PDF + report)
```

---

## Module Dependencies

**Dependency Graph** (simplified):

```
┌──────────────────────────────────────┐
│       User-Facing APIs (core/)       │
├──────────────────────────────────────┤
│ Workflows, mergeAddenda, splitSet    │
└─────────┬──────────────────┬─────────┘
          │                  │
    ┌─────▼─────┐    ┌──────▼──────┐
    │ Analyze/  │    │ Locators/   │
    │ Document  │    │ Layout      │
    │ Context   │    └──────┬──────┘
    └─────┬─────┘           │
          │          ┌──────▼──────┐
          │          │ Parser/     │
          │          │ Standards   │
          │          └─────────────┘
          │
          ├─────────────────────┐
          │                     │
      ┌───▼────┐          ┌─────▼──┐
      │Journey │          │ Other  │
      │Extractor│         │ Modules│
      └─────────┘         └────────┘
```

---

## Integration Patterns

### Pattern 1: Using Core APIs Directly

```typescript
import { mergeAddenda } from '@conset-pdf/core';

const report = await mergeAddenda({
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  outputPdfPath: 'result.pdf',
  type: 'drawings'
});
```

**Best For**: Simple operations, scripts, quick processing

---

### Pattern 2: Using Workflow Engine

```typescript
import { createMergeWorkflowRunner } from '@conset-pdf/core';

const runner = createMergeWorkflowRunner();
const inventory = await runner.analyze(input);

// User review step
if (inventory.issues.length === 0) {
  const result = await runner.execute(executeInput);
}
```

**Best For**: Interactive applications, user review, corrections

---

### Pattern 3: Custom Extraction

```typescript
import { DocumentContext } from '@conset-pdf/core';
import { RoiSheetLocator } from '@conset-pdf/core';

const docContext = new DocumentContext('pdf.path');
await docContext.initialize();

const locator = new RoiSheetLocator(profile);

for (let i = 0; i < docContext.pageCount; i++) {
  const page = await docContext.getPageContext(i);
  const result = await locator.locate(page);
  // ... custom processing
}
```

**Best For**: Custom analysis, advanced scenarios

---

### Pattern 4: Extending with Custom Logic

```typescript
import { createWorkflowRunner } from '@conset-pdf/core';
import type { WorkflowImpl } from '@conset-pdf/core';

const impl: WorkflowImpl<Input, Input, ExecuteInput> = {
  async analyze(input) { /* ... */ },
  async applyCorrections(input, inv, corr) { /* ... */ },
  async execute(input) { /* ... */ }
};

const runner = createWorkflowRunner('custom', impl);
```

**Best For**: Custom workflows, domain-specific operations

---

## Module Selection Guide

**Choose the right module for your task:**

| Task | Module | API |
|------|--------|-----|
| Merge addenda | `core/` | `mergeAddenda()` |
| Interactive merge | `workflows/` | `createMergeWorkflowRunner()` |
| Split documents | `core/` | `splitSet()` |
| Detect sheet IDs | `locators/` | `SheetLocator.locate()` |
| Normalize IDs | `standards/` | `normalizeDrawingsDiscipline()` |
| Parse narrative | `narrative/` | `parseNarrativeAlgorithmic()` |
| Fix bookmarks | `workflows/` | `createBookmarksWorkflowRunner()` |
| Patch specs | `specs/` | `applySpecsPatch()` |
| Direct PDF access | `analyze/` | `DocumentContext` |
| Extract text | `transcript/` | `TranscriptExtractor` |
| Custom extraction | `locators/` | Implement `SheetLocator` |

---

## Common Import Patterns

```typescript
import {
  // High-level APIs
  mergeAddenda,
  splitSet,
  assembleSet,

  // Workflows
  createMergeWorkflowRunner,
  createBookmarksWorkflowRunner,
  createWorkflowRunner,

  // Document access
  DocumentContext,

  // Layouts & Locators
  loadLayoutProfile,
  RoiSheetLocator,
  CompositeLocator,

  // Standards
  normalizeDrawingsDiscipline,
  normalizeSpecsMasterformat,

  // Narrative
  parseNarrativeAlgorithmic,
  validateNarrativeAgainstInventory,

  // Utilities
  fileExists,
  writeJson,

  // Types
  type ConsetDocType,
  type MergeAddendaOptions,
  type MergeReport,
  type InventoryResult,
  type SheetLocator
} from '@conset-pdf/core';
```

---

## Testing Module Functionality

Each module has corresponding tests:

```
tests/
├── narrative/
├── standards/
├── transcript/
├── workflows/
└── smoke/
```

**Running tests**:
```bash
npm test                          # All tests
npm run test:smoke               # Smoke tests (invariant checks)
npm run verify                   # Build + test + invariants
```

---

## Performance Tips by Module

| Module | Tip |
|--------|-----|
| `analyze/` | Use one DocumentContext for multiple operations |
| `transcript/` | Install PyMuPDF for accurate text (95%+ vs 15-25% with PDF.js) |
| `locators/` | Batch extract text before location detection |
| `standards/` | Cache normalization results if processing > 1000 IDs |
| `core/` | Use workflows for large batches to enable progress reporting |
| `bookmarks/` | Section detection is cached after first run |

---

## Next Steps

1. **Start with**: [Core API Documentation](./CORE_API.md) for API reference
2. **Understand patterns**: [Workflow Engine Guide](./WORKFLOWS.md)
3. **Deep dive**: [Transcript System](./TRANSCRIPT_SYSTEM.md) for extraction details
4. **Sheet detection**: [Locators Guide](./LOCATORS.md)
5. **Command-line**: [CLI Reference](./CLI_REFERENCE.md)

---

**Questions?** Review test files in `tests/` for concrete examples.

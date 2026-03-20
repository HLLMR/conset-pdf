# Public API

**Last verified**: 2026-03-01

## Stable API Guarantees

**Stable** (backward compatible):
- `mergeAddenda()` - Options and return type
- `splitSet()` - Options and return type
- `assembleSet()` - Options and return type
- `DocumentContext` - Public methods
- `PageContext` - Public methods
- `SheetLocator` interface
- Layout profile schema
- Workflow engine types (`InventoryResult`, `CorrectionOverlay`, `ExecuteResult`, etc.)
- `createMergeWorkflowRunner()` - Factory function

**Experimental** (may change):
- Internal planner/applyPlan functions
- Report structure details (fields may be added)
- Locator implementations (use interface, not concrete classes)
- Workflow engine internals (use factory functions, not direct implementation access)

## MergeAddendaOptions

```typescript
interface MergeAddendaOptions {
  originalPdfPath: string;              // Required: Original PDF path
  addendumPdfPaths: string[];          // Required: Array of addendum PDF paths
  outputPdfPath?: string;               // Optional: Output path (required if !dryRun)
  type: ConsetDocType;                  // Required: 'drawings' | 'specs'
  mode?: 'replace+insert' | 'replace-only' | 'append-only';  // Default: 'replace+insert'
  strict?: boolean;                      // Default: false
  dryRun?: boolean;                      // Default: false
  verbose?: boolean;                     // Default: false
  reportPath?: string;                   // Optional: Path to write JSON report
  regenerateBookmarks?: boolean;         // Default: false
  inventoryOutputDir?: string;          // Optional: Directory for inventory JSON files
  locator?: SheetLocator;                // Optional: Custom locator instance
  patterns?: {
    drawingsSheetId?: string;           // Optional: Custom regex pattern
    specsSectionId?: string;             // Optional: Custom regex pattern
  };
}
```

### Locator Typed

**Recommended**: Provide `locator` option for ROI-based detection:

```typescript
import { RoiSheetLocator, loadLayoutProfile } from '@conset-pdf/core';

const layout = await loadLayoutProfile('layout.json');
const locator = new RoiSheetLocator(layout);

await mergeAddenda({
  // ... other options
  locator,
});
```

**Fallback**: If no locator provided, uses `LegacyTitleblockLocator` (auto-detected title block).

## Execution Order

**Fast + Safe** execution order:

1. **Parse** (detection phase):
   - Load original PDF once
   - Extract text per page (demand-driven, cached)
   - Detect sheet IDs using locator
   - Build ID maps

2. **Plan** (planning phase):
   - Match addendum pages to original pages
   - Generate merge plan (replacements + insertions)
   - Handle duplicates (highest confidence wins)

3. **Apply** (assembly phase):
   - Copy pages in planned order
   - Regenerate bookmarks (if requested)
   - Write output PDF

**Performance**: Text extraction cached per page. First page ~100ms, subsequent ~2ms.

## Return Types

### MergeReport

```typescript
interface MergeReport {
  kind: ConsetDocType;
  originalPath: string;
  addendumPaths: string[];
  outputPath?: string;
  
  replaced: Array<{
    id: string;
    originalPageIndexes: number[];
    addendumPageIndexes: number[];
    addendumSource: string;
  }>;
  
  inserted: Array<{
    id: string;
    insertedAtIndex: number;
    pageCount: number;
    addendumSource: string;
  }>;
  
  appendedUnmatched: Array<{
    reason: 'no-id' | 'ambiguous' | 'unmatched';
    addendumSource: string;
    pageIndexes: number[];
  }>;
  
  warnings: string[];
  notices?: string[];
  stats: {
    originalPages: number;
    finalPagesPlanned: number;
    parseTimeMs: number;
    mergeTimeMs: number;
  };
}
```

## DocumentContext API

**Stable methods**:

```typescript
class DocumentContext {
  constructor(pdfPath: string);
  
  // Initialization
  async initialize(): Promise<void>;
  
  // Page access
  get pageCount(): number;
  async getPageContext(pageIndex: number): Promise<PageContext>;
  
  // Text extraction (demand-driven)
  async extractTextForPage(pageIndex: number): Promise<PageContext>;
  async extractTextForPages(pageIndexes: number[]): Promise<Map<number, PageContext>>;
  
  // PDF bytes access (for hashing/metadata)
  getPdfBytes(): Uint8Array;
  
  // Path access
  get pdfPath(): string;
}
```

## PageContext API

**Stable methods**:

```typescript
class PageContext {
  // Page info (cached)
  get pageIndex(): number;
  get pageWidth(): number;
  get pageHeight(): number;
  get rotation(): number;
  get isLandscape(): boolean;
  
  // Page info (memoized)
  getInfo(): { width: number; height: number; rotation: number; isLandscape: boolean };
  
  // Text access (cached, synchronous)
  getText(): string;
  getTextItems(): TextItemWithPosition[];
  getVisualTextItems(): Array<{ str: string; x: number; y: number; width: number; height: number }>;
  
  // ROI text access
  getTextItemsInROI(
    roi: NormalizedROI,
    options?: {
      padNorm?: number;
      intersectionMode?: 'strict' | 'overlap';
      overlapThreshold?: number;
    }
  ): TextItemWithPosition[];
  
  // Title block (cached)
  getTitleBlockBounds(): TitleBlockBounds | null;
  hasTitleBlockBounds(): boolean;
  setTitleBlockBounds(bounds: TitleBlockBounds): void;
  
  // Detection cache
  setDetectionResult(key: string, result: any): void;
  getDetectionResult<T>(key: string): T | undefined;
}
```

## SheetLocator Interface

**Stable contract**:

```typescript
interface SheetLocator {
  locate(pageContext: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}
```

**Implementations** (use interface, not concrete):
- `RoiSheetLocator` - ROI-based detection
- `LegacyTitleblockLocator` - Auto-detected title block
- `CompositeLocator` - ROI-first with fallback
- `SpecsSectionLocator` - Specs section detection

## Layout Profile Schema

**Stable schema**:

```typescript
interface LayoutProfile {
  name: string;
  version: string;
  source?: 'auto-detected' | 'manual' | 'user-defined';
  
  sheetId: {
    rois: NormalizedROI[];
    regex?: string;
    anchorKeywords?: string[];
  };
  
  sheetTitle?: {
    rois: NormalizedROI[];
    maxLength?: number;
  };
}
```

## Error Handling

**Throws**:
- File not found
- Invalid PDF format
- Invalid layout profile
- Missing required options

**Returns warnings** (in report):
- ROI detection failures
- Duplicate IDs
- Unmatched pages
- Legacy fallback usage

## Workflow Engine API

### createMergeWorkflowRunner()

**Factory function** for merge workflow:

```typescript
import { createMergeWorkflowRunner } from '@conset-pdf/core';

const runner = createMergeWorkflowRunner();

// Analyze (dry-run inventory)
const inventory = await runner.analyze({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  profile: layoutProfile,
  narrativePdfPath: 'narrative.pdf', // Optional: advisory analysis
  options: {
    verbose: true,
    inventoryOutputDir: './inventories',
  },
});

// Apply corrections
const corrected = await runner.applyCorrections(
  analyzeInput,
  inventory,
  {
    ignoredRowIds: ['row-id-1'],
    overrides: {
      'row-id-2': { fields: { normalizedId: 'A-101' } }
    }
  }
);

// Execute
const result = await runner.execute({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  outputPdfPath: 'output.pdf',
  profile: layoutProfile,
});
```

### InventoryResult

```typescript
interface InventoryResult {
  workflowId: 'merge';
  laneId?: string;
  rows: InventoryRowBase[];  // Stable row.id (UID), row.normalizedId (detected/overridden)
  issues: Issue[];
  conflicts: Conflict[];
  summary: {
    totalRows: number;
    rowsOk: number;
    rowsWarning: number;
    rowsError: number;
    rowsConflict: number;
    rowsWithIds: number;
    rowsWithoutIds: number;
    issuesCount: number;
    conflictsCount: number;
    // Merge-specific:
    replaced?: number;
    inserted?: number;
    unmatched?: number;
  };
  meta?: Record<string, unknown>;
}
```

### CorrectionOverlay

```typescript
interface CorrectionOverlay {
  laneId?: string;  // Optional lane identifier
  ignoredRowIds?: string[];  // Rows to ignore (visible but excluded from counts)
  ignoredPages?: number[];  // Page numbers to ignore
  overrides: {
    [rowId: string]: {
      action?: string;  // Action override (future)
      fields?: Record<string, unknown>;  // Field overrides (normalizedId, sheetId, etc.)
      resolvedConflictChoice?: 'narrative' | 'detected' | 'manual';  // Conflict resolution
    };
  };
  notes?: string;  // Optional notes
}
```

**Current limitations**:
- Narrative parsing: Advisory only (narrative path stored, conflicts not generated)
- Action overrides: Not implemented (only ID overrides supported)
- Multi-lane merge: Single doc-type only (`'drawings'` or `'specs'`, not `'both'` in core)

## Additional Exports

### Narrative Processing

```typescript
// Extract text from narrative PDF
import { extractNarrativeTextFromPdf } from '@conset-pdf/core';
const narrativeDoc = await extractNarrativeTextFromPdf('narrative.pdf');

// Parse narrative instructions
import { parseNarrativeAlgorithmic } from '@conset-pdf/core';
const instructions = parseNarrativeAlgorithmic(narrativeDoc);
```

### Utilities

```typescript
import { fileExists, writeJson } from '@conset-pdf/core';

// Check if file exists
const exists = await fileExists('path/to/file.pdf');

// Write JSON file
await writeJson('output.json', data);
```

### Layout Profile Loading

```typescript
import { loadLayoutProfile, createInlineLayout } from '@conset-pdf/core';

// Load from file
const profile = await loadLayoutProfile('layout.json');

// Create inline from ROI strings
const inlineProfile = createInlineLayout(
  '0.1,0.05,0.3,0.05',  // sheet-id-roi
  '0.1,0.1,0.6,0.05'    // sheet-title-roi (optional)
);
```

### Workflow Runner Factory

```typescript
import { createWorkflowRunner } from '@conset-pdf/core';

// Generic workflow runner (for custom workflows)
const runner = createWorkflowRunner('merge', workflowImpl);
```

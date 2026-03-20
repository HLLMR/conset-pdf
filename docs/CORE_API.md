# Core API Documentation

**Package**: `@conset-pdf/core`  
**Version**: 1.0.0

## Table of Contents

1. [High-Level APIs](#high-level-apis)
2. [Workflow Engine APIs](#workflow-engine-apis)
3. [Types & Interfaces](#types--interfaces)
4. [Utility APIs](#utility-apis)

---

## High-Level APIs

These are the primary public APIs for PDF operations.

### `mergeAddenda(options: MergeAddendaOptions): Promise<MergeReport>`

Merge one or more addenda PDFs into an original PDF set, replacing or inserting sheets based on detected sheet IDs.

#### Parameters

```typescript
interface MergeAddendaOptions {
  originalPdfPath: string;           // Path to original PDF
  addendumPdfPaths: string[];        // Array of addendum PDF paths (chronological order)
  outputPdfPath?: string;            // Output path (optional when dryRun=true)
  type: ConsetDocType;               // 'drawings' | 'specs'
  mode?: 'replace+insert' | 'replace-only' | 'append-only';
  strict?: boolean;                  // Fail if any page has no ID
  dryRun?: boolean;                  // Plan only, don't write output
  verbose?: boolean;                 // Verbose console output
  reportPath?: string;               // Optional path to write JSON report
  regenerateBookmarks?: boolean;     // Regenerate bookmarks after merge
  inventoryOutputDir?: string;       // Dir for inventory JSON output
  locator?: SheetLocator;            // Custom locator (advanced)
  patterns?: {                       // Custom regex patterns
    drawingsSheetId?: string;
    specsSectionId?: string;
  };
}
```

#### Return Value

```typescript
interface MergeReport {
  kind: ConsetDocType;
  originalPath: string;
  addendumPaths: string[];
  outputPath?: string;

  replaced: Array<{
    id: string;
    originalPageIndexes: number[];    // 0-based
    addendumPageIndexes: number[];    // 0-based within addendum
    addendumSource: string;
  }>;

  inserted: Array<{
    id: string;
    insertedAtIndex: number;          // 0-based
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

#### Example

```typescript
import { mergeAddenda } from '@conset-pdf/core';

const report = await mergeAddenda({
  originalPdfPath: 'drawings.pdf',
  addendumPdfPaths: ['addendum1.pdf', 'addendum2.pdf'],
  outputPdfPath: 'merged.pdf',
  type: 'drawings',
  mode: 'replace+insert',
  verbose: true,
  reportPath: 'merge-report.json'
});

console.log(`Replaced ${report.replaced.length} sheets`);
console.log(`Inserted ${report.inserted.length} sheets`);
```

#### Notes

- **Chronological Order**: `addendumPdfPaths` should be in chronological order (earliest first)
- **ID Detection**: Uses `SheetLocator` (ROI-based or legacy auto-detect)
- **Deterministic**: Same inputs produce same output (idempotent within version)
- **Strict Mode**: When `strict=true`, fails if any page lacks an ID; otherwise appends unmatched pages to end

---

### `splitSet(options: SplitSetOptions): Promise<SplitEntry[]>`

Split a PDF by sheet IDs (drawings) or section IDs (specs) into multiple output files.

#### Parameters

```typescript
interface SplitSetOptions {
  inputPdfPath: string;              // Path to input PDF
  outputDir: string;                 // Output directory for split files
  type: ConsetDocType;               // 'drawings' | 'specs'
  groupBy?: 'prefix' | 'section' | 'division';  // Grouping strategy
  prefixes?: string[];               // Filter by these prefixes (drawings)
  tocJsonPath?: string;              // Optional mapping JSON
  pattern?: string;                  // Custom regex for ID extraction
  verbose?: boolean;                 // Verbose output
}
```

#### Return Value

```typescript
interface SplitEntry {
  key: string;                       // e.g., "M" or "23 09 00"
  title?: string;                    // e.g., spec header name
  startPage: number;                 // 1-based
  endPage: number;                   // 1-based, inclusive
  fileName: string;                  // Output filename (without path)
}
```

#### Example

```typescript
import { splitSet } from '@conset-pdf/core';

const entries = await splitSet({
  inputPdfPath: 'drawings.pdf',
  outputDir: './split-output',
  type: 'drawings',
  groupBy: 'prefix',  // Group by discipline prefix (A, S, M, E, etc.)
  verbose: true
});

// Results in files like: split-output/A-sheets.pdf, split-output/M-sheets.pdf
console.log(`Created ${entries.length} file groups`);
```

#### Notes

- **Grouping**: `prefix` groups by drawing discipline, `section` by spec section, `division` by MasterFormat division
- **Output Files**: Automatically named using the group key (e.g., "A-sheets.pdf", "23-partitions.pdf")
- **Page Ranges**: Uses 1-based page numbers for human readability

---

### `assembleSet(options: AssembleSetOptions): Promise<void>`

Assemble multiple PDF files into a single output PDF.

#### Parameters

```typescript
interface AssembleSetOptions {
  inputDir: string;                  // Dir containing input PDFs
  outputPdfPath: string;             // Output PDF path
  type: ConsetDocType;               // 'drawings' | 'specs'
  orderJsonPath?: string;            // JSON file specifying file order
  verbose?: boolean;                 // Verbose output
}
```

#### Example

```typescript
import { assembleSet } from '@conset-pdf/core';

// order.json format:
// {
//   "order": ["A-sheets.pdf", "S-sheets.pdf", "M-sheets.pdf"]
// }

await assembleSet({
  inputDir: './split-output',
  outputPdfPath: 'assembled.pdf',
  type: 'drawings',
  orderJsonPath: 'order.json',
  verbose: true
});
```

---

## Workflow Engine APIs

The workflow engine provides a **consistent 3-phase pattern** for complex operations.

### `createMergeWorkflowRunner(): WorkflowRunner<MergeAnalyzeInput, MergeAnalyzeInput, MergeExecuteInput>`

Creates a merge workflow runner with analyze → execute phases.

#### Analyze Phase (Dry-Run)

```typescript
const runner = createMergeWorkflowRunner();

const inventory = await runner.analyze({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum1.pdf'],
  profile: layoutProfile,  // Optional: layout profile
  options: {
    mode: 'replace+insert',
    verbose: true
  },
  narrativePdfPath: 'addendum-notes.pdf'  // Optional: for advisory analysis
});

// inventory contains:
// {
//   rows: InventoryRow[]        // Sheet/section mappings
//   issues: Issue[]             // Detected problems
//   conflicts: Conflict[]       // Source disagreements
//   stats: {}
// }
```

#### Execute Phase (Produces Output)

```typescript
const result = await runner.execute({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum1.pdf'],
  outputPdfPath: 'result.pdf',
  profile: layoutProfile,
  options: { /* ... */ },
  analyzed: {
    plan: inventory.plan  // Reuse plan from analyze
  }
});

// result: ExecuteResult
// {
//   outputPath: string
//   warnings: string[]
//   stats: { pagesWritten, timeMs }
// }
```

---

### `createSpecsPatchWorkflowRunner(): WorkflowRunner<SpecsPatchAnalyzeInput, SpecsPatchAnalyzeInput, SpecsPatchExecuteInput>`

Applies patches to specification PDFs.

#### Usage

```typescript
const runner = createSpecsPatchWorkflowRunner();

const inventory = await runner.analyze({
  specsPdfPath: 'specs.pdf',
  patches: [
    { sectionId: '23 09 00', action: 'revise', content: 'New text' }
  ]
});

const result = await runner.execute({
  specsPdfPath: 'specs.pdf',
  outputPdfPath: 'patched-specs.pdf',
  patches: [/* ... */]
});
```

---

### `createBookmarksWorkflowRunner(): WorkflowRunner<BookmarksAnalyzeInput, BookmarksAnalyzeInput, BookmarksExecuteInput>`

Repairs or regenerates PDF bookmarks.

#### Usage

```typescript
const runner = createBookmarksWorkflowRunner();

const inventory = await runner.analyze({
  pdfPath: 'document.pdf',
  type: 'drawings',
  sectionStartStrategy: 'footer-first'  // or 'inventory'
});

const result = await runner.execute({
  pdfPath: 'document.pdf',
  outputPdfPath: 'fixed-bookmarks.pdf',
  type: 'drawings',
  sectionStartStrategy: 'footer-first',
  allowInvalidDestinations: false
});
```

---

### `createWorkflowRunner<IAnalyze, ICorrections, IExecute>(): WorkflowRunner<...>`

**Expert/Advanced API** for creating custom workflow implementations.

```typescript
interface WorkflowImpl<IAnalyze, ICorrections, IExecute> {
  analyze(input: IAnalyze): Promise<InventoryResult>;
  applyCorrections(
    input: ICorrections,
    inventory: InventoryResult,
    corrections: CorrectionOverlay
  ): Promise<InventoryResult>;
  execute(input: IExecute): Promise<ExecuteResult>;
}

const runner = createWorkflowRunner('custom-workflow', myImpl);
await runner.analyze(input);
await runner.execute(executionInput);
```

---

## Types & Interfaces

### Document Types & Constants

```typescript
type ConsetDocType = 'drawings' | 'specs';

type WorkflowId = 'merge' | 'split' | 'assemble' | 'bookmark' | 'specs-patch';
type Severity = 'error' | 'warning' | 'info';
type RowStatus = 'ok' | 'warning' | 'error' | 'conflict';
type Confidence = number;  // 0.0 to 1.0
```

### Inventory & Issues

```typescript
interface InventoryResult {
  rows: InventoryRowBase[];
  issues: Issue[];
  conflicts: Conflict[];
  stats: Record<string, unknown>;
  plan?: MergePlan;  // Merge-specific
}

interface InventoryRowBase {
  id: string;              // Unique within workflow
  laneId?: string;         // Multi-lane workflows
  source?: string;         // Origin (filename)
  page?: number;
  status: RowStatus;       // 'ok', 'warning', 'error', 'conflict'
  confidence: Confidence;  // 0.0-1.0
  action?: string;         // 'replace', 'insert', 'skip'
  notes?: string;
  tags?: string[];
}

interface Issue {
  id: string;
  severity: Severity;
  code: string;           // e.g., 'NO_ID', 'LOW_CONFIDENCE'
  message: string;
  rowIds: string[];
  details?: Record<string, unknown>;
}

interface Conflict {
  id: string;
  nature: string;         // e.g., 'source-disagreement'
  description: string;
  values: Record<string, unknown>;
  rowIds: string[];
}
```

### Correction Overlay

```typescript
interface CorrectionOverlay {
  corrections: Array<{
    id: string;           // Row ID being corrected
    status?: RowStatus;
    action?: string;
    details?: Record<string, unknown>;
  }>;
  metadata?: {
    correctedAt: string;
    correctedBy?: string;
  };
}
```

### Layout & Locators

```typescript
interface LayoutProfile {
  profileId: string;
  sheetIdRoi?: RoiBounds;      // { x, y, width, height }; 0-1 normalized
  sheetTitleRoi?: RoiBounds;
}

interface RoiBounds {
  x: number;      // 0.0 to 1.0
  y: number;
  width: number;
  height: number;
}

interface SheetLocator {
  locate(page: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}

interface SheetLocationResult {
  id?: string;
  sheetIdNormalized?: string;
  sectionIdNormalized?: string;
  title?: string;
  confidence: number;  // 0.0-1.0
  method: string;      // e.g., 'roi', 'legacy', 'text'
  warnings: string[];
  context?: string;
}
```

### Transcript Types

```typescript
interface LayoutTranscript {
  filePath: string;
  extractionEngine: string;      // e.g., "pymupdf-1.24.1"
  extractionDate: string;        // ISO timestamp
  pages: LayoutPage[];
  metadata: TranscriptMetadata;
}

interface LayoutPage {
  pageNumber: number;            // 1-based
  pageIndex: number;             // 0-based
  width: number;                 // in points
  height: number;
  rotation: number;              // 0, 90, 180, 270
  spans: LayoutSpan[];
  lines?: LayoutLine[];
  images?: LayoutImage[];
  bbox: BBox;
}

interface LayoutSpan {
  text: string;
  bbox: BBox;                    // Bounding box
  font: {
    name: string;
    size: number;
  };
  color?: string;
}

interface BBox {
  x0: number;                    // Canonical: top-left origin
  y0: number;
  x1: number;
  y1: number;
}
```

### Narrative Types

```typescript
interface NarrativeInstructionSet {
  meta: {
    fileHash: string;
    pageCount: number;
    extractedAtIso: string;
  };
  drawings: DrawingInstruction[];
  specs: SpecInstruction[];
  issues: NarrativeParseIssue[];
}

interface DrawingInstruction {
  kind: "sheetChange";
  changeType: "revised_reissued" | "unknown";
  sheetIdRaw: string;
  sheetIdNormalized: string;
  titleRaw?: string;
  notes?: string[];
  evidence: {
    pageNumber: number;
    rawLine: string;
  };
  source: "algorithmic";
}

interface SpecInstruction {
  kind: "specSectionChange";
  sectionIdRaw: string;
  sectionIdNormalized: string;
  titleRaw?: string;
  actions: Array<{
    verb: "add" | "revise" | "delete" | "replace" | "unknown";
    targetRaw?: string;
    rawText: string;
  }>;
  evidence: {
    pageNumber: number;
    rawBlock: string;
  };
  source: "algorithmic";
}
```

---

## Utility APIs

### Document/Page Context

```typescript
class DocumentContext {
  constructor(pdfPath: string);
  
  async initialize(): Promise<void>;
  
  get pageCount(): number;
  
  async getPageContext(pageIndex: number): Promise<PageContext>;
  
  async extractTextForPages(pageIndexes: number[]): Promise<void>;
  
  getInstrumentation(): {
    loadId: string;
    totalLoads: number;
  };
}

class PageContext {
  getText(): string;
  getTextWithLayout(): LayoutSpan[];
  getPageIndex(): number;
  getPageNumber(): number;
}
```

#### Example

```typescript
const docContext = new DocumentContext('document.pdf');
await docContext.initialize();

// Extract text from specific pages
await docContext.extractTextForPages([0, 1, 2]);

const page0 = await docContext.getPageContext(0);
const text = page0.getText();
```

### Layout Profiles

```typescript
async function loadLayoutProfile(filePath: string): Promise<LayoutProfile>;

function createInlineLayout(
  sheetIdRoi: string,  // "x,y,width,height" format
  sheetTitleRoi?: string
): LayoutProfile;
```

### Sheet Locators

```typescript
export class RoiSheetLocator implements SheetLocator {
  constructor(profile: LayoutProfile);
  async locate(page: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}

export class LegacyTitleblockLocator implements SheetLocator {
  async locate(page: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}

export class CompositeLocator implements SheetLocator {
  constructor(primary: SheetLocator, fallback?: SheetLocator);
  async locate(page: PageContext): Promise<SheetLocationResult>;
  getName(): string;
}
```

### File Utilities

```typescript
async function fileExists(filePath: string): Promise<boolean>;

async function writeJson(filePath: string, data: unknown): Promise<void>;

async function ensureDir(dirPath: string): Promise<void>;
```

### Standards & Normalization

```typescript
// Drawings disciplines
function normalizDrawingsDiscipline(raw: string): string;
function getDrawingsDisciplineInfo(code: string): {
  code: string;
  name: string;
  sortOrder: number;
};

// Specs MasterFormat
function normalizeSpecsMasterformat(raw: string): string;
function getSpecsDivisionInfo(code: string): {
  code: string;
  name: string;
  level: number;
};
```

### Narrative Processing

```typescript
async function extractNarrativeTextFromPdf(
  pdfPath: string
): Promise<NarrativeTextDocument>;

function parseNarrativeAlgorithmic(
  document: NarrativeTextDocument
): NarrativeInstructionSet;

async function validateNarrativeAgainstInventory(
  instructions: NarrativeInstructionSet,
  inventory: InventoryRowBase[]
): Promise<NarrativeValidationReport>;
```

---

## Error Handling

All async functions may throw errors. Common patterns:

```typescript
try {
  const report = await mergeAddenda( /* ... */ );
} catch (error) {
  if (error instanceof Error) {
    console.error('Merge failed:', error.message);
    // Check report warnings for non-fatal issues
  }
}

// Check report for warnings/issues instead of exceptions
if (report.warnings.length > 0) {
  console.log('Warnings:', report.warnings);
}
```

---

## Best Practices

### 1. Always Check Reports
```typescript
// Non-fatal issues appear in report, not exceptions
const report = await mergeAddenda( /* ... */ );
for (const warning of report.warnings) {
  console.warn(`Warning: ${warning}`);
}
```

### 2. Use Workflow Runners for Complex Operations
```typescript
// Better: Use workflow for structured analysis
const runner = createMergeWorkflowRunner();
const inventory = await runner.analyze(input);
// User can review inventory, apply corrections
const result = await runner.execute(executeInput);
```

### 3. Prefer Deterministic Output
```typescript
// Same inputs → same outputs (within version)
const report1 = await mergeAddenda(options);
const report2 = await mergeAddenda(options);
// report1.stats should match report2.stats
```

### 4. Cache DocumentContext for Reuse
```typescript
// One PDF load for multiple operations
const docContext = new DocumentContext('doc.pdf');
await docContext.initialize();
await docContext.extractTextForPages([0, 1, 2, 3]);
// Reuse for multiple page accesses
```

---

## Version Compatibility

- **Node.js**: ≥ 18.0.0
- **TypeScript**: 5.3+
- **Module System**: ES Modules (use `import`, not `require`)

---

**See Also**:
- [Workflow Engine Guide](./WORKFLOWS.md)
- [CLI Reference](./CLI_REFERENCE.md)
- [Transcript System](./TRANSCRIPT_SYSTEM.md)

# Workflow Engine Guide

**Component**: Workflow Engine (`packages/core/src/workflows/`)  
**Pattern**: Analyze → ApplyCorrections → Execute

## Overview

The **Workflow Engine** provides a framework for multi-phase PDF operations. Instead of a single function that does everything, workflows separate concerns into logical phases:

1. **ANALYZE** (Dry-Run): Plan the operation without writing files
2. **APPLY CORRECTIONS** (Optional): Let user review/modify the plan
3. **EXECUTE**: Perform the actual operation with output files

This enables **interactive, reversible workflows** where users can preview and correct plans before execution.

---

## Workflow Pattern

### Phase 1: ANALYZE

**Purpose**: Inspect input, generate execution plan, detect issues  
**Side Effects**: None (read-only, no file writes)  
**Returns**: `InventoryResult` with plan, issues, conflicts

```typescript
const runner = createMergeWorkflowRunner();

const inventory = await runner.analyze({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  // ... other options
});

// inventory contains:
// {
//   rows: InventoryRow[]      // Page-by-page analysis
//   issues: Issue[]           // Detected problems
//   conflicts: Conflict[]     // Cross-source disagreements
//   stats: { /* timing, counts */ }
//   plan?: MergePlan          // Execution plan (merge-specific)
// }
```

#### What Gets Analyzed?

For **merge workflow**:
- ✓ Original PDF sheet IDs & pages
- ✓ Addendum PDF sheet IDs & pages
- ✓ Matching pairs (original → replacement)
- ✓ New sheets (insertions)
- ✓ Unmatched pages (ambiguous, missing ID)
- ✓ Confidence scores per match
- ✓ Warnings (low confidence, duplicates, etc.)

For **specs-patch workflow**:
- ✓ Spec sections & page ranges
- ✓ Patch applicability
- ✓ Section parsing issues

For **bookmarks workflow**:
- ✓ Current bookmark structure
- ✓ Page destinations (valid/invalid)
- ✓ Section anchors (from footer, inventory, etc.)

---

### Phase 2: APPLY CORRECTIONS (Optional)

**Purpose**: Modify the plan based on user feedback  
**Prerequisites**: Must have completed `analyze()` first  
**Returns**: Modified `InventoryResult`

```typescript
const corrections: CorrectionOverlay = {
  corrections: [
    {
      id: 'A1.0',           // Row ID being corrected
      status: 'ok',         // Change status (ok/warning/error/conflict)
      action: 'replace',    // Change action
      details: {            // Custom patch data
        newTitle: 'Title Changed'
      }
    },
    {
      id: 'M2.1',
      action: 'skip'        // Skip this page instead of inserting
    }
  ],
  metadata: {
    correctedAt: new Date().toISOString(),
    correctedBy: 'user@example.com'
  }
};

const correctedInventory = await runner.applyCorrections(
  originalInput,
  inventory,
  corrections
);
```

#### Correction Types

| Field | Purpose | Example |
|-------|---------|---------|
| `id` | Row ID to correct | "A1.0", "23 09 00" |
| `status` | New row status | "ok", "warning", "error" |
| `action` | New action | "replace", "insert", "skip" |
| `details` | Custom patch data | `{ newTitle: "..." }` |

---

### Phase 3: EXECUTE

**Purpose**: Write output files and complete the operation  
**Side Effects**: Creates output files  
**Returns**: `ExecuteResult` with success/failure details

```typescript
const result = await runner.execute({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  outputPdfPath: 'result.pdf',      // Where to write
  analyzed: {
    plan: inventory.plan             // Reuse plan from analyze
  },
  corrections: correctedInventory    // Optional: apply corrections
});

// result: ExecuteResult
// {
//   outputPath: string
//   warnings: string[]
//   stats: {
//     pagesWritten: number
//     timeMs: number
//   }
// }
```

---

## Workflow Implementations

### 1. Merge Workflow

**Factory**: `createMergeWorkflowRunner()`

```typescript
const runner = createMergeWorkflowRunner();

// Analyze: Plan the merge
const inventory = await runner.analyze({
  docType: 'drawings',              // 'drawings' | 'specs'
  originalPdfPath: 'orig.pdf',
  addendumPdfPaths: ['add1.pdf', 'add2.pdf'],
  profile?: layoutProfile,          // Optional layout profile
  options?: {
    mode: 'replace+insert',         // Merge strategy
    strict: false,                  // Fail on missing ID?
    verbose: true
  },
  narrativePdfPath?: 'notes.pdf'   // Optional narrative for validation
});

// Execute: Write result
const result = await runner.execute({
  docType: 'drawings',
  originalPdfPath: 'orig.pdf',
  addendumPdfPaths: ['add1.pdf', 'add2.pdf'],
  outputPdfPath: 'result.pdf',
  profile: layoutProfile,
  analyzed: { plan: inventory.plan }
});
```

**Returned Rows**:
```typescript
interface MergeRow extends InventoryRowBase {
  originalPageIdx?: number;         // Page in original PDF
  addendumSource?: string;          // Which addendum?
  addendumPageIdx?: number;         // Page in that addendum
  type: 'original' | 'replacement' | 'insertion' | 'unmatched';
}
```

**Issues Detected**:
- `NO_ID` - Page has no detectable ID
- `LOW_CONFIDENCE` - ID detected but confidence < threshold
- `DUPLICATE` - Same ID appears multiple times
- `AMBIGUOUS` - ID found but with low confidence on multiple pages

---

### 2. Specs-Patch Workflow

**Factory**: `createSpecsPatchWorkflowRunner()`

```typescript
const runner = createSpecsPatchWorkflowRunner();

const inventory = await runner.analyze({
  specsPdfPath: 'specs.pdf',
  patches: [
    {
      sectionId: '23 09 00',
      action: 'revise' | 'add' | 'delete' | 'replace',
      content?: 'Patch content or description'
    }
  ]
});

const result = await runner.execute({
  specsPdfPath: 'specs.pdf',
  outputPdfPath: 'patched-specs.pdf',
  patches: [ /* ... */ ]
});
```

---

### 3. Bookmarks Workflow

**Factory**: `createBookmarksWorkflowRunner()`

```typescript
const runner = createBookmarksWorkflowRunner();

const inventory = await runner.analyze({
  pdfPath: 'document.pdf',
  type: 'drawings',                 // 'drawings' | 'specs'
  sectionStartStrategy: 'footer-first'  // 'footer-first' | 'inventory'
});

const result = await runner.execute({
  pdfPath: 'document.pdf',
  outputPdfPath: 'bookmarked.pdf',
  type: 'drawings',
  sectionStartStrategy: 'footer-first',
  allowInvalidDestinations: false   // Strict validation
});
```

**Section Start Strategies**:
- `footer-first` - Extract section codes from footer band using OCR
- `inventory` - Use page inventory mapping (requires preprocessing)

---

### 4. Custom Workflows (Expert API)

**Factory**: `createWorkflowRunner<IAnalyze, ICorrections, IExecute>()`

For advanced use cases, implement your own workflow:

```typescript
interface MyAnalyzeInput { /* ... */ }
interface MyExecuteInput { /* ... */ }

const impl: WorkflowImpl<MyAnalyzeInput, MyAnalyzeInput, MyExecuteInput> = {
  async analyze(input: MyAnalyzeInput): Promise<InventoryResult> {
    // 1. Read input files
    // 2. Parse content
    // 3. Generate execution plan
    // 4. Return InventoryResult with rows, issues, conflicts
    return {
      rows: [ /* */ ],
      issues: [ /* */ ],
      conflicts: [ /* */ ],
      stats: { /* */ }
    };
  },

  async applyCorrections(
    input: MyAnalyzeInput,
    inventory: InventoryResult,
    corrections: CorrectionOverlay
  ): Promise<InventoryResult> {
    // Merge corrections into inventory
    // Return modified inventory
    return modifiedInventory;
  },

  async execute(input: MyExecuteInput): Promise<ExecuteResult> {
    // Write output files
    // Return execution result
    return {
      outputPath: input.outputPath,
      warnings: [ /* */ ],
      stats: { /* */ }
    };
  }
};

const runner = createWorkflowRunner<MyAnalyzeInput, MyAnalyzeInput, MyExecuteInput>(
  'my-workflow',
  impl
);

const inventory = await runner.analyze(myInput);
const result = await runner.execute(myExecuteInput);
```

---

## Core Data Structures

### InventoryResult

Structure returned by `analyze()`:

```typescript
interface InventoryResult {
  rows: InventoryRowBase[];         // Page-by-page analysis
  issues: Issue[];                  // Detected problems
  conflicts: Conflict[];            // Cross-source disagreements
  stats: Record<string, unknown>;   // Timing, counts, etc.
  plan?: MergePlan;                 // Workflow-specific execution plan
}
```

### InventoryRowBase

Base row for all workflows (extended by specific workflows):

```typescript
interface InventoryRowBase {
  id: string;                       // Unique within workflow
  laneId?: string;                  // Multi-lane workflows (future)
  source?: string;                  // Origin (filename, page number, etc.)
  page?: number;                    // Page number (1-based) or index
  status: RowStatus;                // 'ok' | 'warning' | 'error' | 'conflict'
  confidence: Confidence;           // 0.0 to 1.0
  action?: string;                  // 'replace' | 'insert' | 'skip' | etc.
  notes?: string;                   // Human-readable notes
  tags?: string[];                  // Categorization tags
}
```

### Issue

Problem detected during analysis:

```typescript
interface Issue {
  id: string;                       // Unique issue ID
  severity: Severity;               // 'error' | 'warning' | 'info'
  code: string;                     // Machine-readable code
  message: string;                  // Human-readable message
  rowIds: string[];                 // Affected rows
  details?: Record<string, unknown>;  // Additional context
}
```

### Conflict

Cross-source disagreement (e.g., narrative vs. detection):

```typescript
interface Conflict {
  id: string;                       // Unique conflict ID
  nature: string;                   // 'source-disagreement', etc.
  description: string;              // What disagrees?
  values: Record<string, unknown>;  // Values from different sources
  rowIds: string[];                 // Affected rows
}
```

### CorrectionOverlay

User-provided corrections to override plan:

```typescript
interface CorrectionOverlay {
  corrections: Array<{
    id: string;                     // Row ID being corrected
    status?: RowStatus;             // Override status
    action?: string;                // Override action
    details?: Record<string, unknown>;  // Patch data
  }>;
  metadata?: {
    correctedAt: string;            // ISO timestamp
    correctedBy?: string;           // User identifier
  };
}
```

### ExecuteResult

Result of `execute()`:

```typescript
interface ExecuteResult {
  outputPath: string;               // Path to generated file
  warnings: string[];               // Non-fatal issues
  stats: {
    pagesWritten: number;           // Total pages in output
    timeMs: number;                 // Execution time
  };
}
```

---

## Common Patterns

### Pattern 1: Dry-Run Inspection

```typescript
const runner = createMergeWorkflowRunner();

// Dry-run: analyze without output
const inventory = await runner.analyze(input);

console.log(`Replacements: ${inventory.rows.filter(r => r.action === 'replace').length}`);
console.log(`Insertions: ${inventory.rows.filter(r => r.action === 'insert').length}`);
console.log(`Issues: ${inventory.issues.length}`);

if (inventory.issues.some(i => i.severity === 'error')) {
  console.error('Errors detected, aborting');
  return;
}

// OK to execute
const result = await runner.execute({
  ...input,
  outputPdfPath: 'result.pdf'
});
```

### Pattern 2: User Review & Correction

```typescript
// Step 1: Analyze
const inventory = await runner.analyze(input);

// Step 2: Display results to user
console.log(JSON.stringify(inventory, null, 2));

// Step 3: Get corrections from user (e.g., via CLI prompt)
const corrections = getUserCorrections(inventory);

// Step 4: Apply corrections
const correctedInventory = await runner.applyCorrections(
  input,
  inventory,
  corrections
);

// Step 5: Execute
const result = await runner.execute({
  ...input,
  outputPdfPath: 'result.pdf',
  corrections
});
```

### Pattern 3: Batch Processing

```typescript
const runner = createMergeWorkflowRunner();

for (const docPair of documentQueue) {
  // Analyze
  const inventory = await runner.analyze({
    docType: docPair.type,
    originalPdfPath: docPair.original,
    addendumPdfPaths: docPair.addenda,
    verbose: true
  });

  // Check for fatal errors
  if (inventory.issues.some(i => i.severity === 'error')) {
    console.warn(`Skipping ${docPair.original}: ${inventory.issues[0].message}`);
    continue;
  }

  // Execute
  const result = await runner.execute({
    ...input,
    outputPdfPath: docPair.output
  });

  console.log(`Completed: ${docPair.output}`);
}
```

---

## Error Handling in Workflows

### Pre-Execution Validation

Check `inventory.issues` **before** calling `execute()`:

```typescript
const inventory = await runner.analyze(input);

// Filter by severity
const errors = inventory.issues.filter(i => i.severity === 'error');
const warnings = inventory.issues.filter(i => i.severity === 'warning');

if (errors.length > 0 && !options.force) {
  throw new Error(`${errors.length} errors detected:${
    errors.map(e => `\n  - ${e.code}: ${e.message}`).join('')
  }`);
}

if (warnings.length > 0) {
  console.warn(`${warnings.length} warnings detected`);
  // Continue anyway
}
```

### Execution Errors

Errors during `execute()` throw exceptions:

```typescript
try {
  const result = await runner.execute(input);
} catch (error) {
  if (error instanceof Error) {
    console.error(`Execution failed: ${error.message}`);
  }
}
```

---

## Extending Workflows

### Adding a Custom Workflow Step

Extend the standard workflow without reimplementation:

```typescript
const standard = createMergeWorkflowRunner();

const enhanced = {
  async analyze(input) {
    const inventory = await standard.analyze(input);
    
    // Add custom analysis
    const custom = await myCustomAnalysis(inventory);
    inventory.stats.custom = custom;
    
    return inventory;
  },

  async applyCorrections(input, inventory, corrections) {
    // Add custom correction logic
    const enhanced = applyCustomCorrections(inventory, corrections);
    return await standard.applyCorrections(input, enhanced, corrections);
  },

  async execute(input) {
    // Add custom pre-execution hook
    onBeforeExecute(input);
    const result = await standard.execute(input);
    // Add custom post-execution hook
    onAfterExecute(result);
    return result;
  }
};
```

---

## Performance Considerations

### Caching in Analyze

The `analyze()` phase should **cache PDF operations**:

```typescript
// ✓ Good: Load PDF once, reuse
const docContext = new DocumentContext(pdfPath);
await docContext.initialize();

const pages = [
  await docContext.getPageContext(0),
  await docContext.getPageContext(1),
  // ... reuse for all analysis
];

// ✗ Bad: Load PDF multiple times
for (let i = 0; i < pageCount; i++) {
  const context = new DocumentContext(pdfPath);  // Repeated load!
  await context.initialize();
}
```

### Parallel Analysis

Analyze multiple pages in parallel:

```typescript
const pageIndexes = Array.from({ length: pageCount }, (_, i) => i);

// Extract all pages at once
await docContext.extractTextForPages(pageIndexes);

// Then process in parallel
const results = await Promise.all(
  pageIndexes.map(async (i) => {
    const page = await docContext.getPageContext(i);
    return analyzePageLocally(page);  // No I/O
  })
);
```

---

## Testing Workflows

### Unit Testing a Workflow

```typescript
import { createMergeWorkflowRunner } from '@conset-pdf/core';

describe('Merge Workflow', () => {
  it('should detect replacements', async () => {
    const runner = createMergeWorkflowRunner();

    const inventory = await runner.analyze({
      docType: 'drawings',
      originalPdfPath: 'test/inputs/original.pdf',
      addendumPdfPaths: ['test/inputs/addendum.pdf']
    });

    expect(inventory.rows).toContainEqual(
      expect.objectContaining({
        id: 'A1.0',
        action: 'replace'
      })
    );
  });

  it('should apply corrections', async () => {
    const runner = createMergeWorkflowRunner();

    const inventory = await runner.analyze( /* ... */ );

    const corrected = await runner.applyCorrections(
      input,
      inventory,
      {
        corrections: [
          { id: 'A1.0', action: 'skip' }
        ]
      }
    );

    const modified = corrected.rows.find(r => r.id === 'A1.0');
    expect(modified?.action).toBe('skip');
  });
});
```

---

## Next Steps

- Review [Merge Workflow Tests](../tests/workflows/) for real examples
- Study [CLI Commands](./CLI_REFERENCE.md#merge-addenda) for workflow usage patterns
- Examine [DocumentContext](./TRANSCRIPT_SYSTEM.md) for caching patterns


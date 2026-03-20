# Getting Started Guide

**Quick onboarding for conset-pdf v3**  
**Target**: New developers integrating or extending the system

## 5-Minute Quick Start

### Installation

```bash
# As an npm package
npm install @conset-pdf/core

# Or clone and develop locally
git clone <repo>
cd conset-pdf
npm install
npm run build
```

### Basic Usage (Merge Addenda)

```typescript
import { mergeAddenda } from '@conset-pdf/core';

const report = await mergeAddenda({
  originalPdfPath: 'drawings.pdf',
  addendumPdfPaths: ['addendum1.pdf', 'addendum2.pdf'],
  outputPdfPath: 'merged.pdf',
  type: 'drawings'
});

console.log(`Replaced ${report.replaced.length} sheets`);
console.log(`Inserted ${report.inserted.length} sheets`);
if (report.warnings.length > 0) {
  console.warn('Warnings:', report.warnings);
}
```

### CLI Usage

```bash
# Merge addenda from command line
conset-pdf merge-addenda \
  --original drawings.pdf \
  --addenda addendum1.pdf addendum2.pdf \
  --output merged.pdf \
  --type drawings

# Detect sheet IDs
conset-pdf detect --pdf drawings.pdf --type drawings

# Split by discipline
conset-pdf split-set --input drawings.pdf --output-dir ./split --type drawings
```

---

## What This Tool Does

Conset PDF v3 automates **construction document workflows**:

| Workflow | Input | Output | Use Case |
|----------|-------|--------|----------|
| **Merge** | Original PDF + addenda PDFs | Updated PDF | Apply plan changes to drawing/spec sets |
| **Split** | Large PDF | Multiple PDFs per discipline | Distribute work by responsibility |
| **Assemble** | Multiple PDFs | Single merged PDF | Combine finished work |
| **Fix Bookmarks** | PDF with bad bookmarks | PDF with fixed bookmarks | Make PDFs navigable |
| **Validate Narrative** | Narrative + document PDF | Validation report | Verify changes were applied correctly |

---

## Core Concepts (5 Minutes Read)

### Document Types

The system works with **two document types**:

1. **Drawings** (architectural/engineering)
   - ID: Sheet number (e.g., "A1.2")
   - Standard: UDS disciplines (A=Architecture, S=Structural, M=Mechanical, E=Electrical)
   - Example: A1.2 = First architectural sheet

2. **Specs** (specification sections)
   - ID: MasterFormat code (e.g., "23 09 00")
   - Standard: CSI MasterFormat 2020
   - Example: 23 09 00 = Heating, Ventilating, Air Conditioning (HVAC)

---

### Locating Sheet IDs

The system **detects sheet IDs** using **locators**:

```
ROI Locator        Legacy Locator       Composite (Recommended)
  ↓                  ↓                     ↓
[Fast, Accurate]  [Slow, Fallback]    [Best of Both]
Requires profile  No setup needed      Try fast first, then fallback
Examples:         Auto-detects         Handles all layouts
· Bottom-right    title block
· Grid
```

**Layout profiles** define where to look:
```json
{
  "sheetIdRoi": { "x": 0.05, "y": 0.85, "width": 0.2, "height": 0.1 },
  "sheetTitleRoi": { "x": 0.05, "y": 0.75, "width": 0.9, "height": 0.08 }
}
```

See: [Locators Guide](./LOCATORS.md)

---

### Workflow Pattern

Complex operations use a **3-phase workflow**:

```
Phase 1: ANALYZE (dry-run)
  ↓
  Inspect plan, detect issues
  "What would happen?"
  
Phase 2: APPLY CORRECTIONS (user review)
  ↓
  User can fix/override decisions
  "Let me adjust this..."
  
Phase 3: EXECUTE (write files)
  ↓
  Actually create output
  "Perfect! Do it!"
```

```typescript
const runner = createMergeWorkflowRunner();

// Phase 1: Dry-run plan
const inventory = await runner.analyze({
  docType: 'drawings',
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf']
});

console.log(`Issues found: ${inventory.issues.length}`);
inventory.issues.forEach(issue => {
  console.log(`  - ${issue.code}: ${issue.message}`);
});

// Phase 3: Execute if OK
if (inventory.issues.every(i => i.severity !== 'error')) {
  const result = await runner.execute({
    docType: 'drawings',
    originalPdfPath: 'original.pdf',
    addendumPdfPaths: ['addendum.pdf'],
    outputPdfPath: 'result.pdf'
  });
  console.log(`Success: ${result.outputPath}`);
}
```

See: [Workflow Engine Guide](./WORKFLOWS.md)

---

### PDF Text Extraction

Internally, the system **extracts text** from PDFs using:

1. **PyMuPDF** (primary, 95-99% accuracy)
   - Fast, accurate bounding boxes
   - Requires Python runtime: `pip install pymupdf`
   - Recommended for production

2. **PDF.js** (fallback, 15-25% accuracy)
   - Pure JavaScript, zero dependencies
   - Used automatically if PyMuPDF unavailable
   - OK for preview/analysis, not layout work

The `DocumentContext` class manages **single-load semantics**:

```typescript
const docContext = new DocumentContext('document.pdf');
await docContext.initialize();  // Loads once

// Reuse across multiple operations
for (let i = 0; i < docContext.pageCount; i++) {
  const page = await docContext.getPageContext(i);
  const text = page.getText();  // Already cached, instant
}
```

See: [Transcript System](./TRANSCRIPT_SYSTEM.md)

---

## Next: Choose Your Path

### Path 1: Simple Scripts

**Goal**: Quick automation without complexity  
**Time**: 15 minutes

```bash
# Use CLI or simple function calls
conset-pdf merge-addenda \
  --original orig.pdf \
  --addenda add1.pdf add2.pdf \
  --output result.pdf \
  --type drawings
```

**Read**:
- [CLI Reference](./CLI_REFERENCE.md)
- [Core API - High-Level](./CORE_API.md#high-level-apis)

---

### Path 2: Interactive Application

**Goal**: User review → apply corrections → execute  
**Time**: 1-2 hours

```typescript
// User-facing workflow
const runner = createMergeWorkflowRunner();
const inventory = await runner.analyze(input);

// Show inventory to user
displayToUser(inventory);

// User makes corrections
const corrections = getUserCorrections();

// Execute with corrections
const result = await runner.execute(input, corrections);
```

**Read**:
- [Workflow Engine Guide](./WORKFLOWS.md)
- [Core API - Workflows](./CORE_API.md#workflow-engine-apis)

---

### Path 3: Custom Integration

**Goal**: Extend system with custom logic  
**Time**: 2-4 hours

```typescript
// Implement custom workflow
class MyWorkflow implements WorkflowImpl {
  async analyze(input) { /* ... */ }
  async execute(input) { /* ... */ }
}

const runner = createWorkflowRunner('my-workflow', new MyWorkflow());
```

**Read**:
- [Module Ecosystem](./MODULES.md)
- [Advanced Customization](#advanced-customization) (below)

---

### Path 4: Deep Integration

**Goal**: Understanding every component  
**Time**: 1-2 days

**Read in order**:
1. [Codebase Overview](./CODEBASE_OVERVIEW.md)
2. [Core API Documentation](./CORE_API.md)
3. [Module Ecosystem](./MODULES.md)
4. [Workflow Engine](./WORKFLOWS.md)
5. [Transcript System](./TRANSCRIPT_SYSTEM.md)
6. [Locators Guide](./LOCATORS.md)
7. Review test code in `tests/`

---

## Common Tasks

### Task 1: Merge Two PDFs

```typescript
import { mergeAddenda } from '@conset-pdf/core';

const report = await mergeAddenda({
  originalPdfPath: './original.pdf',
  addendumPdfPaths: ['./addendum.pdf'],
  outputPdfPath: './merged.pdf',
  type: 'drawings'
});

console.log(`Done! ${report.replaced.length} replacements, ${report.inserted.length} insertions`);
```

**See**: [CLI - merge-addenda](./CLI_REFERENCE.md#merge-addenda) | [API - mergeAddenda](./CORE_API.md#mergeaddendaoptions)

---

### Task 2: Detect Sheet IDs in PDF

```typescript
import { DocumentContext, RoiSheetLocator, loadLayoutProfile } from '@conset-pdf/core';

const profile = await loadLayoutProfile('layout-profile.json');
const locator = new RoiSheetLocator(profile);

const docContext = new DocumentContext('drawing.pdf');
await docContext.initialize();

for (let i = 0; i < docContext.pageCount; i++) {
  const page = await docContext.getPageContext(i);
  const result = await locator.locate(page);
  console.log(`Page ${i + 1}: ${result.sheetIdNormalized} (${result.confidence})`);
}
```

**See**: [CLI - detect](./CLI_REFERENCE.md#detect) | [Locators Guide](./LOCATORS.md)

---

### Task 3: Split PDF by Discipline

```typescript
import { splitSet } from '@conset-pdf/core';

const entries = await splitSet({
  inputPdfPath: './drawings.pdf',
  outputDir: './split-output',
  type: 'drawings',
  groupBy: 'prefix'  // Discipline prefix: A, S, M, E, etc.
});

console.log(`Created ${entries.length} file groups`);
entries.forEach(entry => {
  console.log(`  ${entry.key}: ${entry.fileName}`);
});
```

**See**: [CLI - split-set](./CLI_REFERENCE.md#split-set) | [API - splitSet](./CORE_API.md#splitsetoptions)

---

### Task 4: Fix Bookmarks

```typescript
import { createBookmarksWorkflowRunner } from '@conset-pdf/core';

const runner = createBookmarksWorkflowRunner();

const result = await runner.execute({
  pdfPath: './document.pdf',
  outputPdfPath: './bookmarked.pdf',
  type: 'drawings',
  sectionStartStrategy: 'footer-first'
});

console.log(`Bookmarks fixed: ${result.outputPath}`);
```

**See**: [CLI - fix-bookmarks](./CLI_REFERENCE.md#fix-bookmarks) | [Workflows](./WORKFLOWS.md#3-bookmarks-workflow)

---

### Task 5: Create Custom Locator

```typescript
import type { SheetLocator, SheetLocationResult } from '@conset-pdf/core';
import type { PageContext } from '@conset-pdf/core';

class MyLocator implements SheetLocator {
  async locate(page: PageContext): Promise<SheetLocationResult> {
    const text = page.getText();
    const match = text.match(/ID:\s*([A-Z]\d+\.\d+)/);
    
    if (match) {
      return {
        id: match[1],
        sheetIdNormalized: match[1],
        confidence: 0.95,
        method: 'custom'
      };
    }
    
    return { confidence: 0.0, method: 'custom', warnings: [] };
  }

  getName(): string {
    return 'my-locator';
  }
}

// Use it
const locator = new MyLocator();
const report = await mergeAddenda({
  originalPdfPath: 'original.pdf',
  addendumPdfPaths: ['addendum.pdf'],
  outputPdfPath: 'result.pdf',
  type: 'drawings',
  locator  // Pass custom locator
});
```

**See**: [Locators Guide - Custom](./LOCATORS.md#advanced-custom-locator)

---

## Troubleshooting

### PyMuPDF Not Available

**Symptom**: "PyMuPDF not found, falling back to PDF.js"

**Solution**:
```bash
# Install PyMuPDF
pip install pymupdf

# Or verify it's in PATH
which pymupdf  # Linux/Mac
where pymupdf  # Windows
```

If still failing:
```bash
export CONSET_PDF_PYMUPDF=/path/to/pymupdf
npm start
```

---

### Sheet IDs Not Detected

**Symptom**: All pages return confidence < 0.5

**Possible Causes**:
1. Layout profile coordinates wrong
2. PDF.js fallback (use PyMuPDF instead)
3. Document format not standard

**Debug**:
```typescript
const page = await docContext.getPageContext(0);
const text = page.getText();
console.log(text.substring(0, 500));  // Print first 500 chars

// Is the sheet ID visible in text?
if (!text.includes('A1')) {
  console.warn('Sheet ID not in extracted text!');
  // Maybe it's an image (scanned PDF)
}
```

---

### Merge Slower Than Expected

**Symptom**: Merge takes > 10 seconds for 100-page PDF

**Possible Causes**:
1. PDF.js backend (slow + inaccurate)
2. Re-extracts text unnecessarily
3. Legacy locator on every page

**Optimize**:
```typescript
// Use ROI locator (2-5x faster)
const profile = await loadLayoutProfile('profile.json');
const locator = new RoiSheetLocator(profile);

const report = await mergeAddenda({
  // ...
  locator  // Use fast ROI instead of auto-detect
});
```

---

### Merge Results Contain Too Many Warnings

**Symptom**: report.warnings.length > 50

**Check**:
```typescript
report.warnings.forEach(w => console.log(w));
```

**Common Warnings**:
- "Low confidence on page X" → Sheet ID detected but confidence < 0.8
- "Multiple IDs detected on page X" → Ambiguous detection
- "Page has no ID" → Skip/append page missing clear ID

**Fix**:
- Refine layout profile
- Check document format consistency
- Use interactive workflow to review and correct

---

## Best Practices

### 1. Always Use Composite Locators in Production

```typescript
import { CompositeLocator, RoiSheetLocator, LegacyTitleblockLocator } from '@conset-pdf/core';

const profile = await loadLayoutProfile('profile.json');
const roi = new RoiSheetLocator(profile);
const legacy = new LegacyTitleblockLocator();

const locator = new CompositeLocator(roi, legacy);  // ✓ Good

// Don't do...
const locator = new RoiSheetLocator(profile);       // ✗ Single point of failure
```

---

### 2. Check Warnings Before Using Reports

```typescript
const report = await mergeAddenda( /* ... */ );

if (report.warnings.length > 10) {
  console.warn(`⚠️ Many warnings detected`);
  report.warnings.slice(0, 5).forEach(w => console.log(`  - ${w}`));
  console.log(`  ... and ${report.warnings.length - 5} more`);
  
  // Maybe use interactive workflow instead
}
```

---

### 3. Batch Extract Before Analyze

```typescript
// ✓ Good: Extract all pages at once
const indexes = Array.from({ length: 100 }, (_, i) => i);
await docContext.extractTextForPages(indexes);

// Then analyze (fast, text already cached)
for (const i of indexes) {
  const page = await docContext.getPageContext(i);
  // ... process
}

// ✗ Avoid: Extracting on-demand in loop
for (let i = 0; i < 100; i++) {
  const page = await docContext.getPageContext(i);  // Extract only for this page
  // ... process
}
```

---

### 4. Validate Layout Profile with Test PDF

```typescript
// Before production run, test profile with sample
const result = await mergeAddenda({
  originalPdfPath: 'sample-original.pdf',
  addendumPdfPaths: ['sample-addendum.pdf'],
  outputPdfPath: '/tmp/test-merge.pdf',
  type: 'drawings',
  layout: newProfile,
  dryRun: true  // Don't write output
});

if (result.warnings.length > 0) {
  console.log('Profile needs adjustment:');
  result.warnings.forEach(w => console.log(`  - ${w}`));
}
```

---

## Environment Setup

### Node.js

```bash
# Verify Node version (18+)
node --version  # Should be v18.0.0 or higher

# Install dependencies
npm install

# Build TypeScript
npm run build

# Run tests
npm test

# Run development server (if applicable)
npm run dev
```

### Python (for PyMuPDF)

```bash
# Check Python availability
python --version

# Install PyMuPDF
pip install pymupdf

# Verify
python -c "import fitz; print(fitz.__version__)"
```

### Development

```bash
# Watch mode build
npm run build -- --watch

# Run CLI during dev
npx conset-pdf --help

# IDE: TypeScript checking in VS Code
# Install TypeScript extension, or
npm install -g typescript
```

---

## Key Files Reference

| File | Purpose |
|------|---------|
| [CODEBASE_OVERVIEW.md](./CODEBASE_OVERVIEW.md) | Project structure & architecture |
| [CORE_API.md](./CORE_API.md) | API reference for all functions |
| [CLI_REFERENCE.md](./CLI_REFERENCE.md) | Command-line interface guide |
| [WORKFLOWS.md](./WORKFLOWS.md) | 3-phase workflow pattern |
| [TRANSCRIPT_SYSTEM.md](./TRANSCRIPT_SYSTEM.md) | PDF extraction & caching |
| [LOCATORS.md](./LOCATORS.md) | Sheet ID detection strategies |
| [MODULES.md](./MODULES.md) | Module ecosystem overview |
| [GETTING_STARTED.md](./GETTING_STARTED.md) | This file |

---

## Next Steps

1. **Immediate**: Run a basic merge operation
   ```bash
   conset-pdf merge-addenda --original test.pdf --addenda test-add.pdf --output result.pdf --type drawings
   ```

2. **Short-term**: Create a layout profile for your document format
   ```bash
   conset-pdf detect --pdf sample.pdf --type drawings
   # Adjust --sheet-id-roi and --sheet-title-roi until confidence > 0.9
   ```

3. **Medium-term**: Integrate into your application
   ```typescript
   import { createMergeWorkflowRunner } from '@conset-pdf/core';
   // Build interactive workflow
   ```

4. **Long-term**: Understand full architecture
   - Read [Codebase Overview](./CODEBASE_OVERVIEW.md)
   - Study test cases in `tests/`
   - Contribute custom workflows or extensions

---

## Getting Help

- **API Questions**: See [Core API](./CORE_API.md)
- **CLI Questions**: See [CLI Reference](./CLI_REFERENCE.md)
- **Architecture Questions**: See [Codebase Overview](./CODEBASE_OVERVIEW.md)
- **Workflow Questions**: See [Workflow Engine](./WORKFLOWS.md)
- **Examples**: Review test files in `tests/`
- **Errors**: Check [Troubleshooting](#troubleshooting) above

---

**Ready to start?** Choose your path above or run:
```bash
npm install @conset-pdf/core
# Then follow Path 1, 2, 3, or 4 above
```

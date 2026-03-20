# Phase 06 - Workflow Breakdown: Extract Documents (Split)

**Document Type**: Post-Mortem Workflow Breakdown  
**Workflow**: Extract Documents (Split Set)  
**Implementation Status**: Complete — Engine + CLI + GUI  
**Date**: 2026-03-19

---

## Domain Purpose

Split a combined construction document PDF into organized subsets based on detected page identities. For drawings, group by discipline prefix (M, E, P, etc.) producing one PDF per discipline. For specs, group by division or section producing one PDF per division or per section. The workflow also produces a table of contents JSON and an audit trail report.

Supported document types: **drawings** and **specifications**.

---

## Workflow Engine Entry Points

| Phase | Method | Key Type |
|---|---|---|
| Analyze | `ExtractWorkflow.analyze(input)` | Returns `InventoryResult` |
| Corrections | `ExtractWorkflow.applyCorrections(input, corrections)` | Returns revised `InventoryResult` |
| Execute | `ExtractWorkflow.execute(input)` | Returns `ExecuteResult` |

Factory: `createExtractWorkflowRunner()` in `workflows/engine.ts`  
Implementation: `workflows/split/` (or equivalent split workflow module)  
CLI Command: `split-set`  
GUI: Split Drawings wizard (`conset-pdf-gui/src/split-drawings-wizard.js`)

---

## Analyze Phase

### Input

```typescript
{
  docType: 'drawings' | 'specs';
  inputPdfPath: string;
  profile?: LayoutProfile;         // Optional, for drawings ROI detection
  customRegexPattern?: string;     // Optional override for ID detection pattern
  options?: {
    verbose?: boolean;
  };
}
```

### Processing Steps

1. **DocumentContext initialization** for the input PDF
   - PyMuPDF sidecar spawned, transcript cached

2. **Identity detection** per page (same as merge workflow):
   - **Drawings**: `CompositeLocator` → `RoiSheetLocator` (profile) or `LegacyTitleblockLocator` (fallback)
   - **Specs**: `SpecsSectionLocator` text pattern matching

3. **Standards enrichment**:
   - Drawings: discipline classification adds sort metadata
   - Specs: MasterFormat division classification adds division grouping metadata

4. **Group analysis**:
   - For **drawings**: pages grouped by discipline prefix (e.g., `A` for Architectural, `M` for Mechanical, `E` for Electrical). Pages with no detected discipline are placed in an `Unknown` group.
   - For **specs**: pages grouped by division number (first two digits of section ID). Pages across the same section run consecutively; split points are at section ID transitions.
   - Page ranges per group computed and included in row metadata

5. **InventoryResult assembly**

### Output: InventoryResult

Rows include detected page identities and planned group assignments. Summary includes:
- Total pages
- Pages with detected IDs vs. unidentified
- Group count and distribution
- Issues: `NO_ID`, `LOW_CONFIDENCE`, `DUPLICATE`, `UNMATCHED`

---

## Grouping Logic

### Drawings Grouping

Drawings are grouped by the discipline designator — the alphabetic prefix(es) of the sheet ID:

| Sheet ID Example | Discipline | Group |
|---|---|---|
| A-101, A1-01 | Architectural | A |
| M-001, M1-01 | Mechanical | M |
| E-101 | Electrical | E |
| FP-101 | Fire Protection | FP |
| G-001 | General | G |

Sort order follows the canonical UDS discipline order from `standards/datasets/drawingsOrderHeuristic.ts`. Discipline-ordered grouping mirrors how a physical drawing set is organized.

Pages without a detectable discipline are placed in an unmatched trailing group. Pages before the first identified discipline (cover sheet, index, etc.) are kept as a prefix group.

### Specs Grouping

Specs are grouped by **division** (Division 00, 01, 02, ... 49). Each division group contains all section pages belonging to that division. Sections within a division are ordered by section number tuple (division, section, subsection).

**Legacy ID coexistence**: 5-digit legacy section IDs (`XXYYY`) are normalized to the same grouping model. Mixed modern/legacy books are handled by explicit normalization mode semantics (dual-format support from `parser/specsSectionId.ts`).

A page is assigned to the group of the last detected section header on or before it. Pages preceding the first section header are assigned to Division 00 (procurement/administrative).

---

## Multi-Division Specs Split (Section-Level)

When splitting at section granularity (vs. division granularity), each section becomes its own output file. This is the most granular split mode:

- One PDF per spec section (e.g., `23_05_00_Common_Work_Results.pdf`)
- Useful for sending individual sections to subconstractors or reviewers
- Output naming follows configurable filename format pattern

---

## Corrections Phase

User-reviewable before execute. Supported corrections:

- **Ignore Rows**: Exclude specific pages from split output (e.g., skip blank or duplicate pages)
- **Override IDs**: Correct detected sheet/section IDs to place pages in correct groups
- **Override Groups**: Manually assign pages to different groups (drawings only)

---

## Execute Phase

### Input

```typescript
{
  ...analyze inputs...
  outputDir: string;              // Directory for split output PDFs
  filenamePattern?: string;       // Format for output file names
  options?: {
    verbose?: boolean;
    reportPath?: string;
    inventoryOutputDir?: string;
  };
  corrections?: CorrectionOverlay;
}
```

### Processing

1. Apply correction overlay
2. For each group, collect pages in sorted order
3. Extract page subset from input PDF for each group (byte-exact page copy via pdf-lib)
4. Write group PDF to output directory:
   - Filename follows pattern: e.g., `A_Architectural.pdf`, `23_HVAC.pdf`
   - Custom pattern can include discipline code, discipline name, division, date
5. Write table of contents JSON (all groups, page ranges, output file paths)
6. Write audit trail report

### Execute Output: ExecuteResult

```
{
  outputs: { [groupKey: string]: string }  // Group identifier → output PDF path
  summary: {
    success: true;
    groupsExtracted: number;
    totalPagesExtracted: number;
    unmatchedPages: number;
    parseTimeMs: number;
    splitTimeMs: number;
  }
  warnings?: string[]
}
```

Additional outputs written to disk:
- `<outputDir>/toc.json` — table of contents (groups, page ranges, file paths)
- `<reportPath>` — audit trail (if requested)

---

## Output File Naming

Default patterns:
- **Drawings**: `{disciplineCode}_{disciplineName}.pdf` → e.g., `M_Mechanical.pdf`
- **Specs**: `{division0}_{division1}_{divisionTitle}.pdf` → e.g., `23_HVAC.pdf`

Custom pattern: configurable via `filenamePattern` argument; supports substitution tokens.

Output filenames are deterministic: same input + same profile + same corrections = same output file names.

---

## Section/Division Grouping Algorithm Detail

### Drawings: Discipline Sequence Preservation

The UDS discipline ordering table defines a canonical sequence (G, C, L, A, I, S, M, P, E, F, T, ...). Split output PDFs are ordered by this sequence, not by appearance order in input. This matches how full drawing sets are typically organized in project binders.

### Specs: Footer-Based Section Boundary Detection

For specs splitting, page-to-section assignment uses footer band text when available:
- Footer bands (bottom ~12% of page) are scanned for section ID patterns
- Lower-confidence fallback: heading band text
- Final fallback: `pageIndexHint` from prior detection

**Current limitation**: The footer section map builder (`specs/footerSectionMap.ts`) is a stub. Footer boundary detection in production specs split currently relies on heading-based detection. The footer-first approach is architecturally designed but not fully implemented. See R-002 and failure mode catalog.

---

## GUI Integration

Split Drawings wizard (`conset-pdf-gui/src/split-drawings-wizard.js`):

1. Select document type (drawings / specs)
2. Select input PDF and optional layout profile
3. Review inventory (grouped page table with discipline/division assignments)
4. Select output directory and filename format
5. Execute and review results

IPC channels:
- `split:analyze`
- `split:applyCorrections`
- `split:execute`

---

## Known Failure Modes

| Mode | Symptom | Root Cause |
|---|---|---|
| Cross-discipline pages | Page appears in wrong group | Sheet ID uses non-standard prefix not covered by UDS table |
| Undetected section boundaries | Section pages span across group boundaries | No visible section header; section begins mid-page |
| Legacy/modern ID coexistence | Groups have incorrect grouping | 5-digit IDs normalized differently than 6-digit siblings on same pages |
| Single-page sections | Empty or missing group output | Section detected but only cover page included with no body |

---

## Rust Implementation Notes

- Implement disk-streaming page extraction (never load entire multi-hundred-page PDF into memory)
- Port discipline ordering table from `standards/datasets/drawingsOrderHeuristic.ts` verbatim
- Port MasterFormat division table from `standards/datasets/masterformatDivisions.ts` verbatim
- Footer-first section anchoring should be implemented completely from day one (the TypeScript stub should not be replicated)
- The output filename pattern substitution engine should be deterministic and documented
- Table of contents JSON schema should be formally specified (it was ad hoc in the prototype)

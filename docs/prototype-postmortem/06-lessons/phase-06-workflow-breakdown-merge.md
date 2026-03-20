# Phase 06 - Workflow Breakdown: Update Documents (Merge)

**Document Type**: Post-Mortem Workflow Breakdown  
**Workflow**: Update Documents (Merge)  
**Implementation Status**: Complete — Engine + CLI + GUI  
**Date**: 2026-03-19

---

## Domain Purpose

Merge one or more addenda PDFs into an original construction document set. Updated sheets (same ID, new revision) replace their originals; new sheets (ID not present in original) are inserted in the correct discipline/division sequence. The output is a merged PDF containing the combined, up-to-date document set.

Supported document types: **drawings** and **specifications**.

---

## Workflow Engine Entry Points

| Phase | Method | Key Type |
|---|---|---|
| Analyze | `MergeWorkflow.analyze(input: MergeAnalyzeInput)` | Returns `InventoryResult` |
| Corrections | `MergeWorkflow.applyCorrections(input, corrections: CorrectionOverlay)` | Returns revised `InventoryResult` |
| Execute | `MergeWorkflow.execute(input: MergeExecuteInput)` | Returns `ExecuteResult` |

Factory: `createMergeWorkflowRunner()` in `workflows/engine.ts`  
Implementation: `workflows/merge/mergeWorkflow.ts`

---

## Analyze Phase

### Input

```typescript
MergeAnalyzeInput {
  docType: 'drawings' | 'specs';
  originalPdfPath: string;
  addendumPdfPaths: string[];          // Ordered: first addendum = earliest
  profile?: LayoutProfile;              // Required for ROI detection on drawings
  options?: {
    mode?: 'replace+insert' | 'replace-only' | 'append-only';
    strict?: boolean;
    verbose?: boolean;
    inventoryOutputDir?: string;
  };
  narrativePdfPath?: string;           // Optional advisory narrative PDF
}
```

### Processing Steps

1. **DocumentContext initialization** for original + each addendum PDF
   - Spawns PyMuPDF sidecar (`extract-transcript.py`) per document
   - Caches `LayoutTranscript` per document

2. **Identity detection** per page, per document:
   - **Drawings**: `CompositeLocator` → `RoiSheetLocator` (if profile provided) or `LegacyTitleblockLocator` (if `ENABLE_LEGACY_LOCATOR = true`)
   - **Specs**: `SpecsSectionLocator` — pure text pattern matching, no profile required
   - Drawing IDs normalized by `parser/drawingsSheetId.ts`
   - Spec IDs normalized by `parser/specsSectionId.ts`

3. **Standards enrichment** (via `workflows/mappers/merge.ts`):
   - Drawings rows: `normalizeDrawingsDiscipline()` → `DrawingsDisciplineMeta` (discipline code, canonical order, confidence)
   - Specs rows: `normalizeSpecsMasterformat()` → `SpecsMasterformatMeta` (division number, title, order)

4. **Merge plan construction** (`core/planner.ts`):
   - For each addendum page, match `normalizedId` against original inventory
   - **Replace**: addendum page ID exists in original → planned action = `'replace'`
   - **Insert**: addendum page ID is new → planned action = `'insert'`; insertion position determined by discipline/division sort order
   - **Append-only mode**: all addendum pages appended to end regardless of ID match
   - **Replace-only mode**: unmatched addendum pages not inserted
   - Duplicate ID handling: highest-confidence detection wins; lower-confidence entries flagged `'conflict'`
   - Cover pages (no detectable ID) are preserved as `'keep'`

5. **Narrative processing** (if `narrativePdfPath` provided):
   - `narrative/text-extract.ts`: extract page text from narrative PDF
   - `narrative/parse-algorithmic.ts`: parse text into `NarrativeInstructionSet` (drawing/spec revision instructions)
   - `narrative/validate.ts`: validate instructions against detected inventory
   - Produces `NarrativeValidationReport` with issue codes: `NARR_SHEET_NOT_FOUND`, `NARR_NEAR_MATCH`, `NARR_AMBIGUOUS_MATCH`, `NARR_INVENTORY_NOT_MENTIONED`
   - **Advisory only**: narrative results are appended to `InventoryResult.narrative` and `.narrativeValidation`; they never modify detected inventory

6. **InventoryResult assembly** with rows, issues, conflicts, summary, meta, and optional narrative

### Output: InventoryResult

```
{
  rows: InventoryRowBase[]        // One row per page across all input PDFs
  issues: Issue[]                 // NO_ID, LOW_CONFIDENCE, DUPLICATE, UNMATCHED, ROI_DETECTION_FAILURE, LEGACY_FALLBACK
  conflicts: Conflict[]           // (reserved; currently empty)
  summary: {
    totalRows, rowsWithIds, rowsWithoutIds,
    rowsOk, rowsWarning, rowsError,
    replaced, inserted, unmatched
  }
  meta: {
    docType, originalPdfPath, addendumPdfPaths, mode, strict
  }
  narrative?: NarrativeInstructionSet
  narrativeValidation?: NarrativeValidationReport
}
```

Row fields of note:
- `id`: Stable identifier — `${sourcePdfBasename}:${pageIndex}:${idPart}` — must survive corrections round-trip
- `normalizedId`: Detected/corrected sheet or section ID
- `action`: `'replace'` | `'insert'` | `'keep'` | `'append'`
- `source`: `'roi'` | `'legacy'` | `'text'`
- `confidence`: 0.0–1.0

---

## Drawings Detection Lane

### Primary Path: ROI-Based

`RoiSheetLocator` applies `LayoutProfile` ROI bounding boxes to extract text spans from specific page regions:

1. Load profile ROI boxes (already normalized: width/height-relative coordinates)
2. Filter `pageContext.textItems` to spans that fall within each ROI region
3. Reconstruct wrapped/multi-span text using `readingOrder.ts` (visual sort → concatenate)
4. Run `drawingsSheetId.ts` regex battery against assembled text
5. Return highest-confidence match with ROI region name as debug context

**Portrait vs. Landscape**: ROI coordinates use the rotation=0 normalized coordinate space from the transcript canonicalize step. Page rotation compensation is applied before any region filtering.

**Known edge case**: Very thin title blocks (< 2% page height) can cause ROI filters to exclude legitimate spans, yielding a false `NO_ID` for pages that have detectable IDs outside the expected title block zone.

### Fallback Path: Legacy Title Block

`LegacyTitleblockLocator` uses heuristic bounding box inference to locate title block text without a profile. Gated by `ENABLE_LEGACY_LOCATOR` in `featureFlags.ts`. Results are lower confidence and flagged in issues as `LEGACY_FALLBACK`.

---

## Specs Detection Lane

`SpecsSectionLocator` scans all text items from `pageContext` for patterns matching CSI MasterFormat section IDs:

- Modern: `XX YY ZZ` (two-digit division + four-digit section, space-separated)
- Legacy: `XXYYY` (five-digit, no separator)

Detection is purely additive: the first confident match per page wins. No profile required. For specs, detection accuracy is high because section IDs are printed in large type in headers and footers.

**Limitation**: Section detection works best on first-section pages; non-section cover pages, appendices, and cross-reference pages may produce `NO_ID`.

---

## Multi-Addendum Handling

Multiple addenda are provided as an ordered list. The planner processes them sequentially:

1. Each addendum is inventoried independently
2. Replace/insert logic is applied against the cumulative merged state after each addendum
3. A later addendum replacing a page that an earlier addendum already replaced supersedes it (last-wins semantics; highest confidence among same-ID entries wins if both have equal sequence position)

The current implementation does not explicitly model addendum revision sequences beyond ordering. The first-pass collision handling always selects the highest-confidence detection, not newest-addendum. This is a known simplification.

---

## Corrections Phase

User-reviewable before execute. Two supported correction types:

### Ignore Rows
```json
{ "ignoredRowIds": ["original-pdf:4:", "addendum1-pdf:10:A-999"] }
```
Pages with matching stable IDs are excluded from the merge output. They remain visible in the UI inventory but are marked inactive.

### Override IDs
```json
{
  "overrides": {
    "original-pdf:5:": { "fields": { "normalizedId": "A-102" } }
  }
}
```
Corrects a detected ID. Stable row `id` is unchanged; `normalizedId` is updated.

**Known limitation**: `applyCorrections()` currently re-runs `analyze()` internally and then overlays the corrections. It does not mutate prior plan state. This means any changes in the source PDFs between analyze and execute would be re-detected. See R-007 and SOT-005.

---

## Execute Phase

### Input
```typescript
MergeExecuteInput {
  ...same fields as MergeAnalyzeInput...
  outputPdfPath: string;
  options?: {
    reportPath?: string;
    regenerateBookmarks?: boolean;
    inventoryOutputDir?: string;
  };
  analyzed?: { plan?: MergePlan };  // Plan state from analyze (partially used)
  corrections?: CorrectionOverlay;
}
```

### Processing

1. Re-runs detection (or uses provided plan — partially wired)
2. Applies correction overlay
3. `core/applyPlan.ts` assembles merged PDF:
   - **Original pages** not being replaced are copied verbatim (byte-exact, via pdf-lib `copyPages`)
   - **Replaced pages** are swapped from the corresponding addendum source
   - **Inserted pages** are placed at the calculated insertion index
   - Page assembly is in-memory (pdf-lib PDFDocument)
4. `utils/pikepdfWriter.ts` writes final output through pikepdf sidecar passthrough
5. Optional: `core/report.ts` writes audit JSON to `reportPath`
6. Optional: if `regenerateBookmarks: true`, bookmarks workflow is invoked post-merge

### Execute Output: ExecuteResult

```
{
  outputs: { outputPdfPath: string; drawings?: string; specs?: string; }
  summary: {
    success: true;
    replaced, inserted, unmatched, finalPages,
    parseTimeMs, mergeTimeMs
  }
  warnings?: string[]
}
```

---

## Merge Modes

| Mode | Behavior |
|---|---|
| `replace+insert` (default) | Replace existing + insert new sheets in sequence |
| `replace-only` | Replace existing only; unmatched addendum pages are dropped |
| `append-only` | Append all addendum pages to end; no replacement |

---

## Byte-Verbatim Page Invariant

The planner marks every original page as either replaced (by addendum page) or kept. Kept pages in `applyPlan.ts` are passed through `pdf-lib`'s `copyPages()` which does not re-encode, re-compress, or modify PDF stream content. This is the prototype implementation of Non-Negotiable #12.

**Current limitation**: pikepdf passthrough writing may apply normalization passes. The byte-exact guarantee is strongest for in-memory copy of kept pages; the final pikepdf step may rewrite the container. True byte-exact streaming per-page copy requires disk-streaming merge (ADR-004 target architecture, not yet implemented).

---

## Known Failure Modes

| Mode | Symptom | Root Cause |
|---|---|---|
| ROI over-exclusion | `NO_ID` on pages with readable title blocks | ROI bounding box misses spans for atypical title block layouts |
| Low-confidence drawing ID | `LOW_CONFIDENCE` warning; incorrect ID detection | Non-standard sheet ID format not covered by parser battery |
| Blank ID | `NO_ID` + `UNMATCHED` | Sheet has no printed ID (cover, blank, general notes with no number) |
| Specs section span across pages | ID detected on wrong page | Spec section header spans or is repeated across multiple pages |
| Multi-addendum clobber | Earlier addendum page lost | Both addenda include same sheet; last-wins semantics apply |
| Large file OOM | Crash during page assembly | In-memory pdf-lib assembly exceeds available memory (>500MB input PDFs) |

---

## CLI Command

```
conset-pdf merge-addenda --original <path> --addenda <path1> <path2> \
  --output <path> --layout <profile.json> --report <report.json>
```

Options: `--mode`, `--strict`, `--narrative`, `--verbose`, `--inventory-dir`

See `docs/CLI.md` and Phase 1 CLI inventory for full option reference.

---

## GUI Integration

4-step Update Documents wizard in `conset-pdf-gui/src/merge-wizard.js`:

1. Select document type (drawings / specs)
2. Select original PDF and addendum PDFs
3. Review inventory (table with ID, action, status columns); apply corrections
4. Execute and review results

IPC channels:
- `merge:analyze` → `MergeWorkflow.analyze()`
- `merge:applyCorrections` → `MergeWorkflow.applyCorrections()`
- `merge:execute` → `MergeWorkflow.execute()`

---

## Rust Implementation Notes

- Implement disk-streaming merge from day one: never load all pages into memory simultaneously
- Byte-exact page copy: pass through PDF page streams without re-encoding
- Multi-addendum: explicitly model addendum revision ordering and last-addendum-wins semantics
- Narrative validation: implement as a separate advisory phase, never blocking or auto-applying
- ROI detection: port regex patterns and profile schema verbatim; add profile auto-selection wiring that was missing in the prototype
- Corrections: implement true cursor state (don't re-analyze; apply corrections to prior state)

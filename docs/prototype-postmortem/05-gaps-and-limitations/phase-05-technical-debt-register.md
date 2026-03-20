# Phase 05 - Technical Debt Register

## Scope

Step 32 captures active technical debt that should be intentionally resolved, replaced, or preserved with rationale during Rust migration.

## Debt Items

## TD-001 DocumentContext still keeps PDF.js document for bookmarks

- Evidence:
  - temporary bookmark-only PDF.js path in `conset-pdf/packages/core/src/analyze/documentContext.ts`
- Current effect:
  - mixed extraction stack (transcript + PDF.js) remains in document lifecycle.

## TD-002 pdf-lib bookmark writer remains in active merge path

- Evidence:
  - `PdfLibBookmarkWriter` import and use in `conset-pdf/packages/core/src/core/applyPlan.ts`
  - sidecar bookmark writer exists separately in `conset-pdf/packages/core/src/bookmarks/pikepdfBookmarkWriter.ts`
- Current effect:
  - superseded-vs-active writer boundary is not fully converged.

## TD-003 Legacy bookmark utility retained

- Evidence:
  - deprecation + removal TODO in `conset-pdf/packages/core/src/utils/bookmarks.ts`
  - legacy locator still imports helper in `conset-pdf/packages/core/src/locators/legacyTitleblockLocator.ts`
- Current effect:
  - deprecated path still participates in fallback behavior.

## TD-004 Specs patch rendering requires Playwright runtime

- Evidence:
  - Playwright dependency in `conset-pdf/packages/core/src/specs/render/pdfRenderer.ts`
  - workflow import/use in `conset-pdf/packages/core/src/workflows/specs-patch/specsPatchWorkflow.ts`
- Current effect:
  - large browser dependency and runtime complexity for specs regeneration.

## TD-005 Feature-flag discipline is narrow

- Evidence:
  - only explicit flag currently exposed is legacy locator toggle in `conset-pdf/packages/core/src/config/featureFlags.ts`
  - primary runtime use appears in `conset-pdf/packages/core/src/locators/compositeLocator.ts`
- Current effect:
  - feature-gate architecture exists but is sparsely exercised.

## TD-006 Merge analyzed-plan/corrections contract still mixed

- Evidence:
  - type notes in `conset-pdf/packages/core/src/workflows/merge/types.ts` still describe phase-1 limitations
  - execute path applies corrections but can re-analyze inventory in `conset-pdf/packages/core/src/workflows/merge/mergeWorkflow.ts`
- Current effect:
  - docs/types and runtime behavior are partially out of sync.

## TD-007 Legacy docs status drift (high impact)

- Evidence:
  - roadmap and GUI roadmap still contain stale placeholder/abandoned status statements compared with active code paths.
- Current effect:
  - migration teams can over/under-scope parity work if they trust markdown status labels over executable code.

## Prioritization for Rust

- P1:
  - TD-001, TD-002, TD-006, TD-007
- P2:
  - TD-003, TD-004
- P3:
  - TD-005

# Legacy Code

**Last verified**: 2026-03-01

**WARNING: Do not extend this code. It exists only for backward compatibility and fallback scenarios.**

## What Legacy Code Exists

### Legacy PDF Extraction Functions

**Location**: `src/utils/pdf.ts`

**Functions**:
- `getPageInfo(pdfPath, pageIndex)` - Used only in legacy path of `findSheetIdWithFullDetection`
- `extractBookmarks(pdfPath)` - Used only in legacy path of `findSheetIdWithFullDetection`
- `extractPageTextWithPositions(pdfPath, pageIndex)` - Used only in legacy paths
- `extractPageLines(pdfPath, pageIndex)` - Used only in legacy path of `autoDetectTitleBlock`
- `getPdfPageCount(pdfPath)` - Used as fallback in `report.ts` when DocumentContext not provided

**Status**: Marked as `LEGACY` with comments. Quarantined for fallback use only.

### Legacy Detection Functions

**Location**: `src/parser/drawingsSheetId.ts`

**Function**: `findSheetIdWithFullDetection(docContextOrPdfPath, pageIndex, ...)`

**Status**: Accepts either `DocumentContext` (preferred) or `pdfPath` string (legacy fallback). When `DocumentContext` is provided, uses cached data. When `pdfPath` is provided, falls back to direct PDF loading.

**Usage**: Only called by `LegacyTitleblockLocator` when `DocumentContext` is not available.

### Legacy Title Block Locator

**Location**: `src/locators/legacyTitleblockLocator.ts`

**Status**: Wraps legacy detection logic. Updated to accept `DocumentContext` for single-load compliance. Falls back to legacy path only when `DocumentContext` not available.

**Usage**: Used as fallback in `CompositeLocator` when ROI detection fails.

## Why Legacy Code Exists

1. **Backward Compatibility**: Some code paths may not have access to `DocumentContext` yet
2. **Fallback Scenarios**: When ROI detection fails, legacy detection provides a safety net
3. **Migration Path**: Legacy code is being phased out as more commands migrate to the new architecture

## Which Commands Use Legacy Code

### Active Commands (Use New Architecture)

- **`merge-addenda`**: Uses `DocumentContext` + `PageContext` + locators
- **`detect`**: Uses `DocumentContext` + `PageContext` + locators
- **`split-set`**: Uses `DocumentContext` + `PageContext` (refactored in Phase 3A)

### Legacy Fallback Paths

- **`LegacyTitleblockLocator`**: Falls back to legacy detection when `DocumentContext` not available
- **`report.ts`**: Falls back to `getPdfPageCount()` when `DocumentContext` not provided

## Boundaries

### Do Not Extend Legacy Code

**Rule**: Do not add new features to legacy functions. Instead:
1. Refactor to use `DocumentContext` + `PageContext`
2. Update callers to provide `DocumentContext`
3. Remove legacy fallback once all callers are updated

### Migration Path

If you need to use legacy functions in a new context:

1. Create a `DocumentContext` instance
2. Call `docContext.initialize()` once
3. Use `docContext.getPageContext(pageIndex)` to get cached `PageContext`
4. Use `PageContext` methods (`getText()`, `getTextItems()`, etc.) instead of direct extraction

## Deleted Functions

The following functions were deleted during refactoring:

- **`extractAllPagesText(pdfPath)`** - Deleted in Phase 3A
  - **Reason**: Replaced by `DocumentContext` + `PageContext` in `split-set` command
  - **Replacement**: Use `DocumentContext.extractTextForPages()` or iterate through `PageContext` instances

## Active Architecture Path

The active merge-addenda command path uses:
- `DocumentContext` for single-load PDF caching
- `PageContext` for per-page extraction caching
- Locators (`RoiSheetLocator`, `LegacyTitleblockLocator`, `SpecsSectionLocator`) that consume cached data

**No legacy functions are called in the active path.**

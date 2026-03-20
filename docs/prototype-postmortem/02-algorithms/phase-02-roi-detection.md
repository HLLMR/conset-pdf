# Phase 02 - ROI Detection

## Scope

Document profile-driven ROI sheet/spec detection, reading-order reconstruction, and fallback behavior.

## Source Evidence

- `packages/core/src/locators/roiSheetLocator.ts`
- `packages/core/src/locators/roiSpecsSectionLocator.ts`
- `packages/core/src/analyze/readingOrder.ts`
- `packages/core/src/locators/compositeLocator.ts`

## ROI Sheet Detection

`RoiSheetLocator` behavior:

- Iterates configured ROIs in profile order.
- Uses `PageContext.getTextItemsInROI()` with:
  - `intersectionMode: 'overlap'`
  - `overlapThreshold: 0.3`
  - small ROI padding (`DEFAULT_ID_PAD_NORM = 0.002`)
- Candidate generation steps:
  - test regex on single items first
  - if no match, attempt 2-item spatial merges (same line and near-below)
- Best candidate selected via scoring (length, suffix presence, structure, compactness, position, anchor proximity, token count).

### ROI failure codes

- `ROI_EMPTY`
- `ROI_LOW_TEXT_DENSITY`
- `ROI_NO_PATTERN_MATCH`

All ROI failures are accumulated into warnings/context for downstream diagnostics.

## ROI Specs Detection

`RoiSpecsSectionLocator` behavior:

- Strict anchor requirement:
  - `\bSECTION\s+(\d{2}\s+\d{2}\s+\d{2})\b`
- Candidate must be uppercase section header text.
- Reject likely inline references via pre-match heuristics (for example, `per section`, `according to`, trailing conjunctions).
- Title extraction only occurs if a section ID is positively detected.
- If title ROI resolves to `PART 1 - GENERAL` only, it expands upward and retries.

## Reading-Order Assembly

`readingOrder.ts` provides deterministic visual order:

- line grouping tolerance defaults to `0.6 * medianHeight`
- gap tolerance defaults to `0.15 * medianHeight`
- lines sorted top-to-bottom, spans left-to-right
- preserves short tokens and hyphenated fragments

Used by ROI locators through `assembleTextVisual()`.

## Fallback Chain

`CompositeLocator` behavior:

- ROI is attempted first.
- ROI result accepted immediately if ID exists and confidence >= `0.60`.
- If ROI returns low-confidence ID, ROI still wins (with warning).
- Legacy fallback is feature-flag controlled:
  - enabled only when `ENABLE_LEGACY_LOCATOR=true`
- If legacy fallback disabled and ROI fails, returns explicit warnings and no ID.

## Inputs and Outputs

- Inputs:
  - `PageContext`
  - layout profile ROIs and optional regex/anchor keywords
- Outputs:
  - ID, normalized ID, title (optional), confidence, method, warnings, context

## Invariants

- ROI order is deterministic.
- Returned method encodes which ROI matched.
- Warning payload includes per-ROI failure rationale.

## Failure Modes

- Misconfigured ROI boxes can fail with low-density/no-match signals.
- Inline section references in body text can be mistaken for headers without strict checks.
- Legacy fallback may be unavailable by policy flag, leaving ROI-only behavior.

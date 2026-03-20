# Phase 05 - Failure Modes Catalog

## Scope

Step 31 catalogs known failure modes and edge behaviors from tests, fixtures, and run artifacts.

## Catalog

## 1) Footer-first anchoring implementation gap

- Symptom:
  - footer validation artifacts show parsed footer IDs, but footer map output remains empty.
- Evidence:
  - parsed IDs present in `conset-pdf/test-output/23_MECH_FULL_fresh_run/04b-footer-validation.json`
  - `buildFooterSectionMap()` is currently a stub in `conset-pdf/packages/core/src/specs/footerSectionMap.ts`
- Impact:
  - deterministic section-boundary mapping cannot rely on full footer map structure yet.

## 2) Footer section parser is Division-23 specific

- Symptom:
  - parser only recognizes `23 XX XX` footer patterns.
- Evidence:
  - explicit pattern constraints in `conset-pdf/packages/core/src/specs/footerSectionIdParser.ts`
- Impact:
  - mixed-discipline spec books can under-detect section boundaries when non-23 sections dominate.

## 3) Schedule extraction fallback chain is incomplete

- Symptom:
  - geometry-first extraction exists, but python fallback implementations are TODO.
- Evidence:
  - TODO note in `conset-pdf/packages/core/src/transcript/schedules/extractor.ts`
- Impact:
  - complex tables (merged cells, rotated headers, sparse grid lines) may produce partial extraction only.

## 4) Narrative input failure soft-falls to no narrative

- Symptom:
  - missing narrative file does not fail analyze; narrative validation is omitted.
- Evidence:
  - behavior covered in `conset-pdf/tests/workflows/merge-narrative.test.ts`
- Impact:
  - merge proceeds without narrative safety checks if input path is missing/invalid.

## 5) Legacy specs ID coexistence requires compatibility mapping

- Symptom:
  - legacy 5-digit sections and modern IDs coexist; grouping behavior depends on mode.
- Evidence:
  - `conset-pdf/tests/workflows/split-legacy.test.ts`
- Impact:
  - migration must preserve dual-format normalization and explicit mode semantics.

## 6) Drawing ID detection confidence remains heuristic

- Symptom:
  - detection admits low-confidence IDs and rejects below-threshold candidates.
- Evidence:
  - thresholding and warning pathways in `conset-pdf/packages/core/src/parser/drawingsSheetId.ts`
- Impact:
  - blank/ambiguous/non-standard IDs still require correction overlay or manual review.

## 7) Bookmark destination correctness is regression-sensitive

- Symptom:
  - dedicated regression suites exist to prevent destination/page mismatch behavior.
- Evidence:
  - `conset-pdf/packages/core/src/bookmarks/tests/bookmarkPageMapping.test.ts`
  - `conset-pdf/packages/core/src/bookmarks/tests/bookmarkDestinations.test.ts`
  - `conset-pdf/packages/core/src/bookmarks/tests/bookmarkViewerCompatibility.test.ts`
- Impact:
  - destination writing and section-heading resolution remain high-risk areas for refactor drift.

## Fixture and Artifact Notes

- Primary run artifact reviewed: `conset-pdf/test-output/23_MECH_FULL_fresh_run/`
- Primary regression fixture families reviewed:
  - `conset-pdf/tests/workflows/`
  - `conset-pdf/packages/core/src/specs/tests/`
  - `conset-pdf/packages/core/src/bookmarks/tests/`

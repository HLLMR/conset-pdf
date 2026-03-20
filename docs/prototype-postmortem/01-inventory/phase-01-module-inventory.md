# Phase 01 - Core Module Inventory

## Scope

Phase 1, Step 1 from `docs/postMortemDocExtraction.md`: enumerate implemented modules in `packages/core/src/` and record implementation status.

## Method

- Source of truth priority followed `00-admin/source-of-truth.md`: code and tests first, then docs.
- Reviewed live module tree and file inventory under `packages/core/src/`.
- Cross-checked status claims against `README.md` and `ROADMAP.md`.
- Status labels used: Complete, Partial, Planned-but-not-started, Abandoned.

## Inventory Matrix

| Module | Representative Components | Impl File Count (ts/js/py) | Status | Notes |
|---|---|---:|---|---|
| analyze | `documentContext`, `pageContext`, `readingOrder` | 3 | Complete | Matches plan scope; active and used by detection workflows. |
| bookmarks | `reader`, `validator`, `treeBuilder`, `headingResolver`, `corrections`, pikepdf sidecar writer | 35 | Complete | Includes TS core + Python sidecar utilities; workflow and tests present. |
| config | `featureFlags` | 1 | Complete | Minimal but implemented and exported. |
| core | `mergeAddenda`, `splitSet`, `assembleSet`, `planner`, `applyPlan`, `report` | 6 | Partial | `mergeAddenda` path is active; `assembleSet` documented as abandoned/deprecated. |
| layout | profile `load`, `types` | 2 | Complete | Implemented and exported for ROI/profile workflows. |
| locators | `compositeLocator`, `roiSheetLocator`, `roiSpecsSectionLocator`, `sheetLocator`, `specsSectionLocator`, `legacyTitleblockLocator` | 6 | Complete | Full chain implemented; legacy locator is deprecated but still present. |
| narrative | `text-extract`, `parse-algorithmic`, `normalize`, `validate` | 6 | Complete | Algorithmic parser + validation implemented and integrated. |
| parser | `drawingsSheetId`, `specsSectionId`, `normalize` | 3 | Complete | Core ID parsing and normalization utilities implemented. |
| specs | extract pipeline, footer tools, inventory, patch/apply, patch/validator | 26 | Partial | Extraction and inventory are active; patch workflow is retained but marked deprecated/abandoned at roadmap level. |
| standards | `normalizeDrawingsDiscipline`, `normalizeSpecsMasterformat` (modern + v3), `registry`, datasets, `compare` | 14 | Complete | Dataset-backed normalization and sort comparators implemented. |
| submittals | `submittalParser`, submittal types | 2 | Complete | Implemented under transcript-era parser enhancements. |
| transcript | extractors, factory, canonicalize, quality, candidates, abstraction, ML compiler, profiles, schedules | 26 | Complete | Described as fully complete in roadmap; implementation footprint is substantial. |
| workflows | engine, merge, specs-patch, bookmarks, mappers | 16 | Partial | Engine and primary workflows active; specs-patch retained but deprecated/abandoned. |
| utils | bookmarks legacy helpers, bookmark writers, pdf/fs/sort | 9 | Complete | Active utility layer with some deprecated legacy helpers retained for compatibility. |

## Additional Observations

- Unlisted module in plan: `text/` exists and is implemented (4 files), supporting page-region and band slicing used by extraction/bookmark logic.
- `__tests__/` root exists as test scaffolding and is not counted as an implementation module.

## Status Summary (Step 1)

- Complete: 11
- Partial: 3
- Planned-but-not-started: 0
- Abandoned (module-level): 0

Submodule-level abandonment/deprecation exists inside otherwise implemented modules (`core/assembleSet`, specs-patch-related paths).

## Evidence Reviewed

- `packages/core/src/*` live directory and file inventory
- `packages/core/src/index.ts`
- `README.md`
- `ROADMAP.md`
- `docs/prototype-postmortem/00-admin/source-of-truth.md`

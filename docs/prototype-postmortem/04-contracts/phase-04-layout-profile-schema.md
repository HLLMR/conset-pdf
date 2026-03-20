# Phase 04 - Layout Profile Schema Contract

## Status

Drafted from executable source (`layout/types.ts`, `layout/load.ts`) and canonical template (`layouts/layout-template.json`).

## Contract Purpose

`LayoutProfile` is the user-facing wire contract for drawings ROI detection.

It defines where and how sheet ID/title extraction runs, using normalized ROI coordinates and optional validation constraints.

This contract is consumed by:

- CLI `--layout` and inline ROI pathways
- `RoiSheetLocator` drawing detection
- profile validation/load pipeline

## Canonical Type Definitions

### `NormalizedROI`

```ts
interface NormalizedROI {
  x: number;
  y: number;
  width: number;
  height: number;
}
```

Semantics:

- normalized coordinate space: each field in `[0, 1]`
- origin convention at contract boundary: bottom-left PDF-style normalized space
- interpreted as fractional region of page width/height

Constraints enforced by `validateROI()`:

- `0 <= x <= 1`
- `0 <= y <= 1`
- `0 < width <= 1`
- `0 < height <= 1`
- `x + width <= 1`
- `y + height <= 1`

### `LayoutProfile`

```ts
interface LayoutProfile {
  name: string;
  version: string;
  description?: string;
  page?: {
    orientation?: 'landscape' | 'portrait';
    roiSpace?: 'visual' | 'pdf';
    minWidth?: number;
    maxWidth?: number;
    minHeight?: number;
    maxHeight?: number;
  };
  sheetId: {
    rois: NormalizedROI[];
    regex?: string;
    anchorKeywords?: string[];
  };
  sheetTitle?: {
    rois: NormalizedROI[];
    maxLength?: number;
  };
  validation?: {
    requireInSheetList?: boolean;
    allowedPrefixes?: string[];
  };
  createdAt?: string;
  updatedAt?: string;
  source?: 'auto-detected' | 'manual' | 'user-defined';
}
```

## Required vs Optional Fields

Required:

- `name`
- `version`
- `sheetId`
- `sheetId.rois` (must be non-empty array)

Optional:

- `description`
- `page` and all subfields
- `sheetId.regex`
- `sheetId.anchorKeywords`
- `sheetTitle` and subfields
- `validation` and subfields
- metadata fields (`createdAt`, `updatedAt`, `source`)

## Loader Validation Contract (`loadLayoutProfile`)

Hard-fail errors:

- missing `name` or `version`
- missing/invalid `sheetId.rois`
- empty `sheetId.rois`
- any invalid ROI bounds in `sheetId.rois` or `sheetTitle.rois`

Non-fatal warnings:

- very small ROI area (`< 0.01`)
- very large ROI area (`> 0.5`)
- missing `anchorKeywords`
- very narrow prefix allow-list (`allowedPrefixes.length < 3`)

Defaults applied:

- ensure `profile.page` object exists
- default `profile.page.roiSpace = 'visual'` if omitted

## Coordinate and Conversion Rules

Contract-level ROI convention is normalized bottom-left.

Runtime text-item filtering in `PageContext.getTextItemsInROI()` converts ROI to absolute coordinates against cached visual-space text items (top-left item origin), using:

- `absX = roi.x * pageWidth`
- `absY = pageHeight * (1 - roi.y - roi.height)`
- `absWidth = roi.width * pageWidth`
- `absHeight = roi.height * pageHeight`

Implication:

- profile authors express ROIs in stable normalized terms
- execution handles origin reconciliation internally

## Fallback and Matching Semantics

`sheetId.rois` and optional `sheetTitle.rois` are ordered fallback arrays.

- order is significant
- extraction tries each ROI in sequence
- first acceptable match is used for ID/title path

This ordering is part of contract behavior and should remain deterministic.

## Versioning Guidance

`version` is a required string but currently unconstrained by semantic parser in loader.

Current template uses `"1.0.0"`.

Rust migration guidance:

- preserve string `version` as wire field
- add explicit semantic version policy in parser/validator if stricter compatibility handling is needed

## Canonical Template Snapshot

`layouts/layout-template.json` currently demonstrates:

- `page.orientation = "landscape"`
- `page.roiSpace = "visual"`
- single ROI for `sheetId` and `sheetTitle`
- `anchorKeywords` sample set
- `validation.allowedPrefixes` sample list
- `source = "user-defined"`

## Rust Porting Requirements

- keep JSON wire shape backward-compatible with current `LayoutProfile`
- keep ROI bound validation and non-empty `sheetId.rois` hard requirement
- preserve ordered ROI fallback semantics
- preserve normalized ROI coordinate contract
- preserve defaulting behavior for omitted `page.roiSpace`

## Evidence

- `packages/core/src/layout/types.ts`
- `packages/core/src/layout/load.ts`
- `packages/core/src/analyze/pageContext.ts`
- `layouts/layout-template.json`
- `docs/QUICK_START.md`